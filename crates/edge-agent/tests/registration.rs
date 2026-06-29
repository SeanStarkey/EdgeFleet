//! End-to-end registration test: boot the real control-plane application and
//! have the edge agent register against it over HTTP.

use std::path::PathBuf;

use chrono::Utc;
use edge_agent::{DeviceIdentity, load_identity, register, save_identity};
use edgefleet_types::DeviceRegistrationRequest;

async fn start_control_plane() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind ephemeral port");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move {
        control_plane::serve(listener).await.expect("serve");
    });
    format!("http://{addr}")
}

fn registration(device_id: &str) -> DeviceRegistrationRequest {
    DeviceRegistrationRequest {
        device_id: device_id.to_owned(),
        display_name: Some("Integration test device".to_owned()),
        agent_version: "0.1.0".to_owned(),
        capabilities: vec!["telemetry".to_owned(), "heartbeat".to_owned()],
    }
}

#[tokio::test]
async fn agent_registers_and_reregistration_is_idempotent() {
    let base_url = start_control_plane().await;
    let client = reqwest::Client::new();
    let request = registration("edge-int-001");

    let first = register(&client, &base_url, &request)
        .await
        .expect("first registration succeeds");
    let second = register(&client, &base_url, &request)
        .await
        .expect("second registration succeeds");

    assert_eq!(first.device_id, "edge-int-001");
    assert!(first.auth_token.starts_with("eftok_"));
    // A restart / retry keeps the same identity.
    assert_eq!(first.auth_token, second.auth_token);
    assert_eq!(first.accepted_at, second.accepted_at);
}

#[tokio::test]
async fn agent_persists_and_reuses_identity_across_restarts() {
    let base_url = start_control_plane().await;
    let client = reqwest::Client::new();
    let request = registration("edge-int-002");
    let state_path = temp_state_path();

    // First "boot": no persisted identity, register and persist.
    assert_eq!(load_identity(&state_path, "edge-int-002").unwrap(), None);
    let response = register(&client, &base_url, &request)
        .await
        .expect("registration succeeds");
    let identity = DeviceIdentity::from(response);
    save_identity(&state_path, &identity).unwrap();

    // Second "boot": the persisted identity is loaded and matches what the
    // control plane returns again.
    let reused = load_identity(&state_path, "edge-int-002")
        .unwrap()
        .expect("identity persisted");
    assert_eq!(reused, identity);

    let response_again = register(&client, &base_url, &request)
        .await
        .expect("re-registration succeeds");
    assert_eq!(response_again.auth_token, reused.auth_token);

    let _ = std::fs::remove_file(&state_path);
}

fn temp_state_path() -> PathBuf {
    let unique = format!(
        "edgefleet-int-{}-{}.json",
        std::process::id(),
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    );
    std::env::temp_dir().join(unique)
}
