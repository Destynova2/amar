NOAA_API = https://api.tidesandcurrents.noaa.gov
NOAA_MDAPI = $(NOAA_API)/mdapi/prod/webapi/stations
NOAA_DATAGETTER = $(NOAA_API)/api/prod/datagetter
STATIONS = 8443970 9414290 8729840 9447130
YEARS = 2026 2031 2036

.PHONY: fmt clippy test fetch-noaa pack-noaa m0-validate

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
