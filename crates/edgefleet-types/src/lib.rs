use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

pub const CURRENT_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ContractError {
    #[error("{field} must not be empty")]
    EmptyField { field: &'static str },
    #[error("schema_version must be greater than zero")]
    InvalidSchemaVersion,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetryEvent {
    pub device_id: String,
    pub event_id: String,
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "type")]
    pub event_type: String,
    pub schema_version: u16,
    pub payload: Value,
}

impl TelemetryEvent {
    pub fn new(
        device_id: impl Into<String>,
        event_id: impl Into<String>,
        timestamp: DateTime<Utc>,
        event_type: impl Into<String>,
        payload: Value,
    ) -> Self {
        Self {
            device_id: device_id.into(),
            event_id: event_id.into(),
            timestamp,
            event_type: event_type.into(),
            schema_version: CURRENT_SCHEMA_VERSION,
            payload,
        }
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        require_non_empty("device_id", &self.device_id)?;
        require_non_empty("event_id", &self.event_id)?;
        require_non_empty("type", &self.event_type)?;

        if self.schema_version == 0 {
            return Err(ContractError::InvalidSchemaVersion);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceRegistrationRequest {
    pub device_id: String,
    pub display_name: Option<String>,
    pub agent_version: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceRegistrationResponse {
    pub device_id: String,
    pub auth_token: String,
    pub accepted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    pub device_id: String,
    pub timestamp: DateTime<Utc>,
    pub agent_version: String,
    pub queue_depth: u32,
    pub status: DeviceStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceStatus {
    Online,
    Degraded,
    Offline,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandRequest {
    pub command_id: String,
    pub device_id: String,
    pub command_type: String,
    pub payload: Value,
    pub issued_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandAck {
    pub command_id: String,
    pub device_id: String,
    pub status: CommandStatus,
    pub acknowledged_at: DateTime<Utc>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandStatus {
    Accepted,
    Running,
    Succeeded,
    Failed,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OtaMetadata {
    pub rollout_id: String,
    pub artifact_id: String,
    pub version: String,
    pub uri: String,
    pub sha256: String,
    pub signature: String,
    pub created_at: DateTime<Utc>,
}

fn require_non_empty(field: &'static str, value: &str) -> Result<(), ContractError> {
    if value.trim().is_empty() {
        return Err(ContractError::EmptyField { field });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn telemetry_event_serializes_contract_type_field() {
        let event = TelemetryEvent::new(
            "edge-042",
            "01JZ9X6N9VD4Y7Y3P2Z5F8K4QG",
            DateTime::parse_from_rfc3339("2026-06-18T19:42:10Z")
                .unwrap()
                .with_timezone(&Utc),
            "sensor.reading",
            json!({
                "temperature_c": 41.2,
                "humidity": 0.61,
                "fan_rpm": 2380
            }),
        );

        let serialized = serde_json::to_value(event).unwrap();

        assert_eq!(serialized["type"], "sensor.reading");
        assert_eq!(serialized["schema_version"], CURRENT_SCHEMA_VERSION);
        assert_eq!(serialized["payload"]["temperature_c"], 41.2);
    }

    #[test]
    fn telemetry_event_rejects_missing_idempotency_key() {
        let event = TelemetryEvent::new(
            "edge-042",
            " ",
            Utc::now(),
            "sensor.reading",
            json!({ "temperature_c": 41.2 }),
        );

        assert_eq!(
            event.validate(),
            Err(ContractError::EmptyField { field: "event_id" })
        );
    }

    #[test]
    fn device_status_uses_wire_friendly_names() {
        assert_eq!(
            serde_json::to_string(&DeviceStatus::Degraded).unwrap(),
            "\"degraded\""
        );
    }
}
