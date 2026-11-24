const { chromium } = require('playwright');
(async () => {
  const b = await chromium.launch(); const p = await b.newPage({ viewport: { width: 1440, height: 900 } });
  const errores = []; p.on('pageerror', e => errores.push(String(e).slice(0, 200)));
  await p.goto(process.argv[2]); await p.waitForSelector('#lienzo path.leaflet-interactive');
  await require('./_sesion').entrar(p);
  const secciones = await p.locator('nav button[data-s]').evaluateAll(bs => bs.filter(x => !x.hidden).map(x => x.dataset.s)); console.log('secciones visibles para admin:', secciones.length);
  for (const s of secciones) { await p.click(`nav button[data-s=${s}]`); await p.waitForTimeout(450); }
  await p.click('nav button[data-s=alimentaria]'); await p.waitForSelector('#ttalla tbody tr td b'); console.log('talla:', (await p.textContent('#ttalla tbody tr:first-child')).replace(/\s+/g, ' ').slice(0, 90), '| filas', await p.locator('#ttalla tbody tr').count());
  await p.click('nav button[data-s=mapa]'); await p.selectOption('#indicador', 'retardo_en_talla_pct'); await p.waitForTimeout(500); console.log('leyenda:', (await p.textContent('#leyenda')).replace(/\s+/g, ' '));
  await p.screenshot({ path: process.argv[3] + '/33-talla.png' });
  console.log('capas:', await p.evaluate(() => fetch('/capas').then(r => r.json()).then(j => j.capas.length)), '| errores js:', JSON.stringify(errores)); await b.close();
})();
