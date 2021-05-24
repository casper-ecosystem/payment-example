prepare:
	rustup target add wasm32-unknown-unknown

build-contract:
	cd contract && cargo build --release --target wasm32-unknown-unknown

test-payment:
	mkdir -p tests/wasm
	cp contract/target/wasm32-unknown-unknown/release/wallet_contract.wasm tests/wasm/
	cp contract/target/wasm32-unknown-unknown/release/send_tokens.wasm tests/wasm/
	cp contract/target/wasm32-unknown-unknown/release/collect.wasm tests/wasm/

	cd tests && cargo test -- --nocapture

test: build-contract test-payment

clippy:
	cd contract && cargo clippy --all-targets --all -- -D warnings -A renamed_and_removed_lints
	cd tests && cargo clippy

check-lint: clippy
	cd contract && cargo fmt --all -- --check

lint: clippy
	cd contract && cargo fmt --all

format:
	cd contract && cargo fmt 
	cd tests && cargo fmt

clean:
	cd contract && cargo clean
	cd tests && cargo clean
	rm tests/wasm/*.wasm