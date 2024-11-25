.PHONY: run build build_release run_release lint clean test

filename ?= ./2048.obj

run:
	cargo run -- $(filename)

build:
	cargo build

build_release:
	cargo build --release

run_release:
	cargo run --release -- $(filename)

lint:
	cargo clippy

clean:
	cargo clean

test:
	cargo test
