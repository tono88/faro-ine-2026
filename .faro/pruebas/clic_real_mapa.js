// Clic REAL del ratón sobre el mapa en los tres niveles y varios tamaños de ventana: debe abrir la ficha o el globo.
// (Las demás pruebas abren la ficha por código y no detectan fallos del clic.)
// Uso: NO_PROXY=localhost node clic_real_mapa.js http://localhost:8082
const { chromium } = require('playwright');
(async () => {
  const b = await chromium.launch(); let fallos = 0;
  for (const [w, h] of [[1440, 900], [1920, 1020], [1280, 650], [1024, 600]]) {
    const p = await b.newPage({ viewport: { width: w, height: h } }); const er = []; p.on('pageerror', e => er.push(String(e).slice(0, 150)));
    await p.goto(process.argv[2]); await p.waitForSelector('#lienzo path.leaflet-interactive'); await p.waitForTimeout(700);
    const res = [];
    for (const n of ['municipio', 'departamento']) {
      await p.click(`.niveles button[data-n=${n}]`); await p.waitForTimeout(1300);
      // Punto del lienzo que sí está a la vista y cae sobre un polígono.
      const pt = await p.evaluate(() => { const r = document.querySelector('#lienzo').getBoundingClientRect(); for (const fy of [.5, .6, .4, .7]) for (const fx of [.5, .55, .45, .6]) { const x = r.left + r.width * fx, y = r.top + r.height * fy; if (y < innerHeight - 5 && document.elementFromPoint(x, y)?.tagName === 'path') return [x, y]; } return null; });
      if (!pt) { res.push(n + ': sin polígono a la vista'); fallos++; continue; }
      await p.mouse.move(pt[0], pt[1]); await p.waitForTimeout(300); await p.mouse.down(); await p.waitForTimeout(80); await p.mouse.up(); await p.waitForTimeout(1500);
      const ok = await p.isVisible('#ficha h2'); res.push(`${n}: ${ok ? 'ficha de ' + (await p.textContent('#ficha h2')) : 'NO ABRE'}`); if (!ok) fallos++;
      await p.evaluate(() => { document.querySelector('#ficha').hidden = true; });
    }
    await p.click('.niveles button[data-n=lugar]'); await p.waitForTimeout(2500); await p.evaluate(() => mapa.setView([14.74, -91.16], 13)); await p.waitForTimeout(1200);
    const pl = await p.evaluate(() => { const r = document.querySelector('#lienzo').getBoundingClientRect(); let mejor = null; capaNivelLugar.eachLayer(c => { const q = mapa.latLngToContainerPoint(c.getLatLng()); if (!mejor && q.x > 60 && q.y > 60 && q.x < r.width - 60 && q.y < Math.min(r.height, innerHeight - r.top) - 60) mejor = [r.left + q.x, r.top + q.y]; }); return mejor; });
    if (pl) { await p.mouse.move(pl[0], pl[1]); await p.waitForTimeout(200); await p.mouse.down(); await p.waitForTimeout(80); await p.mouse.up(); await p.waitForTimeout(900); const ok = await p.isVisible('.leaflet-popup-content'); res.push('lugar: ' + (ok ? 'globo de ' + (await p.textContent('.leaflet-popup-content b')) : 'NO ABRE')); if (!ok) fallos++; } else res.push('lugar: sin punto a la vista');
    console.log(`${w}x${h}`, res.join(' | '), er.length ? JSON.stringify(er) : ''); await p.close();
  }
  console.log(fallos ? `FALLOS: ${fallos}` : 'todo abre con clic real'); await b.close(); process.exit(fallos ? 1 : 0);
})();
