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

## Estado al 23-nov-2026, 17:30
Hecho: núcleo de homologación con memoria, lectura de Excel/CSV reales, salida estándar con datos
personales eliminados o seudonimizados, solicitudes para RENAP/PNC/SE-CONRED, cliente CKAN del INE
con procedencia, servidor axum + PostgreSQL + pantalla web, sincronización automática con el INE,
crecimiento de edificaciones y regla de calidad del aire.

No probado todavía: `docker compose` en una máquina con Docker; descargas en vivo desde
datos.ine.gob.gt (la API real responde 105 conjuntos con el formato esperado); la pantalla en un
navegador.

## Pendiente, en orden
1. Levantar con Docker y corregir lo que falle.
2. Descargar el "Listado de lugares poblados" (Censo 2018) y **verificar `datos/catalogo/municipios.csv`
   código por código** (hoy es provisional); cargar lugares poblados con coordenadas.
3. Asignación de operadores de campo minimizando distancia (usa esas coordenadas).
4. Red vial con OpenStreetMap: lugares poblados sin conexión y enlaces probables.
5. Serie real de Open Buildings 2.5D Temporal por municipio (`scripts/`, requiere Earth Engine).
6. Precios de referencia con Guatecompras (OCDS).
7. Entregables: PDF, video, README final. Invitar al jurado al repositorio.
