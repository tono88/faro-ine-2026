# FARO · Precio de referencia a partir de cotizaciones
# Calcula media, mediana y moda por producto y elige según la regla de la Dirección de Precios. Cambie el archivo por el suyo.
#
# Uso: en PowerShell, desde la carpeta del proyecto y con FARO levantado (docker compose up -d).
# Ejecuta dentro del contenedor el mismo procedimiento que precios.sh, con el mismo programa.
# Lo que genera queda dentro del contenedor; para copiarlo a esta carpeta:
#   docker compose cp faro:/app/datos/referencia ./referencia_reproducida
$ErrorActionPreference = "Stop"
docker compose exec -T faro sh scripts/reproducir/precios.sh
if ($LASTEXITCODE -ne 0) { throw "El procedimiento terminó con error $LASTEXITCODE" }
