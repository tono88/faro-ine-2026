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

Agregado después (v0.11.0): `vtex-catalogo` baja a subcategorías; lecturas `shopify` y `woocommerce`;
`max_consultas` por tarea; 26 tareas; levantamiento de 85,907 precios (18 tiendas), que el servidor
recarga si el CSV cambia. Mapa: `aislar()`, `verTodo()`, paneles `encima` y `puntos`, botón `#volverpais`.
Canasta: `graficaGrande()`. Pendientes: `pista_de_codigo()` con `datos/estandar/paises_ine.csv`
(diccionario de estadísticas vitales del INE). Resultados: `cobertura()` por departamento.
Sobre los códigos de RENAP (RESUELTO en v0.15.1): son los del INE partidos en dos columnas (departamento
1–22 y municipio numerado dentro del departamento) y solo valen si la columna de país es 320; con otro
país son divisiones de ese país. La solicitud lo declara con `pais = [...]` y `sin_dato = ["-1", "0"]`. Sin catálogo legible: Farma Value,
Galeno, Suma, Super del Barrio; bloqueados: Kemik, Platino, Fayco, Agencias Way, MEM. Por integrar
(verificados): Max y Tecnofácil (`__NEXT_DATA__`, traen EAN), Steren (GraphQL), Pacifiko, Office Depot,
El Gallo más Gallo, Celasa.

Agregado después (v0.12.0), de las presentaciones: `/informe-precio` + `/api/informe_precio` (informe
imprimible desde precios publicados, parámetro `sin` = direcciones apartadas por la persona);
`crates/faro-territorio/src/balance.rs` + `POST /api/balance` + sección "Hoja de balance: derivados"
(ejemplo ficticio en `datos/ejemplos/tasas_extraccion_ficticias.csv`; las tasas reales las da el INE);
`/api/buscar_lugar` y buscador sobre el mapa (también por alias). Estado de cada propuesta de las
presentaciones: tabla al final de `docs/01-necesidades-ine.md`. No hechos: catálogo del Manual de
Características del IPC (no es público), ICC, edades y comparación entre censos.

Agregado después (v0.12.1): arreglo del clic en el mapa (`lienzo.focus` con `preventScroll`; prueba
`.faro/pruebas/clic_real_mapa.js`, que usa clic REAL: correrla siempre que se toque el mapa). HBA con
datos reales: `tasas_extraccion_fao_guatemala.csv` (FAO TCF) + `faro hba` → `hba_2024_cuentas.csv`;
`GET /api/balance`. Regla reafirmada por el equipo: nada inventado en el portal; lo sintético solo como
archivo de muestra que el usuario decide cargar.

Agregado después (v0.13.0): `faro censo-lugares` → `censo2018_lugar_poblado.csv`; `faro edades` →
`poblacion_municipio_edades.csv`; `piramide()` y bloque de censo en la ficha; fotos:
`crates/faro-fuentes/src/foto.rs` (huella de forma + color, crate `image`), columna `imagen`, tabla
`huellas_foto`, parámetro `como` del informe, `faro fotos` (calibración; resultado en `docs/13-fotos.md`:
0 falsos, 9.7 % de aciertos con fotos distintas). El servidor necesita internet para bajar fotos.
Sin fuente descargable: censos 2002/1994 por municipio (PDF escaneado; microdatos tras Radware).

Agregado después (v0.14.0): `precios::explicar_diferencia` (oferta / atípico / por confirmar),
tabla `decisiones_precio`, `POST /api/publicados/decision`, pestaña "Diferencias de 50 % o más";
`POST /api/muestreos/ejecutar_todas`; menú `#menurubros`; pistas de pendientes reducidas a
«Extranjero»/«Ignorado» (decisión del equipo: NO sugerir países ni edades para municipios; se quitó
`paises_ine.csv`). Entregables: `entregables/FARO-documento-tecnico.pdf` (generar con
`scripts/entregables/generar_pdf.js <url>`; regenerarlo al cerrar versión), `docs/14`–`docs/18`.

Agregado después (v0.15.0): `faro verificar` → `docs/19` (márgenes de proyección = error observado,
constantes `ERROR_OBSERVADO_P80/P95` en `crecimiento.rs`; `estadistica.rs`: `ic_mediana`, `wilson`, `t_975`);
`.faro/pruebas/verificacion_interfaz.js` → `docs/20`; seguridad (`docs/21`): middleware `seguridad()`,
`con_sesion()`, `autorizar_en(seccion)`, `INTENTOS`/`ARGON`/`PESADAS`, `clave_aceptable(clave, usuario)`,
contraseña inicial que BLOQUEA los cambios hasta cambiarla, `FARO_ESCUCHA` (127.0.0.1 por omisión) y
`FARO_HTTPS`. Grupos (`grupos`, `usuarios.grupo_id`, `/api/grupos`). Tableros (`tablero.rs`, tabla
`tableros`, `/api/tableros*`, sección "Tableros"). Los guiones de prueba entran con
`.faro/pruebas/_sesion.js` (cambia admin/admin por `Faro-Prueba-2026` en bases nuevas).
Pendiente de seguridad: grupo de conexiones a la base, HMAC en la llave de pareo, envío en una
transacción, `cargo audit` (la tipografía ya se sirve localmente desde la v0.21.0).

Agregado después (v0.15.1): campo `pais`/`pais_local` de la columna (`recepcion.rs`: `en_el_extranjero`);
`es_sin_dato` compara los números con su signo (antes "-1" anulaba el "1"). Archivos reales vueltos a
pasar: RENAP 100 %, PNC 100 %, CONRED 99.9 % (`docs/06`). El pendiente 1 de esta lista queda hecho.

Agregado después (v0.15.2): `corte` en la solicitud (columnas de registro; RENAP lo declara) y columna
`hechos.corte`; vista `envios_vigentes` (último envío por solicitud y corte); los pendientes y las
estadísticas solo miran envíos vigentes; mismo archivo + misma solicitud = reemplazo; `DELETE
/api/envios/:id`. Los originales NO se guardan (datos personales): "reanalizar" = volver a subir.

Agregado después (v0.16.0), todo probado con servidor nativo + Chromium y en contenedor:
- **Tableros**: diseñador nuevo en `index.html` (bloque «Tableros: galería, diseñador y vista publicada»:
  `tabCambiar`, `tabDeshacer`, `pintarEstudio`, `pintarFiltros`, `pintarProps`, `abrirDeEnlace`); `tablero.rs`
  con `Filtro`, `Reconocido`, piezas `dispersion` y `texto`, `ancho`; `tableros.publico` (borrador por omisión),
  `POST /api/tableros/:id/publicar`, enlace `/#tablero=N`. Prueba: `.faro/pruebas/disenador_tableros.js`.
- **Trabajo de campo**: `campo.rs` (`ordenar_ruta`, `formulario_admitido`, `enlace_formulario`, `cruzar`),
  tablas `rutas_campo`, `paradas_campo`, `visitas_campo`, rol `operador`, grupo «Operadores de campo»,
  sección `campo` («Mi ruta de campo») y panel `#campocoord` en «Operadores de campo»; `/api/campo*`;
  `FARO_FORMULARIOS`. La foto NO se guarda. Geolocalización: exige HTTPS salvo en localhost.
  Prueba: `.faro/pruebas/trabajo_de_campo.js <url> <carpeta> [foto vivienda] [foto comercio]`.
- **Clasificador de fotos**: `crates/faro-fuentes/src/fachada.rs` (tract-onnx 0.23, Places365-ResNet18 en
  `datos/modelos/`, 45 MB, pesos CC BY; `PROCEDENCIA.md`; `scripts/modelo/exportar.py`), `faro fachadas`,
  `datos/referencia/fachadas_prueba.csv`, `docs/24`. Reglas: `MINIMO_ENTRE_LOS_DOS` 0.25, `PARTE_MINIMA` 0.65.
  Las 16 fotos de prueba NO están en el repositorio (licencias de Commons).
- **Censo**: `faro censo-indicadores` (12 cuadros, 51 cifras, 15 indicadores `censo_*_pct`) y techos por lugar;
  `faro uso-suelo` (GHSL NRES → `uso_construido_municipio.csv`, `uso_construido_bloques.csv`).
- **Proyecciones**: `crecimiento.rs`: `proyeccion_tasa_constante`, `extrapolar` (exponencial ≤ 10 años,
  logística después; `ajustar_logistica`), `GET /api/poblacion?extender=AAAA`. Pedido expreso del equipo:
  exponencial a corto plazo y logística a largo plazo; siempre rotuladas como no oficiales.
- **Animación**: fotos de NASA GIBS (Landsat WELD 1985/1990/1998/2000) y EOX s2cloudless 2016 en `detalle()`;
  corre una vez; `#animbucle`. Capa `#capasat`. Prueba: `.faro/pruebas/animacion_con_foto.js` (usa el proxy).
- **Protección estadística**: `umbral_de_celda()` (`FARO_UMBRAL_CELDA`, 3), `ve_conteos_exactos`, supresión
  complementaria en `api_registros_municipio`. **Bitácora**: tabla `bitacora`, `bitacora()`, `/api/bitacora`.
- **Comparativo**: `datos/estandar/mejoras.json` → sección `mejoras` y `docs/22` (`faro mejoras`). Regla del
  equipo: **cada cosa que se gane se agrega ahí** (antes / con FARO + cifra / cómo escala) y al README.
- `faro citas` regenera `docs/11` desde `fuentes.json`; `docs/23` (conformidad legal; no es dictamen).
- `[hidden] { display:none !important }` es global: antes los menús por grupo no ocultaban bien.
- Envíos y pendientes: `envios.reglas` (huella de la solicitud) y marca «reglas anteriores».
- En esta nube: `dockerd` y la base temporal `pgtmp` se caen entre turnos; `scratchpad/reiniciar.sh`
  recompila y levanta el servidor nativo en 8095. La construcción de Docker con tract tarda más.

Agregado después (v0.16.1): identidad. Equipo «La Sociedad de Almas»; lema «Que cada dato llegue a
puerto»; logotipo en `docs/identidad/*.svg` (icono, horizontal claro y oscuro) y en línea en el
encabezado de `index.html` (también como icono de pestaña); misión, visión, objetivos y valores en
`docs/25-identidad.md`, README y bloque `.identidad` de «Acerca». Ámbar `#f5a623` = color propio de
FARO. Si cambia el texto de misión/visión/objetivos, cambiarlo en los tres sitios.

Agregado después (v0.16.2): `faro vial` también escribe `red_vial_tipos_municipio.csv` (km por clase
y superficie; campos `highway`, `surface`, `adm2_pcode`, `adm2_name` del .dbf; el municipio se acepta
solo si nombre y código coinciden con el catálogo) y `red_vial_principal.geojson` (4.8 MB). Mapa:
`#capavias`; indicadores `vias_*`; sección «Red vial»: `#tvialtipos`, `#tvialsup`, `#tvialmuni`.
Superficie anotada solo en 38 % de la red: nunca presentar «% pavimentado» sin decirlo. RIC y
Dirección General de Caminos: no se halló capa abierta de vías (búsqueda del 24-nov).

Agregado después (v0.16.3): visibilidad por permiso. `puede(seccion)` en `index.html`; botones de menú con
`data-sesion` (no aparecen sin sesión; el manejador del menú no abre un botón oculto); elementos con
`data-requiere="seccion"` (paneles de carga y botones de cambio) se ocultan sin permiso y dejan una
`.nota-acceso`. Al agregar un panel de carga o un botón que cambie datos: ponerle `data-requiere`.
`POST /api/balance` pide sesión. Prueba: `.faro/pruebas/visibilidad_sin_sesion.js`.

Agregado después (v0.16.4): mapa: `soltarLienzos()` (un `L.canvas` vacío sigue recibiendo clics: se quita
al cambiar de nivel o apagar una capa); `.leaflet-encima-pane canvas { pointer-events:none }` (las capas
de lienzo de ese panel NO pueden ser interactivas; la vía bajo el puntero se nombra con
`viaBajoElPuntero`); al cargar se aplican las casillas `#capa*` marcadas. Prueba con clic real:
`.faro/pruebas/mapa_capas_y_niveles.js` (correrla junto a `clic_real_mapa.js` al tocar el mapa).
Asistentes: `asistente(cont, {...})` (lista `.pasos` + bloques `.paso[data-paso]`), usado en
`#campocoord` (5 pasos) y en el diálogo `#campovisita` (4 pasos). Demostración: `rutas_campo.demo`,
`POST/DELETE /api/campo/demo`, `simulada` solo vale en rutas demo; siempre rotulada.

Agregado después (v0.17.0): rol `institucion` + columna `usuarios.institucion` (`Sesion.institucion`);
`quien_envia(e, cabeceras, &sol)` autoriza `api_enviar` (misma institución y contraseña ya cambiada);
`personal_del_ine()` (admin/analista/consulta) en pendientes y equivalencias; `api_envios` filtra por
institución; `instituciones(&e)` = las que tienen solicitud. Pantalla: `deInstitucion()`, `#enviapor`,
`#uinst`, `select[data-inst]`. Recorridos guiados: lista `RECORRIDOS` (una entrada por sección + `portal`;
pasos `{en, sube, t, x}`), `recorrer(cual)`, `#recorrido` en el encabezado y `button.recorrer` en cada
sección. **Al agregar una sección o un panel importante: sumar su paso a `RECORRIDOS`** (el manual lo toma
de ahí). Manual gráfico: `scripts/entregables/generar_manual.js <url> <admin> <clave> [usuario institución]
[clave]` → `entregables/FARO-manual-de-usuario.pdf`; metadatos por sección en `SECCIONES` de ese guion;
en la nube se generó contra una base nueva (`faromanual`) con los archivos de ejemplo ficticios.
Prueba: `.faro/pruebas/institucion_y_recorridos.js`. Regenerar manual y PDF técnico al cerrar versión.

Agregado después (v0.17.1): vista materializada `precios_ultimos` (último precio por rubro, tienda y url) +
`asegurar_ultimos(&bd)` (la refresca si cambió `max(id)`/`count(*)` de `precios_observados`; a lo sumo cada
30 s mientras un muestreo agrega). `/api/publicados?parte=rubros|precios|codigo|diferencias|cambios|informantes`
(sin `parte` va todo). Pantalla: `rubros()` pide `precios` y luego las demás en orden (`PARTES_RUBRO`,
`rubroListas`, `rubroTurno`, `rellenoTabla`, `resumenRubro`); clases `.cargando`, `.cargando-barra`, `.relleno`
reutilizables para cualquier carga lenta. Animación: `FOTO_ENTERA` (WMS: GIBS `wms/epsg4326/best/wms.cgi`, EOX
`tiles.maps.eox.at/wms` capa `s2cloudless` = 2016, CC BY; los años posteriores son NC, no usarlos),
`pedirEntera`, `precargar`, `cuadroListo`; las teselas quedan solo como respaldo.
Prueba: `.faro/pruebas/carga_precios_y_fotos.js` (usa el proxy para las fotos).

Agregado después (v0.18.0, revisión para la entrega; sin funciones nuevas). Nota conceptual: matriz en
`docs/26` (revisarla al cerrar cada versión). **Ninguna contraseña en el código**: `FARO_CLAVE_INICIAL`
(compose la deja en `admin`; sin ella el servidor genera una al azar y la imprime), `DATABASE_URL`
obligatoria, guiones de prueba con `FARO_CLAVE_PRUEBA` (y `FARO_CLAVE_INICIAL`); `scratchpad/reiniciar.sh`
ya la pone. `FARO_ESCUCHA` también vale sin Docker (127.0.0.1 por omisión; el Dockerfile fija 0.0.0.0).
`con_sesion()` y `ve_conteos_exactos()` rechazan sesiones con contraseña inicial. `celdas_ocultas()` es la
única lista de celdas protegidas: **toda ruta nueva que entregue conteos por municipio debe usarla**.
`api_envio_borrar` borra `datos/privado/envios/<n>`. Nombre visible único. Bitácora visible en «Usuarios»
(`#tbitacora`, `bitacoraVer`). Muestreo: se detiene ante 403/429. Fuentes: 37 (`ghsl-articulo`, `osm-teselas`,
`commons-fachadas`); FAO NO es CC BY; `precios-referencia` es ODC-BY; `cba` no declara licencia.
`datos/crudo/manifiesto.jsonl` e `inventario_ine.csv` SÍ se versionan (valor por omisión de `faro fuentes`).
Entregables: `scripts/entregables/` → `terceros.js` (docs/27), `preparar_demo.js <url> <clave>` (instalación
nueva lista para demostrar, todo ficticio), `generar_presentacion.js`, `generar_manual.js`, `generar_pdf.js`.
`generar_presentacion.js` escribe PDF y PPTX del mismo contenido (lista `D`); el PPTX necesita `pptxgenjs`
(en esta nube: `NODE_PATH=<scratchpad>/npm/node_modules:$NODE_PATH`). La diapositiva de problemas se lee de la
tabla «Los problemas que atiende» del README (decisión del equipo: NO centrarla en el ejemplo de San José).
`LICENCIA.md`: autoría + licencia al INE; la licencia para terceros es decisión pendiente del equipo.
`docs/28`: guía para defender el código. `docs/16`: guion y plan de grabación del video (siguiente paso).
Prueba de clon limpio: `scratchpad/clon_limpio.sh`. Tres auditorías con agentes (citas, documentos, legal):
repetirlas antes de una entrega; sus hallazgos están resumidos en `docs/26`, sección 7.
No decidir por el equipo: firma de los commits (hoy los firma el asistente), licencia para terceros.

Agregado después (v0.19.0; sin cambios en el portal). Video: `scripts/entregables/grabar_video.js <url> <clave>
[config.json]` → `entregables/FARO-video-demo.mp4` + `.srt` + `-tiempos.json`. Lleva el guion (`GUION`: texto y
duración por escena; ~2.35 palabras por segundo). Captura con CDP `Page.startScreencast` y Chromium lanzado con
`--force-device-scale-factor=1.5 --window-size=1280,720` y `viewport: null` (así los cuadros salen a 1920×1080;
con `deviceScaleFactor` del contexto salen a 1280×720, y el zoom por CSS rompe Leaflet). Lo que se desplaza es
`<main>`, no la ventana. El cursor es un elemento `popover` que se vuelve a subir al abrir un diálogo modal.
La escena del teléfono se graba aparte y se compone con ffmpeg. El resultado de un envío trae valores por fila:
en cámara quedan bajo la banda opaca (no quitar ese cuidado). En esta nube: `scratchpad/video/toma.sh` (base
`farovideo`, servidor release en 8097, `preparar_real.js`, `config_real.json` con los archivos reales de la
sesión; nada de eso va al repositorio; borrar la base y `datos/privado/envios` al terminar).
Presentación (16 diapositivas): frentes por dirección, autocontenido/riesgo, crece sin programar, petición de
piloto, apéndice. Decir «el portal no depende de una IA externa para funcionar», NUNCA «no usamos IA» (se
programó con un asistente, declarado). En defunciones de RENAP pasan 15 de 82 columnas (no 14).
Falta: la narración de Ana Lucía (la graba el equipo) y el montaje final.

Agregado después (v0.20.0): idiomas de la pantalla. `crates/faro-servidor/web/idiomas.js` (servido en
`/idiomas.js`; lo cargan `index.html`, `reporte.html` e `informe_precio.html`) traduce al vuelo con
diccionarios que son datos: `datos/estandar/idiomas/<código>.json` (clave = el texto en español; `#` = un
número; `<1>…</1>` y `<1/>` = etiquetas; `{0}` = un dato que pasa igual; `ctx|texto` para palabras con dos
sentidos, con `data-ctx` en el elemento). `plantilla.json` (lista de textos de la pantalla) la arma
`scripts/idiomas/extraer.js <url> <admin> <clave>`; `plantilla_extra.json` trae los moldes hechos a mano;
`scripts/idiomas/comprobar.js [--completar] [--podar]` valida y dice la cobertura. `/api/idiomas` y
`/api/idiomas/:codigo` leen del disco en cada pedido. **Regla del equipo: se traduce la interfaz, nunca los
datos**; lo que es dato se marca `data-no-traducir` (nombres de personas, citas); dentro de `tbody`/`select`
no se aplican moldes. El inglés
lo tradujo el asistente (declarado en el README). **Al agregar o cambiar texto en la pantalla: correr
`extraer.js`, traducir lo nuevo en `en.json` y correr `comprobar.js --completar --podar`** (si no, `cargo
test` falla o el texto sale en español). Selector: `#idiomabtn`/`#idiomalista`; `localStorage.faro_idioma`;
`?idioma=en`; `volverASeccion()`. Prueba: `.faro/pruebas/idiomas.js`; `FARO_IDIOMA=en` en
`verificacion_interfaz.js`. Documento: `docs/29-idiomas-e-inclusion.md` (también el marco de la Ley de
Idiomas Nacionales, Decreto 19-2003). El video y la presentación llevan una mención final de inclusión.
Los documentos (`docs/`), el manual y la presentación siguen solo en español.

Agregado después (v0.20.1): kaqchikel ampliado por pedido expreso del equipo («un porcentaje considerable,
dejando dicho que se completaría con asesoría de un experto»). 680 rótulos (34.1 %): 110 de nivel `fuente`
(idénticos a la fuente) y 570 `compuesto` (armados con palabras de las fuentes; gramática SIN revisar).
Fuentes: memoria de traducción de Mozilla en Pontoon (`https://pontoon.mozilla.org/translation-memory/cak.all-projects.tmx`,
59,421 frases; MPL 2.0) y *Kaqchikel Choltzij* de la ALMG-CLK (2011). Archivos: `cak.json`, `cak_revision.csv`
(rótulo, nivel, base, columnas para quien revise) y `cak_palabras.csv` (cada palabra con su fuente); ya no
existe `cak_glosario.csv`. `cargo test` y `comprobar.js` exigen que cada rótulo esté en la hoja de revisión y
cada palabra en la lista. **Reglas vigentes:** `cak` sigue en estado `borrador` hasta que conste
`revisado_por`; no agregar una palabra kaqchikel que no esté en una fuente; no traducir nombres propios
(«Guatemala» se queda); decir siempre «borrador sin revisar, a completar con asesoría experta», NUNCA «el
portal está traducido al kaqchikel». Cómo se rehace: paquetes de evidencia por rótulo → redacción con
agentes → validación de palabras (guiones en el scratchpad de la sesión: `kaq_herr/paquetes.py`,
`armar_cak.py`; no están en el repositorio). Encabezado en teléfono: a 400 px o menos se oculta la palabra
«FARO» (queda el icono) y el botón de sesión se recorta con puntos suspensivos.

Agregado después (v0.21.0): identidad según la *Guía de diseño gráfico para Datatón* que el INE envió a los
equipos (25-nov). **Paleta**: azul radiante `#0080ff`, azul oscuro `#004080`, blanco; tintes `#66b3ff`,
`#b3d9ff`, `#e5f2ff`; **amarillo `#ffd500`** en lugar del ámbar `#f5a623` (decisión del equipo). **Tipografía**:
Ubuntu Sans (UFL 1.0) en `crates/faro-servidor/web/vendor/ubuntu-sans/` (woff2 reducidos; rutas
`/vendor/ubuntu-sans/ubuntu-sans-<peso>.woff2`); ya no se usa Google Fonts y la CSP no lo admite. Los
generadores la incrustan con `scripts/entregables/_tipografia.js` (`CSS`, `ine(nombre)`). **Logotipo del INE**
(esto reemplaza el «Sin logotipo del INE» de la v0.6.0): solo en el cintillo inferior, esquina inferior
derecha, recuadro con la esquina superior izquierda curva: `.pie-ine` al pie del menú, `.ine` en presentación
y manual, portada del documento técnico, `.vz-ine` en las tarjetas del video; siempre con «Proyecto
participante del Datatón INE 2026» cerca. Archivos del INE en `docs/identidad/ine/` (no modificarlos; al
incrustar varios SVG en una misma página usar `<img>`, porque comparten nombres de clase). El PowerPoint usa
Ubuntu Sans como tipografía del tema: quien lo edite debe tenerla instalada.

Agregado después (v0.21.0, segunda parte): carta de presentación del equipo en `docs/identidad/equipo/`
(`sociedad-de-almas.svg` = emblema ORIGINAL dibujado para el proyecto: placa pentagonal con vetas y tres
llamas. El equipo pidió conservar el guiño a la placa de la foto que envió, que es de una obra con derechos:
se conserva la forma pentagonal y las vetas, NUNCA la calavera ni otro elemento copiado; `qr-*.svg` con segno; `carta-de-presentacion.svg`).
`scripts/entregables/pie_equipo.js` pone la carta al final del README y de cada `docs/NN-*.md` tras la marca
`<!-- equipo -->` (**volver a correrlo después de regenerar docs 11, 20, 22 y 27**); `_tipografia.js`:
`carta()`, `sinPie()`. La carta va también en la contraportada del manual, la última página del documento
técnico, la diapositiva de cierre (HTML y PPTX, `cartaPng`) y la tarjeta `cierre` del video (`CARTA`).
Sin fotos de los integrantes mientras el equipo no las envíe (no tomarlas de LinkedIn).
`docs/30-preguntas-del-jurado.md` → `entregables/FARO-preguntas-y-respuestas.pdf` con
`scripts/entregables/generar_preguntas.js` (no necesita el portal; `### N.N pregunta`, párrafos
`**Seguimiento: …**`). README: sección «Conformidad con el reglamento del Datatón y con la ley» (resumen de
`docs/26` y `docs/23`; si cambia una cifra allá, cambiarla aquí) y bloque «Cómo generar las tres claves»
(también en `docs/15`). `generar_pdf.js` exporta `aHtml` y `esc`.

Agregado después (v0.21.1): menú según quién mira. `delIne()` en `index.html` (admin/analista/consulta);
botones de menú con `data-ine="1"` (`mejoras`, `proceso`, `resultados`, `muestreos`) solo para el personal del
INE cuyo grupo los incluya; los `data-sesion` tampoco los ve el rol `operador`; `#tabcrear` (panel para armar
tableros) solo con `puede('tableros')`. **Al agregar una sección interna: ponerle `data-ine` o `data-sesion`.**
Es regla de pantalla, no de servidor. `generar_pdf.js <url> <usuario> <clave>` entra para sus capturas.
Prueba: `.faro/pruebas/visibilidad_sin_sesion.js` (sin sesión, admin, institución, operador).

Agregado después (v0.22.0): **«Guía para evaluadores»** = la sección `mejoras` renombrada, primer botón del
menú (`.guia-eval`, rojo `#c8102e`), visible para todos (ya no lleva `data-ine`); tabla `.guia-criterios` al
inicio; `FARO_GUIA_EVALUACION=0` la oculta (`guia_de_evaluacion()` → `/api/sesion.guia_evaluacion`).
**Comparar territorios**: sección `comparar` (pública; acompaña a `mapa` en los grupos), `comparar()`,
`compPintar()`, `COMP_COLOR`, usa `INDICADORES` y `resumenDepartamento`. **Certificación**: permiso/sección
`certificar` en `SECCIONES`, grupo «Acceso a la información», tabla `certificaciones`, `POST /api/certificaciones`
(`autorizar_en(.., "certificar")`), `GET /api/certificaciones/:folio`; en `reporte.html`: `prepararCertificacion`,
`#certificar`, `#certform`, `#constancia` (marca de agua «PRUEBA» por `data-marca`, «SIN FIRMA»), `#correo` (mailto).
**Reglas:** la constancia SIEMPRE es borrador sin firma; el nombre y correo del solicitante NO van al servidor.
Los PDF de la resolución que envió el equipo como modelo traen datos personales: no van al repositorio.
**Fuentes nuevas**: `faro vehiculos` (SAT, 327 MB por líneas, Latin-1, `|`; 337 municipios; 3 nombres sin asignar),
`faro pib` (Banguat), `faro gestion-municipal` (SEGEPLAN) → `datos/referencia/{parque_vehicular_municipio,
pib_departamento,gestion_municipal}.csv`; se leen en `armar_capas` (`tabla()`); 37 indicadores. SAT y Banguat
dicen «derechos reservados»: se publican sumas, nunca el original. Bloqueados: SENACYT, UNIS; Mintrab solo PDF.
Encabezado: `.marca .para` («for» + `/vendor/ine-horizontal.svg`), pedido expreso del equipo.
`00-ENTREGABLES/` = SOLO los tres entregables de la nota conceptual (repositorio, presentación PDF, video; decisión del equipo: los demás son material de apoyo y no van ahí ni en la lista inicial del README); copia numerada de `entregables/` (`scripts/entregables/copiar_entregables.js`; **correrlo
después de regenerar cualquier entregable**). Portal en línea del equipo: https://faro.test-tecnodyne.site/.
Idiomas: al extraer, la plantilla cambia según la base usada; se conserva lo anterior y se suma lo nuevo.
Prueba: `.faro/pruebas/comparar_y_certificar.js`.

## Pendiente, en orden
1. (Hecho en v0.15.1–v0.15.2; el equipo debe volver a subir sus archivos por la pantalla para que
   su base tome las reglas nuevas.)
2. Históricos de Guatecompras (OCDS) para precios de referencia: el sitio responde con un desafío
   de Cloudflare a clientes automáticos desde la nube; probar desde la máquina del equipo.
3. Ampliar las reglas del IPMC a más artículos y sumar proveedores con precios (ferreterías del
   interior: sus precios no están en línea; haría falta cotización directa).
4. (Hecho por el equipo: hay etiquetas v0.1.0 a v0.17.1 en el remoto.) Faltan los releases de v0.18.0, v0.19.0 y v0.20.0.
5. Más fuentes de gobierno ya verificadas (descarga directa, con código de municipio), por integrar
   en este orden: CONRED incidentes 2024 y MAGA afectaciones a cultivos 2024 (portal del INE,
   conjunto de8af067…); SEGEPLAN ranking de gestión municipal 2025 (JSON); MSPAS inventario de
   sistemas de agua 2024; MINEDUC establecimientos 2025; PNC hechos de tránsito 2025; INAB
   PROBOSQUE 2024; SAT parque vehicular (solo nombres de municipio: buen caso de homologación).
   Bloqueados desde la nube (Cloudflare): MINFIN, datos.gob.gt, SIGSA/MSPAS, MEM, MIDES, CONAP.
6. Entregables: presentación, manual, documento técnico y clip del video, hechos (se regeneran al
   cerrar versión). Falta la **narración** del video (la graba el equipo; guion en `docs/16`; no hay
   sintetizador de voz en esta nube), invitar al jurado al repositorio y la nota de consentimiento
   (lista en `docs/26`, sección 8).
7. De la v0.16.0: medir el clasificador con fotos de campo del INE; bajar los pesos del sitio oficial
   de Places365 y comparar la huella; publicar con HTTPS para el trabajo de campo; decidir el umbral de
   celdas; firma electrónica de constancias; GHS-BUILT-C (10 m) para altura y tipo de edificio.
8. Idiomas: completar y revisar el kaqchikel con personas hablantes (ALMG); revisión profesional del
   inglés; otros idiomas nacionales; auditoría de accesibilidad (WCAG) con lectores de pantalla.
