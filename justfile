#!/usr/bin/env just --justfile

release:
  cargo build --release    

lint:
  cargo clippy

run:
  cargo run

example-consul: (simulate-example "/var/backups/consul" "examples/consul.lines")

example-postgres: (simulate-example "/var/backups/pg" "examples/postgres.lines")

example-hub: (simulate-example "/var/backups/hub" "examples/hub.lines")

example-upsource: (simulate-example "/var/backups/upsource" "examples/upsource.lines")

[private]
simulate-example path lines:
    cargo run -- --config examples/config.toml simulate {{ path }} --input {{ lines }}
