# Datos de referencia derivados de fuentes oficiales

## `poblacion_municipio.csv`

Población por municipio y año, 2015–2030 (total, hombres, mujeres): 340 municipios × 16 años.

Fuente: INE, "Estimaciones y proyecciones de la población total por municipio según sexo y edad,
2015–2030" (conjunto `proyecciones-de-poblacion-2015-2030` de <https://datos.ine.gob.gt>, licencia
ODC-BY). Descargado el 24-nov-2026 (UTC); SHA-256 del libro
`b00fa494491b5b06e2fb7dac98df2441b6670e8d9a748cc97c2e72ed5012f499` y de su diccionario `d26a68bf7a458d7a7b09b3ee72a74384f2ef9559ca5142dc55fc7e07ce59eb25`.

Cómo se obtuvo (`faro poblacion --entrada <libro> --diccionario <diccionario>`): se suman las
edades simples por municipio, año y sexo. Se comprueba que hombres + mujeres da el total publicado
en cada municipio y año. No se estima ni se ajusta ninguna cifra.

**Hallazgo.** El libro identifica 16 municipios del departamento de Guatemala con códigos que no
existen (`0123` a `0138`) en lugar de `0102` a `0117`. Su propio diccionario los nombra (por
ejemplo, `0123` = Santa Catarina Pinula). FARO lo detectó al cruzar con el catálogo, ubicó cada
uno por ese nombre (coincidencia exacta y única dentro del departamento) y dejó la tabla con el
código oficial. Quien cruce el libro original por código pierde esos 16 municipios, entre ellos
Mixco y Villa Nueva.

Uso: denominador de tasas por 100,000 habitantes y crecimiento poblacional por municipio.

## `red_vial_lugares.csv`, `red_vial_enlaces.csv`, `red_vial_resumen.json`

Resultado de cruzar los lugares poblados del Censo 2018 con la red de caminos de OpenStreetMap,
tratada como un grafo (`faro vial --carreteras <zip>`).

Fuente de los caminos: © colaboradores de OpenStreetMap, exportación "Guatemala Roads" de
Humanitarian OpenStreetMap Team en HDX (`hotosm_gtm_roads`, Shapefile), corte del 3-nov-2026,
licencia ODbL 1.0. Descargado el 24-nov-2026 (UTC); SHA-256
`cd884627eddfa037a0c49e1fd2b327ab5da611cb91d10beeee13ccfad9793977`. Estas tres tablas derivan de
esos datos y se comparten bajo la misma licencia ODbL.

Método: se usan las vías para vehículos (se excluyen senderos, vías peatonales, en construcción y
propuestas). Dos vías están unidas si comparten un punto. El componente conexo con más kilómetros
es la red principal. Para cada lugar se mide la distancia en línea recta de su centroide a la vía
más cercana.

- `red_vial_lugares.csv`: los 707 lugares que no quedaron conectados. `tramo_suelto` (163): hay
  una vía a 500 m o menos, pero ese tramo no está unido a la red principal en el mapa.
  `sin_camino` (544): no hay vía para vehículos a 500 m; se anota la más cercana hasta 5 km.
- `red_vial_enlaces.csv`: 83 enlaces probables, uno por tramo suelto con lugares poblados: el
  hueco más corto (hasta 5 km) entre el tramo y la red principal, y los lugares que dependen de él.
- `red_vial_resumen.json`: totales y parámetros usados.

Cifras: 330,598 vías, 144,866 km, 2,101 tramos no unidos entre sí; 97.0 % de los kilómetros
está en la red principal; 19,016 de 19,723 lugares quedan conectados.

**Cómo leerlo.** Mide el mapa, no el terreno. Un hueco puede ser un camino que falta dibujar, un
paso en lancha o un vado (OpenStreetMap no registra los transbordadores como camino). El
resultado dice dónde mirar primero; la confirmación es en campo o en imagen. Los umbrales (500 m,
5 km) son parámetros del comando.
