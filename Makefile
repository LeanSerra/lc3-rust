.PHONY: run build build_release run_release lint clean test

run:
	cargo run

build:
	cargo build

build_release:
	cargo build --release

run_release:
	cargo run --release

lint:
	cargo clippy

clean:
	cargo clean

test:
	cargo test
