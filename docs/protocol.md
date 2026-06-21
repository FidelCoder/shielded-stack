# Protocol Codegen

The Rust `lwd-client` crate generates protocol bindings from vendored Zcash light wallet protobuf files. The Go exporter uses vendored generated bindings under `go/walletrpc`.

## Source Files

```text
proto/walletrpc/service.proto
proto/walletrpc/compact_formats.proto
```

These files come from the public `zcash/lightwallet-protocol` repository.

## Rust Build Flow

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

## Go Bindings

Generated Go files live under:

```text
go/walletrpc/service.pb.go
go/walletrpc/service_grpc.pb.go
go/walletrpc/compact_formats.pb.go
```

They mirror the public lightwalletd protocol bindings and are used by `lwd-exporter`.

## Refreshing Protobufs

When refreshing these files, update both protobuf files together and run:

```sh
cargo test --manifest-path rust/Cargo.toml
```
