#!/bin/sh
# FARO · Superficie construida por municipio (GHSL)
# Descarga los 30 mosaicos de GHSL que cubren Guatemala (10 épocas, 3 mosaicos) y suma la superficie construida por municipio. Requiere haber generado antes la capa de municipios (limites).
#
# Uso: ejecútelo desde la carpeta del proyecto. Por defecto usa el programa "faro" que esté en
# su sistema; para usar el del contenedor: FARO="docker compose exec -T faro faro" sh construido.sh
# Con los mismos datos de origen produce las mismas tablas que usa la pantalla.
set -eu
FARO="${FARO:-faro}"
for epoca in 1975 1980 1985 1990 1995 2000 2005 2010 2015 2020; do
  for mosaico in R8_C10 R7_C10 R8_C9; do
    $FARO fuentes --manifiesto datos/crudo/manifiesto.jsonl url "https://jeodpp.jrc.ec.europa.eu/ftp/jrc-opendata/GHSL/GHS_BUILT_S_GLOBE_R2023A/GHS_BUILT_S_E${epoca}_GLOBE_R2023A_54009_100/V1-0/tiles/GHS_BUILT_S_E${epoca}_GLOBE_R2023A_54009_100_V1_0_${mosaico}.zip" --fuente ghsl-built-s --editor "Comisión Europea, JRC: GHSL GHS-BUILT-S R2023A" --licencia "CC BY 4.0"
  done
done
$FARO construido || [ $? -eq 2 ]
echo "Listo."
