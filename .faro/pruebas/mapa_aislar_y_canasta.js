// Prueba: Ver en el mapa desde Red vial (enciende enlaces), vista aislada, botón de volver al país y gráfica ampliada de la canasta.
// Uso: NO_PROXY=localhost node mapa_aislar_y_canasta.js http://localhost:8099 carpeta
const { chromium } = require('playwright');
(async () => {
  const b = await chromium.launch(); const p = await b.newPage({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: 2 });
  const errores = []; p.on('pageerror', e => errores.push(String(e).slice(0, 200))); const dir = process.argv[3];
  const t = async s => (await p.textContent(s)).replace(/\s+/g, ' ').trim();
  await p.goto(process.argv[2]); await p.waitForSelector('#lienzo path.leaflet-interactive');
  // 1. Desde el nivel de departamentos, ir a Red vial y pedir "Ver en el mapa".
  await p.click('.niveles button[data-n=departamento]'); await p.waitForTimeout(1200);
  await p.click('nav button[data-s=vial]'); await p.waitForSelector('#tvialenl button[data-ver]'); await p.click('#tvialenl tbody tr:nth-child(3) button[data-ver]'); await p.waitForTimeout(3000);
  console.log('tras Ver en el mapa → nivel:', await p.evaluate(() => nivel), '| capa marcada:', await p.isChecked('#capaenlaces'), '| enlaces dibujados:', await p.locator('.leaflet-encima-pane path').count(), '| globo:', (await p.locator('.leaflet-popup-content').count()) ? (await t('.leaflet-popup-content')).slice(0, 80) : 'no');
  await p.screenshot({ path: `${dir}/50-ver-en-mapa.png` });
  // 2. Volver al país.
  await p.click('#volverpais'); await p.waitForTimeout(800); console.log('zoom tras volver:', await p.evaluate(() => mapa.getZoom()));
  // 3. Aislar un departamento y luego un municipio.
  await p.uncheck('#capaenlaces'); await p.click('.niveles button[data-n=departamento]'); await p.waitForTimeout(800);
  await p.evaluate(() => { let c; capaDep.eachLayer(x => { if (x.feature.properties.nombre === 'Sololá') c = x; }); fichaDepartamento(c.feature.properties, c); });
  await p.click('#ficha [data-bajar]'); await p.waitForTimeout(1500);
  console.log('aislado:', await t('#migas'), '| nivel', await p.evaluate(() => nivel), '| leyenda:', (await t('#leyenda')).slice(0, 90));
  await p.screenshot({ path: `${dir}/51-solo-departamento.png` });
  await p.evaluate(() => ficha(geoMuni.features.find(f => f.properties.nombre === 'Panajachel').properties)); await p.waitForSelector('#ficha [data-solo]'); await p.click('#ficha [data-solo]'); await p.waitForTimeout(1200);
  await p.click('#ficha .cerrar'); await p.click('.niveles button[data-n=lugar]'); await p.waitForTimeout(2500);
  console.log('solo municipio:', await t('#migas'), '| puntos visibles:', await p.evaluate(() => capaNivelLugar.getLayers().filter(c => c.options.fillOpacity > 0).length));
  await p.screenshot({ path: `${dir}/52-solo-municipio-lugares.png` });
  await p.click('#migas [data-todo]'); await p.waitForTimeout(600); console.log('tras ver todo:', JSON.stringify(await t('#migas')), '| puntos:', await p.evaluate(() => capaNivelLugar.getLayers().filter(c => c.options.fillOpacity > 0).length));
  // 4. Canasta: ampliar el histórico.
  await p.click('nav button[data-s=canasta]'); await p.waitForSelector('#tcanasta button.ampliar'); await p.click('#tcanasta tbody tr:first-child button.ampliar'); await p.waitForSelector('#detalle[open] svg');
  console.log('canasta:', (await t('#detcuerpo')).slice(0, 330)); await p.screenshot({ path: `${dir}/53-canasta-ampliada.png` });
  console.log('errores js:', JSON.stringify(errores)); await b.close();
})();
