# Catálogo de artículos de precios

`ipmc_articulos.csv`: los 89 artículos del Índice de Precios de Materiales de Construcción (IPMC)
con su especificación, unidad, índice de septiembre de 2026 y variación frente a septiembre de 2025.

Fuente: INE, conjunto `indice-de-precios-de-materiales-de-construccion-ipmc` de
<https://datos.ine.gob.gt> (libros "IPMC Septiembre 2026" e "IPMC Año 2025"). Descargado el
24-nov-2026 (UTC); huellas en `datos/crudo/manifiesto.jsonl`. Se genera con `faro ipmc`.

`precios_referencia_ine.csv`: 4,990 precios de referencia emitidos por el INE entre 2016 y 2023
(producto, características, presentación, precio, fecha, modalidad, categoría y número de
operación en Guatecompras). Fuente: INE, conjunto `precios-de-referencia` de
<https://datos.ine.gob.gt> (libro "Precios de Referencia - Septiembre 2023"), descargado el
24-nov-2026 (UTC). Se genera con `faro referencias`. Se omiten 5 filas sin precio numérico y las
características se recortan a 300 caracteres. Las 222 categorías se conservan como vienen escritas.

Los nombres y unidades se copian del INE sin cambios. Este catálogo es el estándar contra el que
se homologan los productos de los proveedores.

`cba_precios.csv`: precio promedio mensual de los 34 productos de la canasta básica alimentaria,
de diciembre de 2020 a octubre de 2023. Fuente: INE, conjunto
`precios-promedio-canasta-basica-alimentaria` de <https://datos.ine.gob.gt>, descargado el
24-nov-2026 (UTC). Se genera con `faro canasta`. El conjunto no indica la unidad de cada producto.
