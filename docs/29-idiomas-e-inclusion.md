# Idiomas e inclusión

*FARO · Que cada dato llegue a puerto.* Un dato no llega a puerto si quien lo necesita no puede leer
la pantalla que lo muestra. Este documento dice qué hace FARO para que más personas puedan usarlo,
qué está comprobado y qué falta.

## 1. De un vistazo

| Idioma de la pantalla | Estado | Cuánto cubre | De dónde sale |
|---|---|---|---|
| **Español** | Idioma de origen | 100 % | Así está escrito el portal |
| **Inglés** | Completo | 100 % de los 1,995 textos de la pantalla | Traducción del equipo con el asistente de IA declarado en el README; sin revisión de un traductor profesional |
| **Kaqchikel** | **Borrador sin revisar** | 680 rótulos (34.1 %): menú, botones, títulos, encabezados | Armado con vocabulario escrito por hablantes (Mozilla y la Academia de Lenguas Mayas). Falta la revisión de una persona experta. Lo demás aparece en español |

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
   La lista de esos textos (`plantilla.json`, 1,995) la arma `scripts/idiomas/extraer.js`
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

## 4. Kaqchikel: un borrador amplio, por revisar

El equipo no habla kaqchikel. El pedido fue tener en pantalla una parte considerable del portal en
ese idioma y dejar dicho que debe completarse con la asesoría de una persona experta. Así se hizo,
y esto es exactamente lo que hay y lo que no.

**Cómo se armó**

1. **Fuentes escritas por hablantes.**
   - La **memoria de traducción de Mozilla al kaqchikel** (Pontoon): 59,421 frases de Firefox,
     Thunderbird, Common Voice y otros programas, traducidas y revisadas por su comunidad de
     hablantes; 53,261 se pudieron alinear con su versión en español. Licencia MPL 2.0.
   - El ***Kaqchikel Choltzij: Kolon chuqa' K'ak'a' taq Tzij***, vocabulario de neologismos y
     palabras rescatadas de la Comunidad Lingüística Kaqchikel de la Academia de Lenguas Mayas de
     Guatemala (2011, 401 páginas), con secciones de ambiente social, matemáticas e informática. De
     ahí salen los términos que un navegador no tiene: población (*winaqilem*), municipio
     (*tinamït*), hogar (*ochochil*), mapa (*ruwäch setul*), crecimiento (*nimirisanem*).
   - Como apoyo, textos legales traducidos por la propia Academia (Ley de Idiomas Nacionales y Ley
     de Acceso a la Información Pública).
2. **Un paquete de evidencia por rótulo.** Para cada uno de los 1,135 rótulos cortos de la pantalla
   se reunieron las frases de esas fuentes que contienen cada una de sus palabras.
3. **Redacción con el asistente de IA, con una regla:** usar solo palabras que aparezcan en la
   evidencia, cuidar el sentido (un «registro» de programa no es el registro de una persona) y
   dejar el rótulo en español si a alguna palabra le falta respaldo en el sentido correcto.
4. **Comprobación automática.** Un programa descartó todo rótulo con una palabra que no conste en
   las fuentes. Quedaron **680** (34.1 % de la pantalla); **455** se dejaron en español por falta de
   respaldo.

**Qué tan confiable es**

| Nivel | Cuántos | Qué significa |
|---|---|---|
| `fuente` | 110 | El rótulo completo existe igual en la fuente, con el mismo sentido («Guardar» → *Tiyak*, «Cerrar sesión» → *Titz'apïx molojri'ïl*) |
| `compuesto` | 570 | Se armó combinando palabras de las fuentes («Población y crecimiento» → *Winaqilem chuqa' nimirisanem*). **La gramática no la ha revisado una persona hablante: puede estar mal** |

Las 437 palabras distintas que se usan tienen anotada su fuente: 378 vienen de la memoria de
Mozilla, 48 del diccionario de la Academia y 11 son siglas, nombres propios y unidades que se
copian igual.

Lo que esto garantiza es que **no hay palabras inventadas**. Lo que **no** garantiza es que cada
frase esté bien construida, que el término elegido sea el que usaría un estadístico kaqchikel, ni
que las variantes regionales estén bien resueltas. Por eso:

- **La pantalla lo dice.** Al elegir kaqchikel aparece un aviso: es un borrador sin revisar, puede
  tener errores y debe corregirse y completarse con asesoría experta antes de un uso oficial.
- **Hay una hoja de revisión.** `datos/estandar/idiomas/cak_revision.csv` trae cada rótulo con su
  nivel, las frases de las fuentes en que se basó y dos columnas vacías para la persona que
  revise: la corrección y su nombre.
- **Las dudas conocidas están anotadas.** Entre ellas: «tablero» (*pas*, que también es «panel»),
  «visita» de campo (*tz'etoj*, que en la fuente es la visita a una página), «envío» (*taqoj*, que
  también es «adjunto»), «precio de referencia» (*rajil retal*), «departamento» (*tinamital*, solo
  en Mozilla), los términos estadísticos tomados del diccionario sin ejemplo de uso (*nuk'umolaj*
  «muestreo», *junejqalem* «equivalencia», *runik'ajal* «mediana») y las discrepancias entre
  fuentes («gobierno», «ley», «estadística», «censo»).
- **Lo que no tiene fuente sigue en español**: «lugar poblado», «red vial», «canasta básica»,
  «homologar», «rubro», «tasa», «superficie construida», y todos los textos largos (explicaciones y
  recorridos guiados).
- **Los nombres no se traducen**: «Guatemala» y los nombres de departamentos y municipios quedan
  como los publica su fuente, aunque exista un nombre en kaqchikel.

### Cómo lo revisa y lo completa una persona experta (sin programar)

1. Abrir `cak_revision.csv` (se abre en Excel o LibreOffice). Empezar por los rótulos de nivel
   `compuesto` y por el menú.
2. Para corregir un rótulo: escribir la forma correcta en la columna `correccion` y su nombre en
   `revisado_por`; llevar esa corrección a `cak.json` (el texto a la derecha del español).
3. Para traducir lo que falta: llenar en `cak.json` las entradas vacías, dejando igual las marcas
   (`<1>…</1>`, `<1/>`), los `#` y los `{0}`, y agregar su fila a `cak_revision.csv`.
4. Una palabra nueva se anota en `cak_palabras.csv`, con su fuente o con el nombre de quien la
   avala: una persona experta **es** una fuente válida.
5. `node scripts/idiomas/comprobar.js` dice cuánto cubre y si algo quedó sin anotar.
6. Cuando la revisión esté completa: `"estado": "revisado"` y el nombre en `"revisado_por"`. El
   aviso de borrador deja de mostrarse.

A quién acudir: la Academia de Lenguas Mayas de Guatemala y su Comunidad Lingüística Kaqchikel son
la autoridad en la escritura del idioma, y la Ley de Idiomas Nacionales (art. 9) le encarga las
traducciones oficiales. El equipo no los ha contactado; es el siguiente paso si el INE adopta la
herramienta.

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
| 14 y 15. Servicios públicos | El Estado vela por que se informe y se atienda en el idioma de cada comunidad lingüística, de forma gradual | Kaqchikel en borrador en 34 % de los rótulos; falta la revisión de una persona experta |

Esto no es un dictamen: describe qué hace la herramienta frente al texto de la ley. El detalle de
las demás normas está en [`23-conformidad-legal.md`](23-conformidad-legal.md).

## 8. Más allá del idioma

Lo que ya estaba y también es inclusión, con su comprobación:

- **Funciona en el teléfono.** Las 25 secciones se revisan en seis tamaños de pantalla, de un
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
| Cada traducción conserva números, etiquetas y datos; cada rótulo en kaqchikel consta en la hoja de revisión y cada una de sus palabras en la lista con fuente; el inglés cubre al menos 99 % | `cargo test` (`los_diccionarios_conservan_marcas_numeros_y_datos`) |
| El código de idioma no deja leer otros archivos del servidor | `cargo test` (`el_codigo_de_idioma_no_deja_salir_de_la_carpeta`) |
| En un navegador real: el menú, los recorridos, los avisos y el reporte cambian de idioma; **los nombres de los departamentos, las cifras y las citas no cambian**; el encabezado cabe en un teléfono; el kaqchikel avisa que es un borrador sin revisar | `.faro/pruebas/idiomas.js` (41 comprobaciones) |
| Cobertura de cada idioma y entradas mal formadas | `node scripts/idiomas/comprobar.js` |
| La pantalla en inglés no se desborda en ningún tamaño (25-nov-2026: 25 secciones en seis tamaños, sin hallazgos) | `FARO_IDIOMA=en node .faro/pruebas/verificacion_interfaz.js <url>` |

## 10. Límites

- El kaqchikel es un borrador sin revisar: 34 % de la pantalla, con 570 rótulos compuestos cuya
  gramática no ha visto una persona hablante. No debe usarse como traducción oficial.
- El inglés no tiene revisión profesional.
- Los documentos (`docs/`), el manual y la presentación están solo en español.
- Los mensajes que arma el servidor con un dato en medio pueden salir en español si su forma no
  está prevista en el diccionario; el resto de la pantalla sigue traducida.
- El selector de archivos («Elegir archivo») lo dibuja el navegador en su propio idioma.

## Fuentes

- Mozilla y colaboradores de la localización al kaqchikel. *Memoria de traducción de los
  proyectos de Mozilla al kaqchikel (`cak`)*, Pontoon. <https://pontoon.mozilla.org/cak/>.
  Licencia MPL 2.0. Descargada el 25 de noviembre de 2026.
- Academia de Lenguas Mayas de Guatemala, Comunidad Lingüística Kaqchikel. (2011). *Kaqchikel
  Choltzij: Kolon chuqa' K'ak'a' taq Tzij*.
  <https://almg.org.gt/wp-content/uploads/2026/09/Diccionario-2-Kaqchikel.pdf>. No declara
  licencia; se usan términos sueltos, citando la fuente. Descargado el 25 de noviembre de 2026.
- Las dos descargas están en `datos/crudo/manifiesto.jsonl` con su huella.
- Congreso de la República de Guatemala. (2003). *Ley de Idiomas Nacionales, Decreto 19-2003*.
  Texto consultado el 25 de noviembre de 2026 en
  <https://digi.usac.edu.gt/bvsalud/documentos/eblueinfo/ML/ML_016.pdf>.

<!-- equipo -->
---

<p align="center"><img src="identidad/equipo/carta-de-presentacion.svg" alt="Equipo La Sociedad de Almas: Estuardo Sandoval, Ana Echeverría y Alejandro Sandoval, con el código QR del perfil de LinkedIn de cada uno" width="760"></p>

<p align="center"><a href="https://www.linkedin.com/in/estuardoantoniosandoval/">Estuardo Sandoval</a> · <a href="https://www.linkedin.com/in/analuciaecheverria/">Ana Echeverría</a> · <a href="https://www.linkedin.com/in/alejandro-jos%C3%A9-sandoval-acevedo-998781aa/">Alejandro Sandoval</a><br>Equipo La Sociedad de Almas · Datatón INE 2026</p>
