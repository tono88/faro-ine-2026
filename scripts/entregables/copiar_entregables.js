// Copia a la carpeta 00-ENTREGABLES de la raíz los entregables que pide la nota conceptual, y solo
// esos (repositorio, presentación en PDF y video), numerados, para que sean lo primero que se ve en el repositorio. Los originales siguen en
// entregables/ (ahí los escriben los generadores). Uso: node scripts/entregables/copiar_entregables.js
const fs = require('fs'), path = require('path'), crypto = require('crypto');
const raiz = path.resolve(__dirname, '../..'), de = path.join(raiz, 'entregables'), a = path.join(raiz, '00-ENTREGABLES');
const LISTA = [
  ['FARO-presentacion.pdf', '2-Presentacion-FARO.pdf', 'Presentación en formato PDF para la exposición ante el jurado (16 diapositivas)', true],
  ['FARO-video-demo.mp4', '3-Video-demostrativo-FARO.mp4', 'Video demostrativo de 2 a 3 minutos, grabado del portal en marcha', true],
];
fs.mkdirSync(a, { recursive: true });
for (const f of fs.readdirSync(a)) if (f !== 'LEEME.md' && !LISTA.some(l => l[1] === f)) fs.unlinkSync(path.join(a, f));
const huella = f => crypto.createHash('sha256').update(fs.readFileSync(f)).digest('hex').slice(0, 12);
const filas = LISTA.map(([origen, destino, que, pedido]) => { fs.copyFileSync(path.join(de, origen), path.join(a, destino)); const kb = Math.round(fs.statSync(path.join(a, destino)).size / 1024);
  return `| ${destino[0]} | [\`${destino}\`](${destino}) | ${que} | ${kb.toLocaleString('es-GT')} KB | \`${huella(path.join(a, destino))}\` |`; });
const version = (fs.readFileSync(path.join(raiz, 'docs/08-versiones.md'), 'utf8').match(/## (v[\d.]+)/) || [])[1] || '';
fs.writeFileSync(path.join(a, 'LEEME.md'), `# Entregables de FARO · Datatón INE 2026, categoría 2

Equipo **La Sociedad de Almas**: Estuardo Sandoval · Ana Echeverría · Alejandro Sandoval. Versión ${version}.

Esta carpeta reúne **los tres entregables que pide la nota conceptual** («Entregables y presentación»,
categoría 2), numerados, y nada más. Los dos archivos son copia de los de [\`entregables/\`](../entregables/).

| N.º | Archivo | Qué es | Tamaño | Huella (SHA-256, inicio) |
|---|---|---|---|---|
| 1 | [Repositorio](https://github.com/tono88/faro-ine-2026) y [\`README.md\`](../README.md) | Enlace al repositorio con el código fuente y la documentación (carpeta [\`docs/\`](../docs/), 31 documentos) | — | — |
${filas.join('\n')}

**Para evaluar:** el portal trae una **«Guía para evaluadores»** al inicio del menú (en rojo): dice qué revisar en
cada criterio de la nota conceptual y dónde está. Portal en línea: <https://faro.test-tecnodyne.site/> (despliegue del
equipo); para instalarlo en su máquina, vea «Cómo desplegarlo» en el [\`README.md\`](../README.md).

El video no lleva narración grabada: sus subtítulos y el guion de la narradora están en
[\`entregables/FARO-video-demo.srt\`](../entregables/FARO-video-demo.srt) y [\`docs/16-guion-video.md\`](../docs/16-guion-video.md).

Material de apoyo, que no es entregable: manual de usuario, documento técnico, presentación editable y
preguntas probables, en [\`entregables/\`](../entregables/) y [\`docs/\`](../docs/).
`);
console.log(LISTA.length, 'entregables copiados a 00-ENTREGABLES');
