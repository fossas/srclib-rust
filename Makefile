install: build
	mkdir -p .bin
	cp target/release/srclib-rust .bin/

build:
	cargo build --release

prune:
	cargo clean

# This just enforces that the toolchain actually runs against a repo.
test: docker
	docker run --rm srclib-rust:test

docker:
	docker build -t srclib-rust:test .