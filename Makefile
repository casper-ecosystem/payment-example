prepare:
	rustup default nightly-2021-06-17-x86_64-unknown-linux-gnu
	rustup target add wasm32-unknown-unknown

rust-test-only:
	cargo test -p tests

copy-wasm-file-to-test:
	cp target/wasm32-unknown-unknown/release/*.wasm tests/wasm

test: build-contract copy-wasm-file-to-test rust-test-only

clippy:
	cargo clippy --all-targets --all -- -D warnings

check-lint: clippy
	cargo fmt --all -- --check

format:
	cargo fmt --all

lint: clippy format

build-contract:
	cargo build --release -p deposit_contract --target wasm32-unknown-unknown

wasm-strip:
	wasm-strip target/wasm32-unknown-unknown/release/deposit_contract.wasm
	wasm-strip target/wasm32-unknown-unknown/release/deposit_session.wasm
	wasm-strip target/wasm32-unknown-unknown/release/deposit_into_session.wasm

clean:
	cargo clean