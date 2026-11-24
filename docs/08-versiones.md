# Versiones

Cada versión es un release de GitHub sobre el historial de commits.

## v0.16.3 · 24-nov-2026
- Sin sesión ya no aparecen en el menú las secciones que piden entrar («Recibir envío», «Revisar
  pendientes», «Equivalencias», «Operadores de campo»), y los paneles para cargar archivos y los
  botones que cambian algo solo se muestran a quien tiene permiso en esa sección; en su lugar
  queda una nota. Cargar una tabla propia en la hoja de balance ahora pide sesión.
- README e instalación: usuario y contraseña iniciales (`admin` / `admin`) a la vista.

## v0.16.2 · 24-nov-2026
- Red vial: tipo y superficie de cada vía. El dato ya venía en el archivo de OpenStreetMap y solo
  se usaba para armar el grafo. Ahora `faro vial` suma por municipio los kilómetros de vías
  principales, secundarias, locales, de terracería y senderos, y cuántos tienen la superficie
  anotada (`red_vial_tipos_municipio.csv`), y prepara la capa de carreteras
  (`red_vial_principal.geojson`). En el portal: capa «Carreteras» en el mapa, cuatro indicadores
  nuevos y tablas por tipo, por superficie y por municipio en «Red vial».
- README: «Los problemas que atiende», uno por necesidad planteada por el INE.

## v0.16.1 · 24-nov-2026
- Identidad del proyecto: logotipo (faro con su luz), lema «Que cada dato llegue a puerto», misión,
  visión, objetivos, valores, colores y voz (`docs/25-identidad.md`, `docs/identidad/`). En el
  portal: logotipo en el encabezado y en la pestaña, y bloque de identidad en «Acerca y fuentes».
  Equipo: La Sociedad de Almas.

## v0.16.0 · 24-nov-2026
- **Diseñador de tableros** rehecho: lienzo con paleta de visualizaciones y de datos, panel de
  propiedades, deshacer y rehacer, arrastrar para ordenar, anchos, y dos piezas nuevas (dispersión
  con su correlación, y texto). **Los filtros se aplican al tablero ya creado**: territorio en
  cascada y condiciones sobre indicadores, también con palabras («solo Quiché»). Un tablero nace
  como **borrador** y se **publica**: lo publicado se ve sin cuenta, con su enlace (`/#tablero=N`).
- **Trabajo de campo**: rol «operador» y grupo «Operadores de campo»; rutas por operador con las
  paradas ordenadas; registro de visita desde el teléfono con la posición que entrega el
  dispositivo; enlace a **Microsoft Forms** con lugar, posición y número de visita ya escritos;
  cruce de cada visita con satélite y censo; visitas en CSV.
- **Clasificador de fotos** (vivienda, comercio u otro): Places365-ResNet18 corriendo en el
  servidor, sin nube; la foto no se guarda. Prueba con 16 fotos: 11 aciertos, 0 errores, 5 no
  concluyentes (`faro fachadas`, `docs/24`).
- **Censo 2018**: quince indicadores de condiciones de vida por municipio y material del techo por
  lugar poblado (`faro censo-indicadores`), en el mapa, la ficha y los tableros.
- **Uso del suelo**: superficie residencial y no residencial por municipio y por kilómetro con el
  componente NRES del GHSL (`faro uso-suelo`).
- **Animación del crecimiento urbano** sobre foto satelital (Landsat de NASA y Sentinel-2 de EOX);
  corre una vez y tiene casilla para repetir. Capa satelital opcional en el mapa.
- **Proyecciones**: tasa constante junto a la recta para la superficie construida; para la
  población más allá de 2030, curva exponencial a corto plazo y logística a largo plazo, probadas
  contra el propio INE (`docs/19`).
- **Protección estadística**: los conteos menores que `FARO_UMBRAL_CELDA` (3) no se publican a
  quien no es analista, con supresión complementaria.
- **Bitácora** de acciones y **análisis de conformidad legal** (`docs/23`).
- **Antes y ahora**: 23 puntos en el portal y en `docs/22`, de una sola fuente (`faro mejoras`).
- Pendientes y envíos dicen si se analizaron con reglas anteriores a las vigentes.
- Correcciones: los menús por grupo no ocultaban bien los botones; 45 comentarios del código
  describían un comportamiento anterior o eran confusos.
- README reescrito; `faro citas` regenera el documento de fuentes desde `fuentes.json`.

## v0.15.2 · 24-nov-2026
- Historial de envíos con reemplazo real: cada archivo queda en el historial con su fecha; en las
  estadísticas cuenta el más reciente de cada **corte** (mes de registro que declara la solicitud o,
  si no declara, mes del hecho). Un envío reemplazado deja de mostrar sus pendientes. El mismo
  archivo de la misma solicitud siempre reemplaza al anterior. Botón para eliminar un envío.
- Corrección: los archivos mensuales de RENAP traen hechos de meses anteriores inscritos tarde; con
  la regla anterior (mes del hecho) el archivo de un mes habría pisado meses de otro. Ahora el
  corte es el mes de registro (`corte` en la solicitud).
- La ficha del municipio ya no cuenta dos veces un reenvío.

## v0.15.1 · 24-nov-2026
- Resuelta la correspondencia de los códigos de RENAP: son los del INE en dos columnas y solo valen
  cuando el país es Guatemala. La solicitud declara la columna de país; los lugares de otro país ya
  no quedan pendientes. Con los archivos reales: nacimientos y defunciones al 100 %.
- Corrección: declarar `-1` como «sin dato» anulaba también el `1`; ahora los números se comparan con su signo.

## v0.15.0 · 24-nov-2026
- Verificación estadística (`faro verificar` → `docs/19`): consistencia de todas las tablas; prueba
  hacia atrás de la proyección de superficie construida en 1,020 casos, que mostró que el margen
  anterior solo acertaba 45 % de las veces: ahora el margen es el error realmente observado y cubre
  80 % y 95 %; intervalo de confianza de la mediana en el informe de precios; intervalos de Wilson
  para la comparación por foto; sensibilidad del umbral de diferencias de precio.
- Verificación de interfaz (`docs/20`): 22 secciones en laptop de 15.6" (HD y Full HD), tableta
  (vertical y horizontal) y celular, con clic y toque real en el mapa; se corrigieron textos
  pequeños en celular y botones bajos en tableta.
- Seguridad (`docs/21`): auditoría completa; 16 de 20 hallazgos corregidos (contraseña inicial
  obligatoria de cambiar, borrado garantizado del archivo temporal, freno de intentos, contraseñas
  fuertes, lecturas sensibles con sesión, cabeceras, rechazo de peticiones de otro origen, topes de
  tamaño y de consultas pesadas, portal atendido solo en la propia máquina por omisión).
- Grupos de usuarios con acceso por secciones.
- Tableros a pedido: de una frase a un tablero, con editor, guardado e impresión.
- Manual, guiones y documento técnico reescritos.

## v0.14.0 · 24-nov-2026
- Precios: diferencias de 50 % o más entre tiendas explicadas por regla (oferta, tienda atípica)
  o confirmadas por una persona; lo marcado como error sale de comparaciones, búsquedas e informes.
- Menú de rubros bajo "Precios por rubro"; botón "Ejecutar todas ahora" en los muestreos.
- Pendientes: ya no se sugieren países ni edades para códigos de municipio; solo lo que el
  diccionario del INE define para esa columna («Extranjero», «Ignorado»).
- Entregables: documento técnico en PDF con capturas, ficha técnica, manual de usuario,
  instalación y dependencias, guion del video y guion de la presentación.

## v0.13.0 · 24-nov-2026
- Censo 2018 por lugar poblado (población, sexo, grandes grupos de edad, área, hogares y viviendas
  de 20,036 lugares): en el mapa, la ficha, el buscador y como capa (`faro censo-lugares`).
- Pirámide de edad y sexo por municipio con las proyecciones del INE (`faro edades`).
- Comparación por foto en el informe de precio de referencia: miniaturas, huella visual propia
  (forma y color) y umbrales calibrados con productos de igual código de barras (`faro fotos`).
  El levantamiento de precios se rehízo guardando la dirección de cada foto (85,907 precios).

## v0.12.1 · 24-nov-2026
- Corrección: el clic sobre un municipio (y, en ventanas bajas, sobre departamentos y lugares) no
  abría la ficha, porque al tomar el foco el mapa la página se desplazaba. Ahora hay una prueba
  de clic real en cuatro tamaños de ventana.
- La calculadora de derivados usa datos reales: tasas de extracción de Guatemala de la FAO,
  contrastadas con las cuentas de la Hoja de Balance de Alimentos 2024 del INE (`faro hba`).

## v0.12.0 · 24-nov-2026
Tres peticiones de las presentaciones del lanzamiento:
- Dirección de Precios e Índices de Referencia: informe de precio de referencia imprimible a partir
  de los precios publicados (media, mediana, moda, regla de elección, fuentes y metodología), con
  la opción de apartar lo que no es comparable.
- Hoja de Balance de Alimentos: calculadora de productos derivados con utilización y tasas de
  extracción, y avisos de consistencia.
- Coordinación Regional: buscador de departamento, municipio y lugar poblado sobre el mapa
  (también por nombres de uso común y por código).

## v0.11.0 · 24-nov-2026
- Catálogos a tope: se recorren las subcategorías (sin el límite de 2,500 por consulta) y todas
  las categorías de los supermercados. Lecturas nuevas para tiendas Shopify y WooCommerce.
  Tiendas nuevas: Farmacias Batres, Intelaf, Click, iShop, Macrosistemas, Kalea y Ferco (26 tareas).
- Levantamiento ampliado: 85,907 precios de 18 tiendas; 16,291 productos de supermercado con el
  mismo código de barras en dos o más tiendas. El servidor lo recarga solo si la tabla cambia.
- Mapa: botón para volver a todo el país; vista aislada de un departamento o de un municipio;
  las capas encendidas a mano quedan siempre encima. "Ver en el mapa" (red vial) enciende la capa
  de enlaces y abre la explicación del enlace elegido.
- Canasta básica: el histórico de cada producto se amplía (gráfica con ejes, máximo, mínimo,
  variaciones y tabla de valores).
- Pendientes: pista de qué podría ser un código inexistente (catálogo de países y edades del
  diccionario de estadísticas vitales del INE).
- Resultados: cobertura de la recepción por departamento (22 delegaciones).

## v0.10.0 · 24-nov-2026
- Muestreos: tres lecturas nuevas (catálogo por categorías con código de barras y precio de lista,
  mapa del sitio con datos estructurados, listado paginado) y 16 tareas más: supermercados
  (Walmart, Paiz, Maxi Despensa, La Torre), tecnología (Elektra, Siman, Cemaco, Walmart), farmacia
  (Cruz Verde, Meykos, Walmart), ferretería (EPA, Cemaco, Construfácil), hogar y vehículos.
  Los catálogos corren en segundo plano.
- Sección "Precios por rubro": 26,976 precios de 11 tiendas; búsqueda con media, mediana y moda;
  mismo producto entre tiendas por código de barras; cambios atípicos entre muestreos; tiendas
  informantes; descarga CSV.
- Mapa en tres niveles: departamentos, municipios y lugares poblados; ficha de departamento.
- Al pasar sobre un enlace probable se explica con qué se calculó.
- Vistas para teléfono (menú, ficha como hoja inferior, tablas y controles táctiles), probadas a 390 px.

## v0.9.0 · 24-nov-2026
- Inicio de sesión con usuarios y roles (administrador, analista, consulta), perfil y manejo de
  usuarios; la bitácora registra a quien inició sesión. Usuario inicial `admin` / `admin`.
- Inseguridad alimentaria aguda por departamento (CIF, SESAN), verificada y citada, en una
  sección propia, en el mapa y en la ficha.
- Desnutrición crónica en escolares por municipio (Censo de Talla 2024, SESAN).
- La ficha del municipio cabe completa sobre el mapa.

## v0.8.0 · 24-nov-2026
- Muestreos automáticos configurables de precios publicados (tiendas con catálogo VTEX); los de
  construcción entran a la comparación contra el IPMC.
- Rubro de alimentos: canasta básica del INE (34 productos) junto a precios publicados.
- Ficha flotante de municipio con mini tablero, animación del crecimiento construido y
  estadísticos de la confianza; reporte municipal para imprimir.
- Capas nuevas: caminos alrededor de cada enlace, huella construida en bloques de 1 km,
  departamentos y lugares poblados.
- Códigos numéricos que no existen: se proponen sus lecturas posibles; carga de la tabla de
  códigos de una institución.
- Archivo de ejemplo en cada carga, unidades en las tablas, algoritmo visible y procedimiento
  descargable (Bash y PowerShell) en cada etapa. Se retira la institución de ejemplo.

## v0.7.0 · 24-nov-2026
- El cálculo de población se puede repetir desde la pantalla, paso a paso, contra el portal del INE.
- Flujo de proceso por etapas y responsables, con cifras en vivo.
- Precios: 4,990 precios de referencia del INE consultables; reglas del IPMC ampliadas a 42 artículos.
- Fuentes y citas en la pantalla y en la documentación; descripción de lo hecho.

## v0.6.0 · 24-nov-2026
- Mapa de municipios con indicadores y capas; publicación de capas (GeoJSON y CSV) con catálogo.
- Pantalla rediseñada con la paleta y la tipografía del INE, menú por áreas y versión para móvil.
- Capa de límites municipales verificada contra el censo (`faro limites`).
- Crecimiento urbano real con GHSL, sin Earth Engine (`faro construido`).
- Materiales de construcción: 116 proveedores, 2,332 precios publicados, homologación de productos
  contra el IPMC del INE con memoria y comparación por artículo (`faro ipmc`, `faro construccion`).

## v0.5.0 · 24-nov-2026
- Red vial como grafo con caminos reales de OpenStreetMap: lugares poblados en tramos sueltos,
  sin camino mapeado y enlaces probables (`faro vial`, pestaña "Red vial").
- Documento de logros y hallazgos.

## v0.4.0 · 24-nov-2026
- Precios de referencia a partir de cotizaciones (media, mediana, moda; precios extremos;
  proporción de cotizaciones directas), en línea de comandos y en la pantalla.

## v0.3.0 · 24-nov-2026
- Población por municipio 2015–2030 (proyecciones del INE) y tasas por 100 mil habitantes.
- Hallazgo: 16 códigos de municipio inexistentes en el libro de proyecciones, reubicados con su diccionario.
- Equivalencias de valores de catálogo (sexo, pueblo, tipo de vehículo...) con memoria por variable.
- Correcciones: letras dañadas, códigos propios de una institución, lugares de otro departamento,
  columna de respaldo, valores "sin dato".
- Pantalla: lista del catálogo para elegir el lugar, avance de recepción por institución,
  crecimiento poblacional, archivos de ejemplo descargables.
- Puerto por defecto 8082.

## v0.2.0 · 23-nov-2026
- Despliegue con Docker Compose probado; `iniciar.bat` busca un puerto libre.
- Catálogo verificado contra el Censo 2018: 340 municipios código por código y 20,036 lugares
  poblados con coordenadas.
- Asignación de operadores de campo con distancia total mínima (método húngaro).

## v0.1.0 · 23-nov-2026
- Núcleo de homologación con memoria, solicitudes de datos, recepción de Excel/CSV reales con
  datos personales eliminados o seudonimizados.
- Cliente del portal de datos abiertos del INE con procedencia.
- Servidor con PostgreSQL y pantalla web; crecimiento de edificaciones y calidad del aire.
