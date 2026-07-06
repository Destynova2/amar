# amar

> Calcule une hauteur de maree astronomique hors ligne pres d'une station NOAA
> connue, avec datum et provenance explicites.

## Demarrage rapide

```bash
make pack-noaa
cargo run -p amar -- tide --lat 37.806 --lon -122.465 --at 2026-08-15T12:00:00Z
make m0-validate
```

## Ce que fait M0

- Charge un pack NOAA JSON versionne et checksomme.
- Cherche la station harmonique la plus proche dans un rayon de 20 km.
- Calcule `h(t)` avec les constantes harmoniques du pack.
- Affiche toujours le datum, la source et la methode.
- Compare le moteur aux predictions officielles NOAA avec un p95 par station.

## Exemples

San Francisco :

```bash
cargo run -p amar -- tide --lat 37.806 --lon -122.465 --at 2026-08-15T12:00:00Z
```

Boston :

```bash
cargo run -p amar -- tide --lat 42.354 --lon -71.052 --at 2026-08-15T12:00:00Z
```

Hors couverture :

```bash
cargo run -p amar -- tide --lat 48.383 --lon -4.495 --at 2026-08-15T12:00:00Z
```

## Usage

| Commande | Role |
|---|---|
| `amar tide --lat <deg> --lon <deg> --at <utc>` | Calcule une hauteur instantanee |
| `amar validate` | Affiche le p95 vs predictions NOAA |
| `amar pack-noaa` | Compile les fixtures NOAA brutes en pack M0 |

## Structure

```text
crates/amar-core       moteur harmonique pur
crates/amar-pack       contrat JSON des packs
crates/amar-data       chargement et validation
crates/amar            CLI M0
crates/amar-calibrate  squelette M2
fixtures/noaa          sources NOAA brutes
data/packs             packs offline
```

## Developpement

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
make m0-validate
```

## Licence

Code sous licence Apache-2.0. Les donnees NOAA incluses sont dans le domaine
public des Etats-Unis ; voir `DATA_LICENSES.md`.
