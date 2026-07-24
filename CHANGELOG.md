# Changelog

All notable milestone changes for EdgeFleet are recorded here.

## Unreleased

Work toward `v0.2.0-mvp`.

### Added

- Control-plane HTTP service (Axum) with `/healthz`, `/readyz`, and an
  idempotent `POST /api/v1/devices/register` endpoint backed by an in-memory
  device registry.
- Edge agent registers with the control plane over HTTP (reqwest) and persists
  its device identity (device id, auth token, acceptance time) to a local state
  file, reusing it across restarts. Configurable via `EDGEFLEET_STATE_PATH`.
- Agent registration includes telemetry profile metadata derived from the
  configured sample payload, including field types, labels, units, display
  order, and display hints for future dashboard rendering.
- End-to-end registration integration test that boots the control plane and
  drives the agent registration flow, including idempotent re-registration.

## `v0.1.0-foundation` - 2026-06-23

Phase 0 foundation was completed and tagged.

### Added

- Rust workspace with `edge-agent`, `control-plane`, `edgefleet-types`, and
  `fleet-simulator` crates.
- Shared telemetry, registration, heartbeat, command, and OTA metadata wire
  models.
- React, TypeScript, Tailwind, and Vite dashboard scaffold.
- Docker Compose stack for PostgreSQL, NATS, Rust scaffold services, and the
  dashboard profile.
- GitHub Actions CI for Rust format, lint, test, and build checks plus
  dashboard lint, test, and build checks.
- Repository documentation for architecture, local development, contribution
  workflow, phase plan, and coding-agent expectations.

### Validation

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo build --workspace --all-targets --locked`
- `npm run lint`
- `npm run test`
- `npm run build`
- Dashboard dev server smoke check
- Docker Compose infrastructure smoke check
