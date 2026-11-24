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
- Commits pequeños y frecuentes en `main`: el historial es evidencia para el jurado. Al cerrar cada
  bloque de trabajo, fusionar la rama de trabajo en `main` y subirla (autorizado por el equipo el 24-nov).
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
(19 secciones), catálogo Censo 2018 verificado (20,036 lugares poblados con coordenadas), asignación
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

Red vial hecha (`faro vial`, `crates/faro-territorio/src/vial.rs` y `shp.rs`, pestaña "Red vial"):
la fuente es la exportación de HOT en HDX (`hotosm_gtm_roads`, Shapefile), que sí se descarga desde
la nube; Geofabrik no. Resultados en `datos/referencia/red_vial_*`. Logros en `docs/09-logros.md`
(agregar ahí cada hallazgo nuevo).

Los archivos reales de instituciones NO van a `datos/crudo` ni a ninguna carpeta del repositorio:
se suben por la pantalla (pestaña 1), que los procesa en un temporal y no los guarda.

Agregado el 24-nov (v0.6.0), todo probado en contenedor + Chromium:
- Mapa (Leaflet incluido en `web/vendor/`), capas en `/capas` (catálogo JSON, GeoJSON/CSV con CORS),
  pantalla con menú lateral y paleta del INE (#0051b3, #4d9ae1, Noto Sans). Sin logotipo del INE.
- `faro limites` (COD-AB de HDX → `datos/referencia/municipios.geojson`, verificado contra el censo).
- `faro construido` (GHSL → `superficie_construida_municipio.csv`); lector propio de GeoTIFF en
  `geotiff.rs` porque el crate `tiff` falla con teselas vacías. Opciones en `docs/10-imagenes-satelitales.md`.
- Construcción: `faro ipmc` (catálogo en `datos/catalogo_precios/`, NO en `datos/catalogo/`, que solo
  lleva lugares), `faro construccion`, reglas en `datos/estandar/ipmc_reglas.toml` (borrador, 42 de 89
  artículos), `crates/faro-homologa/src/productos.rs`. Las decisiones de producto se guardan en la
  memoria con dimensión `ipmc`.
- Decisión del equipo (24-nov): el README no lleva aclaraciones sobre proyectos anteriores del equipo;
  no volver a agregarlas. La regla de no copiar código de proyectos previos sigue vigente.

Agregado después (v0.7.0): `poblacion::procesar` (pasos con constancia) y `POST /api/reproducir/poblacion`
(el contenedor necesita internet; en esta nube se probó con el servidor fuera de Docker y una base
temporal); sección "Proceso" (`/api/proceso`, `docs/12-proceso.md`); `faro referencias` y
`/api/referencias` (precios de referencia del INE); fuentes y citas en `datos/estandar/fuentes.json`
(única fuente para la pantalla; `docs/11-fuentes-y-citas.md` se regenera a mano con ese contenido);
`docs/00-que-se-hizo.md`. Al agregar una fuente nueva: manifiesto + `fuentes.json` + doc de citas.

Agregado después (v0.8.0): muestreos (`crates/faro-fuentes/src/muestreo.rs`, `datos/estandar/muestreos.toml`,
tablas `muestreos` y `precios_observados`, sección "Muestreos automáticos"); canasta básica
(`faro canasta`, `/api/canasta`); ficha flotante y `/api/municipio/:codigo`; modal de detalle con
animación (bloques de `superficie_construida_bloques.csv`); `/reporte?municipio=`; plantillas
`/api/solicitudes/:id/plantilla.csv`; `POST /api/equivalencias/tabla`; `scripts/reproducir/*.sh|ps1`
servidos en `/scripts/`; `homologar::lecturas_de_codigo` (códigos numéricos de otro departamento).
La solicitud de ejemplo está en `datos/ejemplos/` (no se muestra en la pantalla, por decisión del equipo).
El portal no usa modelos externos en ejecución: todo es código determinista (está dicho en "Acerca").
Los `.ps1` no se han probado en Windows.

Agregado después (v0.9.0): cuentas (`usuarios`, `sesiones`; Argon2; galleta `faro_sesion`; roles
admin/analista/consulta; `/api/sesion`, `/api/perfil`, `/api/usuarios`). `autorizar()` devuelve quién
actúa y ese nombre va a la bitácora; `FARO_TOKEN` queda solo como clave de servicio (`x-faro-token`).
Usuario inicial admin/admin con aviso para cambiarla (pedido expreso del equipo). CIF de SESAN:
`faro cif --texto` (texto de `pdftotext -layout`), `datos/referencia/cif_departamentos.csv`,
sección "Seguridad alimentaria"; dato por departamento, no por municipio. Censo de Talla 2024
(SESAN): `faro talla`, `datos/referencia/censo_talla_municipio.csv`, por municipio.

Agregado después (v0.10.0): lecturas de muestreo `vtex-catalogo`, `sitemap` y `magento` en
`muestreo.rs` (19 tareas; los catálogos corren en segundo plano, `EN_CURSO`); columnas `ean` y
`precio_lista`; `/api/publicados` y `/api/publicados.csv`; sección "Precios por rubro" (mismo código
de barras entre tiendas, cambios entre muestreos, informantes); levantamiento en
`datos/referencia/precios_publicados.csv` (se carga una vez, tarea `levantamiento-inicial`; no lleva
los rubros `construccion` ni `alimentos` para no mover las cifras de esas secciones). Mapa con niveles
(`cambiarNivel`, `resumenDepartamento`, `fichaDepartamento`), globo del enlace probable (`COMO_ENLACE`),
estilos para teléfono (`@media (max-width: 700px)`). Suma, Super del Barrio y Mi Super Fresh no tienen
catálogo en línea; PriceSmart pide navegador con sesión. Memoria técnica y guiones de prueba: `.faro/`.

## Pendiente, en orden
1. Volver a pasar los archivos reales de RENAP/PNC/SE-CONRED por la pantalla y actualizar
   `docs/06-hallazgos-datos-reales.md`; si RENAP trae columna con el nombre del municipio,
   declararla como `respaldo` en sus solicitudes.
2. Históricos de Guatecompras (OCDS) para precios de referencia: el sitio responde con un desafío
   de Cloudflare a clientes automáticos desde la nube; probar desde la máquina del equipo.
3. Ampliar las reglas del IPMC a más artículos y sumar proveedores con precios (ferreterías del
   interior: sus precios no están en línea; haría falta cotización directa).
4. Publicar releases v0.1.0–v0.9.0 (la sesión en la nube no tiene permiso; ver `docs/08-versiones.md`).
5. Más fuentes de gobierno ya verificadas (descarga directa, con código de municipio), por integrar
   en este orden: CONRED incidentes 2024 y MAGA afectaciones a cultivos 2024 (portal del INE,
   conjunto de8af067…); SEGEPLAN ranking de gestión municipal 2025 (JSON); MSPAS inventario de
   sistemas de agua 2024; MINEDUC establecimientos 2025; PNC hechos de tránsito 2025; INAB
   PROBOSQUE 2024; SAT parque vehicular (solo nombres de municipio: buen caso de homologación).
   Bloqueados desde la nube (Cloudflare): MINFIN, datos.gob.gt, SIGSA/MSPAS, MEM, MIDES, CONAP.
6. Entregables: PDF, video (2–3 min). Invitar al jurado al repositorio.
