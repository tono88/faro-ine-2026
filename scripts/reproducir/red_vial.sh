#!/bin/sh
# FARO · Red vial como grafo
# Descarga los caminos de OpenStreetMap (exportación de HOT en HDX) y calcula lugares sin conexión, enlaces probables y los caminos alrededor de cada enlace.
#
# Uso: ejecútelo desde la carpeta del proyecto. Por defecto usa el programa "faro" que esté en
# su sistema; para usar el del contenedor: FARO="docker compose exec -T faro faro" sh red_vial.sh
# Con los mismos datos de origen produce las mismas tablas que usa la pantalla.
set -eu
FARO="${FARO:-faro}"
$FARO fuentes --manifiesto datos/crudo/manifiesto.jsonl url "https://production-raw-data-api.s3.amazonaws.com/ISO3/GTM/roads/hotosm_gtm_roads_osm_shp.zip" --fuente openstreetmap-hot --editor "Colaboradores de OpenStreetMap, exportado por Humanitarian OpenStreetMap Team (HDX: hotosm_gtm_roads)" --licencia "ODbL 1.0" || [ $? -eq 2 ]
$FARO vial --carreteras datos/crudo/openstreetmap-hot/hotosm_gtm_roads_osm_shp.zip || [ $? -eq 2 ]
echo "Listo."
