# Comparación de productos por foto

## Para qué sirve

En el informe de precio de referencia, la búsqueda por palabras mezcla productos distintos
("laptop 8gb 256gb" trae varias marcas y modelos). La foto ayuda a quien revisa a quedarse solo con
los que son el mismo producto: se elige uno como referencia y el informe deja los que **se ven
como él** o **comparten su código de barras**. Orden de confianza: código de barras, foto, nombre.
La decisión sigue siendo de una persona.

## Cómo se calcula la huella (sin servicios externos ni modelos entrenados)

1. Se descarga la foto publicada por la tienda (reducida a 250 px donde la tienda lo permite).
2. Se recorta el fondo: el color de las esquinas se toma como fondo y se conserva el rectángulo
   que contiene el empaque.
3. **Forma (64 bits)**: el recorte se lleva a 32×32 en grises, se le aplica la transformada del
   coseno y, de las 64 frecuencias más bajas, cada una aporta un bit (mayor o no que la mediana).
4. **Color (48 valores)**: el recorte se reduce a una cuadrícula de 4×4 con el color medio de cada celda.
5. Dos fotos "se ven iguales" si difieren en **18 bits o menos** de forma **y** en **0.16 o menos**
   de color medio.

Código: `crates/faro-fuentes/src/foto.rs`. Cada huella se calcula una vez y se guarda (tabla `huellas_foto`).

## Calibración con datos reales (24-nov-2026)

`faro fotos --pares 300` toma del levantamiento de precios productos que tienen el **mismo código
de barras en dos tiendas con fotos distintas** (casi todos, La Torre frente a Walmart) y pares de
productos distintos, y mide la huella de cada par. Detalle par por par:
`datos/referencia/fotos_calibracion.csv`.

| Umbral (bits, color) | Mismo producto reconocido | Productos distintos aceptados por error |
|---|---|---|
| 10 bits, 0.08 | 13 de 300 (4.3 %) | 0 de 300 |
| **18 bits, 0.16 (en uso)** | **29 de 300 (9.7 %)** | **0 de 300** |
| 22 bits, 0.16 | 48 de 300 (16.0 %) | 5 de 300 (1.7 %) |
| 24 bits, 0.16 | 66 de 300 (22.0 %) | 7 de 300 (2.3 %) |

## Lo que esto significa, sin adornos

- **Cuando la huella dice que dos fotos son iguales, en la calibración no se equivocó nunca.**
- **Reconoce pocas veces el mismo producto cuando cada tienda tomó su propia foto** (otro ángulo,
  otro encuadre): 1 de cada 10. Entre fotos distintas del mismo producto la diferencia mediana de
  forma es de 30 bits, casi lo mismo que entre productos distintos (32). Sirve para detectar la
  **misma imagen reutilizada** (fotos del fabricante, frecuentes en tecnología y electrodomésticos),
  no para reconocer un producto fotografiado de otra manera.
- Por eso se eligió el umbral que no acepta errores y el código de barras sigue siendo la
  evidencia principal: 16 mil productos de supermercado se comparan por código, no por foto.
- Reconocer el mismo producto en fotos distintas requeriría un modelo entrenado de visión; no se
  usó porque el portal no incorpora modelos externos.

## Repetir

```bash
faro fotos --entrada datos/referencia/precios_publicados.csv --pares 300
```
