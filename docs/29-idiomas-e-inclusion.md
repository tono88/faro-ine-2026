# Idiomas e inclusión

*FARO · Que cada dato llegue a puerto.* Un dato no llega a puerto si quien lo necesita no puede leer
la pantalla que lo muestra. Este documento dice qué hace FARO para que más personas puedan usarlo,
qué está comprobado y qué falta.

## 1. De un vistazo

| Idioma de la pantalla | Estado | Cuánto cubre | De dónde sale |
|---|---|---|---|
| **Español** | Idioma de origen | 100 % | Así está escrito el portal |
| **Inglés** | Completo | 100 % de los 1,873 textos de la pantalla | Traducción del equipo con el asistente de IA declarado en el README; sin revisión de un traductor profesional |
| **Kaqchikel** | **Borrador** | 32 rótulos (1.7 %) | Solo términos que ya existen, escritos y revisados por hablantes, en una fuente abierta. Lo demás aparece en español |

El idioma se cambia con el botón del globo, en el encabezado, en cualquier pantalla y sin iniciar
sesión. La elección se recuerda en ese navegador. Un enlace puede fijarla: `…/?idioma=en`.

**La regla:** se traduce la **interfaz**, nunca los **datos**.

## 2. Qué se traduce y qué no

| Se traduce | No se traduce |
|---|---|
| Menú, títulos, botones, etiquetas de los campos | Nombres de departamentos, municipios y lugares poblados |
| Explicaciones y notas de cada sección | Nombres de instituciones, tiendas y productos |
| Los recorridos guiados (todas las secciones) | Valores de los catálogos que envían las instituciones |
| Encabezados de las tablas y nombres de los indicadores | Las cifras (pasan tal cual, con su mismo formato) |
| Avisos, confirmaciones y mensajes de error de la pantalla | Las citas bibliográficas y los títulos oficiales de las publicaciones |
| El comparativo «Antes y ahora» y los reportes imprimibles | Los archivos que se descargan (CSV, GeoJSON) y lo que guarda la base |

Por qué así:

- **Traducir un dato es alterarlo.** «San Juan Sacatepéquez» es un nombre oficial con un código; si la
  pantalla lo cambiara, dejaría de coincidir con la fuente. Es la misma regla de todo FARO: el
  sistema no adivina ni reescribe lo que viene de quien produce el dato.
- **Lo que se descarga es idéntico en cualquier idioma.** El idioma vive solo en el navegador de
  quien mira: no toca la base, ni las capas publicadas, ni la bitácora.
- **Es lo que manda la ley para los nombres.** La Ley de Idiomas Nacionales pide respetar la
  escritura propia de los nombres de lugares (ver la sección 7).

## 3. Cómo funciona

El portal sigue escrito en español. Un guion aparte, `crates/faro-servidor/web/idiomas.js`, traduce
la pantalla al vuelo con un diccionario cuya **clave es el propio texto en español**:

```json
"Revisar pendientes <1/>": "Review pending items <1/>",
"{0} filas cargadas; {1} lugares sin reconocer.": "{0} rows loaded; {1} places not recognized."
```

- Los **números** se cambian por `#` al buscar la frase y se devuelven tal cual a la traducción: «10,034
  registros» y «51 registros» son la misma frase, y la cifra nunca pasa por el diccionario.
- Las **etiquetas** dentro de una frase (una negrita, un enlace, un campo) se reutilizan: no se
  pierden los enlaces ni lo que hace cada botón.
- Un tramo `{0}` es un **dato** dentro de la frase (un nombre de lugar, de institución, de archivo): pasa
  sin cambios.
- Una palabra con dos sentidos lleva **contexto**: «media» es el promedio, y `confianza|media` es el
  nivel medio de confianza.
- Lo que la pantalla pinta después (tablas, avisos, fichas del mapa) se traduce al aparecer.

Los diccionarios son **datos, no código**: `datos/estandar/idiomas/<código>.json`. El servidor los
lee del disco en cada pedido (`/api/idiomas`, `/api/idiomas/<código>`), así que **una corrección se
ve al recargar la página, sin programar y sin reiniciar**.

No hay servicio de traducción: es un diccionario fijo. El mismo texto da siempre la misma
traducción, funciona sin internet y ningún texto del portal sale del servidor del INE.

### Cómo se protege el dato

1. **Solo cambia lo que está en el diccionario, y en el diccionario solo hay textos de la pantalla.**
   La lista de esos textos (`plantilla.json`, 1,873) la arma `scripts/idiomas/extraer.js`
   recorriendo el portal: un texto entra si está escrito en el código de la pantalla o en los textos
   propios del servidor. En la misma pasada quedaron fuera 2,394 textos que eran datos (lugares,
   productos, tiendas).
2. **Dentro de las tablas no se aplican moldes** (`{0}`): solo coincidencias exactas. Así una frase
   con hueco no puede calzar por casualidad con el nombre de un producto.
3. **Los nombres de personas y las citas van marcados** (`data-no-traducir`): no se tocan aunque
   coincidan con una palabra de la pantalla.
4. **Una traducción mal formada se descarta sola.** Si no conserva los mismos números, etiquetas y
   datos que su clave, el portal deja el texto en español. `cargo test` y
   `scripts/idiomas/comprobar.js` lo revisan antes de cada entrega.

Salvedad: la protección es por diccionario. Si un valor de un archivo coincidiera letra por letra
con una frase de la pantalla, se mostraría traducido en la pantalla (nunca en la base ni en las
descargas). Para que no pase con palabras sueltas, una palabra vista solo dentro de tablas no entra
en la plantilla.

## 4. Kaqchikel: por qué es un borrador

El equipo no habla kaqchikel y no hay manera de comprobar una traducción automática a un idioma
maya. Presentar como kaqchikel un texto que nadie revisó sería inventar, y además una falta de
respeto a sus hablantes. Por eso:

- **Solo se usan términos con fuente.** Cada rótulo en kaqchikel existe ya, con el mismo sentido, en
  la localización comunitaria de **Firefox** al kaqchikel (`cak`), hecha y revisada por hablantes y
  publicada con licencia MPL 2.0. El glosario `datos/estandar/idiomas/cak_glosario.csv` trae 71
  términos, cada uno con su archivo y su mensaje de origen: 32 ya se ven en la pantalla (los que
  coinciden palabra por palabra con un rótulo de FARO: «Guardar», «Cancelar», «Cerrar sesión»,
  «Contraseña»…) y el resto queda de reserva para quien complete la traducción.
- **No se componen frases.** No se unen palabras del glosario para formar oraciones nuevas: la
  gramática la pone una persona hablante.
- **Se dice en la pantalla.** Al elegir kaqchikel aparece un aviso: es un borrador, lo que falta está
  en español y los datos no se traducen. El selector muestra el porcentaje real.
- **`cargo test` no deja entrar un término sin fuente**: cada entrada de `cak.json` debe estar en el
  glosario.

Faltan sobre todo los términos propios de la estadística (población, municipio, precio, mapa): no
están en esa fuente y FARO no los inventa.

### Cómo se completa (sin programar)

1. Abrir `datos/estandar/idiomas/cak.json`: trae los 1,873 textos de la pantalla, vacíos los
   que faltan.
2. Escribir la traducción a la derecha de cada texto, dejando igual las marcas (`<1>…</1>`, `<1/>`),
   los `#` y los `{0}`.
3. Anotar en `cak_glosario.csv` cada término nuevo, con su fuente o con el nombre de quien lo
   revisó.
4. Correr `node scripts/idiomas/comprobar.js`: dice cuánto cubre y si alguna entrada quedó mal.
5. Cuando una persona hablante lo haya revisado, cambiar `"estado": "borrador"` por `"revisado"` y
   poner su nombre en `"revisado_por"`. El aviso desaparece.

A quién acudir: la Academia de Lenguas Mayas de Guatemala y su Comunidad Lingüística Kaqchikel son
la autoridad en la escritura del idioma. El equipo no los ha contactado; es el siguiente paso si el
INE adopta la herramienta.

## 5. Inglés

Lo tradujo el equipo con el asistente de IA que está declarado en el README, a partir de un
glosario fijo (envío → *submission*, homologar → *harmonize*, lugar poblado → *populated place*…).
No lo ha revisado un traductor profesional. Las siglas, los nombres de instituciones y los títulos
oficiales se quedan en español. Se corrige editando `en.json`.

## 6. Sumar otro idioma

K'iche', Q'eqchi', Mam o cualquier otro se suma igual, sin tocar el código:

```bash
cp datos/estandar/idiomas/cak.json datos/estandar/idiomas/quc.json   # código ISO 639 del idioma
# editar nombre, estado y las traducciones
node scripts/idiomas/comprobar.js --completar
```

El selector lo muestra solo, con su estado y su cobertura. Si la pantalla cambia, se vuelve a correr
`scripts/idiomas/extraer.js` (arma la plantilla) y `comprobar.js --completar` (agrega a cada
diccionario lo nuevo, vacío).

## 7. Marco legal

La Ley de Idiomas Nacionales (Decreto 19-2003 del Congreso de la República) reconoce los idiomas
mayas, garífuna y xinka. Lo que toca a una herramienta como esta:

| Artículo | Qué dice | Qué hace FARO |
|---|---|---|
| 8. Utilización | Los idiomas mayas, garífuna y xinka pueden usarse sin restricciones en el ámbito público y privado | La pantalla admite cualquier idioma como un archivo más |
| 10. Estadísticas | Las instituciones del Estado deben llevar y reportar datos sobre la pertenencia sociolingüística de quienes usan sus servicios | Una solicitud puede pedir esa variable y homologarla contra su catálogo, como ya se hace con «pueblo» |
| 11. Registros | La escritura propia de los nombres de personas y de lugares debe respetarse en los actos registrales | Los nombres de lugares no se traducen ni se reescriben en ningún idioma |
| 14 y 15. Servicios públicos | El Estado vela por que se informe y se atienda en el idioma de cada comunidad lingüística, de forma gradual | Kaqchikel empieza como borrador con fuente; falta la revisión de hablantes |

Esto no es un dictamen: describe qué hace la herramienta frente al texto de la ley. El detalle de
las demás normas está en [`23-conformidad-legal.md`](23-conformidad-legal.md).

## 8. Más allá del idioma

Lo que ya estaba y también es inclusión, con su comprobación:

- **Funciona en el teléfono.** Las 24 secciones se revisan en seis tamaños de pantalla, de un
  celular de 360 px a una laptop Full HD: sin desborde, sin texto de menos de 11 px y con controles
  que se pueden tocar ([`20-verificacion-interfaz.md`](20-verificacion-interfaz.md)).
- **Se explica solo.** Cada sección tiene un recorrido guiado, paso a paso, con un botón siempre a
  la vista; también están traducidos.
- **No pide instalar ni pagar nada.** Quien envía datos desde una institución solo necesita un
  navegador y su cuenta; no hay licencias ni servicios de pago.
- **Lenguaje llano.** La pantalla dice «lugares por revisar», no términos de programación; el foco
  del teclado es visible y los botones tienen nombre para los lectores de pantalla.
- **Respeta a quien no quiere animaciones** (`prefers-reduced-motion`) y se puede imprimir.

Lo que **no** se ha hecho: una auditoría formal de accesibilidad (WCAG) con lectores de pantalla y
personas usuarias; la medición del contraste de cada color; versiones en otros idiomas nacionales.

## 9. Cómo se comprueba

| Qué | Con qué |
|---|---|
| Cada traducción conserva números, etiquetas y datos; el kaqchikel solo trae términos del glosario; el inglés cubre al menos 99 % | `cargo test` (`los_diccionarios_conservan_marcas_numeros_y_datos`) |
| El código de idioma no deja leer otros archivos del servidor | `cargo test` (`el_codigo_de_idioma_no_deja_salir_de_la_carpeta`) |
| En un navegador real: el menú, los recorridos, los avisos y el reporte cambian de idioma; **los nombres de los departamentos, las cifras y las citas no cambian**; el encabezado cabe en un teléfono; el kaqchikel avisa que es borrador | `.faro/pruebas/idiomas.js` (37 comprobaciones) |
| Cobertura de cada idioma y entradas mal formadas | `node scripts/idiomas/comprobar.js` |
| La pantalla en inglés no se desborda en ningún tamaño (25-nov-2026: 24 secciones en seis tamaños, sin hallazgos) | `FARO_IDIOMA=en node .faro/pruebas/verificacion_interfaz.js <url>` |

## 10. Límites

- El kaqchikel es un comienzo, no una traducción: 1.7 % de la pantalla.
- El inglés no tiene revisión profesional.
- Los documentos (`docs/`), el manual y la presentación están solo en español.
- Los mensajes que arma el servidor con un dato en medio pueden salir en español si su forma no
  está prevista en el diccionario; el resto de la pantalla sigue traducida.
- El selector de archivos («Elegir archivo») lo dibuja el navegador en su propio idioma.

## Fuentes

- Mozilla y colaboradores de la localización al kaqchikel. *Firefox, localización `cak`*.
  <https://github.com/mozilla-l10n/firefox-l10n/tree/main/cak> y *Mozilla Accounts, localización
  `cak`*. <https://github.com/mozilla/fxa-content-server-l10n/tree/main/locale/cak>. Licencia MPL 2.0.
  Descargados el 25 de noviembre de 2026; cada archivo está en `datos/crudo/manifiesto.jsonl` con su
  huella.
- Congreso de la República de Guatemala. (2003). *Ley de Idiomas Nacionales, Decreto 19-2003*.
  Texto consultado el 25 de noviembre de 2026 en
  <https://digi.usac.edu.gt/bvsalud/documentos/eblueinfo/ML/ML_016.pdf>.
