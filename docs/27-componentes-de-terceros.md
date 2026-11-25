# Componentes de terceros y sus licencias

*Generado con `node scripts/entregables/terceros.js` a partir de `Cargo.lock`. No editar a mano.*

La nota conceptual del Datatón pide que las librerías y componentes de terceros conserven sus
licencias originales y se listen en la documentación. Todo lo que FARO usa es de código abierto o de
uso libre; **no hay ningún componente de pago**. El código propio de FARO (los cinco *crates* de
`crates/`, la pantalla y los guiones) se escribió durante el evento; su licencia está en
[`LICENCIA.md`](../LICENCIA.md).

## 1. Bibliotecas de Rust usadas directamente (21)

| Biblioteca | Versión | Licencia | Para qué se usa | En |
|---|---|---|---|---|
| [`argon2`](https://crates.io/crates/argon2) | 0.5.3 | MIT OR Apache-2.0 | Huella de contraseñas | servidor |
| [`axum`](https://crates.io/crates/axum) | 0.7.9 | MIT | Servidor web | servidor |
| [`calamine`](https://crates.io/crates/calamine) | 0.30.1 | MIT | Lectura de Excel (xlsx, xls, ods) | homologa |
| [`chrono`](https://crates.io/crates/chrono) | 0.4.45 | MIT OR Apache-2.0 | Fechas | fuentes, homologa |
| [`clap`](https://crates.io/crates/clap) | 4.6.9 | MIT OR Apache-2.0 | Línea de comandos `faro` | cli |
| [`csv`](https://crates.io/crates/csv) | 1.4.0 | Unlicense/MIT | Lectura y escritura de CSV | cli, homologa, servidor, territorio |
| [`image`](https://crates.io/crates/image) | 0.25.10 | MIT OR Apache-2.0 | Abrir fotos (huella visual y clasificador) | fuentes |
| [`rand_core`](https://crates.io/crates/rand_core) | 0.6.4 | MIT OR Apache-2.0 | Azar del sistema para la sal de las contraseñas y los identificadores de sesión | servidor |
| [`regex`](https://crates.io/crates/regex) | 1.13.2 | MIT OR Apache-2.0 | Extracción de precios y limpieza de texto | fuentes, homologa |
| [`serde`](https://crates.io/crates/serde) | 1.0.229 | MIT OR Apache-2.0 | Leer y escribir estructuras (JSON, TOML, CSV) | fuentes, homologa, servidor, territorio |
| [`serde_json`](https://crates.io/crates/serde_json) | 1.0.154 | MIT OR Apache-2.0 | JSON del API y de los catálogos | cli, fuentes, homologa, servidor |
| [`sha2`](https://crates.io/crates/sha2) | 0.10.9 | MIT OR Apache-2.0 | Huellas SHA-256 de descargas y llaves de pareo | fuentes, homologa, servidor |
| [`strsim`](https://crates.io/crates/strsim) | 0.11.1 | MIT | Parecido entre nombres, solo para proponer (nunca para asignar) | homologa |
| [`tokio`](https://crates.io/crates/tokio) | 1.54.1 | MIT | Ejecución asíncrona del servidor | servidor |
| [`tokio-postgres`](https://crates.io/crates/tokio-postgres) | 0.7.18 | MIT OR Apache-2.0 | Conexión a PostgreSQL | servidor |
| [`toml`](https://crates.io/crates/toml) | 0.8.23 | MIT OR Apache-2.0 | Solicitudes de datos y reglas declarativas | fuentes, homologa |
| [`tract-onnx`](https://crates.io/crates/tract-onnx) | 0.23.10 | MIT OR Apache-2.0 | Ejecutar el modelo Places365 dentro del servidor, sin nube | fuentes |
| [`unicode-normalization`](https://crates.io/crates/unicode-normalization) | 0.1.26 | (MIT OR Apache-2.0) AND Unicode-3.0 | Quitar tildes y normalizar texto | homologa |
| [`ureq`](https://crates.io/crates/ureq) | 2.12.1 | MIT OR Apache-2.0 | Descargas HTTP de fuentes abiertas y muestreos | fuentes |
| [`weezl`](https://crates.io/crates/weezl) | 0.1.12 | MIT OR Apache-2.0 | Descompresión LZW del lector propio de GeoTIFF | territorio |
| [`zip`](https://crates.io/crates/zip) | 4.6.1 | MIT | Abrir archivos comprimidos de las fuentes | cli |

## 2. Bibliotecas que llegan por las anteriores

Las dependencias directas traen otras 309 bibliotecas (330 en total contando las propias de cada sistema operativo, con sus
versiones exactas en `Cargo.lock`). Por licencia declarada:

| Licencia | Bibliotecas |
|---|---|
| MIT OR Apache-2.0 | 198 |
| MIT | 41 |
| Apache-2.0 OR MIT | 20 |
| Unicode-3.0 | 18 |
| Unlicense OR MIT | 9 |
| Apache-2.0 | 6 |
| Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | 6 |
| Zlib OR Apache-2.0 OR MIT | 4 |
| (MIT OR Apache-2.0) AND Unicode-3.0 | 3 |
| Zlib | 2 |
| MIT OR Zlib OR Apache-2.0 | 2 |
| BSD-3-Clause OR Apache-2.0 | 2 |
| ISC | 2 |
| Apache-2.0 OR BSL-1.0 OR MIT | 2 |
| CDLA-Permissive-2.0 | 2 |
| BSD-2-Clause OR Apache-2.0 OR MIT | 2 |
| MIT OR Apache-2.0 OR Zlib | 2 |
| 0BSD OR MIT OR Apache-2.0 | 1 |
| BlueOak-1.0.0 OR MIT OR Apache-2.0 | 1 |
| (Apache-2.0 OR MIT) AND BSD-3-Clause | 1 |
| MIT AND BSD-3-Clause | 1 |
| MIT OR Apache-2.0 OR LGPL-2.1-or-later | 1 |
| Apache-2.0 AND ISC | 1 |
| Apache-2.0 OR ISC OR MIT | 1 |
| Apache-2.0 OR BSL-1.0 | 1 |
| BSD-3-Clause | 1 |

Todas declaran una licencia permisiva (MIT, Apache-2.0, BSD, ISC, Zlib, Unicode, MPL-2.0 u otra equivalente): ninguna obliga a publicar el código de FARO.

La lista completa, con versión y licencia de cada una, se obtiene con `cargo metadata --format-version 1`
o con `cargo tree`.

## 3. Componentes que no son de Rust

| Componente | Versión | Licencia | Dónde está y para qué |
|---|---|---|---|
| [Leaflet](https://leafletjs.com) | 1.9.4 | BSD de 2 cláusulas | Incluido en `crates/faro-servidor/web/vendor/` con su archivo de licencia: mapas de la pantalla |
| [Noto Sans](https://fonts.google.com/noto/specimen/Noto+Sans) | — | SIL Open Font License 1.1 | Tipografía; el navegador la pide a Google Fonts si hay internet (si no, usa la del sistema) |
| [PostgreSQL](https://www.postgresql.org) | 16 | Licencia PostgreSQL (permisiva) | Base de datos; imagen oficial `postgres:16` |
| Imágenes de Docker `rust:1-bookworm` y `debian:bookworm-slim` | — | Software libre (Debian; Rust: MIT o Apache-2.0) | Compilar y ejecutar el portal |
| [Places365-ResNet18](http://places2.csail.mit.edu) (MIT CSAIL) | — | Pesos: CC BY; código: MIT | `datos/modelos/` ([procedencia](../datos/modelos/PROCEDENCIA.md)): clasificador de fotos del trabajo de campo, sin cambios |
| [Playwright](https://playwright.dev) y Chromium | — | Apache-2.0 / BSD | Solo para las pruebas de pantalla y para generar los PDF; no forman parte del portal |
| [pptxgenjs](https://gitbrent.github.io/PptxGenJS/) | 4 | MIT | Solo para generar la presentación editable en PowerPoint; no forma parte del portal |
| [FFmpeg](https://ffmpeg.org) | — | LGPL / GPL, según cómo se compile | Solo para unir los cuadros del video demostrativo; no forma parte del portal |
| [PyTorch](https://pytorch.org) y torchvision | — | BSD de 3 cláusulas | Solo en `scripts/modelo/exportar.py`, para convertir el modelo a ONNX una vez; no forman parte del portal |

## 4. Servicios que el navegador consulta (opcionales, gratuitos)

No son componentes del portal: los pide el navegador de quien mira, si hay internet y si la persona
enciende esa capa. Ningún dato del INE ni de las instituciones sale hacia ellos.

| Servicio | Qué entrega | Condiciones |
|---|---|---|
| Mapa base de OpenStreetMap | Teselas del mapa de fondo | © colaboradores de OpenStreetMap (ODbL); política de uso de teselas de la OSMF |
| NASA GIBS | Mosaicos anuales Landsat (WELD) de 1985, 1990, 1998 y 2000 | Datos de la NASA, de uso libre con reconocimiento |
| EOX IT Services | Mosaico Sentinel-2 sin nubes de 2016 | CC BY 4.0 (contiene datos Copernicus Sentinel modificados, 2016) |
| Microsoft Forms del INE | La encuesta de campo, si el INE la configura | Cuenta institucional del INE; las respuestas no pasan por FARO |

Los **datos** (INE, OpenStreetMap, Comisión Europea, SESAN, FAO y los demás) tienen su propia lista,
con institución, fecha de descarga y licencia: [`11-fuentes-y-citas.md`](11-fuentes-y-citas.md).
