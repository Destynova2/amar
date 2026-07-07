NOAA_API = https://api.tidesandcurrents.noaa.gov
NOAA_MDAPI = $(NOAA_API)/mdapi/prod/webapi/stations
NOAA_DATAGETTER = $(NOAA_API)/api/prod/datagetter
STATIONS := $(shell awk 'NF { print $$1 }' data/stations.txt)
NOAA_STATION_COUNT := $(words $(STATIONS))
BREST_PACK = data/packs/amar-data-brest-experimental.json
BREST_STATION_COUNT := $(shell test -f $(BREST_PACK) && printf 1 || printf 0)
FRANCE_PACK = data/packs/amar-data-france-experimental.json
FRANCE_STATION_COUNT := $(shell test -f $(FRANCE_PACK) && grep -c '"station_id"' $(FRANCE_PACK) || printf 0)
STATION_COUNT := $(shell expr $(NOAA_STATION_COUNT) + $(BREST_STATION_COUNT) + $(FRANCE_STATION_COUNT))
CONTAINER_ENGINE ?= $(shell if command -v docker >/dev/null 2>&1; then printf docker; elif command -v podman >/dev/null 2>&1; then printf podman; fi)
CONTAINER_IMAGE ?= amar:local
CONTAINER_PORT ?= 3000
HOST_TARGET := $(shell rustc -vV | awk '/^host:/ { print $$2 }')
TARGET ?=
RELEASE_TARGET := $(if $(TARGET),$(TARGET),$(HOST_TARGET))
TAG ?= $(shell git describe --tags --exact-match 2>/dev/null || git describe --tags --abbrev=0 2>/dev/null || printf v0.0.0-dev)
DIST_DIR ?= dist
DIST_ROOT ?= dist
DIST_NAME := amar-$(TAG)-$(TARGET)
DIST_STAGE := $(DIST_ROOT)/$(DIST_NAME)
DIST_TARBALL := $(DIST_ROOT)/$(DIST_NAME).tar.gz
CARGO_BUILD ?= cargo build
TARGET_ARG := $(if $(TARGET),--target $(TARGET),)
RELEASE_BIN := $(if $(TARGET),target/$(TARGET)/release/amar,target/release/amar)
YEARS = 2026 2031 2036
HILO_YEARS = 2026
HILO_DRIFT_YEARS = 2031
HILO_DRIFT_STATIONS = 9447130 8410140

.PHONY: fmt clippy test fetch-noaa fetch-noaa-hilo check-noaa-fixtures pack-noaa fetch-refmar build-brest-pack calibrate-france m0-validate m2-benchmark m3-check release dist-tarball m1-smoke container container-smoke

fmt:
	cargo fmt --all --check

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

test:
	cargo test --workspace

fetch-noaa:
	for station in $(STATIONS); do \
		mkdir -p fixtures/noaa/$$station; \
		curl -fsSL -o fixtures/noaa/$$station/station.json "$(NOAA_MDAPI)/$$station.json?expand=details"; \
		curl -fsSL -o fixtures/noaa/$$station/datums.json "$(NOAA_MDAPI)/$$station/datums.json?units=metric"; \
		curl -fsSL -o fixtures/noaa/$$station/harcon.json "$(NOAA_MDAPI)/$$station/harcon.json?units=metric"; \
		for year in $(YEARS); do \
			curl -fsSL -o fixtures/noaa/$$station/predictions_$${year}-08-15_$${year}-08-21.json "$(NOAA_DATAGETTER)?product=predictions&application=amar&begin_date=$${year}0815&end_date=$${year}0821&datum=MLLW&station=$$station&time_zone=gmt&units=metric&interval=h&format=json"; \
		done; \
	done

fetch-noaa-hilo:
	for station in $(STATIONS); do \
		mkdir -p fixtures/noaa/$$station; \
		for year in $(HILO_YEARS); do \
			curl -fsSL -o fixtures/noaa/$$station/hilo_$${year}-08-15_$${year}-08-21.json "$(NOAA_DATAGETTER)?product=predictions&application=amar&begin_date=$${year}0815&end_date=$${year}0821&datum=MLLW&station=$$station&time_zone=gmt&units=metric&interval=hilo&format=json"; \
		done; \
	done
	for station in $(HILO_DRIFT_STATIONS); do \
		mkdir -p fixtures/noaa/$$station; \
		for year in $(HILO_DRIFT_YEARS); do \
			curl -fsSL -o fixtures/noaa/$$station/hilo_$${year}-08-15_$${year}-08-21.json "$(NOAA_DATAGETTER)?product=predictions&application=amar&begin_date=$${year}0815&end_date=$${year}0821&datum=MLLW&station=$$station&time_zone=gmt&units=metric&interval=hilo&format=json"; \
		done; \
	done

check-noaa-fixtures:
	@set -eu; \
	missing=0; \
	for station in $(STATIONS); do \
		for file in station.json datums.json harcon.json; do \
			if [ ! -f fixtures/noaa/$$station/$$file ]; then \
				echo "missing fixtures/noaa/$$station/$$file"; \
				missing=1; \
			fi; \
		done; \
		for year in $(YEARS); do \
			file=fixtures/noaa/$$station/predictions_$${year}-08-15_$${year}-08-21.json; \
			if [ ! -f $$file ]; then \
				echo "missing $$file"; \
				missing=1; \
			fi; \
		done; \
	done; \
	if [ "$$missing" != "0" ]; then \
		echo "NOAA fixtures are missing; run make fetch-noaa"; \
		exit 1; \
	fi

pack-noaa: check-noaa-fixtures
	@test -n "$(EXTRACTED_AT)" || { echo "EXTRACTED_AT is required: make pack-noaa EXTRACTED_AT=YYYY-MM-DD"; exit 1; }
	cargo run -p amar -- pack-noaa --fixtures fixtures/noaa --out data/packs/noaa_m0.json --extracted-at "$(EXTRACTED_AT)" $(foreach station,$(STATIONS),--station $(station))

fetch-refmar:
	cargo run -p amar-calibrate -- fetch-refmar

build-brest-pack:
	cargo run -p amar-calibrate -- build-brest-pack

calibrate-france:
	cargo run -p amar-calibrate -- calibrate-france

m0-validate:
	cargo run -p amar -- validate --pack data/packs/noaa_m0.json --fixtures fixtures/noaa

m2-benchmark:
	cargo run -p amar -- benchmark-brest --brest-p95-limit-cm 19 --p95-limit-cm 40 --min-rms-factor 2

m3-check: test m0-validate m2-benchmark
	cargo run -p amar -- validate-hilo --pack data/packs/noaa_m0.json --fixtures fixtures/noaa

release:
	CARGO_PROFILE_RELEASE_STRIP=symbols $(CARGO_BUILD) --release --locked -p amar $(TARGET_ARG)
	install -d "$(DIST_DIR)/packs"
	install -m 755 "$(RELEASE_BIN)" "$(DIST_DIR)/amar"
	cp data/packs/noaa_m0.json "$(DIST_DIR)/packs/noaa_m0.json"
	cp data/packs/amar-data-brest-experimental.json "$(DIST_DIR)/packs/amar-data-brest-experimental.json"
	cp data/packs/amar-data-france-experimental.json "$(DIST_DIR)/packs/amar-data-france-experimental.json"
	cp LICENSE "$(DIST_DIR)/LICENSE"
	cp LIMITATIONS.md "$(DIST_DIR)/LIMITATIONS.md"
	printf '%s\n' \
		'# Installation' \
		'' \
		'Cette archive contient le binaire amar et les trois packs JSON versionnés :' \
		'' \
		'- packs/noaa_m0.json' \
		'- packs/amar-data-brest-experimental.json' \
		'- packs/amar-data-france-experimental.json' \
		'' \
		'Depuis ce répertoire extrait :' \
		'' \
		'```bash' \
		'./amar serve' \
		'```' \
		'' \
		'Commande explicite équivalente :' \
		'' \
		'```bash' \
		'./amar serve --pack packs/noaa_m0.json --pack packs/amar-data-brest-experimental.json --pack packs/amar-data-france-experimental.json --addr 127.0.0.1:3000' \
		'```' \
		'' \
		'Smoke CLI :' \
		'' \
		'```bash' \
		'./amar tide --pack packs/noaa_m0.json --lat 37.806 --lon -122.465 --at 2026-08-15T12:00:00Z' \
		'```' \
		> "$(DIST_DIR)/install.md"

dist-tarball:
	@test -n "$(TARGET)" || { echo "TARGET is required: make dist-tarball TARGET=<triple>"; exit 1; }
	rm -rf "$(DIST_STAGE)"
	$(MAKE) release TARGET="$(TARGET)" TAG="$(TAG)" DIST_DIR="$(DIST_STAGE)" CARGO_BUILD="$(CARGO_BUILD)"
	tar -C "$(DIST_ROOT)" -czf "$(DIST_TARBALL)" "$(DIST_NAME)"
	@printf '%s\n' "$(DIST_TARBALL)"

m1-smoke:
	cargo build -p amar
	@set -eu; \
	LOG=$$(mktemp); \
	BODY=$$(mktemp); \
	BASE=; \
	target/debug/amar serve --addr 127.0.0.1:0 >$$LOG 2>&1 & \
	PID=$$!; \
	trap 'kill $$PID >/dev/null 2>&1 || true; wait $$PID >/dev/null 2>&1 || true; rm -f $$LOG $$BODY' EXIT; \
	for attempt in 1 2 3 4 5 6 7 8 9 10; do \
		BASE=$$(sed -n 's/^amar serve listening on //p' $$LOG); \
		if [ -n "$$BASE" ]; then break; fi; \
		if ! kill -0 $$PID >/dev/null 2>&1; then cat $$LOG; exit 1; fi; \
		sleep 0.1; \
	done; \
	test -n "$$BASE"; \
	READY=0; \
	for attempt in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39 40; do \
		if curl -fsS $$BASE/health >$$BODY 2>/dev/null; then READY=1; break; fi; \
		sleep 0.1; \
	done; \
	if [ "$$READY" != "1" ]; then cat $$LOG; exit 1; fi; \
	grep -q '"version"' $$BODY; \
	grep -q '"station_count":$(STATION_COUNT)' $$BODY; \
	CODE=$$(curl -sS -o $$BODY -w '%{http_code}' $$BASE/coverage); \
	test "$$CODE" = "200"; \
	grep -q '"stations"' $$BODY; \
	grep -q '"noaa:9414290"' $$BODY; \
	CODE=$$(curl -sS -o $$BODY -w '%{http_code}' -H 'content-type: application/json' -d '{"lat":37.806,"lon":-122.465,"datetime":"2026-08-15T12:00:00Z"}' $$BASE/tide); \
	test "$$CODE" = "200"; \
	grep -q '"height_m"' $$BODY; \
	grep -q '"id":"noaa:9414290"' $$BODY; \
	grep -q '"method":"station_harmonics_v0_distance_heuristic"' $$BODY; \
	grep -q '"astronomical_tide_only"' $$BODY; \
	CODE=$$(curl -sS -o $$BODY -w '%{http_code}' -H 'content-type: application/json' -d '{"lat":48.383,"lon":-4.495,"datetime":"2026-08-15T12:00:00Z"}' $$BASE/tide); \
	test "$$CODE" = "200"; \
	grep -q '"height_m"' $$BODY; \
	grep -q '"id":"refmar:3"' $$BODY; \
	grep -q '"method":"calibrated_station_experimental"' $$BODY; \
	grep -q '"residual_benchmark_cm":' $$BODY; \
	grep -q '"experimental"' $$BODY; \
	grep -q '"not_shom"' $$BODY; \
	CODE=$$(curl -sS -o $$BODY -w '%{http_code}' -H 'content-type: application/json' -d '{"lat":91,"lon":0,"datetime":"2026-08-15T12:00:00Z"}' $$BASE/tide); \
	test "$$CODE" = "400"; \
	grep -q '"error":"invalid_request"' $$BODY; \
	grep -q 'latitude must be between -90 and 90 degrees' $$BODY

container:
	@test -n "$(CONTAINER_ENGINE)" || { echo "docker or podman is required"; exit 1; }
	$(CONTAINER_ENGINE) build -f Containerfile -t $(CONTAINER_IMAGE) .

container-smoke: container
	@test -n "$(CONTAINER_ENGINE)" || { echo "docker or podman is required"; exit 1; }
	@set -eu; \
	BODY=$$(mktemp); \
	CID=; \
	trap 'if [ -n "$$CID" ]; then $(CONTAINER_ENGINE) stop $$CID >/dev/null 2>&1 || true; fi; rm -f $$BODY' EXIT; \
	CID=$$($(CONTAINER_ENGINE) run --rm -d -p 127.0.0.1:$(CONTAINER_PORT):3000 $(CONTAINER_IMAGE)); \
	READY=0; \
	for attempt in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39 40; do \
		if curl -fsS http://127.0.0.1:$(CONTAINER_PORT)/health >$$BODY 2>/dev/null; then READY=1; break; fi; \
		sleep 0.25; \
	done; \
	if [ "$$READY" != "1" ]; then $(CONTAINER_ENGINE) logs $$CID || true; exit 1; fi; \
	grep -q '"station_count":$(STATION_COUNT)' $$BODY; \
	CODE=$$(curl -sS -o $$BODY -w '%{http_code}' -H 'content-type: application/json' -d '{"lat":37.806,"lon":-122.465,"datetime":"2026-08-15T12:00:00Z"}' http://127.0.0.1:$(CONTAINER_PORT)/tide); \
	test "$$CODE" = "200"; \
	grep -q '"height_m"' $$BODY; \
	grep -q '"id":"noaa:9414290"' $$BODY
