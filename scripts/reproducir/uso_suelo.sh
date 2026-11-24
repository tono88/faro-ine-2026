#!/bin/sh
# FARO · Uso del suelo construido: residencial y no residencial (GHSL)
# Descarga el componente no residencial de GHSL (2000 y 2020, tres mosaicos) y, con la superficie construida total, calcula hectáreas residenciales y no residenciales por municipio y por bloque de 1 km. Requiere haber corrido antes el procedimiento «construido».
#
# Uso: ejecútelo desde la carpeta del proyecto. Por defecto usa el programa "faro" que esté en
# su sistema; para usar el del contenedor: FARO="docker compose exec -T faro faro" sh uso_suelo.sh
# Con los mismos datos de origen produce las mismas tablas que usa la pantalla.
set -eu
FARO="${FARO:-faro}"
for epoca in 2000 2020; do
  for mosaico in R8_C10 R7_C10 R8_C9; do
    $FARO fuentes --manifiesto datos/crudo/manifiesto.jsonl url "https://jeodpp.jrc.ec.europa.eu/ftp/jrc-opendata/GHSL/GHS_BUILT_S_GLOBE_R2023A/GHS_BUILT_S_NRES_E${epoca}_GLOBE_R2023A_54009_100/V1-0/tiles/GHS_BUILT_S_NRES_E${epoca}_GLOBE_R2023A_54009_100_V1_0_${mosaico}.zip" --fuente ghsl-built-s-nres --editor "Comisión Europea, JRC: GHSL GHS-BUILT-S R2023A, componente no residencial (NRES)" --licencia "CC BY 4.0"
  done
done
$FARO uso-suelo
echo "Listo."
