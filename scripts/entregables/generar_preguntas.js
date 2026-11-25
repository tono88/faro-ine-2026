// Genera entregables/FARO-preguntas-y-respuestas.pdf a partir de docs/30-preguntas-del-jurado.md:
// las preguntas probables del jurado con su respuesta y, en las difíciles, la pregunta de seguimiento.
// No necesita el portal en marcha. Uso: node scripts/entregables/generar_preguntas.js
const { chromium } = require('playwright'); const fs = require('fs'); const path = require('path');
const { aHtml, esc } = require('./generar_pdf.js'); const T = require('./_tipografia.js');
const raiz = path.resolve(__dirname, '../..');
(async () => {
  const md = T.sinPie(fs.readFileSync(path.join(raiz, 'docs/30-preguntas-del-jurado.md'), 'utf8'));
  const version = (fs.readFileSync(path.join(raiz, 'docs/08-versiones.md'), 'utf8').match(/## (v[\d.]+)/) || [])[1] || '';
  // El cuerpo va sin el título (lo lleva la portada). Cada pregunta (h4) con lo que le sigue es un bloque.
  let cuerpo = aHtml(md.replace(/^# .*\n/, ''));
  const preguntas = (cuerpo.match(/<h4>/g) || []).length, seguimientos = (cuerpo.match(/<b>Seguimiento:/g) || []).length;
  cuerpo = cuerpo.replace(/<p><b>Seguimiento: ([^<]*)<\/b>/g, '<p class="seg"><span>Seguimiento</span><b>$1</b><br>')
    .replace(/<h4>([\d.]+) /g, '<h4><i>$1</i>').replace(/<h4>/g, '</div><div class="q"><h4>').replace(/<h3>/g, '</div><h3>').replace('</div>', '') + '</div>';
  const html = `<!doctype html><html lang="es"><meta charset="utf-8"><title>FARO · Preguntas probables del jurado</title><style>
    ${T.CSS}
    @page { size: Letter; margin: 17mm 16mm 18mm; } body { font: 10.5pt/1.45 "Ubuntu Sans", "DejaVu Sans", Arial, sans-serif; color: #313131; }
    .portada { height: 236mm; display: flex; flex-direction: column; justify-content: center; border-left: 8px solid #0080ff; padding-left: 22px; break-after: page; }
    .portada h1 { font-size: 30pt; color: #004080; margin: 6mm 0 2mm; line-height: 1.1; } .portada p { color: #5b6672; font-size: 12pt; max-width: 150mm; margin: 2mm 0; } .portada .cifras { display: flex; gap: 6mm; margin: 8mm 0 4mm; } .portada .cifras div { background: #e5f2ff; border-top: 4px solid #ffd500; border-radius: 6px; padding: 4mm 6mm; } .portada .cifras b { display: block; font-size: 22pt; color: #004080; line-height: 1.1; } .portada .cifras span { font-size: 9.5pt; color: #004080; }
    h3 { font-size: 15pt; color: #004080; border-bottom: 2px solid #0080ff; padding-bottom: 3px; margin: 9mm 0 3mm; break-after: avoid; }
    .q { border: 1px solid #b3d9ff; border-left: 5px solid #0080ff; border-radius: 6px; padding: 3mm 4.5mm 1.5mm; margin: 0 0 4mm; break-inside: avoid; }
    .q h4 { font-size: 11.5pt; color: #004080; margin: 0 0 1.5mm; } .q h4 i { font-style: normal; display: inline-block; background: #004080; color: #fff; border-radius: 4px; font-size: 9pt; padding: 0 5px; margin-right: 6px; vertical-align: 1px; }
    .q p { margin: 0 0 2mm; } .q p.seg { background: #fffbe5; border-left: 4px solid #ffd500; border-radius: 0 5px 5px 0; padding: 2mm 3.5mm; margin: 2.5mm 0; } .q p.seg span { display: inline-block; font-size: 7.5pt; font-weight: 700; letter-spacing: 1px; text-transform: uppercase; color: #004080; background: #ffd500; border-radius: 3px; padding: 0 5px; margin-right: 6px; vertical-align: 1px; } .q p.seg b { color: #004080; }
    em { color: #5b6672; } table { border-collapse: collapse; width: 100%; font-size: 9.5pt; margin: 2mm 0 4mm; } th, td { border: 1px solid #d9dfe6; padding: 4px 7px; text-align: left; vertical-align: top; } th { background: #e5f2ff; color: #004080; } tr { break-inside: avoid; }
    code { background: #e5f2ff; padding: 0 3px; border-radius: 3px; font-size: 9pt; } ul { margin: 1mm 0 3mm; padding-left: 5mm; } li { margin-bottom: 1.2mm; }
    .fin { break-before: page; display: flex; flex-direction: column; justify-content: center; height: 228mm; text-align: center; } .fin h3 { border: 0; } .fin p { color: #5b6672; } .fin .carta { width: 170mm; margin: 8mm auto 0; } .fin .carta svg { display: block; width: 100%; height: auto; }
  </style><body>
  <div class="portada"><img src="data:image/svg+xml;base64,${fs.readFileSync(path.join(raiz, 'docs/identidad/faro-icono.svg')).toString('base64')}" style="width:30mm"><h1>Preguntas probables del jurado<br>y cómo responderlas</h1><p style="font-size:15pt;color:#004080;font-weight:700">FARO · Que cada dato llegue a puerto</p><p>Guía de ensayo del equipo La Sociedad de Almas para los tres minutos de preguntas de la final.</p>
  <div class="cifras"><div><b>${preguntas}</b><span>preguntas con respuesta</span></div><div><b>${seguimientos}</b><span>preguntas de seguimiento</span></div><div><b>3 min</b><span>para responder</span></div></div>
  <p>Datatón INE 2026 · Categoría 2: Tecnológica y ciencias de datos<br>Versión ${esc(version)} · ${new Date().toLocaleDateString('es-GT', { day: 'numeric', month: 'long', year: 'numeric' })}</p><p style="margin-top:16mm;font-size:10pt">Proyecto participante del Datatón INE 2026</p><img src="${T.ine('ine-horizontal')}" alt="Instituto Nacional de Estadística" style="width:52mm;margin-top:2mm"></div>
  <div>${cuerpo.replace(/\*(\([^*]+?\))\*/g, '<em>$1</em>').replace(/\*([^*<]+)\*/g, '<em>$1</em>')}
  <div class="fin"><h3>Equipo La Sociedad de Almas</h3><p>Que cada dato llegue a puerto.</p><div class="carta">${T.carta()}</div></div></body></html>`;
  const b = await chromium.launch(); const d = await b.newPage(); await d.setContent(html, { waitUntil: 'load' });
  const salida = path.join(raiz, 'entregables', 'FARO-preguntas-y-respuestas.pdf');
  await d.pdf({ path: salida, format: 'Letter', printBackground: true, displayHeaderFooter: true, headerTemplate: '<span></span>', footerTemplate: '<div style="font-size:8px;color:#5b6672;width:100%;text-align:center">FARO · Preguntas probables del jurado · <span class="pageNumber"></span> de <span class="totalPages"></span></div>', margin: { top: '17mm', bottom: '18mm', left: '16mm', right: '16mm' } });
  console.log('escrito', salida, Math.round(fs.statSync(salida).size / 1024), 'KB ·', preguntas, 'preguntas,', seguimientos, 'de seguimiento'); await b.close();
})();
