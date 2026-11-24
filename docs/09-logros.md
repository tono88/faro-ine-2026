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

## Operadores de campo (necesidad planteada por la Gerencia)

7. **Asignación con la menor distancia total posible**, calculada de forma exacta. Con el ejemplo
   ficticio de Sololá (19 operadores, 7 puestos) el total baja de 305 km a 163 km frente a repartir
   en el orden de las listas. Las cifras del ejemplo no son del INE; el método sí está probado
   contra todos los repartos posibles en tablas pequeñas.

## Lo que ya no hay que hacer a mano

8. **Lo que una persona decide una vez no se vuelve a preguntar**, sea un lugar ("El puerto" es San
   José, Escuintla), un código propio de una institución o un valor de catálogo ("Motobici" es
   motocicleta). En la prueba de punta a punta, el reenvío del mismo archivo pasó de 70 % a 100 %
   de lugares resueltos sin intervención.
