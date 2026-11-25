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
  `docker run -d --name pgtmp -e POSTGRES_PASSWORD=<clave> -e POSTGRES_USER=faro -e POSTGRES_DB=faro -p 55432:5432 postgres:16` y
  `DATABASE_URL="host=127.0.0.1 port=55432 user=faro password=<clave> dbname=faro" FARO_TOKEN=<clave de servicio> FARO_SAL=<secreto> FARO_CLAVE_INICIAL=<inicial> FARO_PUERTO=8099 FARO_SYNC_HORAS=0 ./target/debug/faro-servidor` (valores de una base desechable; ninguno se escribe en el repositorio).
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

## Bloques v0.15.1 y v0.15.2 (24-nov)
- RENAP: códigos = INE en dos columnas, válidos solo con país 320. Se descubrió contando combinaciones en el archivo real (máximo de municipio por departamento = catálogo).
- `es_sin_dato("-1")` anulaba "1" porque `basico` quita el signo: comparar números con signo.
- Los archivos mensuales traen hechos de décadas (registro tardío): la unidad de reemplazo es el corte (mes de registro), no el periodo del hecho.
- Prueba de punta a punta de envíos con archivos reales: servidor nativo + `pgtmp`, subir con `curl -F archivo=@... -H "x-faro-token: $FARO_TOKEN"`, mirar `/api/envios`, y al final `DELETE FROM envios` (la base temporal no debe quedarse con nada).
- Los archivos reales están en `/root/.claude/uploads/...` solo durante la sesión; NUNCA copiarlos al repositorio.

## Bloque v0.16.0 (24-nov)
- Tableros: el usuario reportó que «los filtros no se aplican al tablero ya creado». Causa de fondo: el territorio vivía en el editor y con nivel municipio el alcance era el departamento, así que cambiarlo casi no se notaba. Ahora la barra de filtros es del tablero (`pintarFiltros`), vale en la vista publicada sin guardar, y el título sigue al territorio (`ponerTerritorio`).
- Deshacer: un `change` que no cambia nada no debe ocupar un paso (`tabCambiar` compara antes y después). Playwright dispara `change` dos veces (el explícito y el del desenfoque).
- `nav button { display:flex }` anulaba el atributo `hidden`: los menús por grupo no ocultaban de verdad. Las pruebas miraban `.hidden`, no la pantalla. Regla global `[hidden]` y pruebas con `getComputedStyle`.
- El HTML va incrustado en el binario: tras tocar `index.html` hay que recompilar antes de probar.
- Fotos satelitales: EOX `s2cloudless_3857` (2016) es CC BY; los años 2018+ son no comerciales; GIBS Landsat WELD solo tiene 1983–85, 1988–90 y 1998–2000, hasta nivel 12. Esri World Imagery NO se puede usar (licencia). El navegador de pruebas necesita `proxy` para verlas.
- Landsat no se usa directamente: las cifras vienen del GHSL. Decirlo siempre así.
- GHSL NRES: solo 1.6 % de lo construido es no residencial; sirve para cruzar visitas, no para un mapa vistoso.
- Censo: cada cuadro trae hoja `_2` (municipio) y `_3` (lugar poblado); las columnas se comprueban por palabra del encabezado. Sipacate (0514) y Petatán (1333) traen 0 en 2015.
- Logística: el techo se busca en una rejilla (linealizando); si el mejor techo es el mayor probado, no hay techo. Contra el INE a 5 años la logística (0.17 %) gana a la exponencial (1.05 %); aun así la regla pedida es exponencial corto / logística largo.
- Places365: el sitio oficial no responde desde la nube; pesos desde un espejo verificado (ver PROCEDENCIA). tract 0.23: `to_plain_array_view`. En depuración ~0.8 s por foto.
- Clasificador: decidir por suma de grupos, con «otro» si juntos < 25 % y «no concluyente» si la parte < 65 %; así en 16 fotos no hay ningún error. «Otro» y «no concluyente» NO son duda.
- Geolocalización: Playwright `geolocation` + `permissions`; `localhost` cuenta como contexto seguro. En producción hace falta HTTPS.
- Agentes en paralelo que sirvieron: marco legal con citas verificadas, fuentes de imagen satelital, exportación del modelo, revisión de comentarios (45 hallazgos, varios eran comentarios pegados a la función equivocada).
- La revisión de comentarios destapó un fallo real: el ingreso no calculaba Argon2 si el usuario no existía (docs/21 decía lo contrario). Corregido con huella de relleno.
- v0.17.0. Recorridos: motor propio (foco con `box-shadow` gigante + globo `position:fixed`); los pasos cuyo elemento no está a la vista se saltan; `const RECORRIDOS` es accesible desde `page.evaluate`, y de ahí salen las marcas del manual.
- Manual gráfico: páginas de tamaño fijo (carta apaisada, `@page margin:0`); una clase de página no puede llamarse igual que una rejilla interior (`caminos` rompió el diseño). Capturas JPEG a escala 1.5: 11 MB.
- Cuentas de institución: el extractor `Json` responde 422 antes de autorizar; para probar un 403 hay que mandar un cuerpo válido.
- v0.18.0. Auditar con agentes independientes rinde: hallaron una resta que deshacía la supresión complementaria (dos rutas con reglas distintas), dos licencias mal declaradas y documentos del día 1 que seguían en el PDF técnico. Regla: una sola función decide qué celdas se ocultan.
- Clon limpio: `git clone --depth 1` + `.env.example` + Dockerfile de nube → imagen en 252 s. Tras borrar `target/` hubo que liberar la caché de construcción de Docker (27 GB).
- `faro citas` exige que el `tipo` de cada fuente pertenezca a un grupo; `fuentes.json` va con sangría de 1 espacio.
- v0.20.0. Idiomas sin tocar el código de la pantalla: traducir el DOM con un diccionario cuya clave es el texto en español (números → `#`, etiquetas → marcas numeradas que se REUTILIZAN, no se recrean: así no se pierden ids ni manejadores). `MutationObserver` + `takeRecords()` para no reaccionar a los cambios propios. La traducción se inserta como nodos de texto (nunca `innerHTML`).
- Separar interfaz de dato: cosechar en marcha lo que se ve y aceptar solo lo que está escrito en el código de la pantalla o en los textos del servidor. Las cadenas del código se cosechan con el mismo guion (`recogerHtml`), con `${…}` como hueco → molde `{0}`. Un molde con poco texto fijo se descarta (podría calzar con un dato).
- Carrera real: una tabla que termina de cargar después de limpiar la cosecha se colaba como «texto del código» (167 productos). Limpiar, cosechar y leer en un solo `evaluate`.
- Kaqchikel: la única fuente abierta alineable es la localización de Mozilla (`firefox-l10n/cak`, mismos identificadores que `es-MX` y `en-US`): 3,470 cadenas; 32 coinciden palabra por palabra con rótulos de FARO. MediaWiki no tiene `cak.json`; Common Voice y AMO no tienen `cak`. No componer frases.
- Agentes para traducir: seis trozos de ~18 mil caracteres, salida como objeto id → traducción (no lista: evita desalinearse), con validación de marcas, `#` y `{n}`. Unificar ortografía después (salió británica y estadounidense mezclada).
- Una palabra con dos sentidos («media») no se puede traducir por diccionario sin contexto: `data-ctx`.
- Encabezado en teléfono de 360 px: el selector queda solo con el globo (≤ 400 px).

