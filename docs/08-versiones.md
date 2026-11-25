# Versiones

Cada versión es un release de GitHub sobre el historial de commits.

## v0.19.0 · 25-nov-2026 · Video y presentación de venta

Sin cambios en el portal.

- **Video demostrativo** (`entregables/FARO-video-demo.mp4`, 2:55, 1920×1080): un recorrido del
  portal en marcha con un caso real de principio a fin (RENAP sube 10,034 defunciones), los otros
  cuatro frentes y tres tarjetas de pilares (convergencia, autocontenido, crece sin programar), con
  rótulos, transiciones y el cursor a la vista. Sin voz: la narración se graba aparte. Lo genera
  `scripts/entregables/grabar_video.js`, que también escribe los subtítulos y la línea de tiempo.
  Se grabó con los archivos reales en una base temporal; en pantalla solo hay conteos.
- **Guion de la narradora** y plan de montaje en `docs/16`: 327 palabras, escena por escena.
- **Presentación recompuesta** (PDF y PowerPoint, 14 diapositivas y 2 de apéndice): frentes por
  dirección del INE, «Autocontenido: el riesgo que el INE no corre», «Crece sin programar», cierre
  con una petición (un piloto) y las limitaciones al apéndice. Guion en `docs/17`.
- **Corrección de una cifra**: en defunciones de RENAP hoy pasan **15** columnas de 82, no 14 (la
  solicitud creció a 16; el nombre se elimina y el documento se cambia por una llave).

## v0.18.0 · 25-nov-2026 · Revisión para la entrega

Sin funciones nuevas: una revisión completa contra la nota conceptual del evento, con tres
auditorías independientes (fuentes y licencias; documentos contra código; seguridad, secreto
estadístico y legalidad). Detalle en [`26-conformidad-nota-conceptual.md`](26-conformidad-nota-conceptual.md).

- **Entregables que faltaban.** Presentación en PDF para el jurado
  (`entregables/FARO-presentacion.pdf`, 13 diapositivas, `generar_presentacion.js`); `LICENCIA.md`
  (autoría del equipo y licencia perpetua al INE, como fijan los términos); lista de componentes de
  terceros con su licencia (`docs/27`, generada de `Cargo.lock` con `terceros.js`); matriz de
  conformidad (`docs/26`); guía para explicar y defender el código (`docs/28`); guion y plan de
  grabación del video (`docs/16`); guion que deja una instalación nueva lista para demostrar
  (`preparar_demo.js`).
- **Presentación también en PowerPoint, editable** (`entregables/FARO-presentacion.pptx`), del mismo
  contenido que el PDF. La diapositiva de problemas ya no gira en torno a un solo ejemplo: lista los
  diez problemas de la tabla del README, con quién planteó cada uno.
- **Ninguna contraseña en el código.** La inicial del administrador la da la instalación
  (`FARO_CLAVE_INICIAL`; con Docker sigue siendo `admin`; si falta, se genera al azar); `DATABASE_URL`
  ya no tiene valor por omisión; los guiones de prueba leen su clave del entorno.
- **Con la contraseña inicial no se lee nada interno** (antes solo se impedían los cambios). El
  servidor sin Docker atiende por omisión solo en la propia máquina (`FARO_ESCUCHA`).
- **Secreto estadístico.** La ficha del municipio oculta ahora las mismas celdas que el mapa
  (`celdas_ocultas`): antes se podía despejar una celda protegida con el total del departamento.
  Una celda protegida ya no se muestra como cero ni como «no llegó nada».
- **Rótulos.** «Registros por 100 mil habitantes» se describe como lo que es (no es una tasa anual);
  las curvas después de 2030 dicen «no oficial» en la tabla; el informe de precio dice que no es un
  documento oficial del INE; el reporte describe el margen de proyección real.
- **Fuentes.** Dos licencias del INE corregidas (precios de referencia: ODC-BY; canasta básica: no
  declara), FAO corregida (términos no comerciales, no CC BY), cita de referencia de GHSL, atribución
  enlazada de OpenStreetMap, capas de límites rotuladas «FARO, a partir de…». 37 fuentes. El
  manifiesto de descargas y el inventario del portal del INE se versionan.
- **Otras correcciones.** Al eliminar un envío se borra su salida fila por fila; el nombre visible
  no puede ser el de otra cuenta; el CSV de visitas neutraliza fórmulas; las equivalencias de
  valores quedan en la bitácora, que ahora se consulta en «Usuarios»; un muestreo se detiene si la
  tienda lo rechaza (403 o 429); `iniciar.bat` ya no descarga un archivo que nadie leía.
- **Documentos puestos al día**: arquitectura (`03`), plan y cómo terminó (`04`), modelos (`07`),
  necesidades, investigación, mapa institucional, hallazgos, logros, imágenes satelitales, manual,
  instalación, presentación, ficha técnica, seguridad y conformidad legal.
- **Instalación comprobada desde un clon limpio** (imagen en 4 minutos; portal arriba con 85,907
  precios). 94 pruebas de Rust (una nueva: una celda protegida no se puede despejar por resta) y la
  batería de pantalla completa, con dos comprobaciones nuevas (contraseña inicial y nombre único).

## v0.17.1 · 25-nov-2026
- **Precios por rubro, carga rápida y a la vista.** El precio más reciente de cada producto se
  guarda ya calculado (vista `precios_ultimos`, que el servidor refresca cuando cambian las
  observaciones) en vez de recalcularse varias veces en cada visita. La pantalla pide primero los
  precios y después, una por una, las vistas pesadas (mismo código, diferencias, cambios,
  informantes). Medido en la base de prueba (85,907 precios): la lista aparece en 0.24 s; antes
  toda la pantalla esperaba 3 s o más. Mientras llega algo, se ve: aviso «Buscando precios…»,
  barra que avanza, filas de relleno y un indicador en cada pestaña que aún se calcula. El menú
  lateral ya no dispara la consulta completa al abrir el portal.
- **Animación del crecimiento: la foto llega entera.** Cada foto satelital se pide como una sola
  imagen del recuadro del municipio (servicio WMS de NASA GIBS y de EOX) en lugar de decenas de
  teselas que aparecían por bloques: 5 imágenes para toda la animación. Se traen todas al abrir,
  con una barra de avance, y la animación no pasa a un cuadro cuya foto no ha llegado. Si el
  servicio falla, esa foto vuelve al método anterior.
- Prueba nueva: `.faro/pruebas/carga_precios_y_fotos.js`.

## v0.17.0 · 25-nov-2026
- **Cuentas de institución.** Rol nuevo, «Institución», atado a una institución (PNC, RENAP,
  SE-CONRED…). Quien entra con esa cuenta ve todo lo público y sube **solo** los archivos de las
  solicitudes de su institución; en el historial ve solo sus envíos. No ve pendientes,
  equivalencias ni lo que enviaron las demás, y no puede borrar. El servidor lo exige en cada
  pedido (no es solo el menú): subir un archivo de otra institución responde 403. Con la
  contraseña inicial todavía no puede enviar. Pendientes y equivalencias quedan para el personal
  del INE (administrador, analista y consulta).
- **Recorridos guiados** en las 24 secciones, escritos para el portal (sin biblioteca externa): el
  botón «Recorrido guiado» está siempre en el encabezado y cada sección tiene el suyo, «Ver cómo
  funciona esta pantalla». Señala cada elemento, dice para qué sirve, se maneja con el teclado
  (flechas y Esc) y funciona en el teléfono. Desde el primer paso se abre el recorrido general.
- **Manual de usuario gráfico** (`entregables/FARO-manual-de-usuario.pdf`, 36 páginas): portada,
  qué es FARO, un camino por tipo de usuario, una página por sección con la captura marcada en los
  mismos puntos que el recorrido, la página «Su archivo, en tres pasos» para instituciones,
  preguntas frecuentes e identidad. Se regenera con `scripts/entregables/generar_manual.js`.
- Prueba nueva: `.faro/pruebas/institucion_y_recorridos.js`. Documentos puestos al día.

## v0.16.4 · 25-nov-2026
- Mapa, dos fallos corregidos: (1) al recargar, el navegador dejaba marcadas casillas de capas que
  no se dibujaban; ahora se aplican. (2) Tras pasar por «Lugares poblados», el mapa dejaba de
  responder a los clics: el lienzo de los puntos quedaba encima aunque estuviera vacío; ahora se
  retira. La capa de carreteras ya no recibe el puntero (no tapa las fichas) y la vía que está bajo
  el puntero se nombra en la leyenda. Prueba con clic real: `mapa_capas_y_niveles.js`.
- «Operadores de campo» es ahora un asistente de cinco pasos: operador, paradas, encuesta, revisar
  y crear, seguimiento. El registro de una visita también va por pasos: posición, foto, registrar,
  resultado y encuesta.
- Recorrido de demostración en «Mi ruta de campo»: seis lugares poblados reales de Sololá y dos
  visitas ficticias, rotuladas; solo ahí se admite una posición simulada.

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
