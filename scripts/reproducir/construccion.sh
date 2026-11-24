#!/bin/sh
# FARO · Materiales de construcción contra el IPMC
# Descarga el IPMC del INE, arma el catálogo de artículos y homologa contra él los precios publicados por los proveedores.
#
# Uso: ejecútelo desde la carpeta del proyecto. Por defecto usa el programa "faro" que esté en
# su sistema; para usar el del contenedor: FARO="docker compose exec -T faro faro" sh construccion.sh
# Con los mismos datos de origen produce las mismas tablas que usa la pantalla.
set -eu
FARO="${FARO:-faro}"
$FARO fuentes --manifiesto datos/crudo/manifiesto.jsonl descargar indice-de-precios-de-materiales-de-construccion-ipmc --recurso "Septiembre 2026" || [ $? -eq 2 ]
$FARO fuentes --manifiesto datos/crudo/manifiesto.jsonl descargar indice-de-precios-de-materiales-de-construccion-ipmc --recurso "2025" || [ $? -eq 2 ]
$FARO ipmc --actual "$(ls datos/crudo/indice-de-precios-de-materiales-de-construccion-ipmc/7a374e7a*.xlsx | head -1)" --anterior "$(ls datos/crudo/indice-de-precios-de-materiales-de-construccion-ipmc/01216a18*.xlsx | head -1)" || [ $? -eq 2 ]
$FARO construccion || [ $? -eq 2 ]
echo "Listo."
