# Procedimientos para repetir cada etapa

Cada archivo repite una etapa fuera de la pantalla, con los mismos datos de origen y el mismo programa (`faro`), y produce las mismas tablas. Hay una versión para Bash (`.sh`) y otra para PowerShell (`.ps1`). También se descargan desde la pantalla, junto a cada resultado.

| Procedimiento | Qué hace |
|---|---|
| `catalogo` | Catálogo oficial de lugares (Censo 2018). Descarga el Listado de lugares poblados del portal del INE, verifica el catálogo código por código y carga los lugares con coordenadas. |
| `poblacion` | Población por municipio y año (proyecciones del INE). Descarga el libro de proyecciones y su diccionario, suma las edades, cruza los códigos con el catálogo y escribe la tabla municipio-año. |
| `limites` | Capa de municipios y departamentos. Descarga los límites oficiales (CONRED/OCHA), los verifica contra los lugares poblados del censo y escribe las capas para los mapas. |
| `red_vial` | Red vial como grafo. Descarga los caminos de OpenStreetMap (exportación de HOT en HDX) y calcula lugares sin conexión, enlaces probables y los caminos alrededor de cada enlace. |
| `construido` | Superficie construida por municipio (GHSL). Descarga los 30 mosaicos de GHSL que cubren Guatemala (10 épocas, 3 mosaicos) y suma la superficie construida por municipio. Requiere haber generado antes la capa de municipios (limites). |
| `construccion` | Materiales de construcción contra el IPMC. Descarga el IPMC del INE, arma el catálogo de artículos y homologa contra él los precios publicados por los proveedores. |
| `precios_ine` | Precios de referencia y canasta básica del INE. Descarga la base de precios de referencia y los precios promedio de la canasta básica del portal del INE y los deja en tablas limpias. |
| `registros` | Recibir y homologar un envío. Valida un archivo de una institución contra su solicitud, homologa lugares y valores y escribe la salida estándar, los pendientes y el reporte. Cambie la solicitud y el archivo por los suyos. |
| `operadores` | Asignación de operadores de campo. Reparte operadores entre puestos con la menor distancia total. Cambie los dos archivos por los suyos. |
| `precios` | Precio de referencia a partir de cotizaciones. Calcula media, mediana y moda por producto y elige según la regla de la Dirección de Precios. Cambie el archivo por el suyo. |

Un código de salida 2 de `faro` significa "terminó, pero dejó cosas por revisar" (por ejemplo, pendientes); no es un error.
