# Changelog

All notable milestone changes for EdgeFleet are recorded here.

## `v0.1.0-foundation` - 2026-06-23

Phase 0 foundation is complete and ready to tag.

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
