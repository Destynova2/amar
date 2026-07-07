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
