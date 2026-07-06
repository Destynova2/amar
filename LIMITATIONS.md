# Limitations M2

- Marée astronomique seule.
- Non utilisable pour la navigation.
- Brest est calculé uniquement via le pack expérimental `refmar:3`.
- Brest n'utilise pas de constantes SHOM : les constantes sont dérivées des
  observations REFMAR et ne leur sont pas équivalentes.
- Brest est borné à une station, un datum, une période et un pack. Aucun autre
  port français n'est supporté.
- La validation Brest mesure un résidu = niveau d'eau observé − marée
  astronomique prédite (météo incluse), pas une validation officielle.
- Brest n'a pas de grade A/B/C. La réponse expose le p95 du benchmark figé.
- Pas de PM/BM, pas de séries temporelles, pas de coefficient.
- Pas de pression, pas de surcote météo.
- Pas de grille.
- Pas d'OpenAPI, pas de page démo, pas de Docker.
- Couverture limitée aux 8 stations NOAA harmoniques et à Brest expérimental.
- Résolution spatiale volontairement conservatrice : station la plus proche à
  20 km maximum par défaut, sinon refus utile avec station la plus proche.
- Confiance NOAA heuristique, basée seulement sur la distance à la station :
  A <= 2 km, B <= 10 km, C <= 20 km.
- Un rayon CLI/API supérieur à 20 km n'étend pas le domaine : les sources
  plus lointaines sont refusées.
- Méthode de calcul : `station_harmonics_v0`, avec corrections nodales
  Schureman.
