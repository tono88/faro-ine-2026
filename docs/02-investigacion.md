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

**Universidad Rafael Landívar.** No se localizó en línea el estudio específico de techos de
vivienda con imágenes satelitales y proyección de crecimiento. Lo más cercano encontrado:

- *Perfil urbano de Guatemala. Análisis del crecimiento de áreas urbanas*, coordinado por el
  Instituto de Investigación y Proyección sobre Ciencia y Tecnología (Incyt-URL): analiza aumento
  de población y expansión territorial e incluye modelos predictivos.
  <https://biblioteca.url.edu.gt/vrip/galeria-libros-incyt/>
- A. Alonso (Iarna-URL), "Información satelital como herramienta para el diseño habitable
  climático omnicomprensivo": usa temperatura satelital (GLDAS, 42 años), no techos.
  <https://sie.url.edu.gt/wp-content/uploads/publicaciones/ensayos_ri/2025/ensri-2025-cuad-cedc_inf_satelit_dis_habit-aar.pdf>

Pendiente: confirmar con la universidad el título y autores del estudio de techos.

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

## 4. Pendiente de investigar (días 2 y 3)

- Asignación de operadores: problema de asignación (algoritmo húngaro) y variantes con capacidad.
- Completar red vial: predicción de enlaces en grafos y comparación con OpenStreetMap.
