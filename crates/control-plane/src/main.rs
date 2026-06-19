use chrono::Utc;
use edgefleet_types::{DeviceRegistrationRequest, DeviceRegistrationResponse};

const ROUTES: &[&str] = &[
    "GET /healthz",
    "GET /readyz",
    "POST /api/v1/devices/register",
    "POST /api/v1/telemetry",
    "POST /api/v1/heartbeats",
    "GET /api/v1/commands",
    "POST /api/v1/commands/{command_id}/ack",
    "GET /api/v1/ota/metadata",
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registration = DeviceRegistrationRequest {
        device_id: "edge-local-001".to_owned(),
        display_name: Some("Local development agent".to_owned()),
        agent_version: env!("CARGO_PKG_VERSION").to_owned(),
        capabilities: vec!["telemetry".to_owned(), "heartbeat".to_owned()],
    };

    let response = DeviceRegistrationResponse {
        device_id: registration.device_id.clone(),
        auth_token: "development-token-placeholder".to_owned(),
        accepted_at: Utc::now(),
    };

    println!("control-plane scaffold routes:");
    for route in ROUTES {
        println!("- {route}");
    }
    println!("{}", serde_json::to_string_pretty(&registration)?);
    println!("{}", serde_json::to_string_pretty(&response)?);

    Ok(())
}
