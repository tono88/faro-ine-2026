# FARO · Uso del suelo construido: residencial y no residencial (GHSL)
# Descarga el componente no residencial de GHSL (2000 y 2020, tres mosaicos) y, con la superficie construida total, calcula hectáreas residenciales y no residenciales por municipio y por bloque de 1 km. Requiere haber corrido antes el procedimiento «construido».
#
# Uso: en PowerShell, desde la carpeta del proyecto y con FARO levantado (docker compose up -d).
# Ejecuta dentro del contenedor el mismo procedimiento que uso_suelo.sh, con el mismo programa.
# Lo que genera queda dentro del contenedor; para copiarlo a esta carpeta:
#   docker compose cp faro:/app/datos/referencia ./referencia_reproducida
$ErrorActionPreference = "Stop"
docker compose exec -T faro sh scripts/reproducir/uso_suelo.sh
if ($LASTEXITCODE -ne 0) { throw "El procedimiento terminó con error $LASTEXITCODE" }
