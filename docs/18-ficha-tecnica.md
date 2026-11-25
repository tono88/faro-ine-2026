# Ficha técnica

| | |
|---|---|
| **Nombre** | FARO · recepción y homologación de registros administrativos |
| **Lema** | Que cada dato llegue a puerto |
| **Equipo** | La Sociedad de Almas: Estuardo Sandoval, Ana Echeverría y Alejandro Sandoval |
| **Misión** | Dar al INE una forma única, verificable y respetuosa de las personas para pedir, recibir y entender los registros de otras instituciones, y convertirlos en estadística útil. Identidad completa: `docs/25-identidad.md` |
| **Categoría** | 2 · Tecnológica y ciencias de datos (Datatón INE 2026) |
| **Qué es** | Herramienta web y de línea de comandos para que el INE pida, reciba y homologue registros de otras instituciones, y los aproveche junto a sus datos abiertos de territorio y precios |
| **Problemas que atiende** | Datos que ya existen pero no llegan en forma utilizable: registros sin estándar de envío y con datos personales; avance de recepción sin seguimiento; crecimiento urbano sin medir entre censos; caminos sin mapear; operadores lejos de su puesto y visitas sin constancia; censo disperso en decenas de cuadros; precios difíciles de reunir y de comparar; falta de tableros y calculadoras con software libre; errores en fuentes oficiales que nadie avisa |
| **Principio** | El sistema no adivina: asigna solo con coincidencia exacta y única o con una equivalencia confirmada por una persona; lo demás lo propone |
| **Bases públicas del INE usadas** | Lugares poblados y cuadros del Censo 2018; proyecciones de población 2015–2030; IPMC; precios de referencia emitidos; canasta básica alimentaria; Hoja de Balance de Alimentos 2024; diccionario de estadísticas vitales; catálogo del portal de datos abiertos |
| **Otras fuentes abiertas** | OpenStreetMap (HOT/HDX), límites COD-AB (HDX), GHSL (Comisión Europea), SESAN (CIF y Censo de Talla), FAO (coeficientes técnicos), mosaicos Landsat (NASA GIBS) y Sentinel-2 (EOX), precios publicados por 18 tiendas en línea. Citas: `docs/11-fuentes-y-citas.md` |
| **Tecnología** | Rust (núcleo, servidor y CLI), PostgreSQL 16, HTML y JavaScript sin marcos, Leaflet; Docker Compose |
| **Servicios de pago o en la nube** | Ninguno. Opcionales y gratuitos, pedidos por el navegador: mapa base de OpenStreetMap y fotos satelitales de NASA y EOX. La encuesta de campo se abre en Microsoft Forms del INE, si se configura. El desarrollo usó un asistente de IA para programar (declarado en el README) |
| **Modelos de IA en ejecución** | Uno, local: el clasificador de fotos del trabajo de campo (Places365-ResNet18, MIT CSAIL, pesos CC BY), que corre dentro del servidor y solo marca visitas para revisión. Todo lo demás son reglas deterministas y a la vista |
| **Protección estadística** | Solo agregados; conteos menores que 3 ocultos para quien no es analista, con supresión complementaria; bitácora de acciones. Análisis de conformidad legal: `docs/23-conformidad-legal.md` |
| **Instalación** | `docker compose up -d --build` → <http://localhost:8082>. Detalle: `docs/15-instalacion-y-dependencias.md` |
| **Reproducibilidad** | Cada tabla derivada tiene un comando `faro …` (y las principales, además, un procedimiento descargable desde el portal); las descargas quedan en un manifiesto versionado con fecha y huella SHA-256. La instalación se probó desde un clon limpio |
| **Verificación** | Estadística (`docs/19`), de interfaz en laptop de 15.6", tableta y celular (`docs/20`) y de seguridad (`docs/21`), las tres con informe regenerable |
| **Acceso** | Usuarios con rol (administrador, analista, consulta, operador de campo e institución) y grupo por secciones; una cuenta por institución que solo sube sus archivos; contraseñas con Argon2; bitácora de acciones a la vista de los administradores |
| **Idiomas e inclusión** | Pantalla en español, inglés (completo) y kaqchikel (borrador: 680 rótulos, 36 % de la pantalla, armados con vocabulario escrito por hablantes y pendientes de revisión experta). Se traduce la interfaz, nunca los datos. Un idioma es un archivo; sin servicio de traducción. Funciona en teléfono; recorridos guiados en todas las secciones. Detalle: `docs/29-idiomas-e-inclusion.md` |
| **Pruebas** | 96 pruebas automáticas (`cargo test`) y recorridos de la pantalla en Chromium, incluidos teléfono con posición simulada y clic real en el mapa |
| **Datos personales** | Ninguno en el repositorio; los archivos reales se procesan en un temporal y no se guardan |
| **Limitaciones conocidas** | Ver la tabla de estado en `docs/01-necesidades-ine.md` y los avisos de cada sección |
| **Licencia** | Autoría del equipo; licencia no exclusiva, gratuita, perpetua e irrevocable al INE (`LICENCIA.md`). Componentes de terceros con su licencia: `docs/27-componentes-de-terceros.md` |
| **Conformidad con la nota conceptual** | Lineamiento por lineamiento y criterio por criterio: `docs/26-conformidad-nota-conceptual.md` |
| **Entregables** | Repositorio; presentación (`entregables/FARO-presentacion.pdf`); manual de usuario gráfico; documento técnico; video de 2 a 3 minutos (guion en `docs/16-guion-video.md`) |
| **Repositorio** | <https://github.com/tono88/faro-ine-2026> (historial desde el 23-nov-2026) |

<!-- equipo -->
---

<p align="center"><img src="identidad/equipo/carta-de-presentacion.svg" alt="Equipo La Sociedad de Almas: Estuardo Sandoval, Ana Echeverría y Alejandro Sandoval, con el código QR del perfil de LinkedIn de cada uno" width="760"></p>

<p align="center"><a href="https://www.linkedin.com/in/estuardoantoniosandoval/">Estuardo Sandoval</a> · <a href="https://www.linkedin.com/in/analuciaecheverria/">Ana Echeverría</a> · <a href="https://www.linkedin.com/in/alejandro-jos%C3%A9-sandoval-acevedo-998781aa/">Alejandro Sandoval</a><br>Equipo La Sociedad de Almas · Datatón INE 2026</p>
