.PHONY: setup
setup:
	cd dev-tools/pg && make setup

.PHONY: clean
clean:
	cd dev-tools/pg && make clean

.PHONY: start
start:
	mprocs

.PHONY: build
build:
	@echo "==>> features"

.PHONY: start-pg
start-pg:
	cd dev-tools/pg && make start

.PHONY: migrate
migrate:
	pgmt migrate --url postgres://admin:admin@localhost:5432/app_dev  ./migrations
