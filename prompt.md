# prompt.md — journal des prompts du projet amar

Log verbatim de tous les prompts utilisateur ayant mené au projet, dans l'ordre chronologique.
Convention : chaque entrée = date approximative, canal, prompt brut (fautes incluses, c'est un log).

---

## 0. Modèles utilisés

Le projet est mené par orchestration : un modèle pilote (Claude Code) qui délègue le codage à codex en tmux et fait relire les décisions par un second modèle (ChatGPT Pro) pour la revue croisée.

| Phase | Prompts | Rôle | Modèle(s) |
|---|---|---|---|
| Origine | P1–P8 | idéation, faisabilité | Claude (claude.ai) |
| Planification | P9–P15 | orchestration + relecture croisée | Claude Code (Opus 4.8, contexte 1M) ; relectures **ChatGPT Pro** (raisonnement étendu) via dunst/Firefox |
| Développement | P16–P66 | codage des jalons | **codex exec** en tmux = **gpt-5.5** (reasoning `xhigh`) ; orchestration + revue *correctness* : Claude Code (Opus 4.8, 1M) ; relectures ponctuelles ChatGPT Pro |

Notes :
- **codex** tourne en sandbox `workspace-write` + `network_access=true`, un tmux par itération de jalon.
- La **revue correctness d'abord, puis audits `cli-*`** après chaque jalon est faite côté orchestrateur (Claude Code).
- Les versions de modèle reflètent l'état au moment du log (2026-07) ; codex lu depuis `~/.codex/config.toml`.

---

## 1. Origine — conversation claude.ai (≈ 2026-07-05)

### P1 — la question fondatrice
> mecansieme de calcule des marée permanant a traver le monde ?

### P2 — généralisation algorithmique
> ok donc on pourait theoriquement calculer les marée via un algo sur toute la plannette et pour tout les port ou lieu endroit ?

### P3 — le problème des données
> on peut pas avoir les donée, il y a pas d'autre source ? le dernier kilomètre reste un problème de données, on pourait récupere les marée historique pour predire le futur ?

### P4 — précision et niveau observé
> tes options : interpoler FES2022 (précision moyenne en côtier) on pourait indique le degre de precision sur le retour aussi ? et est ce que le niveaux de la mer est affichable

### P5 — le service web
> et faire un service web en rust tu fait un post en crul avec le point gps et la date et les info meteo ou pas et tu recup la marée ?

### P6 — bathymétrie et courants
*(prompt non retranscrit dans l'archive — question sur l'accès à la bathymétrie et aux courants pour améliorer la précision ; la réponse portait sur GEBCO/EMODnet/Litto3D, CMEMS, TELEMAC/SCHISM, SWOT, GNSS-IR)*

### P7 — le nom
> et un nomr pour le projet / programme ?

→ Décision : **amar** (ouverture Amar, ECO A00, 1.Ch3 — contient « mar », la mer).

---

## 2. Session Claude Code — 2026-07-06

### P8 — la commande du plan (ce prompt)
> ok fait un plan avec "mecansieme de calcule des marée permanant a traver le monde ?
> [… recollage intégral de la conversation claude.ai ci-dessus …]
> " l'idee un service ou on fait un curl avec le gps et date et on recupere la marée en retour un fois le plan fait fait le relire a chatgpt avec dunst le l'ai mis en version pro etendu et echanger pour avoir le plans le plus propre et simple et realisable et decoupper et tout en autoportant ne dev rien pour le moment plannifie organise imagine ameliorer recherche d'autre idée c'est tout et aussi log tout les prompte y comprit celui la dans un prompt.md

Consignes extraites :
- Un service : `curl` avec GPS + date → marée en retour.
- Rédiger le plan, puis le faire **relire par ChatGPT (version pro étendue) via dunst**, échanger pour obtenir le plan le plus propre, simple, réalisable.
- Découpage en lots **autoportants**.
- **Ne rien développer** pour le moment : planifier, organiser, imaginer, améliorer, chercher d'autres idées.
- Logger tous les prompts (y compris celui-ci) dans `prompt.md`.

### P9 — choix du canal ChatGPT
> non utilise chatgpt dans firefox

### P10 — autonomie dunst
> donne de l'autonomie a dunst arrete de me demander

### P11 — permissions dunst persistantes
> ajout des autorisation a dunst pour le piloter depuis claude code

→ Fait : règle `mcp__dunst` (serveur entier) ajoutée à `~/.claude/settings.json`, doublons retirés du `.claude/settings.local.json` du projet.

### P12 — vérification de la lecture complète
> tu a bien scroll chatgpt pour avoir tout sont retour ?

→ Oui : réponse tour 1 capturée intégralement (verdict → « Sources »), y compris retour arrière pour la fin du tableau des lots. Tour 2 (9 points) capturé d'un écran. Le tout intégré dans PLAN.md v2.1.

### P13 — pré-mortem
> ok fait un pre mortem et fait un pre mortem avec gpt aussi

→ Double autopsie dans `PREMORTEM.md` : volet Claude (8 causes + 3 tripwires), volet ChatGPT Pro étendu (tour 3 de la même conversation), puis synthèse fusionnée injectée dans PLAN.md.

### P14 — cycle revue → données → MVP → pré-mortem final
> ok refait un revue et amelioratin du plan ensuite resoume les a chatgpt ensuite regarde pour faire des mvp ensuite resoumet les a chatgpt et ensuite un premrteme final entre claude et chatgpt et aussi regarde les donée que l'on a, celle que l'on doit recuperer et celle que l'on aura pas et ajuste le/les mvp en conséquence

→ Exécuté en séquence : audit des données vérifié en ligne (`DATA.md` — API NOAA testées, LO 2.0 REFMAR confirmée, EOT20 CC-BY confirmé, constantes SHOM confirmées non distribuées) ; revue interne → PLAN v2.3 ; tour A ChatGPT (14 points intégrés) ; `MVP.md` (M0 moteur / M1 curl / M2 Brest / M3 fenêtres) ajusté aux données ; tour B ChatGPT sur les MVP ; pré-mortem final croisé (tour C) dans `PREMORTEM.md`.

### P15 — audits structurels, 100 % Rust, plume, validation finale
> et lance les skill hanoi et tangle sur le plan pour regarde si on peut l'ameliorer l'optimiser etc et on fait le dev en rust a 100/100 tout en rust et passe le skill plume partout et re une validation final

→ Fait : audit hanoi (historique des revues déplacé en annexe, Lot 2 scindé 2a/2b, table de jalons unique M ↔ lots ↔ versions, réservation de la fenêtre benchmark dès M2a) ; audit tangle (85/100 — boucle Lot 2b ⇄ Lot 4 levée, benchmark possédé par 2b, crate contrat `amar-pack` créée au Lot 0) ; décision 100 % Rust actée partout (`amar-calibrate` en Rust, UTide contre-vérification manuelle seulement, « Brest au premier mois » remplacé honnêtement) ; passe plume sur les cinq documents (tirets, flèches en prose, gras rhétorique, numérotation) ; tour final ChatGPT = pré-mortem croisé + GO/NO-GO.

---

## 3. Développement — M0 → v0.7 (2026-07-06 → 2026-07-07)

*(Log rattrapé le 2026-07-10 : ces prompts n'avaient pas été journalisés au fil de l'eau. Les relances de suivi pures — « ca avance ? », « regarde la ci » — sont incluses telles quelles.)*

### P16 — source de données / porte de sortie
> si [porte de sortie] tu peux utiliser dunst et sinon on a pas d'autre source de donée (même si on doit faire de l'agregation le truc dispo depui 2023 ?

→ REFMAR/data.shom.fr accessible par API (pas de blocage), calibration souveraine Brest retenue.

### P17 — lancement du dev
> ok lance le dev du lot 0 et M0 dans des tmux avec codex et lacne les parrtie que tu juge utilise dans cli cycle pour le code quand tu juge utile

→ Lot 0 + M0 en tmux avec codex ; audits cli-* lancés quand utiles (instruction récurrente, mise en mémoire).

### P18 — précision cm
> on peut pas être precis a 1 ou 2 cm ?

→ Convention annuelle NOS (V0 1er janvier, f/u mi-année) + correction Z0 → p95 NOAA 0,1-0,3 cm (M0.3).

### P19 — go m0.3
> go m0.3

### P20 — go m1
> go m1

### P21 — rappel cli-cycle
> et note toi de lancer les parrtie que tu juge utilise dans cli cycle pour le code quand tu juge utile

### P22 — go m2
> go m2

### P23 — go m3
> go m3

### P24 — améliorer encore la précision
> on peut pas encore ameliorer la precision ?

→ M2.2 : 16→37 constituants + calibration multi-année → Brest p95 26,6→15,8 cm ; plancher météo ~7 cm (baromètre inverse r²=43,8 %).

### P25 — M2.2 go
> [chemin M2.2 collé] go

### P26 — marqueurs IA / grob / audits
> les marquer ai pour ameliorer le dev ai sont bien la tu a pris les bonne idée dans le projet grob surtout la ci/cd et tu a lancer hanoi xray et tangle ?

### P27 — publication + élargissement France
> publique sur destynova2 et go l'élargissement France (v0.4, ~50 marégraphes REFMAR à quelques heures de pipeline chacun), et ducoup on a les coef et la hauteur de marée a combien de précision et il manque quoi la meto le barométre on gagnerait combien de précision avec ?

→ Repo public Destynova2 ; v0.4 = 11 ports France ; gain baromètre ~0,9 cm (Brest 7,9→7,0).

### P28 — v0.4 fin ?
> v0,4 fin ?

### P29 — Maison Blanche / taille de maille
> et sur le port de la maison blanceh a brest on est bien et est ce que l'on peut amliore / reduire la taille de la maille ?

### P30 — capteurs amateurs
> et on a pas une liste des capteur amateur avec relever qq part ?

### P31 — recherche forums
> tu a chercher sur gitub reddit les forum etc ?

→ GitHub/Reddit/Arduino/Hackaday : registre de capteurs amateurs = niche vide, confirmé.

### P32 — commit/push
> tout est fini commit et push ?

### P33 — grob CI/CD + image Docker
> ok tu a commit et push et regarde le projet grob pour t'inspirer de la ci cd et tu fait comme grob une image docker que l'on peut utiliser ?

→ v0.5 : Containerfile scratch/musl + workflow container GHCR multi-arch.

### P34 — fedora boot ?
> c'est quoi le fedora boot que je vien de voire ?

### P35 — commit/push
> ok tout est bon ? commit et push ?

### P36 — pousser le tag
> ba go pousser le commit d'avance non ?

### P37 — étapes suivantes
> la ci est passer c'est quoi les etape d'apres ?

### P38 — regarde la ci
> regarde la ci

### P39 — CI verte ? (saragossa)
> vert [log d'échec CI metal_backend collé] ???

→ Divergence bit-à-bit ULP (toolchain Metal) sur azerozero/saragossa, projet **séparé**, pas amar.

### P40 — binaire
> a mince et peut être faire un bineaire serait pratique non ?

→ v0.6 : workflow release, 4 binaires + SHA256SUMS téléchargeables (v0.6.2 verte après itérations).

### P41 — hygiène courte
> [menu hygiène : binaires release / crons / v0.7 France complète] go faire cela

### P42 — regarde la ci
> regarde la ci

---

## 4. Précision, courants, datum, courbe — v0.7 → v0.11 (2026-07-08 → 2026-07-10)

### P43 — il reste quoi à faire
> ok il reste quoi a faire ?

→ v0.7 : Méditerranée exclue à facteur < 2 (résultat honnête), Cherbourg/Calais retentés, 21 ports + outre-mer.

### P44 — précision partout / à améliorer
> et la on a combien de précision partout ? et il reste quoi a ameliorer ?

→ US 0,1-0,3 cm vs oracle ; France 7-20 cm RMS (météo, pas marée). Seul levier : météo/baromètre.

### P45 — courants derrière l'île
> et les courant derrier l'ile ou autre avec les vond et les mouvement ?

→ Courant de marée en un point = faisable (vecteur harmonique) ; tourbillons/sillage/vent = hydro spatiale 2D/3D, hors scope.

### P46 — coefficients bons 10 ans ?
> et tout les coef sont bon pour 10 ans ?

→ Astro validé 2026/2031/2036 (≤0,3 cm) ; hauteur absolue dérive ~3-4 cm/décennie (montée mer) → horizon valid_until.

### P47 — courants (données), validation 10 ans, valid_until
> [table courants collée] pour moi regarde sur meteo france il sont dispo et regarde par satélite gps pour les trouver exactemetn pour qalibrer nickel [...] on peut validée sur les 10 derniére année ? [...] oui fait le

→ Recherche courants : radar HF Iroise (Etalab) = analogue REFMAR mais surface + hors chenaux étroits. v0.8 = validation décennale mesurée + valid_until tamponné.

### P48 — courbe / symétrie / résolution
> et sur les coef la courbe de chaque marée est bonne c'est pas une symetrie ?
> minute pas minute heure par heure

→ Non symétrique (37 harmoniques dont overtides M4/M6/M8) ; modèle continu, exact à la minute (horaire échantillonne sous Nyquist).

### P49 — ça avance (v0.9)
> ca avance ?

### P50 — mutualiser les sources / maille
> et en mixans les source mutualisant les info on a une meilleur maille ?

→ Oui en principe (ossature EOT20 + ancres, confiance continue) mais faible à la côte ; backlog Lot 7 grille.

### P51 — satellite/GPS = position des sondes
> [...] tu est sur tu a aussi valider avec codex et de qu'il y a de recent l'idée etait de trouver l'emplacement des sonde pour avoir une precision autour

→ Positions déjà connues (catalogue REFMAR, sous-mètre) ; GNSS-IR/SONEL = densifier + rattacher le datum ; précision « autour » limitée par la décorrélation km, pas les coordonnées.

### P52 — bug « pas de retour »
> et le bug ou on avait pas de retour ?

→ Forme de réponse `tide` incohérente (point unique `height_m` vs série `series[]`) → null si mauvaise forme supposée. À normaliser (v0.11).

### P53 — proposer les 2 datums / condition réelle
> et proposer les 2 ? et lequel est bon en condition reel ?

→ Exposer ZH (navigation) + IGN69 (terrestre). Condition réelle : notre calage récent colle à l'eau (−1,1 cm) ; ZH = convention.

### P54 — Le Havre + Cherbourg/Calais
> [récap gain Le Havre + inclusion Cherbourg/Calais] oui et oui

→ v0.10 : catalogue amar-core étendu (additif) + sélection par port ; Le Havre 19,5→13,7 cm ; Cherbourg/Calais inclus → 23 ports.

### P55 — tous les harmoniques partout
> ba prend tout les harmonique dispo non et partout cela ameliorera la precision

→ Non : surajustement sur ports propres (Concarneau 0,0 gain). Règle = significatif + séparable par port (Rayleigh + SNR, style UTide), jugé sur validation réservée.

### P56 — courbe Brest 2027
> et sinon tu peux me faire la courbe de la marée de brest pour les 24h aujourdh'hui de l'année pro ?

→ Marégramme 2027-07-08 (artefact) : coef 91, marnage 5,99 m, 2 PM / 2 BM tracés.

### P57 — marée d'aujourd'hui, binaire local
> et si tu donne la marée d'aujourd'hui que avec le bin local ?

→ 2026-07-08 coef 52 (morte-eau), marnage 3,46 m. Contraste vive-eau/morte-eau à même date civile.

### P58 — comparaison marée.info
> et regrede le screen sur le bureau c'est maree info d'aujourd'hui
> aujourd'hiu a 21 04

→ vs SHOM/marée.info : timing 0-6 min, coef 52 vs 54, hauteur +7-14 cm (décalage datum ~10 cm). À 21:04 : amar 4,49 m ≈ courbe.

### P59 — souverain + correction datum
> [...] on peut reste souverain et aire la correction non ?

→ Oui : le datum est géodésique public (RAM/SONEL), ≠ constantes harmoniques SHOM. Correction souveraine possible.

### P60 — quand le ZH a été calibré
> ZH officiel (rattaché au RAM/SHOM) il a ete qualibrer quand ? cela explique surement le delta ?

→ Oui : niveau moyen SHOM d'époque ancienne ; le ~10 cm ≈ montée de la mer depuis (Brest ~1,3 mm/an). Empreinte déjà mesurée dans le biais décennal.

### P61 — marge de sécurité ?
> [...] lit ~10 cm sous l'eau réelle c'est peut être aussi une marge de securiter ?

→ Oui pour la profondeur (lire bas = sûr) ; mais trompeur pour tirant d'air/submersion (sous-estime). Datum selon le risque.

### P62 — sens de la sécurité
> c'est pas l'inverse plus tu a d'eau moin tu risque de toucher ?

→ Cohérent : lire bas = la réalité donne plus d'eau que prévu = sûr pour l'échouage. Le danger, c'est lire haut.

### P63 — fini ?
> ok tu a fini ?

→ Non à ce moment : container v0.10.1 encore en build.

### P64 — URL SHOM + commit/push
> ok tu pense quoi de "[URL data.shom.fr/donnees/refmar/3#...]" et tu a commit et push ?

→ URL décodée = portail SHOM station 3 = Brest + calque GEBCO ; source souveraine pour le RAM (v0.11). Tout commité/poussé (v0.10.1).

### P65 — go v0.11
> go v11

→ v0.11 : double datum souverain (ZH officiel par défaut + IGN69) + normalisation réponse `tide`. Tag v0.11.0 vert.

### P66 — hygiène commit/prompts/CI (ce prompt)
> tu a commit et push y comprit les prompet et la ci cd passe toute seul ?

→ Code v0.11 commité/poussé (tag v0.11.0, tout vert) ; **prompt.md remis à jour** (P16→P66, ce rattrapage) ; CI/CD automatique sur push/tag, verte — 2 patchs infra en route (v0.8.1 timeout container, v0.10.1 station_count smoke).
