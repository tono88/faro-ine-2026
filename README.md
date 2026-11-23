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
6. **Crecimiento urbano.** A partir de edificaciones por año vistas por satélite, calcula el
   ritmo de crecimiento de cada municipio y lo proyecta con su margen (`faro crecimiento`).

## Probarlo

Requiere [Rust](https://rustup.rs) estable (probado con 1.97). El núcleo no usa servicios en la nube ni de pago.
El módulo de crecimiento urbano obtiene sus datos con Google Earth Engine (servicio en la nube de Google, sin costo para uso no comercial).

```bash
cargo test                      # 21 pruebas
cargo build --release

# 1. Recibir un envío de ejemplo (datos ficticios)
./target/release/faro recibir \
  --solicitud datos/solicitudes/ejemplo-hechos-transito-v1.toml \
  --entrada   datos/ejemplos/envio_hechos_transito.csv \
  --salida    salida

# 2. Ver lo que quedó por decidir
cat salida/pendientes.csv

# 3. Una persona confirma las equivalencias
./target/release/faro confirmar "El puerto"          --codigo 0509 --contexto 05 --institucion Ejemplo --usuario analista
./target/release/faro confirmar "Puerto de San José" --codigo 0509 --contexto 05 --institucion Ejemplo --usuario analista

# 4. Recibir de nuevo: ahora esas filas salen resueltas por memoria
./target/release/faro recibir --solicitud datos/solicitudes/ejemplo-hechos-transito-v1.toml \
  --entrada datos/ejemplos/envio_hechos_transito.csv --salida salida
```

```bash
# Fuentes del INE (requiere internet)
./target/release/faro fuentes inventario                      # todos los conjuntos y recursos
./target/release/faro fuentes descargar censo-2018-lugares-poblados --formato XLSX
./target/release/faro fuentes citar                           # tabla de fuentes y fechas

# Crecimiento urbano (ejemplo con datos ficticios)
./target/release/faro crecimiento --entrada datos/ejemplos/edificaciones_ficticias.csv --proyectar 2030
```

Salidas en `salida/`: `homologado.csv` (el envío más código y estado por columna geográfica),
`pendientes.csv` (valores distintos por revisar, con candidatos) y `reporte.json`.

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
| INE, Censo 2018, "Listado de lugares poblados" ([datos.ine.gob.gt](https://datos.ine.gob.gt/es/dataset/censo-2018-lugares-poblados), CC BY) | Catálogo oficial de lugares con código y coordenadas: fuente principal | Pendiente de cargar (día 2) |
| `datos/catalogo/departamentos.csv` | 22 departamentos con código INE | Incluido |
| `datos/catalogo/municipios_muestra.csv` | Muestra de municipios (Escuintla y Petén) para pruebas; se reemplaza por el listado oficial | Incluido, por verificar contra el listado oficial |
| Google Research, Open Buildings 2.5D Temporal (CC BY 4.0 / ODbL) | Edificaciones por año 2016–2023 para crecimiento urbano | Script listo (`scripts/`), pendiente de ejecutar |
| `datos/estandar/` | Núcleo común de variables y mapa de registros e instituciones (borrador del equipo) | Incluido |
| `datos/ejemplos/` | Envío, solicitud y serie de edificaciones **ficticios** para demostración | Incluido |

El repositorio no contiene datos de personas. Los ejemplos son inventados.

## Estructura

```
crates/faro-homologa/   biblioteca: normalizar, catálogo, homologar, memoria, solicitud
crates/faro-fuentes/    cliente CKAN y descargas con procedencia
crates/faro-territorio/ crecimiento de edificaciones y proyección
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

## Componentes de terceros

Bibliotecas de Rust de código abierto (licencias MIT o Apache-2.0): `serde`, `serde_json`, `toml`,
`csv`, `strsim`, `unicode-normalization`, `chrono`, `clap`, `ureq`, `sha2`. Versiones exactas en `Cargo.lock`.

## Uso de inteligencia artificial

El código y la documentación se desarrollaron con ayuda de un asistente de IA (Claude, de
Anthropic), bajo dirección y revisión del equipo. Los commits generados con el asistente lo
indican en su mensaje.
