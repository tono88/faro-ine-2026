# Verificación estadística y de consistencia

Este documento lo escribe `faro verificar` a partir de las tablas del repositorio: cada cifra se
vuelve a calcular al ejecutarlo. Repetir: `faro verificar`.

## 1. Consistencia de las tablas

| Comprobación | Resultado | Detalle |
|---|---|---|
| Población: hombres + mujeres = total | cuadra | 5440 filas, 340 municipios; 0 filas distintas |
| Edades: la suma de los grupos = total del municipio y año | cuadra | 1020 municipios-año comparados; 0 distintos |
| Censo 2018 por lugar poblado: sexo, edad y área suman el total | cuadra | 20036 lugares; población 14901286; 0 filas distintas |
| Censo 2018 frente a la proyección 2018 del INE | cuadra | censados 14901286; proyectados 16346950; diferencia 8.8 % (la proyección corrige la omisión censal: debe ser mayor) |
| Red vial: cada lugar sin conexión tiene código de 7 dígitos y existe en el censo | cuadra | 707 lugares revisados |

## 2. Proyección de superficie construida: prueba hacia atrás

El portal proyecta 2030 con la recta de las cinco épocas 2000–2020. Para saber cuánto acierta ese
método se repitió hacia atrás en los 340 municipios: con las cinco épocas anteriores se proyectó a
diez años y se comparó con lo observado.

| Año proyectado | Con datos de | Error mediano | Error del 90 % | Dentro del margen de 8 de cada 10 | Dentro del margen al 95 % |
|---|---|---|---|---|---|
| 2010 | 1980–2000 | 8.0 % | hasta 19.2 % | 316 de 340 (93 %) | 339 de 340 (100 %) |
| 2015 | 1985–2005 | 7.2 % | hasta 19.3 % | 298 de 340 (88 %) | 327 de 340 (96 %) |
| 2020 | 1990–2010 | 13.6 % | hasta 34.3 % | 201 de 340 (59 %) | 302 de 340 (89 %) |

En los 1020 casos: lo observado cayó dentro del margen de 8 de cada 10 en 80 % y dentro del margen al 95 % en 95 %.

**Qué se corrigió gracias a esta prueba.** El margen anterior salía solo del error del ajuste (± 2 errores
estándar) y contenía lo observado en 461 de 1020 casos (45 %): prometía mucha más precisión de la real, porque la
serie satelital es suave pero no crece en línea recta. Ahora el margen es el error realmente observado al
proyectar diez años: 20 % de lo proyectado en 8 de cada 10 casos y 47 % en 95 de cada 100 (constantes
`ERROR_OBSERVADO_P80` = 0.2 y `ERROR_OBSERVADO_P95` = 0.47 del código).

También se probaron otros métodos con la misma prueba (recta de tres épocas, crecimiento exponencial,
tasa o incremento de la última década): ninguno fue mejor en las tres décadas, así que se conserva la
recta, que es la más fácil de explicar. Proyectar 2020 es lo que peor sale en todos (error mediano de 11 % a
14 %, siempre por debajo de lo observado): la capa satelital muestra una aceleración en su última época.

## 3. Intervalo de confianza de la mediana de precios

El informe de precio de referencia da, junto a la mediana, un intervalo que no supone ninguna
distribución (estadísticos de orden). Cobertura exacta según la cantidad de precios:

| Precios | Puestos que forman el intervalo | Cobertura exacta |
|---|---|---|
| 5 | no se da intervalo | no alcanza 95 % |
| 6 | 1.º y 6.º | 96.9 % |
| 8 | 1.º y 8.º | 99.2 % |
| 10 | 2.º y 9.º | 97.9 % |
| 15 | 4.º y 12.º | 96.5 % |
| 20 | 6.º y 15.º | 95.9 % |
| 30 | 10.º y 21.º | 95.7 % |
| 50 | 18.º y 33.º | 96.7 % |

## 4. Diferencias de precio entre tiendas

16302 productos tienen el mismo código de barras en dos o más tiendas. Diferencia entre la más cara y la más barata:
mediana 1.6 %, percentil 90 24.8 %, percentil 99 100.0 %.

| Umbral para revisar | Productos que lo superan | Proporción |
|---|---|---|
| 25 % o más | 1628 | 10.0 % |
| 50 % o más (en uso) | 539 | 3.3 % |
| 100 % o más | 210 | 1.3 % |
| 200 % o más | 32 | 0.2 % |

Con el umbral en uso, las reglas reparten los 539 casos así: atipico 7 (1 %), oferta 412 (76 %), por_confirmar 120 (22 %). El umbral de 50 % deja a revisar cerca del 3 % de los productos: bajarlo a 25 % triplicaría el trabajo y subirlo a 100 % dejaría pasar errores evidentes.

## 5. Comparación por foto

Con los umbrales en uso (18 bits, 0.16 de color) sobre los pares medidos:

| | Resultado | Intervalo de confianza al 95 % (Wilson) |
|---|---|---|
| Productos distintos aceptados por error | 0 de 300 | entre 0.0 % y 1.3 % |
| Mismo producto reconocido (fotos distintas) | 29 de 300 | entre 6.8 % y 13.5 % |

Lectura: no haber visto ningún error en 300 pares no prueba que el error sea cero; con 95 % de confianza es menor que 1.3 %.

## 6. Tasas de extracción de la FAO frente a las cuentas 2024 del INE

«Con cifras del INE» es la producción del derivado dividida entre todo el primario que el INE reporta
como procesado. Coincide con la tasa de extracción cuando lo procesado va a ese derivado; si el
primario tiene otros usos industriales (el maíz va sobre todo a masa y tortilla, no a harina), sale
menor y no contradice la tasa.

| Primario → derivado | FAO | Con cifras del INE | Diferencia (puntos) |
|---|---|---|---|
| Trigo → Harina de Trigo | 72.0 % | 72.0 % | -0.0 |
| Trigo → Salvado de Trigo | 22.0 % | 22.0 % | 0.0 |
| Arroz en Cáscara → Arroz Elaborado | 62.0 % | 69.3 % | 7.3 |
| Arroz en Cáscara → Salvado de Arroz | 8.0 % | 7.9 % | -0.1 |
| Maíz → Harina de Maíz | 98.0 % | 22.4 % | -75.6 |
| Sorgo → Harina de Sorgo | 95.0 % | 95.0 % | -0.0 |
| Yuca (Mandioca) → Tapioca de Yuca | 25.0 % | 12.5 % | -12.5 |
| Yuca (Mandioca) → Yuca Seca | 40.0 % | 20.0 % | -20.0 |
| Caña de azúcar → Azúcar Caña (En Bruto) | 10.0 % | 10.2 % | 0.2 |
| Caña de azúcar → Melazas | 3.9 % | 4.1 % | 0.2 |
| Soja → Aceite de Soja | 18.0 % | 18.6 % | 0.6 |
| Almendras de Palma → Aceite de Almendras Palm | 45.0 % | 45.1 % | 0.1 |
| Tomates → Jugo de Tomate | 65.0 % | 65.0 % | -0.0 |
| Hortalizas Frescas Nep → Hortalizas Conserv Otras | 80.0 % | 2.9 % | -77.1 |
| Café Verde → Café Tostado | 80.0 % | 80.0 % | -0.0 |
| Cacao en grano → Pasta de Cacao | 80.0 % | 80.0 % | -0.0 |
| Pasta de Cacao → Manteca de Cacao | 47.0 % | 47.0 % | 0.0 |
| Pasta de Cacao → Cacao en Polvo | 53.0 % | 53.0 % | -0.0 |

18 tasas pudieron contrastarse (el nombre del producto coincide en ambas fuentes). En 13 la diferencia es de 2 puntos o menos (mediana de todas: 0.1 puntos). Las que se apartan mucho son primarios con varios usos (maíz, yuca, hortalizas) y el arroz elaborado (69.3 % frente a 62 %).

## Comprobaciones de los supuestos del código

| Comprobación | Resultado | Detalle |
|---|---|---|
| Las constantes de margen del código coinciden con el error observado | cuadra | observado: 20.1 % y 47.3 % |

## Resultado

Todas las comprobaciones de consistencia cuadran.
