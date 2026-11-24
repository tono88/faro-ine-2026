#!/bin/sh
# FARO · Recibir y homologar un envío
# Valida un archivo de una institución contra su solicitud, homologa lugares y valores y escribe la salida estándar, los pendientes y el reporte. Cambie la solicitud y el archivo por los suyos.
#
# Uso: ejecútelo desde la carpeta del proyecto. Por defecto usa el programa "faro" que esté en
# su sistema; para usar el del contenedor: FARO="docker compose exec -T faro faro" sh registros.sh
# Con los mismos datos de origen produce las mismas tablas que usa la pantalla.
set -eu
FARO="${FARO:-faro}"
$FARO recibir --solicitud datos/ejemplos/solicitud_hechos_transito.toml --entrada datos/ejemplos/envio_hechos_transito.csv --salida salida/envio || [ $? -eq 2 ]
echo "Listo."
