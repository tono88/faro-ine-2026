# Versiones

Cada versión es un release de GitHub sobre el historial de commits.

## v0.4.0 · 24-nov-2026
- Precios de referencia a partir de cotizaciones (media, mediana, moda; precios extremos;
  proporción de cotizaciones directas), en línea de comandos y en la pantalla.

## v0.3.0 · 24-nov-2026
- Población por municipio 2015–2030 (proyecciones del INE) y tasas por 100 mil habitantes.
- Hallazgo: 16 códigos de municipio inexistentes en el libro de proyecciones, reubicados con su diccionario.
- Equivalencias de valores de catálogo (sexo, pueblo, tipo de vehículo...) con memoria por variable.
- Correcciones: letras dañadas, códigos propios de una institución, lugares de otro departamento,
  columna de respaldo, valores "sin dato".
- Pantalla: lista del catálogo para elegir el lugar, avance de recepción por institución,
  crecimiento poblacional, archivos de ejemplo descargables.
- Puerto por defecto 8082.

## v0.2.0 · 23-nov-2026
- Despliegue con Docker Compose probado; `iniciar.bat` busca un puerto libre.
- Catálogo verificado contra el Censo 2018: 340 municipios código por código y 20,036 lugares
  poblados con coordenadas.
- Asignación de operadores de campo con distancia total mínima (método húngaro).

## v0.1.0 · 23-nov-2026
- Núcleo de homologación con memoria, solicitudes de datos, recepción de Excel/CSV reales con
  datos personales eliminados o seudonimizados.
- Cliente del portal de datos abiertos del INE con procedencia.
- Servidor con PostgreSQL y pantalla web; crecimiento de edificaciones y calidad del aire.
