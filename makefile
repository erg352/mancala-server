.PHONY: build-frontend
.PHONY: run
.PHONY: run-release

build-frontend:
	cd frontend && npm install && npm run build && rm -rf ../static && mkdir ../static && cp -r ./dist/* ../static

run:
	make build-frontend
	cargo run -- --log TRACE --port 6969

run-release:
	make build-frontend
	cargo run --release -- --port 6969
