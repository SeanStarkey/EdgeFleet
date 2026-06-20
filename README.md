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

EdgeFleet is currently in the foundation stage. The implementation roadmap
lives in [PLAN.md](PLAN.md).

The Rust workspace has been scaffolded with:

- `crates/edgefleet-types`: shared telemetry, registration, heartbeat, command,
  and OTA wire models.
- `crates/edge-agent`: edge agent binary scaffold.
- `crates/control-plane`: control plane binary scaffold and initial API route
  outline.
- `crates/fleet-simulator`: local simulator tooling scaffold.

The remaining first milestone work is:

- Dashboard scaffold.
- Docker Compose infrastructure.
- One simulated device registering, sending a heartbeat, and emitting one
  telemetry event against the control plane.

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

### Prerequisites

Install:

- Rust `1.95` or newer. The workspace uses Rust 2024 edition.
- `rustfmt` and `clippy`, usually installed with the standard Rust toolchain.
- Docker with Docker Compose, once the local infrastructure stack is added.
- Node.js and npm, once the dashboard scaffold is added.

From a clean checkout, verify the Rust toolchain with:

```bash
rustc --version
cargo --version
```

### First-Time Setup

Clone the repository and run the Rust checks from the repository root:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

These commands are the current baseline for Phase 0 work. They format the
workspace, compile all targets under Clippy with warnings treated as errors, and
run the shared type tests.

### Running The Current Scaffold

The repository currently contains Rust scaffolds for the control plane, edge
agent, shared wire types, and fleet simulator. The binaries do not start network
listeners yet; they print representative route, registration, heartbeat, and
telemetry payloads so the shared contracts can be exercised while the API and
runtime pieces are being built.

```bash
cargo run -p control-plane
cargo run -p edge-agent
cargo run -p fleet-simulator
```

Expected output:

- `control-plane` prints the planned health, registration, telemetry,
  heartbeat, command, and OTA metadata routes plus example registration JSON.
- `edge-agent` prints a sample heartbeat and telemetry event for a local
  development device.
- `fleet-simulator` generates sample telemetry events for three simulated
  devices.

### Local Stack Status

The Phase 0 Docker Compose stack and dashboard scaffold are still planned work.
After they exist, the local stack will include PostgreSQL, NATS, the control
plane, dashboard, and simulated agents, and the expected command will be:

```bash
docker compose up --build
```

Dashboard-specific commands such as `npm install`, `npm run lint`, `npm run
test`, and `npm run build` will be documented after the frontend scaffold is
created.

### Configuration

The current Rust scaffolds do not require environment variables, local
credentials, databases, NATS, or signing keys. As those pieces are introduced,
this README will document required variables, default ports, credentials for
local-only development, and demo startup steps.

Do not commit local secrets, signing keys, or machine-specific configuration.

### Development Workflow

Before changing behavior, check [PLAN.md](PLAN.md) for the current phase and
validation criteria. Keep Phase 0 work focused on repository foundation,
contracts, local development, and scaffolding.

When changing shared API or event contracts:

- Put reusable wire types in `crates/edgefleet-types`.
- Keep operational telemetry fields typed and device-specific readings in
  `serde_json::Value`.
- Preserve `event_id` as the idempotency key for retries, replay, and duplicate
  delivery.
- Add or update focused tests for validation and serialization behavior.

When changing documentation or local workflow:

- Keep this README accurate for clean-checkout setup.
- Update [CONTRIBUTING.md](CONTRIBUTING.md) if contributor expectations change.
- Update [EdgeFleet.md](EdgeFleet.md) if product positioning or architecture
  changes.
- Update [PLAN.md](PLAN.md) when phase status, scope, or validation status
  changes.

## Documentation

- [PLAN.md](PLAN.md): execution roadmap, phases, release sequence, validation
  gates.
- [EdgeFleet.md](EdgeFleet.md): product positioning, architecture notes, and
  original project concept.
- [CONTRIBUTING.md](CONTRIBUTING.md): contribution workflow, local checks, and
  documentation expectations.
- [AGENTS.md](AGENTS.md): guidance for coding agents working in this repository.

## License

EdgeFleet is licensed under the MIT License. See [LICENSE.md](LICENSE.md).
