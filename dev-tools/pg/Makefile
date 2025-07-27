.PHONY: start
start:
	pnpm exec pge start --user admin --password admin app_dev app_test

.PHONY: clean
clean:
	rm -rf .pg-data
	rm -rf node_modules/

.PHONY: setup
setup:
	pnpm install
