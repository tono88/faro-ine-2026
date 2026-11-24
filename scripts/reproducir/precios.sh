#!/bin/sh
# FARO · Precio de referencia a partir de cotizaciones
# Calcula media, mediana y moda por producto y elige según la regla de la Dirección de Precios. Cambie el archivo por el suyo.
#
# Uso: ejecútelo desde la carpeta del proyecto. Por defecto usa el programa "faro" que esté en
# su sistema; para usar el del contenedor: FARO="docker compose exec -T faro faro" sh precios.sh
# Con los mismos datos de origen produce las mismas tablas que usa la pantalla.
set -eu
FARO="${FARO:-faro}"
$FARO precios --cotizaciones datos/ejemplos/cotizaciones_ficticias.csv --salida salida/precios_referencia.csv || [ $? -eq 2 ]
echo "Listo."
