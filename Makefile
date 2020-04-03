install: build
	mkdir -p .bin
	cp target/release/srclib-rust .bin/

build:
	cargo build --release

prune:
	cargo clean