# Idiomas de la pantalla

Aquí vive la traducción de la pantalla de FARO. Son datos, no código: se corrigen con un editor de
texto y el cambio se ve al recargar la página.

| Archivo | Qué es | Se edita a mano |
|---|---|---|
| `en.json` | Inglés (completo) | Sí |
| `cak.json` | Kaqchikel (**borrador sin revisar**): trae todos los textos; vacíos los que faltan | Sí |
| `cak_revision.csv` | Hoja de revisión: cada rótulo en kaqchikel, su nivel (`fuente` o `compuesto`), en qué se basó, y columnas para la corrección y el nombre de quien revisa | Sí |
| `cak_palabras.csv` | Cada palabra kaqchikel usada, con su fuente y un ejemplo | Sí |
| `plantilla.json` | Todos los textos de la pantalla en español. La genera `scripts/idiomas/extraer.js` | No |
| `plantilla_extra.json` | Frases que la herramienta no alcanza a ver (llevan un dato en medio) | Sí |

**La regla: se traduce la interfaz, nunca los datos.** Los nombres de lugares, instituciones y
productos, los valores de los catálogos, las cifras y las citas no están aquí y no deben agregarse.

## Cómo se lee una entrada

```json
"Revisar pendientes <1/>": "Review pending items <1/>",
"<1>Entrada.</1> Cada celda de # m trae los metros construidos.": "<1>Input.</1> Each # m cell holds the built-up metres.",
"{0} filas cargadas; {1} lugares sin reconocer.": "{0} rows loaded; {1} places not recognized."
```

A la izquierda, el texto en español (no se cambia). A la derecha, la traducción. Hay que dejar
iguales:

- `#`: un número. Misma cantidad y en el mismo orden (si el orden debe cambiar, `#1`, `#2`…).
- `<1>…</1>`: una negrita o un enlace; se traduce lo de adentro y se deja la marca alrededor.
- `<1/>`: una pieza que no se traduce (un campo, un código, una cifra).
- `{0}`, `{1}`: un dato (un nombre de lugar, de institución, de archivo).
- `&nbsp;`, `&lt;`, `&amp;`, `&num;`: se dejan tal cual.
- `confianza|media`: la parte antes de `|` es el contexto de una palabra con dos sentidos.

Una entrada vacía (`""`) significa «sin traducir»: ese texto se muestra en español.

## Revisar y completar el kaqchikel

El kaqchikel es un borrador: 680 rótulos armados con vocabulario escrito por hablantes (la memoria
de traducción de Mozilla y el *Kaqchikel Choltzij* de la Comunidad Lingüística Kaqchikel de la
ALMG). No hay palabras inventadas, pero 570 rótulos son composiciones cuya gramática no ha
revisado una persona hablante.

1. Abrir `cak_revision.csv` (en Excel o LibreOffice). Empezar por los de nivel `compuesto`.
2. Corregir: escribir la forma correcta en `correccion` y el nombre en `revisado_por`, y llevar la
   corrección a `cak.json`.
3. Traducir lo que falta: llenar en `cak.json` las entradas vacías y agregar su fila a
   `cak_revision.csv`.
4. Una palabra nueva se anota en `cak_palabras.csv` con su fuente o con el nombre de la persona
   experta que la avala. `cargo test` rechaza una palabra que no esté ahí.
5. Comprobar: `node scripts/idiomas/comprobar.js`.
6. Al terminar la revisión: `"estado": "revisado"` y el nombre en `"revisado_por"`. El aviso de
   borrador deja de mostrarse.

## Sumar otro idioma

Copiar `cak.json` con el código ISO 639 del idioma (por ejemplo `quc.json` para k'iche'), cambiar
`idioma`, `nombre`, `nombre_es` y `estado`, y llenar las traducciones. El selector lo muestra solo.

Más detalle: [`docs/29-idiomas-e-inclusion.md`](../../../docs/29-idiomas-e-inclusion.md).
