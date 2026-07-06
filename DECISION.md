# Décision M0

Date : 2026-07-06.

## État

M0 est livré avec le moteur dégradé prévu par la règle des 14 jours :
`method = harmonic_basic_no_nodal`.

Le CLI répond hors ligne sur le pack NOAA M0, avec station la plus proche dans
un rayon de 20 km, datum MLLW, source et méthode affichés.

## Validation

Commande :

```bash
make m0-validate
```

Résultat mesuré :

| Station | Régime | Échantillons | Méthode | p95 |
|---|---|---:|---|---:|
| `noaa:8443970` Boston | semi-diurne | 504 | `harmonic_basic_no_nodal` | 41,4 cm |
| `noaa:9414290` San Francisco | mixte | 504 | `harmonic_basic_no_nodal` | 61,2 cm |
| `noaa:8729840` Pensacola | diurne | 504 | `harmonic_basic_no_nodal` | 29,0 cm |
| `noaa:9447130` Seattle | témoin hors réglage | 504 | `harmonic_basic_no_nodal` | 132,8 cm |

## Décision

Continuer seulement après avoir remplacé le mode M0 par
`station_harmonics_v0` avec corrections nodales Schureman complètes et une
convention d'argument astronomique validée contre NOAA.

Ne pas commencer M1 depuis cet état sans corriger ce point : le CLI est
utilisable et traçable, mais la précision M0 est explicitement insuffisante.
