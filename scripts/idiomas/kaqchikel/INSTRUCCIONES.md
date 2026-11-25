# Borrador en kaqchikel de los rótulos de FARO (para revisión posterior por una persona experta)

FARO es un portal de estadística de Guatemala. Su pantalla está en español y se quiere un
**borrador** de sus rótulos cortos en kaqchikel (ISO 639-3 `cak`, ortografía de la ALMG). Ese
borrador lo revisará después una persona hablante; lo que se necesita ahora es que esté armado
**solo con vocabulario que ya existe escrito por hablantes**, no con palabras inventadas.

## Qué recibes

Un archivo JSON con una lista de rótulos. Cada uno trae:

- `id` y `es`: el rótulo en español.
- `identico`: si la memoria de traducción de Mozilla (Firefox, Thunderbird, Common Voice…; hecha y
  revisada por hablantes) tiene exactamente ese texto, sus traducciones, con el original en inglés
  entre corchetes y cuántas veces aparece.
- `evidencia`: para cada palabra de contenido del rótulo, hasta cinco frases cortas de esa memoria
  (`español => kaqchikel  [inglés] xveces`) y, si existe, entradas de `LEXICO` (diccionarios y
  vocabularios de la ALMG u otras fuentes, con su fuente).

Las líneas que empiezan por `DICCIONARIO ALMG-CLK 2011:` vienen del vocabulario *Kaqchikel
Choltzij: Kolon chuqa' K'ak'a' taq Tzij* de la Comunidad Lingüística Kaqchikel de la Academia de
Lenguas Mayas (castellano seguido de su equivalente kaqchikel). Ese libro tiene secciones (artes
del idioma, ambiente natural, ambiente social, matemáticas, informática) y una misma palabra puede
salir con sentidos distintos: «mapa» geográfico es `ruwäch setul`, no `tz'eteb'äl` (mapa de bits);
elige el que corresponda a un portal de estadística y territorio. Las líneas `LEXICO:` son términos
ya comprobados en esa y otras fuentes de la ALMG, con una nota cuando hay dudas. Para los términos
del territorio, la población y la estadística, **prefiere el diccionario de la ALMG** a la memoria
de Mozilla; para acciones de pantalla (guardar, buscar, descargar), prefiere la memoria.

Puedes buscar más ejemplos en la memoria completa con `grep` (es un JSON de una sola línea por
unidad en `tm.jsonl`, campos `es`, `cak`, `en`): por ejemplo
`grep -i '"es": "[^"]*descarg' tm.jsonl | head -20`. Úsalo con moderación. El diccionario completo en texto está en
`lexico/clk_plano.txt` (la parte castellano ⇒ kaqchikel empieza en la línea 11304; cada entrada
empieza por la palabra en castellano): `grep -n -i '^vivienda' lexico/clk_plano.txt`. Todas las
rutas son relativas a la carpeta de trabajo. Escribe siempre el saltillo como apóstrofo recto `'`.

## Qué entregas

UN archivo JSON (con la herramienta Write, en la ruta que se te indique): un objeto que asocia cada
`id` (como texto) con

```json
{"cak": "<rótulo en kaqchikel>", "nivel": "fuente" | "compuesto", "base": ["<español => kaqchikel que usaste>", "..."]}
```

o con `null` si no se puede traducir con respaldo. **Todos los id del archivo de entrada deben
estar** (con objeto o con `null`).

- `"nivel": "fuente"`: el rótulo completo existe en la memoria con el mismo sentido (viene en
  `identico` o lo hallaste con grep).
- `"nivel": "compuesto"`: lo armaste tú combinando palabras y patrones que sí están en la evidencia.
- `base`: las frases de evidencia de donde tomaste cada palabra (una por palabra de contenido).

## Reglas

1. **Cada palabra kaqchikel que escribas debe aparecer, tal cual o en una frase de la evidencia.**
   No uses palabras de tu memoria que no veas escritas en la evidencia o en un resultado de grep.
   No inventes neologismos ni adaptes palabras del español.
2. **Cuida el sentido.** Mira el inglés entre corchetes. «Registro» como *log* de un programa no es
   «registro» como dato de una persona; «moda» estadística no es *fashion*; «media» no es *sock*;
   «población» no es *post town*. Si la evidencia es de otro sentido, no la uses.
3. **Si a una palabra de contenido le falta respaldo en el sentido correcto, devuelve `null`** para
   ese rótulo. Es preferible dejar un rótulo en español que escribir algo dudoso. Devuelve también
   `null` si el rótulo es un nombre propio, una sigla, un identificador o no es lenguaje natural.
4. **Sigue los patrones de la memoria**, que son los que ya conocen quienes usan programas en
   kaqchikel:
   - Botones y acciones: la forma que usa la memoria para esa acción (`Tiyak` «Guardar», `Tikanöx`
     «Buscar», `Titz'apïx` «Cerrar», `Tiyuj` «Eliminar», `Tiqasäx` «Descargar»…).
   - Plural: `taq` antes del nombre (`Taq yakb'äl` «Archivos»).
   - «sin X / ningún X»: `Majun X`. «nuevo X»: `K'ak'a' X`. «todos»: `Ronojel`. «y»: `chuqa'`.
     «en»: `pa`. «con»: `rik'in`. «de/para»: `richin`, o el poseído con `ru-`/`ki-` como lo muestre
     un ejemplo. «por»: `ruma` o `chi` según el ejemplo. «más»: `ch'aqa' chik`.
   - Usa una construcción solo si la ves en un ejemplo parecido; no deduzcas conjugaciones nuevas.
5. **No toques**: los números (`#`), las marcas `<1>…</1>` y `<1/>` (deben quedar las mismas), las
   entidades (`&nbsp;`, `&lt;`, `&amp;`, `&num;`), los signos y símbolos (`…`, `→`, `·`, `%`, `Q`),
   las siglas y nombres propios (INE, RENAP, PNC, IPMC, CSV, GeoJSON, FARO, OpenStreetMap…): se
   copian igual.
6. Mayúscula inicial como en el español. Ortografía exacta de la evidencia (saltillo `'`, diéresis
   `ä ë ï ö ü`).
7. Prioriza calidad sobre cantidad, pero **no devuelvas `null` por comodidad**: si todas las palabras
   de contenido tienen respaldo en el sentido correcto, compón el rótulo.

## Ejemplos

- `{"id": 845, "es": "Formulario", "identico": ["Nojwuj  [Form] x2"]}` →
  `"845": {"cak": "Nojwuj", "nivel": "fuente", "base": ["Formulario => Nojwuj"]}`
- `"Sin precios"` con evidencia `Precio => Rajil [Price]` →
  `{"cak": "Majun rajil", "nivel": "compuesto", "base": ["Precio => Rajil"]}`
- `"Cuenta creada"` con `Cuenta => Rub'i' taqoya'l [Account]` y `… creada => … xtz'uk [created]` →
  `{"cak": "Xtz'uk rub'i' taqoya'l", "nivel": "compuesto", "base": ["Cuenta => Rub'i' taqoya'l", "Nueva clave API creada => K'ak'a' API k'uxatzij xtz'uk"]}`
- `"Lineamientos del Datatón INE #"` (sin evidencia) → `null`
- `"Población"` con única evidencia `Post town` (otro sentido) y sin entrada de LEXICO → `null`

Al terminar, comprueba con un guion corto de python3 que el archivo es JSON válido, que tiene los
mismos id que la entrada y que cada traducción conserva las mismas marcas (`</?\d+/?>`), la misma
cantidad de `#` y las mismas entidades que su `es`. Responde solo: cuántos tradujiste, cuántos
`null`, y hasta diez dudas importantes.
