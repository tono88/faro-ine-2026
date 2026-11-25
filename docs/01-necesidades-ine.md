# Necesidades planteadas por el INE (lanzamiento, 23 nov 2026)

## Planteadas por la Gerencia

| # | Necesidad | Cómo la atiende FARO |
|---|---|---|
| 1 | **Integrar encuestas con registros administrativos** de otras instituciones (MP, Salud y otras), que hoy no tienen un estándar para compartir datos. Que el INE lidere una homologación del envío. X-Road existe, pero no está pensado para datos estadísticos. | **Núcleo del proyecto.** Solicitud de datos declarativa, validación del envío, homologación con memoria. Ver [arquitectura](03-arquitectura.md). |
| 2 | Operadores de campo asignados lejos de donde viven; la asignación no optimiza distancias. | Módulo secundario, **funcionando**: asignación operador–puesto que minimiza la distancia total (método húngaro, exacto), usando las coordenadas del listado de lugares poblados. Distancia en línea recta; con la red vial (necesidad 4) pasaría a distancia por carretera. |
| 3 | Detectar crecimiento urbano a partir de imágenes. | Módulo secundario, **funcionando**: superficie construida por municipio y por bloque de 1 km, de 1975 a 2020, con la capa satelital abierta GHSL (`faro construido`); animación sobre la foto satelital de cada época. Open Buildings quedó como opción no usada. |
| 4 | Carreteras sin mapear; uso de teoría de grafos para completar. | Módulo secundario, **funcionando**: grafo vial de OpenStreetMap (144,866 km); componentes conexos, lugares poblados en tramos sueltos o sin camino mapeado y el enlace más corto que probablemente falta. |

## Planteadas en las presentaciones

| Dirección | Propuesta | Relación con FARO (el estado de cada una está en la tabla del final) |
|---|---|---|
| Índices y Estadísticas Continuas (DESA) | Validación de bases de datos con herramientas de licencia libre; tableros dinámicos | La validación de envíos es el paso 2 del núcleo |
| Índices y Estadísticas Continuas (DCR) | Herramienta con datos del censo por municipio y lugar poblado; tablero de avance de recopilación de las 22 delegaciones | El catálogo de lugares poblados es la base común; el reporte de recepción alimenta un tablero de avance |
| Índices y Estadísticas Continuas (IPP) | Registro de informantes, productos y precios; alertas de variaciones atípicas | Mismo patrón: solicitud + validación + alertas |
| Índices y Estadísticas Continuas (IPC, HBA, IPMC, ICC) | Catálogo digital del Manual de Características; calculadora de derivados HBA; sistematizar procesos | La calculadora de derivados de la HBA y el IPMC están hechos; el Manual de Características del IPC no es público y el ICC no se hizo |
| Precios e Índices de Referencia | Investigación de precios de mercado con cotizaciones, catálogos en línea y Guatecompras; media, mediana y moda | Módulo secundario: homologar descripciones de productos entre proveedores y calcular el precio de referencia |
| Censos y Encuestas | Validación inteligente, seguimiento cartográfico, comparación de censos | Seis cuidados de uso de datos que FARO respeta (factor de expansión, dominios, no identificar personas) |

## Lineamientos que condicionan la entrega

- Entrega: jueves 26 de noviembre, 09:00. Finalistas notificados el jueves entre 16:00 y 18:00.
- Presentación final: 27 de noviembre, 7 minutos de exposición y 3 de preguntas.
- Al menos una base pública del INE como fuente principal; otras fuentes abiertas, citadas.
- Repositorio Git desde el día 1 con acceso para el jurado; prototipo funcional y reproducible.
- Servicios de pago o en la nube deben declararse.
- Los integrantes conservan la autoría y otorgan al INE una licencia no exclusiva, gratuita, perpetua e
  irrevocable (categoría 2; la cesión de derechos es la regla de la categoría 1). Ver `LICENCIA.md`.
- Enviar por correo la nota de consentimiento firmada por todos los integrantes.

## Estado frente a cada propuesta de las presentaciones (24-nov-2026)

| Propuesta | Dirección | Estado en FARO |
|---|---|---|
| Sistema para registrar informantes, productos y precios, con alertas de variaciones atípicas | DIEC · IPP | Hecho: muestreos automáticos, "Precios por rubro" (tiendas informantes, cambios de 15 % o más entre muestreos) |
| Sistema para estandarizar los procedimientos del IPMC y del ICC | DIEC · IPMC/ICC | Parcial: canasta del IPMC como catálogo, homologación de productos y precios por artículo; el ICC no se abordó |
| Catálogo digital del Manual de Características para dispositivos móviles | DIEC · IPC | No hecho: el Manual no es público; la pantalla sí funciona en teléfonos y el catálogo del IPMC se consulta desde ella |
| Calculadora de derivados de la Hoja de Balance de Alimentos | DIEC · HBA | Hecho: sección "Hoja de balance: derivados" con las tasas de Guatemala de la FAO, contrastadas con la HBA 2024 del INE; admite otra tabla |
| Validación de bases de datos con herramientas de licencia libre; tableros dinámicos | DIEC · DESA | Hecho: validación de cada envío contra su solicitud, resultados y mapa; diseñador de tableros con filtros y publicación por enlace |
| Herramienta con datos del censo por municipio y lugar poblado para las delegaciones | DIEC · DCR | Hecho: mapa en tres niveles, buscador, ficha y reporte por municipio, con población, hogares y viviendas del Censo 2018 por lugar poblado |
| Tablero de avance de recopilación de las 22 delegaciones | DIEC · DCR | Hecho: cobertura por departamento en "Resultados y avance" y registros en el mapa por departamento |
| Investigación de precios con catálogos en línea; media, mediana y moda; informe técnico | DPIR | Hecho: precio de referencia desde cotizaciones y, ahora, informe imprimible desde precios publicados. Falta Guatecompras (bloqueado desde la nube) |
| Proyecciones por territorio; cambios entre censos; limpiar códigos especiales; no identificar personas | DCE | Hecho salvo la comparación entre censos: población 2015–2030 por municipio, sexo y edad (pirámides), valores "sin dato", salida sin datos personales. Los censos 2002 y 1994 por municipio no están en formato descargable |

## Pedidos atendidos en la versión 0.16.0 (24-nov-2026)

| Pedido | Estado en FARO |
|---|---|
| Que un grupo de usuarios acceda al trabajo de campo y cada operador conozca su ruta | Hecho: rol y grupo «Operadores de campo»; sección «Mi ruta de campo» con las paradas en orden |
| Encuestas conectadas a Microsoft Forms con la geolocalización automática | Hecho: el teléfono entrega la posición al registrar la visita y Forms se abre con lugar, posición y número de visita. FARO no lee las respuestas: se unen por el número de visita |
| Foto del lugar y un modelo ligero que confirme que es una vivienda | Hecho, con límites declarados: Places365 corre en el servidor; probado con 16 fotos (11 aciertos, 0 errores, 5 no concluyentes) |
| Ver la foto satelital durante la animación de la mancha urbana; que corra una sola vez | Hecho: Landsat (NASA) y Sentinel-2 (EOX) de fondo; corre una vez y hay casilla para repetir |
| Uso del suelo según el tipo de techo, a partir de Landsat | No de esa forma: Landsat no distingue techos (cuadros de 30 m). En su lugar: residencial y no residencial del GHSL, y el techo declarado en el Censo 2018 por lugar poblado. El método con dron queda citado como siguiente paso |
| Que los filtros se apliquen al tablero ya creado y saber cómo se publican | Hecho: barra de filtros sobre todo el tablero; borrador y publicado, con enlace |
| Un modelo distinto de la recta para población y crecimiento | Hecho: tasa constante junto a la recta (superficie construida) y, para población después de 2030, exponencial a corto plazo y logística a largo plazo |
| Análisis de conformidad legal y protección estadística | Hecho: `docs/23`; conteos menores que 3 protegidos |
| Apartado comparativo de qué se tenía, qué mejora y cómo escala | Hecho: sección «Antes y ahora» (31 puntos) y `docs/22` |
| Más indicadores | Hecho: quince del Censo 2018, uno de uso del suelo y tres de vías en los tableros (29 en total; el mapa tiene además el porcentaje de vías pavimentadas, con su advertencia) |
| Mapeo y tipo de carretera | Hecho con lo que ya se tenía: tipo de vía en toda la red y superficie donde está anotada (38 %), por municipio y en el mapa. No se halló una capa abierta del RIC ni de la Dirección General de Caminos |
| Inclusión: que la pantalla se lea en más de un idioma (pedido del equipo, 25-nov) | Hecho en inglés (completo). Kaqchikel **en borrador**: 680 rótulos (34 % de la pantalla) armados con vocabulario escrito por hablantes; falta la revisión de una persona experta y los textos largos. Se traduce la interfaz, nunca los datos (`docs/29`) |

<!-- equipo -->
---

<p align="center"><img src="identidad/equipo/carta-de-presentacion.svg" alt="Equipo La Sociedad de Almas: Estuardo Sandoval, Ana Echeverría y Alejandro Sandoval, con el código QR del perfil de LinkedIn de cada uno" width="760"></p>

<p align="center"><a href="https://www.linkedin.com/in/estuardoantoniosandoval/">Estuardo Sandoval</a> · <a href="https://www.linkedin.com/in/analuciaecheverria/">Ana Echeverría</a> · <a href="https://www.linkedin.com/in/alejandro-jos%C3%A9-sandoval-acevedo-998781aa/">Alejandro Sandoval</a><br>Equipo La Sociedad de Almas · Datatón INE 2026</p>
