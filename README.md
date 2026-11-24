# FARO · Datatón INE 2026

Categoría 2: Tecnológica y ciencias de datos.
Equipo: Estuardo Sandoval · Ana Echeverría · Alejandro Sandoval.

FARO es una herramienta para que el INE **pida, reciba y homologue** registros administrativos
de otras instituciones de forma estándar, y los deje listos para integrarse con censos y encuestas.

## El problema

Las instituciones que envían datos al INE no tienen un formato común. El mismo municipio llega
escrito como "Puerto de San José", "El puerto", "PTO. SAN JOSE" o "San José" (que además existe
en Escuintla y en Petén). Cada envío se limpia a mano y lo aprendido se pierde.

## Qué hace hoy

1. **Solicitud de datos.** El INE declara una vez qué columnas necesita, de qué tipo, cuáles son
   obligatorias y cuáles sirven para el pareo (`datos/solicitudes/ejemplo-hechos-transito-v1.toml`).
2. **Recepción.** Cada envío se valida contra la solicitud: reconoce columnas aunque vengan con
   otro nombre ("DEPTO.", "Muni"), y reporta fila por fila fechas inválidas, vacíos y valores
   fuera de catálogo.
3. **Homologación.** Los nombres de lugar se llevan al código oficial del INE. El sistema solo
   asigna cuando no hay duda; lo demás lo propone con candidatos y lo decide una persona.
4. **Memoria.** Cada decisión queda grabada (qué texto, qué código, quién, cuándo, de qué
   institución). El siguiente envío ya no pregunta lo mismo.
5. **Fuentes con procedencia.** Descubre y descarga conjuntos del portal del INE y deja
   registrada la URL, la fecha y la huella de cada archivo (`faro fuentes`).
6. **Datos personales protegidos.** Solo salen las columnas pedidas; nombres se eliminan y
   documentos de identidad se sustituyen por una llave no reversible.
7. **Probado con archivos reales** de RENAP, PNC, SE-CONRED e INSIVUMEH compartidos por el INE:
   51,719 filas en menos de 3 segundos, entre 97.9 % y 100 % de lugares con código oficial.
   Ver [hallazgos](docs/06-hallazgos-datos-reales.md).
8. **Más que lugares.** Los valores de catálogos cerrados (sexo, pueblo, tipo de vehículo y los
   que declare cada solicitud) que no se reconocen también se revisan en pantalla y se aprenden
   por variable, para todas las instituciones. Cada solicitud puede declarar qué valores
   significan "sin dato" (98, 99, 999) y una columna de respaldo (el nombre, cuando el código
   propio de la institución no existe).
9. **Correcciones que ya no hay que hacer a mano.** Letras dañadas por la codificación
   ("Due±as" es "Dueñas"), códigos propios de una institución que se traducen una vez, y nombres
   que existen en otro departamento ("Camotán" enviado en El Progreso): se proponen, no se asignan.
10. **Población y tasas.** Proyecciones de población del INE por municipio, 2015–2030, como
    denominador: los resultados salen también por 100 mil habitantes, y la pantalla muestra el
    crecimiento poblacional de los 340 municipios (`faro poblacion`).
11. **Avance de la recepción.** Qué se pidió a cada institución, qué ha llegado, cuándo y con qué
    calidad.
12. **Catálogo oficial verificado.** Departamentos, municipios y 20,036 lugares poblados del
    Censo 2018 con coordenadas, comprobados código por código contra el listado del INE
    (`faro catalogo verificar`).
13. **Operadores de campo cerca de su casa.** Reparte operadores entre puestos con la menor
    distancia total posible, usando las coordenadas del catálogo (`faro asignar` y la pantalla).
    Cálculo exacto (método húngaro); distancia en línea recta, no por carretera.
14. **Precios de referencia.** A partir de cotizaciones calcula media, mediana y moda por producto
    y elige como lo explica la Dirección de Precios e Índices de Referencia (mediana si hay
    precios extremos, media si son parecidos); avisa si faltan cotizaciones directas o se mezclan
    unidades (`faro precios` y la pantalla).
15. **Crecimiento urbano.** A partir de edificaciones por año vistas por satélite, calcula el
    ritmo de crecimiento de cada municipio y lo proyecta con su margen (`faro crecimiento`).

## Levantarlo con Docker

```bash
cp .env.example .env          # cambie las tres claves
docker compose up --build     # base de datos PostgreSQL + FARO
```

Abra <http://localhost:8082>. Si ese puerto está ocupado, cambie `FARO_PUERTO` en `.env`
(`iniciar.bat` elige solo uno libre y muestra la dirección).

La pantalla tiene ocho secciones: recibir un envío, revisar pendientes, equivalencias
confirmadas, resultados y avance por institución, conjuntos del portal del INE (se sincroniza
sola al arrancar y cada 24 horas), crecimiento poblacional y urbano, asignación de operadores de
campo y precios de referencia. Cada sección enlaza un archivo de ejemplo (ficticio) para probarla.

En Linux, el contenedor escribe las descargas de fuentes abiertas en `datos/crudo` con un usuario
sin privilegios; antes del primer `faro fuentes ...` dentro del contenedor ejecute
`chmod o+w datos/crudo`. En Windows y macOS (Docker Desktop) no hace falta.

## Probarlo sin Docker

Requiere [Rust](https://rustup.rs) estable (probado con 1.97) y, para el servidor, PostgreSQL 16.

```bash
cargo test                       # 50 pruebas
cargo build --release

# Línea de comandos (no necesita base de datos)
./target/release/faro recibir --solicitud datos/solicitudes/ejemplo-hechos-transito-v1.toml \
  --entrada datos/ejemplos/envio_hechos_transito.csv --salida salida
./target/release/faro confirmar "El puerto" --codigo 0509 --contexto 05 --institucion Ejemplo --usuario analista
./target/release/faro fuentes inventario                       # conjuntos del INE (requiere internet)
./target/release/faro fuentes descargar censo-2018-lugares-poblados --formato XLSX
./target/release/faro catalogo verificar --listado <listado de lugares poblados.xlsx>   # catálogo contra el Censo 2018
./target/release/faro crecimiento --entrada datos/ejemplos/edificaciones_ficticias.csv --proyectar 2030
./target/release/faro aire --entrada <archivo de calidad del aire.xlsx>
./target/release/faro precios --cotizaciones datos/ejemplos/cotizaciones_ficticias.csv
./target/release/faro poblacion --entrada <proyecciones.xlsx> --diccionario <diccionario.xlsx>   # tabla municipio-año
./target/release/faro asignar --operadores datos/ejemplos/operadores_ficticios.csv --puestos datos/ejemplos/puestos_ficticios.csv

# Servidor
DATABASE_URL="host=localhost user=faro password=... dbname=faro" FARO_TOKEN=... FARO_SAL=... ./target/release/faro-servidor
```

Si una solicitud sustituye identificadores por llaves, defina el secreto `FARO_SAL`.

## Estados de homologación

| Estado | Significado | ¿Se asigna código? |
|---|---|---|
| `exacto` | Coincide con un único nombre oficial (sin importar tildes, mayúsculas o abreviaturas) | Sí |
| `memoria` | Equivalencia ya confirmada por una persona | Sí |
| `sugerido` | Un candidato muy parecido y sin competencia | No, falta confirmar |
| `ambiguo` | Varios lugares con ese nombre y falta el nivel superior | No |
| `revisar` | Hay candidatos, ninguno claro | No |
| `sin_coincidencia` | Nada parecido en el catálogo | No |

## Datos

| Fuente | Uso | Estado |
|---|---|---|
| INE, Censo 2018, "Listado de lugares poblados" ([datos.ine.gob.gt](https://datos.ine.gob.gt/es/dataset/censo-2018-lugares-poblados), CC BY) | Catálogo oficial de lugares con código y coordenadas: fuente principal | Cargado: 20,036 lugares poblados, 19,723 con coordenadas (`faro catalogo importar`) |
| `datos/catalogo/` | 22 departamentos, 340 municipios y 20,036 lugares poblados con código INE y alias | Incluido; verificado código por código contra el listado oficial ([detalle](datos/catalogo/LEEME.md)) |
| `datos/solicitudes/` | Solicitudes de datos para RENAP (defunciones, nacimientos), PNC (detenidos) y SE-CONRED (emergencias) | Incluido |
| Archivos de ejemplo de instituciones compartidos por el INE (23-nov-2026) | Diseño y prueba de las solicitudes | **No están en el repositorio** (contienen datos personales) |
| Google Research, Open Buildings 2.5D Temporal (CC BY 4.0 / ODbL) | Edificaciones por año 2016–2023 para crecimiento urbano | Script listo (`scripts/`), pendiente de ejecutar |
| `datos/estandar/` | Núcleo común de variables y mapa de registros e instituciones (borrador del equipo) | Incluido |
| `datos/ejemplos/` | Envío, solicitud, serie de edificaciones, operadores y puestos **ficticios** para demostración ([detalle](datos/ejemplos/LEEME.md)) | Incluido |

El repositorio no contiene datos de personas. Los ejemplos son inventados.

## Estructura

```
crates/faro-homologa/   biblioteca: lectura, solicitud, homologar, memoria, recepción
crates/faro-fuentes/    cliente CKAN y descargas con procedencia
crates/faro-territorio/ crecimiento de edificaciones y proyección
crates/faro-servidor/   servidor web, base de datos y pantalla
crates/faro-cli/        ejecutable `faro`
scripts/                script de Earth Engine para edificaciones por municipio
datos/catalogo/         catálogos oficiales en CSV (codigo,nombre,nivel,padre)
datos/estandar/         núcleo común de variables y mapa de registros
datos/ejemplos/         solicitud y envío de demostración
docs/                   necesidades del INE, investigación, arquitectura y plan
```

## Documentación

- [Necesidades planteadas por el INE](docs/01-necesidades-ine.md)
- [Investigación y referencias](docs/02-investigacion.md)
- [Arquitectura y relación con X-Road](docs/03-arquitectura.md)
- [Plan hasta la entrega](docs/04-plan.md)
- [Cómo se conectan todas las instituciones](docs/05-mapa-institucional.md)
- [Qué pasó al recibir los archivos reales](docs/06-hallazgos-datos-reales.md)
- [Modelos de código abierto que tendría sentido entrenar](docs/07-modelos.md)

## Componentes de terceros

Bibliotecas de Rust de código abierto (licencias MIT o Apache-2.0): `serde`, `serde_json`, `toml`,
`csv`, `strsim`, `unicode-normalization`, `chrono`, `clap`, `ureq`, `sha2`, `calamine`, `axum`, `tokio`, `tokio-postgres`. Base de datos: PostgreSQL 16. Versiones exactas en `Cargo.lock`.

## Uso de inteligencia artificial

El código y la documentación se desarrollaron con ayuda de un asistente de IA (Claude, de
Anthropic), bajo dirección y revisión del equipo. Los commits generados con el asistente lo
indican en su mensaje.
