# Investigación y referencias

Consultado el 23 de noviembre de 2026.

## 1. Integración de registros administrativos y X-Road

- El Organismo Ejecutivo hizo obligatoria la adopción de X-Road con el Oficio Circular 004-2026
  (22 oct 2026). La Comisión Presidencial de Gobierno Abierto y Electrónico (GAE) acompaña la
  implementación con guías y una hoja de ruta.
  <https://gae.gob.gt/gobierno-de-guatemala-implementa-x-road-en-el-sistema-publico/>
- Del 18 al 20 de noviembre de 2026 la GAE realizó un taller de X-Road con más de 60
  instituciones del Ejecutivo.
  <https://gae.gob.gt/mas-de-60-instituciones-publicas-se-articulan-para-transformar-la-interoperabilidad-en-guatemala/>
- X-Road resuelve el **transporte** seguro entre instituciones (quién habla con quién, firma,
  bitácora). No resuelve el **contenido**: qué columnas se piden, con qué nombres y cómo se
  llaman los lugares. Ese es el espacio de FARO.
- Marco conceptual citado por el INE: CEPAL (2023), *Documento metodológico para el
  aprovechamiento estadístico de registros administrativos económicos*.
- El INE ya coordina Oficinas Coordinadoras Sectoriales de Estadística (salud, educación,
  seguridad, ambiente, migración) para armonizar conceptos y estándares: son el canal natural
  para acordar las solicitudes de datos.

## 2. Crecimiento urbano a partir de imágenes

**Universidad Rafael Landívar.** La investigadora es Michelle Tercero (Iarna-URL; temas urbanos,
planificación y geoinformática).

- Charla "Monitoreo de crecimiento de la mancha urbana y su aplicación en el Observatorio Urbano
  de Guatemala". <https://www.youtube.com/watch?v=IooJRPRnhXc>
- Perfil de investigadora: <https://www.researchgate.net/profile/Michelle-Tercero>
- *Perfil urbano de Guatemala. Análisis del crecimiento de áreas urbanas* (Incyt-URL): aumento de
  población, expansión territorial y modelos predictivos.
  <https://biblioteca.url.edu.gt/vrip/galeria-libros-incyt/>

Pendiente: ver la charla completa y contactarla para conocer el método de techos y si los datos
son reutilizables.

**Trabajos similares y datos abiertos utilizables.**

- **Google Open Buildings 2.5D Temporal**: presencia, conteo y altura de edificaciones por año,
  2016–2023, derivado de Sentinel-2, con cobertura de América Latina. Licencia CC BY 4.0 / ODbL.
  Permite medir crecimiento sin entrenar un modelo propio.
  <https://sites.research.google/gr/open-buildings/temporal>
- **WorldPop** usó ese conjunto para calcular conteos, alturas y superficie construida y
  estudiar expansión urbana donde el censo está desactualizado. <https://www.worldpop.org/?p=14919>
- **Burkina Faso, censo 2019**: edificaciones vistas por satélite más datos demográficos para
  estimar población en zonas no censadas (Darin et al., *Population*, 2022).
  <https://eprints.soton.ac.uk/473749/>
- **Colombia**: cartografía social más cobertura de edificaciones por satélite para estimaciones
  poscensales en zonas de difícil acceso (Sanchez-Cespedes et al.).
- **Centroamérica**: metodología con Landsat 5, 7 y 8 para medir la mancha urbana de las
  capitales. <https://www.researchgate.net/publication/338390826>
- **BID**: estimación de déficit habitacional combinando censo y luminosidad satelital.
  <https://www.iadb.org/es/quienes-somos/topicos/desarrollo-urbano-y-vivienda/estimacion-del-deficit-habitacional>

## 3. Precios de referencia

- La DPIR calcula precios de referencia con cotizaciones de mercado y fuentes secundarias
  (históricos de Guatecompras), usando media, mediana y moda, y recomienda que más del 50 % de
  las fuentes sean cotizaciones del mercado relevante.
- Guatecompras publica datos en estándar OCDS, reutilizables para precios históricos adjudicados.

## 3a. Idiomas de la pantalla (día 3)

Pregunta: ¿se puede ofrecer la pantalla en kaqchikel sin inventar la traducción?

- **Fuente con términos revisados por hablantes y licencia abierta:** la localización comunitaria
  de Firefox al kaqchikel (`cak`), de Mozilla (MPL 2.0). Sus archivos comparten identificador de
  mensaje con las versiones en español e inglés, así que cada término se puede alinear y citar.
  Se revisaron 3,470 cadenas (Firefox y Mozilla Accounts).
- **Lo que no se halló** (búsqueda del 25-nov): MediaWiki no tiene la interfaz en kaqchikel en su
  repositorio; Common Voice y el sitio de complementos de Mozilla no tienen esa localización.
  En esa primera búsqueda no se revisaron los materiales de la Academia de Lenguas Mayas de
  Guatemala; se hizo en la segunda (abajo).
- **Segunda búsqueda (el mismo día), para ampliar el borrador:** la **memoria de traducción
  completa** de Mozilla en Pontoon (59,421 frases en kaqchikel, 53,261 alineadas con el español) y
  el *Kaqchikel Choltzij: Kolon chuqa' K'ak'a' taq Tzij* de la Comunidad Lingüística Kaqchikel de
  la ALMG (2011), con secciones de ambiente social, matemáticas e informática; además, la Ley de
  Idiomas Nacionales y la Ley de Acceso a la Información traducidas por la ALMG. Un diccionario
  jurídico y un vocabulario de matemáticas de la ALMG son escaneos sin texto: quedan por leer.
  Con eso se armó un borrador de 680 rótulos (`29`, sección 4).
- **Resultado de la primera búsqueda:** 32 rótulos de FARO coinciden palabra por palabra con un término de esa fuente. Los
  términos propios de la estadística (población, municipio, precio, mapa) no están; no se
  inventaron. Marco legal revisado: Ley de Idiomas Nacionales, Decreto 19-2003, artículos 8, 10,
  11, 14 y 15 (`23`, sección 2.14).
- **Regla que se siguió:** traducir la interfaz y dejar los nombres oficiales, los valores y las
  cifras como los publica la fuente.

Detalle: [`29-idiomas-e-inclusion.md`](29-idiomas-e-inclusion.md).

## 4. Lo que quedó por investigar el día 1, y en qué terminó

- Asignación de operadores: problema de asignación (algoritmo húngaro) y variantes con capacidad.
  **Hecho** el día 2: `crates/faro-territorio/src/asignacion.rs`, con capacidad por puesto.
- Completar red vial: predicción de enlaces en grafos y comparación con OpenStreetMap. **Hecho** el
  día 2 con un criterio más simple y explicable: componentes conexos y el hueco más corto hasta la
  red principal (`vial.rs`); no se usó predicción de enlaces.
