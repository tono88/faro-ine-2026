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
transacción, tipografía servida localmente, `cargo audit`.

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

## Pendiente, en orden
1. (Hecho en v0.15.1–v0.15.2; el equipo debe volver a subir sus archivos por la pantalla para que
   su base tome las reglas nuevas.)
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
7. De la v0.16.0: medir el clasificador con fotos de campo del INE; bajar los pesos del sitio oficial
   de Places365 y comparar la huella; publicar con HTTPS para el trabajo de campo; decidir el umbral de
   celdas; firma electrónica de constancias; GHS-BUILT-C (10 m) para altura y tipo de edificio.
