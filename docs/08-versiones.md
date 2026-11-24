# Versiones

Cada versión es un release de GitHub sobre el historial de commits.

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
