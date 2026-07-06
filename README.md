# amar

> Calcule une marée astronomique hors ligne près d'une station NOAA connue, avec
> datum, source, confiance et refus explicite hors couverture.

amar v0.1 est le « curl de la vision » : on lance un serveur local, on envoie
`lat/lon/datetime`, et la réponse donne soit une hauteur traçable, soit un refus
utile.

## Installation

Depuis un clone du dépôt :

```bash
make release && mkdir -p ~/.local/bin ~/.local/share/amar/packs && install -m 755 dist/amar ~/.local/bin/amar && cp dist/packs/noaa_m0.json ~/.local/share/amar/packs/noaa_m0.json
```

Lancer le serveur :

```bash
amar serve --pack ~/.local/share/amar/packs/noaa_m0.json --addr 127.0.0.1:3000
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
  "height_m": 0.76,
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

### 2. Brest est refusé, volontairement

Brest n'est pas calculé en M1. Le serveur refuse plutôt que d'inventer une
hauteur hors zone, et indique la station embarquée la plus proche. Le calcul
Brest expérimental est prévu pour M2.

```bash
curl -i -H 'content-type: application/json' \
  -d '{"lat":48.383,"lon":-4.495,"datetime":"2026-08-15T12:00:00Z"}' \
  http://127.0.0.1:3000/tide
```

```json
{
  "error": "no_supported_source",
  "message": "no supported source within 20.0 km; nearest station is noaa:8410140 Eastport at 4652.019 km",
  "max_distance_km": 20.0,
  "nearest_source": {
    "kind": "station",
    "id": "noaa:8410140",
    "name": "Eastport",
    "distance_km": 4652.019,
    "data_version": "2026-07-06"
  }
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
| `POST /tide` | Hauteur instantanée près d'une station supportée |
| `GET /health` | Version, nombre de stations, version du pack |
| `GET /coverage` | Stations embarquées et rayon accepté |

La couverture par défaut est limitée à 20 km autour de chaque station.

Barème de confiance M1 :

| Distance | Grade | Sigma |
|---:|---|---:|
| <= 2 km | A | 8 cm |
| <= 10 km | B | 15 cm |
| <= 20 km | C | 30 cm |

Cette confiance est une heuristique de distance, pas une calibration
empirique. La méthode exposée est
`station_harmonics_v0_distance_heuristic`.

## CLI

Le même binaire garde l'usage CLI M0 :

```bash
amar tide --lat 37.806 --lon -122.465 --at 2026-08-15T12:00:00Z --pack ~/.local/share/amar/packs/noaa_m0.json
```

Depuis le dépôt :

```bash
make m0-validate
make m1-smoke
```

Commandes disponibles :

| Commande | Rôle |
|---|---|
| `amar tide --lat <deg> --lon <deg> --at <utc>` | Calcule une hauteur instantanée |
| `amar serve --addr 127.0.0.1:3000` | Sert l'API locale |
| `amar validate` | Affiche le p95 vs prédictions NOAA |
| `amar pack-noaa` | Compile les fixtures NOAA brutes en pack |

## Données

Le pack M1 contient 8 stations NOAA harmoniques : Boston, San Francisco,
Pensacola, Seattle, Eastport, Honolulu, Key West et Galveston Pier 21.

Les fixtures, URLs d'origine et checksums sont listés dans
[`DATA_LICENSES.md`](DATA_LICENSES.md).

## Développement

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
make m0-validate
make m1-smoke
```

## Licence

Code sous licence Apache-2.0. Les données NOAA incluses sont dans le domaine
public des États-Unis ; voir [`DATA_LICENSES.md`](DATA_LICENSES.md).
