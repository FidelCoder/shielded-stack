# Dashboards

Grafana dashboard definitions live in `dashboards/`.

## Lightwalletd Overview

`dashboards/lightwalletd-overview.json` tracks:

- configured endpoint count
- reachable endpoint count
- probe failures
- latency and latency percentiles
- reported block height
- estimated block height
- height lag trends
- per-endpoint health

The latency percentile panels use `quantile_over_time` over recent gauge samples emitted by `lwd-exporter`.
