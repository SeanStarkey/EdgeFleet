//! In-memory device registry.
//!
//! Registration is idempotent: re-registering a known `device_id` returns the
//! original auth token and acceptance time so an agent that restarts keeps a
//! stable identity. Mutable metadata (display name, agent version, declared
//! capabilities) is refreshed on every registration.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use edgefleet_types::{DeviceRegistrationRequest, DeviceRegistrationResponse};
use uuid::Uuid;

/// A persisted device record held by the control plane.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceRecord {
    pub device_id: String,
    pub display_name: Option<String>,
    pub agent_version: String,
    pub capabilities: Vec<String>,
    pub auth_token: String,
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
    devices: Arc<Mutex<HashMap<String, DeviceRecord>>>,
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
        if request.device_id.trim().is_empty() {
            return Err(RegistrationError::EmptyField { field: "device_id" });
        }
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
                auth_token: mint_token(),
                registered_at: now,
            });

        // Refresh mutable metadata while keeping identity (token, first-seen
        // time) stable across re-registrations.
        record.display_name = request.display_name.clone();
        record.agent_version = request.agent_version.clone();
        record.capabilities = request.capabilities.clone();

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
    pub fn get(&self, device_id: &str) -> Option<DeviceRecord> {
        self.devices
            .lock()
            .expect("registry mutex poisoned")
            .get(device_id)
            .cloned()
    }
}

fn mint_token() -> String {
    format!("eftok_{}", Uuid::new_v4().simple())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(device_id: &str) -> DeviceRegistrationRequest {
        DeviceRegistrationRequest {
            device_id: device_id.to_owned(),
            display_name: Some("Test device".to_owned()),
            agent_version: "0.1.0".to_owned(),
            capabilities: vec!["telemetry".to_owned()],
        }
    }

    #[test]
    fn first_registration_mints_identity() {
        let registry = DeviceRegistry::new();
        let now = Utc::now();

        let response = registry.register(&request("edge-1"), now).unwrap();

        assert_eq!(response.device_id, "edge-1");
        assert!(response.auth_token.starts_with("eftok_"));
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
        let second = registry.register(&second_request, Utc::now()).unwrap();

        assert_eq!(first.auth_token, second.auth_token);
        assert_eq!(first.accepted_at, second.accepted_at);
        assert_eq!(registry.count(), 1);

        // Mutable metadata is refreshed; identity stays stable.
        let record = registry.get("edge-1").unwrap();
        assert_eq!(record.agent_version, "0.2.0");
        assert_eq!(record.capabilities, ["telemetry", "ota"]);
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
    fn empty_device_id_is_rejected() {
        let registry = DeviceRegistry::new();
        let error = registry
            .register(&request("  "), Utc::now())
            .expect_err("blank device id should be rejected");

        assert_eq!(error, RegistrationError::EmptyField { field: "device_id" });
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
