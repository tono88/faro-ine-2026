# FARO · Materiales de construcción contra el IPMC
# Descarga el IPMC del INE, arma el catálogo de artículos y homologa contra él los precios publicados por los proveedores.
#
# Uso: en PowerShell, desde la carpeta del proyecto y con FARO levantado (docker compose up -d).
# Ejecuta dentro del contenedor el mismo procedimiento que construccion.sh, con el mismo programa.
# Lo que genera queda dentro del contenedor; para copiarlo a esta carpeta:
#   docker compose cp faro:/app/datos/referencia ./referencia_reproducida
$ErrorActionPreference = "Stop"
docker compose exec -T faro sh scripts/reproducir/construccion.sh
if ($LASTEXITCODE -ne 0) { throw "El procedimiento terminó con error $LASTEXITCODE" }
