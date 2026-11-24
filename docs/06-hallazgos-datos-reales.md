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
- Nacimientos: 1,197 filas con un código de municipio de residencia de la madre que no existe (en 943 de ellas tampoco existe el departamento).
- Emergencias: 2 filas con un municipio que no pertenece al departamento indicado (lo delata el
  contexto), nombres partidos (`La De Mocracia`), y letras dañadas por la codificación (`Vi±as` por `Viñas`).
- Emergencias: 174 celdas numéricas con contenido no numérico (por ejemplo `**`).
- Pueblo de pertenencia: aparecen categorías fuera del catálogo del censo (por ejemplo, descripciones
  de color de piel). FARO las lista para que la OCSE decida, no las reasigna.

**Datos personales.** El archivo de defunciones trae CUI, nombres y direcciones de la persona
fallecida, su madre y su padre. Con la solicitud de FARO, de 82 columnas salen 14: el nombre se
elimina, el CUI se sustituye por una llave no reversible (permite parear sin identificar) y las
direcciones nunca pasan porque no se piden.

**La memoria funciona con datos reales.** Tras confirmar una vez que `La De Mocracia`
(Huehuetenango) es La Democracia, el reenvío del mismo archivo resolvió esas 6 filas solo.

## Hallazgos en los datos abiertos del propio INE (24-nov-2026)

- **Proyecciones de población por municipio, 2015–2030.** El libro identifica 16 municipios del
  departamento de Guatemala con códigos que no existen (`0123` a `0138`) en lugar de `0102` a
  `0117`; su diccionario los nombra bien. FARO lo detectó al cruzar con el catálogo y los ubicó por
  nombre. Detalle en [`datos/referencia/LEEME.md`](../datos/referencia/LEEME.md).
- **Listado de lugares poblados, Censo 2018.** Los 340 códigos de municipio coinciden con el
  catálogo; 313 de 20,036 lugares se publican sin coordenadas. Detalle en
  [`datos/catalogo/LEEME.md`](../datos/catalogo/LEEME.md).

## Qué se corrige ahora y qué sigue decidiendo una persona

| Situación vista en los archivos | Qué hace FARO |
|---|---|
| Letras dañadas (`Vi±as`, `Due±as`) | Las repara al comparar: no hay otra lectura posible. |
| Código de municipio que no existe (por ejemplo `37` en Escuintla) | Si el archivo trae además el nombre, lo usa como respaldo (`respaldo` en la solicitud). Si no, la persona lo busca en la lista del catálogo y queda aprendido para ese departamento. |
| Municipio que no pertenece al departamento indicado (`Camotán` en El Progreso) | Muestra dónde sí existe (Chiquimula) y la persona decide. |
| Valor de catálogo no previsto (sexo, pueblo, tipo de vehículo) | Se elige el valor permitido en pantalla y queda aprendido por variable. |
| `98`, `99`, `-1`, `IGNORADO` | Salen vacíos si la solicitud los declara como "sin dato" (`sin_dato`). |

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
