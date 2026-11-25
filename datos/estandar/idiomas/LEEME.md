# Idiomas de la pantalla

Aquí vive la traducción de la pantalla de FARO. Son datos, no código: se corrigen con un editor de
texto y el cambio se ve al recargar la página.

| Archivo | Qué es | Se edita a mano |
|---|---|---|
| `en.json` | Inglés (completo) | Sí |
| `cak.json` | Kaqchikel (**borrador**): trae todos los textos; vacíos los que faltan | Sí |
| `cak_glosario.csv` | De dónde sale cada término en kaqchikel: fuente, mensaje y licencia | Sí |
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

## Completar el kaqchikel

1. Llenar en `cak.json` las entradas vacías.
2. Agregar a `cak_glosario.csv` cada término nuevo, con su fuente o con el nombre de quien lo
   revisó. `cargo test` rechaza un término que no esté en el glosario: en kaqchikel no entra nada
   sin procedencia.
3. Comprobar: `node scripts/idiomas/comprobar.js`.
4. Cuando una persona hablante lo haya revisado: `"estado": "revisado"` y su nombre en
   `"revisado_por"`. El aviso de borrador deja de mostrarse.

## Sumar otro idioma

Copiar `cak.json` con el código ISO 639 del idioma (por ejemplo `quc.json` para k'iche'), cambiar
`idioma`, `nombre`, `nombre_es` y `estado`, y llenar las traducciones. El selector lo muestra solo.

Más detalle: [`docs/29-idiomas-e-inclusion.md`](../../../docs/29-idiomas-e-inclusion.md).
