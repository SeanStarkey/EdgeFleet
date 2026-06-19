use chrono::Utc;
use edgefleet_types::{DeviceStatus, HeartbeatRequest, TelemetryEvent};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let now = Utc::now();
    let telemetry = TelemetryEvent::new(
        "edge-local-001",
        "01JZ9X6N9VD4Y7Y3P2Z5F8K4QG",
        now,
        "sensor.reading",
        json!({
            "temperature_c": 41.2,
            "humidity": 0.61,
            "fan_rpm": 2380
        }),
    );
    telemetry.validate()?;

    let heartbeat = HeartbeatRequest {
        device_id: telemetry.device_id.clone(),
        timestamp: now,
        agent_version: env!("CARGO_PKG_VERSION").to_owned(),
        queue_depth: 0,
        status: DeviceStatus::Online,
    };

    println!("edge-agent scaffold ready");
    println!("{}", serde_json::to_string_pretty(&heartbeat)?);
    println!("{}", serde_json::to_string_pretty(&telemetry)?);

    Ok(())
}
