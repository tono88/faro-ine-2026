# FARO · Inseguridad alimentaria aguda por departamento (CIF, SESAN)
# Descarga el informe oficial, extrae su texto y lee y verifica la tabla de población por fase. El contenedor no trae pdftotext: instálelo en él o use cif.sh en un sistema que lo tenga.
#
# Uso: en PowerShell, desde la carpeta del proyecto y con FARO levantado (docker compose up -d).
# Ejecuta dentro del contenedor el mismo procedimiento que cif.sh, con el mismo programa.
# Lo que genera queda dentro del contenedor; para copiarlo a esta carpeta:
#   docker compose cp faro:/app/datos/referencia ./referencia_reproducida
$ErrorActionPreference = "Stop"
docker compose exec -T faro sh scripts/reproducir/cif.sh
if ($LASTEXITCODE -ne 0) { throw "El procedimiento terminó con error $LASTEXITCODE" }
