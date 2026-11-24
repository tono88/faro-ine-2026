#!/bin/sh
# FARO · Condiciones de vida por municipio y techos por lugar poblado (Censo 2018)
# Descarga los cuadros del Censo 2018 del portal del INE y calcula quince indicadores por municipio (alfabetismo, internet, agua, drenaje, electricidad, techo, piso, empleo, educación, basura, tenencia) y el material del techo por lugar poblado. Si el INE cambia el orden de una columna, el proceso se detiene.
#
# Uso: ejecútelo desde la carpeta del proyecto. Por defecto usa el programa "faro" que esté en
# su sistema; para usar el del contenedor: FARO="docker compose exec -T faro faro" sh censo_indicadores.sh
# Con los mismos datos de origen produce las mismas tablas que usa la pantalla.
set -eu
FARO="${FARO:-faro}"
$FARO fuentes --manifiesto datos/crudo/manifiesto.jsonl descargar censo-2018-lugares-poblados --formato XLSX || [ $? -eq 2 ]
$FARO censo-indicadores
echo "Listo."
