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
