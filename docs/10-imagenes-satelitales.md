# Crecimiento urbano con imágenes satelitales abiertas

Earth Engine pide plan de pago en algunos casos. Estas son las opciones abiertas que se evaluaron
el 24-nov-2026 (se comprobó que cada descarga responde) y la que FARO usa.

## Lo que usa FARO

**GHSL GHS-BUILT-S R2023A** (Comisión Europea, JRC): superficie construida por celda de 100 m,
cada cinco años de 1975 a 2020, descarga directa sin cuenta. Guatemala cabe en 3 mosaicos por época
(unos 4 MB cada uno). Con `faro construido` la serie por municipio sale en 8 segundos.
Detalle y cifras: [`datos/referencia/LEEME.md`](../datos/referencia/LEEME.md).

```bash
faro fuentes url "https://jeodpp.jrc.ec.europa.eu/ftp/jrc-opendata/GHSL/GHS_BUILT_S_GLOBE_R2023A/GHS_BUILT_S_E2020_GLOBE_R2023A_54009_100/V1-0/tiles/GHS_BUILT_S_E2020_GLOBE_R2023A_54009_100_V1_0_R8_C10.zip" --fuente ghsl-built-s --editor "Comisión Europea, JRC" --licencia "CC BY 4.0"
# (queda anotado en datos/crudo/manifiesto.jsonl: dirección, fecha, tamaño y huella)
faro construido
```

Los mosaicos de Guatemala son `R8_C10`, `R7_C10` y `R8_C9`; las épocas, de `E1975` a `E2020`.

## Otras opciones

| Opción | Qué da | Por qué no es la principal |
|---|---|---|
| Google Open Buildings 2.5D Temporal (sin Earth Engine) | Edificios y altura, 0.5 m, anual 2016–2023; los GeoTIFF están en un depósito público | Unos 1,680 archivos y cerca de 140 GB por año para Guatemala. Sirve para una muestra de municipios |
| World Settlement Footprint Evolution (DLR) | Mancha urbana anual 1985–2015, 30 m | Termina en 2015; buena para validar GHSL |
| ESA WorldCover | Cobertura del suelo 10 m | Solo 2020 y 2021, con métodos distintos |
| Cobertura anual Impact/Esri (Planetary Computer) | Clase "construido" 10 m, 2017–2023 | Alternativa reciente; descarga no probada |
| Sentinel-2 y Landsat crudos (Planetary Computer, AWS, Copernicus) | Imágenes | Habría que clasificarlas |
| Huellas de edificios (Microsoft, Google v3, Overture) | Polígonos de una fecha | Sin serie de tiempo |

## Sobre Earth Engine

Desde abril de 2026 Earth Engine tiene niveles no comerciales; el nivel "Community" no pide
cuenta de facturación. Si el uso del equipo es no comercial, conviene revisar el registro. No es
necesario para lo que FARO calcula hoy.

## Estudios parecidos

No se encontró un estudio de la Universidad Rafael Landívar sobre techos por municipio. Lo más
cercano: análisis multitemporal de palma africana 2010–2020 (IARNA), mapa de cambio de uso de la
tierra 2001–2010 (MAGA, URL, USAC y otros) y un estudio de la mancha urbana de las capitales
centroamericanas 1975–2014 con Landsat.

## Cómo se usó Landsat, exactamente

**FARO no procesa imágenes Landsat crudas.** Las usa de dos maneras, las dos indirectas:

1. **Como insumo del GHSL.** La capa GHS-BUILT-S, de donde salen todas las cifras de superficie
   construida, la produce la Comisión Europea a partir de mosaicos Landsat (épocas de 1975 a 2014)
   y de Sentinel-2 (2018). FARO descarga esa capa ya clasificada y suma sus celdas por municipio.
   Las cifras de crecimiento urbano del portal vienen, entonces, de Landsat y Sentinel-2 a través del GHSL.
2. **Como foto de fondo de la animación.** Al tocar un municipio en «Población y crecimiento», la
   animación de la mancha urbana se dibuja sobre una foto satelital: mosaicos anuales Landsat de
   1985, 1990, 1998 y 2000 (NASA GIBS) para las épocas hasta 2005, y el mosaico Sentinel-2 sin
   nubes de 2016 (EOX) desde 2010. Cada cuadro dice qué foto lleva. No hay mosaico Landsat libre
   anterior a 1985, así que 1975 y 1980 se muestran sobre la foto de 1985. **La foto es una
   referencia visual: sobre ella no se mide nada.**

La animación corre una vez, de 1975 a 2020, y se detiene; la casilla «Repetir sin parar» la deja en
bucle. Se puede quitar la foto o quitar los bloques para ver solo el terreno. El mapa principal
tiene además una capa opcional con el mosaico Sentinel-2 de 2016.

| Foto | Servicio | Detalle | Licencia |
|---|---|---|---|
| Landsat, mosaico anual | NASA GIBS, capa `Landsat_WELD_CorrectedReflectance_TrueColor_Global_Annual` (reflectancia de superficie WELD v3.1) | 30 m | Uso libre; se reconoce a NASA GIBS/ESDIS |
| Sentinel-2, mosaico 2016 | EOX, capa `s2cloudless` | 10 m | CC BY 4.0: «Sentinel-2 cloudless de EOX IT Services GmbH (contiene datos Copernicus Sentinel modificados, 2016)» |

Desde la v0.17.1 cada foto se pide **entera**, como una sola imagen del recuadro del municipio
(servicio WMS de cada proveedor), y no por teselas: son cinco imágenes para toda la animación, se
traen al abrir y la animación no avanza a un cuadro cuya foto no ha llegado. Si el servicio WMS
falla, esa foto vuelve a pedirse por teselas (WMTS). Los mosaicos de EOX de 2018 en adelante **no**
se usan: su licencia es no comercial.

Se descartaron: la imagen de Esri (su licencia exige programas o suscripción de Esri), los
mosaicos de EOX de 2018 en adelante (licencia no comercial) y los de la USGS (piden iniciar sesión).

## Uso del suelo y tipo de techo: qué se puede y qué no

**Con Landsat no se distinguen techos.** Cada celda mide 30 m por lado, 900 m²: en ella caben
varias casas. Los proyectos que sí clasifican techos usan imagen de centímetros o de menos de un metro:

- **Open AI Caribbean Challenge** (DrivenData, Banco Mundial y WeRobotics, 2019): material del
  techo con imágenes de dron de unos 4 cm; incluye dos zonas de **Mixco**. Cinco clases: concreto,
  lámina en buen estado, lámina irregular, incompleto y otro. Las soluciones ganadoras son de
  código abierto. <https://www.drivendata.org/competitions/58/disaster-response-roof-type/>
- **NASA DEVELOP, «Guatemala and Panama Urban Development»** (2022): midió la extensión urbana
  con Landsat, pero los techos (teja, metal) con imágenes WorldView.
  <https://ntrs.nasa.gov/citations/20220018398>
- **Dymaxion Labs y TECHO** (2020): detección de asentamientos en Ciudad de Guatemala con imagen
  submétrica y la encuesta de TECHO. <https://arxiv.org/abs/2006.14490>

**Lo que FARO sí hace, con datos abiertos y oficiales:**

| Pregunta | Fuente | Resultado |
|---|---|---|
| ¿Cuánto de lo construido es residencial? | GHSL, componente no residencial (NRES), 100 m | `faro uso-suelo`: por municipio (2000 y 2020) y por bloque de 1 km. En 2020, 1,486 de 93,297 ha son no residenciales (1.6 %); las mayores superficies están en Guatemala (296 ha), Villa Nueva (187), Mixco (73), Amatitlán (66) y Escuintla (56) |
| ¿De qué son los techos? | Censo 2018, cuadro C2: lo declaró cada hogar | Por municipio (68.1 % de las viviendas del país tiene techo de lámina) y por lugar poblado (20,254 lugares) |

Las dos cosas se cruzan con cada visita de campo: el portal dice cuánto hay construido en el
kilómetro donde está el encuestador, qué parte es residencial y cuál es el techo más frecuente en
ese lugar poblado ([`24-clasificador-de-fotos.md`](24-clasificador-de-fotos.md) explica la parte de la foto).

**Siguiente paso posible.** El mismo proyecto GHSL publica GHS-BUILT-C a 10 m (2018), con altura y
tipo de edificio (residencial o no, en cinco tramos de altura). Para Guatemala son tres mosaicos de
6 a 79 MB. Y con imagen de dron del INE o de municipalidades se puede seguir el método del reto de
DrivenData, que ya incluye a Mixco.
