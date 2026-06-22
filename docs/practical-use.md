# Practical Use

Shielded Stack is useful when a team needs to operate, publish, or consume Zcash `lightwalletd` service health in a repeatable way.

Local tests only prove that the code works. Practical use starts when real `lightwalletd` endpoints are added to the operations registry and the status workflow probes them on a schedule.

## Who Uses It

- Wallet teams can choose a healthy `lightwalletd` endpoint instead of hard-coding one server.
- Exchanges and payment services can monitor the light client endpoint they depend on for shielded wallet support.
- Infrastructure operators can prove their endpoint is reachable, synced, and responding to the expected gRPC method.
- Community maintainers can keep a public registry of active, maintenance, and retired endpoints.

## What It Provides

- A public YAML registry of known endpoints.
- A Rust CLI that validates, probes, and benchmarks those endpoints.
- A Go exporter that turns endpoint checks into Prometheus metrics.
- A JSON status file that applications, dashboards, or scripts can consume.
- Deployment examples for Docker, Kubernetes, Helm, Prometheus, and Grafana.

## Real Adoption Flow

1. An operator runs `zcashd` and `lightwalletd`.
2. The operator exposes a public `lightwalletd` gRPC URL, usually on port `9067`.
3. The operator adds that URL to `shielded-stack-ops/endpoints/mainnet.yaml` or `testnet.yaml`.
4. `ssctl registry validate` checks the endpoint metadata.
5. `ssctl registry status` probes active endpoints with `GetLightdInfo`.
6. `status/current.json` becomes the current machine-readable view of endpoint health.
7. Wallets, service backends, monitoring systems, or dashboards consume that status data.

## Practical Test With A Real Endpoint

Replace the URL with a real public `lightwalletd` URL:

```sh
cargo run --manifest-path rust/Cargo.toml -p ssctl -- \
  health https://example.com:9067 --json
```

A useful response has:

- `reachable: true`
- a recent `latest_block_height`
- a small `height_lag` when compared with `estimated_block_height`
- `message: "ok"`

If the URL is a placeholder, private host, dead server, or non-`lightwalletd` service, the probe should fail. That failure is expected and useful because it prevents bad endpoints from being published as healthy.

## How Applications Consume It

The lowest-friction integration point is `shielded-stack-ops/status/current.json`.

An application can:

- load the JSON file over HTTPS from GitHub raw content or a hosted mirror
- filter services where `status` is `active` and `reachable` is `true`
- prefer low `latency_ms` and low `height_lag`
- fall back to another endpoint when the chosen endpoint becomes unreachable

Example selection logic:

```text
active + reachable + lowest height_lag + lowest latency_ms
```

That gives applications a simple, auditable way to avoid stale or dead `lightwalletd` servers.

## How Operators Consume It

Operators can run the Go exporter next to their monitoring stack:

```sh
SHIELDED_STACK_ENDPOINTS=https://example.com:9067 \
go run ./go/cmd/lwd-exporter
```

Then Prometheus scrapes:

```text
http://localhost:9467/metrics
```

Grafana can use the dashboard in `dashboards/lightwalletd-overview.json` to show reachability, latency, reported block height, estimated height, and height lag.

## Why This Is Useful

Zcash light wallets and services rely on `lightwalletd` to retrieve compact blocks and submit transactions without operating a full wallet stack in every client. If the endpoint is stale, overloaded, misconfigured, or offline, users see wallet sync failures and poor reliability.

This project turns endpoint health into public, reviewable, machine-readable data.

## Current Limitation

The operations registry is intentionally empty until real public endpoints are added. With an empty registry, validation and status generation can pass, but the status file will show zero services. The next practical milestone is to onboard at least one mainnet and one testnet endpoint.
