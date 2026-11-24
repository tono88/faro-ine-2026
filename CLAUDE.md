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
- Todo junto: `docker compose up -d --build` (o `iniciar.bat` en Windows) → http://localhost:8082
  (puerto en `FARO_PUERTO` de `.env`; `iniciar.bat` busca uno libre si está ocupado).
- Servidor sin Docker: PostgreSQL 16 + variables `DATABASE_URL`, `FARO_TOKEN`, `FARO_SAL`.

## Estado al 24-nov-2026
Hecho y probado en contenedor + Chromium: núcleo de homologación con memoria (lugares y valores de
catálogo por dimensión), lectura de Excel/CSV reales, salida estándar sin datos personales,
solicitudes RENAP/PNC/SE-CONRED, cliente CKAN con procedencia, servidor axum + PostgreSQL + pantalla
(8 pestañas), catálogo Censo 2018 verificado (20,036 lugares poblados con coordenadas), asignación
de operadores (método húngaro), población INE 2015–2030 por municipio (`datos/referencia/`), tasas
por 100 mil habitantes, avance de recepción por institución, precios de referencia a partir de
cotizaciones (`faro precios`), crecimiento de edificaciones, aire.

Campos de la solicitud además de los básicos: `dimension`, `respaldo`, `sin_dato` (ver
`crates/faro-homologa/src/solicitud.rs`). Estados nuevos: `fuera_de_contexto`, `fuera_de_catalogo`.

Hallazgos en datos del INE: el libro de proyecciones trae 16 códigos de municipio inexistentes
(0123–0138); se reubican con su diccionario (`datos/referencia/LEEME.md`).

Versiones: se publican como releases de GitHub (v0.1.0, v0.2.0, v0.3.0...). Al cerrar un bloque de
trabajo: actualizar README y `docs/08-versiones.md` y crear el release.

Cuidado: no usar `sed` masivo sobre el repositorio (un reemplazo de puerto dañó códigos del
catálogo; se restauró). En esta nube la imagen se construye con una copia del Dockerfile que agrega
el certificado del proxy; el Dockerfile del repositorio no lleva nada de eso.

No probado: `iniciar.bat` y `crear_env.ps1` en Windows; sincronización con el INE desde dentro del
contenedor; las solicitudes de RENAP/PNC con los archivos reales tras estos cambios (los archivos
no están en el repositorio: volver a pasarlos localmente).

## Pendiente, en orden
1. Red vial con OpenStreetMap: lugares poblados sin conexión y enlaces probables. Desde la nube no
   se pudo descargar el extracto (download.geofabrik.de corta la conexión): hacerlo en la máquina
   del equipo (`iniciar.bat` ya lo descarga a `datos/crudo/openstreetmap/`).
2. Históricos de Guatecompras (OCDS) para precios de referencia: el sitio responde con un desafío
   de Cloudflare a clientes automáticos desde la nube; probar desde la máquina del equipo. El
   cálculo ya está (`faro precios`); falta la fuente secundaria real.
3. Serie real de Open Buildings 2.5D Temporal por municipio (`scripts/`, requiere Earth Engine).
4. Volver a pasar los archivos reales de RENAP/PNC/SE-CONRED (no están en el repositorio) y
   actualizar `docs/06-hallazgos-datos-reales.md`; si RENAP trae columna con el nombre del
   municipio, declararla como `respaldo` en sus solicitudes.
5. Publicar releases v0.1.0–v0.4.0 (la sesión en la nube no tiene permiso; ver `docs/08-versiones.md`).
6. Entregables: PDF, video (2–3 min), README final con las secciones mínimas de la Nota
   Conceptual. Invitar al jurado al repositorio. Fusionar la rama de trabajo en `main`.
