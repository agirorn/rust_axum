.PHONY: clean
clean:
	cargo clean

.PHONY: build
build:
	cargo build

.PHONY: dev
dev:
	yardman \
		Makefile \
		'src/*.rs' \
		'src/**/*.rs' \
		'tests/*.rs' \
		'../migrations/*.sql' \
		'make dev-exec'

.PHONY: dev-exec
dev-exec:
	clear
	# cargo build
	# /usr/bin/time -p cargo test -- --nocapture
	cargo test -- --nocapture
	make build

