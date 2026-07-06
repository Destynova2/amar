# Décisions M0-M1

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

Le refus hors rayon est volontairement utile. Brest
`lat=48.383 lon=-4.495` retourne `422 no_supported_source` avec Eastport comme
station la plus proche à 4652,019 km. Brest reste donc un cas dogfood visible,
mais non calculé avant M2.

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
