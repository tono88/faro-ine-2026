@echo off
rem Levanta FARO (base de datos + servidor), descarga las fuentes abiertas y abre el portal.
chcp 65001 >nul
cd /d "%~dp0"
set LOG=registro_inicio.txt
echo ==== inicio %date% %time% ==== > %LOG%

docker version >> %LOG% 2>&1
if errorlevel 1 (
  echo ESTADO: DOCKER_NO_DISPONIBLE >> %LOG%
  echo Docker no esta corriendo. Abra Docker Desktop, espere a que arranque y vuelva a ejecutar este archivo.
  pause
  exit /b 1
)

powershell -NoProfile -ExecutionPolicy Bypass -File crear_env.ps1 >> %LOG% 2>&1
rem El puerto sale de .env: crear_env.ps1 elige uno libre si el configurado esta ocupado.
set PUERTO=8082
for /f "tokens=2 delims==" %%p in ('findstr /b "FARO_PUERTO=" .env') do set PUERTO=%%p
echo PUERTO: %PUERTO% >> %LOG%

echo Construyendo y levantando FARO. La primera vez tarda varios minutos...
docker compose up -d --build >> %LOG% 2>&1
if errorlevel 1 (
  echo ESTADO: FALLO_AL_LEVANTAR >> %LOG%
  echo No se pudo levantar. El detalle esta en %LOG%.
  pause
  exit /b 1
)
echo ESTADO: LEVANTADO >> %LOG%

set F=docker compose exec -T faro faro fuentes --manifiesto datos/crudo/manifiesto.jsonl
echo Descargando fuentes abiertas del INE y OpenStreetMap...
echo ---- inventario INE >> %LOG%
%F% inventario --salida datos/crudo/inventario_ine.csv >> %LOG% 2>&1
echo ---- lugares poblados >> %LOG%
%F% descargar censo-2018-lugares-poblados --formato XLSX --recurso "lugares poblados" >> %LOG% 2>&1
echo ---- proyecciones de poblacion >> %LOG%
%F% descargar proyecciones-de-poblacion-2015-2030 >> %LOG% 2>&1
echo ---- OpenStreetMap >> %LOG%
%F% url https://download.geofabrik.de/central-america/guatemala-latest.osm.pbf --fuente openstreetmap --editor "Colaboradores de OpenStreetMap (extracto de Geofabrik)" --licencia "ODbL 1.0" >> %LOG% 2>&1
docker compose ps >> %LOG% 2>&1
echo ESTADO: LISTO %date% %time% >> %LOG%

echo.
echo FARO esta en http://localhost:%PUERTO%
echo La clave de acceso que pide la pantalla es:
findstr FARO_TOKEN .env
start http://localhost:%PUERTO%
pause
