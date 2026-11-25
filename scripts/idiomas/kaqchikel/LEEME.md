# Cómo se armó el borrador en kaqchikel

Estos guiones repiten el proceso con el que se armaron los 680 rótulos en kaqchikel de la pantalla
(`datos/estandar/idiomas/cak.json`). El resultado es un **borrador sin revisar**: garantiza que
cada palabra consta en una fuente escrita por hablantes, no que la gramática esté bien. Detalle en
[`docs/29-idiomas-e-inclusion.md`](../../../docs/29-idiomas-e-inclusion.md).

1. **Fuentes** (quedan en `datos/crudo/`, con su huella en el manifiesto):

   ```bash
   faro fuentes url --fuente mozilla-pontoon-cak --editor "Mozilla (Pontoon)" --licencia "MPL 2.0" https://pontoon.mozilla.org/translation-memory/cak.all-projects.tmx
   faro fuentes url --fuente mozilla-pontoon-cak --editor "Mozilla (Pontoon)" --licencia "MPL 2.0" https://pontoon.mozilla.org/translation-memory/es-MX.all-projects.tmx
   faro fuentes url --fuente almg-kaqchikel --editor "ALMG, Comunidad Lingüística Kaqchikel" --licencia "no declara licencia" https://almg.org.gt/wp-content/uploads/2026/09/Diccionario-2-Kaqchikel.pdf
   pdftotext datos/crudo/almg-kaqchikel/Diccionario-2-Kaqchikel.pdf trabajo/clk_plano.txt
   ```

2. **Memoria alineada** (kaqchikel, español e inglés por mensaje): `unir_memoria.py`.
3. **Paquetes de evidencia**, uno por rótulo de `plantilla.json`: `paquetes.py`.
4. **Redacción.** Cada paquete se redacta siguiendo [`INSTRUCCIONES.md`](INSTRUCCIONES.md): solo
   palabras que estén en la evidencia; `null` si falta respaldo en el sentido correcto. En el
   Datatón lo hizo el asistente de IA declarado en el README, en seis partes. **Lo ideal es que lo
   haga, o lo revise, una persona hablante.**
5. **Validación y escritura**: `armar.py --escribir` descarta todo rótulo con una palabra que no
   esté en las fuentes y escribe `cak.json`, `cak_revision.csv` y `cak_palabras.csv`.
6. `node scripts/idiomas/comprobar.js` y `cargo test`.

`terminos.json` (términos comprobados a mano en las fuentes de la ALMG, con su página) no se
incluye: se rehace leyendo el diccionario; sin él los paquetes llevan igual las entradas del
diccionario.
