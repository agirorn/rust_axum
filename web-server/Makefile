.PHONY: clean
clean:
	cargo clean

.PHONY: build
build:
	cargo build

.PHONY: dev
dev:
	cargo watch -s 'make dev-exec'

.PHONY: dev-exec
dev-exec:
	clear
	cargo build
	RUST_LOG=debug cargo run --release

.PHONY: dev-call
dev-call:
	cargo watch -s 'make dev-call-exec'

.PHONY: dev-call-exec
dev-call-exec:
	clear
	sleep 0.5
	wait-for-port 3000 && make curl

.PHONY: curl-it
curl-it:
	wait-for-port 3000 && make curl

.PHONY: test-curl
test-curl:
	curl http://localhost:3000/
