// Copia los entregables a la carpeta 00-ENTREGABLES de la raíz, numerados como los pide la nota
// conceptual, para que sean lo primero que se ve en el repositorio. Los originales siguen en
// entregables/ (ahí los escriben los generadores). Uso: node scripts/entregables/copiar_entregables.js
const fs = require('fs'), path = require('path'), crypto = require('crypto');
const raiz = path.resolve(__dirname, '../..'), de = path.join(raiz, 'entregables'), a = path.join(raiz, '00-ENTREGABLES');
const LISTA = [
  ['FARO-presentacion.pdf', '2-Presentacion-FARO.pdf', 'Presentación para la exposición ante el jurado (PDF, 16 diapositivas)', true],
  ['FARO-presentacion.pptx', '2-Presentacion-FARO-editable.pptx', 'La misma presentación, editable en PowerPoint, con notas del orador', false],
  ['FARO-video-demo.mp4', '3-Video-demostrativo-FARO.mp4', 'Video demostrativo de 2 a 3 minutos, grabado del portal en marcha', true],
  ['FARO-video-demo.srt', '3-Video-demostrativo-FARO-subtitulos.srt', 'Subtítulos del video', false],
  ['FARO-manual-de-usuario.pdf', '4-Manual-de-usuario-FARO.pdf', 'Manual de usuario gráfico, una página por sección', false],
  ['FARO-documento-tecnico.pdf', '5-Documento-tecnico-FARO.pdf', 'Documento técnico: ficha, conformidad, arquitectura, verificación, seguridad, fuentes', false],
  ['FARO-preguntas-y-respuestas.pdf', '6-Preguntas-y-respuestas-FARO.pdf', 'Preguntas probables del jurado con sus respuestas', false],
];
fs.mkdirSync(a, { recursive: true });
for (const f of fs.readdirSync(a)) if (f !== 'LEEME.md' && !LISTA.some(l => l[1] === f)) fs.unlinkSync(path.join(a, f));
const huella = f => crypto.createHash('sha256').update(fs.readFileSync(f)).digest('hex').slice(0, 12);
const filas = LISTA.map(([origen, destino, que, pedido]) => { fs.copyFileSync(path.join(de, origen), path.join(a, destino)); const kb = Math.round(fs.statSync(path.join(a, destino)).size / 1024);
  return `| ${destino[0]} | [\`${destino}\`](${destino}) | ${que}${pedido ? ' · **lo pide la nota conceptual**' : ''} | ${kb.toLocaleString('es-GT')} KB | \`${huella(path.join(a, destino))}\` |`; });
const version = (fs.readFileSync(path.join(raiz, 'docs/08-versiones.md'), 'utf8').match(/## (v[\d.]+)/) || [])[1] || '';
fs.writeFileSync(path.join(a, 'LEEME.md'), `# Entregables de FARO · Datatón INE 2026, categoría 2

Equipo **La Sociedad de Almas**: Estuardo Sandoval · Ana Echeverría · Alejandro Sandoval. Versión ${version}.

Esta carpeta reúne los entregables, numerados. Son copia de los que están en [\`entregables/\`](../entregables/)
(los generan los guiones de \`scripts/entregables/\` a partir del portal en marcha).

| N.º | Archivo | Qué es | Tamaño | Huella (SHA-256, inicio) |
|---|---|---|---|---|
| 1 | [Repositorio](https://github.com/tono88/faro-ine-2026) y [\`README.md\`](../README.md) | Código fuente y documentación (carpeta [\`docs/\`](../docs/), 31 documentos) · **lo pide la nota conceptual** | — | — |
${filas.join('\n')}

**Para evaluar:** el portal trae una **«Guía para evaluadores»** al inicio del menú (en rojo): dice qué revisar en
cada criterio de la nota conceptual y dónde está. Portal en línea: <https://faro.test-tecnodyne.site/> (despliegue del
equipo); para instalarlo en su máquina, vea «Cómo desplegarlo» en el [\`README.md\`](../README.md).

El video no lleva narración grabada: trae subtítulos, y el guion de la narradora está en
[\`docs/16-guion-video.md\`](../docs/16-guion-video.md).
`);
console.log(LISTA.length, 'entregables copiados a 00-ENTREGABLES');
