# FARO · Superficie construida por municipio (GHSL)
# Descarga los 30 mosaicos de GHSL que cubren Guatemala (10 épocas, 3 mosaicos) y suma la superficie construida por municipio. Requiere haber generado antes la capa de municipios (limites).
#
# Uso: en PowerShell, desde la carpeta del proyecto y con FARO levantado (docker compose up -d).
# Ejecuta dentro del contenedor el mismo procedimiento que construido.sh, con el mismo programa.
# Lo que genera queda dentro del contenedor; para copiarlo a esta carpeta:
#   docker compose cp faro:/app/datos/referencia ./referencia_reproducida
$ErrorActionPreference = "Stop"
docker compose exec -T faro sh scripts/reproducir/construido.sh
if ($LASTEXITCODE -ne 0) { throw "El procedimiento terminó con error $LASTEXITCODE" }
