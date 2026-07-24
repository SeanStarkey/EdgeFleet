//! In-memory device registry.
//!
//! Registration is idempotent: re-registering a known `device_id` returns the
//! original auth token and acceptance time so an agent that restarts keeps a
//! stable identity. Mutable metadata (display name, agent version, declared
//! capabilities, and telemetry profile) is refreshed on every registration.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use edgefleet_types::{
    AuthToken, DeviceId, DeviceRegistrationRequest, DeviceRegistrationResponse, TelemetryProfile,
};

/// A persisted device record held by the control plane.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceRecord {
    pub device_id: DeviceId,
    pub display_name: Option<String>,
    pub agent_version: String,
    pub capabilities: Vec<String>,
    pub telemetry_profile: TelemetryProfile,
    pub auth_token: AuthToken,
    pub registered_at: DateTime<Utc>,
}

/// Errors that can occur while registering a device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistrationError {
    EmptyField { field: &'static str },
}

/// Thread-safe in-memory store of device records keyed by `device_id`.
#[derive(Debug, Clone, Default)]
pub struct DeviceRegistry {
    devices: Arc<Mutex<HashMap<DeviceId, DeviceRecord>>>,
}

impl DeviceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a device, returning its stable identity.
    ///
    /// The first registration for a `device_id` mints an auth token and records
    /// the acceptance time. Later registrations reuse both, making the call safe
    /// to retry during reconnect storms.
    pub fn register(
        &self,
        request: &DeviceRegistrationRequest,
        now: DateTime<Utc>,
    ) -> Result<DeviceRegistrationResponse, RegistrationError> {
        if request.agent_version.trim().is_empty() {
            return Err(RegistrationError::EmptyField {
                field: "agent_version",
            });
        }

        let mut devices = self.devices.lock().expect("registry mutex poisoned");
        let record = devices
            .entry(request.device_id.clone())
            .or_insert_with(|| DeviceRecord {
                device_id: request.device_id.clone(),
                display_name: request.display_name.clone(),
                agent_version: request.agent_version.clone(),
                capabilities: request.capabilities.clone(),
                telemetry_profile: request.telemetry_profile.clone(),
                auth_token: AuthToken::generate(),
                registered_at: now,
            });

        // Refresh mutable metadata while keeping identity (token, first-seen
        // time) stable across re-registrations.
        record.display_name = request.display_name.clone();
        record.agent_version = request.agent_version.clone();
        record.capabilities = request.capabilities.clone();
        record.telemetry_profile = request.telemetry_profile.clone();

        Ok(DeviceRegistrationResponse {
            device_id: record.device_id.clone(),
            auth_token: record.auth_token.clone(),
            accepted_at: record.registered_at,
        })
    }

    /// Number of registered devices.
    pub fn count(&self) -> usize {
        self.devices.lock().expect("registry mutex poisoned").len()
    }

    /// Fetch a snapshot of a device record by id.
    pub fn get(&self, device_id: &DeviceId) -> Option<DeviceRecord> {
        self.devices
            .lock()
            .expect("registry mutex poisoned")
            .get(device_id)
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use edgefleet_types::{
        CURRENT_SCHEMA_VERSION, TelemetryEventProfile, TelemetryFieldProfile, TelemetryValueType,
    };

    fn request(device_id: &str) -> DeviceRegistrationRequest {
        DeviceRegistrationRequest {
            device_id: DeviceId::new(device_id).unwrap(),
            display_name: Some("Test device".to_owned()),
            agent_version: "0.1.0".to_owned(),
            capabilities: vec!["telemetry".to_owned()],
            telemetry_profile: telemetry_profile("temperature_c", "Temperature", Some("C")),
        }
    }

    fn telemetry_profile(field_name: &str, label: &str, unit: Option<&str>) -> TelemetryProfile {
        TelemetryProfile {
            event_types: vec![TelemetryEventProfile {
                event_type: "sensor.reading".to_owned(),
                schema_version: CURRENT_SCHEMA_VERSION,
                payload_fields: vec![TelemetryFieldProfile {
                    name: field_name.to_owned(),
                    value_type: TelemetryValueType::Number,
                    label: Some(label.to_owned()),
                    unit: unit.map(str::to_owned),
                    display_order: 0,
                    display_hint: Some("gauge".to_owned()),
                }],
            }],
        }
    }

    #[test]
    fn first_registration_mints_identity() {
        let registry = DeviceRegistry::new();
        let now = Utc::now();

        let response = registry.register(&request("edge-1"), now).unwrap();

        assert_eq!(response.device_id.as_str(), "edge-1");
        assert!(response.auth_token.to_string().starts_with("eftok_"));
        assert_eq!(response.accepted_at, now);
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn re_registration_is_idempotent() {
        let registry = DeviceRegistry::new();
        let first = registry.register(&request("edge-1"), Utc::now()).unwrap();

        // Later registration at a different time with updated metadata.
        let mut second_request = request("edge-1");
        second_request.agent_version = "0.2.0".to_owned();
        second_request.capabilities = vec!["telemetry".to_owned(), "ota".to_owned()];
        second_request.telemetry_profile = telemetry_profile("voltage_v", "Voltage", Some("V"));
        let second = registry.register(&second_request, Utc::now()).unwrap();

        assert_eq!(first.auth_token, second.auth_token);
        assert_eq!(first.accepted_at, second.accepted_at);
        assert_eq!(registry.count(), 1);

        // Mutable metadata is refreshed; identity stays stable.
        let record = registry.get(&DeviceId::new("edge-1").unwrap()).unwrap();
        assert_eq!(record.agent_version, "0.2.0");
        assert_eq!(record.capabilities, ["telemetry", "ota"]);
        assert_eq!(
            record.telemetry_profile.event_types[0].payload_fields[0].name,
            "voltage_v"
        );
        assert_eq!(
            record.telemetry_profile.event_types[0].payload_fields[0]
                .unit
                .as_deref(),
            Some("V")
        );
    }

    #[test]
    fn distinct_devices_get_distinct_tokens() {
        let registry = DeviceRegistry::new();
        let a = registry.register(&request("edge-a"), Utc::now()).unwrap();
        let b = registry.register(&request("edge-b"), Utc::now()).unwrap();

        assert_ne!(a.auth_token, b.auth_token);
        assert_eq!(registry.count(), 2);
    }

    #[test]
    fn empty_agent_version_is_rejected() {
        let registry = DeviceRegistry::new();
        let mut req = request("edge-1");
        req.agent_version = "  ".to_owned();

        let error = registry
            .register(&req, Utc::now())
            .expect_err("blank agent version should be rejected");

        assert_eq!(
            error,
            RegistrationError::EmptyField {
                field: "agent_version"
            }
        );
    }
}
