# Registry Validation

`ssctl registry validate` enforces the shared endpoint registry rules used by the operations repository.

## Registry Rules

- `updated_at` must use UTC timestamp form: `YYYY-MM-DDTHH:MM:SSZ`.
- Endpoint IDs must be lowercase ASCII letters, digits, and hyphens.
- Endpoint IDs must not start or end with a hyphen.
- Endpoint IDs must not contain repeated hyphens.
- Endpoint IDs must be unique within a registry.
- Endpoint URLs must be unique within a registry.

## Endpoint URL Rules

- URLs must start with `http://` or `https://`.
- URLs must include a host.
- URLs must include an explicit port.
- URLs must not include a path.
- URLs must not include a query string.
- URLs must not include a fragment.

Valid examples:

```text
https://example.com:9067
http://127.0.0.1:9067
```

Invalid examples:

```text
https://example.com
https://example.com:9067/status
https://example.com:9067?network=mainnet
```

## Capability Rules

Allowed capabilities:

- `grpc`
- `tls`
- `tor`

Active endpoints must declare `grpc`. Capabilities must not be duplicated.
