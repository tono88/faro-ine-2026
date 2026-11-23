# FARO · Datatón INE 2026

Categoría 2: Tecnológica y ciencias de datos.
Equipo: Estuardo Sandoval · Ana Echeverría · Alejandro Sandoval.

FARO es una herramienta para que el INE **pida, reciba y homologue** registros administrativos
de otras instituciones de forma estándar, y los deje listos para integrarse con censos y encuestas.

## El problema

Las instituciones que envían datos al INE no tienen un formato común. El mismo municipio llega
escrito como "Puerto de San José", "El puerto", "PTO. SAN JOSE" o "San José" (que además existe
en Escuintla y en Petén). Cada envío se limpia a mano y lo aprendido se pierde.

## Qué hace hoy (día 1)

1. **Solicitud de datos.** El INE declara una vez qué columnas necesita, de qué tipo, cuáles son
   obligatorias y cuáles sirven para el pareo (`datos/ejemplos/solicitud_hechos_transito.toml`).
2. **Recepción.** Cada envío se valida contra la solicitud: reconoce columnas aunque vengan con
   otro nombre ("DEPTO.", "Muni"), y reporta fila por fila fechas inválidas, vacíos y valores
   fuera de catálogo.
3. **Homologación.** Los nombres de lugar se llevan al código oficial del INE. El sistema solo
   asigna cuando no hay duda; lo demás lo propone con candidatos y lo decide una persona.
4. **Memoria.** Cada decisión queda grabada (qué texto, qué código, quién, cuándo, de qué
   institución). El siguiente envío ya no pregunta lo mismo.

## Probarlo

Requiere [Rust](https://rustup.rs) estable (probado con 1.97). No usa servicios en la nube ni de pago.

```bash
cargo test                      # 11 pruebas
cargo build --release

# 1. Recibir un envío de ejemplo (datos ficticios)
./target/release/faro recibir \
  --solicitud datos/ejemplos/solicitud_hechos_transito.toml \
  --entrada   datos/ejemplos/envio_hechos_transito.csv \
  --salida    salida

# 2. Ver lo que quedó por decidir
cat salida/pendientes.csv

# 3. Una persona confirma las equivalencias
./target/release/faro confirmar "El puerto"          --codigo 0509 --contexto 05 --institucion Ejemplo --usuario analista
./target/release/faro confirmar "Puerto de San José" --codigo 0509 --contexto 05 --institucion Ejemplo --usuario analista

# 4. Recibir de nuevo: ahora esas filas salen resueltas por memoria
./target/release/faro recibir --solicitud datos/ejemplos/solicitud_hechos_transito.toml \
  --entrada datos/ejemplos/envio_hechos_transito.csv --salida salida
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
| `datos/ejemplos/` | Envío y solicitud **ficticios** para demostración | Incluido |

El repositorio no contiene datos de personas. Los ejemplos son inventados.

## Estructura

```
crates/faro-homologa/   biblioteca: normalizar, catálogo, homologar, memoria, solicitud
crates/faro-cli/        ejecutable `faro`
datos/catalogo/         catálogos oficiales en CSV (codigo,nombre,nivel,padre)
datos/ejemplos/         solicitud y envío de demostración
docs/                   necesidades del INE, investigación, arquitectura y plan
```

## Documentación

- [Necesidades planteadas por el INE](docs/01-necesidades-ine.md)
- [Investigación y referencias](docs/02-investigacion.md)
- [Arquitectura y relación con X-Road](docs/03-arquitectura.md)
- [Plan hasta la entrega](docs/04-plan.md)

## Componentes de terceros

Bibliotecas de Rust de código abierto (licencias MIT o Apache-2.0): `serde`, `serde_json`, `toml`,
`csv`, `strsim`, `unicode-normalization`, `chrono`, `clap`. Versiones exactas en `Cargo.lock`.

## Uso de inteligencia artificial

El código y la documentación se desarrollaron con ayuda de un asistente de IA (Claude, de
Anthropic), bajo dirección y revisión del equipo. Los commits generados con el asistente lo
indican en su mensaje.
