# FARO · Condiciones de vida por municipio y techos por lugar poblado (Censo 2018)
# Descarga los cuadros del Censo 2018 del portal del INE y calcula quince indicadores por municipio (alfabetismo, internet, agua, drenaje, electricidad, techo, piso, empleo, educación, basura, tenencia) y el material del techo por lugar poblado. Si el INE cambia el orden de una columna, el proceso se detiene.
#
# Uso: en PowerShell, desde la carpeta del proyecto y con FARO levantado (docker compose up -d).
# Ejecuta dentro del contenedor el mismo procedimiento que censo_indicadores.sh, con el mismo programa.
# Lo que genera queda dentro del contenedor; para copiarlo a esta carpeta:
#   docker compose cp faro:/app/datos/referencia ./referencia_reproducida
$ErrorActionPreference = "Stop"
docker compose exec -T faro sh scripts/reproducir/censo_indicadores.sh
if ($LASTEXITCODE -ne 0) { throw "El procedimiento terminó con error $LASTEXITCODE" }
