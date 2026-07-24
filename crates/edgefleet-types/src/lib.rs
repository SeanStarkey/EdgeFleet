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
#[serde(try_from = "UncheckedTelemetryEvent")]
pub struct TelemetryEvent {
    device_id: String,
    event_id: String,
    timestamp: DateTime<Utc>,
    #[serde(rename = "type")]
    event_type: String,
    schema_version: u16,
    payload: Value,
}

#[derive(Deserialize)]
struct UncheckedTelemetryEvent {
    device_id: String,
    event_id: String,
    timestamp: DateTime<Utc>,
    #[serde(rename = "type")]
    event_type: String,
    schema_version: u16,
    payload: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TelemetryProfile {
    pub event_types: Vec<TelemetryEventProfile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelemetryEventProfile {
    #[serde(rename = "type")]
    pub event_type: String,
    pub schema_version: u16,
    pub payload_fields: Vec<TelemetryFieldProfile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelemetryFieldProfile {
    pub name: String,
    pub value_type: TelemetryValueType,
    pub label: Option<String>,
    pub unit: Option<String>,
    pub display_order: u32,
    pub display_hint: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelemetryValueType {
    Null,
    Boolean,
    Number,
    String,
    Array,
    Object,
}

impl TelemetryEvent {
    pub fn new(
        device_id: impl Into<String>,
        event_id: impl Into<String>,
        timestamp: DateTime<Utc>,
        event_type: impl Into<String>,
        payload: Value,
    ) -> Result<Self, ContractError> {
        Self::try_from(UncheckedTelemetryEvent {
            device_id: device_id.into(),
            event_id: event_id.into(),
            timestamp,
            event_type: event_type.into(),
            schema_version: CURRENT_SCHEMA_VERSION,
            payload,
        })
    }

    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    pub fn event_id(&self) -> &str {
        &self.event_id
    }

    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }

    pub fn event_type(&self) -> &str {
        &self.event_type
    }

    pub fn schema_version(&self) -> u16 {
        self.schema_version
    }

    pub fn payload(&self) -> &Value {
        &self.payload
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

impl TryFrom<UncheckedTelemetryEvent> for TelemetryEvent {
    type Error = ContractError;

    fn try_from(unchecked: UncheckedTelemetryEvent) -> Result<Self, Self::Error> {
        let event = Self {
            device_id: unchecked.device_id,
            event_id: unchecked.event_id,
            timestamp: unchecked.timestamp,
            event_type: unchecked.event_type,
            schema_version: unchecked.schema_version,
            payload: unchecked.payload,
        };
        event.validate()?;

        Ok(event)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceRegistrationRequest {
    pub device_id: String,
    pub display_name: Option<String>,
    pub agent_version: String,
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub telemetry_profile: TelemetryProfile,
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
        )
        .unwrap();

        let serialized = serde_json::to_value(&event).unwrap();

        assert_eq!(serialized["type"], "sensor.reading");
        assert_eq!(serialized["schema_version"], CURRENT_SCHEMA_VERSION);
        assert_eq!(serialized["payload"]["temperature_c"], 41.2);

        let deserialized = serde_json::from_value::<TelemetryEvent>(serialized).unwrap();
        assert_eq!(deserialized, event);
    }

    #[test]
    fn registration_request_serializes_telemetry_profile_metadata() {
        let request = DeviceRegistrationRequest {
            device_id: "edge-042".to_owned(),
            display_name: Some("Lab freezer".to_owned()),
            agent_version: "0.1.0".to_owned(),
            capabilities: vec!["telemetry".to_owned()],
            telemetry_profile: TelemetryProfile {
                event_types: vec![TelemetryEventProfile {
                    event_type: "sensor.reading".to_owned(),
                    schema_version: CURRENT_SCHEMA_VERSION,
                    payload_fields: vec![TelemetryFieldProfile {
                        name: "temperature_c".to_owned(),
                        value_type: TelemetryValueType::Number,
                        label: Some("Temperature".to_owned()),
                        unit: Some("C".to_owned()),
                        display_order: 0,
                        display_hint: Some("gauge".to_owned()),
                    }],
                }],
            },
        };

        let serialized = serde_json::to_value(request).unwrap();

        assert_eq!(
            serialized["telemetry_profile"]["event_types"][0]["type"],
            "sensor.reading"
        );
        assert_eq!(
            serialized["telemetry_profile"]["event_types"][0]["payload_fields"][0]["name"],
            "temperature_c"
        );
        assert_eq!(
            serialized["telemetry_profile"]["event_types"][0]["payload_fields"][0]["unit"],
            "C"
        );
    }

    #[test]
    fn telemetry_event_rejects_missing_idempotency_key() {
        assert_eq!(
            TelemetryEvent::new(
                "edge-042",
                " ",
                Utc::now(),
                "sensor.reading",
                json!({ "temperature_c": 41.2 }),
            ),
            Err(ContractError::EmptyField { field: "event_id" })
        );
    }

    #[test]
    fn telemetry_event_deserialization_rejects_invalid_contract() {
        let error = serde_json::from_value::<TelemetryEvent>(json!({
            "device_id": "edge-042",
            "event_id": "event-042",
            "timestamp": "2026-06-18T19:42:10Z",
            "type": " ",
            "schema_version": CURRENT_SCHEMA_VERSION,
            "payload": { "temperature_c": 41.2 }
        }))
        .unwrap_err();

        assert!(error.to_string().contains("type must not be empty"));
    }

    #[test]
    fn device_status_uses_wire_friendly_names() {
        assert_eq!(
            serde_json::to_string(&DeviceStatus::Degraded).unwrap(),
            "\"degraded\""
        );
    }
}
