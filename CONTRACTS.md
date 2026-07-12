# Contrats executables

Ce fichier liste les invariants qui doivent rester vérifiables par test ou
gate. Les conventions de domaine détaillées restent dans `CONVENTIONS.md`.

## Convention harmonique annuelle

Le moteur publié `station_harmonics_v0` suit la convention annuelle NOS :

- `V0` est calculé au 1er janvier UTC de l'année civile ;
- l'argument avance ensuite avec la vitesse du constituant ;
- les corrections nodales `f/u` sont calculées au milieu de l'année civile ;
- la phase Greenwich publiée est soustraite après argument et correction
  nodale.

Changer cette convention change les résultats numériques et doit être traité
comme une nouvelle méthode, jamais comme un refactor interne.

## Pureté du moteur

`amar-core` est pur :

- pas d'I/O ;
- pas d'accès horloge ;
- pas de fuseau local ;
- `predict_height` dépend uniquement du modèle et du timestamp UTC.

Les packs et benchmarks figés ne doivent pas être régénérés par une exécution
du moteur.

## Traversées de seuil

`threshold_crossings` doit trouver toutes les traversées de seuil dans
`[from,to]` pour les signes observables par l'échantillonnage interne de six
minutes, puis raffiner chaque racine à la tolérance contractuelle.

`tide_windows` peut clamper ses sorties à `[from,to]`, mais ne doit pas perdre
une traversée interne. Tout changement de marge de scan doit prouver que les
fenêtres retournées restent identiques.

## Artefacts figés

Les benchmarks REFMAR et les packs publiés sont byte-identiques entre
releases, sauf décision explicite de nouvelle version de données.

Le gate `make check-frozen-shas` épingle :

- `fixtures/refmar/benchmark_brest_v1.json` ;
- `fixtures/refmar/benchmark_brest_decennial_v1.json` ;
- `fixtures/refmar/benchmarks/*.json` ;
- `data/packs/noaa_m0.json` ;
- `data/packs/amar-data-brest-experimental.json` ;
- `data/packs/amar-data-france-experimental.json`.

Le gate vérifie aussi la cohérence interne des benchmarks. Un changement de
SHA figé bloque la livraison.
