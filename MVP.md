# amar — Découpage en MVP

**Version** : v2 — relecture ChatGPT intégrée (tour B, 11 points) + décision **100 % Rust** (toolchain unique, aucun Python nulle part).
**Règle de lecture** : chaque MVP est une preuve autonome, livrable et arrêtable. On peut geler le projet après n'importe lequel sans laisser de zombie. Les dépendances de données renvoient à `DATA.md` ; tout repose sur les catégories A (acquis) et B (récupérable) — rien sur C (inaccessible).

## Kit d'arrêt — obligatoire pour chaque MVP

Pour qu'un M soit « arrêtable proprement », il produit systématiquement :
- un tag Git, un binaire en release, un pack checksummé ;
- un README d'usage, un `LIMITATIONS.md` (ce que ça ne fait pas), un `DECISION.md` (continuer ou geler, et pourquoi) ;
- une commande de reproduction unique : `make m0-validate`, `make m1-smoke`, `make m2-benchmark` ;
- le critère non négociable : si l'artefact n'est pas utilisable par quelqu'un d'autre, le milestone est échoué, même si le code est beau.

---

## M0 — « Le moteur prouvé » (~2 semaines de travail effectif)

**Preuve visée** : *« La somme de sinusoïdes est juste : mon `h(t)` colle aux prédictions officielles NOAA, et je peux le montrer en une commande. »*

- **Contenu** : `CONVENTIONS.md` (timebox 2–3 jours, conventions copiées de NOAA) → `amar-core` (h(t) instantané, types forts, déterministe) → pack **3 stations NOAA de type harmonique** (une semi-diurne, une mixte, une diurne — ex. Boston, San Francisco, Pensacola ; à confirmer, en excluant les stations subordonnées) → CLI `amar tide --lat --lon --at` lisant le pack local → golden tests vs oracle NOAA avec **p95 affiché par la commande de test**.
- **Hors périmètre** : serveur HTTP, Brest, confiance, `/coverage`, release publique.
- **Données** : DATA.md A1 + A2 uniquement (constantes + oracle NOAA). Aucun blocage possible.
- **DoD** : `cargo test` vert avec p95 documenté par station ; `amar tide --lat 37.806 --lon -122.465 --at 2026-08-15T12:00:00Z` répond en offline ; règle des 14 jours respectée (sinon release dégradée `harmonic_basic_no_nodal` quand même — valable **seulement si** la réponse porte `method` + avertissement clair + p95 affiché).
- **Kill check** : si à 14 jours effectifs il n'y a pas de prédiction comparable à NOAA → moteur dégradé publié + itération en public, ou gel propre.
- **Estimation ChatGPT** : 2 semaines crédible mais tendu.

## M1 — « Le curl de la vision » (~2 semaines, cumul ~4)

**Preuve visée** : *« N'importe qui installe amar en une commande et obtient une marée traçable — ou un refus expliqué. »*

- **Contenu** : M0 + `amar-server` (`POST /tide`, `GET /health`, `GET /coverage`), pack étendu à **5–10 stations NOAA** (couverture des régimes), résolveur conservateur (`max_distance_km`, `422 no_supported_source`), réponse complète (datum, source, `data_version`, confiance heuristique, warnings), **release binaire téléchargeable + README à 3 curls** (dont le curl Brest → 422 expliqué : décision produit visible).
- **Hors périmètre** : PM/BM, séries, pression, matrice de validation publique, Brest calculé.
- **Données** : catégorie A. Tâche B4 (sélection des stations).
- **DoD** : la boucle « installer → demander → comprendre pourquoi oui/non » exécutable par un tiers en une commande ; snapshots API ; les 3 cas de test (nominal / hors rayon / invalide).
- **Kill check** : personne (même pas l'auteur) ne peut lancer le binaire en une commande → on ne passe pas à M2, on répare ou on gèle.

## M2 — « Brest expérimental » (3–5 semaines, cumul ~7–9)

**Preuve visée** : *« amar donne la marée à Brest, calibrée par nous, légalement, avec une incertitude honnête. »* C'est la cause de mort n° 1 des deux pré-mortems, traitée dans la fenêtre des 8–10 semaines du critère d'arrêt.

Scindé en interne (même milestone public, deux portes de sortie) :

### M2a — le compilateur de données (Rust, version brutale et bornée)
- **Cadre imposé par la validation finale** : constituants **fixés d'avance** (pas de sélection automatique), moindres carrés **linéaires** sin/cos, une station, une période, diagnostics minimaux. Solveur numérique pris dans une lib d'algèbre linéaire mature (nalgebra ou équivalent), jamais écrit à la main. UTide est un oracle de contre-vérification manuelle, pas un cahier des charges à égaler — le jour où on veut Rayleigh/poids/robustesse, c'est le Lot 8, pas M2a. **Porte de sortie publiée** : si la calibration dérive, on tague l'état, on documente, on gèle Brest — on ne contamine pas le cœur.
- **Contenu** : outil `amar-calibrate` (crate/xtask **Rust** — décision 100 % Rust, pas de Python dans la chaîne) : téléchargement ≥ 1 an d'observations REFMAR Brest (Licence Ouverte 2.0, attribution ; attention au type de produit — séries « validées horaires » filtrées, lacunes > 1,5 h à gérer) → contrôle qualité → **réservation immédiate d'une fenêtre d'observations hors calibration** (le futur `benchmark_brest_v1` de M2b en dépend — la découvrir manquante en M2b coûterait une recalibration complète) → moindres carrés harmoniques → constantes + Z₀ → pack `amar-data-brest-experimental` (flags `experimental`, `not_official`, `not_shom`, champs `calibration_period`/`validation_period`) avec **datum actionnable** (zéro hydrographique via RAM Shom). Discipline « compilateur de données » : `Cargo.lock`, checksums des observations d'entrée, exécution idempotente, artefact JSON final commité. Contre-vérification ponctuelle des constantes avec UTide **à la main, hors chaîne de build** — jamais une dépendance.
- **Porte de sortie** : si le pack existe et est reproductible, M2a est un livrable même si M2b traîne.

### M2b — exposition + benchmark
- **Contenu** : la station Brest servie par `/tide` et le CLI ; **`benchmark_brest_v1` figé** (timestamps, masque de lacunes, datum, observations, checksums publiés) sur une période **hors calibration** ; vocabulaire imposé « résidu = observé − marée astronomique prédite, météo incluse » ; RMS + biais + MAE + p95 + max, deux baselines (release précédente, modèle naïf). **Pas de grade A/B pour Brest au début** : champs `experimental`, `residual_benchmark_cm`, `validation_period`, `not_shom`.
- **Garde anti-aspiration** (le vrai risque d'ordre selon la revue) : M2 se limite à une station, un datum, une période, un pack. Tout élargissement attend l'après-tripwire.
- **Hors périmètre** : autres ports FR, généralisation de l'outil de calibration (durcissement v0.4), toute prétention d'officialité.
- **Données** : catégorie B1 + B3 (observations REFMAR + RAM). Risque principal : qualité des séries → parade : période de calibration choisie après inspection des trous.
- **DoD** : `amar tide --lat 48.383 --lon -4.495 --at <demain>` répond une hauteur / ZH Brest avec σ élargi et flags ; le README raconte l'usage brestois réel (seuil de sortie kayak/pêche à pied) ; matrice à deux régimes publiée (NOAA : erreur moteur ; Brest : résidu benchmark).
- **Kill check (fusionné des pré-mortems)** : à 8–10 semaines de travail réel il faut ensemble : binaire une commande + prédiction NOAA reproductible + cas Brest utile. Sinon gel propre (publier conventions, scripts, leçons).
- **Estimation ChatGPT** : 2–3 semaines optimiste, plutôt **3–5** avec le nettoyage REFMAR/datum réel — et le choix 100 % Rust (moindres carrés à écrire, pas de UTide) charge M2a : la garde « une station » n'en est que plus stricte.

## M3 — « La raison de revenir » (optionnel, post-tripwire, ~2 semaines)

**Preuve visée** : *« amar répond à la vraie question : est-ce que je peux sortir demain matin ? »*

- **Contenu** : `next_high`/`next_low`, séries (`duration_h`/`step_min`), **fenêtres de marée** (`hauteur > seuil` entre deux dates) — en API et CLI. C'est le G5 du pré-mortem : un `/tide` instantané est un moteur, pas un produit.
- **Données** : aucune nouvelle.
- **DoD** : `amar window --above 4.5m --from ... --to ...` sur Brest et sur une station NOAA ; iCal en backlog.
- **Note** : M3 n'entre PAS dans le critère d'arrêt — c'est la première itération *après* la survie.

## Séquence et garde-fous

| Semaine (effectif) | Jalon | Tripwire |
|---|---|---|
| 0–2 | M0 | Règle des 14 jours (moteur dégradé plutôt que tunnel) |
| 2–4 | M1 | Binaire une commande, sinon stop |
| 4–9 | M2 (M2a puis M2b) | Cas Brest utile ; une station, un datum, une période, un pack |
| 8–10 | **Point d'arrêt global** | Les 3 preuves ensemble, sinon gel propre |
| 10+ | M3 puis v0.2 (matrice) | — |

**Verdict ChatGPT (tour B)** : découpage validé (M0 = moteur, M1 = promesse curl, M2 = ancrage Brest, pas de fusion M0/M1) ; M2 avant matrice publique acceptable *grâce au* benchmark figé — dangereux seulement si Brest est présenté comme validé « métrologiquement » ; M3 hors tripwire confirmé (« PM/BM/fenêtres sont utiles, mais seulement après un noyau vivant »).

**Ordre assumé** : la validation lourde (matrice publique, v0.2 du plan) vient APRÈS M2 — les pré-mortems sont formels : la validation parfaite avant l'usage tue la livraison. M0–M2 embarquent chacun leur validation minimale suffisante (golden tests p95 ; résidus hors période de calibration).
