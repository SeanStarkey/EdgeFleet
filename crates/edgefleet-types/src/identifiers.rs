use std::borrow::Borrow;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};
use thiserror::Error;
use uuid::{Uuid, Version};

const MAX_DEVICE_ID_LENGTH: usize = 128;
const AUTH_TOKEN_PREFIX: &str = "eftok_";

/// Validation failures for shared fleet identifiers.
#[derive(Debug, Error)]
pub enum IdentifierError {
    #[error("device_id must not be empty")]
    EmptyDeviceId,
    #[error("device_id must not exceed {MAX_DEVICE_ID_LENGTH} characters")]
    DeviceIdTooLong,
    #[error(
        "device_id may contain only ASCII letters, digits, hyphens, underscores, periods, and colons"
    )]
    InvalidDeviceIdCharacter,
    #[error("event_id must be a valid UUID")]
    InvalidEventId(#[source] uuid::Error),
    #[error("event_id must be a UUIDv7")]
    InvalidEventIdVersion,
    #[error("auth_token must use the eftok_<UUIDv4> format")]
    InvalidAuthToken,
}

/// Stable device identifier used across agent and control-plane contracts.
///
/// Device ids contain at most 128 ASCII letters, digits, hyphens, underscores,
/// periods, or colons.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct DeviceId(String);

impl DeviceId {
    /// Validate and construct a device identifier.
    pub fn new(value: impl Into<String>) -> Result<Self, IdentifierError> {
        let value = value.into();

        if value.is_empty() {
            return Err(IdentifierError::EmptyDeviceId);
        }
        if value.chars().count() > MAX_DEVICE_ID_LENGTH {
            return Err(IdentifierError::DeviceIdTooLong);
        }
        if !value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "-_.:".contains(character))
        {
            return Err(IdentifierError::InvalidDeviceIdCharacter);
        }

        Ok(Self(value))
    }

    /// Borrow the identifier's wire representation.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DeviceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for DeviceId {
    type Err = IdentifierError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl TryFrom<String> for DeviceId {
    type Error = IdentifierError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for DeviceId {
    type Error = IdentifierError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl AsRef<str> for DeviceId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for DeviceId {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<'de> Deserialize<'de> for DeviceId {
    fn deserialize<DeserializerType>(
        deserializer: DeserializerType,
    ) -> Result<Self, DeserializerType::Error>
    where
        DeserializerType: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(DeserializerType::Error::custom)
    }
}

/// Sortable telemetry id backed by a UUIDv7.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventId(Uuid);

impl EventId {
    /// Generate a UUIDv7 from the current time and process-local sequence.
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Borrow the underlying UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for EventId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.hyphenated().fmt(formatter)
    }
}

impl FromStr for EventId {
    type Err = IdentifierError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let uuid = Uuid::parse_str(value).map_err(IdentifierError::InvalidEventId)?;
        if uuid.get_version() != Some(Version::SortRand) {
            return Err(IdentifierError::InvalidEventIdVersion);
        }

        Ok(Self(uuid))
    }
}

impl Serialize for EventId {
    fn serialize<SerializerType>(
        &self,
        serializer: SerializerType,
    ) -> Result<SerializerType::Ok, SerializerType::Error>
    where
        SerializerType: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for EventId {
    fn deserialize<DeserializerType>(
        deserializer: DeserializerType,
    ) -> Result<Self, DeserializerType::Error>
    where
        DeserializerType: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_str(&value).map_err(DeserializerType::Error::custom)
    }
}

/// Authentication credential serialized as `eftok_<UUIDv4>`.
///
/// Its `Debug` representation is always redacted.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct AuthToken(Uuid);

impl AuthToken {
    /// Generate a new random authentication token.
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }

    /// Borrow the UUID portion of the credential.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl fmt::Debug for AuthToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("AuthToken([REDACTED])")
    }
}

impl fmt::Display for AuthToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{AUTH_TOKEN_PREFIX}{}", self.0.simple())
    }
}

impl FromStr for AuthToken {
    type Err = IdentifierError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let uuid = value
            .strip_prefix(AUTH_TOKEN_PREFIX)
            .and_then(|value| Uuid::parse_str(value).ok())
            .filter(|uuid| uuid.get_version() == Some(Version::Random))
            .ok_or(IdentifierError::InvalidAuthToken)?;

        Ok(Self(uuid))
    }
}

impl Serialize for AuthToken {
    fn serialize<SerializerType>(
        &self,
        serializer: SerializerType,
    ) -> Result<SerializerType::Ok, SerializerType::Error>
    where
        SerializerType: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for AuthToken {
    fn deserialize<DeserializerType>(
        deserializer: DeserializerType,
    ) -> Result<Self, DeserializerType::Error>
    where
        DeserializerType: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_str(&value).map_err(DeserializerType::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_id_rejects_invalid_values() {
        assert!(matches!(
            DeviceId::new(""),
            Err(IdentifierError::EmptyDeviceId)
        ));
        assert!(matches!(
            DeviceId::new("edge device"),
            Err(IdentifierError::InvalidDeviceIdCharacter)
        ));
        assert!(matches!(
            DeviceId::new("x".repeat(MAX_DEVICE_ID_LENGTH + 1)),
            Err(IdentifierError::DeviceIdTooLong)
        ));
    }

    #[test]
    fn device_id_round_trips_as_a_json_string() {
        let device_id = DeviceId::new("plant-1:edge_042").unwrap();
        let serialized = serde_json::to_string(&device_id).unwrap();

        assert_eq!(serialized, "\"plant-1:edge_042\"");
        assert_eq!(
            serde_json::from_str::<DeviceId>(&serialized).unwrap(),
            device_id
        );
    }

    #[test]
    fn event_id_is_uuid_v7_and_round_trips_as_a_json_string() {
        let first = EventId::new();
        let second = EventId::new();
        let serialized = serde_json::to_string(&first).unwrap();

        assert_eq!(first.as_uuid().get_version(), Some(Version::SortRand));
        assert!(first < second);
        assert_eq!(serde_json::from_str::<EventId>(&serialized).unwrap(), first);
    }

    #[test]
    fn event_id_rejects_other_uuid_versions() {
        let uuid_v4 = Uuid::new_v4().to_string();

        assert!(matches!(
            EventId::from_str(&uuid_v4),
            Err(IdentifierError::InvalidEventIdVersion)
        ));
    }

    #[test]
    fn auth_token_validates_round_trips_and_redacts_debug_output() {
        let token = AuthToken::generate();
        let serialized = serde_json::to_string(&token).unwrap();

        assert!(token.to_string().starts_with(AUTH_TOKEN_PREFIX));
        assert_eq!(
            serde_json::from_str::<AuthToken>(&serialized).unwrap(),
            token
        );
        assert_eq!(format!("{token:?}"), "AuthToken([REDACTED])");
        assert!(!format!("{token:?}").contains(&token.as_uuid().to_string()));
    }

    #[test]
    fn auth_token_rejects_invalid_values() {
        assert!(matches!(
            AuthToken::from_str("not-a-token"),
            Err(IdentifierError::InvalidAuthToken)
        ));
        assert!(matches!(
            AuthToken::from_str(&format!("{AUTH_TOKEN_PREFIX}{}", Uuid::now_v7())),
            Err(IdentifierError::InvalidAuthToken)
        ));
    }
}
