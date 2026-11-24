#!/bin/sh
# FARO · Precios de referencia y canasta básica del INE
# Descarga la base de precios de referencia y los precios promedio de la canasta básica del portal del INE y los deja en tablas limpias.
#
# Uso: ejecútelo desde la carpeta del proyecto. Por defecto usa el programa "faro" que esté en
# su sistema; para usar el del contenedor: FARO="docker compose exec -T faro faro" sh precios_ine.sh
# Con los mismos datos de origen produce las mismas tablas que usa la pantalla.
set -eu
FARO="${FARO:-faro}"
$FARO fuentes --manifiesto datos/crudo/manifiesto.jsonl descargar precios-de-referencia || [ $? -eq 2 ]
$FARO referencias --entrada "$(ls datos/crudo/precios-de-referencia/d2c5d27a*.xlsx | head -1)" || [ $? -eq 2 ]
$FARO fuentes --manifiesto datos/crudo/manifiesto.jsonl descargar precios-promedio-canasta-basica-alimentaria || [ $? -eq 2 ]
$FARO canasta --entrada "$(ls datos/crudo/precios-promedio-canasta-basica-alimentaria/7a488e9d*.csv | head -1)" || [ $? -eq 2 ]
echo "Listo."
