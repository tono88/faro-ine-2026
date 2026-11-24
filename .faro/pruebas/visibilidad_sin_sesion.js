// Sin sesión no se ven las secciones de recepción ni los paneles para cargar archivos; con sesión de
// administrador, sí. Uso: NO_PROXY=localhost node .faro/pruebas/visibilidad_sin_sesion.js http://localhost:8082
const { chromium } = require('playwright'); const { entrar } = require('./_sesion.js');
(async () => {
  const base = process.argv[2] || 'http://localhost:8082'; const nav = await chromium.launch(); const p = await (await nav.newContext({ viewport: { width: 1440, height: 900 } })).newPage();
  const errores = []; p.on('pageerror', e => errores.push(e.message)); let fallas = 0; const ok = (t, c, x = '') => { console.log((c ? '  ok  ' : ' FALLA ') + t + (x ? ' — ' + x : '')); if (!c) fallas++; };
  const visibles = () => p.evaluate(() => ({ menu: [...document.querySelectorAll('nav > button[data-s]')].filter(b => getComputedStyle(b).display !== 'none').map(b => b.dataset.s),
    archivos: [...document.querySelectorAll('input[type=file]')].filter(i => { if (i.closest('dialog')) return false; const sec = i.closest('section'), boton = document.querySelector(`nav > button[data-s="${sec.id}"]`); if (boton.hidden) return false; let e = i; while (e) { if (e.hidden) return false; e = e.parentElement; } return true; }).map(i => i.id),
    acciones: [...document.querySelectorAll('[data-requiere]')].filter(x => !x.hidden).length }));
  await p.goto(base); await p.waitForSelector('#lienzo path.leaflet-interactive'); await p.waitForTimeout(800);
  let v = await visibles();
  ok('sin sesión: el menú no trae recepción, pendientes, equivalencias, operadores, ruta ni cuenta', !['recibir', 'pendientes', 'memoria', 'operadores', 'campo', 'perfil', 'usuarios'].some(s => v.menu.includes(s)) && v.menu.includes('mapa') && v.menu.includes('tableros'), v.menu.join(','));
  ok('sin sesión: ningún control para subir archivos ni acción de cambio a la vista', v.archivos.length === 0 && v.acciones === 0, JSON.stringify(v.archivos));
  ok('sin sesión: cargar una tabla propia se rechaza', (await p.evaluate(async () => (await fetch('/api/balance', { method: 'POST', headers: { 'content-type': 'text/csv' }, body: 'primario,derivado,tasa_extraccion_pct\nA,B,50\n' })).status)) === 401);
  await p.click('nav > button[data-s=muestreos]'); await p.waitForSelector('#tmuestreos tbody tr'); ok('sin sesión: los muestreos se ven, sin botones de ejecutar', (await p.locator('#tmuestreos button[data-e]').count()) === 0 && !(await p.isVisible('#muestreostodas')));
  await p.click('nav > button[data-s=precios]'); await p.waitForTimeout(400); ok('sin sesión: queda una nota donde iría la carga', await p.locator('#precios .nota-acceso:not([hidden])').count() >= 1);
  await p.evaluate(() => document.querySelector('nav button[data-s=recibir]').click()); await p.waitForTimeout(300);
  ok('sin sesión: una sección oculta no se abre aunque se la llame; se ofrece entrar', (await p.evaluate(() => document.querySelector('section.activa').id)) !== 'recibir' && await p.evaluate(() => document.querySelector('#ingreso').open));
  await p.evaluate(() => document.querySelector('#ingreso').close());
  await entrar(p); v = await visibles();
  ok('administrador: ve todas las secciones', ['recibir', 'pendientes', 'memoria', 'operadores', 'campo', 'usuarios'].every(s => v.menu.includes(s)), v.menu.length + ' secciones');
  ok('administrador: ve los controles de carga', ['archivo', 'csvtabla', 'csvedif', 'csvops', 'hbatabla', 'csvprecios'].every(i => v.archivos.includes(i)), v.archivos.join(','));
  ok('sin errores de consola', errores.length === 0, errores.slice(0, 2).join(' | '));
  await nav.close(); console.log(fallas ? `\n${fallas} falla(s)` : '\nTodo bien'); process.exit(fallas ? 1 : 0);
})();
