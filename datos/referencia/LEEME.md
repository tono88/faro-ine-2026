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
