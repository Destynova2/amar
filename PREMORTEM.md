# amar — Pré-mortem

**Exercice** : nous sommes le 6 juillet 2027. Le repo `amar` est mort — dernier commit il y a neuf mois, aucune release utilisée, le dossier prend la poussière à côté des autres projets A00. On remonte le fil : qu'est-ce qui l'a tué ?

Deux autopsies indépendantes : celle de Claude (ci-dessous), celle de ChatGPT Pro étendu (section 2), puis la synthèse fusionnée (section 3) — ce qui entre réellement dans le plan.

**Contexte assumé** : développeur solo, projet perso sans deadline, basé à Brest, plan v2.1 gelé.

---

## 1. Autopsie Claude

Classement par probabilité × impact, décroissant.

### C1 — Mort d'inutilité personnelle : la v0.1 est US-only, le dev est à Brest ⚠️ le plus probable
La v0.1 ne sert que des stations NOAA. Or l'utilisateur n° 1 (le seul garanti) vit à Brest et voulait la marée chez lui. Le curl marche sur San Francisco, personne ne le tape deux fois. Le pack France est en v0.4, derrière un point d'interrogation juridique sur les constantes SHOM. Le projet meurt entre v0.1 et v0.4 : techniquement vivant, émotionnellement mort.
- **Signaux avant-coureurs** : après la v0.1, le dev ne lance plus jamais le binaire pour lui-même ; la question licence SHOM reste « à vérifier » pendant des mois.
- **Parade** : brancher la boucle de dogfooding dès v0.1 — soit vérifier la licence SHOM **avant** le Lot 0 (un mail au SHOM coûte zéro code), soit prévoir une station Brest calibrée sur observations REFMAR (Licence Ouverte, pas besoin des constantes SHOM) comme **station n° 11 du pack v0.1**, même avec une incertitude élevée affichée.

### C2 — CONVENTIONS.md tourne à la thèse : paralysie avant la première ligne
La correction n° 17 (geler les conventions avant de coder) était juste, mais elle devient un piège pour un perfectionniste solo : trois semaines dans Schureman, l'IHO et les échelles de temps TT/UT1, zéro ligne de Rust, l'élan retombe.
- **Signaux** : plus de dix jours sur CONVENTIONS.md ; des sections « à trancher plus tard » qui s'accumulent ; relecture de papiers au lieu d'écrire des tests.
- **Parade** : timeboxer CONVENTIONS.md à 2–3 jours, avec une règle : chaque convention se décide en copiant NOAA (l'oracle) — pas en arbitrant la littérature. Le doc fige *ce que fait NOAA*, pas *ce qui est théoriquement le mieux*.

### C3 — Scope creep malgré la garde : le moteur est trop amusant
Le tueur identifié par la revue, version solo dev : PM/BM « c'est trivial », le coefficient « deux lignes », une petite courbe SVG « pour voir »… La v0.1 gonfle six mois et ne sort jamais. Le pattern existe déjà : le prototype claude.ai avait PM/BM, coefficient, pression et page démo *avant le premier commit du vrai repo*.
- **Signaux** : une PR qui touche `amar-core` ET `amar-server` ; un endpoint non listé dans le plan ; « tant que j'y suis » dans un message de commit.
- **Parade** : la garde du plan devient mécanique — la v0.1 est définie par une liste **fermée** de routes et de champs JSON ; tout ajout = issue étiquetée `post-v0.1`, jamais de code.

### C4 — Le bug de convention compensé : la validation ment
Signe de phase inversé compensé par un argument astronomique décalé : les golden tests NOAA passent (p95 ~2 cm), tout va bien. Jusqu'au pack n° 2, qui arrive avec une autre convention de source, ou aux dates 2030+ où les corrections nodales divergent. Découvert huit mois plus tard : toute la matrice de validation est à refaire, confiance détruite, motivation aussi.
- **Signaux** : p95 excellent sur les stations du pack mais dégradé sur une station *hors* pack ; erreurs qui croissent avec l'horizon temporel (2027 < 2030 < 2035).
- **Parade** : dans les golden tests dès le Lot 1 — au moins une station **tenue à l'écart** de tout réglage, et des points de test à +5 et +10 ans pour exposer la dérive nodale.

### C5 — L'oracle NOAA n'est pas reproductible à < 2 cm : découragement en v0.2
Les prédictions officielles NOAA peuvent inclure des traitements non documentés dans l'API des constantes. Le moteur plafonne à 4–6 cm de p95, l'objectif < 2 cm (v0.2) semble inatteignable, le projet s'enlise dans le debugging d'un écart qui ne vient pas d'un bug.
- **Signaux** : semaines passées à chasser un écart constant ou saisonnier ; l'écart est le même sur toutes les stations (signature d'une différence de méthode, pas d'un bug).
- **Parade** : déclarer dès maintenant que la cible v0.2 est *p95 documenté et stable*, la barre < 2 cm étant **révisable sur diagnostic** — un écart expliqué (méthode) vaut mieux qu'un écart chassé (bug fantôme).

### C6 — La sur-ingénierie des data packs : l'usine à gaz pour un utilisateur
Signature des packs, format binaire versionné, tuilage, quantification int16… pour dix stations JSON qui tiennent dans 50 Ko. Le générique dévore le concret ; trois mois d'infra pour zéro valeur visible.
- **Signaux** : plus de code dans `amar-data` que dans `amar-core` ; discussions de format sans consommateur ; « ça servira pour la grille ».
- **Parade** : v0.1 = JSON + checksum SHA-256, point. La signature et le binaire compact n'existent que le jour où un pack dépasse 10 Mo ou un tiers télécharge un pack.

### C7 — L'accident du « not_for_navigation » : peur juridique paralysante
Quelqu'un s'échoue ou se fait piéger par la marée en s'appuyant sur amar ; ou plus probablement : le dev *imagine* ce scénario, ajoute des disclaimers partout, perd l'envie de publier.
- **Signaux** : hésitation à mettre le repo en public ; multiplication des warnings au-delà des trois définis.
- **Parade** : régler la question une fois : licence avec clause de non-responsabilité standard (Apache-2.0 la contient), warnings définis au Lot 3, et on n'y revient plus.

### C8 — La panne d'ingestion silencieuse : NOAA change son API
Le script d'ingestion casse (format, rate limit, URL), personne ne s'en aperçoit car les packs sont figés ; le jour d'une mise à jour, plus rien ne marche et la remise en route coûte un week-end de rétro-ingénierie.
- **Signaux** : le script n'a pas tourné depuis six mois ; pas de test d'ingestion en CI.
- **Parade** : l'ingestion est un script **rejouable en CI** (mensuel, non bloquant) qui diffe sa sortie contre le pack figé — la dérive de l'API devient une notification, pas une découverte.

### Tripwires — critères d'arrêt honnêtes (Claude)
- **Gel assumé** : si la v0.1 n'est pas sortie 8 semaines après le premier commit du Lot 0 → on gèle, on écrit un post-mortem court, on garde le plan pour plus tard. Pas de zombie.
- **Pivot données** : si ni la licence SHOM ni la calibration REFMAR ne permettent une station Brest en 6 mois → le projet pivote (outil de calibration pur, ou US-only assumé) ou s'arrête — mais on décide.
- **Réalité d'usage** : si trois mois après v0.2 le dev lui-même ne l'utilise pas chaque semaine → geler. Un outil de marée que son propre auteur breton n'utilise pas est réfuté par l'évidence.

---

## 2. Autopsie ChatGPT Pro étendu

*(réflexion 5m34s — même conversation que la revue du plan. Cadrage d'entrée : « si amar meurt malgré un plan sain, ce sera par données, validation, conventions, licences et énergie solo — pas par Axum. »)*

### G1 — Le projet n'a jamais été utile à Brest
v0.1 sur 5–10 stations NOAA : propre, légal, validable, mais émotionnellement loin du développeur. Amar est mort parce que son auteur ne l'a jamais utilisé pour *sa* rade, *ses* horaires, *son* zéro hydrographique.
- **Signaux tôt** : tous les exemples sont San Francisco/Seattle ; aucun `curl` Brest même faux ou expérimental ; pas de cas d'usage local (pêche à pied, port, kayak, cale, seuil de hauteur).

### G2 — Le cœur harmonique a mangé le projet
`CONVENTIONS.md` était nécessaire mais est devenu un tunnel : phase GMT vs locale, signes, nodal f/u, époque, datums, compatibilité NOAA, 37 constituants. Beaucoup de notes, peu de prédictions exposées.
- **Signaux tôt** : plus de 2 semaines sans endpoint fonctionnel ; plusieurs réécritures des conventions ; écarts NOAA inexpliqués mais aucune version « known imperfect » taguée.

### G3 — La validation a tué la livraison
La matrice publique v0.2 est saine, mais elle a transformé amar en banc métrologique avant d'être un produit : vouloir expliquer chaque cm au lieu d'expédier une prédiction honnête avec avertissements.
- **Signaux tôt** : le script de validation devient plus gros que le moteur ; les seuils changent souvent ; aucune release tant que le p95 n'est pas « beau ».

### G4 — La France est restée bloquée par les données
NOAA permet de construire, pas de conquérir l'usage brestois. Constantes SHOM : incertain. REFMAR : observations ouvertes mais calibration longue (Rayleigh, datums, qualité, trous, zéro hydrographique). La v0.4 France est devenue le vrai projet caché.
- **Signaux tôt** : beaucoup de temps à lire les licences SHOM/REFMAR ; aucun pack `amar-data-fr-experimental` ; pas de décision claire « France officielle impossible pour l'instant ».

### G5 — La v0.1 prouvait la technique, pas la vision utilisateur
`/tide` instantané est le bon noyau, mais peu de gens veulent seulement « hauteur à telle date » : ils veulent PM/BM, fenêtre où hauteur > X, coefficient, calendrier — « est-ce que je peux sortir demain matin ? ». Amar a sorti un moteur, pas une raison de revenir.
- **Signaux tôt** : les retours demandent tous `next_high`/`next_low`, seuils ou iCal ; personne ne réutilise `/tide` seul ; les exemples du README ne racontent aucun usage concret.

### G6 — Le « souverain » a dérivé vers l'over-engineering
Data packs signés, binaire nu, manifestes, checksums, crates séparées, `/coverage`, validation par release : tout se justifie individuellement, mais c'est lourd pour un solo dev sans deadline. Le socle a pris la place de la marée.
- **Signaux tôt** : beaucoup de structure Cargo/CI/manifestes ; peu de fixtures ; aucun tag téléchargeable ; « il faut juste nettoyer le format data pack » revient plusieurs fois.

### G7 — Les datums ont cassé la confiance
Compris trop tard : `height_m` sans conversion locale exploitable frustre les usages réels. MLLW NOAA, zéro hydrographique, LAT, NM, station datum : techniquement explicites mais **pas actionnables** pour un utilisateur français.
- **Signaux tôt** : réponse correcte mais incompréhensible ; README plein d'avertissements datum ; impossibilité de répondre simplement « hauteur au-dessus du zéro des cartes à Brest ».

### G8 — Aucune boucle de distribution
Projet perso sans deadline : sans utilisateur témoin, amar a cessé de recevoir de l'énergie. Pas de release, pas de démo, pas de CLI vraiment utile, pas de paquet installable.
- **Signaux tôt** : personne sauf l'auteur n'a lancé le binaire ; pas de « golden path » en une commande ; aucun journal de validation publié ; le dernier commit est une refactorisation interne.

### Les 3 parades ChatGPT
1. **« Mode Brest expérimental » dès v0.1** : pack séparé `amar-data-brest-experimental` construit sur observations REFMAR si juridiquement redistribuable, sinon fixture locale non redistribuée. Marqué brutalement `experimental`, `not_official`, `datum_uncertain`. Objectif : dogfooder amar à Brest dès le premier mois.
2. **Un artefact utilisable avant la matrice parfaite** : v0.1 = binaire téléchargeable + README avec 3 curls + commande locale `amar tide --lat 48.38 --lon -4.49 --at 2026-08-15T09:00:00Z`. Même si Brest répond `422`, c'est une décision produit visible : la boucle « installer → demander → comprendre pourquoi oui/non » doit exister.
3. **Limiter explicitement le tunnel scientifique** : règle de calendrier — si `CONVENTIONS.md` + moteur ne donnent pas une prédiction NOAA comparable en **14 jours de travail effectif**, geler les corrections avancées et livrer un moteur marqué `method: harmonic_basic_no_nodal`. *« Le danger n'est pas d'avoir une v0.1 imparfaite ; c'est de ne jamais avoir de v0.1. »*

### Critère d'arrêt ChatGPT
Tuer ou geler si, au bout de **8 à 10 semaines de travail réel**, il n'existe pas ces trois choses **ensemble** :
1. un binaire lançable en une commande ;
2. une prédiction NOAA hors ligne reproductible avec provenance/datum ;
3. un cas Brest, même expérimental, qui répond soit une hauteur documentée, soit un refus clair et utile.

**Signal de mort principal** : *le projet sait expliquer pourquoi il ne faut pas lui faire confiance, mais ne donne toujours aucune marée utile à son auteur.* Dans ce cas, geler proprement : publier `CONVENTIONS.md`, les scripts d'ingestion, les leçons datums/licences, et transformer amar en bibliothèque expérimentale ou note technique — pas en cimetière de crates.

---

## 3. Synthèse fusionnée → ce qui entre dans le plan

### Convergences (les deux autopsies, indépendamment)

| Cause de mort | Claude | ChatGPT | Verdict |
|---|---|---|---|
| Pas de Brest = pas de dogfooding = mort émotionnelle | C1 | G1 + G4 | **Cause n° 1 des deux côtés.** Non négociable : un cas Brest dès v0.1 |
| CONVENTIONS.md devient un tunnel | C2 | G2 | Timebox + échappatoire dégradée obligatoires |
| La validation/le perfectionnisme bloque la release | C5 | G3 | Le p95 se documente, il ne bloque pas |
| Over-engineering du socle (packs signés, formats) | C6 | G6 | v0.1 = JSON + SHA-256, rien de plus |

### Apports propres à chaque autopsie

- **ChatGPT seul** : G5 (un `/tide` instantané n'est pas une raison de revenir — prévoir tôt la première feature d'usage : `next_high`/`next_low` ou fenêtres) ; G7 (un datum *explicite* n'est pas un datum *actionnable* — il faut pouvoir répondre « au-dessus du zéro des cartes ») ; G8 (la distribution est une feature : release, une commande, golden path).
- **Claude seul** : C4 (bug de convention compensé — station témoin + tests à +5/+10 ans) ; C7 (peur juridique — la trancher une fois via la licence) ; C8 (ingestion NOAA qui casse en silence — rejouable en CI mensuelle).

### Modifications appliquées au plan (v2.1 → v2.2)

1. **Lot 2** : ajout du pack `amar-data-brest-experimental` (11ᵉ source, flags `experimental`/`not_official`/`datum_uncertain`) — dogfooding Brest dès le premier mois. Décision explicite à consigner : « France officielle impossible pour l'instant » est un état valide, l'absence de décision non.
2. **Lot 3** : l'artefact v0.1 inclut le CLI `amar tide --lat --lon --at` (même binaire que le serveur), un binaire téléchargeable en release et un README à 3 curls. La boucle « installer → demander → comprendre » fait partie de la DoD.
3. **Lot 0/1 — anti-tunnel** : CONVENTIONS.md timeboxé (2–3 jours, on copie les conventions NOAA) ; règle des 14 jours de travail effectif → sinon release d'un moteur dégradé marqué `harmonic_basic_no_nodal`, écarts tagués « known imperfect ».
4. **Lot 1 — anti-bug-compensé** : une station golden tenue hors réglage + points de test à +5/+10 ans (dérive nodale).
5. **Lot 2 — anti-panne silencieuse** : script d'ingestion rejoué en CI mensuelle (non bloquant), diff contre les packs figés.
6. **v0.3** : `next_high`/`next_low` et fenêtres de marée remontent en tête — c'est la « raison de revenir ».
7. **Critère d'arrêt global (fusion des tripwires)** : à 8–10 semaines de travail réel, il faut ensemble : binaire en une commande + prédiction NOAA offline reproductible + cas Brest (hauteur documentée ou refus utile). Sinon : gel propre — publier conventions, scripts et leçons, convertir en bibliothèque expérimentale/note technique. Et à tout moment : si l'auteur lui-même ne l'utilise pas chaque semaine trois mois après v0.2 → gel.

---

## 4. Pré-mortem final (après MVP, 100 % Rust, audits hanoi/tangle, données vérifiées)

### Verdict Claude

Le plan a maintenant absorbé six tours de revue, deux audits structurels et un audit de données vérifié en ligne. Trois risques résiduels, dans l'ordre :

1. **La planification est devenue l'activité.** Six revues, zéro commit. Le document est excellent et c'est précisément le symptôme : chaque tour de relecture est plus gratifiant qu'un `cargo new`. Signal : un septième tour de revue sans code. Parade : le prochain artefact produit est le Lot 0, pas un document. Ce pré-mortem est le dernier acte de planification.
2. **Le seul gros risque technique neuf vient de la décision 100 % Rust** : les moindres carrés harmoniques de M2a s'écrivent à la main (pas de UTide dans la chaîne). C'est borné par la garde une-station, la contre-vérification UTide manuelle et la porte de sortie M2a, mais c'est le morceau le plus probable pour faire sauter la fenêtre 8–10 semaines. À accepter en connaissance de cause : c'est un choix de souveraineté, pas d'efficacité.
3. **Le reste est couvert** : conventions (timebox + règle des 14 jours), données (licences vérifiées, pas de dépendance à l'inaccessible), validation (benchmark figé, deux régimes), arrêt (tripwires datés). Les causes de mort des pré-mortems 1 et 2 ont toutes une parade dans le plan.

**GO** — à condition que le prochain livrable soit du code.

### Verdict ChatGPT (réflexion 1m39s)

Pré-mortem final en trois causes :
1. **Inertie de planification** : le projet meurt avec un plan excellent et aucun `cargo test`.
2. **`amar-calibrate` devient un clone UTide** : sélection automatique de constituants, Rayleigh, trous, poids, robustesse, diagnostics… et M2 explose.
3. **Brest déçoit** : résidus météo élevés, datum subtil, résultat expérimental peu « magique » après beaucoup d'effort.

Accord explicite sur le risque dominant : *« la planification devenue activité. Le plan est assez bon ; toute revue supplémentaire a un rendement négatif. »*

Sur le 100 % Rust : *« acceptable, mais seulement en version brutale et bornée. »* Erreur si on réimplémente UTide ; acceptable si M2a = constituants fixés, moindres carrés linéaires sin/cos, une station, une période, diagnostics minimaux. Utiliser une lib d'algèbre linéaire mature, ne pas écrire le solveur numérique soi-même. UTide reste un oracle manuel, pas un cahier des charges implicite à égaler.

**Verdict : GO-avec-conditions.**
1. Le prochain livrable est un commit M0 exécutable, pas une modification de plan.
2. M0 n'a pas Brest, pas serveur, pas calibration, pas PM/BM.
3. M2a a une porte de sortie publiée : si la calibration Rust dérive, taguer l'état, documenter, geler Brest plutôt que contaminer le cœur.
4. Chaque milestone se juge sur artefact lançable par un tiers, pas sur qualité architecturale.

### Synthèse finale

Les deux verdicts concordent : **GO**, l'unique condition dure étant que **le prochain artefact soit du code**. Les deux pré-mortems finaux désignent le même tueur (l'inertie de planification) et le même risque technique (la calibration Rust qui enfle) — la parade est écrite dans les deux : version brutale et bornée de `amar-calibrate`, lib d'algèbre linéaire mature, porte de sortie publiée. Ce document est le dernier acte de planification.

