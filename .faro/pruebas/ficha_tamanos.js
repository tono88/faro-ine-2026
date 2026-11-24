const { chromium } = require('playwright');
(async () => {
  const b = await chromium.launch();
  for (const [w, h] of [[1920, 1020], [1440, 900], [1280, 720]]) {
    const p = await b.newPage({ viewport: { width: w, height: h } }); await p.goto(process.argv[2]); await p.waitForSelector('#lienzo path.leaflet-interactive');
    await p.evaluate(() => ficha(geoMuni.features[100].properties)); await p.waitForSelector('#ficha .mini');
    const c = await p.evaluate(() => { const f = document.querySelector('#ficha'), r = f.getBoundingClientRect(), m = document.querySelector('#lienzo').getBoundingClientRect(); return {dentro: r.top >= m.top && r.bottom <= m.bottom + 1, ficha: Math.round(r.height), mapa: Math.round(m.height), conDesplazamiento: f.scrollHeight > f.clientHeight + 2, visibleEnPantalla: r.bottom <= innerHeight + scrollY + 400}; });
    console.log(w + 'x' + h, JSON.stringify(c)); if (w === 1920) await p.screenshot({ path: process.argv[3] + '/32-ficha-1920.png' }); await p.close();
  }
  await b.close();
})();
