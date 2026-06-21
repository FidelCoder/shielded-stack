# Containers

The `Container` GitHub Actions workflow builds and publishes the `lwd-exporter` image to GitHub Container Registry.

## Image

```text
ghcr.io/fidelcoder/shielded-stack/lwd-exporter
```

Tags are generated for:

- branch names
- version tags such as `v0.1.0`
- commit SHA tags such as `sha-<commit>`
- `latest` on the default branch

## Local Build

```sh
docker build -f deploy/docker-compose/lwd-exporter.Dockerfile -t lwd-exporter:local .
```

## Runtime

```sh
docker run --rm -p 9467:9467 \
  -e SHIELDED_STACK_ADDR=:9467 \
  -e SHIELDED_STACK_ENDPOINTS=https://example.com:9067 \
  lwd-exporter:local
```
