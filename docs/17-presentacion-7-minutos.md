# Presentación ante el jurado (7 minutos + 3 de preguntas)

## Estructura

| Minuto | Tema | Idea que debe quedar |
|---|---|---|
| 0:00–0:45 | El problema | Las instituciones no tienen un estándar para enviar datos; el mismo lugar llega escrito de muchas formas |
| 0:45–2:15 | Demostración 1: pedir, recibir, homologar | El sistema no adivina; lo que decide una persona queda aprendido |
| 2:15–3:30 | Demostración 2: territorio | Mapa en tres niveles, censo por lugar poblado, red vial como grafo, operadores |
| 3:30–5:00 | Demostración 3: precios | 85 mil precios publicados; código de barras; reglas para diferencias; informe de precio de referencia |
| 5:00–6:00 | Confianza | Todo citado y reproducible; hallazgos en los datos; nada inventado; sin modelos externos |
| 6:00–7:00 | Adopción | Un comando para levantarlo; usuarios y roles; qué falta y qué sigue |

## Hallazgos para mencionar (uno por bloque)

- El libro de proyecciones del INE trae 16 códigos de municipio que no existen; se reubicaron con su propio diccionario.
- 707 lugares poblados no conectan con la red vial mapeada; 83 enlaces probables suman 51 km.
- 16 mil productos de supermercado se comparan entre tiendas por código de barras; de las diferencias de 50 % o más, tres de cada cuatro son ofertas.
- Las tasas de extracción de la FAO para Guatemala (de los años noventa) siguen coincidiendo con la Hoja de Balance 2024 del INE: trigo 72 %.

## Preguntas probables y respuesta corta

| Pregunta | Respuesta |
|---|---|
| ¿Usa inteligencia artificial? | En ejecución, no: todo son reglas fijas y a la vista; el mismo dato da siempre el mismo resultado. Se usó asistencia de IA para programar, y está declarado en el README. |
| ¿Qué pasa con los datos personales? | Los archivos reales se procesan en un temporal y no se guardan; la salida estándar no lleva nombres ni identificadores, y en el repositorio no hay ningún dato personal. |
| ¿Cómo se relaciona con X-Road? | X-Road transporta; FARO define qué se pide y comprueba lo que llega. La solicitud de FARO puede viajar por X-Road u otro canal. |
| ¿Qué tan difícil es agregar una institución? | Un archivo de solicitud (columnas, niveles geográficos, catálogos) y nada de código. |
| ¿Y si el sistema se equivoca al homologar? | Solo asigna con coincidencia exacta y única o con una equivalencia confirmada; lo demás queda pendiente. Cada decisión tiene autor y se puede revertir. |
| ¿Los precios publicados sirven como precio de referencia? | Son fuentes secundarias: sirven de antecedente; la Dirección de Precios recomienda más de 50 % de cotizaciones directas y el informe lo dice. |
| ¿Por qué Rust? | Un solo ejecutable, rápido y sin dependencias de sistema; se levanta con Docker en cualquier máquina. |
| ¿Qué no hace todavía? | Guatecompras (bloqueado para clientes automáticos), comparación con censos anteriores (no hay fuente descargable), el ICC y el Manual de Características del IPC. |
| ¿Cómo se mantiene? | Los datos abiertos se vigilan solos; cada tabla derivada se regenera con un comando documentado; hay pruebas automáticas del núcleo y de la pantalla. |
| ¿Qué necesita el INE para adoptarlo? | Un servidor con Docker, definir sus solicitudes y cargar sus usuarios. |
