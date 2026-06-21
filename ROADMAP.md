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
- Added vendored light wallet protobuf files.
- Switched Rust gRPC calls to generated `tonic` client code.
- Added generated Go light wallet protocol bindings.
- Switched `lwd-exporter` to gRPC `GetLightdInfo` probes.
- Added exporter metrics for reported height, estimated height, height lag, reachability, and latency.
- Added stricter endpoint registry validation for timestamps, URLs, duplicate IDs, duplicate URLs, endpoint IDs, and capabilities.

## Next

- Add release builds for `ssctl` and `lwd-exporter`.
- Add container publishing workflow.
- Expand dashboard panels for probe failures and latency percentiles.
- Add automated status updates from registry probe output.
