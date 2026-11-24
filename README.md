# FARO · Datatón INE 2026

Categoría 2: Tecnológica y ciencias de datos.
Equipo: Estuardo Sandoval · Ana Echeverría · Alejandro Sandoval.

FARO es una herramienta para que el INE **pida, reciba y homologue** registros administrativos
de otras instituciones de forma estándar, y los deje listos para integrarse con censos y encuestas.

**Qué se ha hecho, en una página:** [descripción](docs/00-que-se-hizo.md) · **Lo más importante que encontró con datos reales:** [logros y hallazgos](docs/09-logros.md) · **De dónde sale cada dato:** [fuentes y citas](docs/11-fuentes-y-citas.md) · **Cómo fluye el trabajo:** [proceso](docs/12-proceso.md).

## El problema

Las instituciones que envían datos al INE no tienen un formato común. El mismo municipio llega
escrito como "Puerto de San José", "El puerto", "PTO. SAN JOSE" o "San José" (que además existe
en Escuintla y en Petén). Cada envío se limpia a mano y lo aprendido se pierde.

## Para quién

- **Coordinación del SEN y oficinas sectoriales (OCSE):** piden y reciben registros de RENAP, PNC,
  SE-CONRED, ministerios y municipalidades con una solicitud estándar, y ven el avance.
- **Registros administrativos (DESA):** validan cada envío con una herramienta de licencia libre
  y dejan de corregir a mano lo mismo cada mes.
- **Censos y Encuestas / Cartografía:** catálogo de lugares poblados verificado, lugares sin
  camino en el mapa, y asignación de operadores cerca de su casa.
- **Precios e Índices de Referencia:** cálculo documentado del precio de referencia.
- **Instituciones que envían:** saben exactamente qué columnas se les piden y reciben un reporte
  claro de lo que hay que corregir.

## Qué hace hoy

1. **Solicitud de datos.** El INE declara una vez qué columnas necesita, de qué tipo, cuáles son
   obligatorias y cuáles sirven para el pareo (`datos/solicitudes/renap-defunciones-v1.toml`).
2. **Recepción.** Cada envío se valida contra la solicitud: reconoce columnas aunque vengan con
   otro nombre ("DEPTO.", "Muni"), y reporta fila por fila fechas inválidas, vacíos y valores
   fuera de catálogo.
3. **Homologación.** Los nombres de lugar se llevan al código oficial del INE. El sistema solo
   asigna cuando no hay duda; lo demás lo propone con candidatos y lo decide una persona.
4. **Memoria.** Cada decisión queda grabada (qué texto, qué código, quién, cuándo, de qué
   institución). El siguiente envío ya no pregunta lo mismo.
5. **Fuentes con procedencia.** Descubre y descarga conjuntos del portal del INE y deja
   registrada la URL, la fecha y la huella de cada archivo (`faro fuentes`).
6. **Datos personales protegidos.** Solo salen las columnas pedidas; nombres se eliminan y
   documentos de identidad se sustituyen por una llave no reversible.
7. **Probado con archivos reales** de RENAP, PNC, SE-CONRED e INSIVUMEH compartidos por el INE:
   51,719 filas en menos de 3 segundos, entre 97.9 % y 100 % de lugares con código oficial.
   Ver [hallazgos](docs/06-hallazgos-datos-reales.md).
8. **Más que lugares.** Los valores de catálogos cerrados (sexo, pueblo, tipo de vehículo y los
   que declare cada solicitud) que no se reconocen también se revisan en pantalla y se aprenden
   por variable, para todas las instituciones. Cada solicitud puede declarar qué valores
   significan "sin dato" (98, 99, 999) y una columna de respaldo (el nombre, cuando el código
   propio de la institución no existe).
9. **Correcciones que ya no hay que hacer a mano.** Letras dañadas por la codificación
   ("Due±as" es "Dueñas"), códigos propios de una institución que se traducen una vez, y nombres
   que existen en otro departamento ("Camotán" enviado en El Progreso): se proponen, no se asignan.
10. **Población y tasas.** Proyecciones de población del INE por municipio, 2015–2030, como
    denominador: los resultados salen también por 100 mil habitantes, y la pantalla muestra el
    crecimiento poblacional de los 340 municipios (`faro poblacion`).
11. **Avance de la recepción.** Qué se pidió a cada institución, qué ha llegado, cuándo y con qué
    calidad.
12. **Catálogo oficial verificado.** Departamentos, municipios y 20,036 lugares poblados del
    Censo 2018 con coordenadas, comprobados código por código contra el listado del INE
    (`faro catalogo verificar`).
13. **Operadores de campo cerca de su casa.** Reparte operadores entre puestos con la menor
    distancia total posible, usando las coordenadas del catálogo (`faro asignar` y la pantalla).
    Cálculo exacto (método húngaro); distancia en línea recta, no por carretera.
14. **Red vial como grafo.** Cruza los lugares poblados del censo con los caminos de OpenStreetMap:
    cuáles quedan en un tramo desconectado de la red principal, cuáles no tienen camino mapeado
    cerca y cuál es el enlace más corto que probablemente falta (`faro vial` y la pantalla).
    Resultado real: 707 lugares por revisar y 83 enlaces probables. Ver [logros](docs/09-logros.md).
15. **Materiales de construcción contra el IPMC.** Toma los 89 artículos del Índice de Precios de
    Materiales de Construcción del INE como catálogo oficial y homologa contra él los productos
    que publican los proveedores: comparable, por revisar o de otra medida, con el precio llevado
    a la unidad oficial. Incluye un levantamiento real de 116 proveedores y 2,332 precios
    (`faro ipmc`, `faro construccion` y la pantalla).
16. **Mapa y capas publicables.** Un mapa por municipio con población, crecimiento, superficie
    construida, conexión vial y registros recibidos; cada capa tiene una dirección estable en
    GeoJSON o CSV para un portal o un SIG (`/capas`).
17. **Antecedentes de precios del INE.** Búsqueda en los 4,990 precios de referencia que el INE ya
    emitió y publica en su portal (`faro referencias` y la pantalla).
18. **Todo se puede repetir y auditar.** El cálculo de población se rehace desde la pantalla contra
    el portal del INE, mostrando cada paso; la sección "Proceso" enseña qué hace cada etapa y quién.
19. **Muestreos automáticos.** Tareas configurables que el portal repite solo: leen las páginas
    públicas de tiendas en línea y guardan cada precio con su dirección y fecha. Seis lecturas:
    búsqueda por términos, catálogo completo por categorías y subcategorías (con código de barras
    y precio de lista), mapa del sitio con datos estructurados, listado paginado y los catálogos
    públicos de tiendas Shopify y WooCommerce. Se encienden y se les cambia
    la frecuencia desde la pantalla (`datos/estandar/muestreos.toml`, 26 tareas).
19a. **Precios por rubro.** Supermercado, ferretería, tecnología, hogar, farmacia y vehículos:
    85,907 precios publicados por 18 tiendas (levantamiento del 24-nov-2026, en
    `datos/referencia/precios_publicados.csv`). Búsqueda con media, mediana y moda; el **mismo
    producto entre tiendas por su código de barras** (16,291 productos de supermercado), alertas de
    **cambios atípicos** entre muestreos, tiendas informantes y descarga en CSV.
19b. **Mapa en tres niveles, con vista aislada.** Departamentos (cifras sumadas de sus municipios,
    tasas recalculadas), municipios y lugares poblados del censo (aldeas, caseríos, cantones). Se
    puede ver solo un departamento o solo un municipio (lo demás se atenúa y las clases se
    recalculan) y volver a todo el país con un botón. La pantalla se adapta a teléfonos.
19c. **Pistas para códigos que no existen.** Cuando llega un código de lugar inexistente, el portal
    dice qué más podría ser según el diccionario de estadísticas vitales del INE (un país, una
    edad, «Extranjero», «Ignorado») para detectar columnas corridas; no decide con eso.
19d. **Cobertura por departamento.** En cuántos municipios de cada departamento ya hay registros
    de cada solicitud: seguimiento de la recopilación por delegación.
19e. **Informe de precio de referencia.** De una búsqueda en los precios publicados sale un informe
    imprimible con media, mediana y moda, el precio recomendado según la regla de la Dirección de
    Precios, las fuentes con su enlace y la metodología; la persona aparta lo que no es comparable
    y el informe se recalcula (`/informe-precio`).
19f. **Hoja de Balance de Alimentos: derivados.** Calculadora con las tasas de extracción de
    Guatemala publicadas por la FAO (35 derivados de 19 productos), contrastadas con las cuentas de
    la HBA 2024 del INE: con sus propias cifras, el trigo da 72.0 % de harina (FAO: 72) y la caña
    10.2 % de azúcar (FAO: 10). Admite cargar otra tabla (`faro hba`, `balance.rs`).
19h. **Censo 2018 por lugar poblado.** Población por sexo y edad, área urbana y rural, hogares y
    viviendas de los 20,036 lugares poblados (cuadros A1, B8 y C1 del INE; `faro censo-lugares`):
    en el globo de cada punto, en la ficha del municipio, en el buscador y como capa publicada.
19i. **Edad y sexo por municipio.** Pirámide de población de cada municipio con las proyecciones
    del INE (2018, 2026 y 2030; `faro edades`), verificada contra los totales ya publicados.
19j. **Comparación por foto.** En el informe de precio se ve la foto de cada producto y se puede
    dejar solo los que se ven como uno de referencia o comparten su código de barras. La huella
    visual se calcula en el propio servidor y sus umbrales están calibrados con datos reales
    ([`docs/13-fotos.md`](docs/13-fotos.md)): no confunde productos distintos, pero reconoce poco
    cuando cada tienda tomó su propia foto.
19k. **Diferencias de precio explicadas.** Cuando el mismo código de barras cuesta 50 % o más en
    una tienda que en otra, el portal aplica reglas: de 537 casos en supermercados, 411 son ofertas
    (el precio regular quita la diferencia), 7 señalan una tienda con precio atípico y 119 no
    tienen regla y esperan la confirmación de una persona («es correcto» / «es un error»). Lo
    marcado como error deja de contar en comparaciones e informes.
19l. **Entregables.** Documento técnico en PDF (`entregables/FARO-documento-tecnico.pdf`, se
    regenera con `scripts/entregables/generar_pdf.js`), ficha técnica, manual de usuario,
    instalación y dependencias, guion del video y guion de la presentación (`docs/14` a `docs/18`).
19g. **Buscador de territorio.** Sobre el mapa: departamento, municipio o lugar poblado por nombre,
    nombre común o código; lleva a su ficha o a su punto.
20. **Canasta básica.** Precio promedio del INE de 34 productos, con su serie, junto a los
    precios publicados que recoge el muestreo (`faro canasta`).
21. **Ficha y reporte por municipio.** Al tocar un municipio: cifras, gráficas, animación del
    crecimiento construido, estadísticos de la confianza y un reporte para imprimir (`/reporte`).
22. **Cada etapa se puede repetir fuera del portal.** Junto a cada resultado se descarga el
    procedimiento (Bash o PowerShell) que lo reproduce con los mismos datos (`scripts/reproducir/`).
23. **Usuarios y roles.** Se entra con usuario y contraseña. Administrador (todo, incluido manejar
    usuarios), analista (recibe, decide, configura) y consulta (solo ve). Cada decisión queda a
    nombre de quien inició sesión.
24. **Datos de otras instituciones, citados.** Inseguridad alimentaria aguda por departamento
    (CIF, SESAN), leída del informe oficial, verificada y cruzada con la población del INE (`faro cif`),
    y desnutrición crónica en escolares por municipio (Censo de Talla 2024, SESAN; `faro talla`).
25. **Precios de referencia.** A partir de cotizaciones calcula media, mediana y moda por producto
    y elige como lo explica la Dirección de Precios e Índices de Referencia (mediana si hay
    precios extremos, media si son parecidos); avisa si faltan cotizaciones directas o se mezclan
    unidades (`faro precios` y la pantalla).
26. **Crecimiento urbano real.** Superficie construida por municipio de 1975 a 2020 con la capa
    satelital abierta GHSL de la Comisión Europea, sin Earth Engine; ritmo de crecimiento y
    proyección con su margen (`faro construido`, `faro crecimiento`).

## Levantarlo con Docker

```bash
cp .env.example .env          # cambie las tres claves
docker compose up --build     # base de datos PostgreSQL + FARO
```

Abra <http://localhost:8082> y entre con el usuario inicial `admin` y la contraseña `admin`; la
pantalla le pedirá cambiarla en "Mi perfil". Los demás usuarios se crean en "Usuarios". Si ese puerto está ocupado, cambie `FARO_PUERTO` en `.env`
(`iniciar.bat` elige solo uno libre y muestra la dirección).

La pantalla abre en un mapa por municipio y tiene veintiuna secciones agrupadas por área (incluye el flujo de proceso y las fuentes con sus citas): recibir un envío, revisar pendientes, equivalencias
confirmadas, resultados y avance por institución, conjuntos del portal del INE (se sincroniza
sola al arrancar y cada 24 horas), crecimiento poblacional y urbano, asignación de operadores de
campo, red vial, materiales de construcción, precios de referencia y capas publicadas. Cada sección enlaza un archivo de ejemplo (ficticio) para probarla.

En Linux, el contenedor escribe las descargas de fuentes abiertas en `datos/crudo` con un usuario
sin privilegios; antes del primer `faro fuentes ...` dentro del contenedor ejecute
`chmod o+w datos/crudo`. En Windows y macOS (Docker Desktop) no hace falta.

## Probarlo sin Docker

Requiere [Rust](https://rustup.rs) estable (probado con 1.97) y, para el servidor, PostgreSQL 16.

```bash
cargo test                       # 69 pruebas
cargo build --release

# Línea de comandos (no necesita base de datos)
./target/release/faro recibir --solicitud datos/ejemplos/solicitud_hechos_transito.toml \
  --entrada datos/ejemplos/envio_hechos_transito.csv --salida salida
./target/release/faro confirmar "El puerto" --codigo 0509 --contexto 05 --institucion Ejemplo --usuario analista
./target/release/faro fuentes inventario                       # conjuntos del INE (requiere internet)
./target/release/faro fuentes descargar censo-2018-lugares-poblados --formato XLSX
./target/release/faro catalogo verificar --listado <listado de lugares poblados.xlsx>   # catálogo contra el Censo 2018
./target/release/faro crecimiento --entrada datos/ejemplos/edificaciones_ficticias.csv --proyectar 2030
./target/release/faro aire --entrada <archivo de calidad del aire.xlsx>
./target/release/faro vial --carreteras <hotosm_gtm_roads_osm_shp.zip>   # red vial contra lugares poblados
./target/release/faro precios --cotizaciones datos/ejemplos/cotizaciones_ficticias.csv
./target/release/faro poblacion --entrada <proyecciones.xlsx> --diccionario <diccionario.xlsx>   # tabla municipio-año
./target/release/faro asignar --operadores datos/ejemplos/operadores_ficticios.csv --puestos datos/ejemplos/puestos_ficticios.csv

# Servidor
DATABASE_URL="host=localhost user=faro password=... dbname=faro" FARO_TOKEN=... FARO_SAL=... ./target/release/faro-servidor
```

Si una solicitud sustituye identificadores por llaves, defina el secreto `FARO_SAL`.

## Estados de homologación

| Estado | Significado | ¿Se asigna código? |
|---|---|---|
| `exacto` | Coincide con un único nombre oficial (sin importar tildes, mayúsculas o abreviaturas) | Sí |
| `memoria` | Equivalencia ya confirmada por una persona | Sí |
| `sugerido` | Un candidato muy parecido y sin competencia | No, falta confirmar |
| `ambiguo` | Varios lugares con ese nombre y falta el nivel superior | No |
| `revisar` | Hay candidatos, ninguno claro | No |
| `sin_coincidencia` | Nada parecido en el catálogo | No |
| `fuera_de_contexto` | El nombre existe, pero en otro departamento o municipio | No, se muestra dónde existe |
| `codigo_invalido` | Llegó un código que no existe | No; se busca en la lista y queda aprendido |
| `fuera_de_catalogo` | Valor no previsto de una variable (sexo, pueblo...) | No; se elige el valor permitido y queda aprendido |

## Datos

| Fuente | Uso | Estado |
|---|---|---|
| INE, Censo 2018, "Listado de lugares poblados" ([datos.ine.gob.gt](https://datos.ine.gob.gt/es/dataset/censo-2018-lugares-poblados), CC BY) | Catálogo oficial de lugares con código y coordenadas: fuente principal | Cargado: 20,036 lugares poblados, 19,723 con coordenadas (`faro catalogo importar`) |
| `datos/catalogo/` | 22 departamentos, 340 municipios y 20,036 lugares poblados con código INE y alias | Incluido; verificado código por código contra el listado oficial ([detalle](datos/catalogo/LEEME.md)) |
| INE, "Estimaciones y proyecciones de la población total por municipio, 2015–2030" ([datos.ine.gob.gt](https://datos.ine.gob.gt/es/dataset/proyecciones-de-poblacion-2015-2030), ODC-BY), descargado el 24-nov-2026 | Población por municipio y año: denominador de tasas y crecimiento poblacional | Incluido resumido en `datos/referencia/poblacion_municipio.csv` ([detalle](datos/referencia/LEEME.md)) |
| © Colaboradores de OpenStreetMap, exportación de carreteras de HOT en [HDX](https://data.humdata.org/dataset/hotosm_gtm_roads) (ODbL 1.0), corte del 3-nov-2026, descargado el 24-nov-2026 | Red vial como grafo | Resultados derivados en `datos/referencia/red_vial_*` (ODbL); el archivo original no se versiona |
| CONRED / OCHA, límites administrativos de Guatemala ([HDX: cod-ab-gtm](https://data.humdata.org/dataset/cod-ab-gtm), CC BY-IGO), descargado el 24-nov-2026 | Figuras de los municipios para los mapas y para sumar por municipio | Capa simplificada y verificada en `datos/referencia/municipios.geojson` |
| Comisión Europea, JRC, GHSL GHS-BUILT-S R2023A (datos abiertos con atribución), descargado el 24-nov-2026 | Superficie construida 1975–2020 para crecimiento urbano | Resumida por municipio en `datos/referencia/superficie_construida_municipio.csv` |
| INE, Índice de Precios de Materiales de Construcción ([datos.ine.gob.gt](https://datos.ine.gob.gt/es/dataset/indice-de-precios-de-materiales-de-construccion-ipmc)), descargado el 24-nov-2026 | Catálogo oficial de 89 artículos, con índice y variación | `datos/catalogo_precios/ipmc_articulos.csv` |
| Páginas públicas de 116 negocios de materiales de construcción, consultadas el 24-nov-2026 | Perfil de proveedores y precios publicados | `datos/referencia/proveedores_construccion.csv` y `precios_construccion.csv`, con la dirección de cada página |
| INE, Precios de referencia ([datos.ine.gob.gt](https://datos.ine.gob.gt/es/dataset/precios-de-referencia)), descargado el 24-nov-2026 | Antecedentes: 4,990 precios emitidos de 2016 a 2023 | `datos/catalogo_precios/precios_referencia_ine.csv` |
| SESAN y Grupo Técnico de la CIF Guatemala, análisis de inseguridad alimentaria aguda, octubre 2026 – febrero 2027 ([informe](https://portal.sesan.gob.gt/wp-content/uploads/2026/11/CIF_Guatemala-ct2026-Feb2027.pdf)), descargado el 24-nov-2026 | Personas por fase de la CIF en cada departamento | `datos/referencia/cif_departamentos.csv` |
| SESAN, SIINSAN, "Retardo en talla, Censo 2024" ([libro](https://portal.siinsan.gob.gt/wp-content/uploads/VULNERABILIDAD_DC_MUNICIPAL_QUINTO_CENSO_2024_2.xlsx)), descargado el 24-nov-2026 | Desnutrición crónica en escolares por municipio | `datos/referencia/censo_talla_municipio.csv` |
| `datos/solicitudes/` | Solicitudes de datos para RENAP (defunciones, nacimientos), PNC (detenidos) y SE-CONRED (emergencias) | Incluido |
| Archivos de ejemplo de instituciones compartidos por el INE (23-nov-2026) | Diseño y prueba de las solicitudes | **No están en el repositorio** (contienen datos personales) |
| Google Research, Open Buildings 2.5D Temporal (CC BY 4.0 / ODbL) | Edificaciones por año 2016–2023 | Opcional: script listo (`scripts/`); la serie que se usa es GHSL ([opciones](docs/10-imagenes-satelitales.md)) |
| `datos/estandar/` | Núcleo común de variables y mapa de registros e instituciones (borrador del equipo) | Incluido |
| `datos/ejemplos/` | Envío, solicitud, serie de edificaciones, operadores y puestos **ficticios** para demostración ([detalle](datos/ejemplos/LEEME.md)) | Incluido |

El repositorio no contiene datos de personas. Los ejemplos son inventados. Cada descarga queda en
`datos/crudo/manifiesto.jsonl` con URL, fecha, licencia y huella SHA-256 (`faro fuentes citar`).

## Estructura

```
crates/faro-homologa/   biblioteca: lectura, solicitud, homologar, memoria, recepción
crates/faro-fuentes/    cliente CKAN y descargas con procedencia
crates/faro-territorio/ población, red vial, asignación de operadores, precios, edificaciones, aire
crates/faro-servidor/   servidor web, base de datos y pantalla
crates/faro-cli/        ejecutable `faro`
scripts/                script de Earth Engine para edificaciones por municipio
datos/catalogo/         catálogos oficiales en CSV (codigo,nombre,nivel,padre)
datos/estandar/         núcleo común de variables y mapa de registros
datos/referencia/       tablas derivadas de fuentes oficiales (población, red vial)
datos/solicitudes/      qué pide el INE a cada institución
datos/ejemplos/         archivos ficticios de demostración
docs/                   necesidades del INE, investigación, arquitectura y plan
```

## Documentación

- [Necesidades planteadas por el INE](docs/01-necesidades-ine.md)
- [Investigación y referencias](docs/02-investigacion.md)
- [Arquitectura y relación con X-Road](docs/03-arquitectura.md)
- [Plan hasta la entrega](docs/04-plan.md)
- [Cómo se conectan todas las instituciones](docs/05-mapa-institucional.md)
- [Qué pasó al recibir los archivos reales](docs/06-hallazgos-datos-reales.md)
- [Modelos de código abierto que tendría sentido entrenar](docs/07-modelos.md)
- [Versiones](docs/08-versiones.md)
- [Logros y hallazgos](docs/09-logros.md)
- [Crecimiento urbano con imágenes satelitales abiertas](docs/10-imagenes-satelitales.md)
- [Fuentes y citas](docs/11-fuentes-y-citas.md)
- [Flujo de proceso](docs/12-proceso.md)
- [Qué se ha hecho](docs/00-que-se-hizo.md)

## Limitaciones y mejoras

- **Los umbrales son parámetros, no normas del INE.** Cuándo un nombre es "muy parecido", cuándo
  un precio es "extremo", qué distancia cuenta como "tener camino": los valores por defecto son del
  equipo y deben fijarlos las unidades responsables.
- **Distancias en línea recta.** La asignación de operadores y la red vial no usan tiempo de viaje.
  El siguiente paso es calcular rutas sobre el mismo grafo vial.
- **La red vial mide el mapa.** Un hueco puede ser un camino sin dibujar o un paso en lancha.
- **Crecimiento urbano a 100 m y cada cinco años.** GHSL mide superficie construida, no
  viviendas; para detalle de edificios haría falta Open Buildings (140 GB por año).
- **Muestreo de precios solo en catálogos VTEX.** Otras tecnologías de tienda necesitan su propio
  lector. La canasta básica del INE no indica unidades, así que no se compara sola.
- **Precios de construcción: pocas fuentes y solo en línea.** 16 proveedores con precios, casi
  todos cadenas de la capital; las reglas del IPMC cubren 42 de 89 artículos y son un borrador.
- **Límites municipales operativos.** La capa de mapas no sirve para definir linderos.
- **Precios sin históricos de Guatecompras.** El cálculo usa las cotizaciones que se le den.
- **Usuarios sin institución asignada.** Hay tres roles, pero todavía no se limita a cada usuario a
  los envíos de su institución. El usuario inicial `admin` / `admin` debe cambiarse al instalar.
- **Texto libre sin codificar.** Causa de muerte (CIE-10), delito y ocupación quedan como texto.
- **Intercambio entre sistemas.** Hoy se suben archivos; el diseño prevé recibir lo mismo por
  X-Road ([arquitectura](docs/03-arquitectura.md)).
- `iniciar.bat` no se ha probado en todas las versiones de Windows.

## Componentes de terceros

Bibliotecas de Rust de código abierto (licencias MIT o Apache-2.0): `serde`, `serde_json`, `toml`,
`csv`, `strsim`, `unicode-normalization`, `chrono`, `clap`, `ureq`, `sha2`, `calamine`, `zip`, `regex`,
`weezl`, `argon2`, `axum`, `tokio`, `tokio-postgres`. Mapas: Leaflet 1.9.4 (BSD de 2 cláusulas), incluido en
`crates/faro-servidor/web/vendor/`; la tipografía Noto Sans se carga de Google Fonts si hay internet. Base de datos: PostgreSQL 16 (licencia PostgreSQL). Imágenes de Docker:
`rust`, `debian` y `postgres` oficiales. Versiones exactas en `Cargo.lock`. No se copió código de
proyectos previos; los algoritmos (homologación, método húngaro, grafo vial, lector de Shapefile)
se escribieron durante el evento.

## Servicios en la nube

FARO no depende de ningún servicio de pago ni en la nube: corre completo en una computadora con
Docker. Consulta por internet solo fuentes abiertas (portal del INE, HDX). El desarrollo usó
GitHub y un asistente de IA en la nube (ver abajo).

## Uso de inteligencia artificial

El código y la documentación se desarrollaron con ayuda de un asistente de IA (Claude, de
Anthropic), bajo dirección y revisión del equipo. Los commits generados con el asistente lo
indican en su mensaje.
