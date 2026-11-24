# Logros y hallazgos

Lo que FARO encontró o resolvió con datos reales durante el evento. Cada punto dice de dónde sale
y cómo repetirlo.

## Hallazgos en datos abiertos del INE

1. **El libro de proyecciones de población trae 16 municipios con un código que no existe.**
   Las "Estimaciones y proyecciones de la población total por municipio, 2015–2030" codifican 16
   municipios del departamento de Guatemala como `0123`–`0138` en vez de `0102`–`0117` (entre
   ellos Mixco y Villa Nueva). El diccionario del propio libro los nombra bien. FARO lo detectó al
   cruzar con el catálogo y los reubicó por nombre, con coincidencia exacta y única. Quien cruce
   el libro original por código pierde esos 16 municipios.
   Detalle: [`datos/referencia/LEEME.md`](../datos/referencia/LEEME.md). Repetir: `faro poblacion`.

2. **El catálogo de 340 municipios coincide código por código con el Censo 2018**, y en 25 el
   nombre de uso común no es el oficial (por ejemplo "Nebaj" es "Santa María Nebaj"). FARO guarda
   ambos, de modo que cualquiera de los dos se reconoce. 313 de 20,036 lugares poblados se
   publican sin coordenadas.
   Detalle: [`datos/catalogo/LEEME.md`](../datos/catalogo/LEEME.md). Repetir: `faro catalogo verificar`.

## Hallazgos en registros administrativos (archivos compartidos por el INE)

3. **Cada institución escribe lo mismo de otra forma**, y hay errores que hoy pasan inadvertidos:
   códigos de municipio que no existen, municipios en el departamento equivocado, letras dañadas
   por la codificación, categorías fuera de catálogo. 51,719 filas de cuatro instituciones
   procesadas en menos de 3 segundos, con 97.9 % a 100 % de lugares con código oficial.
   Detalle (solo conteos): [`06-hallazgos-datos-reales.md`](06-hallazgos-datos-reales.md).

4. **De 82 columnas con datos personales salen 14**: el nombre se elimina, el CUI se sustituye por
   una llave no reversible que permite parear sin identificar, y las direcciones nunca pasan.

## Red vial (necesidad planteada por la Gerencia)

5. **707 lugares poblados del censo no quedan conectados a la red de caminos del mapa.** De 19,723
   lugares con coordenadas, 163 están en un tramo de camino que no se une a la red principal y 544
   no tienen ninguna vía para vehículos a 500 m. Alta Verapaz, Izabal y Petén concentran la mayoría.

6. **83 enlaces que probablemente faltan, 51 km en total.** Para cada tramo suelto, el grafo da el
   hueco más corto hasta la red principal. El primero de la lista: 843 km de caminos en San Andrés,
   Petén, de los que dependen 28 lugares poblados, separados de la red por un hueco de 143 metros.
   Un hueco puede ser un camino sin dibujar o un paso en lancha: la lista dice dónde verificar.
   Detalle: [`datos/referencia/LEEME.md`](../datos/referencia/LEEME.md). Repetir: `faro vial`.
   Fuente: © colaboradores de OpenStreetMap (ODbL), corte del 3-nov-2026.

## Límites, crecimiento urbano y mapa

7. **La capa oficial de límites trae intercambiados los códigos de Sanarate y Sansare**, y dos
   figuras que son lagos. FARO lo detectó ubicando los lugares poblados del censo dentro de cada
   figura y lo corrigió. Además, **13 % de los lugares poblados cae fuera de la figura de su
   municipio**: en 137 municipios el límite y el censo no coinciden en más de uno de cada diez lugares.
   Detalle: [`datos/referencia/LEEME.md`](../datos/referencia/LEEME.md). Repetir: `faro limites`.

8. **Serie real de crecimiento urbano por municipio, sin Earth Engine.** Con la capa abierta GHSL
   de la Comisión Europea, la superficie construida del país pasa de 30,722 hectáreas en 1975 a
   93,297 en 2020; hay cifra para los 340 municipios cada cinco años y se calcula en 8 segundos.
   Detalle: [`10-imagenes-satelitales.md`](10-imagenes-satelitales.md). Repetir: `faro construido`.

9. **Un mapa y capas publicables.** Población, crecimiento, superficie construida, conexión vial y
   registros recibidos por municipio en un mismo mapa, y siete capas con dirección estable
   (GeoJSON y CSV) para que las consuma un portal o un SIG.

## Datos de otras instituciones de gobierno

- **Inseguridad alimentaria aguda (CIF, SESAN) por departamento, incorporada y verificada.** FARO
  leyó la tabla del informe oficial de noviembre de 2026, comprobó sus sumas y encontró que su
  población coincide exactamente, en los 22 departamentos, con la proyección del INE para 2026:
  una confirmación independiente de la tabla de población ya corregida.
  Detalle: [`datos/referencia/LEEME.md`](../datos/referencia/LEEME.md). Repetir: `faro cif`.

- **Desnutrición crónica en escolares por municipio (Censo de Talla 2024, SESAN).** Los 340
  municipios del libro cruzan con el catálogo por código y por nombre. El dato va al mapa, a la
  ficha de cada municipio y a una capa publicable. Repetir: `faro talla`.

## Precios de materiales de construcción

10. **116 proveedores perfilados y 2,332 precios publicados, comparados contra la canasta oficial
    del IPMC.** Con reglas para 42 de los 89 artículos, FARO declara comparables solo 70 precios
    (coinciden palabras, medidas y presentación), deja 563 para que decida una persona y aparta 99
    por ser de otra medida. Con eso hay precio para 20 artículos; por ejemplo, el saco de cemento UGC de 42.5 kg
    está entre Q81.00 y Q83.04 en tres proveedores. La cifra baja de comparables es deliberada: el
    sistema no compara lo que no puede comprobar.
    Detalle: [`datos/referencia/LEEME.md`](../datos/referencia/LEEME.md). Repetir: `faro construccion`.

## Precios publicados por rubro

10a. **26,976 precios publicados por 11 tiendas en seis rubros, leídos por el propio portal** el
    24-nov-2026 (supermercado 12,330; ferretería 8,278; tecnología 3,107; hogar 2,050; farmacia
    1,100; vehículos 111). Cada precio guarda la dirección de su página y la fecha.
10b. **El código de barras permite comparar el mismo producto entre tiendas sin interpretar nombres.**
    En supermercados, 3,137 productos aparecen con el mismo código en dos o más tiendas y 505 en las
    cuatro (Walmart, Paiz, Maxi Despensa, La Torre). En 717 el precio es idéntico; la diferencia
    mediana entre la tienda más cara y la más barata es 4.6 %, y en 665 productos es de 15 % o más.
    Es la misma regla del núcleo —coincidencia exacta o decide una persona— aplicada a precios.
    Tabla: [`datos/referencia/precios_publicados.csv`](../datos/referencia/precios_publicados.csv).
    Repetir: sección "Muestreos automáticos" → "Ejecutar ahora".
10c. **Tres supermercados conocidos no publican catálogo con precios** (Suma, Super del Barrio,
    Mi Super Fresh: solo páginas informativas) y PriceSmart exige navegador con sesión; no se muestrean.

## Operadores de campo (necesidad planteada por la Gerencia)

11. **Asignación con la menor distancia total posible**, calculada de forma exacta. Con el ejemplo
   ficticio de Sololá (19 operadores, 7 puestos) el total baja de 305 km a 163 km frente a repartir
   en el orden de las listas. Las cifras del ejemplo no son del INE; el método sí está probado
   contra todos los repartos posibles en tablas pequeñas.

## Lo que ya no hay que hacer a mano

12. **Lo que una persona decide una vez no se vuelve a preguntar**, sea un lugar ("El puerto" es San
   José, Escuintla), un código propio de una institución o un valor de catálogo ("Motobici" es
   motocicleta). En la prueba de punta a punta, el reenvío del mismo archivo pasó de 70 % a 100 %
   de lugares resueltos sin intervención.
