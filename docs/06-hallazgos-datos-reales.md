# Qué pasó al recibir los archivos reales

El 23 de noviembre de 2026 la Coordinación del SEN compartió cinco archivos que ejemplifican cómo
las instituciones remiten sus registros. Se procesaron con FARO **sin modificar los archivos**.
Aquí solo hay conteos: ningún dato de personas sale de la máquina donde se procesa ni entra al
repositorio.

## Resultado

| Institución | Registro | Filas | Columnas que llegan | Columnas pedidas | Lugares con código oficial | Tiempo |
|---|---|---|---|---|---|---|
| RENAP | Defunciones (julio 2026) | 10,034 | 82 | 14 | 99.6 % | 0.7 s |
| RENAP | Nacimientos (junio 2026) | 30,093 | 53 | 13 | 97.9 % | 1.7 s |
| PNC | Detenidos (octubre 2026) | 4,255 | 50 | 9 | 100 % | 0.2 s |
| SE-CONRED | Emergencias (2025, 11 hojas) | 7,337 | 24 | 10 | 99.9 % | 0.2 s |
| INSIVUMEH | Calidad del aire (2025, por hora) | 8,760 horas | 10 estaciones | — | — | <1 s |

"Lugares con código oficial" excluye los vacíos y los marcados como ignorados en origen.

## Lo que se encontró

**Cada institución escribe lo mismo de otra forma, y a veces la misma institución también.**

| Variable | RENAP nacimientos | RENAP defunciones | PNC | SE-CONRED |
|---|---|---|---|---|
| Sexo | `M` / `F` | `MASCULINO` / `FEMENINO` | `HOMBRE` / `MUJER` | no aplica |
| Departamento | código `8` | código `8` | no viene (va dentro del municipio) | nombre, con y sin tilde (`Quiche`, `Quiché`) |
| Municipio | correlativo `1` dentro del departamento | correlativo `1` | código completo sin cero (`101`) | nombre |
| Sin dato | `-1` | vacío | `IGNORADO` | vacío, `Todos`, `**` |
| Fecha | `DD/MM/AAAA` | `DD/MM/AAAA` | fecha de Excel | fecha y hora |
| Encabezado | fila 1 | fila 1 | fila 1 (dos columnas llamadas `MUNICIPIO`) | fila 4, bajo totales y títulos |

**Errores que hoy pasan inadvertidos y FARO reporta en vez de corregir en silencio:**

- Defunciones: 23 filas con un código de departamento que no existe y 37 con municipio inexistente.
- Nacimientos: 1,197 filas con municipio de residencia de la madre que no corresponde a su departamento.
- Emergencias: un municipio asignado a un departamento equivocado en 2 filas (lo delata el contexto),
  nombres partidos (`La De Mocracia`), y letras dañadas por la codificación (`Vi±as` por `Viñas`).
- Emergencias: 174 celdas numéricas con texto (`**`) y totales con punto de miles mezclados.
- Pueblo de pertenencia: aparecen categorías fuera del catálogo del censo (por ejemplo, descripciones
  de color de piel). FARO las lista para que la OCSE decida, no las reasigna.

**Datos personales.** El archivo de defunciones trae CUI, nombres y direcciones de la persona
fallecida, su madre y su padre. Con la solicitud de FARO, de 82 columnas salen 14: el nombre se
elimina, el CUI se sustituye por una llave no reversible (permite parear sin identificar) y las
direcciones nunca pasan porque no se piden.

**La memoria funciona con datos reales.** Tras confirmar una vez que `La De Mocracia`
(Huehuetenango) es La Democracia, el reenvío del mismo archivo resolvió esas 6 filas solo.

## Calidad del aire

Regla planteada por el INE como idea general: un día con más de 4 horas sin dato no cuenta, y un
mes con más de 2 días que no cuentan se descarta. Aplicada al archivo de 2025:

- 228 combinaciones estación–contaminante–mes.
- 103 con promedio mensual válido, 75 descartadas por falta de representatividad, 50 sin ningún dato.

Los dos umbrales son parámetros (`faro aire --max-horas-sin-dato 4 --max-dias-no-validos 2`).
La lectura de la regla es nuestra; conviene confirmarla con la unidad de estadísticas ambientales.

## Pendiente

- Diccionarios de variables del INE (hechos delictivos) para codificar delito y causa.
- Decidir con RENAP el significado de `PERIODO_EDAD` (unidad de la edad).
- Verificar el catálogo de municipios código por código contra el listado oficial.
