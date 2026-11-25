// Prueba la sección "Precios por rubro" (búsqueda, pestañas, tiendas) y el globo del enlace probable.
// Uso: NO_PROXY=localhost node precios_por_rubro.js http://localhost:8099 carpeta_de_capturas
const { chromium } = require('playwright'); const { entrar } = require('./_sesion.js');
(async () => {
  const b = await chromium.launch(); const p = await b.newPage({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: 2 });
  const errores = []; p.on('pageerror', e => errores.push(String(e).slice(0, 200))); const dir = process.argv[3];
  const t = async s => (await p.textContent(s)).replace(/\s+/g, ' ').trim();
  await p.goto(process.argv[2]); await p.waitForSelector('#lienzo path.leaflet-interactive'); await entrar(p); // «Muestreos automáticos» solo lo ve el personal del INE
  await p.click('nav button[data-s=rubros]'); await p.waitForSelector('#trubros tbody tr td a', { timeout: 20000 });
  console.log('tarjetas:', await t('#rubrotarjetas')); console.log('resumen:', await t('#rubroresumen'));
  await p.click('#rubrovistas button[data-v=iguales]'); console.log('iguales:', (await t('#trubros tbody tr:first-child')).slice(0, 200));
  await p.screenshot({ path: `${dir}/40-rubros-mismo-codigo.png` });
  await p.selectOption('#rubrosel', 'tecnologia'); await p.waitForTimeout(1500); await p.fill('#rubroq', 'televisor 55'); await p.click('#rubrobuscar'); await p.waitForTimeout(1500);
  await p.click('#rubrovistas button[data-v=filas]'); console.log('tele:', await t('#rubroresumen')); console.log(' fila:', (await t('#trubros tbody tr:nth-child(2)')).slice(0, 160));
  await p.fill('#rubroq', ''); await p.click('#rubrobuscar'); await p.waitForTimeout(1500);
  await p.click('#rubrovistas button[data-v=cambios]'); console.log('cambios:', (await t('#trubros tbody tr:first-child')).slice(0, 200), '| filas', await p.locator('#trubros tbody tr').count());
  await p.screenshot({ path: `${dir}/41-rubros-cambios.png` });
  await p.click('#rubrovistas button[data-v=informantes]'); console.log('informantes:', (await t('#trubros tbody')).slice(0, 260));
  await p.click('nav button[data-s=muestreos]'); await p.waitForSelector('#tmuestreos tbody tr'); console.log('tareas:', await p.locator('#tmuestreos tbody tr').count(), '|', (await t('#tmuestreos tbody tr:nth-child(4)')).slice(0, 230));
  await p.screenshot({ path: `${dir}/42-muestreos.png` });
  // Globo del enlace probable
  await p.click('nav button[data-s=mapa]'); await p.check('#capaenlaces'); await p.waitForTimeout(2500);
  const punto = await p.evaluate(() => { let c; capaEnlaces.eachLayer(g => g.eachLayer(x => { if (!c && x.getTooltip && x.getTooltip()) c = x; })); mapa.fitBounds(c.getBounds(), { maxZoom: 14 }); return true; });
  await p.waitForTimeout(800); await p.evaluate(() => { let c; capaEnlaces.eachLayer(g => g.eachLayer(x => { if (!c && x.getTooltip && x.getTooltip()) c = x; })); c.openTooltip(c.getBounds().getCenter()); });
  await p.waitForTimeout(400); console.log('globo:', (await t('.tip-enlace')).slice(0, 200)); await p.screenshot({ path: `${dir}/43-enlace-globo.png` });
  await p.click('.niveles button[data-n=lugar]'); await p.waitForTimeout(2500); await p.screenshot({ path: `${dir}/44-nivel-lugar.png` });
  await p.uncheck('#capaenlaces'); await p.click('.niveles button[data-n=departamento]'); await p.selectOption('#indicador', 'retardo_en_talla_pct'); await p.waitForTimeout(1200); await p.screenshot({ path: `${dir}/45-nivel-departamento.png` });
  console.log('leyenda dep:', await t('#leyenda'), '| errores js:', JSON.stringify(errores)); await b.close();
})();
