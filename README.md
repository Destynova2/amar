# amar

> Calcule une marée astronomique hors ligne près d'une station connue, avec
> datum, source, confiance et refus explicite hors couverture.

amar v0.1.x ajoute Brest expérimental au socle NOAA, puis M3 ajoute les
prochains PM/BM, les séries bornées et les fenêtres de seuil : on lance un
serveur local, on envoie `lat/lon/datetime`, et la réponse donne soit une
hauteur traçable, soit un refus utile.

## Installation

Depuis un clone du dépôt :

```bash
make release && mkdir -p ~/.local/bin ~/.local/share/amar/packs && install -m 755 dist/amar ~/.local/bin/amar && cp dist/packs/*.json ~/.local/share/amar/packs/
```

Lancer le serveur :

```bash
amar serve --pack ~/.local/share/amar/packs/noaa_m0.json --pack ~/.local/share/amar/packs/amar-data-brest-experimental.json --addr 127.0.0.1:3000
```

Le serveur charge le pack au démarrage, puis fonctionne offline.

## Trois curls

### 1. San Francisco répond une hauteur

San Francisco est à moins de 2 km de la station NOAA `9414290`. La réponse est
donc acceptée, avec grade A et sigma 8 cm.

```bash
curl -i -H 'content-type: application/json' \
  -d '{"lat":37.806,"lon":-122.465,"datetime":"2026-08-15T12:00:00Z"}' \
  http://127.0.0.1:3000/tide
```

```json
{
  "height_m": 0.737,
  "next_high": {
    "height_m": 1.758,
    "t": "2026-08-15T21:33:25Z"
  },
  "next_low": {
    "height_m": 0.043,
    "t": "2026-08-15T14:43:27Z"
  },
  "datum": "MLLW",
  "source": {
    "kind": "station",
    "id": "noaa:9414290",
    "name": "San Francisco",
    "distance_km": 0.085,
    "data_version": "2026-07-06"
  },
  "confidence": {
    "grade": "A",
    "sigma_cm": 8,
    "method": "station_harmonics_v0_distance_heuristic"
  },
  "warnings": [
    "astronomical_tide_only",
    "not_for_navigation",
    "no_weather_surge"
  ]
}
```

### 2. Brest répond en expérimental

Brest utilise un pack expérimental calibré depuis les observations horaires
validées REFMAR (`source=4`, attribution `Shom / REFMAR`). Les constantes sont
dérivées des observations REFMAR, non équivalentes aux constantes SHOM. La
réponse ne porte pas de grade A/B/C : elle expose le p95 du benchmark figé.

```bash
curl -i -H 'content-type: application/json' \
  -d '{"lat":48.383,"lon":-4.495,"datetime":"2026-08-15T12:00:00Z"}' \
  http://127.0.0.1:3000/tide
```

```json
{
  "height_m": 1.09,
  "next_high": {
    "height_m": 7.336,
    "t": "2026-08-15T17:43:06Z"
  },
  "next_low": {
    "height_m": 1.088,
    "t": "2026-08-16T00:09:32Z"
  },
  "datum": "zero_hydrographique_brest",
  "source": {
    "kind": "station",
    "id": "refmar:3",
    "name": "Brest",
    "distance_km": 0.011,
    "data_version": "2026-07-06"
  },
  "confidence": {
    "method": "calibrated_station_experimental",
    "residual_benchmark_cm": 26.6,
    "validation_period": "2026-04-01T00:00:00Z/2026-07-01T00:00:00Z"
  },
  "warnings": [
    "astronomical_tide_only",
    "not_for_navigation",
    "no_weather_surge",
    "experimental",
    "not_shom"
  ]
}
```

### 3. Une entrée invalide renvoie 400

Les coordonnées sont validées avant toute recherche de station.

```bash
curl -i -H 'content-type: application/json' \
  -d '{"lat":91,"lon":0,"datetime":"2026-08-15T12:00:00Z"}' \
  http://127.0.0.1:3000/tide
```

```json
{
  "error": "invalid_request",
  "message": "latitude must be between -90 and 90 degrees"
}
```

## API

| Endpoint | Rôle |
|---|---|
| `POST /tide` | Hauteur instantanée et prochains PM/BM près d'une station supportée |
| `POST /tide/series` | Série `[{t,height_m}]`, `duration_h <= 72`, `step_min >= 6` |
| `POST /tide/windows` | Fenêtres `[{start,end}]` au-dessus ou au-dessous d'un seuil, plage <= 31 jours |
| `GET /health` | Version, nombre de stations, version du pack |
| `GET /coverage` | Stations embarquées et rayon accepté |

La couverture par défaut est limitée à 20 km autour de chaque station.
Un rayon demandé plus large est plafonné à 20 km en M1, afin de garder le
grade C dans son domaine documenté.

Barème de confiance NOAA :

| Distance | Grade | Sigma |
|---:|---|---:|
| <= 2 km | A | 8 cm |
| <= 10 km | B | 15 cm |
| <= 20 km | C | 30 cm |

Cette confiance NOAA est une heuristique de distance, pas une calibration
empirique. La méthode exposée est `station_harmonics_v0_distance_heuristic`.

Pour Brest, `confidence.method` vaut `calibrated_station_experimental` et
`residual_benchmark_cm` mesure le p95 du benchmark hors calibration. Le résidu
= niveau d'eau observé − marée astronomique prédite (météo incluse).

Les réponses `/tide/series` et `/tide/windows` gardent la même forme de
`datum`, `source`, `confidence` et `warnings` que `/tide`.

## CLI

Le même binaire garde l'usage CLI M0 :

```bash
amar tide --lat 37.806 --lon -122.465 --at 2026-08-15T12:00:00Z --pack ~/.local/share/amar/packs/noaa_m0.json
amar tide --lat 48.383 --lon -4.495 --at 2026-08-15T12:00:00Z --pack ~/.local/share/amar/packs/noaa_m0.json --pack ~/.local/share/amar/packs/amar-data-brest-experimental.json
```

Série NOAA de 3 h à San Francisco :

```bash
amar tide --lat 37.806 --lon -122.465 --at 2026-08-15T12:00:00Z --duration-h 3 --step-min 60 --pack ~/.local/share/amar/packs/noaa_m0.json
```

Fenêtre NOAA au-dessus de 1,5 m MLLW :

```bash
amar window --lat 37.806 --lon -122.465 --from 2026-08-15T00:00:00Z --to 2026-08-16T00:00:00Z --above 1.5 --pack ~/.local/share/amar/packs/noaa_m0.json
```

```json
{
  "windows": [
    {
      "start": "2026-08-15T06:40:44Z",
      "end": "2026-08-15T10:11:36Z"
    },
    {
      "start": "2026-08-15T19:53:56Z",
      "end": "2026-08-15T23:10:18Z"
    }
  ]
}
```

Fenêtre de sortie kayak à Brest demain matin dans le calendrier du dépôt
(`2026-07-07`), seuil exprimé au zéro hydrographique de Brest :

```bash
amar window --lat 48.383 --lon -4.495 --from 2026-07-07T04:00:00Z --to 2026-07-07T12:00:00Z --above 4.5 --pack ~/.local/share/amar/packs/noaa_m0.json --pack ~/.local/share/amar/packs/amar-data-brest-experimental.json
```

```json
{
  "windows": [
    {
      "start": "2026-07-07T06:00:46Z",
      "end": "2026-07-07T11:30:21Z"
    }
  ]
}
```

Brest reste expérimental : l'incertitude verticale d'une fenêtre de seuil est
de l'ordre du benchmark, soit `residual_benchmark_cm = 26.6`.

Depuis le dépôt :

```bash
make m0-validate
make m1-smoke
make m2-benchmark
make m3-check
```

Commandes disponibles :

| Commande | Rôle |
|---|---|
| `amar tide --lat <deg> --lon <deg> --at <utc>` | Calcule une hauteur instantanée et les prochains PM/BM |
| `amar tide --lat <deg> --lon <deg> --at <utc> --duration-h <h> --step-min <min>` | Calcule une série bornée |
| `amar window --lat <deg> --lon <deg> --from <utc> --to <utc> --above <m>` | Calcule les fenêtres au-dessus d'un seuil |
| `amar serve --addr 127.0.0.1:3000` | Sert l'API locale |
| `amar validate` | Gate p95 vs prédictions NOAA, exit 1 au-delà de 2 cm |
| `amar validate-hilo` | Gate p95 vs PM/BM NOAA, exit 1 au-delà de 10 min ou 3 cm |
| `amar benchmark-brest` | Rejoue `benchmark_brest_v1` et les deux baselines |
| `amar pack-noaa` | Compile les fixtures NOAA brutes en pack |

## Données

Le pack NOAA contient 8 stations harmoniques : Boston, San Francisco,
Pensacola, Seattle, Eastport, Honolulu, Key West et Galveston Pier 21.

Le pack Brest expérimental contient une seule station (`refmar:3`) au zéro
hydrographique de Brest. Les observations d'entrée couvrent
`2025-01-01T00:00:00Z/2026-07-01T00:00:00Z`; la calibration exclut les trois
derniers mois, réservés à `benchmark_brest_v1`.

Les fixtures, URLs d'origine et checksums sont listés dans
[`DATA_LICENSES.md`](DATA_LICENSES.md).

## Développement

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
make m0-validate
make m1-smoke
make fetch-refmar
make build-brest-pack
make m2-benchmark
make fetch-noaa-hilo
make m3-check
```

## Licence

Code sous licence Apache-2.0. Les données NOAA incluses sont dans le domaine
public des États-Unis. Les observations REFMAR et le pack Brest dérivé sont
sous Licence Ouverte 2.0 Etalab avec attribution `Shom / REFMAR`; voir
[`DATA_LICENSES.md`](DATA_LICENSES.md).
