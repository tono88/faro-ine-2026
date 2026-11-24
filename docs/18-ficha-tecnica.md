# Ficha técnica

| | |
|---|---|
| **Nombre** | FARO · recepción y homologación de registros administrativos |
| **Categoría** | 2 · Tecnológica y ciencias de datos (Datatón INE 2026) |
| **Qué es** | Herramienta web y de línea de comandos para que el INE pida, reciba y homologue registros de otras instituciones, y los aproveche junto a sus datos abiertos de territorio y precios |
| **Problema que atiende** | Las instituciones no tienen un estándar para compartir datos con fines estadísticos; el mismo lugar o producto llega escrito de muchas formas |
| **Principio** | El sistema no adivina: asigna solo con coincidencia exacta y única o con una equivalencia confirmada por una persona; lo demás lo propone |
| **Bases públicas del INE usadas** | Lugares poblados y cuadros del Censo 2018; proyecciones de población 2015–2030; IPMC; precios de referencia emitidos; canasta básica alimentaria; Hoja de Balance de Alimentos 2024; diccionario de estadísticas vitales; catálogo del portal de datos abiertos |
| **Otras fuentes abiertas** | OpenStreetMap (HOT/HDX), límites COD-AB (HDX), GHSL (Comisión Europea), SESAN (CIF y Censo de Talla), FAO (coeficientes técnicos), mosaicos Landsat (NASA GIBS) y Sentinel-2 (EOX), precios publicados por 18 tiendas en línea. Citas: `docs/11-fuentes-y-citas.md` |
| **Tecnología** | Rust (núcleo, servidor y CLI), PostgreSQL 16, HTML y JavaScript sin marcos, Leaflet; Docker Compose |
| **Servicios de pago o en la nube** | Ninguno. Opcionales y gratuitos, pedidos por el navegador: mapa base de OpenStreetMap y fotos satelitales de NASA y EOX. La encuesta de campo se abre en Microsoft Forms del INE, si se configura. El desarrollo usó un asistente de IA para programar (declarado en el README) |
| **Modelos de IA en ejecución** | Uno, local: el clasificador de fotos del trabajo de campo (Places365-ResNet18, MIT CSAIL, pesos CC BY), que corre dentro del servidor y solo marca visitas para revisión. Todo lo demás son reglas deterministas y a la vista |
| **Protección estadística** | Solo agregados; conteos menores que 3 ocultos para quien no es analista, con supresión complementaria; bitácora de acciones. Análisis de conformidad legal: `docs/23-conformidad-legal.md` |
| **Instalación** | `docker compose up -d --build` → <http://localhost:8082>. Detalle: `docs/15-instalacion-y-dependencias.md` |
| **Reproducibilidad** | Cada tabla derivada tiene un comando `faro …` y un procedimiento descargable; las descargas quedan en un manifiesto con fecha y huella SHA-256 |
| **Verificación** | Estadística (`docs/19`), de interfaz en laptop de 15.6", tableta y celular (`docs/20`) y de seguridad (`docs/21`), las tres con informe regenerable |
| **Acceso** | Usuarios con rol (administrador, analista, consulta, operador de campo) y grupo por secciones; contraseñas con Argon2; bitácora de acciones |
| **Pruebas** | 92 pruebas automáticas (`cargo test`) y recorridos de la pantalla en Chromium, incluidos teléfono con posición simulada y clic real en el mapa |
| **Datos personales** | Ninguno en el repositorio; los archivos reales se procesan en un temporal y no se guardan |
| **Limitaciones conocidas** | Ver la tabla de estado en `docs/01-necesidades-ine.md` y los avisos de cada sección |
| **Repositorio** | <https://github.com/tono88/faro-ine-2026> (historial desde el 23-nov-2026) |
