#!/bin/sh
# FARO · Inseguridad alimentaria aguda por departamento (CIF, SESAN)
# Descarga el informe oficial, extrae su texto (requiere pdftotext, del paquete poppler-utils) y lee y verifica la tabla de población por fase.
#
# Uso: ejecútelo desde la carpeta del proyecto. Por defecto usa el programa "faro" que esté en
# su sistema; para usar el del contenedor: FARO="docker compose exec -T faro faro" sh cif.sh
# Con los mismos datos de origen produce las mismas tablas que usa la pantalla.
set -eu
FARO="${FARO:-faro}"
$FARO fuentes --manifiesto datos/crudo/manifiesto.jsonl url "https://portal.sesan.gob.gt/wp-content/uploads/2026/11/CIF_Guatemala-ct2026-Feb2027.pdf" --fuente sesan-cif --editor "Secretaría de Seguridad Alimentaria y Nutricional (SESAN); Grupo Técnico de la CIF Guatemala" --licencia "no indicada (publicación oficial)" || [ $? -eq 2 ]
pdftotext -layout datos/crudo/sesan-cif/CIF_Guatemala-ct2026-Feb2027.pdf datos/crudo/sesan-cif/informe.txt
$FARO cif --texto datos/crudo/sesan-cif/informe.txt
echo "Listo."
