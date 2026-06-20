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

## Next

- Generate Rust and Go clients from the official light wallet protobufs.
- Replace placeholder HTTP probing with real lightwalletd gRPC checks.
- Add block-height, latency, and error-rate metrics.
- Add endpoint registry validation against the operations repository format.
- Add benchmark commands for repeated endpoint checks.
- Add release builds for `ssctl` and `lwd-exporter`.
- Add container publishing workflow.
- Expand dashboard panels for height lag, probe failures, and latency percentiles.
