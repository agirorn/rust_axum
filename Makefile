.PHONY: setup
setup:
	test -e .env || cp .env-development .env
	cd dev-tools/pg && make setup

.PHONY: clean
clean:
	cd dev-tools/pg && make clean
	cd web-server && make clean

.PHONY: start
start:
	mprocs

.PHONY: build
build:
	cd web-server && make build

.PHONY: start-pg
start-pg:
	cd dev-tools/pg && make start

.PHONY: migrate
migrate:
	pgmt migrate --url postgres://admin:admin@localhost:5432/app_dev  ./migrations
