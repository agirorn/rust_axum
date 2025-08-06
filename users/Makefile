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
	cargo test -- --nocapture
	make build

