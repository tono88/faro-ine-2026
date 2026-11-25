// Genera el documento entregable en PDF (entregables/FARO-documento-tecnico.pdf) con capturas
// tomadas en el momento del portal en marcha. Requiere Node y Playwright con Chromium.
// Uso: node scripts/entregables/generar_pdf.js http://localhost:8082
const { chromium } = require('playwright'); const fs = require('fs'); const path = require('path');
const base = process.argv[2] || 'http://localhost:8082', raiz = path.resolve(__dirname, '../..'), dir = path.join(raiz, 'entregables'), cap = path.join(dir, 'capturas');
const esc = t => String(t).replace(/[&<>]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
// Markdown mínimo → HTML: títulos, tablas, listas, bloques de código, negritas, código y enlaces.
function aHtml(md) {
  const enlinea = t => esc(t).replace(/\*\*(.+?)\*\*/g, '<b>$1</b>').replace(/`([^`]+)`/g, '<code>$1</code>').replace(/\[([^\]]+)\]\(([^)]+)\)/g, '$1').replace(/&lt;(https?:[^&]+)&gt;/g, '$1');
  const out = []; const L = md.split('\n'); let i = 0;
  while (i < L.length) { const l = L[i];
    if (l.startsWith('```')) { const b = []; i++; while (i < L.length && !L[i].startsWith('```')) b.push(L[i++]); i++; out.push('<pre>' + esc(b.join('\n')) + '</pre>'); continue; }
    if (/^#{1,3} /.test(l)) { const n = l.match(/^#+/)[0].length; out.push(`<h${n + 1}>${enlinea(l.replace(/^#+ /, ''))}</h${n + 1}>`); i++; continue; }
    if (l.startsWith('|')) { const f = []; while (i < L.length && L[i].startsWith('|')) f.push(L[i++]); const celdas = r => r.slice(1, -1).split('|').map(c => c.trim());
      const cab = celdas(f[0]), cuerpo = f.slice(2).map(celdas); out.push('<table>' + (cab.some(c => c) ? '<thead><tr>' + cab.map(c => `<th>${enlinea(c)}</th>`).join('') + '</tr></thead>' : '') + '<tbody>' + cuerpo.map(r => '<tr>' + r.map(c => `<td>${enlinea(c)}</td>`).join('') + '</tr>').join('') + '</tbody></table>'); continue; }
    if (/^(\d+\.|-) /.test(l)) { const ord = /^\d+\./.test(l), it = []; while (i < L.length && (/^(\d+[a-z]?\.|-) /.test(L[i]) || /^ {2,}\S/.test(L[i]))) { if (/^(\d+[a-z]?\.|-) /.test(L[i])) it.push(L[i].replace(/^(\d+[a-z]?\.|-) /, '')); else it[it.length - 1] += ' ' + L[i].trim(); i++; } out.push(`<${ord ? 'ol' : 'ul'}>` + it.map(x => `<li>${enlinea(x)}</li>`).join('') + `</${ord ? 'ol' : 'ul'}>`); continue; }
    if (!l.trim()) { i++; continue; }
    const p = []; while (i < L.length && L[i].trim() && !/^(#|\||```|- |\d+\. )/.test(L[i])) p.push(L[i++].trim()); out.push('<p>' + enlinea(p.join(' ')) + '</p>'); }
  return out.join('\n');
}
(async () => {
  fs.mkdirSync(cap, { recursive: true });
  // Si la sesión sale a internet por un proxy, el navegador lo usa para las fotos satelitales (no para el portal).
  const proxy = process.env.HTTPS_PROXY; const b = await chromium.launch(proxy ? { proxy: { server: proxy, bypass: 'localhost,127.0.0.1' } } : {}); const p = await b.newPage({ viewport: { width: 1360, height: 860 }, deviceScaleFactor: 1 });
  await p.goto(base); await p.waitForSelector('#lienzo path.leaflet-interactive'); await p.waitForTimeout(800);
  const toma = async (n, pie) => { await p.screenshot({ path: path.join(cap, n + '.png') }); return { n, pie }; }; const tomas = [];
  tomas.push(await toma('01-mapa', 'Mapa por municipio: población proyectada por el INE.'));
  await p.fill('#buscalugar', 'panajachel'); await p.waitForSelector('#buscalista li[data-i]'); await p.keyboard.press('Enter'); await p.waitForSelector('#ficha svg'); await p.waitForTimeout(600);
  tomas.push(await toma('02-ficha', 'Ficha de un municipio: censo 2018 por lugar poblado, pirámide de edades y superficie construida.'));
  await p.click('#ficha .cerrar'); await p.click('nav > button[data-s=vial]'); await p.waitForSelector('#tvialenl button[data-ver]'); await p.waitForTimeout(500);
  tomas.push(await toma('03-red-vial', 'Red vial como grafo: lugares sin conexión y enlaces que probablemente faltan.'));
  await p.click('nav > button[data-s=rubros]'); await p.waitForSelector('#trubros tbody tr td a', { timeout: 30000 }); await p.click('#rubrovistas button[data-v=diferencias]'); await p.waitForTimeout(1500);
  tomas.push(await toma('04-diferencias', 'Precios por rubro: diferencias de 50 % o más entre tiendas, explicadas por regla o por confirmar.'));
  await p.click('nav > button[data-s=balance]'); await p.waitForSelector('#thba tbody tr td b'); await p.waitForTimeout(500);
  tomas.push(await toma('05-balance', 'Hoja de balance: tasas de la FAO para Guatemala contrastadas con la HBA 2024 del INE.'));
  await p.click('nav > button[data-s=tableros]'); await p.waitForSelector('#tabejemplos button'); await p.click('#tabejemplos button'); await p.waitForSelector('#tabvista .pieza svg path'); await p.evaluate(() => document.querySelector('#tabvista').scrollIntoView()); await p.waitForTimeout(600);
  tomas.push(await toma('07-tablero', 'Tablero armado a partir de una frase escrita por el usuario.'));
  await p.click('nav > button[data-s=proceso]'); await p.waitForTimeout(1500);
  tomas.push(await toma('06-proceso', 'Proceso: qué hace cada etapa y quién.'));
  await p.click('nav > button[data-s=mejoras]'); await p.waitForSelector('.mejora'); await p.waitForTimeout(400);
  tomas.push(await toma('08-antes-y-ahora', 'Antes y ahora: qué se tenía, qué hace FARO con una cifra comprobable y cómo escala.'));
  await p.evaluate(() => detalle('0101')); await p.waitForSelector('#animlienzo'); await p.waitForFunction(() => /Ver de nuevo/.test(document.querySelector('#animplay').textContent), null, { timeout: 40000 }); await p.waitForTimeout(2500);
  tomas.push(await toma('09-animacion', 'Crecimiento urbano del municipio de Guatemala sobre la foto satelital (si no hay internet, sin foto).')); await p.click('#detalle .cerrar');
  // Capturas del trabajo de campo: las deja el guion .faro/pruebas/trabajo_de_campo.js cuando se le da esta carpeta.
  for (const [n, pie] of [['101-campo-coordinacion', 'Trabajo de campo: quien coordina arma la ruta de cada operador y revisa las visitas.'], ['103-campo-ruta-movil', 'Trabajo de campo en el teléfono del operador: sus paradas en orden y el resultado de cada visita.'], ['102b-campo-foto-comercio', 'Una visita con foto de comercio queda marcada para revisar, con la razón.']]) if (fs.existsSync(path.join(cap, n + '.png'))) tomas.push({ n, pie });
  const leer = f => fs.readFileSync(path.join(raiz, f), 'utf8'); const img = n => 'data:image/png;base64,' + fs.readFileSync(path.join(cap, n + '.png')).toString('base64');
  const figura = t => `<figure><img src="${img(t.n)}"><figcaption>${esc(t.pie)}</figcaption></figure>`;
  const version = (leer('docs/08-versiones.md').match(/## (v[\d.]+)/) || [])[1] || '';
  const html = `<!doctype html><html lang="es"><meta charset="utf-8"><title>FARO · Documento técnico</title><style>
    ${require('./_tipografia.js').CSS}
    @page { size: Letter; margin: 18mm 16mm; } body { font: 10.5pt/1.45 "Ubuntu Sans", "DejaVu Sans", Arial, sans-serif; color: #313131; }
    h1 { font-size: 26pt; color: #004080; margin: 0; } h2 { font-size: 15pt; color: #004080; border-bottom: 2px solid #0080ff; padding-bottom: 3px; margin: 22px 0 8px; break-after: avoid; } h3 { font-size: 12pt; color: #004080; margin: 14px 0 4px; break-after: avoid; } h4 { font-size: 10.5pt; color: #004080; margin: 10px 0 2px; }
    table { border-collapse: collapse; width: 100%; font-size: 9pt; margin: 6px 0 10px; } th, td { border: 1px solid #d9dfe6; padding: 4px 6px; text-align: left; vertical-align: top; } th { background: #e5f2ff; color: #004080; } tr { break-inside: avoid; }
    code { background: #e5f2ff; padding: 0 3px; border-radius: 3px; font-size: 9pt; } pre { background: #f4f6f9; border: 1px solid #d9dfe6; padding: 8px; font-size: 8.5pt; white-space: pre-wrap; }
    figure { margin: 10px 0; break-inside: avoid; } figure img { display: block; max-width: 100%; max-height: 118mm; margin: 0 auto; border: 1px solid #d9dfe6; } figcaption { font-size: 9pt; color: #5b6672; margin-top: 2px; }
    .portada { height: 240mm; display: flex; flex-direction: column; justify-content: center; border-left: 8px solid #0080ff; padding-left: 22px; break-after: page; } .portada p { color: #5b6672; font-size: 12pt; max-width: 150mm; } .sec { break-before: page; }
  </style><body>
  <div class="portada"><img src="data:image/svg+xml;base64,${fs.readFileSync(path.join(raiz, 'docs/identidad/faro-icono.svg')).toString('base64')}" style="width:34mm;margin-bottom:8mm"><h1>FARO</h1><p style="font-size:17pt;color:#004080;font-weight:700">Que cada dato llegue a puerto</p><p style="font-size:13pt;color:#004080">Registros administrativos, territorio y precios</p><p>Equipo La Sociedad de Almas</p><p>Herramienta para que el INE pida, reciba y homologue registros de otras instituciones, y los aproveche junto a sus datos abiertos.</p><p>Datatón INE 2026 · Categoría 2: Tecnológica y ciencias de datos<br>Documento técnico · versión ${esc(version)} · ${new Date().toLocaleDateString('es-GT', { day: 'numeric', month: 'long', year: 'numeric' })}</p><p>Repositorio: github.com/tono88/faro-ine-2026</p><p style="margin-top:22mm;font-size:10pt">Proyecto participante del Datatón INE 2026</p><img src="${require('./_tipografia.js').ine('ine-horizontal')}" alt="Instituto Nacional de Estadística" style="width:52mm;margin-top:2mm"></div>
  ${aHtml(leer('docs/18-ficha-tecnica.md'))}
  <div class="sec">${aHtml(leer('docs/26-conformidad-nota-conceptual.md'))}</div>
  <div class="sec">${aHtml(leer('docs/25-identidad.md').replace(/!\[[^\]]*\]\([^)]*\)\n/, ''))}</div>
  <div class="sec">${aHtml(leer('docs/00-que-se-hizo.md'))}</div>
  <div class="sec">${aHtml(leer('docs/22-antes-y-ahora.md'))}</div>
  <div class="sec"><h2>El portal en imágenes</h2>${tomas.map(figura).join('')}</div>
  <div class="sec">${aHtml(leer('docs/03-arquitectura.md'))}</div>
  <div class="sec">${aHtml(leer('docs/01-necesidades-ine.md'))}</div>
  <div class="sec">${aHtml(leer('docs/09-logros.md'))}</div>
  <div class="sec">${aHtml(leer('docs/19-verificacion-estadistica.md'))}</div>
  <div class="sec">${aHtml(leer('docs/20-verificacion-interfaz.md'))}</div>
  <div class="sec">${aHtml(leer('docs/21-seguridad.md'))}</div>
  <div class="sec">${aHtml(leer('docs/23-conformidad-legal.md'))}</div>
  <div class="sec">${aHtml(leer('docs/24-clasificador-de-fotos.md'))}</div>
  <div class="sec">${aHtml(leer('docs/10-imagenes-satelitales.md'))}</div>
  <div class="sec">${aHtml(leer('docs/14-manual-de-usuario.md'))}</div>
  <div class="sec">${aHtml(leer('docs/15-instalacion-y-dependencias.md'))}</div>
  <div class="sec">${aHtml(leer('docs/29-idiomas-e-inclusion.md'))}</div>
  <div class="sec">${aHtml(leer('docs/28-defensa-del-codigo.md'))}</div>
  <div class="sec">${aHtml(leer('docs/11-fuentes-y-citas.md'))}</div>
  <div class="sec">${aHtml(leer('docs/27-componentes-de-terceros.md'))}</div>
  <div class="sec">${aHtml(leer('LICENCIA.md'))}</div></body></html>`;
  const d = await b.newPage(); await d.setContent(html, { waitUntil: 'load' });
  const salida = path.join(dir, 'FARO-documento-tecnico.pdf');
  await d.pdf({ path: salida, format: 'Letter', printBackground: true, displayHeaderFooter: true, headerTemplate: '<span></span>', footerTemplate: '<div style="font-size:8px;color:#5b6672;width:100%;text-align:center">FARO · Datatón INE 2026 · <span class="pageNumber"></span> de <span class="totalPages"></span></div>', margin: { top: '18mm', bottom: '18mm', left: '16mm', right: '16mm' } });
  console.log('escrito', salida, Math.round(fs.statSync(salida).size / 1024), 'KB'); await b.close();
})();
