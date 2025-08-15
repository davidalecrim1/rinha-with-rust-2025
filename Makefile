TS := $(shell date '+%Y%m%d_%H%M%S')
EXPORT_FILE := reports/report_$(TS).html

build:
	cargo build --bin api
	cargo build --bin worker

run:
	RUST_LOG=debug cargo run --bin api

simple-test-1:
	curl --unix-socket /tmp/hyperlocal.sock http://localhost/payments

simple-test-2:
	curl --unix-socket /tmp/hyperlocal.sock http://localhost/payments-summary

run-processor:
	docker compose -f rinha-docker-compose-arm64.yml up -d

run-docker:
	make run-processor && docker compose up --build -d

load-test:
	K6_WEB_DASHBOARD=true \
	K6_WEB_DASHBOARD_PORT=5665 \
	K6_WEB_DASHBOARD_OPEN=true \
	K6_WEB_DASHBOARD_EXPORT="$(EXPORT_FILE)" \
	k6 run ./rinha-source/rinha-test/rinha.js

super-load-test:
	K6_WEB_DASHBOARD=true \
	K6_WEB_DASHBOARD_PORT=5665 \
	K6_WEB_DASHBOARD_OPEN=true \
	K6_WEB_DASHBOARD_EXPORT="$(EXPORT_FILE)" \
	k6 run -e MAX_REQUESTS=850 ./rinha-source/rinha-test/rinha.js