# amar -- conventions harmoniques M0

Ce fichier fige les conventions du moteur M0. Il copie les conventions NOAA
quand elles sont observables dans les constantes `harcon.json`.

## Unites

- Amplitudes : metres.
- Phases : degres, referencees a Greenwich (`phase_GMT` NOAA).
- Vitesses : degres par heure.
- Temps : entrees RFC 3339 avec offset, converties en UTC. `Z` reste la
  forme canonique des exemples.
- Hauteur : positive vers le haut.
- Datum : celui du pack. Le pack NOAA M0 publie des hauteurs en MLLW.

## Formule

La convention canonique est :

```text
h(t) = Z0 + sum_i f_i * A_i * cos(V_i(t) + u_i - phi_i)
```

Avec :

- `Z0` : niveau moyen de la mer (`MSL`) dans le datum de sortie.
- `A_i` : amplitude du constituant en metres.
- `phi_i` : phase Greenwich NOAA en degres.
- `V_i(t)` : argument astronomique Greenwich en degres.
- `f_i`, `u_i` : corrections nodales Schureman.

## Epoque et arguments

Les arguments astronomiques M0 utilisent les variables `tau, s, h, p, N, pp`
calculees sur UTC avec les polynomes de l'Explanatory Supplement repris par
les implementations T_Tide/UTide :

- epoque de calcul : 1899-12-31 12:00 UTC ;
- unite interne : cycles ;
- `tau` : temps lunaire moyen Greenwich ;
- ordre d'application : calcul de `V_i(t)`, puis corrections nodales `f/u`,
  puis soustraction de `phase_GMT`.

## Etat courant (M0.3)

Le moteur courant livre `method = station_harmonics_v0`.

Cela signifie :

- les constantes harmoniques NOAA sont interpretees autour de `MSL` ;
- le pack NOAA M0 publie en `MLLW`, donc `Z0 = MSL - MLLW` ;
- pour chaque annee civile UTC predite, `V0` est calcule au 1er janvier
  00:00:00 UTC ;
- `V_i(t)` avance ensuite avec la vitesse du constituant et le nombre d'heures
  ecoulees depuis ce 1er janvier ;
- les corrections nodales Schureman `u_i` et `f_i` sont calculees au milieu de
  l'annee civile, puis gardees constantes sur l'annee ;
- `make m0-validate` est un gate bloquant : exit 1 si un p95 depasse 2 cm.

Cette convention reproduit la famille NOS/XTide/libtide observee dans les
tables congen. La variante testee avec `u_i` au 1er janvier et `f_i` au milieu
de l'annee a ete rejetee : elle degradait Boston et Eastport au-dessus du gate
5 cm et laissait un biais constant.

Le bareme de confiance M1 reste separe du moteur harmonique : A <= 2 km,
B <= 10 km, C <= 20 km. Au-dela de 20 km, la source est refusee.

## Brest experimental M2/M2.2

Le pack `amar-data-brest-experimental.json` utilise la meme convention
`station_harmonics_v0` que NOAA : `V0` au 1er janvier UTC, avance par vitesse
du constituant, puis corrections nodales `f/u` au milieu de l'annee civile.
Le calibrateur ajuste donc directement des colonnes lineaires
`f*cos(V+u)` et `f*sin(V+u)`.

La liste M2 historique, conservee comme reference de comparaison
`m2-base16`, etait :

```text
M2, S2, N2, K2, K1, O1, P1, Q1, M4, MS4, MN4, M6, MF, MM, SA, SSA
```

La liste M2.2 publiee, `m22-rayleigh37`, est egalement fixee d'avance :

```text
M2, S2, N2, K2, K1, O1, P1, Q1, M4, MS4, MN4, M6, MF, MM, SA, SSA,
L2, NU2, MU2, 2N2, LAM2, T2, R2, J1, OO1, RHO, 2Q1, M1, S1,
MK3, 2MK3, M3, S4, S6, M8, MSF, 2SM2
```

Elle est l'intersection entre les 37 constituants NOAA supportes par
`amar-core` et le critere de Rayleigh sur la fenetre de calibration Brest.
Le plus petit ecart de frequence entre deux constituants retenus vaut
0,002738 cycle/jour. Il est superieur a la resolution Rayleigh de la fenetre
M2 historique de 455 jours, soit 1/455 = 0,002198 cycle/jour, et a celle de
la fenetre M2.2 de 1916 jours, soit 1/1916 = 0,000522 cycle/jour.

Le calibrateur ne fait aucune selection automatique, aucun critere de Rayleigh
dynamique et aucune ponderation robuste : c'est volontairement un compilateur
borne pour Brest, pas un clone UTide.

Pour Brest, le vocabulaire impose est : résidu = niveau d'eau observé − marée
astronomique prédite (météo incluse). Ce résidu n'est pas une validation
officielle.
