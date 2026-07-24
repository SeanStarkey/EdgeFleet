# EdgeFleet Project Plan

EdgeFleet is a Rust-based edge telemetry and fleet orchestration platform. This
plan breaks the project into milestones that turn the current product concept
into a demonstrable, release-ready portfolio system with a credible path toward
commercial specialization.

## Product Target

EdgeFleet should be completed as a lightweight, self-hostable fleet operations
platform for edge devices that need:

- Device registration and identity
- Reliable telemetry ingestion
- Offline buffering and replay
- Fleet health visibility
- Remote command delivery
- Signed OTA updates with canaries and rollback
- A polished dashboard and demo scenario

The first complete version should optimize for a strong portfolio and demo
story rather than cloud-provider scale. The platform should feel production
shaped: observable, documented, testable, deployable, and easy to run locally.

## Completion Definition

The project is complete when a reviewer can:

- Start the full stack with Docker Compose.
- Register simulated and real agents.
- Watch live telemetry and device health in the dashboard.
- Disconnect agents and see local buffering plus replay after reconnect.
- Send a remote command to a device.
- Run a signed OTA rollout to a canary group and observe success or rollback.
- Inspect logs, metrics, traces, and health endpoints.
- Read clear architecture, operations, and demo documentation.
- Run tests, linting, and CI successfully from a clean checkout.

## Phase 0: Project Foundation

Goal: create the repository shape, development workflow, and architectural
contracts that the rest of the project can build on.

Current status: complete. The Rust workspace exists with crates for the edge
agent, control plane, shared wire types, and fleet simulator tooling. The shared
crate includes initial telemetry, registration, heartbeat, command, and OTA
metadata contracts. The Phase 0 Docker Compose file defines the local
PostgreSQL, NATS, Rust scaffold services, and dashboard profile service shape.
The dashboard scaffold, repository documentation, and CI workflow are in place.

Deliverables:

- Completed: Rust workspace with crates for `edge-agent`, `control-plane`,
  shared types, and simulator tooling.
- Completed: Dashboard app scaffold.
- Completed: Docker Compose file for local PostgreSQL, NATS, control plane,
  dashboard, and simulated agents.
- Completed: Shared telemetry envelope model with typed operational fields and
  flexible `serde_json::Value` payloads.
- Completed: Initial API contract for device registration, telemetry ingestion,
  heartbeats, commands, and OTA metadata.
- Completed: Repository documentation: README, architecture overview, local
  development guide, and contribution notes.
- Completed: GitHub Actions workflow for format, lint, tests, and build checks.

Validation:

- Completed: `cargo fmt`, `cargo clippy`, and `cargo test` pass.
- Completed: Dashboard scaffold starts locally.
- Completed: Docker Compose starts the empty infrastructure stack.

Target release: `v0.1.0-foundation`

## Phase 1: MVP Fleet Telemetry

Goal: prove the core loop of device registration, telemetry flow, and live fleet
visibility.

Deliverables:

- Completed: Edge agent can register with the control plane and persist its
  device identity.
- Completed: Agent registration includes telemetry profile metadata for declared event
  types and payload fields, while ingestion still accepts open-ended JSON
  payloads.
- Completed: Shared newtype identifiers (`DeviceId`, `EventId`, `AuthToken`) in
  `edgefleet-types` so invalid identifiers are unrepresentable, with `EventId`
  backed by a ULID or UUIDv7 so event ids are sortable and dedup-friendly for
  Phase 2 replay.
- Immutable value interfaces for the remaining Phase 1 messages and snapshots:
  `TelemetryProfile`, `TelemetryEventProfile`, `TelemetryFieldProfile`,
  `DeviceRegistrationRequest`, `DeviceRegistrationResponse`,
  `HeartbeatRequest`, `DeviceIdentity`, `AgentConfig`, and `StartupSnapshot`.
  Fields are private, construction and deserialization enforce each type's
  invariants, and callers receive read-only accessors. Updates create new values
  rather than mutating an existing message or snapshot.
- Encapsulated `DeviceRecord` mutation: identity fields (`device_id`,
  `auth_token`, and `registered_at`) remain immutable while registration
  metadata changes only through an explicit refresh method.
- Control plane stores device records, heartbeats, and recent telemetry in
  PostgreSQL behind a `DeviceStore` trait (async fn in trait), keeping the
  in-memory registry as the test implementation and using `sqlx`
  compile-time-checked queries for the PostgreSQL implementation.
- Agent sends structured telemetry events over HTTP or WebSocket.
- Agent runtime structured as concurrent Tokio tasks (heartbeat, telemetry
  sending) wired with channels, supervised with `JoinSet`, and stopped via
  cancellation on SIGTERM; control plane serves with graceful shutdown.
- Dashboard shows device inventory, online/offline status, latest heartbeat,
  and recent telemetry.
- Dashboard renders telemetry payload fields from the shared event contract
  without hard-coded sensor columns, so configurable agent and simulator
  payloads stay visible in the operator UI.
- Basic authentication mechanism for agents, implemented as a typed Axum
  extractor (`FromRequestParts`) with a single control-plane `ApiError` type
  that implements `IntoResponse`.
- Simulator can run a configurable number of agents locally by driving the
  `edge-agent` library crate as concurrent Tokio tasks.
- Seed data or demo script for a small fleet.
- Workspace lint policy via `[workspace.lints]` (deny `unwrap`/`expect` in
  non-test code plus a curated clippy subset) and a dependency audit step
  (`cargo audit` or `cargo deny`) in CI.

Validation:

- Run at least 10 simulated agents locally.
- Dashboard updates device status and telemetry without page reloads.
- A configurable telemetry payload sent by an agent or simulator appears in the
  dashboard with its custom fields and values without frontend code changes.
- Registration stores telemetry profile metadata that the dashboard can use for
  labels, units, field ordering, or other display hints without rejecting
  undeclared telemetry fields.
- Integration tests cover registration, heartbeat, and telemetry ingestion.
- README contains an MVP demo walkthrough.

Target release: `v0.2.0-mvp`

## Phase 2: Reliability And Offline Operation

Goal: make the edge behavior resilient enough to demonstrate real intermittent
connectivity patterns.

Deliverables:

- Agent-side SQLite queue for telemetry buffering.
- Automatic reconnect with exponential backoff and jitter, implemented as a
  pure retry-policy type with injected clock and RNG so schedules are
  deterministic under test.
- Replay after reconnect with idempotent event handling.
- Delivery status tracking for queued events.
- Backpressure limits and queue retention policy.
- Failure simulation for disconnects, delayed networks, telemetry bursts, and
  service restarts.
- Dashboard indicators for stale devices, replay activity, and queue depth.

Validation:

- Simulated agents continue collecting telemetry while the control plane is
  offline.
- Buffered telemetry replays correctly after reconnect.
- Duplicate telemetry submissions are ignored or handled idempotently.
- Tests cover queue persistence, replay ordering, and reconnect behavior.
- Backoff schedules are verified deterministically with `tokio::time::pause`,
  and queue replay ordering and dedup invariants are covered by
  property-based tests (`proptest`).

Target release: `v0.3.0-reliability`

## Phase 3: Messaging, Commands, And Operations

Goal: introduce the event-driven backbone and remote operations workflow.

Deliverables:

- NATS integration for telemetry fanout, device state events, and command
  dispatch.
- Remote command API and dashboard controls.
- Agent command receiver with command acknowledgement.
- Immutable `CommandRequest` and `CommandAck` wire values with validated
  construction and deserialization, so retries and duplicate delivery cannot
  change the meaning associated with an existing command or acknowledgement.
- Command history and status tracking in the control plane, with the command
  lifecycle modeled as typed state transitions (transition methods on the
  status enum that reject invalid moves) rather than free-form status writes.
- Structured logging with `tracing`.
- Prometheus metrics for API, agent, queue, command, and telemetry paths.
- OpenTelemetry tracing for key request and event flows.
- Health and readiness endpoints for each service.
- Grafana dashboard or documented observability queries.

Validation:

- User can send a command from the dashboard and observe acknowledgement.
- NATS restart does not corrupt device or command state.
- Metrics expose useful counters, gauges, and latency histograms.
- Traces connect telemetry ingestion through persistence or fanout.

Target release: `v0.4.0-operations`

## Phase 4: OTA Update Workflow

Goal: complete the most distinctive fleet-management feature: signed remote
updates with safe rollout behavior.

Deliverables:

- Signed update artifact format and verification flow.
- OTA metadata model: artifact, version, target group, rollout state, and
  rollback target.
- Immutable OTA metadata values with validated construction and deserialization.
  Hash, signature, artifact identity, and location remain bound together, with
  a distinct verified type produced after successful cryptographic
  verification.
- Agent update checker and artifact downloader.
- Canary rollout support.
- Rollout pause, resume, cancel, and rollback operations, modeled as a typed
  rollout state machine whose transition methods return errors for invalid
  moves and are exhaustively unit tested.
- Dashboard view for rollout progress and device update status.
- Demo artifact that updates simulated agent version metadata.
- Documentation for signing keys, artifact publishing, and rollback behavior.

Validation:

- A canary rollout updates a small device group before broad rollout.
- Failed update simulation triggers visible failure state and supports rollback.
- Agents reject unsigned or invalid artifacts.
- Tests cover signature verification, rollout state transitions, and rollback.

Target release: `v0.5.0-ota`

## Phase 5: Demo Hardening And Portfolio Polish

Goal: make the project impressive, understandable, and reliable for reviewers.

Deliverables:

- Fleet simulation for 50 to 100 agents with configurable scenarios, reusing
  the `edge-agent` library for each simulated agent and throttling
  registration storms with a semaphore.
- End-to-end demo script covering registration, telemetry, disconnect, replay,
  command delivery, OTA canary, failure, and rollback.
- Load test or benchmark report for telemetry ingestion and agent concurrency.
- Architecture diagrams and operational runbook.
- Dashboard polish for fleet overview, device detail, telemetry charts,
  command history, rollout status, and system health.
- CI coverage for Rust crates, dashboard build, Docker image build, and
  integration tests.
- Public-facing README with screenshots, demo GIFs or videos, and clear setup
  instructions.

Validation:

- Full demo can be run from a clean checkout in under 15 minutes.
- 50 simulated agents run on a development machine without manual intervention.
- CI is green on a clean branch.
- Documentation explains what EdgeFleet is, what it is not, and why it matters.

Target release: `v0.8.0-demo`

## Phase 6: Release Candidate

Goal: close gaps, stabilize the user experience, and prepare the first complete
release.

Deliverables:

- Security pass over authentication, secrets, signing keys, and default config.
- Error handling pass over agent, API, simulator, and dashboard.
- Database migration review and clean bootstrap path.
- End-to-end test coverage for the primary demo scenario.
- Deployment guide for local Docker Compose and a hosted control plane.
- Issue templates, license, changelog, and release checklist.
- Final vertical demo package, such as cold-chain monitoring or solar inverter
  monitoring, with sample payloads, dashboards, and alerts.

Validation:

- Release checklist passes from a fresh clone.
- All high-priority bugs from dogfooding are closed.
- Demo scenario is repeatable and documented.
- Public docs and product positioning are consistent.

Target release: `v1.0.0-rc.1`

## Phase 7: First Stable Release

Goal: publish a stable, portfolio-ready EdgeFleet release.

Deliverables:

- Final bug fixes from the release candidate.
- Versioned Docker images.
- Tagged source release.
- Changelog for all public features.
- Final screenshots or recorded demo.
- Clear roadmap for post-1.0 work.

Validation:

- CI passes on the release commit.
- Docker images run the documented demo.
- Release notes describe setup, capabilities, limitations, and next steps.

Target release: `v1.0.0`

## Release Schedule

The release schedule is milestone-based rather than date-based. Each release
should be cut when its validation criteria pass and the documented demo path for
that phase works from a clean checkout.

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

## Post-1.0 Roadmap

These items should stay out of the critical path unless they become necessary
for the core demo:

- Mutual TLS for agent authentication.
- AWS-compatible telemetry forwarding.
- Multi-tenant organization model.
- Fine-grained rollout policies.
- Delta OTA updates.
- Edge-side rule execution.
- WASM plugin support.
- Device digital twins.
- Multi-region control plane deployment.
- Vertical product packs for cold-chain, solar, agriculture, or industrial
  monitoring.

## Current Next Step

Registration is the first landed Phase 1 slice: the edge agent registers with
the control plane over HTTP, receives an auth token, and persists/reuses its
device identity across restarts. The control plane serves `/healthz`, `/readyz`,
and an idempotent `POST /api/v1/devices/register` backed by an in-memory device
registry.

The next slice extends this loop: have the agent send a heartbeat and emit one
telemetry event over HTTP (the payloads are already built locally), then move
the control-plane device store from in-memory to PostgreSQL so device records,
heartbeats, and recent telemetry survive restarts. The PostgreSQL move should
introduce the `DeviceStore` trait seam, keeping the current in-memory registry
as the test implementation.
