# shielded-stack

Infrastructure tooling for operating reliable Zcash light client services.

This repository contains the software side of the stack:

- Rust command-line tooling for endpoint checks, registry validation, and benchmarks.
- Go services for long-running probes and Prometheus-compatible health endpoints.
- Deployment examples for local, Kubernetes, and Helm-based environments.
- Dashboards and operational docs for service reliability.

## MVP Features

- Probe a `lightwalletd` endpoint through the real gRPC `GetLightdInfo` method.
- Report reachability, block height, latency, vendor, version, and chain name.
- Validate endpoint registry YAML files from the operations repository.
- Probe all active endpoints in a registry file.
- Run repeated endpoint probes and summarize success rate and latency.

## Repository Layout

```text
rust/
  crates/
    ssctl/       Command-line entrypoint.
    lwd-client/  Lightwalletd gRPC client and registry primitives.
    bench/       Repeated probe benchmark primitives.
go/
  cmd/
    lwd-exporter/  HTTP health and metrics service.
  internal/
    probe/         Endpoint probing package.
deploy/
  docker-compose/ Local deployment examples.
  k8s/            Kubernetes manifests.
  helm/           Helm chart.
dashboards/       Grafana dashboard definitions.
docs/             Architecture and operator notes.
```

## Usage

Build the CLI:

```sh
cargo build --manifest-path rust/Cargo.toml -p ssctl
```

Probe one endpoint:

```sh
cargo run --manifest-path rust/Cargo.toml -p ssctl -- health https://example.com:9067 --json
```

Benchmark one endpoint:

```sh
cargo run --manifest-path rust/Cargo.toml -p ssctl -- bench https://example.com:9067 --requests 5 --timeout-seconds 10
```

Validate an endpoint registry:

```sh
cargo run --manifest-path rust/Cargo.toml -p ssctl -- registry validate ../shielded-stack-ops/endpoints/mainnet.yaml
```

Probe all active endpoints in a registry:

```sh
cargo run --manifest-path rust/Cargo.toml -p ssctl -- registry probe ../shielded-stack-ops/endpoints/mainnet.yaml --json
```

## Development

```sh
make test
make build
```

`go test ./go/...` requires a local Go toolchain.

## Work Tracking

See [ROADMAP.md](ROADMAP.md) for completed setup work and next implementation tasks.

## References

- Zcash light client support: https://zcash.readthedocs.io/en/latest/rtd_pages/lightclient_support.html
- Lightwalletd setup: https://zcash.readthedocs.io/en/latest/rtd_pages/lightwalletd.html
- Lightwalletd repository: https://github.com/zcash/lightwalletd
- Light wallet protocol protobufs: https://github.com/zcash/lightwallet-protocol
- ZIP 307: https://zips.z.cash/zip-0307
