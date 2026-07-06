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
	mkdir -p fixtures/noaa/8443970 fixtures/noaa/9414290 fixtures/noaa/8729840 fixtures/noaa/9447130
	curl -fsSL -o fixtures/noaa/8443970/station.json '$(NOAA_MDAPI)/8443970.json?expand=details'
	curl -fsSL -o fixtures/noaa/8443970/datums.json '$(NOAA_MDAPI)/8443970/datums.json?units=metric'
	curl -fsSL -o fixtures/noaa/8443970/harcon.json '$(NOAA_MDAPI)/8443970/harcon.json?units=metric'
	curl -fsSL -o fixtures/noaa/8443970/predictions_2026-08-15_2026-08-21.json '$(NOAA_DATAGETTER)?product=predictions&application=amar&begin_date=20260815&end_date=20260821&datum=MLLW&station=8443970&time_zone=gmt&units=metric&interval=h&format=json'
	curl -fsSL -o fixtures/noaa/8443970/predictions_2031-08-15_2031-08-21.json '$(NOAA_DATAGETTER)?product=predictions&application=amar&begin_date=20310815&end_date=20310821&datum=MLLW&station=8443970&time_zone=gmt&units=metric&interval=h&format=json'
	curl -fsSL -o fixtures/noaa/8443970/predictions_2036-08-15_2036-08-21.json '$(NOAA_DATAGETTER)?product=predictions&application=amar&begin_date=20360815&end_date=20360821&datum=MLLW&station=8443970&time_zone=gmt&units=metric&interval=h&format=json'
	curl -fsSL -o fixtures/noaa/9414290/station.json '$(NOAA_MDAPI)/9414290.json?expand=details'
	curl -fsSL -o fixtures/noaa/9414290/datums.json '$(NOAA_MDAPI)/9414290/datums.json?units=metric'
	curl -fsSL -o fixtures/noaa/9414290/harcon.json '$(NOAA_MDAPI)/9414290/harcon.json?units=metric'
	curl -fsSL -o fixtures/noaa/9414290/predictions_2026-08-15_2026-08-21.json '$(NOAA_DATAGETTER)?product=predictions&application=amar&begin_date=20260815&end_date=20260821&datum=MLLW&station=9414290&time_zone=gmt&units=metric&interval=h&format=json'
	curl -fsSL -o fixtures/noaa/9414290/predictions_2031-08-15_2031-08-21.json '$(NOAA_DATAGETTER)?product=predictions&application=amar&begin_date=20310815&end_date=20310821&datum=MLLW&station=9414290&time_zone=gmt&units=metric&interval=h&format=json'
	curl -fsSL -o fixtures/noaa/9414290/predictions_2036-08-15_2036-08-21.json '$(NOAA_DATAGETTER)?product=predictions&application=amar&begin_date=20360815&end_date=20360821&datum=MLLW&station=9414290&time_zone=gmt&units=metric&interval=h&format=json'
	curl -fsSL -o fixtures/noaa/8729840/station.json '$(NOAA_MDAPI)/8729840.json?expand=details'
	curl -fsSL -o fixtures/noaa/8729840/datums.json '$(NOAA_MDAPI)/8729840/datums.json?units=metric'
	curl -fsSL -o fixtures/noaa/8729840/harcon.json '$(NOAA_MDAPI)/8729840/harcon.json?units=metric'
	curl -fsSL -o fixtures/noaa/8729840/predictions_2026-08-15_2026-08-21.json '$(NOAA_DATAGETTER)?product=predictions&application=amar&begin_date=20260815&end_date=20260821&datum=MLLW&station=8729840&time_zone=gmt&units=metric&interval=h&format=json'
	curl -fsSL -o fixtures/noaa/8729840/predictions_2031-08-15_2031-08-21.json '$(NOAA_DATAGETTER)?product=predictions&application=amar&begin_date=20310815&end_date=20310821&datum=MLLW&station=8729840&time_zone=gmt&units=metric&interval=h&format=json'
	curl -fsSL -o fixtures/noaa/8729840/predictions_2036-08-15_2036-08-21.json '$(NOAA_DATAGETTER)?product=predictions&application=amar&begin_date=20360815&end_date=20360821&datum=MLLW&station=8729840&time_zone=gmt&units=metric&interval=h&format=json'
	curl -fsSL -o fixtures/noaa/9447130/station.json '$(NOAA_MDAPI)/9447130.json?expand=details'
	curl -fsSL -o fixtures/noaa/9447130/datums.json '$(NOAA_MDAPI)/9447130/datums.json?units=metric'
	curl -fsSL -o fixtures/noaa/9447130/harcon.json '$(NOAA_MDAPI)/9447130/harcon.json?units=metric'
	curl -fsSL -o fixtures/noaa/9447130/predictions_2026-08-15_2026-08-21.json '$(NOAA_DATAGETTER)?product=predictions&application=amar&begin_date=20260815&end_date=20260821&datum=MLLW&station=9447130&time_zone=gmt&units=metric&interval=h&format=json'
	curl -fsSL -o fixtures/noaa/9447130/predictions_2031-08-15_2031-08-21.json '$(NOAA_DATAGETTER)?product=predictions&application=amar&begin_date=20310815&end_date=20310821&datum=MLLW&station=9447130&time_zone=gmt&units=metric&interval=h&format=json'
	curl -fsSL -o fixtures/noaa/9447130/predictions_2036-08-15_2036-08-21.json '$(NOAA_DATAGETTER)?product=predictions&application=amar&begin_date=20360815&end_date=20360821&datum=MLLW&station=9447130&time_zone=gmt&units=metric&interval=h&format=json'

pack-noaa: fetch-noaa
	cargo run -p amar -- pack-noaa --fixtures fixtures/noaa --out data/packs/noaa_m0.json --extracted-at 2026-07-06

m0-validate:
	cargo run -p amar -- validate --pack data/packs/noaa_m0.json --fixtures fixtures/noaa
