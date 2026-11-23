# FARO · Datatón INE 2026 — contexto para trabajar en este repositorio

## Qué es
Herramienta para que el INE pida, reciba y homologue registros administrativos de otras
instituciones. Categoría 2 del Datatón INE 2026. **Entrega: jueves 26-nov-2026, 09:00.**
Leer primero `README.md` y `docs/` (necesidades del INE, arquitectura, plan, hallazgos).

## Reglas que no se rompen
- **No inventar datos.** Usar datos reales con fuente, fecha de descarga y licencia
  (`faro fuentes ...` deja el manifiesto). Lo ficticio vive solo en `datos/ejemplos/` y se rotula.
- **Ningún dato personal en el repositorio.** Los archivos de instituciones que compartió el INE
  (RENAP, PNC, SE-CONRED, INSIVUMEH) traen CUI, nombres y direcciones: se procesan localmente y
  nunca se suben. En documentos solo van conteos.
- **El sistema no adivina lugares.** Solo asigna código con coincidencia exacta y única o con una
  equivalencia confirmada por una persona. Lo demás se propone.
- Todo el código se escribe durante el evento (23–26 nov). No copiar código de proyectos previos.
- Base INE pública como fuente principal; declarar terceros, IA y servicios en la nube en el README.
- Commits pequeños y frecuentes en `main`: el historial es evidencia para el jurado.
- Documentación y nombres en español. Rust para núcleo, servidor y CLI.

## Cómo se ejecuta
- Pruebas: `cargo test` (deben pasar todas antes de cada commit).
- Todo junto: `docker compose up -d --build` (o `iniciar.bat` en Windows) → http://localhost:8090
  (puerto en `FARO_PUERTO` de `.env`; `iniciar.bat` busca uno libre si está ocupado).
- Servidor sin Docker: PostgreSQL 16 + variables `DATABASE_URL`, `FARO_TOKEN`, `FARO_SAL`.

## Estado al 23-nov-2026, 18:30
Hecho: núcleo de homologación con memoria, lectura de Excel/CSV reales, salida estándar con datos
personales eliminados o seudonimizados, solicitudes para RENAP/PNC/SE-CONRED, cliente CKAN del INE
con procedencia, servidor axum + PostgreSQL + pantalla web, sincronización automática con el INE,
crecimiento de edificaciones y regla de calidad del aire.

Probado el 23-nov: `docker compose up` en Linux (imagen, base de datos, volúmenes, reinicio) y la
pantalla en Chromium de punta a punta (envío, pendientes, confirmar, resultados, crecimiento);
inventario y descarga en vivo desde datos.ine.gob.gt (105 conjuntos, 876 recursos).

Catálogo verificado contra el "Listado de lugares poblados" del Censo 2018 (`faro catalogo
verificar|importar`): 22 departamentos y 340 municipios coinciden código por código; 20,036 lugares
poblados cargados, 19,723 con coordenadas. Detalle en `datos/catalogo/LEEME.md`.

Asignación de operadores (`faro asignar`, pestaña "Operadores de campo", `POST /api/asignar`):
método húngaro exacto, distancia en línea recta entre centroides; el servidor no guarda dónde vive
nadie. Probada en el navegador con `datos/ejemplos/operadores_ficticios.csv` y `puestos_ficticios.csv`.

No probado todavía: `iniciar.bat` y `crear_env.ps1` en Windows (el cambio de puerto automático se
escribió sin poder ejecutarlo); la sincronización con el INE desde dentro del contenedor.

## Pendiente, en orden
1. Red vial con OpenStreetMap: lugares poblados sin conexión y enlaces probables.
2. Serie real de Open Buildings 2.5D Temporal por municipio (`scripts/`, requiere Earth Engine).
3. Precios de referencia con Guatecompras (OCDS).
4. Entregables: PDF, video, README final. Invitar al jurado al repositorio.
