#!/bin/sh
# FARO · Desnutrición crónica en escolares por municipio (Censo de Talla, SESAN)
# Descarga el libro del SIINSAN, compone el código INE de cada municipio, lo comprueba contra su nombre y escribe la tabla.
#
# Uso: ejecútelo desde la carpeta del proyecto. Por defecto usa el programa "faro" que esté en
# su sistema; para usar el del contenedor: FARO="docker compose exec -T faro faro" sh talla.sh
# Con los mismos datos de origen produce las mismas tablas que usa la pantalla.
set -eu
FARO="${FARO:-faro}"
$FARO fuentes --manifiesto datos/crudo/manifiesto.jsonl url "https://portal.siinsan.gob.gt/wp-content/uploads/VULNERABILIDAD_DC_MUNICIPAL_QUINTO_CENSO_2024_2.xlsx" --fuente sesan-censo-talla --editor "Secretaría de Seguridad Alimentaria y Nutricional (SESAN), SIINSAN" --licencia "no indicada (publicación oficial)" || [ $? -eq 2 ]
$FARO talla --entrada datos/crudo/sesan-censo-talla/VULNERABILIDAD_DC_MUNICIPAL_QUINTO_CENSO_2024_2.xlsx
echo "Listo."
