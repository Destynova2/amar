# amar — Audit des données

**Date** : 2026-07-06. Vérifications faites en ligne (API réelles interrogées, licences vérifiées). Ce document répond à trois questions : qu'a-t-on déjà, que doit-on récupérer, qu'est-ce qu'on n'aura jamais — et ce que ça impose aux MVP.

---

## A. Ce qu'on A (vérifié, prêt à l'emploi)

| Donnée | Vérification | Contenu | Licence |
|---|---|---|---|
| **Constantes harmoniques NOAA** | ✅ API interrogée le 2026-07-06 : `api.tidesandcurrents.noaa.gov/mdapi/prod/webapi/stations/9414290/harcon.json?units=metric` → **37 constituants** (nom, amplitude en mètres, phase GMT + locale, vitesse °/h). ⚠️ **Stations harmoniques uniquement** : NOAA a aussi des stations *subordonnées* (prédictions limitées aux PM/BM par offsets) qui n'ont ni constantes ni oracle « any interval » — à exclure du pack | Ports US instrumentés (type harmonique) | Domaine public (œuvre gouvernementale US) |
| **Prédictions officielles NOAA (l'oracle)** | ✅ API interrogée : `datagetter?product=predictions&datum=MLLW&units=metric&time_zone=gmt` → série `{t, v}` horaire ou 6 min | Prédictions officielles par station, datum au choix (MLLW…) | Domaine public |
| **Datums NOAA par station** | API mdapi `datums.json` (même famille que harcon, non re-testée) | MLLW/MSL/NAVD88… offsets par station | Domaine public |
| **EOT20 (grille mondiale)** | ✅ SEANOE doi:10.17882/79489 — **CC-BY 4.0**, usage commercial autorisé avec attribution, citation Hart-Davis et al. 2021. ⚠️ SA/SSA y **embarquent du signal météo** (modèle empirique altimétrique) : pas « astronomique pur » — à documenter si on les utilise | 17 constituants, grille globale 0,125°, marée océanique + charge | CC-BY 4.0 |
| **Arguments astronomiques** | Pas une donnée : calculés (Schureman/Doodson, littérature publique) | Fréquences, corrections nodales f/u | — |
| **GEBCO (bathymétrie)** | Notoire, non re-vérifiée | Grille globale ~450 m | Libre |

## B. Ce qu'on DOIT récupérer (accessible, travail à faire)

| Donnée | Chemin | Effort | Rôle |
|---|---|---|---|
| **Observations Brest (REFMAR)** | ✅ Licence vérifiée : **Licence Ouverte 2.0 Etalab (avril 2017)**, accès gratuit sur data.shom.fr, API SOS JSON/XML/TXT ; téléchargement contre nom + email (INSPIRE). ⚠️ Le **type de produit** compte : les séries « validées horaires » sont filtrées, et des lacunes > 1,5 h cassent un calcul horaire naïf — choisir et documenter le produit exact | Télécharger ≥ 1 an de hauteurs ; contrôler trous/qualité ; consigner `calibration_period` et `validation_period` | **Calibrer nos propres constantes Brest** (outil Rust `amar-calibrate`) → pack `amar-data-brest-experimental`. Redistribution des dérivés permise (LO 2.0, attribution Shom/REFMAR). **Disclaimer obligatoire : ce ne sont pas les constantes SHOM et elles ne doivent jamais être présentées comme équivalentes** |
| Observations autres ports FR (REFMAR) | Même licence, mêmes API | Idem, port par port | Étendre le mode France expérimental après Brest |
| Zéro hydrographique de Brest | Publié dans les « Références Altimétriques Maritimes » (RAM) du Shom, PDF public | Lecture + report manuel dans le manifeste | Rendre le datum Brest actionnable (hauteur / ZH) |
| Sélection des 5–10 stations NOAA du pack v0 | API NOAA | Choisir pour couvrir semi-diurne/diurne/mixte/grand/petit marnage | Pack v0.1 |
| UHSLC / GESLA (séries longues) | Téléchargement libre (recherche) | Batch | Validation croisée uniquement, pas d'embarquement |

## C. Ce qu'on N'AURA PAS (vérifié — on planifie sans)

| Donnée | Preuve | Conséquence |
|---|---|---|
| **Constantes harmoniques SHOM** | ✅ Vérifié : le SHOM ne les distribue pas (position adossée à une décision OHI « sécurité ») ; demande individuelle possible mais **usage privé, redistribution interdite** — c'est la raison pour laquelle XTide ne couvre pas la France | Pas de pack `amar-data-fr` officiel, jamais. La France passe **exclusivement** par la calibration sur observations REFMAR |
| **Prédictions officielles SHOM** (annuaires, SPM) | Produits sous licence numérotée, payants | Pas d'oracle officiel français. La validation Brest se fait contre les **observations** REFMAR (en le nommant « erreur niveau d'eau total ») ou contre des valeurs d'annuaire consultées manuellement (spot-check, non redistribué) |
| **FES2022** | Licence AVISO : dérivés selon produits, pas de redistribution du produit original, accordée 5 ans | Écarté. EOT20 le remplace pour toute ambition grille |
| **IOC SLSMF en usage commercial** | Interdit sans accord des producteurs ; pas de contrôle qualité | Validation interne uniquement |
| **GESLA en redistribution** | Licences mixtes dont non-commerciales | Validation interne uniquement |

## D. Conséquences sur les MVP

1. **Le MVP cœur est 100 % NOAA** : constantes et oracle du même producteur, domaine public, même datum (MLLW). La boucle moteur-validation est juridiquement et techniquement fermée, aucun blocage de données.
2. **Le MVP Brest est une calibration, pas une importation** : observations REFMAR (LO 2.0), puis `amar-calibrate` (Rust, moindres carrés ; UTide en contre-vérification manuelle uniquement), et on obtient des constantes à nous, redistribuables avec attribution. Il n'existe aucun autre chemin légal. Risques propres : trous de données, datum capteur, pas d'oracle officiel. La validation se fait donc en résidus vs observations (météo incluse), σ élargi, flags `experimental`/`not_official` obligatoires.
3. **La grille mondiale a un GO licence** (EOT20 CC-BY) mais reste un second produit : 17 constituants seulement, précision côtière moyenne — rien n'oblige à la mettre dans un MVP.
4. **L'oracle français n'existe pas** : l'objectif « < 2 cm vs officiel » n'a de sens qu'aux US. À Brest, la métrique honnête est « RMS résiduel vs observations sur N mois ». C'est une autre définition du succès, à écrire telle quelle dans le MVP Brest.
5. La sélection des stations NOAA et le téléchargement REFMAR sont les deux seules tâches de données du chemin critique. Tout le reste est du calcul.

## Sources vérifiées

- [NOAA harcon API (station 9414290)](https://api.tidesandcurrents.noaa.gov/mdapi/prod/webapi/stations/9414290/harcon.json?units=metric) — 37 constituants, mètres, phases GMT/locale
- [NOAA predictions datagetter](https://api.tidesandcurrents.noaa.gov/api/prod/datagetter) — oracle par station/datum
- [REFMAR : diffusion sur data.shom.fr](https://refmar.shom.fr/actualites/diffusion-mesures-maregraphiques-sur-data.shom) et [téléchargement](https://refmar.shom.fr/donnees-refmar-sur-data.shom.fr/telechargement-des-donnees) — Licence Ouverte 2.0 Etalab
- [EOT20 sur SEANOE](https://www.seanoe.org/data/00683/79489/) — CC-BY 4.0, doi:10.17882/79489
- [maree.info CGU](https://maree.info/cgu) et [refmar.shom.fr/prediction-de-la-maree](https://refmar.shom.fr/prediction-de-la-maree) — statut propriétaire des constantes et prédictions SHOM
