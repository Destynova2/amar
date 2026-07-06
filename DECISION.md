# Décision M0.1

Date : 2026-07-06.

## État

M0.1 remplace le moteur dégradé M0 par `method = station_harmonics_v0`.

Le calcul utilise les définitions historiques NOS de Congen/XTide pour les 37
constituants du pack NOAA M0 : origine de `tau` à 1899-12-31 12:00 UTC,
constantes V0 par constituant, et corrections nodales Schureman `f/u`.

Les constituants inconnus ne sont plus prédits avec une époque de secours
J2000 : ils sont refusés au chargement du modèle.

## Validation

Commande :

```bash
make m0-validate
```

Résultat mesuré :

| Station | Régime | Échantillons | Méthode | p95 |
|---|---|---:|---|---:|
| `noaa:8443970` Boston | semi-diurne | 504 | `station_harmonics_v0` | 4,3 cm |
| `noaa:9414290` San Francisco | mixte | 504 | `station_harmonics_v0` | 2,5 cm |
| `noaa:8729840` Pensacola | diurne | 504 | `station_harmonics_v0` | 0,6 cm |
| `noaa:9447130` Seattle | témoin hors réglage | 504 | `station_harmonics_v0` | 2,1 cm |

Fenêtres de dérive nodale :

| Station | 2026 | 2031 | 2036 |
|---|---:|---:|---:|
| `noaa:8443970` Boston | 4,3 cm | 3,9 cm | 4,4 cm |
| `noaa:9414290` San Francisco | 2,3 cm | 2,4 cm | 2,6 cm |
| `noaa:8729840` Pensacola | 0,5 cm | 0,5 cm | 0,7 cm |
| `noaa:9447130` Seattle | 2,1 cm | 1,8 cm | 2,5 cm |

`make m0-validate` échoue désormais si une p95 station ou fenêtre dépasse
5 cm.
M0.2 échoue aussi si une station n'a aucun échantillon de validation.
M0.2 refuse les constituants inconnus au chargement, avant toute prédiction.

## Décision

Continuer M1 depuis M0.1.

La cible M0 est atteinte : les quatre stations, dont Seattle hors réglage,
restent sous 5 cm de p95 sur 2026, 2031 et 2036.
