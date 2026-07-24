use chrono::Utc;
use edgefleet_types::{DeviceId, EventId, TelemetryEvent};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let events = (1..=3)
        .map(|index| -> Result<_, Box<dyn std::error::Error>> {
            Ok(TelemetryEvent::new(
                DeviceId::new(format!("edge-sim-{index:03}"))?,
                EventId::new(),
                Utc::now(),
                "sensor.reading",
                json!({
                    "temperature_c": 38.0 + f64::from(index),
                    "humidity": 0.55,
                    "fan_rpm": 2100 + (index * 25)
                }),
            )?)
        })
        .collect::<Result<Vec<_>, _>>()?;

    println!("fleet-simulator scaffold generated {} events", events.len());
    println!("{}", serde_json::to_string_pretty(&events)?);

    Ok(())
}
