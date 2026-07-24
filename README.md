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

Phase 0 foundation work is complete and tagged as `v0.1.0-foundation`. The
implementation roadmap lives in [PLAN.md](PLAN.md).

The Rust workspace has been scaffolded with:

- `crates/edgefleet-types`: shared telemetry, registration, heartbeat, command,
  and OTA wire models.
- `crates/edge-agent`: edge agent binary scaffold.
- `crates/control-plane`: control plane binary scaffold and initial API route
  outline.
- `crates/fleet-simulator`: local simulator tooling scaffold.
- `dashboard`: React, TypeScript, Tailwind, and Vite dashboard scaffold for the
  operator surface.

Phase 1 is underway. The first vertical slice — device registration — has
landed: the control plane serves health probes and an idempotent
`POST /api/v1/devices/register`, and the edge agent registers over HTTP,
declares telemetry profile metadata for its sample payload fields, and
persists/reuses its device identity across restarts.

The next active work continues the Phase 1 loop:

- The agent sends a heartbeat and emits one telemetry event over HTTP (the
  payloads are already built locally today).
- The control-plane device store moves from in-memory to PostgreSQL.
- Dashboard inventory and telemetry views consume stored registration metadata
  so configurable payload fields can be displayed without hard-coded sensor
  columns.

### Configurable Telemetry Fields

Telemetry events use a stable envelope with typed operational fields and an
open-ended JSON payload. Device-specific readings belong in that payload, so an
agent can emit fields such as `temperature_c`, `voltage_v`, `door_open`, or
`sample_count` without changing the shared event envelope.

The Phase 1 design is to keep ingestion flexible while giving the dashboard
enough metadata to render payloads well:

- Agents and simulators send telemetry payloads as JSON objects.
- Agent registration includes telemetry profile metadata for declared
  event types and payload fields.
- The control plane should store both recent telemetry payloads and profile
  metadata without rejecting undeclared JSON fields.
- The dashboard should use profile metadata for labels, units, field ordering,
  and display hints, then fall back to generic key/value rendering for unknown
  payload fields.

## Roadmap

The implementation roadmap, phase details, validation criteria, and release
sequence live in [PLAN.md](PLAN.md).

## Development

### Prerequisites

Install:

- Rust `1.95` or newer. The workspace uses Rust 2024 edition.
- `rustfmt` and `clippy`, usually installed with the standard Rust toolchain.
- Docker with Docker Compose for the local infrastructure stack.
- Node.js and npm for dashboard development and checks.

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

These commands are the current Rust baseline. They format the workspace, compile
all targets under Clippy with warnings treated as errors, and run the shared
type tests.

GitHub Actions runs the same Rust 1.95 workspace checks on pushes to `main` and
pull requests, plus an explicit locked workspace build:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --all-targets --locked
```

The dashboard scaffold is checked separately with Node.js 22:

```bash
cd dashboard
npm ci
npm run lint
npm run test
npm run build
```

### Running The Registration Slice

The control plane now starts a real HTTP server and the edge agent registers
with it over HTTP. Start the control plane first, then run the agent against it.

```bash
# Terminal 1: start the control plane (listens on 0.0.0.0:8080 by default)
cargo run -p control-plane

# Terminal 2: register an agent with the running control plane
cargo run -p edge-agent
```

The control plane exposes:

- `GET /healthz` — liveness probe, returns `ok`.
- `GET /readyz` — readiness probe, returns `ready`.
- `POST /api/v1/devices/register` — idempotent device registration. Re-registering
  the same `device_id` returns the original auth token and acceptance time.

You can exercise registration directly with `curl`:

```bash
curl -s -X POST http://localhost:8080/api/v1/devices/register \
  -H 'content-type: application/json' \
  -d '{"device_id":"edge-001","display_name":"Demo","agent_version":"0.1.0","capabilities":["telemetry"]}'
```

On a successful registration the agent persists its device identity (device id,
auth token, acceptance time) to a local state file and reuses it on the next
run, so restarts keep a stable identity. The control plane currently keeps
device records in memory; a PostgreSQL-backed store is the next slice.

The `fleet-simulator` remains a scaffold and prints sample telemetry events for
three simulated devices:

```bash
cargo run -p fleet-simulator
```

### Local Stack

The Docker Compose stack lives in [compose.yaml](compose.yaml). It starts local
PostgreSQL and NATS infrastructure, then runs the Rust control-plane,
edge-agent, and fleet-simulator binaries in containers:

```bash
docker compose up --build
```

The control plane now runs as a long-running HTTP service and the edge agent
registers against it on startup. The fleet-simulator remains a scaffold that
prints representative payloads and exits. Offline buffering, reconnect, and
backoff are Phase 2 work, so the agent currently expects the control plane to be
reachable when it starts.

Run the dashboard scaffold locally with:

```bash
cd dashboard
npm ci
npm run dev
```

The Vite dev server listens on `http://127.0.0.1:5173/` by default. The
dashboard reads `VITE_EDGEFLEET_API_URL`, which defaults to
`http://localhost:8080` when unset.

Default local ports:

- PostgreSQL: `localhost:5432`
- NATS client port: `localhost:4222`
- NATS monitoring: `localhost:8222`
- Control plane API: `localhost:8080`
- Dashboard dev server: `localhost:5173`

The compose file includes a dashboard service behind the `dashboard` profile:

```bash
docker compose --profile dashboard up --build
```

Dashboard-specific checks live in `dashboard/package.json`.

### Configuration

The control plane and edge agent do not require local credentials, databases,
NATS, or signing keys when run directly with Cargo; the control plane keeps
device records in memory for now. The `control-plane` binary reads:

- `EDGEFLEET_BIND_ADDR=0.0.0.0:8080`
- `RUST_LOG=info`

The `edge-agent` executable supports local-only defaults plus environment
overrides for its control-plane target, device identity, persisted state
location, and sample telemetry payload:

- `EDGEFLEET_CONTROL_PLANE_URL=http://localhost:8080`
- `EDGEFLEET_DEVICE_ID=edge-local-001`
- `EDGEFLEET_DEVICE_DISPLAY_NAME=Local development agent`
- `EDGEFLEET_AGENT_CAPABILITIES=telemetry,heartbeat`
- `EDGEFLEET_STATE_PATH=edge-agent-state.json`
- `EDGEFLEET_SAMPLE_PAYLOAD_JSON={"temperature_c":41.2,"humidity":0.61,"fan_rpm":2380}`

`EDGEFLEET_STATE_PATH` is where the agent stores its persisted device identity.
It is git-ignored by default so registration tokens never land in source
control.

Use `EDGEFLEET_SAMPLE_PAYLOAD_JSON` to change the sample telemetry fields as
well as their values:

```bash
EDGEFLEET_SAMPLE_PAYLOAD_JSON='{"voltage_v":12.4,"door_open":false,"sample_count":7}' \
  cargo run -p edge-agent
```

At startup, the agent derives registration profile metadata from the configured
sample payload. Each declared field includes its JSON value type, label, stable
display order, and any inferred unit or display hint. The profile helps future
dashboard views render known fields well while telemetry ingestion remains open
to undeclared JSON payload fields.

The Compose stack also provides local-only defaults for the future runtime:

- `EDGEFLEET_DATABASE_URL=postgres://edgefleet:edgefleet-dev@postgres:5432/edgefleet`
- `EDGEFLEET_NATS_URL=nats://nats:4222`
- `EDGEFLEET_BIND_ADDR=0.0.0.0:8080`
- `EDGEFLEET_CONTROL_PLANE_URL=http://control-plane:8080`

Do not commit local secrets, signing keys, or machine-specific configuration.

### Development Workflow

Before changing behavior, check [PLAN.md](PLAN.md) for the current phase and
validation criteria. Phase 1 work should stay focused on the first working
registration, heartbeat, telemetry, persistence, dashboard, and simulator loop.

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
- [CHANGELOG.md](CHANGELOG.md): milestone release notes.
- [AGENTS.md](AGENTS.md): guidance for coding agents working in this repository.

## License

EdgeFleet is licensed under the MIT License. See [LICENSE.md](LICENSE.md).
