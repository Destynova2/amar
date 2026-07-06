NOAA_API = https://api.tidesandcurrents.noaa.gov
NOAA_MDAPI = $(NOAA_API)/mdapi/prod/webapi/stations
NOAA_DATAGETTER = $(NOAA_API)/api/prod/datagetter
STATIONS := $(shell awk 'NF { print $$1 }' data/stations.txt)
STATION_COUNT := $(words $(STATIONS))
YEARS = 2026 2031 2036

.PHONY: fmt clippy test fetch-noaa pack-noaa m0-validate release m1-smoke

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

pack-noaa: fetch-noaa
	cargo run -p amar -- pack-noaa --fixtures fixtures/noaa --out data/packs/noaa_m0.json --extracted-at 2026-07-06 $(foreach station,$(STATIONS),--station $(station))

m0-validate:
	cargo run -p amar -- validate --pack data/packs/noaa_m0.json --fixtures fixtures/noaa

release:
	cargo build --release -p amar
	mkdir -p dist/packs
	install -m 755 target/release/amar dist/amar
	cp data/packs/noaa_m0.json dist/packs/noaa_m0.json
	printf '%s\n' \
		'# Installation' \
		'' \
		'Depuis la racine du dépôt :' \
		'' \
		'```bash' \
		'make release' \
		'mkdir -p ~/.local/bin ~/.local/share/amar/packs && install -m 755 dist/amar ~/.local/bin/amar && cp dist/packs/noaa_m0.json ~/.local/share/amar/packs/noaa_m0.json' \
		'```' \
		'' \
		'Puis lancer le serveur :' \
		'' \
		'```bash' \
		'amar serve --pack ~/.local/share/amar/packs/noaa_m0.json --addr 127.0.0.1:3000' \
		'```' \
		> dist/install.md

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
	test "$$CODE" = "422"; \
	grep -q '"error":"no_supported_source"' $$BODY; \
	grep -q '"nearest_source"' $$BODY; \
	grep -q '"distance_km"' $$BODY; \
	CODE=$$(curl -sS -o $$BODY -w '%{http_code}' -H 'content-type: application/json' -d '{"lat":91,"lon":0,"datetime":"2026-08-15T12:00:00Z"}' $$BASE/tide); \
	test "$$CODE" = "400"; \
	grep -q '"error":"invalid_request"' $$BODY; \
	grep -q 'latitude must be between -90 and 90 degrees' $$BODY
