.PHONY: build test fmt lint rust-build rust-test go-build go-test

build: rust-build go-build

test: rust-test go-test

fmt:
	cargo fmt --manifest-path rust/Cargo.toml
	gofmt -w go

lint:
	cargo check --manifest-path rust/Cargo.toml
	go test ./go/...

rust-build:
	cargo build --manifest-path rust/Cargo.toml

rust-test:
	cargo test --manifest-path rust/Cargo.toml

go-build:
	go build -o bin/lwd-exporter ./go/cmd/lwd-exporter

go-test:
	go test ./go/...

