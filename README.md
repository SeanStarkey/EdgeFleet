# EdgeFleet

EdgeFleet is a distributed edge telemetry and fleet orchestration platform
written in Rust.

It is designed for edge devices that operate in unreliable networks: gateways,
sensors, kiosks, industrial equipment, remote monitoring systems, and other
small fleets that need to keep working when connectivity gets messy.

The goal is a lightweight, self-hostable operations stack that can:

- Register and authenticate edge devices.
- Collect structured telemetry.
- Buffer telemetry locally during outages.
- Replay data safely after reconnect.
- Show fleet health in a dashboard.
- Send remote commands to devices.
- Roll out signed OTA updates with canaries and rollback.
- Expose logs, metrics, traces, and health checks for operators.

EdgeFleet is not meant to be a generic replacement for AWS IoT or other managed
cloud IoT platforms. It is a focused fleet operations project: the interesting
part is the workflow around devices that disconnect, recover, need updates, and
must remain observable.

## Why This Exists

EdgeFleet is a portfolio-grade systems project for demonstrating modern Rust
and distributed systems engineering:

- Async Rust services and agents.
- Offline-first edge communication.
- Event-driven architecture.
- Durable local queues.
- Idempotent telemetry replay.
- Observability and reliability engineering.
- OTA deployment workflows.
- Practical local development and self-hosted deployment.

The project is intentionally scoped around a complete, understandable demo
rather than cloud-scale ingestion. A reviewer should eventually be able to run a
fleet simulation locally, break connectivity, watch agents recover, send
commands, and perform an OTA rollout from the dashboard.

## Architecture

EdgeFleet is planned as four main components.

### Edge Agent

A lightweight Rust agent that runs on edge devices.

Responsibilities:

- Device registration and authentication.
- Telemetry collection.
- Heartbeats and health reporting.
- Local SQLite buffering during outages.
- Automatic reconnect with backoff.
- Idempotent replay after reconnect.
- Remote command handling.
- Signed OTA update verification.

Planned technologies: Tokio, serde, tracing, rustls, SQLite, and an HTTP client
stack suitable for Axum-based services.

### Control Plane API

The central service for fleet state and orchestration.

Responsibilities:

- Device registry.
- Telemetry ingestion.
- Heartbeat tracking.
- Fleet status.
- Command dispatch.
- OTA rollout orchestration.
- Dashboard event streams.
- Health, readiness, metrics, and tracing endpoints.

Planned technologies: Axum, PostgreSQL, NATS, OpenTelemetry, and tracing.

### Messaging Layer

NATS is planned as the event backbone for telemetry fanout, device state
changes, and command delivery.

### Dashboard

A React and Tailwind dashboard for operators.

Planned views:

- Fleet overview.
- Device details.
- Live telemetry.
- Queue and replay status.
- Remote command history.
- OTA rollout progress.
- System health and observability links.

## Telemetry Model

Telemetry uses a stable envelope with an open-ended JSON payload. The platform
keeps operational fields typed while allowing each device type to report its own
domain-specific readings.

```json
{
  "device_id": "edge-042",
  "event_id": "01JZ9X6N9VD4Y7Y3P2Z5F8K4QG",
  "timestamp": "2026-06-18T19:42:10Z",
  "type": "sensor.reading",
  "schema_version": 1,
  "payload": {
    "temperature_c": 41.2,
    "humidity": 0.61,
    "fan_rpm": 2380,
    "custom_field": "device-specific value"
  }
}
```

The `event_id` is part of the reliability story: it allows the control plane to
deduplicate telemetry during retries, reconnects, and offline replay.

## Planned Demo

The complete demo will run a simulated fleet of 50 to 100 agents and show:

- Device registration.
- Live telemetry ingestion.
- Random disconnects and reconnects.
- Local buffering during outages.
- Replay after recovery.
- Remote command delivery.
- Signed OTA canary rollout.
- Failed update simulation and rollback.
- Metrics, traces, logs, and health checks.

## Project Status

EdgeFleet is currently in the planning and foundation stage. The implementation
roadmap lives in [PLAN.md](PLAN.md).

The first milestone is a small vertical slice:

- Rust workspace.
- Shared telemetry types.
- Control plane scaffold.
- Agent scaffold.
- Dashboard scaffold.
- Docker Compose infrastructure.
- One simulated device registering, sending a heartbeat, and emitting one
  telemetry event.

## Roadmap

| Release | Focus |
| --- | --- |
| `v0.1.0-foundation` | Workspace, architecture contracts, CI, local infrastructure |
| `v0.2.0-mvp` | Registration, telemetry ingestion, live dashboard, simulator |
| `v0.3.0-reliability` | Offline queue, reconnect, replay, failure simulation |
| `v0.4.0-operations` | NATS, remote commands, metrics, tracing, health endpoints |
| `v0.5.0-ota` | Signed OTA artifacts, canaries, rollback, rollout dashboard |
| `v0.8.0-demo` | 50-100 agent simulation, polished dashboard, docs, benchmarks |
| `v1.0.0-rc.1` | Security pass, deployment guide, end-to-end release checklist |
| `v1.0.0` | Stable tagged release, Docker images, final demo assets |

See [PLAN.md](PLAN.md) for phase details and validation criteria.

## Development

Implementation has not been scaffolded yet. Once the Rust workspace and
dashboard exist, the expected development commands will be:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
docker compose up --build
```

Dashboard-specific commands will be documented after the frontend scaffold is
created.

## Documentation

- [PLAN.md](PLAN.md): execution roadmap, phases, release sequence, validation
  gates.
- [EdgeFleet.md](EdgeFleet.md): product positioning, architecture notes, and
  original project concept.
- [AGENTS.md](AGENTS.md): guidance for coding agents working in this repository.

## License

EdgeFleet is licensed under the MIT License. See [LICENSE.md](LICENSE.md).
