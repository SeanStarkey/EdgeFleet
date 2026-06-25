use std::collections::HashMap;

use chrono::{DateTime, Utc};
use edgefleet_types::{DeviceRegistrationRequest, DeviceStatus, HeartbeatRequest, TelemetryEvent};
use serde_json::Value;
use thiserror::Error;

const DEFAULT_CONTROL_PLANE_URL: &str = "http://localhost:8080";
const DEFAULT_DEVICE_ID: &str = "edge-local-001";
const DEFAULT_DISPLAY_NAME: &str = "Local development agent";
const DEFAULT_CAPABILITIES: &[&str] = &["telemetry", "heartbeat"];
const DEFAULT_SAMPLE_PAYLOAD_JSON: &str =
    r#"{"temperature_c":41.2,"humidity":0.61,"fan_rpm":2380}"#;

const CONTROL_PLANE_URL_ENV: &str = "EDGEFLEET_CONTROL_PLANE_URL";
const DEVICE_ID_ENV: &str = "EDGEFLEET_DEVICE_ID";
const DISPLAY_NAME_ENV: &str = "EDGEFLEET_DEVICE_DISPLAY_NAME";
const CAPABILITIES_ENV: &str = "EDGEFLEET_AGENT_CAPABILITIES";
const SAMPLE_PAYLOAD_ENV: &str = "EDGEFLEET_SAMPLE_PAYLOAD_JSON";

fn main() -> Result<(), AgentError> {
    let agent = EdgeAgent::new(AgentConfig::from_env()?);
    let startup = agent.startup_snapshot(Utc::now())?;

    println!("edge-agent base executable ready");
    println!("control_plane_url={}", agent.config.control_plane_url);
    println!("network_transport=not_enabled_yet");
    println!("registration:");
    println!("{}", serde_json::to_string_pretty(&startup.registration)?);
    println!("heartbeat:");
    println!("{}", serde_json::to_string_pretty(&startup.heartbeat)?);
    println!("telemetry:");
    println!("{}", serde_json::to_string_pretty(&startup.telemetry)?);

    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
struct AgentConfig {
    control_plane_url: String,
    device_id: String,
    display_name: Option<String>,
    capabilities: Vec<String>,
    sample_payload: Value,
}

impl AgentConfig {
    fn from_env() -> Result<Self, AgentError> {
        Self::from_vars(std::env::vars())
    }

    fn from_vars(vars: impl IntoIterator<Item = (String, String)>) -> Result<Self, AgentError> {
        let vars = vars.into_iter().collect::<HashMap<_, _>>();
        let control_plane_url =
            env_or_default(&vars, CONTROL_PLANE_URL_ENV, DEFAULT_CONTROL_PLANE_URL)?;
        let device_id = env_or_default(&vars, DEVICE_ID_ENV, DEFAULT_DEVICE_ID)?;
        let display_name =
            optional_env(&vars, DISPLAY_NAME_ENV).or_else(|| Some(DEFAULT_DISPLAY_NAME.to_owned()));
        let capabilities = parse_capabilities(vars.get(CAPABILITIES_ENV))?;
        let sample_payload = parse_sample_payload(vars.get(SAMPLE_PAYLOAD_ENV))?;

        Ok(Self {
            control_plane_url,
            device_id,
            display_name,
            capabilities,
            sample_payload,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
struct StartupSnapshot {
    registration: DeviceRegistrationRequest,
    heartbeat: HeartbeatRequest,
    telemetry: TelemetryEvent,
}

#[derive(Debug, Clone, PartialEq)]
struct EdgeAgent {
    config: AgentConfig,
}

impl EdgeAgent {
    fn new(config: AgentConfig) -> Self {
        Self { config }
    }

    fn startup_snapshot(&self, now: DateTime<Utc>) -> Result<StartupSnapshot, AgentError> {
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

#[derive(Debug, Error)]
enum AgentError {
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
    }

    #[test]
    fn config_parses_environment_overrides() {
        let config = AgentConfig::from_vars(env_pairs([
            (CONTROL_PLANE_URL_ENV, "http://control-plane:8080"),
            (DEVICE_ID_ENV, " edge-lab-007 "),
            (DISPLAY_NAME_ENV, " Lab freezer controller "),
            (CAPABILITIES_ENV, "telemetry, heartbeat, ota"),
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

    fn env_pairs<const N: usize>(pairs: [(&str, &str); N]) -> Vec<(String, String)> {
        pairs
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value.to_owned()))
            .collect()
    }
}
