# AGENTS.md

## What This Project Is

EdgeFleet is a Rust-based distributed edge telemetry and fleet orchestration
platform. It is intended to demonstrate production-style systems engineering:
async Rust services, resilient edge communication, local offline buffering,
event-driven messaging, observability, remote commands, and signed OTA update
workflows.

Do not position EdgeFleet as a generic AWS IoT replacement. The stronger
positioning is:

> Lightweight, self-hostable fleet telemetry and operations software for edge
> devices that need reliable offline behavior, remote updates, and simple
> operational visibility.

The project should optimize for a polished portfolio/demo system first. Cloud
scale, multi-region deployments, and advanced edge compute features belong after
the core fleet operations workflow is complete.

## Planned Architecture

### Edge Agent

Rust agent deployed to edge devices, Raspberry Pis, Linux VMs, containers, or
bare-metal systems.

Responsibilities:

- Register and authenticate with the control plane.
- Collect telemetry and health data.
- Persist telemetry locally during outages.
- Reconnect automatically with backoff and jitter.
- Replay buffered telemetry idempotently.
- Receive remote commands.
- Verify and apply signed OTA updates.

Likely technologies: Tokio, reqwest or Axum-compatible HTTP client stack,
SQLite, serde, tracing, rustls.

### Control Plane

Central API and orchestration service.

Responsibilities:

- Device registration and identity.
- Telemetry ingestion.
- Heartbeats and device status.
- Fleet inventory.
- Command dispatch and acknowledgement tracking.
- OTA rollout orchestration.
- WebSocket or server-sent event streams for the dashboard.
- Health, readiness, metrics, and tracing endpoints.

Likely technologies: Axum, PostgreSQL, NATS, OpenTelemetry, tracing.

### Messaging Layer

NATS is the planned event backbone for telemetry fanout, device state events,
and command delivery. Early phases may use direct API/database flows before NATS
is introduced, but do not design APIs in a way that blocks the NATS phase.

### Dashboard

React and Tailwind dashboard for fleet operations.

Expected views:

- Fleet overview.
- Device detail.
- Live telemetry.
- Device health and stale-device status.
- Local queue/replay visibility.
- Remote command history.
- OTA rollout status.
- Basic system health and observability links.

Build the dashboard as an operational tool, not a marketing site. Prioritize
clear scanability, dense but readable fleet state, and workflows an operator
would naturally expect.

## Telemetry Contract

Telemetry events should use a stable envelope with an open-ended JSON payload.
Operational fields stay typed; device-specific readings live in
`serde_json::Value`.

Example envelope:

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
    "fan_rpm": 2380
  }
}
```

Design around idempotency from the start. `event_id` should be suitable for
deduplication during replay, retries, and reconnect storms.

## Expected Commands

The exact commands may evolve as the repository is scaffolded. Prefer these
defaults unless the repo defines different scripts or Make targets.

```bash
# Rust formatting, linting, and tests
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# Run the local stack once Docker Compose exists
docker compose up --build

# Dashboard checks once the dashboard exists
npm install
npm run lint
npm run test
npm run build
```

If the project adds a task runner, Makefile, Justfile, pnpm workspace, or npm
workspace, follow the checked-in project commands instead of these defaults.

## Development Guidelines

- Check `PLAN.md` before choosing or scoping implementation work.
- Keep implementation aligned with the current phase. Do not pull post-1.0
  roadmap items into the critical path unless explicitly asked.
- Prefer simple vertical slices over broad scaffolding. A working registration
  and heartbeat path is more valuable than many empty crates.
- Keep shared wire types in a shared Rust crate once the workspace exists.
- Use structured errors and typed request/response models for control-plane and
  agent APIs.
- Use `tracing` spans around network, queue, persistence, command, and OTA
  boundaries.
- Treat offline behavior as a core product feature, not an afterthought.
- Build replay and command handling with idempotency and duplicate delivery in
  mind.
- Keep secrets, signing keys, and environment-specific config out of source
  control.
- Prefer documented local development defaults that work from a clean checkout.

## Testing Guidance

Before finishing code changes, run the narrowest useful verification plus the
broader checks that match the affected area.

Expected coverage over time:

- Unit tests for shared types, validation, queue behavior, retry policies, and
  rollout state transitions.
- Integration tests for device registration, heartbeat, telemetry ingestion,
  command acknowledgement, and OTA metadata flows.
- Replay tests that simulate control-plane downtime and verify buffered events
  are delivered once connectivity returns.
- Dashboard build and component tests once UI code exists.
- Docker Compose smoke test for the full local stack.

If tests cannot be run because the relevant code or tooling does not exist yet,
say so clearly and describe the manual check used instead.

## Documentation Expectations

When changing behavior, keep the docs in sync:

- Treat `PLAN.md` as the source of truth for current phase, status, and
  release sequence.
- Update `PLAN.md` when phase status, scope, or validation criteria change.
- When a `PLAN.md` deliverable is finished, prefix that bullet with
  `Completed:` while preserving the original item text.
- When a `PLAN.md` validation criterion has passed, prefix that bullet with
  `Completed:` while preserving the original item text.
- Only mark deliverables or validation criteria as `Completed:` after the
  implementation is present and the relevant check has actually been run or
  otherwise verified.
- Update `EdgeFleet.md` if product positioning or architecture changes.
- Add or update `README.md` once implementation begins.
- Document required environment variables, ports, credentials, and local demo
  steps as they are introduced.
- Keep release notes or changelog entries tied to milestone releases once
  releases begin.

Docs should explain what EdgeFleet does, what it intentionally does not do, and
how to run the demo from a clean checkout.

## Git And Collaboration

- Do not revert unrelated changes.
- Read nearby code before editing, and follow existing patterns.
- Keep changes scoped to the current request and current phase.
- Summarize changed behavior and verification steps when finishing work.
- If asked to create a git commit for Codex-authored changes, include
  `Co-authored-by: Codex <codex@openai.com>` in the commit message.
- Do not add the Codex co-author trailer for commits that only contain
  user-authored work or work authored by other AI tools.
