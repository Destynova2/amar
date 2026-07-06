# amar — Plan projet

> **Pitch** : marée astronomique prédite à partir d'un jeu de constantes versionné, pour un datum explicite, hors ligne, avec provenance vérifiable et refus honnête hors zone de validité. On envoie un point GPS et une date en `curl`, la marée revient.

**Statut** : planification **terminée** — verdict croisé GO-avec-conditions. Le prochain artefact est un commit M0 exécutable, pas une modification de ce document.
**Version du plan** : v2.4 (2026-07-06), gelée. Voir `PREMORTEM.md`, `DATA.md`, `MVP.md`.

---

## 1. Vision

**Objectif v0.1 (mot pour mot)** : « Je peux faire un `curl` hors ligne avec lat/lon/date et obtenir une marée astronomique traçable près d'une station connue. »

```bash
curl -X POST http://localhost:3000/tide -d '{
  "lat": 37.806, "lon": -122.465,
  "datetime": "2026-03-21T12:00:00Z"
}'
```

```json
{
  "height_m": 1.42,
  "datum": "MLLW",
  "source": {
    "kind": "station", "id": "noaa:9414290", "name": "San Francisco",
    "distance_km": 1.8, "data_version": "2026-xx"
  },
  "confidence": {
    "grade": "A", "sigma_cm": 8,
    "method": "station_harmonics_v0_distance_heuristic"
  },
  "warnings": ["astronomical_tide_only", "not_for_navigation", "no_weather_surge"]
}
```

Le MVP ne prouve pas « le monde ». Il prouve : **conventions justes, datum explicite, réponse offline, refus honnête, provenance vérifiable.**

## 2. Principes directeurs

1. **Le vrai noyau n'est ni Axum ni Rust** : conventions harmoniques, datums, provenance/licences, validation indépendante, refus propre.
2. **100 % Rust** (décision du 2026-07-06) : une seule toolchain, du moteur au compilateur de données. L'outil de calibration (`amar-calibrate`, moindres carrés harmoniques) est en Rust. UTide sert de contre-vérification manuelle ponctuelle, hors chaîne de build.
3. **Datum explicite partout** : chaque hauteur, chaque fichier, chaque comparaison porte sa référence verticale.
4. **Refus honnête** : hors rayon de validité, `422 no_supported_source`. Jamais de faux cm.
5. **Binaire nu + data packs signés** : le binaire ne contient aucune donnée ; il charge des packs versionnés, checksummés, avec licence par station.
6. **Souverain = auditable** : manifeste de provenance obligatoire, sinon le mot est du marketing.
7. **Autoportant** : chaque lot réalisable, testable, livrable seul.

## 3. Non-objectifs

- **v0.1** : PM/BM, séries temporelles, coefficient SHOM, pression/baromètre inverse, OpenAPI, page démo, Docker, grille mondiale, plus de ~10 stations.
- **Produit (toutes versions)** : prévision météo/surcote ; navigation (warning permanent `not_for_navigation`) ; précision annuaire hors stations calibrées ; hydrodynamique régionale (note R&D uniquement).
- **Vocabulaire** : on ne dit plus « valable pour toujours » ni « tout point océanique répond ».

## 4. Architecture cible

```
 POST /tide ───────▶ API (Axum) ──┐
 GET  /coverage                   ├▶ Résolveur spatial conservateur
 GET  /health                     │    station à ≤ max_distance_km ?
 amar tide --lat --lon --at ──────┘    zone exclue ? → 422 sinon
 (CLI, même binaire)      │
                          ▼
                 Data packs signés            amar-data-noaa-us
                 (manifeste licence,          amar-data-fr (si licence OK)
                  datum, checksum,            amar-data-eot (plus tard)
                  version)
                          ▼
                 Moteur harmonique pur (crate, zéro I/O, zéro horloge)
                 h(t) = Z₀ + Σ fᵢAᵢcos(ωᵢt+φᵢ+uᵢ)
```

## 5. Sources de données — hiérarchie par sûreté de licence

**Audit détaillé et vérifié en ligne : voir `DATA.md`** (API interrogées, licences confirmées, 2026-07-06). L'essentiel :

| Rang | Source | Statut vérifié | Usage |
|---|---|---|---|
| 1 | **NOAA CO-OPS** | ✅ API testée : 37 constituants/station (mètres, phase GMT) + prédictions officielles par datum | Socle v0.x : données embarquées **et** oracle de validation |
| 2 | **REFMAR observations** (data.shom.fr) | ✅ **Licence Ouverte 2.0 Etalab confirmée** — réutilisation et redistribution des dérivés permises avec attribution | **Le seul chemin légal pour la France** : calibration de nos propres constantes (Brest d'abord) |
| 3 | UHSLC / GESLA / IOC | Recherche / licences mixtes / non-commercial | **Validation interne uniquement**, jamais embarqué |
| 4 | **EOT20** | ✅ **CC-BY 4.0 confirmée** (SEANOE, 17 constituants, grille 1/8°, commercial OK avec attribution) | Candidat unique pour le data pack grille (produit séparé) |
| 5 | ~~Constantes SHOM~~ | ❌ **Vérifié : non distribuées** (position OHI « sécurité » ; demande individuelle = usage privé, redistribution interdite — d'où l'absence de la France dans XTide) | Aucun. Rayé du plan |
| 6 | ~~Prédictions officielles SHOM~~ | ❌ Produits sous licence, payants | Pas d'oracle officiel FR → la validation Brest se fait sur **observations** (voir Lot 4) |
| 7 | ~~FES2022~~ | ❌ Licence AVISO incompatible redistribution | Écarté, remplacé par EOT20 |

## 6. Lots

### Cœur v0.1 — « la prédiction stationnaire traçable et validée »

- **Lot 0 — Socle** : repo, workspace Cargo (`amar-core`, `amar-pack` — la crate **contrat** qui porte le schéma de manifeste, pour que lecteur `amar-data`, écrivain `amar-calibrate` et serveur dépendent du contrat et jamais l'un de l'autre —, `amar-data`, `amar-server`, `amar-calibrate`), CI, licence code (Apache-2.0 : clause de non-responsabilité incluse, la question juridique est tranchée une fois), **`DATA_LICENSES.md` dès le premier commit**, fixtures NOAA figées pour golden tests, et **`CONVENTIONS.md`** — la spec des conventions harmoniques (époque, signe de phase, degrés/radians, échelle de temps UTC/TT, datum, hauteur positive, ordre des corrections nodales). Timebox anti-tunnel : 2–3 jours, chaque convention se décide en copiant NOAA (l'oracle), pas en arbitrant la littérature. DoD : CI verte, manifeste de licence et conventions gelés.
- **Lot 1 — Moteur minimal** (`amar-core`) : prédiction instantanée `h(t)` uniquement, implémentant `CONVENTIONS.md` à la lettre. Arguments astronomiques, corrections nodales, constituants tels que fournis par la station (NOAA ≈ 37). Garde-fous Rust : types forts (`Degrees`, `Radians`, `Meters`, `UtcDateTime`, `ConstituentId`, `PhaseConvention`, `DatumId`), pas de `HashMap<String, f64>` dans la boucle de prédiction (structures compactes triées validées au chargement), zéro I/O, zéro `now()`, zéro timezone locale, pas d'arrondi JSON dans les tests moteur. Anti-bug-compensé : une station golden tenue hors de tout réglage, plus des points de test à +5 et +10 ans (dérive nodale). Règle des 14 jours (anti-tunnel) : si conventions + moteur ne donnent pas une prédiction comparable à NOAA en 14 jours de travail effectif, on livre un moteur dégradé marqué `method: harmonic_basic_no_nodal`, écarts tagués « known imperfect », et on itère en public. DoD : golden tests vs prédictions officielles NOAA, p95 d'écart documenté (pas de seuil bloquant en v0.1 ; < 2 cm = cible v0.2).
- **Lot 2a — Data pack NOAA** (`amar-data`) : 5–10 stations NOAA **de type harmonique uniquement** (les stations subordonnées, prédites par offsets PM/BM, n'ont ni constantes ni oracle « any interval » — exclues), choisies pour couvrir semi-diurne, diurne, mixte, grand et faible marnage. JSON versionné par station avec **manifeste complet** (source, licence, date d'extraction, checksum, script d'ingestion, datum, nb constituants, version moteur compatible) — le manifeste par pack est la **vérité canonique** ; `DATA_LICENSES.md` ne fait qu'agréger. Format binaire compact = plus tard ; v0.1 = JSON + SHA-256, rien de plus. Le script d'ingestion est **rejoué en CI mensuelle** (non bloquant, diff contre les packs figés). DoD : round-trip + checksums vérifiés en CI.
- **Lot 2b — Pack Brest calibré** (aligné M2a/M2b du découpage MVP — dépend du moteur, Lot 1) : outil `amar-calibrate` (**Rust**, version brutale et bornée : constituants fixés d'avance, moindres carrés linéaires sin/cos via une lib d'algèbre linéaire mature, jamais de solveur maison, jamais un clone UTide — sur ≥ 1 an d'observations REFMAR ; contrôle des trous ; **réserve dès la calibration une fenêtre d'observations hors calibration** pour le futur `benchmark_brest_v1` ; zéro hydrographique documenté depuis les RAM publiques du Shom → le datum Brest est *actionnable*, pas seulement déclaré) → pack `amar-data-brest-experimental` (Licence Ouverte 2.0 vérifiée, attribution Shom/REFMAR ; flags `experimental`, `not_official`, `not_shom` ; champs `calibration_period`/`validation_period` ; disclaimer « constantes dérivées REFMAR, non équivalentes aux constantes SHOM »). Le calibrateur est un **compilateur de données** : versions épinglées par `Cargo.lock`, checksum des observations d'entrée, exécution idempotente, artefact JSON commité ; contre-vérification ponctuelle vs UTide à la main, hors chaîne de build. Son durcissement en CLI public reste au Lot 8. **Dogfooding honnête** : dès M1 le curl Brest existe (refus 422 expliqué) ; la hauteur Brest arrive avec ce lot, en M2 (semaines 4–9) — la décision 100 % Rust rend impossible un pack Brest « au premier mois », on l'assume plutôt que de le promettre. DoD : pack Brest reproductible, datum ZH documenté, fenêtre de validation réservée.
- **Lot 3 — API minimale + artefact utilisable** (`amar-server`) : `POST /tide` (un instant, une hauteur), `GET /health`, `GET /coverage` (liste stations + rayons + datums). Résolveur conservateur : station à ≤ `max_distance_km` (défaut 10–20 km) sinon `422 no_supported_source` ; exclusions manuelles par station. Réponse = le JSON de la vision (datum, source, confidence heuristique, warnings). La distribution est une feature : CLI dans le même binaire (`amar tide --lat 48.38 --lon -4.49 --at 2026-08-15T09:00:00Z`), binaire téléchargeable en release, README à 3 curls qui racontent un usage réel, dont Brest même si la réponse est un refus expliqué. DoD : snapshots API, les 3 cas (nominal, hors rayon 422, entrée invalide 400), et la boucle installer, demander, comprendre pourquoi oui ou non, exécutable par un tiers en une commande.

**= v0.1.** Tourne offline, refuse proprement, chaque chiffre est traçable.

### v0.2+ — extension mesurée

- **Lot 4 — Validation & confiance calibrée** : deux régimes de validation, car il n'existe **pas d'oracle officiel français** (vérifié, `DATA.md`) :
  - *Stations NOAA* : comparaison vs prédictions officielles (même datum MLLW) → « erreur moteur », cible < 2 cm.
  - *Brest* : comparaison vs **observations** REFMAR. Vocabulaire imposé : « **résidu = niveau d'eau observé − marée astronomique prédite (météo incluse)** » — jamais « erreur marée ». Pour comparer les releases sans mentir, ce lot **consomme** le `benchmark_brest_v1` figé **au Lot 2b** (mêmes timestamps, même masque de lacunes, même datum, mêmes observations, checksums publiés) — on ne compare jamais des périodes mouvantes, et le benchmark appartient à celui qui calibre, pas à celui qui valide.

  **Matrice de validation publiée à chaque release** (par station : RMS + biais + MAE + p95 + max, séparés par saison et par état « calme météo » si possible ; période ; oracle *ou* benchmark ; datum), avec **deux baselines systématiques** : la release précédente et une prédiction naïve (constantes anciennes / modèle simple). `sigma_cm` recalé sur ces résidus (estimation empirique produit, assumée). Un détecteur d'extrema **interne** est autorisé pour les tests — pas exposé dans l'API avant le Lot 5.
- **Lot 5 — PM/BM, séries et fenêtres** : extraction des extrema (`next_high`/`next_low`), `duration_h`/`step_min`, et fenêtres de marée (`hauteur > seuil` entre deux dates). Après le cœur, jamais avant, mais en tête de v0.3 : le pré-mortem est net, un `/tide` instantané est un moteur, pas une raison de revenir. L'usage réel, c'est « est-ce que je peux sortir demain matin ? ».
- **Lot 6 — Couche FR/annuaire** : coefficient de marée SHOM (présentation, pas primitive), packs `amar-data-fr` **si et seulement si** la licence des constantes est éclaircie ; sinon calibration locale (Lot 8) sur observations REFMAR.
- **Lot 7 — Polygones de validité** : remplacer les rayons par des zones (presqu'îles, écluses, estuaires), avertissements spécifiques port/lagune/estran.
- **Lot 8 — CLI calibration public** : durcir l'outil interne `amar-calibrate` (né en M2a, déjà en Rust) en commande publique `amar calibrate obs.csv`, avec garde-fous assumés : critère de Rayleigh, 1 mois de données → peu de constituants + incertitude forte affichée ; 1 an → jeu complet. Étiqueté expérimental.

### Projets frères (hors roadmap produit)

- **amar-live** : observé temps réel + surcote (IOC/REFMAR/NOAA). Service connecté, autre métier, autre SLO.
- **Data pack grille** (EOT20) : couverture large avec avertissements côtiers massifs. Produit data séparé.
- **Hydrodynamique régionale** (TELEMAC/SCHISM + Litto3D) : note R&D, pas de jalon.

## 7. Jalons — la table de correspondance unique

Une seule vérité pour trois vocabulaires : MVP (exécution, voir `MVP.md`), lots (contenu), versions (releases). `MVP.md` référence cette table, il ne la duplique pas.

| MVP | Lots | Version | Preuve | Semaines (effectif) |
|---|---|---|---|---|
| M0 « moteur prouvé » | 0, 1, 2a (3 stations) | — (tag `m0`) | p95 vs oracle NOAA affiché en une commande | 0–2 |
| M1 « curl de la vision » | 2a complet, 3 | **v0.1** | Boucle installer→demander→comprendre par un tiers ; curl Brest → 422 expliqué | 2–4 |
| M2 « Brest expérimental » (M2a/M2b) | 2b (qui **possède** `benchmark_brest_v1`) | **v0.1.x** | Hauteur / ZH Brest, σ élargi, benchmark figé | 4–9 |
| — Point d'arrêt global | — | — | Les 3 preuves ensemble, sinon gel propre | 8–10 |
| — | 4 complet | **v0.2** | Matrice de validation publique, moteur < 2 cm vs NOAA | 10+ |
| M3 « raison de revenir » | 5 | **v0.3** | Fenêtres de marée, next_high/next_low (après v0.2 — ordre validé au tour 2) | 10+ |
| — | 6, 7, 8 | **v0.4** | Couche FR/annuaire, polygones, CLI calibration public | ultérieur |
| — | durcissement | **v1.0** | Data packs signés stables, format gelé | ultérieur |

## 8. Risques

| Risque | Parade |
|---|---|
| ~~Constantes SHOM non ouvertes~~ **Confirmé** (DATA.md) | Acté : la France passe par la calibration REFMAR, le plan n'en dépend plus |
| Qualité des observations REFMAR (trous, sauts de datum capteur, séries multiples) | Contrôle qualité dans le pipeline de calibration ; durée ≥ 1 an ; σ élargi et flag `experimental` tant que le résidu n'est pas caractérisé |
| Datums incohérents entre sources | Datum obligatoire dans schéma + réponse ; comparaisons uniquement à datum identique |
| Confiance trop optimiste (faux cm) | Matrice publique + refus conservateur + warnings ; grade recalé sur résidus mesurés |
| Dérive des constantes / du niveau moyen | `data_version` datée dans chaque réponse ; packs ré-émis, jamais « pour toujours » |
| Scope creep (le tueur n°1 identifié) | Toute feature doit répondre : « aide-t-elle à sortir une prédiction stationnaire traçable et validée ? » Sinon → backlog |

## 9. Critères d'arrêt (pré-mortem)

Un projet perso sans deadline meurt en silence ; on fixe les tripwires à l'avance :

- **8–10 semaines de travail réel** après le premier commit, il doit exister ensemble : un binaire lançable en une commande, une prédiction NOAA hors ligne reproductible avec provenance/datum, et un cas Brest (hauteur documentée ou refus clair et utile). Sinon : gel propre.
- **Signal de mort principal** : le projet sait expliquer pourquoi il ne faut pas lui faire confiance, mais ne donne toujours aucune marée utile à son auteur.
- **Réalité d'usage** : trois mois après v0.2, si l'auteur ne l'utilise pas lui-même chaque semaine, on gèle.
- **Gel propre ≠ abandon** : publier `CONVENTIONS.md`, les scripts d'ingestion et les leçons datums/licences, puis convertir en bibliothèque expérimentale ou note technique. Pas de cimetière de crates.

## 10. Backlog exploratoire (non planifié)

iCal PM/BM et grandes marées ; WASM offline ; embarqué ESP32/RPi ; serveur MCP ; GNSS-IR comme pseudo-marégraphes ; courants ; annuaire PDF ; conversion de datum « actionnable » (répondre en « hauteur au-dessus du zéro des cartes » locale). *(Fenêtres de marée et CLI : promus dans les lots.)*

## 11. Journal des revues

- **2026-07-06** — v1 rédigée (Claude). Relecture ChatGPT Pro étendu via dunst, tour 1 (réflexion 8m37s) : verdict « viable en mode stations calibrées, hors ligne, honnête sur sa zone de validité ; irréaliste s'il promet GPS partout + précision durable + grille mondiale + score fiable dans la même trajectoire courte ». 16 corrections intégrées → v2. Priorité retenue : *supprimer tout ce qui n'aide pas à sortir une prédiction stationnaire traçable et validée.*
- **2026-07-06** — Tour 2 (réflexion 3m49s) : v2 validée. Dernier défaut bloquant levé par l'ajout de `CONVENTIONS.md` (Lot 0, correction 17) ; ordre v0.2→v0.3 confirmé (18) ; garde-fous Rust injectés au Lot 1 (19). Plan gelé en v2.1.
- **2026-07-06** — Double pré-mortem (Claude 8 causes + ChatGPT Pro étendu 8 causes, réflexion 5m34s — détail dans `PREMORTEM.md`). Cause de mort n° 1 convergente : *pas de Brest en v0.1 = pas de dogfooding = mort émotionnelle du projet*. 7 modifications appliquées : station Brest expérimentale au Lot 2, CLI + release + README au Lot 3, timebox conventions + règle des 14 jours + station témoin + tests +5/+10 ans aux Lots 0/1, ingestion CI mensuelle au Lot 2, fenêtres de marée promues au Lot 5, section « Critères d'arrêt » ajoutée. Plan gelé en v2.2.
- **2026-07-06** — Revue interne v2.3 + **audit des données vérifié en ligne** (`DATA.md`) : API NOAA testées (37 constituants, oracle prédictions), Licence Ouverte 2.0 REFMAR confirmée, EOT20 CC-BY confirmé, non-distribution des constantes SHOM confirmée (rayées du plan — la France passe définitivement par la calibration REFMAR). Corrections : deux régimes de validation au Lot 4 (oracle NOAA vs observations Brest), production du pack Brest précisée (à l'époque : pipeline Python/UTide), CLI ajouté à l'architecture, risque qualité REFMAR ajouté.
- **2026-07-06** — v2.4, dernière itération de planification. Tour A ChatGPT sur revue + données (14 points : stations harmoniques vs subordonnées, SA/SSA d'EOT20 météo-teintés, type de produit REFMAR, benchmark figé, deux baselines, vocabulaire « résidu », périodes de calibration/validation, disclaimer non-SHOM). `MVP.md` créé (M0/M1/M2a-M2b/M3, kit d'arrêt par milestone) et validé au tour B (11 points). **Décision 100 % Rust** : `amar-calibrate` en Rust, UTide en oracle manuel hors chaîne. Audit hanoi (historique en annexe, Lot 2 scindé 2a/2b, table de jalons unique, fenêtre benchmark réservée dès M2a) et audit tangle (85/100, boucle 2b⇄4 levée, crate contrat `amar-pack`). Passe plume sur les cinq documents. **Pré-mortem final croisé : GO-avec-conditions** (commit M0 exécutable ensuite ; M0 sans Brest/serveur/calibration ; `amar-calibrate` brutal et borné avec porte de sortie publiée ; jugement sur artefact lançable par un tiers). Plan gelé.

## 12. Annexe — détail des corrections par revue

### Tour 1 (v1 → v2)

| # | Correction | Origine |
|---|---|---|
| 1 | Slogan « valable pour toujours » supprimé : les constantes, datums, jauges et niveaux moyens évoluent. On promet un **jeu de constantes versionné + datum explicite** | Verdict |
| 2 | v0.1 réduite à l'os : un point GPS proche d'une station, une date UTC, une hauteur, une source, un datum, une confiance basique. PM/BM, séries, coefficient, pression, OpenAPI, page démo, Docker = **sortis de v0.1** | §1 |
| 3 | Moteur : NOAA utilise ~37 constituants ; 30 jours d'observation ne les séparent pas, il faut ~1 an. L'objectif < 2 cm devient un objectif **v0.2**, pas un bloqueur v0.1 | §1 |
| 4 | Coefficient SHOM = convention de présentation française → couche « FR/annuaire » séparée, jamais dans le cœur | §1 |
| 5 | Baromètre inverse = correction de **niveau d'eau**, pas de la marée astronomique → sorti de `/tide`, réservé à un futur module water-level | §1 |
| 6 | Grille mondiale = **second produit** (data pack optionnel), pas une extension naturelle. « Tout point océanique répond » = promesse supprimée | §1 |
| 7 | Licences durcies : FES2022 non redistribuable tel quel ; constantes SHOM pas garanties ouvertes ; IOC/GESLA = validation interne uniquement. NOAA = seul socle sûr | §2 |
| 8 | Datums verticaux = piège produit n° 1 : `height_m` n'a aucun sens sans référence. Datum obligatoire dans chaque réponse et chaque fichier | §2 |
| 9 | `sigma_cm` unique mélangeait 4 erreurs. Assumé comme **estimation empirique produit**, avec `warnings` explicites | §2 |
| 10 | Validation en deux régimes : moteur vs **prédictions officielles** ; usage vs observations = « erreur niveau d'eau total » | §2 |
| 11 | Pas de « nearest wins » : rayon `max_distance_km`, exclusions manuelles, refus explicite au-delà | §2, §5 |
| 12 | Données = **packs signés séparés du binaire** : résout taille, licence, mise à jour, reproductibilité | §5 |
| 13 | Manifeste de données obligatoire par station. Sans ça, « souverain » n'est pas auditable | §5 |
| 14 | Endpoint `/coverage` (+ `/explain`) : où le service accepte/refuse et pourquoi | §5 |
| 15 | **Matrice de validation publique** à chaque release — plus crédible qu'un grade A–D opaque | §5 |
| 16 | Observé/surcote → projet frère ; calibration DIY → CLI expérimental ; hydrodynamique → note R&D hors roadmap | §1, §3 |

### Tour 2 (validation v2 → v2.1)

| # | Correction | Détail |
|---|---|---|
| 17 | **`CONVENTIONS.md` avant toute ligne de code** | Époque, signe de phase, degrés/radians, échelle de temps, datum, hauteur positive, ordre des corrections nodales. Sans cette spec, les golden tests peuvent « passer » par compensation accidentelle |
| 18 | Ordre confirmé : **v0.2 validation avant v0.3 PM/BM** | Les extrema sont des dérivés de h(t) ; détecteur interne toléré en v0.2, non exposé |
| 19 | Garde-fous Rust dès le Lot 1 | Types forts, pas de `HashMap<String, f64>` dans la boucle, cœur strictement déterministe |

