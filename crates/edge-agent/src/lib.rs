//! Edge agent library.
//!
//! Phase 1 currently implements the registration slice: the agent loads its
//! local configuration, reuses any persisted device identity, registers with
//! the control plane over HTTP, and stores the returned identity so a restart
//! keeps the same device id and auth token. Heartbeat and telemetry payloads
//! are still built locally (see [`EdgeAgent::startup_snapshot`]) ahead of the
//! transport slice that will send them over the wire.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use edgefleet_types::{
    DeviceRegistrationRequest, DeviceRegistrationResponse, DeviceStatus, HeartbeatRequest,
    TelemetryEvent,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

const DEFAULT_CONTROL_PLANE_URL: &str = "http://localhost:8080";
const DEFAULT_DEVICE_ID: &str = "edge-local-001";
const DEFAULT_DISPLAY_NAME: &str = "Local development agent";
const DEFAULT_CAPABILITIES: &[&str] = &["telemetry", "heartbeat"];
const DEFAULT_SAMPLE_PAYLOAD_JSON: &str =
    r#"{"temperature_c":41.2,"humidity":0.61,"fan_rpm":2380}"#;
const DEFAULT_STATE_PATH: &str = "edge-agent-state.json";

const CONTROL_PLANE_URL_ENV: &str = "EDGEFLEET_CONTROL_PLANE_URL";
const DEVICE_ID_ENV: &str = "EDGEFLEET_DEVICE_ID";
const DISPLAY_NAME_ENV: &str = "EDGEFLEET_DEVICE_DISPLAY_NAME";
const CAPABILITIES_ENV: &str = "EDGEFLEET_AGENT_CAPABILITIES";
const SAMPLE_PAYLOAD_ENV: &str = "EDGEFLEET_SAMPLE_PAYLOAD_JSON";
const STATE_PATH_ENV: &str = "EDGEFLEET_STATE_PATH";

/// Persisted device identity stored on the edge device between runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceIdentity {
    pub device_id: String,
    pub auth_token: String,
    pub accepted_at: DateTime<Utc>,
}

impl From<DeviceRegistrationResponse> for DeviceIdentity {
    fn from(response: DeviceRegistrationResponse) -> Self {
        Self {
            device_id: response.device_id,
            auth_token: response.auth_token,
            accepted_at: response.accepted_at,
        }
    }
}

/// Agent configuration, resolved from environment variables with local-friendly
/// defaults.
#[derive(Debug, Clone, PartialEq)]
pub struct AgentConfig {
    pub control_plane_url: String,
    pub device_id: String,
    pub display_name: Option<String>,
    pub capabilities: Vec<String>,
    pub sample_payload: Value,
    pub state_path: PathBuf,
}

impl AgentConfig {
    pub fn from_env() -> Result<Self, AgentError> {
        Self::from_vars(std::env::vars())
    }

    pub fn from_vars(vars: impl IntoIterator<Item = (String, String)>) -> Result<Self, AgentError> {
        let vars = vars.into_iter().collect::<HashMap<_, _>>();
        let control_plane_url =
            env_or_default(&vars, CONTROL_PLANE_URL_ENV, DEFAULT_CONTROL_PLANE_URL)?;
        let device_id = env_or_default(&vars, DEVICE_ID_ENV, DEFAULT_DEVICE_ID)?;
        let display_name =
            optional_env(&vars, DISPLAY_NAME_ENV).or_else(|| Some(DEFAULT_DISPLAY_NAME.to_owned()));
        let capabilities = parse_capabilities(vars.get(CAPABILITIES_ENV))?;
        let sample_payload = parse_sample_payload(vars.get(SAMPLE_PAYLOAD_ENV))?;
        let state_path = PathBuf::from(env_or_default(&vars, STATE_PATH_ENV, DEFAULT_STATE_PATH)?);

        Ok(Self {
            control_plane_url,
            device_id,
            display_name,
            capabilities,
            sample_payload,
            state_path,
        })
    }
}

/// Contract payloads built at startup for the current configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct StartupSnapshot {
    pub registration: DeviceRegistrationRequest,
    pub heartbeat: HeartbeatRequest,
    pub telemetry: TelemetryEvent,
}

/// The edge agent runtime.
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeAgent {
    pub config: AgentConfig,
}

impl EdgeAgent {
    pub fn new(config: AgentConfig) -> Self {
        Self { config }
    }

    pub fn startup_snapshot(&self, now: DateTime<Utc>) -> Result<StartupSnapshot, AgentError> {
        let registration = DeviceRegistrationRequest {
            device_id: self.config.device_id.clone(),
            display_name: self.config.display_name.clone(),
            agent_version: env!("CARGO_PKG_VERSION").to_owned(),
            capabilities: self.config.capabilities.clone(),
        };

        let heartbeat = HeartbeatRequest {
            device_id: self.config.device_id.clone(),
            timestamp: now,
            agent_version: env!("CARGO_PKG_VERSION").to_owned(),
            queue_depth: 0,
            status: DeviceStatus::Online,
        };

        let telemetry = TelemetryEvent::new(
            self.config.device_id.clone(),
            format!(
                "{}-startup-{}",
                self.config.device_id,
                now.timestamp_micros()
            ),
            now,
            "sensor.reading",
            self.config.sample_payload.clone(),
        );
        telemetry.validate()?;

        Ok(StartupSnapshot {
            registration,
            heartbeat,
            telemetry,
        })
    }
}

/// Register a device with the control plane.
///
/// Registration is idempotent on the control plane, so this is safe to call on
/// every startup even when a persisted identity already exists.
pub async fn register(
    client: &reqwest::Client,
    control_plane_url: &str,
    request: &DeviceRegistrationRequest,
) -> Result<DeviceRegistrationResponse, AgentError> {
    let url = format!(
        "{}/api/v1/devices/register",
        control_plane_url.trim_end_matches('/')
    );

    let response = client.post(&url).json(request).send().await?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(AgentError::RegistrationRejected {
            status: status.as_u16(),
            body,
        });
    }

    Ok(response.json::<DeviceRegistrationResponse>().await?)
}

/// Load a persisted device identity, if one exists for this device id.
///
/// A missing state file returns `Ok(None)`. A state file for a different
/// `device_id` is ignored (also `Ok(None)`) so re-pointing the agent at a new
/// device id does not reuse a stale token.
pub fn load_identity(path: &Path, device_id: &str) -> Result<Option<DeviceIdentity>, AgentError> {
    let contents = match std::fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(AgentError::StateIo {
                path: path.display().to_string(),
                source: error,
            });
        }
    };

    let identity: DeviceIdentity =
        serde_json::from_str(&contents).map_err(|source| AgentError::StateCorrupt {
            path: path.display().to_string(),
            source,
        })?;

    if identity.device_id == device_id {
        Ok(Some(identity))
    } else {
        Ok(None)
    }
}

/// Persist a device identity to disk.
pub fn save_identity(path: &Path, identity: &DeviceIdentity) -> Result<(), AgentError> {
    let serialized = serde_json::to_string_pretty(identity)?;
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|source| AgentError::StateIo {
            path: parent.display().to_string(),
            source,
        })?;
    }
    std::fs::write(path, serialized).map_err(|source| AgentError::StateIo {
        path: path.display().to_string(),
        source,
    })
}

/// Run the agent: resolve config, reuse any persisted identity, register with
/// the control plane, and persist the resulting identity.
pub async fn run() -> Result<(), AgentError> {
    let agent = EdgeAgent::new(AgentConfig::from_env()?);
    let snapshot = agent.startup_snapshot(Utc::now())?;
    let config = &agent.config;

    tracing::info!(control_plane_url = %config.control_plane_url, "edge-agent starting");

    match load_identity(&config.state_path, &config.device_id)? {
        Some(identity) => {
            tracing::info!(
                device_id = %identity.device_id,
                "reusing persisted device identity"
            );
        }
        None => {
            tracing::info!(device_id = %config.device_id, "no persisted identity; registering fresh");
        }
    }

    let client = reqwest::Client::new();
    let response = register(&client, &config.control_plane_url, &snapshot.registration).await?;
    let identity = DeviceIdentity::from(response);
    save_identity(&config.state_path, &identity)?;

    tracing::info!(
        device_id = %identity.device_id,
        state_path = %config.state_path.display(),
        "registered with control plane; identity persisted"
    );

    // Heartbeat and telemetry payloads are built but not yet transmitted; the
    // transport slice that sends them follows the registration slice.
    tracing::debug!(
        heartbeat = ?snapshot.heartbeat,
        telemetry_event_id = %snapshot.telemetry.event_id,
        "prepared heartbeat and telemetry payloads (transport slice pending)"
    );

    Ok(())
}

/// Errors raised while running the edge agent.
#[derive(Debug, Error)]
pub enum AgentError {
    #[error("{field} must not be empty")]
    EmptyConfig { field: &'static str },
    #[error("{field} must include at least one non-empty capability")]
    MissingCapabilities { field: &'static str },
    #[error("{field} must be a JSON object")]
    InvalidPayloadShape { field: &'static str },
    #[error("{field} must be valid JSON")]
    InvalidJson {
        field: &'static str,
        source: serde_json::Error,
    },
    #[error("control plane rejected registration (status {status}): {body}")]
    RegistrationRejected { status: u16, body: String },
    #[error("failed to reach control plane")]
    Http(#[from] reqwest::Error),
    #[error("failed to read or write device state at {path}")]
    StateIo {
        path: String,
        source: std::io::Error,
    },
    #[error("device state at {path} is corrupt")]
    StateCorrupt {
        path: String,
        source: serde_json::Error,
    },
    #[error("telemetry contract validation failed")]
    TelemetryContract(#[from] edgefleet_types::ContractError),
    #[error("json serialization failed")]
    Json(#[from] serde_json::Error),
}

fn env_or_default(
    vars: &HashMap<String, String>,
    field: &'static str,
    default: &str,
) -> Result<String, AgentError> {
    match optional_env(vars, field) {
        Some(value) => Ok(value),
        None => {
            if default.trim().is_empty() {
                Err(AgentError::EmptyConfig { field })
            } else {
                Ok(default.to_owned())
            }
        }
    }
}

fn optional_env(vars: &HashMap<String, String>, field: &'static str) -> Option<String> {
    vars.get(field)
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn parse_capabilities(value: Option<&String>) -> Result<Vec<String>, AgentError> {
    let capabilities = value
        .map(|value| value.as_str())
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|capability| !capability.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();

    if !capabilities.is_empty() {
        return Ok(capabilities);
    }

    if value.is_some() {
        return Err(AgentError::MissingCapabilities {
            field: CAPABILITIES_ENV,
        });
    }

    Ok(DEFAULT_CAPABILITIES
        .iter()
        .map(|capability| (*capability).to_owned())
        .collect())
}

fn parse_sample_payload(value: Option<&String>) -> Result<Value, AgentError> {
    let raw = value
        .map(|value| value.as_str())
        .unwrap_or(DEFAULT_SAMPLE_PAYLOAD_JSON);
    let payload = serde_json::from_str::<Value>(raw).map_err(|source| AgentError::InvalidJson {
        field: SAMPLE_PAYLOAD_ENV,
        source,
    })?;

    if !payload.is_object() {
        return Err(AgentError::InvalidPayloadShape {
            field: SAMPLE_PAYLOAD_ENV,
        });
    }

    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_uses_local_defaults() {
        let config = AgentConfig::from_vars([]).unwrap();

        assert_eq!(config.control_plane_url, DEFAULT_CONTROL_PLANE_URL);
        assert_eq!(config.device_id, DEFAULT_DEVICE_ID);
        assert_eq!(config.display_name.as_deref(), Some(DEFAULT_DISPLAY_NAME));
        assert_eq!(config.capabilities, ["telemetry", "heartbeat"]);
        assert_eq!(config.sample_payload["temperature_c"], 41.2);
        assert_eq!(config.sample_payload["humidity"], 0.61);
        assert_eq!(config.sample_payload["fan_rpm"], 2380);
        assert_eq!(config.state_path, PathBuf::from(DEFAULT_STATE_PATH));
    }

    #[test]
    fn config_parses_environment_overrides() {
        let config = AgentConfig::from_vars(env_pairs([
            (CONTROL_PLANE_URL_ENV, "http://control-plane:8080"),
            (DEVICE_ID_ENV, " edge-lab-007 "),
            (DISPLAY_NAME_ENV, " Lab freezer controller "),
            (CAPABILITIES_ENV, "telemetry, heartbeat, ota"),
            (STATE_PATH_ENV, " /var/lib/edgefleet/state.json "),
            (
                SAMPLE_PAYLOAD_ENV,
                r#"{"voltage_v":12.4,"door_open":false,"sample_count":7}"#,
            ),
        ]))
        .unwrap();

        assert_eq!(config.control_plane_url, "http://control-plane:8080");
        assert_eq!(config.device_id, "edge-lab-007");
        assert_eq!(
            config.display_name.as_deref(),
            Some("Lab freezer controller")
        );
        assert_eq!(config.capabilities, ["telemetry", "heartbeat", "ota"]);
        assert_eq!(
            config.state_path,
            PathBuf::from("/var/lib/edgefleet/state.json")
        );
        assert_eq!(config.sample_payload["voltage_v"], 12.4);
        assert_eq!(config.sample_payload["door_open"], false);
        assert_eq!(config.sample_payload["sample_count"], 7);
    }

    #[test]
    fn config_rejects_empty_capability_override() {
        let error = AgentConfig::from_vars(env_pairs([(CAPABILITIES_ENV, " , ")]))
            .expect_err("empty capability override should fail");

        assert!(matches!(
            error,
            AgentError::MissingCapabilities {
                field: CAPABILITIES_ENV
            }
        ));
    }

    #[test]
    fn config_rejects_non_object_payload_override() {
        let error = AgentConfig::from_vars(env_pairs([(SAMPLE_PAYLOAD_ENV, "[1, 2, 3]")]))
            .expect_err("non-object sample payload should fail");

        assert!(matches!(
            error,
            AgentError::InvalidPayloadShape {
                field: SAMPLE_PAYLOAD_ENV
            }
        ));
    }

    #[test]
    fn startup_snapshot_builds_phase_one_contract_payloads() {
        let now = DateTime::parse_from_rfc3339("2026-06-18T19:42:10Z")
            .unwrap()
            .with_timezone(&Utc);
        let agent = EdgeAgent::new(AgentConfig::from_vars([]).unwrap());

        let snapshot = agent.startup_snapshot(now).unwrap();

        assert_eq!(snapshot.registration.device_id, DEFAULT_DEVICE_ID);
        assert_eq!(
            snapshot.registration.capabilities,
            ["telemetry", "heartbeat"]
        );
        assert_eq!(snapshot.heartbeat.device_id, DEFAULT_DEVICE_ID);
        assert_eq!(snapshot.heartbeat.queue_depth, 0);
        assert_eq!(snapshot.heartbeat.status, DeviceStatus::Online);
        assert_eq!(snapshot.telemetry.device_id, DEFAULT_DEVICE_ID);
        assert_eq!(snapshot.telemetry.event_type, "sensor.reading");
        assert_eq!(
            snapshot.telemetry.event_id,
            "edge-local-001-startup-1781811730000000"
        );
        assert_eq!(snapshot.telemetry.payload, agent.config.sample_payload);
    }

    #[test]
    fn identity_round_trips_through_disk() {
        let path = temp_state_path("round-trip");
        let identity = DeviceIdentity {
            device_id: "edge-state-1".to_owned(),
            auth_token: "eftok_abc123".to_owned(),
            accepted_at: Utc::now(),
        };

        save_identity(&path, &identity).unwrap();
        let loaded = load_identity(&path, "edge-state-1").unwrap();

        assert_eq!(loaded.as_ref(), Some(&identity));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn missing_state_file_loads_as_none() {
        let path = temp_state_path("missing");
        let _ = std::fs::remove_file(&path);

        let loaded = load_identity(&path, "edge-state-1").unwrap();

        assert_eq!(loaded, None);
    }

    #[test]
    fn identity_for_other_device_is_ignored() {
        let path = temp_state_path("mismatch");
        let identity = DeviceIdentity {
            device_id: "edge-old".to_owned(),
            auth_token: "eftok_old".to_owned(),
            accepted_at: Utc::now(),
        };
        save_identity(&path, &identity).unwrap();

        let loaded = load_identity(&path, "edge-new").unwrap();

        assert_eq!(loaded, None);
        let _ = std::fs::remove_file(&path);
    }

    fn temp_state_path(label: &str) -> PathBuf {
        let unique = format!(
            "edgefleet-{label}-{}-{}.json",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );
        std::env::temp_dir().join(unique)
    }

    fn env_pairs<const N: usize>(pairs: [(&str, &str); N]) -> Vec<(String, String)> {
        pairs
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value.to_owned()))
            .collect()
    }
}
