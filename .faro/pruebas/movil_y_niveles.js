// Recorre todas las secciones a 390 px (teléfono), mide desbordes horizontales y prueba los niveles del mapa.
// Uso: NO_PROXY=localhost node movil_y_niveles.js http://localhost:8099 carpeta_de_capturas
const { chromium } = require('playwright');
(async () => {
  const b = await chromium.launch(); const ctx = await b.newContext({ viewport: { width: 390, height: 844 }, deviceScaleFactor: 1, hasTouch: true, isMobile: true });
  const p = await ctx.newPage(); const errores = []; p.on('pageerror', e => errores.push(String(e).slice(0, 200)));
  const dir = process.argv[3];
  await p.goto(process.argv[2]); await p.waitForSelector('#lienzo path.leaflet-interactive');
  await require('./_sesion').entrar(p);
  const secciones = await p.locator('nav button[data-s]').evaluateAll(bs => bs.filter(x => !x.hidden).map(x => x.dataset.s));
  for (const s of secciones) {
    await p.click('#menu'); await p.click(`nav button[data-s=${s}]`); await p.waitForTimeout(900);
    const m = await p.evaluate(() => { const ancho = document.documentElement.clientWidth; const fuera = [...document.querySelectorAll('section.activa *')].filter(e => { const r = e.getBoundingClientRect(); if (!r.width || r.right <= ancho + 2) return false; let a = e.parentElement; while (a && a.tagName !== 'SECTION') { const o = getComputedStyle(a).overflowX; if (o === 'auto' || o === 'scroll' || o === 'hidden') return false; a = a.parentElement; } return true; });
      return { pagina: document.documentElement.scrollWidth, ancho, fuera: fuera.slice(0, 4).map(e => e.tagName + '#' + e.id + '.' + String(e.className).slice(0, 30) + ' ' + Math.round(e.getBoundingClientRect().right)) }; });
    console.log(s.padEnd(13), m.pagina > m.ancho ? 'DESBORDA ' + m.pagina : 'ok', m.fuera.length ? JSON.stringify(m.fuera) : '');
    await p.screenshot({ path: `${dir}/m-${s}.png` });
  }
  // Niveles del mapa
  await p.click('#menu'); await p.click('nav button[data-s=mapa]'); await p.waitForTimeout(500);
  await p.click('.niveles button[data-n=departamento]'); await p.waitForTimeout(1500);
  console.log('departamentos:', await p.locator('#lienzo path.leaflet-interactive').count(), '| leyenda:', (await p.textContent('#leyenda')).replace(/\s+/g, ' ').slice(0, 120));
  await p.evaluate(() => { let c; capaDep.eachLayer(x => { if (x.feature.properties.nombre === 'Petén') c = x; }); fichaDepartamento(c.feature.properties, c); }); await p.waitForTimeout(400);
  console.log('ficha dep:', (await p.textContent('#ficha')).replace(/\s+/g, ' ').slice(0, 330));
  await p.screenshot({ path: `${dir}/m-nivel-departamento.png` });
  await p.click('#ficha [data-bajar]'); await p.waitForTimeout(1200); console.log('migas:', (await p.textContent('#migas')).replace(/\s+/g, ' '), '| nivel', await p.evaluate(() => nivel));
  await p.screenshot({ path: `${dir}/m-nivel-municipio.png` });
  await p.click('.niveles button[data-n=lugar]'); await p.waitForTimeout(2500); console.log('lugares:', await p.evaluate(() => capaNivelLugar.getLayers().length), '| leyenda:', (await p.textContent('#leyenda')).replace(/\s+/g, ' '));
  await p.screenshot({ path: `${dir}/m-nivel-lugar.png` });
  console.log('errores js:', JSON.stringify(errores)); await b.close();
})();
