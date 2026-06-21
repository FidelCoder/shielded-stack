# Protocol Codegen

The Rust `lwd-client` crate generates protocol bindings from vendored Zcash light wallet protobuf files.

## Source Files

```text
proto/walletrpc/service.proto
proto/walletrpc/compact_formats.proto
```

These files come from the public `zcash/lightwallet-protocol` repository.

## Build Flow

`rust/crates/lwd-client/build.rs` uses `tonic-build` and a vendored `protoc` binary to generate the Rust client at build time. The generated module is included with:

```rust
pub mod rpc {
    tonic::include_proto!("cash.z.wallet.sdk.rpc");
}
```

The MVP health probe uses:

```text
cash.z.wallet.sdk.rpc.CompactTxStreamer/GetLightdInfo
```

## Refreshing Protobufs

When refreshing these files, update both protobuf files together and run:

```sh
cargo test --manifest-path rust/Cargo.toml
```
