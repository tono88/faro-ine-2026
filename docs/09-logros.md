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
   por la codificación, categorías fuera de catálogo. 51,719 filas de tres instituciones
   (cuatro archivos) procesadas en menos de 3 segundos, con 99.9 % a 100 % de lugares con código
   oficial tras entender los códigos de RENAP.
   Detalle (solo conteos): [`06-hallazgos-datos-reales.md`](06-hallazgos-datos-reales.md).

4. **De 82 columnas con datos personales pasan 15** (se piden 16): el nombre se elimina, el CUI se sustituye por
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

6a. **El tipo de cada vía ya estaba en el archivo; la superficie, solo en parte.** De 144,846 km de
   vías para vehículos: 5,533 km principales, 15,496 km secundarias, 89,916 km locales y 33,901 km
   de terracería. La superficie está anotada en 38 % de la red; de esa parte, 21 % es pavimentada.
   (El grafo completo mide 144,866 km; la suma por clase da 20 km menos, un 0.01 %, y de esos
   kilómetros 430 no tienen un municipio confirmado contra el catálogo.)
   El 62 % restante no se puede clasificar: es trabajo de mapeo pendiente, y el portal lo dice
   municipio por municipio en vez de suponerlo. El código de municipio del archivo se aceptó solo
   cuando su nombre coincide con el catálogo (la capa de origen trae dos códigos intercambiados);
   quedaron 430 km sin asignar. Se buscó una capa de vías en el Registro de Información Catastral
   (RIC) y en la Dirección General de Caminos: no se halló ninguna de descarga abierta.
   Repetir: `faro vial`.

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
   registros recibidos por municipio en un mismo mapa, y 21 capas con dirección estable
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

10a. **85,907 precios publicados por 18 tiendas en seis rubros, leídos por el propio portal** el
    24-nov-2026 (supermercado 63,879; ferretería 9,355; tecnología 8,024; hogar 2,708; farmacia
    1,861; vehículos 80). Cada precio guarda la dirección de su página y la fecha.
10b. **El código de barras permite comparar el mismo producto entre tiendas sin interpretar nombres.**
    En supermercados, 16,291 productos aparecen con el mismo código en dos o más tiendas y 1,740 en
    las cuatro (Walmart, Paiz, Maxi Despensa, La Torre). En 6,580 el precio es idéntico; la
    diferencia mediana entre la tienda más cara y la más barata es 1.6 %, y en 2,756 productos es
    de 15 % o más. Es la misma regla del núcleo —coincidencia exacta o decide una persona—
    aplicada a precios.
    Tabla: [`datos/referencia/precios_publicados.csv`](../datos/referencia/precios_publicados.csv).
    Repetir: sección "Muestreos automáticos" → "Ejecutar ahora".
10c. **Tres supermercados conocidos no publican catálogo con precios** (Suma, Super del Barrio,
    Mi Super Fresh: solo páginas informativas) y PriceSmart exige navegador con sesión; no se muestrean.

10d. **Los códigos de RENAP son los del INE, partidos en dos, y solo valen cuando el país es Guatemala.**
    En sus archivos, el departamento va de 1 a 22 y el municipio es el número dentro del
    departamento: el máximo de cada departamento coincide con el número de municipios del catálogo
    del INE en los 22 (17, 8, 16, 16, 14, 14, 19, 8, 24, 21, 9, 30, 33, 21, 8, 17, 14, 5, 11, 11, 7, 17).
    Departamento 5 y municipio 9 es el 0509. Los «códigos que no existen» (37, 90, 608…) aparecían
    solo en filas cuya columna de país no es 320 (Guatemala): son divisiones de otro país, no
    municipios. La solicitud declara ahora la columna de país (`pais`) y `-1`/`0` como sin dato.
    Resultado con los archivos reales (solo conteos): nacimientos, 30,093 filas, 100.0 % de lugares
    resueltos y un único pendiente (municipio 30 en un departamento que tiene 24, 3 filas);
    defunciones, 10,034 filas, 100 % y ningún pendiente. Antes quedaban 387 valores por revisar.
10d2. **Un archivo mensual no trae solo ese mes.** El archivo de nacimientos inscritos en junio de 2026
    trae hechos ocurridos entre 1914 y 2026 (inscripciones tardías), y el de defunciones de julio,
    desde 1900. Si el reemplazo de un reenvío se hiciera por mes del hecho, el archivo de julio
    pisaría parte de junio. FARO reemplaza por mes de registro y suma los hechos de un mismo mes
    que llegan en archivos distintos.
10e. **Las tasas de extracción de la FAO para Guatemala siguen describiendo lo que el INE publica.**
    Con las cuentas de la Hoja de Balance de Alimentos 2024 (producción del derivado ÷ primario
    procesado): harina de trigo 72.0 % (FAO: 72), salvado de trigo 22.0 % (22), café tostado 80.0 %
    (80), azúcar en bruto 10.2 % (10), melazas 4.1 % (3.9), salvado de arroz 7.9 % (8). La que más
    se aparta es el arroz elaborado: 69.3 % frente a 62. Las tasas de la FAO son de 1992–1996.
    Repetir: `faro hba` y la sección "Hoja de balance: derivados".

10i. **Tres de cada cuatro diferencias grandes de precio entre tiendas son ofertas.** De 537 productos
    de supermercado con el mismo código de barras y 50 % o más de diferencia, en 411 (77 %) el
    precio regular de la tienda barata coincide con el de las demás; en 7 una sola tienda se aparta
    más de tres veces de la mediana mientras las otras coinciden (probable error de publicación,
    como un jabón a Q215.00 frente a Q11.69 y Q13.60); 119 no tienen regla y los confirma una persona.
10f. **La foto sola no basta para decir que dos productos son el mismo; el código de barras sí.**
    Medido con 300 productos de igual código de barras y fotos distintas en dos tiendas: la huella
    visual no aceptó ningún par de productos distintos, pero solo reconoció 29 de los 300 iguales
    (9.7 %). Sirve para detectar la misma imagen reutilizada, no el mismo producto fotografiado de
    otra forma. Detalle y tabla de umbrales: [`13-fotos.md`](13-fotos.md).

## Población y censo

10g. **El INE ya publica el Censo 2018 por lugar poblado, y cuadra con el catálogo.** Los cuadros
    A1, B8 y C1 traen 20,036 lugares cuyos códigos coinciden uno a uno con el catálogo de FARO:
    14,901,286 personas, 3,275,931 hogares y 3,943,085 viviendas; en todas las filas, sexo y edades
    suman el total. Las edades por municipio de las proyecciones suman exactamente los totales ya
    publicados en los 340 municipios y 3 años revisados.
10h. **No hay una fuente descargable para comparar con censos anteriores.** El Censo 2002 por
    municipio está publicado como PDF escaneado (sin texto) y sus microdatos están tras una
    protección contra descargas automáticas; del Censo 1994 no se halló nada descargable. Esa
    comparación queda pendiente y no se aproximó con otras fuentes.

## El portal se revisa a sí mismo

10j. **Una prueba hacia atrás corrigió el margen de las proyecciones.** Proyectando diez años con
    datos anteriores en 340 municipios y tres décadas (1,020 casos), lo observado caía dentro del
    margen que el portal mostraba solo 45 % de las veces. El margen se cambió por el error
    realmente observado (20 % de lo proyectado en 8 de cada 10 casos; 47 % en 95 de cada 100) y
    ahora cumple lo que dice. Ningún método alternativo (exponencial, tres épocas, última década)
    fue mejor en las tres décadas. Detalle: [`19-verificacion-estadistica.md`](19-verificacion-estadistica.md).
10k. **Interfaz comprobada en seis tamaños de pantalla y seguridad auditada.** 24 secciones sin
    desbordes ni controles difíciles de tocar en laptop, tableta y celular
    ([`20-verificacion-interfaz.md`](20-verificacion-interfaz.md)); auditoría sin inyección SQL ni
    fallos de rol, con 16 de 20 hallazgos corregidos ([`21-seguridad.md`](21-seguridad.md)).

10l. **El censo ya dice de qué son los techos, lugar por lugar.** El cuadro C2 del Censo 2018 trae
    el material del techo de las viviendas de 20,254 lugares poblados: 68.1 % de las viviendas del
    país tiene techo de lámina. No hace falta estimarlo desde el satélite, que además no puede:
    Landsat ve cuadros de 30 m. Repetir: `faro censo-indicadores`.
10m. **Quince indicadores de condiciones de vida por municipio, de cuadros que ya estaban descargados.**
    Alfabetismo 81.5 %, uso de internet 29.3 %, agua por tubería en la vivienda 58.9 %, drenaje
    44.9 %, red eléctrica 88.1 %, cocina con leña 54.4 %, piso de tierra 26.5 %, participación
    económica 50.6 %, educación media o superior 31.7 %, recolección de basura 41.9 %, vivienda
    propia 80.2 % y, de esas, 25.8 % con propietaria mujer. Los tres cuadros de hogares suman los
    mismos hogares, y el proceso se detiene si el INE cambia el orden de una columna.
10n. **Solo 1.6 % de lo construido es no residencial, y está concentrado.** Con el componente no
    residencial del GHSL: 1,486 de 93,297 hectáreas en 2020. Guatemala (296 ha), Villa Nueva (187),
    Mixco (73), Amatitlán (66) y Escuintla (56) encabezan; 118 municipios tienen al menos una
    hectárea. Repetir: `faro uso-suelo`.
10o. **Las curvas simples reproducen bien las proyecciones del INE a cinco años.** Con lo publicado
    hasta 2025, la exponencial estima 2030 con un error mediano de 1.05 % y la logística con 0.17 %
    (en los 302 municipios donde la serie deja ver un techo). Para 2050 se separan: 24.6 millones
    con la exponencial y 22.7 con la logística. Esa distancia es lo que pesa el supuesto.
    Detalle: [`19-verificacion-estadistica.md`](19-verificacion-estadistica.md).
10p. **Un modelo de escenas distingue vivienda de comercio sin equivocarse en lo que afirma.** En 16
    fotos de fachadas de Guatemala y Honduras: 11 aciertos, ningún error y 5 «no concluyente»
    (casas coloniales que le parecen iglesias u hostales). Lo dudoso no cuenta en contra de la
    visita. Detalle y límites: [`24-clasificador-de-fotos.md`](24-clasificador-de-fotos.md).
10q. **Los pendientes que «sobraban» eran de reglas anteriores.** Como el archivo original no se
    guarda, un envío analizado antes de un cambio de reglas conserva sus observaciones viejas. El
    portal ahora lo dice en cada pendiente y en cada envío, y basta volver a subir el archivo.
    Al probarlo apareció un fallo propio: la huella de las reglas cambiaba entre lecturas de la
    misma solicitud (sus tablas no tienen orden fijo) y todo envío salía marcado. Se corrigió
    ordenando las claves, con una prueba que lee la solicitud veinte veces.

## Operadores de campo (necesidad planteada por la Gerencia)

11. **Asignación con la menor distancia total posible**, calculada de forma exacta. Con el ejemplo
   ficticio de Sololá (19 operadores, 7 puestos) el total baja de 305 km a 163 km frente a repartir
   en el orden de las listas. Las cifras del ejemplo no son del INE; el método sí está probado
   contra todos los repartos posibles en tablas pequeñas.

11a. **De la asignación a la visita comprobada.** Cada operador ve su ruta con las paradas en
   orden (vecino más cercano y mejora 2-opt), registra la visita con la posición que entrega su
   teléfono y abre la encuesta de Microsoft Forms con el lugar ya escrito. En la prueba, una
   visita a 247 m del punto salió «coherente» y otra a 22 km, «conviene revisar», con la razón.
   Nada se rechaza solo: el supervisor decide.

11b. **Dos fallos del mapa que las pruebas no veían.** Las casillas de capas que el navegador deja
   marcadas al recargar no se dibujaban, y después de ver «Lugares poblados» el mapa dejaba de
   responder: un lienzo vacío quedaba encima y se quedaba con los clics. Los reportó el equipo al
   usar el portal; ahora hay una prueba con clic real que recorre esos pasos.

## Idiomas e inclusión

11c. **Traducir la pantalla sin tocar los datos, y demostrarlo.** Al recorrer el portal para armar
   la lista de textos, la herramienta separó 1,873 textos de interfaz y dejó fuera **2,394 que eran
   datos** (lugares, productos, tiendas). Solo los primeros entraron al diccionario. La prueba de navegador
   cambia de idioma y comprueba que los 22 nombres de departamento, las cifras y las 39 citas salen
   idénticos en español, inglés y kaqchikel.

11d. **El kaqchikel que se puede sostener cubre 1.7 % de la pantalla.** De 1,873 textos, solo 32
   coinciden palabra por palabra con un término ya publicado y revisado por hablantes (la
   localización de Firefox). Faltan sobre todo los términos de la estadística: población,
   municipio, precio, mapa. FARO muestra ese porcentaje en lugar de rellenar con una traducción
   automática que nadie puede comprobar.

11e. **Un fallo propio hallado al construirlo.** En una pasada, la herramienta que arma la lista
   de textos dejó entrar 167 nombres de productos como si fueran texto de la pantalla: una tabla
   terminó de cargar justo después de limpiar la cosecha anterior. Se corrigió leyendo cada
   cosecha en un solo paso, y `cargo test` rechaza cualquier entrada que ya no sea un texto de la
   pantalla.

11f. **Una palabra, dos sentidos.** «media» es el promedio de un precio y también el nivel medio
   de confianza de una proyección. Para no traducir mal una de las dos, la segunda lleva contexto
   (`confianza|media`).

Detalle: [`29-idiomas-e-inclusion.md`](29-idiomas-e-inclusion.md).

## Lo que ya no hay que hacer a mano

12. **Lo que una persona decide una vez no se vuelve a preguntar**, sea un lugar ("El puerto" es San
   José, Escuintla), un código propio de una institución o un valor de catálogo ("Motobici" es
   motocicleta). En la prueba de punta a punta, el reenvío del mismo archivo pasó de 70 % a 100 %
   de lugares resueltos sin intervención.
13. **Cada institución sube solo lo suyo.** Una cuenta atada a PNC no puede subir un archivo de
   RENAP ni ver sus envíos: el servidor responde 403, comprobado en la prueba
   `institucion_y_recorridos.js` junto con otros 30 puntos.
14. **El portal se explica solo.** 24 secciones con recorrido guiado propio; el manual gráfico
   (37 páginas) se genera desde el portal y usa las mismas marcas, así que no se desactualiza.
15. **La instalación se comprobó desde cero.** Un clon limpio del repositorio, los pasos del README
   y un comando: la imagen se construyó en 4 minutos y el portal quedó arriba con sus 85,907 precios
   cargados (25-nov-2026).
16. **Una revisión independiente encontró una resta posible, y se cerró.** El mapa ocultaba la celda
   pequeña y su complemento, pero la ficha del municipio solo ocultaba la pequeña: con el total del
   departamento se podía despejar. Ahora todas las respuestas usan la misma lista de celdas
   ocultas (`celdas_ocultas`). También se corrigieron dos licencias del INE mal declaradas
   (precios de referencia es ODC-BY; la canasta básica no declara licencia).
