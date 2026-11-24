# Bitácora técnica de FARO (memoria de trabajo)

Para quien retome el proyecto (persona o asistente) sin el historial de la conversación.
`CLAUDE.md` tiene las reglas y el estado; aquí va el detalle operativo. Sin secretos ni datos personales.

## Cómo se trabaja
- Rama de trabajo `claude/eager-pasteur-acpxv8`; al cerrar cada bloque: `git checkout main && git merge --ff-only <rama> && git push origin main` (autorizado por el equipo el 24-nov).
- Releases y etiquetas: la sesión en la nube NO puede crearlos (403). Se le dan al usuario comandos `gh release create vX --target <sha>`.
- Antes de cada commit: `cargo test` (69 pruebas al 24-nov). Probar la pantalla con Playwright (guiones en `.faro/pruebas/`).
- El usuario (Estuardo) corre en Windows + Docker Desktop, puerto 8082, PowerShell. No tiene `gh` salvo que lo instale. Quiere: avanzar mucho, README al día, todo citado, merge a main, respuestas en español.
- Decisiones del usuario que no se discuten: sin aclaraciones sobre proyectos previos en el README; sin institución de ejemplo en la pantalla; admin/admin como usuario inicial; puerto 8082.

## Probar en la nube
- Docker: `dockerd` hay que arrancarlo a mano (`nohup dockerd &`). Docker Hub da 429: usar `mirror.gcr.io/library/...`. Dentro de los contenedores no hay internet ni proxy.
- `.faro/pruebas/construir_en_nube.sh [limpio]`: copia el Dockerfile agregando el certificado del proxy (carpeta `$S/ca` con `/root/.ccr/ca-bundle.crt`), construye, levanta y espera a `:8082`.
- Para probar lo que necesita internet (muestreos, reproducir población): servidor fuera de Docker con una base temporal:
  `docker run -d --name pgtmp -e POSTGRES_PASSWORD=prueba -e POSTGRES_USER=faro -e POSTGRES_DB=faro -p 55432:5432 postgres:16` y
  `DATABASE_URL="host=127.0.0.1 port=55432 user=faro password=prueba dbname=faro" FARO_TOKEN=servicio FARO_SAL=sal FARO_PUERTO=8099 FARO_SYNC_HORAS=0 ./target/debug/faro-servidor`.
- `curl` a localhost: usar `--noproxy localhost` (no `--noproxy *` dentro de una variable: el `*` se expande).
- Playwright: `NO_PROXY=localhost node guion.js http://localhost:8099 ...`. Las `<option>` no cuentan como visibles (`state:'attached'`). Botones del menú ocultos (perfil/usuarios) solo tras iniciar sesión.

## Trampas ya pisadas
- NUNCA `sed` masivo en el repo: cambió "8090" dentro de códigos de `lugares_poblados.csv`.
- `datos/catalogo/` SOLO lleva catálogos de lugares (el servidor carga todos sus CSV como lugares). Precios van en `datos/catalogo_precios/`.
- `tiff` (crate) falla con teselas vacías de GHSL → lector propio `geotiff.rs`.
- Un `id` duplicado (`#mapa` sección y lienzo) rompió Leaflet; el lienzo es `#lienzo`. `mapa` es la variable del mapa; el enlace a OSM es `enMapa()`.
- Consultas con `coalesce($2, $4)` necesitan tipos explícitos (`$2::bool`).
- `.mapa-marco` necesita `align-self:start` o la ficha flotante se desborda.
- CSV del catálogo: finales de línea CRLF (se respetan para que el diff sea mínimo).
- El archivo de canasta básica del INE trae comas sin comillas en la descripción.
- El libro de proyecciones del INE codifica 0123–0138 en vez de 0102–0117 (se reubica con su diccionario).
- COD-AB trae intercambiados los códigos de Sanarate/Sansare y dos "municipios" que son lagos (0100, 0700).

## Fuentes: qué responde desde la nube
- Sí: datos.ine.gob.gt (CKAN), data.humdata.org y su S3 (HOT/COD-AB), jeodpp.jrc.ec.europa.eu (GHSL), portal.sesan.gob.gt, portal.siinsan.gob.gt, ranking.segeplan.gob.gt, precios.maga.gob.gt, portal.sat.gob.gt, catálogos VTEX (walmart.com.gt, cemaco.com, construfacil.com.gt, ferco.com.gt).
- No (Cloudflare o corte): download.geofabrik.de, overpass, guatecompras (ocds y www), datos.minfin.gob.gt, datos.gob.gt, sigsa.mspas, mem.gob.gt, mides, conap, Facebook/Instagram, Encuentra24.
- Cada descarga: `faro fuentes --manifiesto datos/crudo/manifiesto.jsonl url|descargar ...` (deja URL, fecha, SHA-256). `datos/crudo/` no se versiona.
- Fuente nueva = manifiesto + `datos/estandar/fuentes.json` (única fuente de citas para la pantalla) + regenerar `docs/11-fuentes-y-citas.md` + `datos/referencia/LEEME.md`.

## Mapa del código
- `crates/faro-homologa`: `normalizar` (clave, letras dañadas), `catalogo`, `homologar` (estados; `lecturas_de_codigo`), `memoria` (lugares y valores por dimensión), `solicitud` (dimension, respaldo, sin_dato), `recepcion`, `lectura` (Excel/CSV), `censo`, `productos` (IPMC: medidas, comparable/revisar/distinto).
- `crates/faro-territorio`: `asignacion` (húngaro), `poblacion` (procesar con pasos), `vial` (grafo, componentes, enlaces), `shp`, `geo`, `geotiff`, `construido` (Mollweide, suma por municipio, bloques), `precios` (media/mediana/moda), `construccion`, `crecimiento`, `aire`.
- `crates/faro-fuentes`: `ckan`, `procedencia`, `muestreo` (VTEX).
- `crates/faro-servidor`: `main.rs` (todo: API, capas, cuentas, muestreos), `esquema.sql`, `web/index.html` (una sola página), `web/reporte.html`, `web/vendor/leaflet.*`.
- `crates/faro-cli`: comandos recibir, homologar, confirmar, fuentes, catalogo, poblacion, limites, vial, construido, ipmc, construccion, referencias, canasta, cif, talla, asignar, precios, crecimiento, aire.
- Datos generados en `datos/referencia/` y `datos/catalogo_precios/`; cada uno con su comando en `scripts/reproducir/`.

## Pantalla (index.html)
- `tabla(sel, columnas, filas, vacio)`, `api(ruta, opciones)`, `aviso(texto, mal)`, `num()`, `q()` (quetzales), `esc()`.
- `cargar = {seccion: funcion}` se llama al cambiar de sección; botones del menú con `data-s`.
- Mapa: `INDICADORES`, `pintarMapa()`, `ficha(p)` (flotante), `detalle(codigo)` (modal con animación), capas opcionales con `capaOpcional()`.
- Sesión: `yo`, `verSesion()`, `pintarSesion()`; un 401 abre el diálogo de ingreso.

## Cifras de referencia (para detectar regresiones)
- Catálogo: 22 / 340 / 20,036 (19,723 con coordenadas). Población país 2018: 16,346,950; 2030: 19,201,062.
- Red vial: 144,866 km; 2,101 tramos; 163 lugares en tramo suelto, 544 sin camino; 83 enlaces (51.3 km).
- Construido: 30,722 ha (1975) → 93,297 ha (2020). Límites: 86.9 % de lugares dentro de su figura.
- Construcción: 116 proveedores, 2,332 precios; 70 comparables, 563 por revisar, 99 de otra medida; 20 artículos con precio; 42 reglas.
- CIF: 18,312,373 analizadas; 3,247,361 en Fase 3+. Talla: 340 municipios (90 muy alta, 90 alta, 89 moderada, 71 baja).

## Pendientes y siguientes ideas
Ver `CLAUDE.md` ("Pendiente, en orden"). Fuentes de gobierno verificadas por integrar: CONRED incidentes y MAGA cultivos (INE, conjunto de8af067), SEGEPLAN ranking (JSON 17 MB), MSPAS agua, MINEDUC establecimientos, PNC tránsito, INAB PROBOSQUE, SAT parque vehicular.

## Bloque v0.10.0 (24-nov, tarde)
- Muestreos nuevos probados fuera de Docker (servidor en :8099 + `pgtmp`). Tiempos: catálogo VTEX ~1–3 min; EPA 100 páginas ~3 min; farmacias 300 páginas ~20 min (3.4 s por página). Reiniciar el servidor corta las corridas en curso.
- La comparación entre muestreos con autounión tardaba 26 s (plan de bucles anidados con parámetros); con `lead()` en ventana, <0.2 s.
- `DISTINCT ON` debe incluir `rubro`: la misma URL de Walmart está en `alimentos` y `supermercado`.
- Los puntos en lienzo (canvas) quedan tapados por polígonos SVG opacos: usar un panel propio (`mapa.createPane('puntos')`, z 450).
- Para probar "cambios" se insertaron filas con fecha anterior en la base temporal (tarea `prueba-temporal`); no van al repositorio.
- Guiones: `.faro/pruebas/movil_y_niveles.js` (390 px, desbordes, niveles) y `precios_por_rubro.js`.
- Ideas pendientes: bajar a subcategorías VTEX para pasar el tope de 2,500 por categoría; avance de recepción por departamento; kemik.gt responde 403.

## Bloque v0.11.0 (24-nov, noche)
- La descarga `/api/publicados.csv` tenía tope de 50,000 filas y cortó el levantamiento; se exporta con `\copy` desde la base temporal (consulta en el historial de este bloque) y el tope subió a 300,000.
- Para levantar sin esperar horas: copia de `muestreos.toml` con `max_resultados = 450` en las tareas `sitemap` (`FARO_MUESTREOS=/tmp/...`). Con el tope real (3,000 páginas a ~4 s) cada farmacia tarda más de 3 horas.
- Walmart a tope: 30,055 productos, ~35 min. No reiniciar el servidor que muestrea: usar otro puerto para probar la pantalla (misma base).
- Para probar pistas y cobertura sin archivos reales: filas `PRUEBA` en `pendientes`/`envios` de la base temporal (se borran) y `page.route` en Playwright.
- Guion nuevo: `.faro/pruebas/mapa_aislar_y_canasta.js`.
- Release: `gh release create` exige el SHA completo o una rama en `--target` (el corto da 422).

## Bloque v0.12.0 (24-nov, noche)
- Presentaciones releídas (texto extraído de los .pptx): propuestas por dirección y su estado en `docs/01-necesidades-ine.md`.
- Guion: `.faro/pruebas/informe_balance_buscador.js`. El informe abre en pestaña nueva (`ctx.waitForEvent('page')`).
- Nombres de WooCommerce traen entidades numéricas: `sin_entidades()`.

## Bloques v0.12.1 y v0.13.0 (24-nov, noche)
- FALLO GRAVE que las pruebas no vieron: el clic real sobre el mapa no abría la ficha (el foco del mapa desplazaba `main`). Las pruebas llamaban `ficha()` por código. Ahora `clic_real_mapa.js` usa ratón real en 4 tamaños: correrla siempre.
- Levantamiento con fotos: `/tmp/cadena.sh` encadena tareas por sitio; ~1 h en total. Exportar con `\copy` filtrando `tarea <> 'levantamiento-inicial'` e incluyendo `imagen`. El CSV pesa 26 MB.
- `www.ine.gob.gt` activa protección anti-bots (Radware) tras ~10 solicitudes; `datos.ine.gob.gt` no.
- Calibración de fotos: los pares útiles son La Torre vs Walmart (Walmart/Paiz/Maxi comparten archivo de foto).
- FAO TCF: sección Guatemala en págs. 222–225 (234–237 del PDF); el primario se deduce del orden de filas.

## Bloque v0.14.0 (24-nov)
- El usuario rechazó las pistas de país/edad en pendientes de municipio ("incoherente"): no volver a proponerlas.
- `dockerd` puede caerse entre turnos: `nohup dockerd &` y `docker start pgtmp`.
- Diferencias de precio: la lista muestra 300 y lo decidido baja al final; para comprobar una decisión usar `q=` o los conteos.
- PDF entregable: `node scripts/entregables/generar_pdf.js http://localhost:PUERTO` (toma capturas en vivo; convertidor Markdown mínimo propio).

## Bloque v0.15.0 (24-nov)
- El margen de regresión (±2 SE o t) NO sirve con series GHSL: cobertura real 45 %. Se usa el error empírico de la prueba hacia atrás. `faro verificar` falla si las constantes dejan de coincidir.
- El contraste HBA (derivado ÷ procesado) solo equivale a la tasa si lo procesado va a ese derivado (maíz no).
- Seguridad: tras estos cambios, cualquier guion que escriba debe entrar con `_sesion.js`; con admin/admin sin cambiar, todo POST da 403.
- Interfaz: `small` dentro de tablas de 13 px baja de 11 px en celular; botones `.chico` necesitan 36 px en táctil.
- Tableros: el intérprete vive en `tablero.rs` (sinónimos en `INDICADORES`); agregar un indicador = una fila allí + que exista la propiedad en `municipios.geojson`.
