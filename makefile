.PHONY: build-debug
.PHONY: build-release
.PHONY: run-debug
.PHONY: run-release

build-debug:
	cargo build
	(cd frontend && trunk build)


build-release:
	cargo build --release
	(cd frontend && trunk build --release)


run-debug:
	make build-debug
	cargo run -- --port 8080 --database data.db --static-routes frontend/dist --log trace

run-release:
	make build-release
	cargo run release -- --port 8080 --database data.db --static-routes frontend/dist --log warn
