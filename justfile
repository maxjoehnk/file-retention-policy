#!/usr/bin/env just --justfile

release:
  cargo build --release

release-musl:
  cargo build --release --target x86_64-unknown-linux-musl

lint:
  cargo clippy

run:
  cargo run

test:
	cargo test

example-consul: (simulate-example "/var/backups/consul" "examples/consul.lines")

example-postgres: (simulate-example "/var/backups/pg" "examples/postgres.lines")

example-hub: (simulate-example "/var/backups/hub" "examples/hub.lines")

example-upsource: (simulate-example "/var/backups/upsource" "examples/upsource.lines")

[private]
simulate-example path lines:
    cargo run -- --config examples/config.toml simulate {{ path }} --input {{ lines }}
