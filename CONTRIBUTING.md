# Contributing To EdgeFleet

Thanks for helping improve EdgeFleet. This project is currently in its
foundation phase, so the best contributions are small vertical slices that make
the local demo more real without pulling later roadmap work into the critical
path.

## Project Direction

EdgeFleet is a lightweight, self-hostable fleet telemetry and operations system
for edge devices that need reliable offline behavior, remote updates, and simple
operational visibility.

Keep contributions aligned with that scope:

- Prioritize a polished portfolio/demo workflow over cloud-scale architecture.
- Treat offline buffering, idempotent replay, and operational visibility as core
  product behavior.
- Avoid positioning the project as a generic AWS IoT replacement.
- Check [PLAN.md](PLAN.md) before changing scope, status, or milestone wording.

## Repository Layout

- `crates/edgefleet-types`: shared wire models for telemetry, registration,
  heartbeats, commands, and OTA metadata.
- `crates/edge-agent`: edge agent binary scaffold.
- `crates/control-plane`: control plane API scaffold.
- `crates/fleet-simulator`: local simulator tooling scaffold.
- [README.md](README.md): project overview and current development commands.
- [EdgeFleet.md](EdgeFleet.md): product positioning and architecture notes.
- [PLAN.md](PLAN.md): roadmap, phase status, release sequence, and validation
  criteria.

## Local Setup

Install a current stable Rust toolchain with `rustup`, then run the workspace
checks from the repository root:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

The current scaffold binaries can be run directly:

```bash
cargo run -p control-plane
cargo run -p edge-agent
cargo run -p fleet-simulator
```

Docker Compose, dashboard, database, and NATS commands will be documented after
those parts of the Phase 0 scaffold exist.

## Development Guidelines

- Read nearby code before editing and follow the existing style.
- Keep changes scoped to the current phase unless a later-phase item was
  explicitly requested.
- Prefer typed request and response models for API boundaries.
- Keep shared wire contracts in `edgefleet-types`.
- Design telemetry ingestion, replay, commands, and OTA flows around
  idempotency and duplicate delivery.
- Use structured errors and `tracing` spans around network, persistence, queue,
  command, and update boundaries as those flows are implemented.
- Do not commit secrets, signing keys, local credentials, or environment-specific
  configuration.

## Documentation Guidelines

Update documentation in the same change when behavior, commands, ports,
environment variables, architecture, or milestone status changes.

- Update [README.md](README.md) for user-facing setup and demo instructions.
- Update [EdgeFleet.md](EdgeFleet.md) for product positioning or architecture
  changes.
- Update [PLAN.md](PLAN.md) when phase status, scope, deliverables, or validation
  criteria change.
- Only mark a `PLAN.md` item as `Completed:` after the implementation exists and
  the relevant check has passed or been otherwise verified.

## Testing Expectations

Run the narrowest useful checks while developing, then run the broader checks
before submitting:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

As the project grows, add focused tests for shared type validation, queue
behavior, retry policy, telemetry replay, registration, heartbeat ingestion,
command acknowledgement, and OTA rollout state transitions.

If a check cannot be run because the relevant tooling does not exist yet, note
that clearly in the change summary.

## Pull Request Checklist

Before opening or merging a change, confirm:

- The change matches the current `PLAN.md` phase or an explicitly requested
  follow-up.
- Rust formatting, linting, and tests pass.
- Public wire types remain documented and backward-compatible for the current
  milestone.
- User-facing behavior changes are reflected in the README or architecture docs.
- New ports, credentials, environment variables, and local demo steps are
  documented.
- No secrets, generated build artifacts, or machine-local files are included.
