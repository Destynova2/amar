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

## Décision M2.2 — précision Brest

Date : 2026-07-06.

Règle graduée : `fixtures/refmar/benchmark_brest_v1.json` est resté
byte-identique. SHA-256 fichier :
`d36f445c320c17ba323fbe572e0cb93d45eba846aeff0260ee9d1b3631a6bf6f`.
Le checksum interne du masque horaire et des observations reste
`531da284f68bb9acf77c9d21b90e0fd3d787809c0ddb0cd4d118c63eddc0ac42`.

### Diagnostic spectral

Commande :

```bash
cargo run -p amar-calibrate -- diagnose --observations fixtures/refmar/brest_validated_hourly_2025-01-01_2026-07-01.csv --pack /private/tmp/amar-brest-m2-base16-15m.json
```

Lecture des bandes : basse fréquence par lissage 48 h, bandes tidales par
sondage harmonique des 37 fréquences supportées.

| Résidu `16 × 15 mois` | Début | Fin | N | RMS cm | Biais cm | <0,5 cpd cm | ~1 cpd cm | ~2 cpd cm | composés cm | tidal cohérent cm |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Calibration | 2025-01-01T00:00:00Z | 2026-04-01T00:00:00Z | 10920 | 17,8 | 0,0 | 12,4 | 0,9 | 10,2 | 1,4 | 10,3 |
| Benchmark | 2026-04-01T00:00:00Z | 2026-07-01T00:00:00Z | 2183 | 13,8 | -1,5 | 5,8 | 1,2 | 11,2 | 1,6 | 11,4 |

Porte de décision : le résidu tidal cohérent est supérieur à 3 cm RMS
équivalent sur la référence, donc M2.2 continue vers l'élargissement des
constituants et la calibration longue.

Diagnostic après pack retenu :

```bash
cargo run -p amar-calibrate -- diagnose
```

| Résidu `37 × multi-année` | Début | Fin | N | RMS cm | Biais cm | <0,5 cpd cm | ~1 cpd cm | ~2 cpd cm | composés cm | tidal cohérent cm |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Calibration | 2021-01-01T00:00:00Z | 2026-04-01T00:00:00Z | 45367 | 15,0 | -0,0 | 13,1 | 0,1 | 0,3 | 0,0 | 0,3 |
| Benchmark | 2026-04-01T00:00:00Z | 2026-07-01T00:00:00Z | 2183 | 7,9 | -1,1 | 5,5 | 0,9 | 2,0 | 1,0 | 2,4 |

Après M2.2, le résidu tidal cohérent repasse sous le plancher de décision ;
le résidu restant est dominé par la basse fréquence météo/seiches.

### Constituants et calibration

La liste publiée passe de 16 à 37 constituants fixes :

```text
M2, S2, N2, K2, K1, O1, P1, Q1, M4, MS4, MN4, M6, MF, MM, SA, SSA,
L2, NU2, MU2, 2N2, LAM2, T2, R2, J1, OO1, RHO, 2Q1, M1, S1,
MK3, 2MK3, M3, S4, S6, M8, MSF, 2SM2
```

Le calcul de séparabilité Rayleigh est documenté dans `CONVENTIONS.md`. La
fenêtre de calibration retenue est `2021-01-01T00:00:00Z` à
`2026-04-01T00:00:00Z`; la validation `2026-04-01T00:00:00Z` à
`2026-07-01T00:00:00Z` reste exclue de la calibration.

QC annuel des observations REFMAR `source=4`, même station, même référence
verticale `zero_hydrographique`, RAM id `Brest`, ZH = -3,635 m par rapport à
`IGN69` :

| Année | Attendus | Observés | Couverture | Trous > 1,5 h | Sauts aberrants |
|---:|---:|---:|---:|---:|---:|
| 2021 | 8760 | 8669 | 98,96 % | 4 | 0 |
| 2022 | 8760 | 8756 | 99,95 % | 1 | 0 |
| 2023 | 8760 | 8760 | 100,00 % | 0 | 0 |
| 2024 | 8784 | 8262 | 94,06 % | 2 | 0 |
| 2025 | 8760 | 8760 | 100,00 % | 0 | 0 |
| 2026 | 4344 | 4343 | 99,98 % | 0 | 0 |

La lacune 2024-11-27T08:00:00Z → 2024-12-17T17:00:00Z est acceptée car la
couverture annuelle reste au-dessus du gate QC 90 % et aucun saut aberrant
n'est détecté.

### Comparaison benchmark figé

Commande :

```bash
cargo run -p amar -- benchmark-brest
```

| Configuration | RMS cm | Biais cm | MAE cm | p95 cm | Max cm |
|---|---:|---:|---:|---:|---:|
| `16 × 15 mois` | 13,8 | -1,5 | 11,0 | 26,6 | 46,9 |
| `37 × 15 mois` | 8,4 | -1,5 | 6,4 | 17,3 | 31,8 |
| `37 × multi-année` | 7,9 | -1,1 | 6,2 | 15,8 | 32,5 |

Le gain p95 est supérieur à 2 cm ; la configuration `37 × multi-année` devient
le pack Brest publié. `data_version = 2026-07-06-m2.2`,
`calibration_period = 2021-01-01T00:00:00Z/2026-04-01T00:00:00Z`,
`residual_benchmark_cm = 15.8`.

Le gate `m2-benchmark` est resserré à `p95 <= 19 cm`, soit p95 arrondi 16 cm +
3 cm de marge.

### Baromètre inverse

Source diagnostic : Open-Meteo Historical Weather API, variable horaire
`surface_pressure` en hPa, `timezone=GMT`, `cell_selection=nearest`, fenêtre
`2026-04-01/2026-06-30`. Cette donnée n'entre ni dans `/tide`, ni dans le
pack, ni dans `benchmark_brest_v1`.

Commande :

```bash
cargo run -p amar-calibrate -- diagnose-ib
```

Formule : `IB = -0,9933 cm/hPa × (P - 1013,25)`.

| N | Corr(residu, IB) | r² | Variance retirée par IB fixe | RMS avant cm | RMS après IB cm | Biais avant cm | Biais après cm |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 2183 | 0,662 | 43,8 % | 43,8 % | 7,9 | 7,0 | -1,1 | 3,9 |

Le plancher astronomique estimé après retrait IB fixe est donc environ
7,0 cm RMS sur le benchmark M2.2. Le reste n'est pas injecté dans la
prédiction : c'est un diagnostic de plancher, pas une correction météo.

### Artefacts M2.2

| Artefact | Rôle |
|---|---|
| `fixtures/refmar/brest_validated_hourly_2021-01-01_2026-07-01.csv` | observations longues REFMAR |
| `fixtures/open_meteo/brest_surface_pressure_2026-04-01_2026-06-30.json` | pression diagnostic IB |
| `fixtures/refmar/benchmark_brest_v1.json` | benchmark figé byte-identique |
| `data/packs/amar-data-brest-experimental.json` | pack Brest M2.2 publié |

Idempotence : un re-run de `cargo run -p amar-calibrate -- build-brest-pack`
réécrit le même pack, SHA-256
`e377e25754e9cbc6a05e732d0dcff2db2c1305b57037432312015ae45d749919`.

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
les constantes Brest sont expérimentales, non SHOM, et le résidu p95 de 15,8 cm
s'applique aux seuils.

M3.1 : les fenêtres de seuil sont clampées à `[from,to]` ; sans croisement, une plage active renvoie `[from,to]`.
M3.1 : le contrat CLI/serveur est factorisé pour bornes, validations et shapes JSON.

## M4 — Performance prouvée

Date : 2026-07-06.

Règle de décision : aucun patch de performance n'est conservé sans preuve
d'équivalence et mesure A/B Criterion. Un gain dans le bruit ou négatif entraîne
le retrait du patch.

### Baseline Criterion

Baseline mesurée après ajout du harnais Criterion, avant XRAY-001, XRAY-002 et
XRAY-006.

Commandes :

```bash
cargo bench -p amar --bench m4_core -- --save-baseline m4-baseline
cargo bench -p amar-calibrate --bench m4_calibrate -- --save-baseline m4-baseline
```

| Benchmark | Baseline |
|---|---:|
| `predict_height_brest37_one_timestamp` | 0,510 µs |
| `predict_series_brest37_72h_step6m_721_points` | 446,77 µs |
| `tide_windows_brest37_31d_above_4m` | 6,100 ms |
| `calibrate_ls_brest37_synthetic/assemble_matrix_only` | 61,73 ms |
| `calibrate_ls_brest37_synthetic/svd_only` | 186,25 ms |

### Résultats A/B retenus

Commandes :

```bash
cargo bench -p amar --bench m4_core -- --baseline m4-baseline
cargo bench -p amar-calibrate --bench m4_calibrate -- --baseline m4-baseline
```

| Benchmark | Baseline | Après | Verdict Criterion |
|---|---:|---:|---|
| `predict_height_brest37_one_timestamp` | 0,510 µs | 0,513 µs | pas de changement détecté |
| `predict_series_brest37_72h_step6m_721_points` | 446,77 µs | 84,39 µs | gain -80,97 % |
| `tide_windows_brest37_31d_above_4m` | 6,100 ms | 1,490 ms | gain -75,55 % |
| `calibrate_ls_brest37_synthetic/assemble_matrix_only` | 61,73 ms | 11,891 ms | gain -80,68 % |
| `calibrate_ls_brest37_synthetic/svd_only` | 186,25 ms | 185,09 ms | dans le bruit |

XRAY-001 est retenu pour les boucles longues : le modèle annuel compilé est
construit une fois par requête et reconstruit au franchissement d'année UTC.
`predict_height` isolé reste sur le chemin direct, car compiler par appel n'a
pas de gain prouvé sur le benchmark à un timestamp.

XRAY-002 est retenu : l'assemblage LS pré-calcule les termes par
`(année, constituant)` et n'appelle plus qu'un `sin_cos` par constituant et par
échantillon. L'ordre flottant historique `V0 + speed*h + u` est conservé pour
garder le pack Brest byte-identique.

XRAY-006 est partiellement retenu : le remplissage direct de `DMatrix` dans le
solveur est conservé. Mesuré contre la baseline intermédiaire `m4-xray002`,
l'assemblage passe de 15,21 ms à 11,88 ms, soit -21,85 %. Les
`Vec::with_capacity` côté `predict_series` et `sample_heights` ont été
tentés puis retirés : `predict_series` régressait de +1,29 % et
`tide_windows` restait dans le bruit.

### Preuves d'équivalence

- Test propriété old-vs-compiled sur timestamps aléatoires 2020-2030 :
  `|Δh| <= 1e-9 m`.
- Test déterministe old-vs-compiled sur ±60 s autour des 1ers janvier UTC
  2020-2031 : `|Δh| <= 1e-9 m`.
- `cargo test --workspace` : 64 tests verts, incluant golden NOAA.
- Pack Brest régénéré byte-identique :
  `e377e25754e9cbc6a05e732d0dcff2db2c1305b57037432312015ae45d749919`.
- `benchmark_brest_v1` inchangé :
  `d36f445c320c17ba323fbe572e0cb93d45eba846aeff0260ee9d1b3631a6bf6f`.

### Gates M4

| Gate | Résultat |
|---|---|
| `cargo fmt --all --check` | vert |
| `cargo clippy --workspace --all-targets -- -D warnings` | vert |
| `make m0-validate` | vert, p95 NOAA max 0,3 cm |
| `make m1-smoke` | vert |
| `make m2-benchmark` | vert, Brest p95 15,8 cm |
| `make m3-check` | vert, hilo p95 max Δt 0,78 min et Δh 0,1 cm |

## v0.4 -- France prioritaire et coefficient

Date : 2026-07-06.

La garde "une station" de M2 est levée. Le pipeline Brest est généralisé dans
`amar-calibrate calibrate-france` :

- découverte du catalogue `service/tidegauges` REFMAR ;
- cache de reprise sous `target/refmar-cache`, par station et fenêtre 31 jours ;
- throttle HTTP par défaut : 600 ms, refusé sous 500 ms ;
- absence de retry storm : une erreur de station exclut le port et la boucle
  continue ;
- observations longues non commitées ; seuls pack, benchmarks 3 mois et
  manifestes SHA-256 sont commités.

Cette session livre le lot prioritaire Manche/Atlantique, avec 11 ports inclus.
`make calibrate-france` reste rejouable pour compléter le catalogue
métropolitain éligible.

Commandes :

```bash
cargo run -p amar-calibrate -- calibrate-france \
  --station 410 --station 4 --station 13 --station 37 --station 34 \
  --station 160 --station 152 --station 54 --station 2 --station 111 \
  --station 55 --station 24 --station 311
cargo run -p amar -- benchmark-brest --brest-p95-limit-cm 19 --p95-limit-cm 40 --min-rms-factor 2
```

Pack France :
`data/packs/amar-data-france-experimental.json`, SHA-256
`9aa5249920396be1fa91cdd1c5ec58301dc5e3c074230290c0a98f83591830b2`.

Périodes :

| Fenêtre | Début | Fin |
|---|---|---|
| Entrée | 2021-01-01T00:00:00Z | 2026-07-01T00:00:00Z |
| Calibration | 2021-01-01T00:00:00Z | 2026-04-01T00:00:00Z |
| Validation | 2026-04-01T00:00:00Z | 2026-07-01T00:00:00Z |

Critère d'inclusion : `p95 <= 40 cm` sur benchmark figé et facteur RMS
`z0_constant / calibrated >= 2`. Brest conserve son gate historique
`p95 <= 19 cm`.

| Port | Station | Couverture validation | RMS cm | p95 cm | RMS z0 cm | Facteur | Décision |
|---|---|---:|---:|---:|---:|---:|---|
| Saint-Malo | `refmar:410` | 100,0 % | 14,7 | 28,5 | 277,7 | 18,83 | inclus |
| Le Havre | `refmar:4` | 100,0 % | 19,5 | 37,4 | 197,8 | 10,14 | inclus |
| Saint-Nazaire | `refmar:37` | 100,0 % | 9,2 | 18,4 | 131,6 | 14,25 | inclus |
| La Rochelle-Pallice | `refmar:34` | 100,0 % | 9,5 | 18,8 | 131,0 | 13,78 | inclus |
| Concarneau | `refmar:160` | 99,8 % | 7,3 | 14,5 | 110,1 | 15,19 | inclus |
| Le Conquet | `refmar:152` | 100,0 % | 7,3 | 15,1 | 149,9 | 20,57 | inclus |
| Roscoff | `refmar:54` | 99,8 % | 8,8 | 17,6 | 200,9 | 22,92 | inclus |
| Dunkerque | `refmar:2` | 100,0 % | 16,4 | 32,9 | 162,5 | 9,91 | inclus |
| Boulogne-sur-Mer | `refmar:111` | 100,0 % | 18,2 | 35,9 | 224,4 | 12,33 | inclus |
| Dieppe | `refmar:24` | 99,7 % | 16,0 | 31,5 | 231,7 | 14,47 | inclus |
| Ouistreham | `refmar:311` | 100,0 % | 18,3 | 35,0 | 192,9 | 10,53 | inclus |
| Cherbourg | `refmar:13` | 0,0 % | NA | NA | NA | NA | exclu : aucune observation validée `source=4` sur la validation |
| Calais | `refmar:55` | 0,0 % | NA | NA | NA | NA | exclu : aucune observation validée `source=4` sur la validation |

Le seuil QC de saut instantané passe de 2,5 m à 4,0 m. Le seuil historique
Brest était trop strict pour Saint-Malo : un port macrotidal peut avoir une
pente horaire naturelle supérieure à 2,5 m autour des plus grands marnages.
Les sauts restent conditionnés à `Δt <= 90 min`.

### Coefficient

Le coefficient de marée est ajouté dans `amar` uniquement. Il n'entre pas dans
`amar-core`.

Convention :

```text
C = 100 * (hauteur PM Brest - niveau moyen Brest) / U
U = 3,05 m
```

Le coefficient est calculé depuis notre pack Brest (`refmar:3`), arrondi,
clampé entre 20 et 120, puis attaché à `next_high` pour les stations REFMAR de
France métropolitaine. Les réponses portent le warning
`coefficient_experimental` parce que ces coefficients dérivent de notre
calibration et non de l'annuaire officiel.

CLI :

```bash
amar coef --at 2026-08-15T12:00:00Z
```

Résultat de référence :

```json
{
  "coefficient": 101,
  "unit_m": 3.05
}
```

## v0.7 -- France RONIM complet

Date : 2026-07-07.

Le pipeline `make calibrate-france` traite maintenant les stations du catalogue
REFMAR dont `state = OK` et `reseau = RONIM`, hors Brest déjà publié dans le
pack dédié `refmar:3`. Les stations REFMAR auxiliaires non RONIM restent hors
pack produit v0.7. Toulon est bien présent dans le catalogue, mais avec
`state = KO`, donc exclu avant calibration.

Les règles de décision ne changent pas :

- throttle HTTP par défaut 600 ms, refus sous 500 ms ;
- produit REFMAR `sources=4`, données horaires validées ;
- QC couverture >= 90 % et absence de saut aberrant ;
- 37 constituants fixes `m22-rayleigh37` ;
- inclusion si `p95 <= 40 cm` et `z0_constant / calibrated >= 2` en RMS ;
- Brest garde son gate historique `p95 <= 19 cm`.

Cherbourg et Calais n'ont toujours pas de données validées `source=4` sur la
fenêtre globale `2026-04-01T00:00:00Z/2026-07-01T00:00:00Z`. Les dernières
périodes à couverture correcte ont donc été figées par station :

| Port | Dernière observation source 4 vue | Fenêtre validation figée | Verdict |
|---|---|---|---|
| Cherbourg | `2026-03-31T22:00:00Z` | `2026-01-01T00:00:00Z/2026-04-01T00:00:00Z` | exclu : p95 41,3 cm > 40 cm |
| Calais | `2025-11-04T05:00:00Z` | `2025-08-01T00:00:00Z/2025-11-01T00:00:00Z` | exclu : p95 44,5 cm > 40 cm |

Pack France :
`data/packs/amar-data-france-experimental.json`, SHA-256
`e94cf45a669a246cc4aa38d1b2c0e60a580905a307b688bcaa7ce3276981b497`.
`data_version = 2026-07-07-v0.7-france`.

Résultat : 21 ports inclus. Toute la Méditerranée mesurable est exclue :
la marée astronomique y existe, mais elle ne bat pas le niveau moyen constant
d'un facteur 2 en RMS sur cette fenêtre. Les ports méditerranéens sans métrique
sont exclus par QC ou état catalogue, pas repêchés.

Commande de reproduction :

```bash
make calibrate-france
make m2-benchmark
```

Table de décision du catalogue RONIM traité :

| Port | Station | Validation | Couverture | RMS cm | p95 cm | RMS z0 cm | Facteur | Décision |
|---|---|---|---:|---:|---:|---:|---:|---|
| Saint-Malo | `refmar:410` | 2026-04/2026-07 | 100,0 % | 14,7 | 28,5 | 277,7 | 18,83 | inclus |
| Le Havre | `refmar:4` | 2026-04/2026-07 | 100,0 % | 19,5 | 37,4 | 197,8 | 10,14 | inclus |
| Saint-Nazaire | `refmar:37` | 2026-04/2026-07 | 100,0 % | 9,2 | 18,4 | 131,6 | 14,25 | inclus |
| La Rochelle-Pallice | `refmar:34` | 2026-04/2026-07 | 100,0 % | 9,5 | 18,8 | 131,0 | 13,78 | inclus |
| Concarneau | `refmar:160` | 2026-04/2026-07 | 99,8 % | 7,3 | 14,5 | 110,1 | 15,19 | inclus |
| Le Conquet | `refmar:152` | 2026-04/2026-07 | 100,0 % | 7,3 | 15,1 | 149,9 | 20,57 | inclus |
| Roscoff | `refmar:54` | 2026-04/2026-07 | 99,8 % | 8,8 | 17,6 | 200,9 | 22,92 | inclus |
| Dunkerque | `refmar:2` | 2026-04/2026-07 | 100,0 % | 16,4 | 32,9 | 162,5 | 9,91 | inclus |
| Boulogne-sur-Mer | `refmar:111` | 2026-04/2026-07 | 100,0 % | 18,2 | 35,9 | 224,4 | 12,33 | inclus |
| Arcachon-Eyrac | `refmar:190` | 2026-04/2026-07 | 100,0 % | 9,4 | 18,3 | 98,5 | 10,46 | inclus |
| Boucau-Bayonne | `refmar:94` | 2026-04/2026-07 | 100,0 % | 6,8 | 13,1 | 94,4 | 13,93 | inclus |
| Dielette | `refmar:628` | 2026-04/2026-07 | 98,7 % | 12,1 | 24,1 | 216,1 | 17,85 | inclus |
| Dieppe | `refmar:24` | 2026-04/2026-07 | 99,7 % | 16,0 | 31,5 | 231,7 | 14,47 | inclus |
| Herbaudière | `refmar:198` | 2026-04/2026-07 | 99,4 % | 8,2 | 16,6 | 124,8 | 15,19 | inclus |
| Les Sables-d'Olonne | `refmar:62` | 2026-04/2026-07 | 99,8 % | 7,7 | 14,9 | 116,0 | 15,09 | inclus |
| Le Crouesty | `refmar:185` | 2026-04/2026-07 | 100,0 % | 8,1 | 16,3 | 121,4 | 15,00 | inclus |
| Mimizan | `refmar:6144` | 2026-04/2026-07 | 100,0 % | 15,1 | 34,0 | 91,0 | 6,01 | inclus |
| Nouméa Numbo | `refmar:659` | 2026-04/2026-07 | 99,8 % | 7,5 | 14,3 | 33,7 | 4,51 | inclus |
| Ouistreham | `refmar:311` | 2026-04/2026-07 | 100,0 % | 18,3 | 35,0 | 192,9 | 10,53 | inclus |
| Pointe des Galets | `refmar:110` | 2026-04/2026-07 | 100,0 % | 5,9 | 11,3 | 16,5 | 2,78 | inclus |
| Port-Tudy | `refmar:71` | 2026-04/2026-07 | 100,0 % | 7,2 | 14,3 | 112,1 | 15,54 | inclus |
| Cherbourg | `refmar:13` | 2026-01/2026-04 | 100,0 % | 20,3 | 41,3 | 145,6 | 7,17 | exclu : p95 > 40 cm |
| Ajaccio | `refmar:300` | 2026-04/2026-07 | 99,7 % | 4,8 | 9,3 | 8,5 | 1,79 | exclu : facteur < 2 |
| Audierne | `refmar:6305` | 2026-04/2026-07 | NA | NA | NA | NA | NA | exclu : couverture calibration 0,890 < 0,900 |
| Calais | `refmar:55` | 2025-08/2025-11 | 98,5 % | 23,2 | 44,5 | 187,2 | 8,07 | exclu : p95 > 40 cm |
| Centuri | `refmar:807` | 2026-04/2026-07 | 99,6 % | 6,1 | 11,1 | 10,3 | 1,70 | exclu : facteur < 2 |
| Ile Saint-Pierre | `refmar:115` | 2026-04/2026-07 | 0,0 % | NA | NA | NA | NA | exclu : couverture validation 0,000 < 0,900 |
| La Figueirette | `refmar:846` | 2026-04/2026-07 | 100,0 % | 5,6 | 11,0 | 9,3 | 1,66 | exclu : facteur < 2 |
| Marseille | `refmar:524` | 2026-04/2026-07 | 100,0 % | 6,3 | 12,0 | 10,0 | 1,59 | exclu : facteur < 2 |
| Monaco | `refmar:22` | 2026-04/2026-07 | 100,0 % | 5,6 | 11,0 | 9,4 | 1,68 | exclu : facteur < 2 |
| Nice | `refmar:339` | 2026-04/2026-07 | 100,0 % | 5,6 | 10,9 | 9,3 | 1,65 | exclu : facteur < 2 |
| Pointe-à-Pitre | `refmar:125` | 2026-04/2026-07 | NA | NA | NA | NA | NA | exclu : couverture calibration 0,748 < 0,900 |
| Port-la-Nouvelle | `refmar:803` | 2026-04/2026-07 | NA | NA | NA | NA | NA | exclu : couverture calibration 0,801 < 0,900 |
| Port-Vendres | `refmar:75` | 2026-04/2026-07 | 100,0 % | 5,1 | 10,3 | 8,0 | 1,57 | exclu : facteur < 2 |
| Port-de-Bouc | `refmar:720` | 2026-04/2026-07 | NA | NA | NA | NA | NA | exclu : couverture calibration 0,558 < 0,900 |
| Port-Ferréol | `refmar:847` | 2026-04/2026-07 | 100,0 % | 5,4 | 10,8 | 8,9 | 1,64 | exclu : facteur < 2 |
| Saint-Jean-de-Luz Socoa | `refmar:95` | 2026-04/2026-07 | NA | NA | NA | NA | NA | exclu : couverture calibration 0,758 < 0,900 |
| Saint-Quay-Portrieux | `refmar:506` | 2026-04/2026-07 | NA | NA | NA | NA | NA | exclu : saut aberrant 12,828 m en calibration |
| Sète | `refmar:250` | 2026-04/2026-07 | 100,0 % | 6,1 | 13,0 | 8,4 | 1,37 | exclu : facteur < 2 |
| Solenzara | `refmar:710` | 2026-04/2026-07 | NA | NA | NA | NA | NA | exclu : saut aberrant -4,994 m en calibration |
| Toulon | `refmar:68` | NA | NA | NA | NA | NA | NA | exclu : station catalogue `state=KO` |

## v0.8 -- validation décennale et validité temporelle

Date : 2026-07-08.

### Validation historique REFMAR

Commande :

```bash
cargo run -p amar-calibrate -- validate-history --report-out target/refmar-cache/history_decennial_v1.csv
```

Le fetch utilise REFMAR source 4, des tranches de 31 jours, le cache
`target/refmar-cache` et le throttle par défaut 600 ms, refusé sous 500 ms.
Aucun recalage n'est fait : les packs publics Brest M2.2 et France v0.7 sont
évalués tels quels sur
`2016-01-01T00:00:00Z/2026-07-01T00:00:00Z`.

Artefacts :

| Artefact | SHA-256 | Role |
|---|---|---|
| `fixtures/refmar/benchmark_brest_decennial_v1.json` | `d371be7da00d4324ce92fd016d3601c5669c3d452dd4373a5b3347f5ed80b5e5` | benchmark Brest historique additif |
| `fixtures/refmar/benchmark_brest_v1.json` | `d36f445c320c17ba323fbe572e0cb93d45eba846aeff0260ee9d1b3631a6bf6f` | benchmark M2.2, reste byte-identique |
| `target/refmar-cache/history_decennial_v1.csv` | `9b1ca9c49eba28d53fc4eb028929b1c9bf04e81fbe3ecf50fdeea5c7daf699b8` | rapport reproductible non commité |

Lecture Brest : le RMS annuel sur vraie mer entière ne reste pas au niveau du
benchmark 3 mois de 7,9 cm ; il reste dans une bande 13,5--16,6 cm, dominée
par la basse fréquence météorologique et par le choix d'années complètes. Il
n'y a pas de croissance monotone du RMS en remontant vers 2016, donc pas de
signature claire de dérive nodale/phase.

Le biais Brest mesure le niveau moyen relatif au Z0 figé 2021--2026 :
moyenne 2016--2020 = -5,3 cm, moyenne 2021--2026 = +0,3 cm, pente linéaire
2016--2026 = +1,18 cm/an, soit +11,8 cm/décennie. Cette valeur est une mesure
empirique brute de vraie mer, donc elle mélange montée lente du niveau moyen,
météo basse fréquence et distribution incomplète de certaines années ; elle
est supérieure à l'ordre de grandeur eustatique seul attendu.

L'alerte 2x RMS benchmark est conservée comme signal non bloquant par défaut :
17 couples station/année la déclenchent. Elle peut être rendue bloquante par
`--fail-on-alert`. Les gates publiés restent inchangés.

### Table annuelle historique

| Port | Station | Année | N | Couverture | RMS cm | Biais cm | p95 cm | Alerte 2x |
|---|---|---:|---:|---:|---:|---:|---:|---|
| Pointe Des Galets | `refmar:110` | 2016 | 8629 | 98.2 % | 8.5 | -4.9 | 15.6 | - |
| Pointe Des Galets | `refmar:110` | 2017 | 8754 | 99.9 % | 7.0 | 3.5 | 13.4 | - |
| Pointe Des Galets | `refmar:110` | 2018 | 8751 | 99.9 % | 9.2 | 6.3 | 17.2 | - |
| Pointe Des Galets | `refmar:110` | 2019 | 8760 | 100.0 % | 10.0 | 7.5 | 18.2 | - |
| Pointe Des Galets | `refmar:110` | 2020 | 8518 | 97.0 % | 6.6 | -0.6 | 12.3 | - |
| Pointe Des Galets | `refmar:110` | 2021 | 8760 | 100.0 % | 6.4 | -2.6 | 12.2 | - |
| Pointe Des Galets | `refmar:110` | 2022 | 8660 | 98.9 % | 5.5 | 0.7 | 10.8 | - |
| Pointe Des Galets | `refmar:110` | 2023 | 8759 | 100.0 % | 7.2 | 2.7 | 14.6 | - |
| Pointe Des Galets | `refmar:110` | 2024 | 8784 | 100.0 % | 7.9 | 3.0 | 14.3 | - |
| Pointe Des Galets | `refmar:110` | 2025 | 8757 | 100.0 % | 7.4 | -1.9 | 13.3 | - |
| Pointe Des Galets | `refmar:110` | 2026 | 4343 | 100.0 % | 8.1 | -5.1 | 16.5 | - |
| Boulogne-sur-mer | `refmar:111` | 2016 | 8764 | 99.8 % | 23.6 | -7.8 | 46.9 | - |
| Boulogne-sur-mer | `refmar:111` | 2017 | 8755 | 99.9 % | 23.2 | -7.0 | 46.9 | - |
| Boulogne-sur-mer | `refmar:111` | 2018 | 8714 | 99.5 % | 23.6 | -7.0 | 48.2 | - |
| Boulogne-sur-mer | `refmar:111` | 2019 | 8749 | 99.9 % | 23.1 | -3.1 | 45.9 | - |
| Boulogne-sur-mer | `refmar:111` | 2020 | 8784 | 100.0 % | 23.9 | -1.5 | 47.0 | - |
| Boulogne-sur-mer | `refmar:111` | 2021 | 8740 | 99.8 % | 22.2 | -4.5 | 44.1 | - |
| Boulogne-sur-mer | `refmar:111` | 2022 | 8760 | 100.0 % | 22.6 | -3.1 | 45.8 | - |
| Boulogne-sur-mer | `refmar:111` | 2023 | 8756 | 100.0 % | 25.3 | 2.6 | 51.9 | - |
| Boulogne-sur-mer | `refmar:111` | 2024 | 8784 | 100.0 % | 24.3 | 3.3 | 49.1 | - |
| Boulogne-sur-mer | `refmar:111` | 2025 | 8760 | 100.0 % | 22.2 | 0.2 | 44.2 | - |
| Boulogne-sur-mer | `refmar:111` | 2026 | 4343 | 100.0 % | 21.3 | 2.5 | 42.3 | - |
| Le Conquet | `refmar:152` | 2016 | 8765 | 99.8 % | 14.6 | -6.3 | 28.0 | - |
| Le Conquet | `refmar:152` | 2017 | 8747 | 99.9 % | 15.6 | -8.6 | 29.6 | oui |
| Le Conquet | `refmar:152` | 2018 | 7948 | 90.7 % | 14.9 | -5.3 | 29.0 | oui |
| Le Conquet | `refmar:152` | 2019 | 8760 | 100.0 % | 15.0 | -3.8 | 30.0 | oui |
| Le Conquet | `refmar:152` | 2020 | 8760 | 99.7 % | 13.0 | -1.7 | 25.5 | - |
| Le Conquet | `refmar:152` | 2021 | 8285 | 94.6 % | 13.3 | -3.9 | 25.3 | - |
| Le Conquet | `refmar:152` | 2022 | 8757 | 100.0 % | 13.1 | -3.6 | 26.4 | - |
| Le Conquet | `refmar:152` | 2023 | 8760 | 100.0 % | 15.8 | 0.7 | 31.9 | - |
| Le Conquet | `refmar:152` | 2024 | 8461 | 96.3 % | 14.9 | 2.0 | 31.4 | - |
| Le Conquet | `refmar:152` | 2025 | 8727 | 99.6 % | 13.0 | 2.5 | 25.6 | - |
| Le Conquet | `refmar:152` | 2026 | 4343 | 100.0 % | 16.2 | 4.2 | 37.3 | - |
| Concarneau | `refmar:160` | 2016 | 8784 | 100.0 % | 14.6 | -6.8 | 27.9 | oui |
| Concarneau | `refmar:160` | 2017 | 8760 | 100.0 % | 16.4 | -10.0 | 30.7 | oui |
| Concarneau | `refmar:160` | 2018 | 8760 | 100.0 % | 15.0 | -5.7 | 29.5 | oui |
| Concarneau | `refmar:160` | 2019 | 8760 | 100.0 % | 15.3 | -5.0 | 31.0 | oui |
| Concarneau | `refmar:160` | 2020 | 8784 | 100.0 % | 12.6 | -1.9 | 24.9 | - |
| Concarneau | `refmar:160` | 2021 | 8616 | 98.4 % | 13.6 | -4.3 | 25.5 | - |
| Concarneau | `refmar:160` | 2022 | 8760 | 100.0 % | 12.8 | -4.1 | 24.9 | - |
| Concarneau | `refmar:160` | 2023 | 8760 | 100.0 % | 15.7 | 0.1 | 31.3 | - |
| Concarneau | `refmar:160` | 2024 | 8784 | 100.0 % | 14.8 | 2.4 | 32.0 | - |
| Concarneau | `refmar:160` | 2025 | 8760 | 100.0 % | 13.5 | 3.4 | 26.9 | - |
| Concarneau | `refmar:160` | 2026 | 4339 | 99.9 % | 16.8 | 4.8 | 38.8 | - |
| Le Crouesty | `refmar:185` | 2016 | 8784 | 100.0 % | 15.4 | -6.3 | 29.3 | - |
| Le Crouesty | `refmar:185` | 2017 | 8760 | 100.0 % | 16.9 | -9.2 | 32.2 | oui |
| Le Crouesty | `refmar:185` | 2018 | 8721 | 99.6 % | 15.7 | -5.2 | 31.0 | - |
| Le Crouesty | `refmar:185` | 2019 | 8757 | 100.0 % | 16.1 | -3.5 | 32.2 | - |
| Le Crouesty | `refmar:185` | 2020 | 8784 | 100.0 % | 13.6 | -0.6 | 27.3 | - |
| Le Crouesty | `refmar:185` | 2021 | 8740 | 99.8 % | 14.5 | -4.0 | 27.6 | - |
| Le Crouesty | `refmar:185` | 2022 | 8760 | 100.0 % | 13.5 | -4.0 | 26.3 | - |
| Le Crouesty | `refmar:185` | 2023 | 8760 | 100.0 % | 17.0 | 0.4 | 34.4 | - |
| Le Crouesty | `refmar:185` | 2024 | 8784 | 100.0 % | 16.0 | 2.5 | 34.8 | - |
| Le Crouesty | `refmar:185` | 2025 | 8604 | 98.2 % | 14.4 | 2.9 | 27.6 | - |
| Le Crouesty | `refmar:185` | 2026 | 4284 | 98.6 % | 17.3 | 4.4 | 39.4 | - |
| Arcachon Eyrac | `refmar:190` | 2016 | 3168 | 36.1 % | 20.8 | -4.5 | 40.0 | oui |
| Arcachon Eyrac | `refmar:190` | 2017 | 8753 | 99.9 % | 18.3 | -7.9 | 35.9 | - |
| Arcachon Eyrac | `refmar:190` | 2018 | 8650 | 98.7 % | 15.5 | -3.5 | 31.3 | - |
| Arcachon Eyrac | `refmar:190` | 2019 | 8285 | 94.6 % | 18.7 | -1.8 | 37.2 | - |
| Arcachon Eyrac | `refmar:190` | 2020 | 8784 | 100.0 % | 16.4 | 0.4 | 32.6 | - |
| Arcachon Eyrac | `refmar:190` | 2021 | 8756 | 100.0 % | 15.7 | -3.6 | 29.9 | - |
| Arcachon Eyrac | `refmar:190` | 2022 | 8760 | 100.0 % | 14.8 | -3.8 | 29.4 | - |
| Arcachon Eyrac | `refmar:190` | 2023 | 8760 | 100.0 % | 19.9 | 1.8 | 40.0 | - |
| Arcachon Eyrac | `refmar:190` | 2024 | 8757 | 99.7 % | 16.7 | 1.5 | 33.5 | - |
| Arcachon Eyrac | `refmar:190` | 2025 | 8760 | 100.0 % | 15.3 | 1.4 | 29.6 | - |
| Arcachon Eyrac | `refmar:190` | 2026 | 4343 | 100.0 % | 19.0 | 6.5 | 43.5 | - |
| Herbaudiere | `refmar:198` | 2016 | 8762 | 99.7 % | 15.4 | -6.6 | 29.2 | - |
| Herbaudiere | `refmar:198` | 2017 | 8760 | 100.0 % | 16.8 | -9.4 | 32.2 | oui |
| Herbaudiere | `refmar:198` | 2018 | 5226 | 59.7 % | 15.3 | -3.4 | 29.9 | - |
| Herbaudiere | `refmar:198` | 2019 | 8690 | 99.2 % | 15.6 | -3.5 | 31.7 | - |
| Herbaudiere | `refmar:198` | 2020 | 8703 | 99.1 % | 13.5 | -1.0 | 26.8 | - |
| Herbaudiere | `refmar:198` | 2021 | 8717 | 99.5 % | 14.1 | -4.2 | 27.2 | - |
| Herbaudiere | `refmar:198` | 2022 | 8751 | 99.9 % | 13.2 | -4.4 | 26.2 | - |
| Herbaudiere | `refmar:198` | 2023 | 8757 | 100.0 % | 16.8 | 1.1 | 35.0 | - |
| Herbaudiere | `refmar:198` | 2024 | 8755 | 99.7 % | 15.3 | 2.8 | 31.7 | - |
| Herbaudiere | `refmar:198` | 2025 | 8633 | 98.6 % | 13.8 | 2.4 | 26.8 | - |
| Herbaudiere | `refmar:198` | 2026 | 4279 | 98.5 % | 16.1 | 4.6 | 37.1 | - |
| Dunkerque | `refmar:2` | 2016 | 8784 | 100.0 % | 21.4 | -3.5 | 43.3 | - |
| Dunkerque | `refmar:2` | 2017 | 8736 | 99.7 % | 22.0 | -0.6 | 43.9 | - |
| Dunkerque | `refmar:2` | 2018 | 8760 | 100.0 % | 22.3 | -4.3 | 46.3 | - |
| Dunkerque | `refmar:2` | 2019 | 8760 | 100.0 % | 22.1 | -1.4 | 44.1 | - |
| Dunkerque | `refmar:2` | 2020 | 8781 | 100.0 % | 23.2 | -0.9 | 45.9 | - |
| Dunkerque | `refmar:2` | 2021 | 8631 | 98.5 % | 21.3 | -3.3 | 42.6 | - |
| Dunkerque | `refmar:2` | 2022 | 8760 | 100.0 % | 21.9 | -2.0 | 43.9 | - |
| Dunkerque | `refmar:2` | 2023 | 8760 | 100.0 % | 23.0 | 2.8 | 46.4 | - |
| Dunkerque | `refmar:2` | 2024 | 8711 | 99.2 % | 23.3 | 2.8 | 46.9 | - |
| Dunkerque | `refmar:2` | 2025 | 8760 | 100.0 % | 20.9 | -1.2 | 41.8 | - |
| Dunkerque | `refmar:2` | 2026 | 4343 | 100.0 % | 21.5 | 2.2 | 44.3 | - |
| Dieppe | `refmar:24` | 2016 | 8779 | 99.9 % | 21.2 | -6.2 | 42.0 | - |
| Dieppe | `refmar:24` | 2017 | 8688 | 99.2 % | 21.0 | -6.7 | 41.6 | - |
| Dieppe | `refmar:24` | 2018 | 8760 | 100.0 % | 21.4 | -6.7 | 43.6 | - |
| Dieppe | `refmar:24` | 2019 | 8760 | 100.0 % | 21.2 | -3.7 | 42.2 | - |
| Dieppe | `refmar:24` | 2020 | 8784 | 100.0 % | 21.4 | -1.6 | 42.6 | - |
| Dieppe | `refmar:24` | 2021 | 8437 | 96.3 % | 20.3 | -4.0 | 39.9 | - |
| Dieppe | `refmar:24` | 2022 | 8699 | 99.3 % | 20.3 | -3.6 | 40.6 | - |
| Dieppe | `refmar:24` | 2023 | 8760 | 100.0 % | 22.7 | 1.8 | 45.8 | - |
| Dieppe | `refmar:24` | 2024 | 8780 | 100.0 % | 22.2 | 3.1 | 44.0 | - |
| Dieppe | `refmar:24` | 2025 | 8760 | 100.0 % | 20.1 | 0.9 | 40.2 | - |
| Dieppe | `refmar:24` | 2026 | 4337 | 99.8 % | 19.6 | 2.9 | 38.2 | - |
| Brest | `refmar:3` | 2016 | 8659 | 98.6 % | 15.3 | -6.8 | 29.8 | - |
| Brest | `refmar:3` | 2017 | 8730 | 99.7 % | 16.4 | -9.3 | 31.4 | oui |
| Brest | `refmar:3` | 2018 | 8750 | 99.9 % | 14.9 | -5.0 | 29.0 | - |
| Brest | `refmar:3` | 2019 | 8496 | 97.0 % | 15.3 | -3.8 | 30.8 | - |
| Brest | `refmar:3` | 2020 | 8699 | 99.0 % | 13.6 | -1.5 | 26.8 | - |
| Brest | `refmar:3` | 2021 | 8669 | 99.0 % | 13.8 | -4.0 | 26.1 | - |
| Brest | `refmar:3` | 2022 | 8756 | 100.0 % | 13.7 | -4.0 | 27.4 | - |
| Brest | `refmar:3` | 2023 | 8760 | 100.0 % | 16.5 | 1.1 | 32.4 | - |
| Brest | `refmar:3` | 2024 | 8262 | 94.1 % | 15.3 | 2.8 | 32.3 | - |
| Brest | `refmar:3` | 2025 | 8760 | 100.0 % | 13.5 | 2.2 | 26.5 | - |
| Brest | `refmar:3` | 2026 | 4343 | 100.0 % | 16.6 | 3.8 | 37.8 | - |
| Ouistreham | `refmar:311` | 2016 | 1787 | 20.3 % | 25.2 | -12.6 | 50.0 | - |
| Ouistreham | `refmar:311` | 2017 | 8718 | 99.5 % | 22.2 | -3.6 | 44.2 | - |
| Ouistreham | `refmar:311` | 2018 | 8219 | 93.8 % | 22.3 | -2.0 | 44.0 | - |
| Ouistreham | `refmar:311` | 2019 | 8760 | 100.0 % | 22.3 | 0.1 | 44.7 | - |
| Ouistreham | `refmar:311` | 2020 | 8784 | 100.0 % | 22.3 | 1.5 | 43.8 | - |
| Ouistreham | `refmar:311` | 2021 | 8731 | 99.7 % | 21.7 | -0.4 | 42.6 | - |
| Ouistreham | `refmar:311` | 2022 | 8752 | 99.9 % | 22.1 | -3.9 | 43.1 | - |
| Ouistreham | `refmar:311` | 2023 | 8760 | 100.0 % | 23.4 | 0.7 | 46.6 | - |
| Ouistreham | `refmar:311` | 2024 | 8637 | 98.3 % | 23.0 | 2.2 | 46.1 | - |
| Ouistreham | `refmar:311` | 2025 | 8725 | 99.6 % | 21.8 | -0.1 | 42.4 | - |
| Ouistreham | `refmar:311` | 2026 | 4343 | 100.0 % | 21.9 | 1.8 | 43.2 | - |
| La Rochelle-pallice | `refmar:34` | 2016 | 8784 | 100.0 % | 16.0 | -6.2 | 30.5 | - |
| La Rochelle-pallice | `refmar:34` | 2017 | 8676 | 99.0 % | 17.6 | -9.0 | 33.9 | - |
| La Rochelle-pallice | `refmar:34` | 2018 | 8165 | 93.2 % | 15.8 | -5.0 | 30.9 | - |
| La Rochelle-pallice | `refmar:34` | 2019 | 8760 | 100.0 % | 16.9 | -2.9 | 33.9 | - |
| La Rochelle-pallice | `refmar:34` | 2020 | 8772 | 99.9 % | 14.6 | -0.5 | 29.5 | - |
| La Rochelle-pallice | `refmar:34` | 2021 | 8630 | 98.5 % | 14.9 | -3.6 | 28.8 | - |
| La Rochelle-pallice | `refmar:34` | 2022 | 8716 | 99.5 % | 14.2 | -3.4 | 28.0 | - |
| La Rochelle-pallice | `refmar:34` | 2023 | 8077 | 92.2 % | 14.8 | -2.1 | 31.0 | - |
| La Rochelle-pallice | `refmar:34` | 2024 | 7078 | 80.6 % | 14.5 | 3.2 | 29.1 | - |
| La Rochelle-pallice | `refmar:34` | 2025 | 8760 | 100.0 % | 15.2 | 3.6 | 30.0 | - |
| La Rochelle-pallice | `refmar:34` | 2026 | 4343 | 100.0 % | 17.6 | 5.6 | 39.5 | - |
| Saint-nazaire | `refmar:37` | 2016 | 8749 | 99.6 % | 17.5 | -6.2 | 33.7 | - |
| Saint-nazaire | `refmar:37` | 2017 | 8760 | 100.0 % | 18.7 | -9.9 | 36.1 | oui |
| Saint-nazaire | `refmar:37` | 2018 | 8760 | 100.0 % | 17.2 | -5.5 | 34.1 | - |
| Saint-nazaire | `refmar:37` | 2019 | 8760 | 100.0 % | 17.8 | -3.5 | 36.0 | - |
| Saint-nazaire | `refmar:37` | 2020 | 8784 | 100.0 % | 15.9 | -0.9 | 31.3 | - |
| Saint-nazaire | `refmar:37` | 2021 | 8756 | 100.0 % | 16.4 | -4.1 | 31.7 | - |
| Saint-nazaire | `refmar:37` | 2022 | 8760 | 100.0 % | 15.1 | -4.9 | 29.2 | - |
| Saint-nazaire | `refmar:37` | 2023 | 8760 | 100.0 % | 19.6 | 1.0 | 39.6 | - |
| Saint-nazaire | `refmar:37` | 2024 | 8766 | 99.8 % | 18.1 | 3.9 | 37.4 | - |
| Saint-nazaire | `refmar:37` | 2025 | 8760 | 100.0 % | 15.5 | 1.5 | 29.6 | - |
| Saint-nazaire | `refmar:37` | 2026 | 4343 | 100.0 % | 18.0 | 4.3 | 40.9 | - |
| Le Havre | `refmar:4` | 2016 | 8773 | 99.9 % | 24.6 | -5.6 | 49.2 | - |
| Le Havre | `refmar:4` | 2017 | 8756 | 100.0 % | 24.8 | -5.3 | 49.3 | - |
| Le Havre | `refmar:4` | 2018 | 8754 | 99.9 % | 24.7 | -5.4 | 48.8 | - |
| Le Havre | `refmar:4` | 2019 | 8736 | 99.7 % | 25.0 | -2.4 | 49.4 | - |
| Le Havre | `refmar:4` | 2020 | 8700 | 99.0 % | 25.1 | -0.9 | 49.9 | - |
| Le Havre | `refmar:4` | 2021 | 8676 | 99.0 % | 24.2 | -3.8 | 47.6 | - |
| Le Havre | `refmar:4` | 2022 | 8467 | 96.7 % | 24.0 | -3.5 | 47.2 | - |
| Le Havre | `refmar:4` | 2023 | 8705 | 99.4 % | 25.9 | 1.7 | 52.0 | - |
| Le Havre | `refmar:4` | 2024 | 8751 | 99.6 % | 25.1 | 3.1 | 50.3 | - |
| Le Havre | `refmar:4` | 2025 | 8694 | 99.2 % | 23.5 | 0.4 | 47.0 | - |
| Le Havre | `refmar:4` | 2026 | 4343 | 100.0 % | 23.9 | 3.1 | 46.5 | - |
| Saint-malo | `refmar:410` | 2016 | 8784 | 100.0 % | 20.3 | -5.4 | 40.4 | - |
| Saint-malo | `refmar:410` | 2017 | 8760 | 100.0 % | 19.7 | -5.6 | 39.5 | - |
| Saint-malo | `refmar:410` | 2018 | 8760 | 100.0 % | 19.6 | -3.4 | 38.3 | - |
| Saint-malo | `refmar:410` | 2019 | 8760 | 100.0 % | 21.2 | -2.3 | 42.7 | - |
| Saint-malo | `refmar:410` | 2020 | 7070 | 80.5 % | 21.0 | 0.6 | 41.8 | - |
| Saint-malo | `refmar:410` | 2021 | 8755 | 99.9 % | 19.6 | -2.9 | 38.1 | - |
| Saint-malo | `refmar:410` | 2022 | 8760 | 100.0 % | 19.3 | -3.8 | 38.2 | - |
| Saint-malo | `refmar:410` | 2023 | 8760 | 100.0 % | 22.6 | 1.2 | 45.3 | - |
| Saint-malo | `refmar:410` | 2024 | 8784 | 100.0 % | 21.4 | 2.0 | 42.4 | - |
| Saint-malo | `refmar:410` | 2025 | 8760 | 100.0 % | 19.6 | 1.3 | 39.5 | - |
| Saint-malo | `refmar:410` | 2026 | 4343 | 100.0 % | 19.9 | 4.1 | 41.2 | - |
| Roscoff | `refmar:54` | 2016 | 8767 | 99.8 % | 15.5 | -6.2 | 30.5 | - |
| Roscoff | `refmar:54` | 2017 | 8692 | 99.2 % | 15.7 | -7.0 | 30.8 | - |
| Roscoff | `refmar:54` | 2018 | 8682 | 99.1 % | 15.1 | -4.2 | 29.9 | - |
| Roscoff | `refmar:54` | 2019 | 8710 | 99.4 % | 15.6 | -3.0 | 31.2 | - |
| Roscoff | `refmar:54` | 2020 | 8779 | 99.9 % | 14.5 | -1.2 | 28.8 | - |
| Roscoff | `refmar:54` | 2021 | 8698 | 99.3 % | 14.2 | -4.0 | 27.2 | - |
| Roscoff | `refmar:54` | 2022 | 8757 | 100.0 % | 14.3 | -4.1 | 28.4 | - |
| Roscoff | `refmar:54` | 2023 | 8750 | 99.9 % | 17.2 | -0.1 | 33.9 | - |
| Roscoff | `refmar:54` | 2024 | 8731 | 99.4 % | 16.0 | 2.5 | 32.6 | - |
| Roscoff | `refmar:54` | 2025 | 8490 | 96.9 % | 14.4 | 3.4 | 28.5 | - |
| Roscoff | `refmar:54` | 2026 | 4259 | 98.0 % | 16.8 | 5.2 | 39.1 | - |
| Mimizan | `refmar:6144` | 2016 | 8784 | 100.0 % | 19.2 | -1.1 | 36.5 | - |
| Mimizan | `refmar:6144` | 2017 | 7662 | 87.5 % | 22.8 | 0.5 | 48.4 | - |
| Mimizan | `refmar:6144` | 2018 | 7099 | 81.0 % | 21.6 | 3.3 | 43.8 | - |
| Mimizan | `refmar:6144` | 2019 | 8715 | 99.5 % | 21.8 | 3.1 | 44.7 | - |
| Mimizan | `refmar:6144` | 2020 | 8707 | 99.1 % | 19.9 | 1.9 | 39.7 | - |
| Mimizan | `refmar:6144` | 2021 | 8666 | 98.9 % | 21.4 | -5.6 | 41.6 | - |
| Mimizan | `refmar:6144` | 2022 | 8172 | 93.3 % | 17.0 | -6.3 | 33.5 | - |
| Mimizan | `refmar:6144` | 2023 | 8760 | 100.0 % | 22.3 | 3.4 | 44.5 | - |
| Mimizan | `refmar:6144` | 2024 | 8784 | 100.0 % | 18.0 | -0.2 | 36.3 | - |
| Mimizan | `refmar:6144` | 2025 | 8760 | 100.0 % | 26.3 | 6.0 | 56.1 | - |
| Mimizan | `refmar:6144` | 2026 | 4343 | 100.0 % | 20.0 | 2.2 | 41.5 | - |
| Les Sables D Olonne | `refmar:62` | 2016 | 8784 | 100.0 % | 15.0 | -6.7 | 28.4 | - |
| Les Sables D Olonne | `refmar:62` | 2017 | 8760 | 100.0 % | 16.9 | -9.6 | 32.2 | oui |
| Les Sables D Olonne | `refmar:62` | 2018 | 8710 | 99.4 % | 14.9 | -5.4 | 29.2 | - |
| Les Sables D Olonne | `refmar:62` | 2019 | 8722 | 99.6 % | 15.5 | -3.6 | 31.8 | oui |
| Les Sables D Olonne | `refmar:62` | 2020 | 8772 | 99.9 % | 13.1 | -1.0 | 26.3 | - |
| Les Sables D Olonne | `refmar:62` | 2021 | 8694 | 99.2 % | 13.9 | -4.0 | 26.5 | - |
| Les Sables D Olonne | `refmar:62` | 2022 | 8751 | 99.9 % | 12.8 | -4.4 | 25.4 | - |
| Les Sables D Olonne | `refmar:62` | 2023 | 8741 | 99.8 % | 16.4 | 0.7 | 35.0 | - |
| Les Sables D Olonne | `refmar:62` | 2024 | 8474 | 96.5 % | 15.4 | 3.3 | 33.4 | - |
| Les Sables D Olonne | `refmar:62` | 2025 | 8235 | 94.0 % | 14.0 | 2.4 | 27.0 | - |
| Les Sables D Olonne | `refmar:62` | 2026 | 4337 | 99.8 % | 16.0 | 4.1 | 37.0 | - |
| Dielette | `refmar:628` | 2016 | 7361 | 83.8 % | 19.8 | -7.2 | 37.9 | - |
| Dielette | `refmar:628` | 2017 | 8197 | 93.6 % | 19.9 | -7.2 | 38.3 | - |
| Dielette | `refmar:628` | 2018 | 7748 | 88.4 % | 19.4 | -5.0 | 38.0 | - |
| Dielette | `refmar:628` | 2019 | 8760 | 100.0 % | 19.3 | -2.2 | 38.2 | - |
| Dielette | `refmar:628` | 2020 | 7378 | 84.0 % | 19.0 | -3.2 | 37.2 | - |
| Dielette | `refmar:628` | 2021 | 8414 | 96.1 % | 18.2 | -4.6 | 34.9 | - |
| Dielette | `refmar:628` | 2022 | 8629 | 98.5 % | 17.5 | -3.6 | 35.0 | - |
| Dielette | `refmar:628` | 2023 | 8760 | 100.0 % | 21.5 | 2.6 | 42.2 | - |
| Dielette | `refmar:628` | 2024 | 8779 | 99.9 % | 19.3 | 2.0 | 39.3 | - |
| Dielette | `refmar:628` | 2025 | 8649 | 98.7 % | 17.9 | 1.1 | 35.5 | - |
| Dielette | `refmar:628` | 2026 | 4222 | 97.2 % | 18.2 | 5.2 | 38.0 | - |
| Noumea Numbo | `refmar:659` | 2016 | 8277 | 94.2 % | 7.8 | -5.3 | 15.1 | - |
| Noumea Numbo | `refmar:659` | 2017 | 7873 | 89.9 % | 6.6 | -3.8 | 13.2 | - |
| Noumea Numbo | `refmar:659` | 2018 | 8448 | 96.4 % | 7.7 | -5.0 | 14.4 | - |
| Noumea Numbo | `refmar:659` | 2019 | 8735 | 99.7 % | 8.4 | -6.2 | 15.3 | - |
| Noumea Numbo | `refmar:659` | 2020 | 8505 | 96.8 % | 7.2 | -4.1 | 12.9 | - |
| Noumea Numbo | `refmar:659` | 2021 | 7949 | 90.7 % | 6.3 | -3.3 | 12.5 | - |
| Noumea Numbo | `refmar:659` | 2022 | 7307 | 83.4 % | 6.5 | 0.5 | 12.4 | - |
| Noumea Numbo | `refmar:659` | 2023 | 8482 | 96.8 % | 5.9 | 0.2 | 10.6 | - |
| Noumea Numbo | `refmar:659` | 2024 | 8644 | 98.4 % | 5.4 | 1.0 | 10.8 | - |
| Noumea Numbo | `refmar:659` | 2025 | 8753 | 99.9 % | 6.1 | 1.2 | 12.3 | - |
| Noumea Numbo | `refmar:659` | 2026 | 4340 | 99.9 % | 7.0 | -1.5 | 13.7 | - |
| Port-tudy | `refmar:71` | 2016 | 8784 | 100.0 % | 13.6 | -5.3 | 26.1 | - |
| Port-tudy | `refmar:71` | 2017 | 8760 | 100.0 % | 15.0 | -7.8 | 28.6 | oui |
| Port-tudy | `refmar:71` | 2018 | 8701 | 99.3 % | 14.1 | -4.2 | 28.0 | - |
| Port-tudy | `refmar:71` | 2019 | 8760 | 100.0 % | 14.4 | -3.3 | 29.3 | - |
| Port-tudy | `refmar:71` | 2020 | 6914 | 78.7 % | 12.3 | -2.0 | 24.4 | - |
| Port-tudy | `refmar:71` | 2021 | 8608 | 98.3 % | 14.0 | -6.0 | 26.6 | - |
| Port-tudy | `refmar:71` | 2022 | 8760 | 100.0 % | 13.5 | -5.1 | 26.6 | - |
| Port-tudy | `refmar:71` | 2023 | 8760 | 100.0 % | 15.4 | 2.0 | 31.2 | - |
| Port-tudy | `refmar:71` | 2024 | 8784 | 100.0 % | 14.6 | 3.5 | 31.7 | - |
| Port-tudy | `refmar:71` | 2025 | 8760 | 100.0 % | 13.0 | 3.4 | 25.7 | - |
| Port-tudy | `refmar:71` | 2026 | 4343 | 100.0 % | 15.7 | 4.0 | 36.9 | - |
| Boucau-bayonne | `refmar:94` | 2016 | 0 | 0.0 % | NA | NA | NA | - |
| Boucau-bayonne | `refmar:94` | 2017 | 5830 | 66.6 % | 12.8 | -5.0 | 26.7 | - |
| Boucau-bayonne | `refmar:94` | 2018 | 8760 | 100.0 % | 13.4 | 0.7 | 27.8 | - |
| Boucau-bayonne | `refmar:94` | 2019 | 8750 | 99.9 % | 16.1 | 0.6 | 35.3 | oui |
| Boucau-bayonne | `refmar:94` | 2020 | 8753 | 99.6 % | 13.8 | 1.8 | 28.1 | oui |
| Boucau-bayonne | `refmar:94` | 2021 | 8259 | 94.3 % | 12.9 | -2.2 | 24.5 | - |
| Boucau-bayonne | `refmar:94` | 2022 | 8760 | 100.0 % | 11.8 | -6.0 | 23.4 | - |
| Boucau-bayonne | `refmar:94` | 2023 | 8760 | 100.0 % | 15.7 | 1.2 | 33.3 | - |
| Boucau-bayonne | `refmar:94` | 2024 | 7956 | 90.6 % | 13.9 | 4.9 | 30.4 | - |
| Boucau-bayonne | `refmar:94` | 2025 | 8760 | 100.0 % | 11.1 | -0.1 | 22.3 | - |
| Boucau-bayonne | `refmar:94` | 2026 | 4343 | 100.0 % | 15.4 | 4.8 | 36.7 | - |


### Horizon de validité par station

Les packs réémis ajoutent par station `data_version`, `valid_from` et
`valid_until`. Pour les stations REFMAR calibrées,
`valid_from = 2021-01-01T00:00:00Z` et
`valid_until = 2031-04-01T00:00:00Z`, soit fin de calibration 2026-04-01
plus 5 ans. Les SHA des packs réémis sont :

| Pack | SHA-256 | Diff attendu |
|---|---|---|
| `data/packs/noaa_m0.json` | `4ceb3f4a0b7a343ca46abf007f8ef69521be4d5ec517967e5b3b853bba99a2a8` | ajout `data_version` par station NOAA |
| `data/packs/amar-data-brest-experimental.json` | `613e9b6374431256f003e54865930a87b7d9f148ce90437c96272854df2568c3` | ajout `data_version`, `valid_from`, `valid_until` |
| `data/packs/amar-data-france-experimental.json` | `a25d10d3cb3240fbd75375074af74a42d277f39f6700b36e7cb5e810d2478772` | ajout `data_version`, `valid_from`, `valid_until` |

Choix API/CLI : hors `[valid_from, valid_until]`, la réponse conserve la
prédiction astronomique et ajoute le warning `outside_validity_period` avec
`source.valid_until`. C'est un avertissement, pas un refus 422 par défaut :
la courbe astronomique reste utile hors horizon, c'est surtout le niveau
absolu qui dérive. Le mode strict `--strict-validity` transforme ce warning
en refus `outside_validity_period` pour les usages qui veulent une borne dure.

## v0.9 -- Investigation Manche harmonique vs surcote

Date : 2026-07-08.

Objectif : trancher Cherbourg et Calais sans passe-droit, et vérifier Le Havre
et Concarneau comme respectivement port estuarien/double-marée et témoin propre.
Les packs NOAA, Brest et France v0.7 restent inchangés dans cette investigation.
Les seuils de gates restent inchangés.

Artefacts temporaires :

| Artefact | Rôle |
|---|---|
| `target/refmar-cache/manche_investigation_v0_9/report.json` | rapport complet de diagnostic |
| `target/refmar-cache/manche_investigation_v0_9/validation_summary.csv` | 37 courant vs sélection UTide |
| `target/refmar-cache/manche_investigation_v0_9/calm_summary.csv` | p95 Cherbourg/Calais en saison calme |
| `target/refmar-cache/manche_investigation_v0_9/skew_surge_2024.csv` | skew surge horaire 2024 |
| `target/refmar-cache/manche_investigation_v0_9/inverse_barometer.csv` | diagnostic baromètre inverse |

### Méthode

Sélection harmonique : UTide `0.3.1`, catalogue de 146 noms incluant `Z0`,
moindres carrés ordinaires, corrections nodales actives, tendance désactivée,
intervalles de confiance linéaires, `Rayleigh_min = 1`, puis reconstruction
avec `SNR >= 2`. UTide ne contient pas `M1` ni `2MK3`; la baseline 37 de
référence reste donc la mesure AMAR v0.7 exacte, et le jeu UTide commun à
35 constituants ne sert que de repère interne.

Le calcul Rayleigh par port est :

| Port | Calibration | T jours | 1/T cpd | Rayleigh | SNR >= 2 | min écart SNR cpd |
|---|---|---:|---:|---:|---:|---:|
| Cherbourg | 2021-01-01/2026-01-01 | 1826 | 0,000548 | 68 | 65 | 0,002119 |
| Calais | 2021-01-01/2025-08-01 | 1673 | 0,000598 | 68 | 62 | 0,002119 |
| Le Havre | 2021-01-01/2026-04-01 | 1916 | 0,000522 | 68 | 63 | 0,002119 |
| Concarneau | 2021-01-01/2026-04-01 | 1916 | 0,000522 | 68 | 66 | 0,002119 |

Lecture : tous les constituants significatifs retenus restent séparés par un
écart minimal supérieur à la résolution Rayleigh de leur fenêtre. Le juge reste
la validation réservée.

Le skew surge est calculé sur 2024 complète par cycle basse-mer -> basse-mer :
`max(niveau observé) - max(marée astronomique prédite)` dans le cycle. C'est un
diagnostic horaire, pas un produit officiel d'extrêmes interpolés.

Le baromètre inverse utilise la formule Brest M2.2 :
`IB = -0,9933 cm/hPa × (P - 1013,25)`, avec pression horaire Open-Meteo
`surface_pressure`, `timezone=GMT`, `cell_selection=nearest`. Cette donnée
n'entre pas dans la prédiction.

### Validation harmonique

| Port | Validation | RMS 37 cm | p95 37 cm | RMS sélection cm | p95 sélection cm | ΔRMS cm | Δp95 cm | Lecture |
|---|---|---:|---:|---:|---:|---:|---:|---|
| Cherbourg | 2026-01/2026-04 | 20,3 | 41,3 | 20,0 | 39,2 | -0,4 | -2,1 | gain harmonique faible |
| Calais | 2025-08/2025-11 | 23,2 | 44,5 | 22,0 | 44,0 | -1,2 | -0,5 | météo domine |
| Le Havre | 2026-04/2026-07 | 19,5 | 37,4 | 13,7 | 27,6 | -5,8 | -9,8 | manque harmonique réel |
| Concarneau | 2026-04/2026-07 | 7,3 | 14,5 | 7,3 | 14,2 | 0,0 | -0,3 | garde-fou propre |

Le Havre montre le cas attendu d'un port d'estuaire/double-marée : les termes
étendus récupèrent un signal harmonique réel. Concarneau valide le garde-fou :
la sélection peut retenir plus de termes statistiquement visibles, mais elle ne
gagne rien en validation. Cherbourg ne gagne que 2,1 cm de p95 sur sa fenêtre
hivernale ; Calais ne revient pas sous 40 cm. Le dépassement historique de
Cherbourg/Calais n'est donc pas un simple manque du jeu 37.

### Saison calme

Pour comparer à la saison des 21 ports inclus, Cherbourg et Calais ont été
rejoués sur une validation réservée de printemps-été antérieure
`2025-04-01T00:00:00Z/2025-07-01T00:00:00Z`, calibration arrêtée au
`2025-04-01T00:00:00Z`.

Commande AMAR 37 :

```bash
cargo run -p amar-calibrate -- calibrate-france \
  --station 13 --station 55 \
  --validation-start 2025-04-01T00:00:00Z \
  --end 2025-07-01T00:00:00Z \
  --pack-out /private/tmp/amar-manche-calm-pack.json \
  --benchmarks-dir /private/tmp/amar-manche-calm-benchmarks \
  --manifests-dir /private/tmp/amar-manche-calm-manifests \
  --generated-at 2026-07-08-v0.9-manche-calm-diagnostic
```

| Port | Couverture | RMS 37 cm | p95 37 cm | Facteur | RMS sélection cm | p95 sélection cm |
|---|---:|---:|---:|---:|---:|---:|
| Cherbourg | 100,0 % | 10,1 | 20,0 | 14,03 | 9,7 | 18,9 |
| Calais | 100,0 % | 16,8 | 33,0 | 11,33 | 15,1 | 29,6 |

Lecture : sur base saisonnière comparable, Cherbourg et Calais passent tous
deux le critère France `p95 <= 40 cm` et le facteur RMS `>= 2`, même avec le
jeu 37 courant. La fenêtre hiver/automne utilisée en v0.7 était donc
saison-sensible.

### Skew surge 2024

| Port | Couverture | Cycles | médiane cm | p95 signé cm | p95 abs cm | max cm | min cm |
|---|---:|---:|---:|---:|---:|---:|---:|
| Cherbourg | 100,0 % | 706 | 2,3 | 31,6 | 32,5 | 53,0 | -51,4 |
| Calais | 99,5 % | 703 | 3,1 | 35,3 | 43,7 | 81,9 | -74,9 |
| Le Havre | 99,6 % | 706 | 0,4 | 34,4 | 40,0 | 89,6 | -69,6 |
| Concarneau | 100,0 % | 707 | 2,5 | 33,9 | 34,3 | 63,7 | -40,4 |
| Dunkerque | 99,2 % | 701 | 3,1 | 37,1 | 44,4 | 79,8 | -73,5 |

Lecture : le skew surge confirme que la métrique résidu p95 dépend fortement
de la saison et de l'exposition météo. Calais et Dunkerque ont les queues
absolues les plus fortes dans ce lot ; Le Havre et Cherbourg montrent aussi des
pics météo très au-dessus du bruit harmonique.

### Baromètre inverse

| Port | Fenêtre | N | Corr(residu, IB) | r² | Variance retirée | RMS avant cm | RMS après IB cm |
|---|---|---:|---:|---:|---:|---:|---:|
| Cherbourg | 2026-01/2026-04 | 2159 | 0,837 | 70,1 % | 69,4 % | 19,9 | 10,8 |
| Calais | 2025-08/2025-11 | 2175 | 0,608 | 36,9 % | 34,7 % | 22,0 | 18,2 |

Lecture : Cherbourg hiver 2026 est majoritairement un cas de surcote
barométrique dans ce diagnostic simple. Calais conserve une part météo nette
mais moins réductible par la seule pression, ce qui laisse probablement une
part vent/détroit.

### Décision

- Le gate France `p95 <= 40 cm` ne change pas.
- Le p95 sur fenêtre courte est saison-sensible. Les décisions France doivent
  désormais être prises soit sur une saison homogène pour tous les ports, soit
  sur une année complète, avec le skew surge comme caractérisation météo.
- Cherbourg : le dépassement v0.7 est surtout météo/saison. En saison calme
  comparable, le port passe le critère ; il devient candidat pour une nouvelle
  `data_version` France, pas pour une réécriture silencieuse du pack v0.7.
- Calais : le dépassement automne 2025 reste au-dessus même avec extension
  harmonique, mais la saison calme passe. Verdict identique : candidat seulement
  dans une nouvelle `data_version` fondée sur une base saisonnière homogène.
- Le Havre : déjà inclus, mais l'extension révèle un vrai manque harmonique du
  jeu 37. Une future `data_version` avec catalogue étendu doit recalibrer Le
  Havre, sans changer le benchmark v0.7 en place.
- Concarneau : aucun gain matériel ; le garde-fou contre le surajustement est
  satisfait.
- Douvres/BODC n'a pas été consommé : les diagnostics obligatoires suffisent à
  trancher harmonique vs météo pour cette itération.

Conclusion : Cherbourg/Calais ne sont pas des échecs harmoniques purs. Leur
exclusion v0.7 était une conséquence d'une fenêtre hiver/automne plus exposée
que la fenêtre printemps-été des 21 inclus. La bonne correction n'est pas un
relâchement du seuil, mais une méthode de comparaison homogène et, pour les
ports déformés comme Le Havre, une future sélection de constituants par port.

## v0.10 -- catalogue et selection par port

Date : 2026-07-08.

Objectif : transformer l'investigation Manche v0.9 en gain publié sans
réécrire silencieusement les données existantes. Le catalogue harmonique étendu
est porté dans `amar-core` de façon additive : les 37 constituants historiques
conservent leurs paramètres, les constituants petit-fond et composés sont
dérivés par combinaison documentée, et les inconnus restent refusés au
chargement.

Méthode publiée pour les ports concernés :

- départ du catalogue de sélection par port porté par `amar-core` ;
- filtrage Rayleigh sur la fenêtre de calibration du port ;
- ajustement moindres carrés, puis conservation des constituants
  significatifs `SNR >= 2` ;
- validation uniquement sur fenêtre réservée ;
- format pack inchangé, `station_harmonics_v0`.

Le pack France devient `2026-07-08-v0.10-france` :
`data/packs/amar-data-france-experimental.json`, SHA-256
`fdb747b29a4aca7a25d54fa71f71d85c7bf2b539ba5f4b2599d5a8cf508b63ef`.

Table des décisions v0.10 :

| Port | Station | Fenêtre validation | Sélection | RMS cm | p95 cm | Facteur | Décision |
|---|---|---|---:|---:|---:|---:|---|
| Le Havre | `refmar:4` | 2026-04/2026-07 | 68 -> 61 | 13,7 | 27,9 | 14,40 | remplace le modèle 37 dans le pack v0.10 |
| Cherbourg | `refmar:13` | 2025-04/2025-07 | 68 -> 62 | 9,6 | 19,0 | 14,86 | inclus |
| Calais | `refmar:55` | 2025-04/2025-07 | 68 -> 61 | 15,0 | 29,0 | 12,68 | inclus |

Avant/après Le Havre : le modèle v0.7 à 37 constituants faisait RMS 19,5 cm,
p95 37,4 cm et facteur 10,14 sur `2026-04/2026-07`. La sélection v0.10
ramène RMS à 13,7 cm et p95 à 27,9 cm sur la même validation réservée. C'est le
gain harmonique attendu de l'investigation v0.9.

Cherbourg et Calais gardent le critère France inchangé : `p95 <= 40 cm` et
facteur RMS `>= 2`. Ils sont inclus sur leur fenêtre saison calme comparable
`2025-04-01T00:00:00Z/2025-07-01T00:00:00Z`.

Note météo : le résidu toutes-saisons reste dominé par la surcote météo
(Cherbourg r² baromètre ≈ 70 %). Cherbourg et Calais sont inclus sur fenêtre
saison calme comparable ; le skew surge annuel est une caractérisation météo.
Les fenêtres hiver/automne peuvent dépasser le seuil et ne sont pas
dissimulées par cette inclusion.

Benchmarks additifs publiés :

| Benchmark | Fichier SHA-256 | Checksum interne |
|---|---|---|
| `benchmark_le_havre_v2` | `42864645e6dca81549b583dfb0304ff3859dd43687ec5538d4be0ef365d943fe` | `3695d3505a842c99ad03fd3f9a2f4a28d786968cf0792354eac8e667245e0719` |
| `benchmark_cherbourg_v1` | `aa3f5ba43339252e4344c824829f3794944ced962076b99cf63fd069965853e3` | `a318bf95f32ed98bbbfd0a7d0e08a2cbd3da92ec2091805748e128e82a733818` |
| `benchmark_calais_v1` | `2cd4d23435704780710cf6d709023cbd8e8df2654e2dac6af73822b7ccc6732a` | `11162fa3e9b310a8d8eda99cb38378e5305dfbb4c94a52e69d7e4968ab087606` |

Garde-fous :

- `benchmark_brest_v1` reste byte-identique :
  `d36f445c320c17ba323fbe572e0cb93d45eba846aeff0260ee9d1b3631a6bf6f`.
- Le pack Brest régénéré reste byte-identique :
  `e377e25754e9cbc6a05e732d0dcff2db2c1305b57037432312015ae45d749919`.
- Concarneau rejoué avec la sélection par port donne RMS 7,3 cm et p95
  14,4 cm contre RMS 7,3 cm et p95 14,5 cm dans le pack courant : pas de
  publication, pas de gain artificiel.
