# Necesidades planteadas por el INE (lanzamiento, 23 nov 2026)

## Planteadas por la Gerencia

| # | Necesidad | Cómo la atiende FARO |
|---|---|---|
| 1 | **Integrar encuestas con registros administrativos** de otras instituciones (MP, Salud y otras), que hoy no tienen un estándar para compartir datos. Que el INE lidere una homologación del envío. X-Road existe, pero no está pensado para datos estadísticos. | **Núcleo del proyecto.** Solicitud de datos declarativa, validación del envío, homologación con memoria. Ver [arquitectura](03-arquitectura.md). |
| 2 | Operadores de campo asignados lejos de donde viven; la asignación no optimiza distancias. | Módulo secundario, **funcionando**: asignación operador–puesto que minimiza la distancia total (método húngaro, exacto), usando las coordenadas del listado de lugares poblados. Distancia en línea recta; con la red vial (necesidad 4) pasaría a distancia por carretera. |
| 3 | Detectar crecimiento urbano a partir de imágenes. | Módulo secundario: edificaciones por año (Google Open Buildings 2.5D Temporal, 2016–2023) por municipio, comparadas con el Censo 2018. |
| 4 | Carreteras sin mapear; uso de teoría de grafos para completar. | Módulo secundario, **funcionando**: grafo vial de OpenStreetMap (144,866 km); componentes conexos, lugares poblados en tramos sueltos o sin camino mapeado y el enlace más corto que probablemente falta. |

## Planteadas en las presentaciones

| Dirección | Propuesta | Relación con FARO |
|---|---|---|
| Índices y Estadísticas Continuas (DESA) | Validación de bases de datos con herramientas de licencia libre; tableros dinámicos | La validación de envíos es el paso 2 del núcleo |
| Índices y Estadísticas Continuas (DCR) | Herramienta con datos del censo por municipio y lugar poblado; tablero de avance de recopilación de las 22 delegaciones | El catálogo de lugares poblados es la base común; el reporte de recepción alimenta un tablero de avance |
| Índices y Estadísticas Continuas (IPP) | Registro de informantes, productos y precios; alertas de variaciones atípicas | Mismo patrón: solicitud + validación + alertas |
| Índices y Estadísticas Continuas (IPC, HBA, IPMC, ICC) | Catálogo digital del Manual de Características; calculadora de derivados HBA; sistematizar procesos | Fuera del alcance de esta entrega |
| Precios e Índices de Referencia | Investigación de precios de mercado con cotizaciones, catálogos en línea y Guatecompras; media, mediana y moda | Módulo secundario: homologar descripciones de productos entre proveedores y calcular el precio de referencia |
| Censos y Encuestas | Validación inteligente, seguimiento cartográfico, comparación de censos | Seis cuidados de uso de datos que FARO respeta (factor de expansión, dominios, no identificar personas) |

## Lineamientos que condicionan la entrega

- Entrega: jueves 26 de noviembre, 09:00. Finalistas notificados el jueves entre 16:00 y 18:00.
- Presentación final: 27 de noviembre, 7 minutos de exposición y 3 de preguntas.
- Al menos una base pública del INE como fuente principal; otras fuentes abiertas, citadas.
- Repositorio Git desde el día 1 con acceso para el jurado; prototipo funcional y reproducible.
- Servicios de pago o en la nube deben declararse.
- Los integrantes ceden al INE los derechos de autor y propiedad intelectual.
- Enviar por correo la nota de consentimiento firmada por todos los integrantes.

## Estado frente a cada propuesta de las presentaciones (24-nov-2026)

| Propuesta | Dirección | Estado en FARO |
|---|---|---|
| Sistema para registrar informantes, productos y precios, con alertas de variaciones atípicas | DIEC · IPP | Hecho: muestreos automáticos, "Precios por rubro" (tiendas informantes, cambios de 15 % o más entre muestreos) |
| Sistema para estandarizar los procedimientos del IPMC y del ICC | DIEC · IPMC/ICC | Parcial: canasta del IPMC como catálogo, homologación de productos y precios por artículo; el ICC no se abordó |
| Catálogo digital del Manual de Características para dispositivos móviles | DIEC · IPC | No hecho: el Manual no es público; la pantalla sí funciona en teléfonos y el catálogo del IPMC se consulta desde ella |
| Calculadora de derivados de la Hoja de Balance de Alimentos | DIEC · HBA | Hecho: sección "Hoja de balance: derivados" con las tasas de Guatemala de la FAO, contrastadas con la HBA 2024 del INE; admite otra tabla |
| Validación de bases de datos con herramientas de licencia libre; tableros dinámicos | DIEC · DESA | Hecho: validación de cada envío contra su solicitud, pistas para códigos inexistentes, resultados y mapa |
| Herramienta con datos del censo por municipio y lugar poblado para las delegaciones | DIEC · DCR | Hecho: mapa en tres niveles, buscador, ficha y reporte por municipio, con población, hogares y viviendas del Censo 2018 por lugar poblado |
| Tablero de avance de recopilación de las 22 delegaciones | DIEC · DCR | Hecho: cobertura por departamento en "Resultados y avance" y registros en el mapa por departamento |
| Investigación de precios con catálogos en línea; media, mediana y moda; informe técnico | DPIR | Hecho: precio de referencia desde cotizaciones y, ahora, informe imprimible desde precios publicados. Falta Guatecompras (bloqueado desde la nube) |
| Proyecciones por territorio; cambios entre censos; limpiar códigos especiales; no identificar personas | DCE | Hecho salvo la comparación entre censos: población 2015–2030 por municipio, sexo y edad (pirámides), valores "sin dato", salida sin datos personales. Los censos 2002 y 1994 por municipio no están en formato descargable |
