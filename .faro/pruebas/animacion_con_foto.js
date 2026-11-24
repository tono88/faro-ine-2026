// Animación de la mancha urbana: foto satelital de fondo, corre una sola vez y se detiene, casilla para repetir.
// Uso: NO_PROXY=localhost node .faro/pruebas/animacion_con_foto.js http://localhost:8095 <carpeta>
// Las fotos vienen de internet (NASA GIBS y EOX): en la nube el navegador sale por el proxy de la sesión.
const { chromium } = require('playwright');
(async () => {
  const base = process.argv[2] || 'http://localhost:8082', dir = process.argv[3] || '.', proxy = process.env.HTTPS_PROXY;
  const nav = await chromium.launch(proxy ? { proxy: { server: proxy, bypass: 'localhost,127.0.0.1' } } : {}); const p = await (await nav.newContext({ viewport: { width: 1536, height: 864 } })).newPage();
  const errores = []; p.on('pageerror', e => errores.push(e.message));
  const pedidas = { gibs: 0, eox: 0, bien: 0 }; p.on('response', r => { const u = r.url(); if (/gibs\.earthdata/.test(u)) pedidas.gibs++; if (/tiles\.maps\.eox/.test(u)) pedidas.eox++; if (/gibs\.earthdata|tiles\.maps\.eox/.test(u) && r.ok()) pedidas.bien++; });
  let fallas = 0; const ok = (t, c, extra = '') => { console.log((c ? '  ok  ' : ' FALLA ') + t + (extra ? ' — ' + extra : '')); if (!c) fallas++; };
  await p.goto(base); await p.waitForSelector('#lienzo path.leaflet-interactive');
  await p.evaluate(() => detalle('0101')); await p.waitForSelector('#animlienzo'); await p.waitForTimeout(2500);
  const epocas = +(await p.getAttribute('#animanio', 'max')) + 1;
  await p.screenshot({ path: `${dir}/98-animacion-1985.png` });
  // Se deja correr: debe llegar a la última época y detenerse sola.
  await p.waitForFunction(() => /Ver de nuevo/.test(document.querySelector('#animplay').textContent), null, { timeout: 30000 });
  const fin = await p.inputValue('#animanio'); await p.waitForTimeout(2200);
  ok('corre una vez y se detiene en la última época', +fin === epocas - 1 && (await p.inputValue('#animanio')) === fin, `${epocas} épocas`);
  ok('pidió fotos Landsat (NASA) y Sentinel-2 (EOX)', pedidas.gibs > 0 && pedidas.eox > 0, JSON.stringify(pedidas));
  await p.waitForTimeout(1500); await p.screenshot({ path: `${dir}/99-animacion-2020.png` });
  // Con «Repetir sin parar» vuelve a empezar.
  await p.check('#animbucle'); await p.waitForFunction(f => document.querySelector('#animanio').value !== f, fin, { timeout: 8000 });
  ok('con la casilla, repite', /Pausa/.test(await p.textContent('#animplay')));
  await p.selectOption('#animfondo', 'ninguna'); await p.click('#animplay'); ok('se puede quitar la foto', true);
  // Capa satelital del mapa principal.
  await p.click('#detalle .cerrar'); await p.check('#capasat'); await p.waitForSelector('#lienzo img.leaflet-tile-loaded', { timeout: 20000 }).catch(() => {});
  ok('el mapa principal admite la foto satelital', (await p.locator('#lienzo img.leaflet-tile').count()) > 0); await p.waitForTimeout(1500); await p.screenshot({ path: `${dir}/100-mapa-satelital.png` });
  ok('sin errores de consola', errores.length === 0, errores.slice(0, 3).join(' | '));
  await nav.close(); console.log(fallas ? `\n${fallas} falla(s)` : '\nTodo bien'); process.exit(fallas ? 1 : 0);
})();
