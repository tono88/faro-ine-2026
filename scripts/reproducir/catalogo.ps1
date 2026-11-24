# FARO · Catálogo oficial de lugares (Censo 2018)
# Descarga el Listado de lugares poblados del portal del INE, verifica el catálogo código por código y carga los lugares con coordenadas.
#
# Uso: en PowerShell, desde la carpeta del proyecto y con FARO levantado (docker compose up -d).
# Ejecuta dentro del contenedor el mismo procedimiento que catalogo.sh, con el mismo programa.
# Lo que genera queda dentro del contenedor; para copiarlo a esta carpeta:
#   docker compose cp faro:/app/datos/referencia ./referencia_reproducida
$ErrorActionPreference = "Stop"
docker compose exec -T faro sh scripts/reproducir/catalogo.sh
if ($LASTEXITCODE -ne 0) { throw "El procedimiento terminó con error $LASTEXITCODE" }
