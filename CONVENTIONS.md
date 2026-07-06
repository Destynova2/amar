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
h(t) = Z0 + sum_i f_i * A_i * cos(V_i(t) + u_i(t) - phi_i)
```

Avec :

- `Z0` : niveau moyen du pack dans le datum de sortie.
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

## Etat courant (M1)

Le moteur courant livre `method = station_harmonics_v0`.

Cela signifie :

- `V_i(t)` est calcule pour les constituants NOAA M0 ;
- les corrections nodales Schureman `f_i` et `u_i` sont appliquees ;
- `make m0-validate` est un gate bloquant : exit 1 si un p95 depasse 5 cm.

Le bareme de confiance M1 reste separe du moteur harmonique : A <= 2 km,
B <= 10 km, C <= 20 km. Au-dela de 20 km, la source est refusee.
