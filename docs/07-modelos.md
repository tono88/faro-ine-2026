# Modelos de código abierto que tendría sentido entrenar

Ordenados por viabilidad para el INE. Ninguno está implementado todavía.

| # | Modelo | Para qué | Con qué se entrena | Viabilidad |
|---|---|---|---|---|
| 1 | Regresión logística sobre rasgos de similitud de texto | Que las sugerencias de lugar mejoren con el uso: aprender qué tan parecido es "suficiente" | Las equivalencias que las personas confirman y rechazan en FARO | Alta: pocos datos, se entrena en segundos, explicable |
| 2 | Pareo probabilístico de registros (Fellegi–Sunter; biblioteca abierta Splink) | Unir registros de la misma persona o empresa entre instituciones cuando no hay identificador común | Los propios registros (estimación EM, sin etiquetas) | Alta dentro del INE; requiere convenio de datos |
| 3 | Clasificador de texto con embeddings multilingües abiertos | Codificar texto libre: causa de muerte a CIE-10, delito a su código, ocupación | Históricos ya codificados por el INE | Media: necesita los diccionarios y el histórico |
| 4 | Detección de edificaciones en imágenes propias | Crecimiento urbano con fotos aéreas o de dron del INE | Imágenes etiquetadas | Baja en el plazo del datatón; por eso se usa Open Buildings, que ya entrega el resultado |

Recomendación: empezar por el 1, porque se alimenta de lo que FARO ya guarda y no exige datos nuevos.
La regla de oro no cambia con ningún modelo: el modelo propone y ordena candidatos; asignar un
código sin coincidencia exacta lo decide una persona.
