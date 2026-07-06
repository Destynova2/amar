# prompt.md — journal des prompts du projet amar

Log verbatim de tous les prompts utilisateur ayant mené au projet, dans l'ordre chronologique.
Convention : chaque entrée = date approximative, canal, prompt brut (fautes incluses, c'est un log).

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
