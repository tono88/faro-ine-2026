// Prueba: menú de rubros, pestaña "Diferencias de 50 % o más" con una decisión, y botón "Ejecutar todas" (sin confirmar).
// Uso: NO_PROXY=localhost node diferencias_y_menu.js http://localhost:8095 carpeta
const { chromium } = require('playwright');
(async () => {
  const b = await chromium.launch(); const p = await b.newPage({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: 2 });
  const errores = []; p.on('pageerror', e => errores.push(String(e).slice(0, 200))); const dir = process.argv[3];
  const t = async s => (await p.textContent(s)).replace(/\s+/g, ' ').trim();
  await p.goto(process.argv[2]); await p.waitForSelector('#lienzo path.leaflet-interactive');
  await require('./_sesion').entrar(p);
  await p.waitForSelector('#menurubros button[data-rubro]'); console.log('menú:', await t('#menurubros'));
  await p.click('#menurubros button[data-rubro=tecnologia]'); await p.waitForResponse(r => r.url().includes('rubro=tecnologia')); await p.waitForTimeout(1200); console.log('rubro elegido:', await p.inputValue('#rubrosel'), '|', (await t('#rubroresumen')).slice(0, 60));
  await p.click('#menurubros button[data-rubro=supermercado]'); await p.waitForResponse(r => r.url().includes('rubro=supermercado')); await p.waitForTimeout(1200);
  await p.click('#rubrovistas button[data-v=diferencias]'); await p.waitForSelector('#trubros button[data-dec]');
  const res = await t('#rubroresumen'); console.log('diferencias:', res.slice(res.indexOf('producto(s) con 50') - 6)); console.log('1.ª fila:', (await t('#trubros tbody tr:first-child')).slice(0, 330));
  await p.screenshot({ path: `${dir}/80-diferencias.png` });
  // Marcar como error la tienda atípica de la primera fila y comprobar que queda decidido.
  const fila = p.locator('#trubros tbody tr').first(); const tienda = await fila.locator('.etq').first().textContent();
  const ean = await fila.locator('code').textContent(); const senalada = await p.evaluate(e => rubroDatos.diferencias.find(d => d.ean === e).tienda, ean);
  await fila.locator(`button[data-dec=error][data-prov="${senalada}"]`).click(); await p.waitForResponse(r => r.url().includes('/api/publicados?')); await p.waitForTimeout(800);
  // Lo decidido baja al final de la lista (se muestran 300): se comprueba con los conteos y la comparación.
  const d = await p.evaluate(([e, prov]) => [rubroDatos.conteo_diferencias, rubroDatos.mismo_codigo.some(m => m.ean === e && m.ofertas.some(o => o.proveedor === prov))], [ean, senalada]);
  console.log('tras marcar error en', senalada, '(' + tienda + '):', JSON.stringify(d), '← [conteo, ¿la tienda sigue en la comparación por código?]');
  await p.evaluate(async ([e, prov]) => { await api('/api/publicados/decision', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ rubro: 'supermercado', ean: e, proveedor: prov, decision: '' }) }); }, [ean, senalada]);
  // Ejecutar todas: se cancela la confirmación.
  await p.click('nav button[data-s=muestreos]'); await p.waitForSelector('#tmuestreos tbody tr'); p.once('dialog', dlg => { console.log('confirmación:', dlg.message().slice(0, 70)); dlg.dismiss(); }); await p.click('#muestreostodas'); await p.waitForTimeout(600);
  console.log('en curso tras cancelar:', await p.evaluate(() => fetch('/api/muestreos').then(r => r.json()).then(j => j.filter(x => x.en_curso).length)), '| errores js:', JSON.stringify(errores)); await b.close();
})();
