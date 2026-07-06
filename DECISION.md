# Décisions M0-M2

Date : 2026-07-06.

## M0.3 retenu

Le gate moteur v0.2 est atteint sur les 8 stations NOAA et les fenêtres
2026/2031/2036 : `M0_P95_LIMIT_M = 0.02`.

La correction retenue est globale :

- convention annuelle NOS/XTide : `V0` au 1er janvier UTC, puis avance par la
  vitesse du constituant ; `u` et `f` au milieu de l'année civile ;
- zéro harmonique NOAA : `Z0 = MSL - MLLW` pour les packs publiés en `MLLW`.

Les constantes harmoniques, stations, APIs et règles serveur restent
inchangées. Le pack a été régénéré depuis les fixtures locales pour corriger le
champ `z0_m` selon la convention `MSL`.

## Diagnostic M0.3

La sortie `amar validate` affiche maintenant `bias_cm`, `std_cm` et `p95_cm`.

Lecture des essais :

- moteur M1.1 instantané : p95 croissant avec le marnage, signature d'une
  convention temporelle ;
- convention annuelle avec `V0` début d'année et `u/f` milieu d'année : la
  forme temporelle colle à l'oracle, mais le biais restant vaut exactement
  `MTL - MSL` ;
- correction globale `MSL - MLLW` : biais ramené à 0 cm, aucun micro-décalage
  temporel nécessaire.

Seattle n'a pas servi au réglage ; elle reste un témoin et passe à 0,1 cm.

## Validation M0.3

Commande :

```bash
make m0-validate
```

Résultat global :

| Station | p95 M1.1 | bias M0.3 | std M0.3 | p95 M0.3 |
|---|---:|---:|---:|---:|
| `noaa:1612340` Honolulu | 0,9 cm | -0,0 cm | 0,0 cm | 0,1 cm |
| `noaa:8410140` Eastport | 3,7 cm | 0,0 cm | 0,1 cm | 0,3 cm |
| `noaa:8443970` Boston | 4,3 cm | 0,0 cm | 0,1 cm | 0,1 cm |
| `noaa:8724580` Key West | 0,7 cm | -0,0 cm | 0,0 cm | 0,1 cm |
| `noaa:8729840` Pensacola | 0,6 cm | 0,0 cm | 0,0 cm | 0,1 cm |
| `noaa:8771450` Galveston Pier 21 | 0,6 cm | 0,0 cm | 0,0 cm | 0,1 cm |
| `noaa:9414290` San Francisco | 2,5 cm | 0,0 cm | 0,0 cm | 0,1 cm |
| `noaa:9447130` Seattle | 2,1 cm | -0,0 cm | 0,1 cm | 0,1 cm |

Fenêtres M0.3 :

| Station | 2026 | 2031 | 2036 |
|---|---:|---:|---:|
| `noaa:1612340` Honolulu | 0,1 cm | 0,0 cm | 0,1 cm |
| `noaa:8410140` Eastport | 0,2 cm | 0,2 cm | 0,3 cm |
| `noaa:8443970` Boston | 0,1 cm | 0,1 cm | 0,2 cm |
| `noaa:8724580` Key West | 0,1 cm | 0,1 cm | 0,1 cm |
| `noaa:8729840` Pensacola | 0,1 cm | 0,0 cm | 0,1 cm |
| `noaa:8771450` Galveston Pier 21 | 0,1 cm | 0,0 cm | 0,1 cm |
| `noaa:9414290` San Francisco | 0,1 cm | 0,1 cm | 0,1 cm |
| `noaa:9447130` Seattle | 0,1 cm | 0,1 cm | 0,2 cm |

Les quatre stations déjà sous 1 cm restent sous 1,5 cm.

## M0.2 conservé

M0.2 reste le socle moteur : `method = station_harmonics_v0`, constantes
NOAA, datum `MLLW`, origine de `tau` à 1899-12-31 12:00 UTC, constantes V0
par constituant, et corrections nodales Schureman `f/u`.

Les constituants inconnus sont refusés au chargement du modèle.

## Pack M1

Le pack NOAA passe de 4 à 8 stations harmoniques. Les nouvelles stations ont
été vérifiées via NOAA `mdapi` avant ajout : `tidal=true`, présence de
`harmonicConstituents`, et endpoint `harcon` avec 37 constituants en mètres.

| Station | Régime visé | Statut M1 |
|---|---|---|
| `noaa:8443970` Boston | semi-diurne | conservée |
| `noaa:9414290` San Francisco | mixte | conservée |
| `noaa:8729840` Pensacola | diurne | conservée |
| `noaa:9447130` Seattle | témoin hors réglage | conservée |
| `noaa:8410140` Eastport | grand marnage semi-diurne | ajoutée |
| `noaa:1612340` Honolulu | faible marnage | ajoutée |
| `noaa:8724580` Key West | faible marnage | ajoutée |
| `noaa:8771450` Galveston Pier 21 | diurne supplémentaire | ajoutée |

Aucune station ajoutée ne dépasse le gate 5 cm ; aucun remplacement n'a été
nécessaire.

## Validation

Commande :

```bash
make m0-validate
```

Résultat mesuré :

| Station | Échantillons | Méthode | p95 |
|---|---:|---|---:|
| `noaa:1612340` Honolulu | 504 | `station_harmonics_v0` | 0,9 cm |
| `noaa:8410140` Eastport | 504 | `station_harmonics_v0` | 3,7 cm |
| `noaa:8443970` Boston | 504 | `station_harmonics_v0` | 4,3 cm |
| `noaa:8724580` Key West | 504 | `station_harmonics_v0` | 0,7 cm |
| `noaa:8729840` Pensacola | 504 | `station_harmonics_v0` | 0,6 cm |
| `noaa:8771450` Galveston Pier 21 | 504 | `station_harmonics_v0` | 0,6 cm |
| `noaa:9414290` San Francisco | 504 | `station_harmonics_v0` | 2,5 cm |
| `noaa:9447130` Seattle | 504 | `station_harmonics_v0` | 2,1 cm |

Fenêtres de dérive nodale :

| Station | 2026 | 2031 | 2036 |
|---|---:|---:|---:|
| `noaa:1612340` Honolulu | 0,8 cm | 0,8 cm | 0,9 cm |
| `noaa:8410140` Eastport | 3,8 cm | 3,0 cm | 3,9 cm |
| `noaa:8443970` Boston | 4,3 cm | 3,9 cm | 4,4 cm |
| `noaa:8724580` Key West | 0,6 cm | 0,6 cm | 0,7 cm |
| `noaa:8729840` Pensacola | 0,5 cm | 0,5 cm | 0,7 cm |
| `noaa:8771450` Galveston Pier 21 | 0,6 cm | 0,6 cm | 0,7 cm |
| `noaa:9414290` San Francisco | 2,3 cm | 2,4 cm | 2,6 cm |
| `noaa:9447130` Seattle | 2,1 cm | 1,8 cm | 2,5 cm |

Les quatre stations M0.2 gardent leurs p95 existants inchangés.

## Serveur M1

`amar serve --addr 127.0.0.1:3000` charge le pack au démarrage, puis répond
100 % offline.

Endpoints retenus :

- `POST /tide`
- `GET /health`
- `GET /coverage`

En M1, le refus hors rayon était volontairement utile. Brest
`lat=48.383 lon=-4.495` retournait `422 no_supported_source` avec Eastport
comme station la plus proche à 4652,019 km. M2 remplace ce comportement par la
réponse expérimentale `refmar:3`.

La confiance M1 est une heuristique de distance :

| Distance à la station | Grade | Sigma |
|---:|---|---:|
| <= 2 km | A | 8 cm |
| <= 10 km | B | 15 cm |
| <= 20 km | C | 30 cm |

Méthode exposée :
`station_harmonics_v0_distance_heuristic`.

M1.1 borne ce contrat : `--max-distance-km` ne peut pas étendre la confiance
au-delà de 20 km ; les sources plus lointaines restent refusées.

## Décision

Taguer M1.

La boucle M1 est atteinte : un tiers peut construire le binaire, lancer le
serveur, obtenir une hauteur NOAA traçable, ou recevoir un refus explicite hors
couverture.

## M2 — Brest expérimental

Date : 2026-07-06.

### Accès données

La porte de sortie `m2a-blocked` n'est pas utilisée : l'accès programmatique
REFMAR fonctionne via `services.data.shom.fr/maregraphie`.

Produit retenu :

- station unique : `shom_id=3`, `BREST`, réseau `RONIM` ;
- produit : `sources=4`, données horaires validées ;
- période d'entrée : `2025-01-01T00:00:00Z/2026-07-01T00:00:00Z` ;
- licence : Licence Ouverte 2.0 Etalab, attribution `Shom / REFMAR` ;
- référence verticale : `zero_hydrographique`, RAM id `Brest`, ZH = -3.635 m
  par rapport à `IGN69` dans la fiche REFMAR.

La collecte longue période passe par le flux 31 jours documenté par le Swagger
REFMAR, en tranches idempotentes, sans formulaire nom/email.

### Calibration

Commande de reproduction :

```bash
make fetch-refmar
make build-brest-pack
```

Fenêtres :

| Fenêtre | Début | Fin | Échantillons attendus | Observés | Couverture | Trous > 1,5 h | Sauts aberrants |
|---|---|---|---:|---:|---:|---:|---:|
| Entrée | 2025-01-01T00:00:00Z | 2026-07-01T00:00:00Z | 13104 | 13103 | 99,99 % | 0 | 0 |
| Calibration | 2025-01-01T00:00:00Z | 2026-04-01T00:00:00Z | 10920 | 10920 | 100,00 % | 0 | 0 |
| Validation | 2026-04-01T00:00:00Z | 2026-07-01T00:00:00Z | 2184 | 2183 | 99,95 % | 0 | 0 |

La seule lacune du benchmark est `2026-06-30T23:00:00Z`, explicitement masquée
dans `benchmark_brest_v1`.

Méthode :

- constituants fixés : `M2, S2, N2, K2, K1, O1, P1, Q1, M4, MS4, MN4, M6,
  MF, MM, SA, SSA` ;
- solveur : moindres carrés linéaires sin/cos via `nalgebra` SVD ;
- convention : même `station_harmonics_v0` que le moteur (`V0` 1er janvier,
  `f/u` milieu d'année civile) ;
- `Z0` ajusté : `4.287 m` au-dessus du zéro hydrographique de Brest.

Constantes dérivées :

| Constituant | Amplitude m | Phase GMT deg |
|---|---:|---:|
| M2 | 2.0437 | 108.39 |
| S2 | 0.7567 | 148.65 |
| N2 | 0.4087 | 90.28 |
| K2 | 0.2162 | 145.84 |
| K1 | 0.0618 | 76.97 |
| O1 | 0.0652 | 327.98 |
| P1 | 0.0210 | 70.67 |
| Q1 | 0.0195 | 287.75 |
| M4 | 0.0586 | 105.12 |
| MS4 | 0.0387 | 181.28 |
| MN4 | 0.0217 | 60.83 |
| M6 | 0.0304 | 353.99 |
| MF | 0.0195 | 173.97 |
| MM | 0.0381 | 225.04 |
| SA | 0.0983 | 276.24 |
| SSA | 0.0131 | 201.26 |

Disclaimer publié dans le pack : constantes dérivées des observations REFMAR,
non équivalentes aux constantes SHOM.

### Benchmark

Commande :

```bash
make m2-benchmark
```

Définition : résidu = niveau d'eau observé − marée astronomique prédite
(météo incluse).

Fenêtre figée : `2026-04-01T00:00:00Z/2026-07-01T00:00:00Z`.

| Modèle | RMS cm | Biais cm | MAE cm | p95 cm | Max cm |
|---|---:|---:|---:|---:|---:|
| `calibrated_station_experimental` | 13,8 | -1,5 | 11,0 | 26,6 | 46,9 |
| `z0_constant` | 153,2 | -7,7 | 131,9 | 267,4 | 361,9 |
| `m2_only` | 60,8 | -7,9 | 50,2 | 111,3 | 178,3 |

Lecture : p95 26,6 cm est dans l'ordre attendu pour un résidu observation moins
marée astronomique prédite avec météo incluse. Le biais est faible et le modèle
bat nettement les deux baselines.

### Artefacts

| Artefact | Rôle |
|---|---|
| `fixtures/refmar/brest_validated_hourly_2025-01-01_2026-07-01.csv` | observations d'entrée |
| `fixtures/refmar/brest_tidegauge.json` | fiche station et datum |
| `fixtures/refmar/benchmark_brest_v1.json` | benchmark hors calibration figé |
| `data/packs/amar-data-brest-experimental.json` | pack Brest expérimental |

Le serveur et le CLI chargent Brest en plus de NOAA. Les réponses NOAA restent
inchangées dans les snapshots ; Brest répond sans grade A/B/C avec
`confidence.method = calibrated_station_experimental`,
`residual_benchmark_cm = 26.6`, les warnings existants, `experimental` et
`not_shom`.

## Décision M2

Taguer M2.

Le milestone reste borné : une station, un datum, une période, un pack. Pas de
M3 dans cette livraison.

## Décision M2.1

- Durcissement post-audit : confidence/warnings partagés CLI-serveur, packs expérimentaux incomplets refusés, benchmark Brest gaté à `p95 <= 30 cm`.
- Calibrateur scindé en `fetch`/`qc`/`solve`/`pack_out` avec QC trous/sauts, garde SA/SSA, tests locaux, artefacts Brest byte-identiques.

## M3 — Extrema, séries, fenêtres

Date : 2026-07-06.

### Extrema

Choix retenu : échantillonnage de `h(t)` toutes les 6 minutes, détection du
triplet local haut/bas, puis raffinement par interpolation parabolique sur les
trois hauteurs du triplet. La hauteur publiée est recalculée par le moteur au
temps raffiné.

Ce choix évite une dérivée numérique bruitée, reste pur et déterministe dans
`amar-core`, et donne une précision largement meilleure que le gate NOAA
`|Δt| <= 10 min` et `|Δh| <= 3 cm` p95.

### Validation NOAA PM/BM

Commande :

```bash
make m3-check
```

Le gate `validate-hilo` compare chaque PM/BM officiel NOAA
`product=predictions&interval=hilo&datum=MLLW&units=metric&time_zone=gmt` au
plus proche extremum prédit de même type.

| Station | Échantillons | p50 Δt min | p95 Δt min | max Δt min | p50 Δh cm | p95 Δh cm | max Δh cm |
|---|---:|---:|---:|---:|---:|---:|---:|
| `noaa:1612340` Honolulu | 23 | 0,25 | 0,47 | 0,47 | 0,0 | 0,0 | 0,0 |
| `noaa:8410140` Eastport | 54 | 0,28 | 0,57 | 0,57 | 0,0 | 0,1 | 0,1 |
| `noaa:8443970` Boston | 27 | 0,27 | 0,48 | 0,60 | 0,0 | 0,1 | 0,1 |
| `noaa:8724580` Key West | 27 | 0,25 | 0,55 | 0,57 | 0,0 | 0,1 | 0,1 |
| `noaa:8729840` Pensacola | 15 | 0,28 | 0,72 | 0,72 | 0,0 | 0,1 | 0,1 |
| `noaa:8771450` Galveston Pier 21 | 21 | 0,28 | 0,78 | 0,80 | 0,0 | 0,0 | 0,1 |
| `noaa:9414290` San Francisco | 27 | 0,22 | 0,45 | 0,45 | 0,0 | 0,1 | 0,1 |
| `noaa:9447130` Seattle | 55 | 0,32 | 0,55 | 0,57 | 0,0 | 0,1 | 0,1 |

Fenêtres par fichier :

| Station | Fenêtre | Échantillons | p95 Δt min | p95 Δh cm |
|---|---|---:|---:|---:|
| `noaa:1612340` Honolulu | 2026 | 23 | 0,47 | 0,0 |
| `noaa:8410140` Eastport | 2026 | 27 | 0,57 | 0,1 |
| `noaa:8410140` Eastport | 2031 | 27 | 0,45 | 0,1 |
| `noaa:8443970` Boston | 2026 | 27 | 0,48 | 0,1 |
| `noaa:8724580` Key West | 2026 | 27 | 0,55 | 0,1 |
| `noaa:8729840` Pensacola | 2026 | 15 | 0,72 | 0,1 |
| `noaa:8771450` Galveston Pier 21 | 2026 | 21 | 0,78 | 0,0 |
| `noaa:9414290` San Francisco | 2026 | 27 | 0,45 | 0,1 |
| `noaa:9447130` Seattle | 2026 | 27 | 0,55 | 0,1 |
| `noaa:9447130` Seattle | 2031 | 28 | 0,52 | 0,1 |

Seattle reste témoin hors réglage. Les fenêtres 2031 sont limitées à Seattle
et Eastport pour couvrir le témoin et le plus grand marnage du pack.

### Brest

Brest n'a pas d'oracle PM/BM officiel dans M3. La couverture repose sur des
tests d'invariants : alternance PM/BM, monotonie entre extrema adjacents et
bornes de fenêtres vérifiant le seuil. Le disclaimer M2 reste applicable :
les constantes Brest sont expérimentales, non SHOM, et le résidu p95 de 26,6 cm
s'applique aux seuils.
