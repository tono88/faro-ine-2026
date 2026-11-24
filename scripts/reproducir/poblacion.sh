#!/bin/sh
# FARO · Población por municipio y año (proyecciones del INE)
# Descarga el libro de proyecciones y su diccionario, suma las edades, cruza los códigos con el catálogo y escribe la tabla municipio-año.
#
# Uso: ejecútelo desde la carpeta del proyecto. Por defecto usa el programa "faro" que esté en
# su sistema; para usar el del contenedor: FARO="docker compose exec -T faro faro" sh poblacion.sh
# Con los mismos datos de origen produce las mismas tablas que usa la pantalla.
set -eu
FARO="${FARO:-faro}"
$FARO fuentes --manifiesto datos/crudo/manifiesto.jsonl descargar proyecciones-de-poblacion-2015-2030 || [ $? -eq 2 ]
$FARO poblacion --entrada "$(ls datos/crudo/proyecciones-de-poblacion-2015-2030/56b97bc0*.xlsx | head -1)" --diccionario "$(ls datos/crudo/proyecciones-de-poblacion-2015-2030/7b47c92a*.xlsx | head -1)" || [ $? -eq 2 ]
echo "Listo."
