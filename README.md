# amar

> Calcule une marée astronomique hors ligne près d'une station connue, avec
> datum, source, confiance et refus explicite hors couverture.

amar v0.7 publie des binaires téléchargeables en plus de l'image GHCR. Le
socle couvre NOAA, Brest expérimental et 21 ports REFMAR RONIM,
avec prochains PM/BM, séries bornées, fenêtres de seuil et coefficient de marée
français dérivé de notre Brest calibré. On lance un serveur local, on envoie
`lat/lon/datetime`, et la réponse donne soit une hauteur traçable, soit un
refus utile.

## Installation

### 1. Binaire GitHub Release

Choisir le `target` adapté à la plateforme :

| Plateforme | Target |
|---|---|
| Linux x86_64 statique | `x86_64-unknown-linux-musl` |
| Linux ARM64 statique | `aarch64-unknown-linux-musl` |
| macOS Apple Silicon | `aarch64-apple-darwin` |
| macOS Intel | `x86_64-apple-darwin` |

Télécharger et extraire la dernière release, par exemple Linux x86_64 :

```bash
target=x86_64-unknown-linux-musl; tag=$(curl -fsSL https://api.github.com/repos/Destynova2/amar/releases/latest | sed -n 's/.*"tag_name": "\(v[0-9][^"]*\)".*/\1/p'); curl -fsSL "https://github.com/Destynova2/amar/releases/latest/download/amar-${tag}-${target}.tar.gz" | tar -xz
```

Lancer le serveur depuis le répertoire extrait :

```bash
cd "amar-${tag}-${target}"
./amar serve
```

L'archive contient `amar`, `packs/`, `install.md`, `LICENSE` et
`LIMITATIONS.md`. Le binaire charge automatiquement les packs présents dans
`packs/`, donc aucun Cargo ni Docker n'est nécessaire.

Vérifier le checksum après téléchargement manuel :

```bash
curl -fsSLO https://github.com/Destynova2/amar/releases/latest/download/SHA256SUMS
shasum -a 256 -c SHA256SUMS --ignore-missing
```

### 2. Docker/Podman

L'image GHCR embarque le binaire et les trois packs versionnés :

```bash
docker run --rm -p 3000:3000 ghcr.io/destynova2/amar
```

Vérifier le serveur :

```bash
curl -fsS http://127.0.0.1:3000/health
```

Exemple San Francisco :

```bash
curl -i -H 'content-type: application/json' \
  -d '{"lat":37.806,"lon":-122.465,"datetime":"2026-08-15T12:00:00Z"}' \
  http://127.0.0.1:3000/tide
```

Le même lancement fonctionne avec Podman rootless :

```bash
podman run --rm -p 3000:3000 ghcr.io/destynova2/amar
```

### 3. Cargo depuis le dépôt

Depuis un clone du dépôt :

```bash
make release
./dist/amar serve
```

Produire localement la même forme d'archive qu'en release :

```bash
make dist-tarball TARGET=x86_64-unknown-linux-musl
```

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
  "height_m": 1.081,
  "next_high": {
    "coefficient": 101,
    "height_m": 7.347,
    "t": "2026-08-15T17:47:37Z"
  },
  "next_low": {
    "height_m": 1.079,
    "t": "2026-08-16T00:14:58Z"
  },
  "datum": "zero_hydrographique_brest",
  "source": {
    "kind": "station",
    "id": "refmar:3",
    "name": "Brest",
    "distance_km": 0.011,
    "data_version": "2026-07-06-m2.2",
    "valid_until": "2031-04-01T00:00:00Z"
  },
  "confidence": {
    "method": "calibrated_station_experimental",
    "residual_benchmark_cm": 15.8,
    "validation_period": "2026-04-01T00:00:00Z/2026-07-01T00:00:00Z"
  },
  "warnings": [
    "astronomical_tide_only",
    "not_for_navigation",
    "no_weather_surge",
    "experimental",
    "not_shom",
    "coefficient_experimental"
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

Pour les stations françaises, `next_high.coefficient` est un entier 20–120
calculé depuis notre Brest calibré (`U = 3,05 m`) et porte le warning
`coefficient_experimental`. Ce coefficient n'est pas l'annuaire officiel SHOM.
Les stations REFMAR calibrées publient un horizon de recalibration
`source.valid_until`; hors période, `/tide` ajoute
`outside_validity_period`, et `--strict-validity` transforme ce warning en
refus.

Les réponses `/tide/series` et `/tide/windows` gardent la même forme de
`datum`, `source`, `confidence` et `warnings` que `/tide`.

## CLI

Le même binaire garde l'usage CLI M0 :

```bash
amar tide --lat 37.806 --lon -122.465 --at 2026-08-15T12:00:00Z --pack ~/.local/share/amar/packs/noaa_m0.json
amar tide --lat 48.383 --lon -4.495 --at 2026-08-15T12:00:00Z --pack ~/.local/share/amar/packs/noaa_m0.json --pack ~/.local/share/amar/packs/amar-data-brest-experimental.json
amar coef --at 2026-08-15T12:00:00Z --pack ~/.local/share/amar/packs/noaa_m0.json --pack ~/.local/share/amar/packs/amar-data-brest-experimental.json
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
de l'ordre du benchmark, soit `residual_benchmark_cm = 15.8`.

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
| `amar benchmark-brest` | Rejoue Brest et les benchmarks REFMAR France avec les baselines |
| `amar coef --at <utc>` | Calcule le coefficient depuis la prochaine PM Brest |
| `amar pack-noaa` | Compile les fixtures NOAA brutes en pack |

## Données

Le pack NOAA contient 8 stations harmoniques : Boston, San Francisco,
Pensacola, Seattle, Eastport, Honolulu, Key West et Galveston Pier 21.

Le pack Brest expérimental contient `refmar:3` au zéro hydrographique de Brest.
Les observations d'entrée couvrent
`2021-01-01T00:00:00Z/2026-07-01T00:00:00Z`; la calibration
`2021-01-01T00:00:00Z/2026-04-01T00:00:00Z` exclut les trois derniers mois,
réservés à `benchmark_brest_v1`.

Le pack France expérimental v0.7 contient 21 ports REFMAR RONIM :
Arcachon-Eyrac, Boucau-Bayonne, Boulogne-sur-Mer, Concarneau, Dielette,
Dieppe, Dunkerque, Herbaudière, La Rochelle-Pallice, Le Conquet, Le Crouesty,
Le Havre, Les Sables-d'Olonne, Mimizan, Nouméa Numbo, Ouistreham, Pointe des
Galets, Port-Tudy, Roscoff, Saint-Malo et Saint-Nazaire.
Chaque port a un benchmark figé de trois mois, un manifeste avec SHA-256 des
observations longues d'entrée, `experimental`, `not_official` et `not_shom`.
Cherbourg et Calais ont été retentés sur leur dernière fenêtre à couverture
correcte, mais restent exclus car leur p95 dépasse 40 cm. En Méditerranée, la
marée astronomique est plus petite que le résidu météo sur ce critère : aucun
port méditerranéen mesurable ne bat `z0_constant` d'un facteur 2 en RMS.

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
make validate-history
make m2-benchmark
make fetch-noaa-hilo
make m3-check
```

## Licence

Code sous licence Apache-2.0. Les données NOAA incluses sont dans le domaine
public des États-Unis. Les observations REFMAR et le pack Brest dérivé sont
sous Licence Ouverte 2.0 Etalab avec attribution `Shom / REFMAR`; voir
[`DATA_LICENSES.md`](DATA_LICENSES.md).
