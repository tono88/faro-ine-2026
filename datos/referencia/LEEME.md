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
esos datos y se comparten bajo la misma licencia ODbL
(<https://opendatacommons.org/licenses/odbl/1-0/>), con reconocimiento a © los colaboradores de
OpenStreetMap (<https://www.openstreetmap.org/copyright>). Lo mismo vale para
`red_vial_tipos_municipio.csv`, `red_vial_principal.geojson` y `red_vial_contexto.geojson`.

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

## `municipios.geojson` y `limites_verificacion.csv`

Capa de los 340 municipios con código INE para los mapas (`faro limites --entrada <zip>`).

Fuente: CONRED / OCHA, límites administrativos de Guatemala (COD-AB, conjunto `cod-ab-gtm` de HDX,
licencia CC BY-IGO), vigentes desde 2019. Descargado el 24-nov-2026 (UTC); SHA-256
`f178eda98c46329380bdbb43f0637b4c43535bc843de6a0b8b960193b8f4363f`. Los contornos se simplificaron (tolerancia ≈ 160 m) para que la capa pese
380 KB; no sirven para medir linderos.

Verificación contra el censo: se ubicó cada lugar poblado del Censo 2018 dentro de las figuras.

- La capa original trae **intercambiados los códigos de Sanarate y Sansare** (El Progreso): la
  figura rotulada `0206` contiene en 98 % lugares de `0207` y viceversa. Se corrigió.
- Trae dos figuras que no son municipios (`0100` y `0700`, los lagos de Amatitlán y Atitlán): se omiten.
- **86.9 % de los lugares poblados cae dentro de la figura de su municipio**; el resto cae en un
  municipio vecino. En 137 municipios menos del 90 % de sus lugares queda dentro de su figura
  (detalle en `limites_verificacion.csv`). No es un error de FARO ni necesariamente del censo:
  estos límites son operativos, no legales. Indica dónde el límite y el censo no coinciden.

## `superficie_construida_municipio.csv`

Hectáreas construidas por municipio cada cinco años, de 1975 a 2020 (`faro construido`).

Fuente: Comisión Europea, Centro Común de Investigación (JRC), GHSL GHS-BUILT-S R2023A, 100 m,
proyección Mollweide (Pesaresi y Politis, 2023), datos abiertos con atribución. 30 mosaicos
(3 por época) descargados el 24-nov-2026 de `jeodpp.jrc.ec.europa.eu`; cada uno está en
`datos/crudo/manifiesto.jsonl` con su URL y SHA-256.

Método: los límites municipales se llevan a Mollweide y se suman las celdas cuyo centro cae dentro
de cada municipio (cada celda trae los metros cuadrados construidos). `celdas` es cuántas se sumaron.

Cifras del país: 30,722 ha en 1975, 60,593 en 2000 y 93,297 en 2020.

Cómo leerlo: es superficie edificada vista por satélite, no viviendas ni personas. Las épocas
anteriores a 2015 vienen de imágenes Landsat y son menos precisas. GHSL publica 2025 y 2030 como
proyecciones; no se usan. Las opciones evaluadas están en `docs/10-imagenes-satelitales.md`.

## `proveedores_construccion.csv` y `precios_construccion.csv`

Levantamiento del 24-nov-2026 en páginas públicas de negocios de materiales de construcción de
Guatemala: 116 perfiles (tipo, ubicación, canales, si publica precios, rubros y la página de
donde salió cada perfil) y 2,332 precios de 16 de ellos, cada uno con el texto del producto tal
como lo publica la tienda, la unidad, la dirección exacta de la página y la fecha.

- Solo negocios e información comercial pública. Sin nombres de personas, teléfonos ni correos.
- 61 perfiles vienen del directorio Páginas Amarillas: su tipo y rubros se dedujeron del nombre y
  la categoría, y no tienen precios.
- No se entró a sitios con bloqueo, captcha o inicio de sesión (Facebook, Instagram, Encuentra24 y
  varios fabricantes quedaron fuera por eso). Casi todos los precios son de cadenas con tienda en
  línea; no hay precios de ferreterías del interior.
- La mayoría de las páginas no dice si el precio incluye IVA.
- Son precios publicados, no cotizaciones: para la Dirección de Precios son fuentes secundarias.

La comparación contra el IPMC (`faro construccion`) usa las reglas de `datos/estandar/ipmc_reglas.toml`.

## Capas agregadas

- `departamentos.geojson`: 22 departamentos, de la misma fuente que los municipios (`faro limites`).
- `red_vial_contexto.geojson`: 2,384 tramos de camino a 1.5 km de cada enlace propuesto, con la
  marca de si pertenecen a la red principal o a un tramo suelto (`faro vial`; © colaboradores de
  OpenStreetMap, ODbL).
- `superficie_construida_bloques.csv`: 26,669 bloques de 1 km con al menos media hectárea
  construida en alguna época, con su centro, su municipio y las hectáreas por época
  (`faro construido`; GHSL). Es la huella edificada que anima el mapa.

## `cif_departamentos.csv`

Personas en cada fase de inseguridad alimentaria aguda (CIF) por departamento, octubre de 2026 a
febrero de 2027: población analizada, fases 1 a 5, fase del área y Fase 3 o superior.

Fuente: SESAN y Grupo Técnico de la CIF Guatemala, "Análisis de inseguridad alimentaria aguda de la
CIF: actualización de la situación proyectada octubre 2026 – febrero 2027", publicado el
19-nov-2026 y corregido el 20-nov-2026. Descargado de `portal.sesan.gob.gt` el 24-nov-2026 (UTC);
SHA-256 `60cc4c24c51aaaba6f6fa78f4ab77c7f5c6d2a2479ec67f38880f92269946edf`.

Cómo se obtuvo: `pdftotext -layout informe.pdf informe.txt` y luego `faro cif --texto informe.txt`,
que lee la "Tabla de población" y comprueba, antes de escribir, que en cada departamento las fases
suman la población y que fase 3 + 4 + 5 da la Fase 3 o superior publicada, y que los 22
departamentos suman el Gran Total del informe. Los nombres se llevan al código INE con el mismo
motor de homologación.

Verificación cruzada: la población analizada de los 22 departamentos coincide exactamente con la
proyección del INE para 2026 que usa FARO (la tabla con los 16 códigos de municipio reubicados).

Las cifras son por departamento; el informe no las publica por municipio. Son 3,247,361 personas
en Fase 3 o superior (18 % de la población analizada) y 11 departamentos con área en Fase 3.

## `censo_talla_municipio.csv`

Porcentaje de escolares con retardo en talla (desnutrición crónica), escolares evaluados y
categoría de vulnerabilidad nutricional, para los 340 municipios.

Fuente: SESAN, SIINSAN, "Retardo en talla, Censo 2024" (V Censo Nacional de Talla en Escolares).
Descargado de `portal.siinsan.gob.gt` el 24-nov-2026 (UTC); SHA-256 `8f8f8f0d92c9bbc72dd8740eb2aed5a775e7956e8043e4abb00553aae4ee5547`.

Cómo se obtuvo: `faro talla --entrada <libro>`. El libro trae código de departamento y correlativo
de municipio; se componen en el código INE y se comprueba, además, que el nombre que trae el libro
lleva al mismo municipio. Coinciden los 340 (uno, "San Raimundo", quedó agregado como otra forma de
escribir San Raymundo). Las cifras y categorías se copian sin cambios: 90 municipios en
vulnerabilidad muy alta, 90 en alta, 89 en moderada y 71 en baja.

## `precios_publicados.csv`

Levantamiento de precios publicados en línea, hecho por los muestreos del portal el 24-nov-2026:
85,907 precios de 18 tiendas en seis rubros (supermercado, ferretería, tecnología, hogar, farmacia,
vehículos). Columnas: rubro, tienda, producto, marca, precio en quetzales, precio de lista (si la
tienda lo publica), código de barras (si lo publica), categoría en la tienda, dirección de la página
y fecha de consulta. Son precios publicados en páginas públicas, no cotizaciones ni precios en tienda
física. El servidor los carga al arrancar (tarea `levantamiento-inicial`) y los reemplaza si esta tabla cambia; los muestreos
posteriores se guardan aparte y permiten ver cambios. Tareas y lecturas: `datos/estandar/muestreos.toml`.

## `hba_2024_cuentas.csv`

Cuentas de utilización y suministro de la Hoja de Balance de Alimentos 2024 del INE, en una tabla
limpia: alimento, producción, importaciones, exportaciones, procesado y alimentos (toneladas métricas).
Se genera con `faro hba --cuentas <libro CUS>` (conjunto `hoja-de-balance-de-alimentos-hba-actualizacion-2024`
del portal de datos abiertos). Sirve para contrastar las tasas de extracción de la FAO
(`datos/estandar/tasas_extraccion_fao_guatemala.csv`, transcritas de las páginas 222–225 del documento,
con la columna `relacion` que dice si el primario aparece en la fuente o se dedujo del nombre).

## `censo2018_lugar_poblado.csv` y `poblacion_municipio_edades.csv`

- `censo2018_lugar_poblado.csv`: Censo 2018 por lugar poblado (20,036 filas): población, hombres,
  mujeres, tres grandes grupos de edad, urbana, rural, hogares, viviendas y viviendas ocupadas.
  Sale de los cuadros A1, B8 y C1 del conjunto `censo-2018-lugares-poblados` del INE con
  `faro censo-lugares`, que comprueba que cada código exista en el catálogo y que sexo y edades sumen el total.
- `poblacion_municipio_edades.csv`: hombres y mujeres por grupo quinquenal de edad (0–4 … 70 y más)
  en cada municipio para 2018, 2026 y 2030, de las proyecciones del INE, con `faro edades`. Reubica
  los 16 códigos inexistentes igual que `faro poblacion` y comprueba que sume los totales publicados.
- `fotos_calibracion.csv`: pares de fotos medidos por `faro fotos` (ver `docs/13-fotos.md`).
- `precios_publicados.csv` trae ahora la columna `imagen` (dirección de la foto publicada).

## Tablas agregadas después (v0.16)

| Archivo | Qué es | Fuente y licencia | Comando |
|---|---|---|---|
| `censo2018_indicadores_municipio.csv` | Quince indicadores de condiciones de vida por municipio, con su numerador y su denominador | INE, Censo 2018, cuadros A5, A9, A11, A12, A13, B1, B2, B3, B4, B6, C2 y C3 (CC BY); descargados el 24-nov-2026 | `faro censo-indicadores` |
| `censo2018_techos_lugar_poblado.csv` | Viviendas por material predominante del techo en cada lugar poblado | INE, Censo 2018, cuadro C2 (CC BY); 24-nov-2026 | `faro censo-indicadores` |
| `uso_construido_municipio.csv`, `uso_construido_bloques.csv` | Superficie construida residencial y no residencial, por municipio y por bloque de 1 km | Comisión Europea, JRC, GHSL GHS-BUILT-S R2023A, componente NRES (CC BY 4.0); 24-nov-2026. Cita obligatoria: Pesaresi et al. (2024), *International Journal of Digital Earth* 17(1) | `faro uso-suelo` |
| `red_vial_tipos_municipio.csv` | Kilómetros de vía por clase y por superficie en cada municipio | © colaboradores de OpenStreetMap, exportación de HOT en HDX (ODbL 1.0); 24-nov-2026 | `faro vial` |
| `red_vial_principal.geojson` | Carreteras principales y secundarias para el mapa | La misma (ODbL 1.0) | `faro vial` |
| `fachadas_prueba.csv` | Resultado del clasificador en 16 fotos de prueba: página, autor y licencia de cada foto, y lo que dijo el modelo | Wikimedia Commons (CC BY o CC BY-SA, según cada foto). **Las fotos no están en el repositorio** | `faro fachadas` |
| `municipios.geojson`, `departamentos.geojson` | Figuras para el mapa | **FARO, a partir de** CONRED / OCHA, COD-AB Guatemala (CC BY-IGO): contornos simplificados, Sanarate y Sansare corregidos, dos lagos quitados | `faro limites` |

Las tasas de extracción de `datos/estandar/tasas_extraccion_fao_guatemala.csv` son una **transcripción
traducida** de un documento de la FAO, que permite su uso para estudio, investigación y productos o
servicios no comerciales citando la fuente; no están bajo CC BY. Un uso distinto debe consultarse
con la FAO (<https://www.fao.org/contact-us/terms/es/>).
