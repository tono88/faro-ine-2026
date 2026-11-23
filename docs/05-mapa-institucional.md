# Cómo se conectan todas las instituciones

Propuesta para que el INE lidere la homologación del envío de registros administrativos.
Base: los 105 conjuntos del portal de datos abiertos del INE (consultado el 23-nov-2026) y la
estructura de registros administrativos presentada por la Dirección de Índices y Estadísticas
Continuas.

## 1. Qué se homologa

Casi todos los registros que recibe el INE comparten las mismas pocas variables. Si esas
llegan iguales, los registros se pueden cruzar entre sí y con censos y encuestas.

| Variable | Problema de hoy | Regla propuesta |
|---|---|---|
| Departamento, municipio, lugar poblado | Cada institución lo escribe distinto | Código INE; si llega texto, se homologa contra el Censo 2018 y se guarda la equivalencia |
| Fecha del hecho | Formatos mezclados; a veces es fecha de registro | AAAA-MM-DD, siempre la de ocurrencia |
| Sexo, pueblo | "M", "Masc.", "Hombre" | Catálogo cerrado con equivalencias |
| Edad | 99 o 999 como "ignorado" | Años cumplidos; vacío si se ignora |
| Clasificaciones (delito, causa CIE-10, CIIU, partida arancelaria) | Catálogos propios | Clasificador oficial acordado por sector |

Definición ejecutable: [`datos/estandar/nucleo_comun.toml`](../datos/estandar/nucleo_comun.toml).

## 2. Quién envía qué

Detalle por conjunto en [`datos/estandar/registros.csv`](../datos/estandar/registros.csv) (40 filas).
La columna `evidencia` dice de dónde sale la institución fuente: del nombre del conjunto, de la
presentación del INE, o **por confirmar** con la OCSE correspondiente.

| Sector (OCSE) | Registros en el portal del INE | Instituciones fuente | Se integra con |
|---|---|---|---|
| Salud | Nacimientos, defunciones, defunciones fetales, matrimonios, divorcios; hospitalarias internas y externas | RENAP*, hospitales privados | Proyecciones de población (tasas), Censo 2018, ENCOVI |
| Educación | Educación formal, establecimientos, educación superior, bibliotecas | MINEDUC*, universidades* | Censo 2018 (asistencia escolar), ENCOVI |
| Seguridad y justicia | Víctimas y detenidos PNC, agraviados y sindicados MP, sentencias OJ, faltas judiciales, INACIF, violencia intrafamiliar y contra la mujer, tránsito | PNC, MP, OJ, INACIF | Proyecciones de población (tasas por 100 mil) |
| Migración | Flujo migratorio, extranjeros atendidos, expulsados e inadmitidos, retornados | Instituto Guatemalteco de Migración* | ENEI, Censo 2018 (migración) |
| Ambiente | Compendio ambiental (8 temas), incentivos forestales | INAB y otras* | Censo 2018 (agua, saneamiento, basura) |
| Económicas | DINESE, comercio exterior, construcciones particulares, destace | SAT, otras* | ENEI, ENIGH 2023, IPC |

\* Por confirmar. No se inventa: se valida con cada OCSE antes de publicar la solicitud.

El mismo hecho aparece en varias instituciones (una muerte violenta está en PNC, MP, INACIF,
RENAP y OJ). Con el núcleo común homologado, esos registros se pueden comparar por municipio,
mes, sexo y edad sin tocar identidades.

## 3. Tres niveles de conexión

Ninguna institución tiene que esperar a otra ni cambiar su sistema para empezar.

| Nivel | Cómo envía | Qué necesita la institución | Estado en FARO |
|---|---|---|---|
| 0. Archivo | CSV o Excel por el canal actual | Nada nuevo | Funciona hoy (`faro recibir`) |
| 1. Portal | Sube el archivo y ve en el momento qué filas fallan y qué lugares hay que confirmar | Un navegador | Día 2 |
| 2. X-Road | Servicio REST publicado en su servidor de seguridad; el INE consulta según calendario | Estar conectada a X-Road | Diseño; mismo contrato |

El contrato (la solicitud de datos) es el mismo en los tres niveles. Eso es lo que permite
empezar hoy con archivos y pasar a X-Road cuando cada institución esté lista.

## 4. Gobernanza

1. El INE propone la solicitud de cada registro a partir del núcleo común.
2. La OCSE del sector la revisa y la aprueba; queda versionada (`hechos-transito-v1`).
3. La institución envía; FARO valida y devuelve un reporte fila por fila.
4. Los lugares dudosos los confirma una persona del INE o de la institución; la equivalencia
   queda en la memoria con nombre, fecha e institución.
5. La memoria de equivalencias se publica: cualquier institución puede reutilizarla.

## 5. Pareo y protección de datos

- **Pareo agregado** (por defecto): territorio + periodo + sexo + edad. No requiere identificadores.
- **Pareo de registros** (empresas con NIT, personas con CUI): solo dentro del INE, con el
  identificador transformado en una llave no reversible y nunca publicado. Requiere convenio,
  como el del INE con la SAT que dio origen al DINESE.
- FARO no publica microdatos pareados ni intenta identificar personas u hogares.

## 6. Qué falta validar

- Institución fuente de las filas marcadas "por confirmar".
- Nombres reales de columnas de cada registro (para llenar los `alias`): se obtienen al
  descargar los conjuntos con `faro fuentes descargar`.
- Catálogos oficiales de pueblo, delito y causa que usa hoy el INE.
