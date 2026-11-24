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
- Se entra con usuario y contraseña; tres roles; la bitácora registra a quien decidió.

## 8. Trazabilidad

- Cada descarga queda con dirección, fecha, licencia y huella.
- Cada decisión queda con quién y cuándo.
- Cada cifra tiene su [cita](11-fuentes-y-citas.md).

## Cifras

| | |
|---|---|
| Pruebas automáticas | 69 |
| Comandos de `faro` | 19 |
| Secciones de la pantalla | 21 |
| Tareas de muestreo de precios | 26 |
| Precios publicados en el levantamiento | 85,907 (18 tiendas, 6 rubros) |
| Capas publicadas | 15 |
| Fuentes citadas | 24 |
| Hallazgos documentados | [12](09-logros.md) |
