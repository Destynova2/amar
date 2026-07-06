# Limitations v0.4

- Marée astronomique seule.
- Les fenêtres de seuil sont des fenêtres de marée astronomique seule :
  pression, vent, surcote, débit, houle et événements météo ne sont pas pris en
  compte.
- L'incertitude publiée s'applique aux seuils. Pour les stations REFMAR
  expérimentales, une fenêtre calculée autour de `above_m`/`below_m` porte une
  incertitude verticale de l'ordre du benchmark figé de la station.
- Non utilisable pour la navigation.
- Les stations françaises expérimentales n'utilisent pas de constantes SHOM :
  les constantes sont dérivées des observations REFMAR et ne leur sont pas
  équivalentes.
- La validation REFMAR mesure un résidu = niveau d'eau observé − marée
  astronomique prédite (météo incluse), pas une validation officielle SHOM.
- Les stations REFMAR n'ont pas de grade A/B/C. La réponse expose le p95 du
  benchmark figé.
- Le pack France v0.4 livré couvre 11 ports Manche/Atlantique :
  Boulogne-sur-Mer, Concarneau, Dieppe, Dunkerque, La Rochelle-Pallice,
  Le Conquet, Le Havre, Ouistreham, Roscoff, Saint-Malo et Saint-Nazaire.
- Cherbourg et Calais sont exclus de ce pack : la fenêtre de validation
  `2026-04-01T00:00:00Z/2026-07-01T00:00:00Z` n'a aucune observation validée
  `source=4`.
- PM/BM, séries temporelles, fenêtres et coefficient sont disponibles pour les
  stations françaises incluses.
- Le coefficient est expérimental : il dérive de notre calibration Brest, pas
  de l'annuaire officiel. Il est borné à 20..120 et publié avec
  `coefficient_experimental`.
- Pas de pression, pas de surcote météo.
- Pas de grille.
- Pas d'OpenAPI, pas de page démo.
- L'image Docker/Podman sert la marée astronomique offline depuis les packs
  embarqués et n'émet aucune donnée sortante.
- Couverture limitée aux 8 stations NOAA harmoniques, à Brest expérimental et
  aux 11 ports du pack France v0.4.
- Résolution spatiale volontairement conservatrice : station la plus proche à
  20 km maximum par défaut, sinon refus utile avec station la plus proche.
- Confiance NOAA heuristique, basée seulement sur la distance à la station :
  A <= 2 km, B <= 10 km, C <= 20 km.
- Un rayon CLI/API supérieur à 20 km n'étend pas le domaine : les sources
  plus lointaines sont refusées.
- Méthode de calcul : `station_harmonics_v0`, avec corrections nodales
  Schureman.
