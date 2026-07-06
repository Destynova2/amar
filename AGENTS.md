# Instructions dépôt

## Ordre de lecture

Lire dans cet ordre avant toute intervention :

1. `CONVENTIONS.md`
2. `DECISION.md`
3. `LIMITATIONS.md`
4. `DATA_LICENSES.md`

## Gates

Tout doit être vert avant commit :

- `make m0-validate` : p95 ≤ 2 cm par station NOAA
- `make m1-smoke`
- `make m2-benchmark` : p95 ≤ 19 cm sur benchmark figé
- `make m3-check` : hilo |Δt| p95 ≤ 10 min, |Δh| ≤ 3 cm

Les seuils de gates ne changent pas en itération de durcissement.

## Règles d'or

- `benchmark_brest_v1` est immuable : byte-identique, SHA publié.
- Seattle `9447130` est la station témoin et ne sert jamais au réglage.
- Les commits sont conventionnels, en anglais, sans mention d'IA ni
  `Co-authored-by`.
- Le périmètre du brief est strict : pas de scope creep.
- Les documents de plan ne se modifient jamais : `PLAN.md`, `MVP.md`,
  `DATA.md`, `PREMORTEM.md`, `prompt.md`.

## Layering

`amar-core` et `amar-pack` sont purs et sans I/O. Le sens des dépendances est :

`core/pack (purs, zéro I/O) ← data ← amar (CLI+serveur)`

`amar-calibrate` reste à côté. Son interface avec le produit est constituée
d'artefacts JSON commités.

Le module `contract` porte tout ce qui est partagé entre CLI et serveur.
`amar-pack` porte tout schéma inter-crates.

## Licences données

- NOAA : domaine public.
- REFMAR : Licence Ouverte 2.0 avec attribution.
- IOC/GESLA : jamais embarqués.
- Le disclaimer non-SHOM est obligatoire sur les stations calibrées.
