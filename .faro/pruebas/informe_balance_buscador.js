// Prueba: buscador de territorio en el mapa, calculadora de derivados (HBA) e informe de precio de referencia.
// Uso: NO_PROXY=localhost node informe_balance_buscador.js http://localhost:8099 carpeta
const { chromium } = require('playwright');
(async () => {
  const b = await chromium.launch(); const ctx = await b.newContext({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: 2 }); const p = await ctx.newPage();
  const errores = []; p.on('pageerror', e => errores.push(String(e).slice(0, 200))); const dir = process.argv[3];
  const t = async s => (await p.textContent(s)).replace(/\s+/g, ' ').trim();
  await p.goto(process.argv[2]); await p.waitForSelector('#lienzo path.leaflet-interactive');
  // Buscador: por alias, municipio; luego un lugar poblado.
  await p.fill('#buscalugar', 'puerto san jose'); await p.waitForSelector('#buscalista li[data-i]'); console.log('búsqueda:', (await t('#buscalista')).slice(0, 110));
  await p.keyboard.press('Enter'); await p.waitForSelector('#ficha [data-solo]'); console.log('→', await t('#migas'), '| ficha:', (await t('#ficha h2')));
  await p.screenshot({ path: `${dir}/60-buscador-municipio.png` });
  await p.fill('#buscalugar', 'chama conop'); await p.waitForSelector('#buscalista li[data-i]'); await p.click('#buscalista li[data-i]'); await p.waitForSelector('.leaflet-popup-content', { timeout: 15000 });
  console.log('lugar:', (await t('.leaflet-popup-content')).slice(0, 120), '| nivel', await p.evaluate(() => nivel));
  // Calculadora
  await p.click('nav button[data-s=balance]'); await p.click('#hbaejemplo'); await p.waitForSelector('#thba tbody tr td b');
  await p.selectOption('#hbaproducto', 'Fruta ficticia B'); await p.waitForTimeout(700); await p.fill('#hbacantidad', '2500'); await p.click('#hbacalcular'); await p.waitForTimeout(700);
  console.log('balance:', await t('#hbatarjetas'), '|', (await t('#thba tbody')).slice(0, 90), '|', await t('#hbaavisos')); await p.screenshot({ path: `${dir}/61-balance.png` });
  // Informe
  await p.click('nav button[data-s=rubros]'); await p.waitForSelector('#trubros tbody tr td a', { timeout: 20000 }); await p.selectOption('#rubrosel', 'tecnologia'); await p.waitForTimeout(1500);
  await p.fill('#rubroq', 'laptop 8gb 256gb'); await p.click('#rubrobuscar'); await p.waitForTimeout(1500);
  const [inf] = await Promise.all([ctx.waitForEvent('page'), p.click('#rubroinforme')]); await inf.waitForSelector('.kpi');
  console.log('informe:', (await inf.textContent('.kpi')).replace(/\s+/g, ' ').slice(0, 230)); await inf.screenshot({ path: `${dir}/62-informe.png`, fullPage: true });
  await inf.check('tbody tr:first-child input[data-u]'); await Promise.all([inf.waitForNavigation(), inf.click('#recalcular')]); await inf.waitForSelector('.kpi');
  console.log('tras apartar uno:', (await inf.textContent('.kpi')).replace(/\s+/g, ' ').slice(0, 120), '|', (await inf.textContent('.aviso')).includes('apartó'));
  console.log('errores js:', JSON.stringify(errores)); await b.close();
})();
