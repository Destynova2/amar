# Limitations M0

- Pas de serveur HTTP.
- Pas de Brest.
- Pas de calibration.
- Pas de PM/BM, pas de series temporelles, pas de coefficient.
- Pas de pression ni surcote meteo.
- Pas de grille.
- Couverture limitee aux stations NOAA harmoniques incluses dans le pack M0.
- Resolution spatiale volontairement conservatrice : station la plus proche a
  20 km maximum, sinon refus `no supported source`.
- Maree astronomique seule.
- Non utilisable pour la navigation.
- Methode M0 : `harmonic_basic_no_nodal`, sans corrections nodales appliquees.
