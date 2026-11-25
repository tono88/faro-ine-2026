// Carga a la vista: «Precios por rubro» muestra algo de inmediato y llega por partes; la animación trae cada
// foto satelital entera (una imagen por foto, no decenas de teselas) antes de empezar.
// Uso: NO_PROXY=localhost node .faro/pruebas/carga_precios_y_fotos.js <url> [carpeta de capturas]
// Las fotos vienen de internet (NASA GIBS y EOX): en la nube el navegador sale por el proxy de la sesión.
const { chromium } = require('playwright'); const path = require('path');
const url = process.argv[2] || 'http://localhost:8082', dir = process.argv[3];
let fallos = 0; const ok = (c, t) => { console.log((c ? '  bien  ' : '  FALLA ') + t); if (!c) fallos++; };
(async () => {
  const proxy = process.env.HTTPS_PROXY; const nav = await chromium.launch(proxy ? { proxy: { server: proxy, bypass: 'localhost,127.0.0.1' } } : {});
  const p = await nav.newPage({ viewport: { width: 1366, height: 850 } }); const errores = []; p.on('pageerror', e => errores.push(e.message));
  // El servidor se hace lento a propósito: los precios tardan 1.2 s y cada vista pesada 1.5 s más.
  await p.route('**/api/publicados?parte=*', async r => { const parte = new URL(r.request().url()).searchParams.get('parte'); await new Promise(s => setTimeout(s, parte === 'precios' ? 1200 : parte === 'rubros' ? 0 : 1500)); r.continue(); });
  await p.goto(url); await p.waitForSelector('#lienzo path.leaflet-interactive');
  console.log('Precios por rubro');
  const t0 = Date.now(); await p.click('nav > button[data-s=rubros]');
  await p.waitForSelector('#rubroresumen .cargando', { timeout: 600 }); ok(Date.now() - t0 < 700, `el aviso «Buscando precios…» aparece al instante (${Date.now() - t0} ms)`);
  ok(await p.$$eval('#trubros .relleno tr', f => f.length) >= 4 && await p.isVisible('#trubros .cargando-barra'), 'la tabla muestra filas de relleno y una barra que avanza');
  if (dir) await p.screenshot({ path: path.join(dir, 'precios_cargando.png') });
  await p.waitForSelector('#trubros tbody tr td a', { timeout: 8000 }); const t1 = Date.now() - t0;
  const a_medias = await p.evaluate(() => ({ giran: document.querySelectorAll('#rubrovistas .gira').length, calc: /calculando/.test(document.querySelector('#rubroresumen').textContent), filas: document.querySelectorAll('#trubros tbody tr').length }));
  ok(a_medias.filas > 10 && a_medias.giran >= 3 && a_medias.calc, `los precios se ven (${a_medias.filas} filas a los ${t1} ms) mientras las otras vistas siguen cargando (${a_medias.giran} pestañas con indicador)`);
  if (dir) await p.screenshot({ path: path.join(dir, 'precios_parcial.png') });
  await p.click('#rubrovistas button[data-v=informantes]'); ok(await p.isVisible('#trubros .cargando'), 'una vista que aún no llega dice que se está calculando');
  await p.waitForFunction(() => !document.querySelector('#rubrovistas .gira') && !/calculando/.test(document.querySelector('#rubroresumen').textContent), null, { timeout: 20000 });
  ok(await p.$$eval('#trubros tbody tr', f => f.length) >= 1 && !(await p.$('#trubros .relleno')), `al llegar, la vista abierta se pinta sola (${Date.now() - t0} ms en total)`);
  for (const v of ['iguales', 'diferencias', 'cambios', 'filas']) { await p.click(`#rubrovistas button[data-v=${v}]`); await p.waitForTimeout(150); if (await p.$('#trubros .relleno')) ok(false, 'la vista ' + v + ' quedó en carga'); }
  ok(true, 'las cinco vistas abren con sus datos');
  await p.unroute('**/api/publicados?parte=*');
  const directo = await p.evaluate(async () => { const t = performance.now(); const r = await fetch('/api/publicados?parte=precios').then(x => x.json()); return [Math.round(performance.now() - t), r.filas.length, r.total]; });
  ok(directo[1] > 0, `sin retraso artificial, los precios responden en ${directo[0]} ms (${directo[2].toLocaleString()} productos del rubro)`);

  console.log('Animación con foto satelital');
  const pedidas = { enteras: [], teselas: 0 }; p.on('request', r => { const u = r.url(); if (/wms/i.test(u) && /GetMap/i.test(u)) pedidas.enteras.push(u); else if (/wmts/.test(u)) pedidas.teselas++; });
  await p.click('nav > button[data-s=mapa]'); await p.evaluate(() => detalle('0101')); await p.waitForSelector('#animlienzo');
  await p.waitForTimeout(250); if (dir) await p.screenshot({ path: path.join(dir, 'animacion_cargando.png') });
  const inicio = Date.now(); await p.waitForFunction(() => +document.querySelector('#animanio').value >= 1, null, { timeout: 30000 });
  ok(true, `la animación arranca cuando las fotos están listas (${Date.now() - inicio} ms de espera)`);
  await p.waitForFunction(() => /Ver de nuevo/.test(document.querySelector('#animplay').textContent), null, { timeout: 40000 }); await p.waitForTimeout(500);
  ok(pedidas.enteras.length >= 1 && pedidas.enteras.length <= 5 && pedidas.teselas === 0, `se pidieron ${pedidas.enteras.length} imágenes enteras y ${pedidas.teselas} teselas (antes: decenas de teselas por foto)`);
  if (dir) await p.screenshot({ path: path.join(dir, 'animacion_final.png') });
  ok(errores.length === 0, 'sin errores de JavaScript' + (errores.length ? ': ' + errores.slice(0, 3).join(' | ') : ''));
  await nav.close(); console.log(fallos ? `\n${fallos} FALLA(S)` : '\nTodo bien.'); process.exit(fallos ? 1 : 0);
})();
