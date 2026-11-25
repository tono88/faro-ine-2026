# Modelos: el que se usa y los que tendría sentido entrenar

## El que FARO usa hoy

| Modelo | Para qué | Dónde corre | Qué decide |
|---|---|---|---|
| Places365-ResNet18 (MIT CSAIL; pesos CC BY, código MIT), sin cambios | Decir si la foto de una visita de campo parece una vivienda o un comercio | Dentro del servidor, con `tract-onnx`; la foto no sale ni se guarda | Nada: marca la visita para revisión. Decide el supervisor |

Detalle, reglas y prueba con 16 fotos: [`24-clasificador-de-fotos.md`](24-clasificador-de-fotos.md).
Procedencia de los pesos: `datos/modelos/PROCEDENCIA.md`. **Todo lo demás en FARO son reglas fijas**:
la homologación, los tableros «desde una frase», las diferencias de precios y la huella de fotos de
producto no usan modelos entrenados.

## Los que tendría sentido entrenar (ninguno está hecho)

Ordenados por viabilidad para el INE.

| # | Modelo | Para qué | Con qué se entrena | Viabilidad |
|---|---|---|---|---|
| 1 | Regresión logística sobre rasgos de similitud de texto | Que las sugerencias de lugar mejoren con el uso: aprender qué tan parecido es «suficiente» | Las equivalencias que las personas confirman y rechazan en FARO | Alta: pocos datos, se entrena en segundos, explicable |
| 2 | Pareo probabilístico de registros (Fellegi–Sunter; biblioteca abierta Splink) | Unir registros de la misma persona o empresa entre instituciones cuando no hay identificador común | Los propios registros (estimación EM, sin etiquetas) | Alta dentro del INE; requiere convenio de datos |
| 3 | Clasificador de texto con embeddings multilingües abiertos | Codificar texto libre: causa de muerte a CIE-10, delito a su código, ocupación | Históricos ya codificados por el INE | Media: necesita los diccionarios y el histórico |
| 4 | Detección de edificaciones en imágenes propias | Crecimiento urbano con fotos aéreas o de dron del INE | Imágenes etiquetadas | Baja en el plazo del Datatón; por eso se usa GHSL, que ya entrega la superficie construida |
| 5 | Ajuste del clasificador de fachadas con fotos de campo del INE | Que distinga la vivienda y el comercio guatemaltecos mejor que un modelo general | Fotos de visitas ya revisadas por supervisores | Media: primero hay que medir el modelo actual con esas fotos |

Recomendación: empezar por el 1, porque se alimenta de lo que FARO ya guarda y no exige datos nuevos.
La regla de oro no cambia con ningún modelo: el modelo propone y ordena candidatos; asignar un
código sin coincidencia exacta lo decide una persona.
