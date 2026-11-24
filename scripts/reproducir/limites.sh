#!/bin/sh
# FARO · Capa de municipios y departamentos
# Descarga los límites oficiales (CONRED/OCHA), los verifica contra los lugares poblados del censo y escribe las capas para los mapas.
#
# Uso: ejecútelo desde la carpeta del proyecto. Por defecto usa el programa "faro" que esté en
# su sistema; para usar el del contenedor: FARO="docker compose exec -T faro faro" sh limites.sh
# Con los mismos datos de origen produce las mismas tablas que usa la pantalla.
set -eu
FARO="${FARO:-faro}"
$FARO fuentes --manifiesto datos/crudo/manifiesto.jsonl url "https://data.humdata.org/dataset/0b20f310-7d22-479c-b7e2-e1bb9737fa72/resource/77e5e79b-d7b0-497c-aea2-2753768ec5f0/download/gtm_admin_boundaries.geojson.zip" --fuente limites-cod-ab --editor "CONRED / OCHA (COD-AB Guatemala, HDX: cod-ab-gtm)" --licencia "CC BY-IGO" || [ $? -eq 2 ]
$FARO limites --entrada datos/crudo/limites-cod-ab/gtm_admin_boundaries.geojson.zip || [ $? -eq 2 ]
echo "Listo."
