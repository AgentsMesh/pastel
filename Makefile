.PHONY: build test lint fmt check install clean

build:
	cargo build --release

test:
	cargo test --workspace

lint:
	cargo clippy --workspace -- -D warnings

fmt:
	cargo fmt --all

check:
	cargo check --workspace

install: build
	cp target/release/pastel /usr/local/bin/

clean:
	cargo clean
