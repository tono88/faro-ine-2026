# Qué se ha hecho

Descripción breve de lo construido durante el Datatón INE 2026 (23–24 de noviembre), en el orden
en que una persona del INE lo usaría. Cada punto dice dónde verlo.

## 1. Pedir, recibir y homologar registros administrativos

- **Solicitud de datos.** El INE declara una vez qué necesita de cada institución. Hay solicitudes
  para RENAP (defunciones, nacimientos), PNC (detenidos) y SE-CONRED (emergencias).
- **Recepción.** El archivo llega como la institución lo tiene. FARO ubica el encabezado, reconoce
  columnas con otro nombre y reporta los problemas fila por fila.
- **Homologación sin adivinar.** Lugares y valores (sexo, pueblo y otros) se llevan al código
  oficial solo cuando la coincidencia es exacta y única. Lo dudoso se propone con candidatos; una
  persona decide y la decisión queda aprendida para todas las instituciones.
- **Datos personales.** Solo salen las columnas pedidas; los nombres se eliminan y el documento de
  identidad se sustituye por una llave no reversible.
- Probado con archivos reales de cuatro instituciones: 51,719 filas en menos de 3 segundos.

Dónde: pantalla "Recibir envío", "Revisar pendientes", "Equivalencias", "Resultados y avance".

## 2. Un catálogo oficial verificado

22 departamentos, 340 municipios y 20,036 lugares poblados del Censo 2018, comprobados código por
código contra el listado del INE, con coordenadas.

## 3. Población, tasas y crecimiento

- Proyecciones del INE por municipio, 2015–2030, como denominador de tasas por 100 mil habitantes.
- El cálculo se puede repetir desde la pantalla: consulta el portal, descarga, procesa y compara.
- Superficie construida por municipio de 1975 a 2020 con la capa satelital abierta GHSL.

Dónde: pantalla "Población y crecimiento" y el mapa.

## 4. Red vial y operadores de campo

- Caminos de OpenStreetMap tratados como grafo: 707 lugares poblados sin conexión a la red
  principal y 83 enlaces que probablemente faltan en el mapa.
- Asignación de operadores a puestos con la menor distancia total posible.

## 5. Precios

- Precio de referencia a partir de cotizaciones, con la regla de la Dirección de Precios.
- Materiales de construcción: la canasta oficial del IPMC (89 artículos) como estándar; 116
  proveedores perfilados y 2,332 precios publicados, homologados contra ella.
- Búsqueda en los 4,990 precios de referencia que el INE ya emitió y publica en su portal.
- Canasta básica: precio promedio del INE de 34 productos, junto a precios publicados.
- Precios por rubro: 85,907 precios publicados por 18 tiendas; el mismo producto entre tiendas por
  código de barras; diferencias de 50 % o más explicadas por regla (oferta, tienda atípica) o
  confirmadas por una persona; informe de precio de referencia imprimible; comparación por foto.
- Hoja de Balance de Alimentos: derivados con las tasas de la FAO para Guatemala, contrastadas con el INE.
- Muestreos automáticos configurables: el portal recoge solo los precios publicados por tiendas
  en línea, con su dirección y fecha.

## 6. Mapa, capas y proceso

- Un mapa por municipio con todos los indicadores; al tocar uno, su ficha con gráficas, la
  animación de su crecimiento y un reporte para imprimir.
- Junto a cada resultado, el procedimiento descargable que lo reproduce fuera del portal.
- Capas en GeoJSON y CSV con dirección estable para un portal o un SIG.
- El flujo de trabajo por etapas y responsables, con cifras en vivo ([proceso](12-proceso.md)).

## 7. Datos de otras instituciones y cuentas de usuario

- Inseguridad alimentaria aguda por departamento (CIF) y desnutrición crónica en escolares por
  municipio (Censo de Talla 2024), ambas de SESAN, verificadas y citadas.
- Se entra con usuario y contraseña; cuatro roles (administrador, analista, consulta y operador de
  campo); una bitácora registra quién hizo qué y cuándo.

## 8. Tableros, grupos y verificación

- Tableros: de una frase a un tablero que se edita en un lienzo, se filtra por territorio y
  condiciones, y se publica con un enlace que abre cualquier visitante.
- Grupos de usuarios: cada quien entra solo a las secciones de su grupo.
- Tres verificaciones con informe: estadística, de interfaz (laptop, tableta, celular) y de seguridad.

## 8a. Trabajo de campo

- Rutas por operador con las paradas ordenadas; cada operador ve la suya en su teléfono.
- Registro de visita con la posición que entrega el teléfono y enlace a la encuesta de Microsoft
  Forms con el lugar, la posición y el número de visita ya escritos.
- Cruce de cada visita con cuatro evidencias: posición, foto (¿vivienda o comercio?), superficie
  construida del kilómetro y techo más frecuente del lugar según el censo.
- La foto se clasifica con un modelo de escenas que corre en el servidor y no se guarda.

## 8b. Más datos del censo y del satélite

- Quince indicadores de condiciones de vida por municipio y material del techo por lugar poblado.
- Superficie residencial y no residencial por municipio y por kilómetro.
- Animación de la mancha urbana sobre la foto satelital (Landsat y Sentinel-2).
- Población después de 2030 con dos curvas no oficiales: exponencial y logística.

## 8c. Cumplimiento

- Protección estadística: los conteos menores que 3 no se publican a quien no es analista.
- Análisis de conformidad legal, norma por norma ([`23`](23-conformidad-legal.md)).
- Comparativo «antes y ahora» de 23 puntos ([`22`](22-antes-y-ahora.md)).

## 9. Trazabilidad

- Cada descarga queda con dirección, fecha, licencia y huella.
- Cada decisión queda con quién y cuándo.
- Cada cifra tiene su [cita](11-fuentes-y-citas.md).

## Cifras

| | |
|---|---|
| Pruebas automáticas | 92 |
| Comandos de `faro` | 29 |
| Secciones de la pantalla | 24 |
| Indicadores por municipio | 26 |
| Tareas de muestreo de precios | 26 |
| Precios publicados en el levantamiento | 85,907 (18 tiendas, 6 rubros) |
| Capas publicadas | 19 |
| Fuentes citadas | 34 |
| Hallazgos documentados | [33](09-logros.md) |
