# Roadmap

This file tracks what has been completed and what remains for the core tooling repository.

## Completed

- Created the Rust workspace.
- Added the `ssctl` command-line entrypoint.
- Added endpoint validation primitives in `lwd-client`.
- Added benchmark planning primitives in `bench`.
- Added the Go `lwd-exporter` HTTP service.
- Added basic HTTP probe logic and Go tests.
- Added Docker Compose files for local exporter runs.
- Added Kubernetes manifests for exporter deployment.
- Added a Helm chart scaffold.
- Added a Grafana dashboard definition.
- Added architecture and lightwalletd reference docs.
- Added GitHub Actions workflow for Rust and Go checks.
- Added real `lightwalletd` gRPC probing through `GetLightdInfo`.
- Added endpoint registry parsing and validation.
- Added registry-wide active endpoint probes.
- Added repeated endpoint benchmark summaries.
- Added JSON and human-readable CLI output.

## Next

- Generate Rust and Go clients directly from the official protobufs.
- Add Go gRPC probing to `lwd-exporter`.
- Add Prometheus metrics for block height, height lag, latency, and probe failures.
- Add endpoint registry validation against stricter schema rules.
- Add release builds for `ssctl` and `lwd-exporter`.
- Add container publishing workflow.
- Expand dashboard panels for height lag, probe failures, and latency percentiles.
