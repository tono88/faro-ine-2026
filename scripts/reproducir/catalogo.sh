#!/bin/sh
# FARO · Catálogo oficial de lugares (Censo 2018)
# Descarga el Listado de lugares poblados del portal del INE, verifica el catálogo código por código y carga los lugares con coordenadas.
#
# Uso: ejecútelo desde la carpeta del proyecto. Por defecto usa el programa "faro" que esté en
# su sistema; para usar el del contenedor: FARO="docker compose exec -T faro faro" sh catalogo.sh
# Con los mismos datos de origen produce las mismas tablas que usa la pantalla.
set -eu
FARO="${FARO:-faro}"
$FARO fuentes --manifiesto datos/crudo/manifiesto.jsonl descargar censo-2018-lugares-poblados --formato XLSX --recurso "lugares poblados" || [ $? -eq 2 ]
$FARO catalogo verificar --listado "$(ls datos/crudo/censo-2018-lugares-poblados/*.xlsx | head -1)" || [ $? -eq 2 ]
$FARO catalogo importar --listado "$(ls datos/crudo/censo-2018-lugares-poblados/*.xlsx | head -1)" || [ $? -eq 2 ]
echo "Listo."
