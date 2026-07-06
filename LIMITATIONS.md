# Limitations M1

- Marée astronomique seule.
- Non utilisable pour la navigation.
- Pas de Brest calculé : Brest retourne volontairement `422 no_supported_source`
  jusqu'au pack expérimental M2.
- Pas de calibration.
- Pas de PM/BM, pas de séries temporelles, pas de coefficient.
- Pas de pression, pas de surcote météo.
- Pas de grille.
- Pas d'OpenAPI, pas de page démo, pas de Docker.
- Couverture limitée aux 8 stations NOAA harmoniques incluses dans le pack M1.
- Résolution spatiale volontairement conservatrice : station la plus proche à
  20 km maximum par défaut, sinon refus utile avec station la plus proche.
- Confiance M1 heuristique, basée seulement sur la distance à la station :
  A <= 2 km, B <= 10 km, C <= 20 km.
- Méthode de calcul : `station_harmonics_v0`, avec corrections nodales
  Schureman.
