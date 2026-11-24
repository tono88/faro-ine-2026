#!/bin/sh
# FARO · Asignación de operadores de campo
# Reparte operadores entre puestos con la menor distancia total. Cambie los dos archivos por los suyos.
#
# Uso: ejecútelo desde la carpeta del proyecto. Por defecto usa el programa "faro" que esté en
# su sistema; para usar el del contenedor: FARO="docker compose exec -T faro faro" sh operadores.sh
# Con los mismos datos de origen produce las mismas tablas que usa la pantalla.
set -eu
FARO="${FARO:-faro}"
$FARO asignar --operadores datos/ejemplos/operadores_ficticios.csv --puestos datos/ejemplos/puestos_ficticios.csv --salida salida/asignacion.csv || [ $? -eq 2 ]
echo "Listo."
