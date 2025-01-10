.PHONY: run
.PHONY: run-release

run:
	cd frontend && npm i && npm run build && rm -rf ../static && mkdir ../static && cp -r ./dist/* ../static
	cargo run -- --log TRACE --port 6969

run-release:
	cd frontend && npm i && npm run build && rm -rf ../static && mkdir ../static && cp -r ./dist/* ../static
	cargo run --release -- --port 6969
