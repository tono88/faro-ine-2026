# FARO · Precios de referencia y canasta básica del INE
# Descarga la base de precios de referencia y los precios promedio de la canasta básica del portal del INE y los deja en tablas limpias.
#
# Uso: en PowerShell, desde la carpeta del proyecto y con FARO levantado (docker compose up -d).
# Ejecuta dentro del contenedor el mismo procedimiento que precios_ine.sh, con el mismo programa.
# Lo que genera queda dentro del contenedor; para copiarlo a esta carpeta:
#   docker compose cp faro:/app/datos/referencia ./referencia_reproducida
$ErrorActionPreference = "Stop"
docker compose exec -T faro sh scripts/reproducir/precios_ine.sh
if ($LASTEXITCODE -ne 0) { throw "El procedimiento terminó con error $LASTEXITCODE" }
