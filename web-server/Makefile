.PHONY: dev
dev:
	cargo watch -s 'make dev-exec'


.PHONY: dev-exec
dev-exec:
	clear
	cargo build
	RUST_LOG=debug cargo run


.PHONY: dev-call
dev-call:
	cargo watch -s 'make dev-call-exec'

.PHONY: dev-call-exec
dev-call-exec:
	clear
	sleep 0.5
	wait-for-port 3000 && curl http://localhost:3000/

.PHONY: curl-it
curl-it:
	wait-for-port 3000 && curl http://localhost:3000/
