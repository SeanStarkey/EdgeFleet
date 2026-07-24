# EdgeFleet

A distributed edge telemetry and fleet orchestration platform written in Rust.

## Overview

EdgeFleet is a lightweight platform for managing edge devices and intermittent-connectivity sensor systems. The project demonstrates modern systems engineering practices using Rust, async networking, distributed messaging, observability, OTA updates, and resilient edge communication patterns.

The platform is designed to showcase:
- Distributed systems architecture
- Async Rust engineering
- Fleet management concepts
- Observability and reliability engineering
- OTA deployment workflows
- Offline-first edge computing patterns

---

# Product Positioning

EdgeFleet should not be positioned as a generic replacement for AWS IoT data
collection. AWS already provides mature managed services for device
connectivity, telemetry ingestion, message routing, fleet indexing, remote jobs,
OTA-style deployments, industrial equipment data collection, and edge runtime
management through services such as AWS IoT Core, AWS IoT Device Management, AWS
IoT Greengrass, and AWS IoT SiteWise.

The stronger positioning is:

> Lightweight, self-hostable fleet telemetry and operations software for edge
> devices that need reliable offline behavior, remote updates, and simple
> operational visibility.

In this framing, EdgeFleet is not just a data pipe. The value is the operational
workflow around the device fleet:

- Install a small Rust agent on edge devices
- Register and authenticate devices
- Collect telemetry with offline buffering and replay
- See device health and fleet status in one dashboard
- Send remote commands
- Roll out signed updates with canaries and rollback
- Understand what happened when devices disconnect, recover, or fail

## Commercial Angle

Selling EdgeFleet as a completely generic IoT ingestion platform would compete
directly with large cloud providers and established IoT platforms. A more viable
commercial strategy would be to start with a specific vertical where downtime,
connectivity gaps, or missed telemetry have obvious cost.

Potential vertical packages:

- Cold-chain and refrigeration monitoring
- Solar inverter or battery fleet monitoring
- Industrial pump station telemetry
- Agricultural sensor gateways
- Remote kiosk or vending machine monitoring
- Construction-site equipment tracking
- Smart-building gateway operations

The generic platform remains the same across these markets, but the first
customer-facing product should be opinionated: prebuilt dashboards, alert rules,
device payload examples, and reports for one target use case.

## AWS Relationship

EdgeFleet can be positioned in three ways relative to AWS:

1. **Portfolio project**: demonstrate senior-level Rust, distributed systems,
   async networking, offline-first design, observability, and deployment
   workflows without hiding the core engineering behind managed services.
2. **Simpler self-hosted alternative**: serve teams that want the core fleet
   operations workflow without committing to AWS-specific IoT concepts or cloud
   architecture.
3. **AWS-compatible companion**: provide the agent, local buffering, dashboard,
   and workflow layer while forwarding telemetry into AWS IoT Core, S3,
   Timestream, SiteWise, or another downstream system.

This keeps the project commercially realistic: EdgeFleet should compete on
developer experience, operational simplicity, self-hostability, and vertical
workflow polish rather than raw cloud-scale message ingestion.

---

# Goals

## Primary Goals
- Demonstrate senior-level Rust engineering
- Build a production-style distributed system
- Showcase async/concurrent programming
- Implement resilient edge communication
- Provide a realistic deployment and operations story

## Secondary Goals
- Benchmark Rust networking performance
- Demonstrate observability tooling
- Explore edge-device deployment patterns
- Create a polished portfolio project with documentation and demos

---

# High-Level Architecture

## Components

### 1. Edge Agent (Rust)
A lightweight agent deployed on edge devices.

Responsibilities:
- Collect telemetry
- Stream metrics/events
- Receive remote commands
- Handle OTA updates
- Persist data during outages
- Reconnect automatically

Technologies:
- Tokio
- Axum client stack
- SQLite
- tracing
- serde
- rustls

---

### 2. Control Plane API
Central orchestration and management service.

Responsibilities:
- Device registration
- Authentication
- Fleet management
- Configuration distribution
- OTA orchestration
- WebSocket event streaming

Technologies:
- Axum
- PostgreSQL
- NATS
- OpenTelemetry

---

### 3. Messaging Layer
Reliable event transport between agents and services.

Responsibilities:
- Telemetry ingestion
- Event fanout
- Device state propagation
- Command delivery

Technologies:
- NATS

---

### 4. Dashboard UI
Web interface for fleet visibility and operations.

Features:
- Device inventory
- Live telemetry
- OTA deployment status
- Health monitoring
- Metrics visualization

Technologies:
- React
- Tailwind
- WebSockets

---

# Core Features

## Device Management
- Device registration
- Device identity/authentication
- Heartbeats
- Status monitoring

## Telemetry
- Structured event ingestion
- Metrics streaming
- Offline buffering
- Replay after reconnect

Telemetry events use a stable envelope with an open-ended JSON payload. The
envelope gives the platform enough structure for routing, authentication,
deduplication, replay, and dashboard display, while the payload allows each
device type to report domain-specific readings without requiring platform
schema changes for every new metric. Event identifiers are UUIDv7 values so
newly generated events remain sortable while retaining unique idempotency keys.

Example telemetry event:

```json
{
  "device_id": "edge-042",
  "event_id": "019f9656-5e97-72b0-ba3f-9ccde815d18a",
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

In Rust, the operational fields should remain typed and the device-specific
body can be represented as `serde_json::Value`.

## OTA Updates
- Signed update artifacts
- Canary deployments
- Rollback support
- Version tracking

## Reliability
- Automatic reconnect
- Exponential backoff
- Persistent local queues
- Idempotent delivery patterns

## Observability
- Structured logging
- Distributed tracing
- Metrics collection
- Health endpoints

---

# Deployment Architecture

## Recommended Hosting

### Control Plane
- Fly.io
- Docker containers
- PostgreSQL
- NATS

### Artifact Storage
- AWS S3
- CloudFront (optional)

### Agent Targets
- Raspberry Pi
- Docker containers
- Linux VMs
- Bare-metal systems

---

# Development Phases

## Phase 1 — MVP
### Goals
- Basic agent communication
- Telemetry ingestion
- Device registration
- Live dashboard

### Deliverables
- Rust edge agent
- Axum control API
- PostgreSQL schema
- Simple React dashboard
- Docker Compose deployment

---

## Phase 2 — Reliability & Operations
### Goals
- Improve resilience
- Add observability
- Support OTA workflows

### Deliverables
- SQLite local buffering
- OTA deployments
- Prometheus metrics
- OpenTelemetry tracing
- NATS integration

---

## Phase 3 — Advanced Features
### Goals
- Demonstrate advanced distributed systems patterns

### Potential Features
- Multi-region support
- Peer failover
- CRDT synchronization
- WASM plugins
- Edge compute tasks
- Fine-grained rollout policies

---

# Demonstration Scenario

## Fleet Simulation
Run 50–100 simulated agents.

Simulation behaviors:
- Random disconnects
- Telemetry bursts
- OTA rollouts
- Failure injection
- Automatic recovery

The simulation demonstrates:
- Resilience
- Scalability
- Async concurrency
- Observability
- Fleet orchestration

---

# Engineering Focus Areas

## Rust Concepts Demonstrated
- Async/await
- Tokio task orchestration
- Error handling
- Ownership and lifetimes
- Trait-based architecture
- Zero-copy patterns
- Structured concurrency

## Systems Engineering Concepts
- Distributed systems
- Event-driven architecture
- Fault tolerance
- Backpressure handling
- Retry strategies
- Operational visibility

---

# Tooling

## CI/CD
- GitHub Actions
- cargo fmt
- cargo clippy
- cargo test
- Docker builds

## Observability
- Prometheus
- Grafana
- OpenTelemetry
- tracing

## Benchmarking
- criterion
- flamegraphs
- load testing

---

# Stretch Goals

- End-to-end encryption
- Mutual TLS authentication
- Delta OTA updates
- Fleet policy engine
- Edge-side rule execution
- Device digital twins

---

# Portfolio Value

This project demonstrates:
- Production-quality Rust engineering
- Distributed systems design
- Reliability engineering
- Infrastructure architecture
- Observability practices
- Modern cloud-native operations

It positions the author as:
- A senior systems engineer
- A distributed systems engineer
- A modern infrastructure/platform engineer
- A Rust-focused backend and edge-computing developer
