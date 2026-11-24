# FARO · Recibir y homologar un envío
# Valida un archivo de una institución contra su solicitud, homologa lugares y valores y escribe la salida estándar, los pendientes y el reporte. Cambie la solicitud y el archivo por los suyos.
#
# Uso: en PowerShell, desde la carpeta del proyecto y con FARO levantado (docker compose up -d).
# Ejecuta dentro del contenedor el mismo procedimiento que registros.sh, con el mismo programa.
# Lo que genera queda dentro del contenedor; para copiarlo a esta carpeta:
#   docker compose cp faro:/app/datos/referencia ./referencia_reproducida
$ErrorActionPreference = "Stop"
docker compose exec -T faro sh scripts/reproducir/registros.sh
if ($LASTEXITCODE -ne 0) { throw "El procedimiento terminó con error $LASTEXITCODE" }
