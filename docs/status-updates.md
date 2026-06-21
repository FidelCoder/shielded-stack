# Status Updates

`ssctl registry status` generates a machine-readable status document from one or more endpoint registries.

## Generate Status

From the `shielded-stack` repository:

```sh
cargo run --manifest-path rust/Cargo.toml -p ssctl -- \
  registry status \
  ../shielded-stack-ops/endpoints/mainnet.yaml \
  ../shielded-stack-ops/endpoints/testnet.yaml \
  --output ../shielded-stack-ops/status/current.json
```

Without `--output`, the status document is printed to stdout.

## Behavior

- Active endpoints are probed with `GetLightdInfo`.
- Maintenance and retired endpoints are included but not probed.
- `height_lag` is calculated as estimated height minus reported height.
- Empty registries produce a valid status file with zero services.
