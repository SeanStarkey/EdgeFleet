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

EdgeFleet is planned around an edge agent, control plane, messaging layer,
operator dashboard, and simulator. The detailed system design, component
boundaries, data flows, telemetry contract, reliability rules, and phase
boundaries live in [docs/architecture.md](docs/architecture.md).

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
- One simulated device registering, sending a heartbeat, and emitting one
  telemetry event against the control plane.

## Roadmap

The implementation roadmap, phase details, validation criteria, and release
sequence live in [PLAN.md](PLAN.md).

## Development

### Prerequisites

Install:

- Rust `1.95` or newer. The workspace uses Rust 2024 edition.
- `rustfmt` and `clippy`, usually installed with the standard Rust toolchain.
- Docker with Docker Compose for the local infrastructure stack.
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

### Local Stack

The Phase 0 Docker Compose stack lives in [compose.yaml](compose.yaml). It
starts local PostgreSQL and NATS infrastructure, then runs the current Rust
control-plane, edge-agent, and fleet-simulator scaffold binaries in containers:

```bash
docker compose up --build
```

The Rust binaries are still scaffold programs, not long-running network
services. They print representative payloads and exit successfully while the
real API runtime is being built.

Default local ports:

- PostgreSQL: `localhost:5432`
- NATS client port: `localhost:4222`
- NATS monitoring: `localhost:8222`
- Control plane API placeholder: `localhost:8080`
- Dashboard dev server placeholder: `localhost:5173`

The compose file includes a dashboard service behind the `dashboard` profile so
the service shape is reserved without breaking the default stack before the
frontend scaffold exists:

```bash
docker compose --profile dashboard up --build
```

Dashboard-specific commands such as `npm install`, `npm run lint`, `npm run
test`, and `npm run build` will be documented after the frontend scaffold is
created.

### Configuration

The current Rust scaffolds do not require environment variables, local
credentials, databases, NATS, or signing keys when run directly with Cargo. The
Compose stack provides local-only defaults for the future runtime:

- `EDGEFLEET_DATABASE_URL=postgres://edgefleet:edgefleet-dev@postgres:5432/edgefleet`
- `EDGEFLEET_NATS_URL=nats://nats:4222`
- `EDGEFLEET_BIND_ADDR=0.0.0.0:8080`
- `EDGEFLEET_CONTROL_PLANE_URL=http://control-plane:8080`

Do not commit local secrets, signing keys, or machine-specific configuration.

### Development Workflow

Before changing behavior, check [PLAN.md](PLAN.md) for the current phase and
validation criteria. Keep Phase 0 work focused on repository foundation,
contracts, local development, and scaffolding.

Use [docs/architecture.md](docs/architecture.md) for system boundaries and
contract rules. Use [CONTRIBUTING.md](CONTRIBUTING.md) for contributor workflow,
testing expectations, and documentation update rules.

## Documentation

- [PLAN.md](PLAN.md): execution roadmap, phases, release sequence, validation
  gates.
- [docs/architecture.md](docs/architecture.md): system architecture, component
  boundaries, data flows, reliability rules, and phase boundaries.
- [EdgeFleet.md](EdgeFleet.md): product positioning, architecture notes, and
  original project concept.
- [CONTRIBUTING.md](CONTRIBUTING.md): contribution workflow, local checks, and
  documentation expectations.
- [AGENTS.md](AGENTS.md): guidance for coding agents working in this repository.

## License

EdgeFleet is licensed under the MIT License. See [LICENSE.md](LICENSE.md).
