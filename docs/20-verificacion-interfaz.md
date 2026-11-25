# Verificación de interfaz por tamaño de pantalla

Este informe lo escribe `.faro/pruebas/verificacion_interfaz.js` recorriendo el portal en un navegador
real (Chromium) en cada tamaño. Repetir: `node .faro/pruebas/verificacion_interfaz.js http://localhost:8082`.

Qué se mide en **cada sección** y en **cada pantalla**:

- **Desborde**: la página no debe necesitar desplazamiento horizontal.
- **Elementos que se salen** del ancho visible sin estar dentro de una zona con desplazamiento propio (las tablas anchas sí lo tienen).
- **Texto pequeño**: ningún texto de menos de 11 px.
- **Controles bajos**: botones, campos y listas de al menos 30 px de alto en pantallas táctiles (22 px con ratón).
- **Clic o toque real en el mapa**: debe abrir la ficha.
- **Errores de JavaScript**.

| Pantalla | Secciones recorridas | Páginas con desborde | Elementos que se salen | Textos pequeños | Controles bajos | Mapa | Errores JS | Resultado |
|---|---|---|---|---|---|---|---|---|
| Laptop 15.6" HD (1366×768) | 24 | 0 | 0 | 0 | 0 | abre la ficha | 0 | bien |
| Laptop 15.6" Full HD (1920×1080) | 24 | 0 | 0 | 0 | 0 | abre la ficha | 0 | bien |
| Tableta vertical (768×1024) | 24 | 0 | 0 | 0 | 0 | abre la ficha | 0 | bien |
| Tableta horizontal (1024×768) | 24 | 0 | 0 | 0 | 0 | abre la ficha | 0 | bien |
| Celular (390×844) | 24 | 0 | 0 | 0 | 0 | abre la ficha | 0 | bien |
| Celular pequeño (360×740) | 24 | 0 | 0 | 0 | 0 | abre la ficha | 0 | bien |

No quedó ningún hallazgo abierto.

## Capturas

Mapa, tableros y precios por rubro en laptop de 15.6", tableta y celular: `entregables/capturas/interfaz/`.

## Criterios de diseño que se comprueban

- Laptop de 15.6": menú lateral fijo y contenido a 1,180 px como máximo, para que las líneas no sean demasiado largas en Full HD.
- Tableta y celular (980 px o menos): el menú pasa a un botón; a 700 px o menos todo va en una columna, la ficha del mapa es una hoja inferior y los campos usan 16 px para que el teléfono no amplíe la página al escribir.
- Las tablas anchas se desplazan dentro de su propio marco, con el encabezado fijo.
- Colores y tipografía de la guía de diseño del INE (#0080FF, #004080, Ubuntu Sans); el foco del teclado siempre es visible.

<!-- equipo -->
---

<p align="center"><img src="identidad/equipo/carta-de-presentacion.svg" alt="Equipo La Sociedad de Almas: Estuardo Sandoval, Ana Echeverría y Alejandro Sandoval, con el código QR del perfil de LinkedIn de cada uno" width="760"></p>

<p align="center"><a href="https://www.linkedin.com/in/estuardoantoniosandoval/">Estuardo Sandoval</a> · <a href="https://www.linkedin.com/in/analuciaecheverria/">Ana Echeverría</a> · <a href="https://www.linkedin.com/in/alejandro-jos%C3%A9-sandoval-acevedo-998781aa/">Alejandro Sandoval</a><br>Equipo La Sociedad de Almas · Datatón INE 2026</p>
