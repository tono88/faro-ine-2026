# Autoría y licencia

**FARO · Que cada dato llegue a puerto.**
© 2026 **La Sociedad de Almas**: Estuardo Sandoval, Ana Echeverría y Alejandro Sandoval.

FARO se desarrolló del 23 al 26 de noviembre de 2026, durante el Datatón INE 2026 (categoría 2:
Tecnológica y ciencias de datos). El historial del repositorio es la constancia.

## 1. Licencia al Instituto Nacional de Estadística

Conforme a los términos y condiciones del Datatón INE 2026 para la categoría 2, los integrantes del
equipo **conservan el reconocimiento de la autoría** y otorgan al **Instituto Nacional de
Estadística de Guatemala (INE)** una licencia **no exclusiva, gratuita, perpetua e irrevocable**
para **usar, reproducir, modificar, adaptar y distribuir** la herramienta y su documentación para
fines institucionales.

## 2. Qué cubre

Todo lo que el equipo escribió para FARO:

- el código de `crates/` (núcleo de homologación, territorio, fuentes, servidor, pantalla y línea
  de comandos) y los guiones de `scripts/` y `.faro/`;
- la documentación de `README.md`, `docs/` y `entregables/`;
- las solicitudes de datos, las reglas y los catálogos de trabajo de `datos/solicitudes/` y
  `datos/estandar/`, y los archivos de ejemplo ficticios de `datos/ejemplos/`;
- la identidad de FARO (nombre, lema y logotipo de `docs/identidad/`).

## 3. Qué no cubre: lo que es de terceros conserva su licencia

- **Componentes de software** (bibliotecas de Rust, Leaflet, PostgreSQL, el modelo Places365):
  cada uno mantiene su licencia original. Lista completa:
  [`docs/27-componentes-de-terceros.md`](docs/27-componentes-de-terceros.md).
- **Datos** del INE y de otras instituciones (OpenStreetMap, Comisión Europea, SESAN, FAO, CONRED/OCHA
  y los demás): cada conjunto mantiene su licencia y debe citarse como indica
  [`docs/11-fuentes-y-citas.md`](docs/11-fuentes-y-citas.md). **La fuente principal de los datos es
  el Instituto Nacional de Estadística.**
- Las tablas derivadas de OpenStreetMap (`datos/referencia/red_vial_*`) se distribuyen bajo la
  **Open Database License (ODbL) 1.0**, con reconocimiento a © los colaboradores de OpenStreetMap.

## 4. Otros usos

Para cualquier uso distinto de los fines institucionales del INE, los derechos siguen siendo de los
autores mientras el equipo no publique una licencia general. Quien quiera reutilizar FARO puede
escribir al equipo por medio del repositorio.

## 5. Sin garantía

FARO se entrega tal cual, como prototipo funcional de un concurso. Sus resultados dependen de los
datos de entrada y de parámetros que debe fijar el INE (ver «Limitaciones» en el `README.md`). Los
análisis de conformidad legal y de seguridad incluidos en `docs/` son trabajo del equipo y no
sustituyen el dictamen de la asesoría jurídica ni una auditoría externa.

## 6. Cómo citar

> La Sociedad de Almas (Sandoval, E., Echeverría, A. y Sandoval, A.) (2026). *FARO: recepción y
> homologación de registros administrativos para el Instituto Nacional de Estadística de Guatemala*
> [software]. Datatón INE 2026. https://github.com/tono88/faro-ine-2026
