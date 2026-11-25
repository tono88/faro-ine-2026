// Mapa: las casillas de capas que el navegador deja marcadas al recargar se aplican; con capas de lienzo
// encendidas y después de pasar por «Lugares poblados», el clic REAL sigue abriendo la ficha.
// Uso: NO_PROXY=localhost node .faro/pruebas/mapa_capas_y_niveles.js http://localhost:8082
const { chromium } = require('playwright');
(async () => {
  const base = process.argv[2] || 'http://localhost:8082'; const nav = await chromium.launch(); const p = await (await nav.newContext({ viewport: { width: 1536, height: 864 } })).newPage();
  const errores = []; p.on('pageerror', e => errores.push(e.message)); let fallas = 0; const ok = (t, c, x = '') => { console.log((c ? '  ok  ' : ' FALLA ') + t + (x ? ' — ' + x : '')); if (!c) fallas++; };
  // Simula lo que hace el navegador al recargar: la casilla llega marcada antes de que el mapa esté listo.
  await p.addInitScript(() => document.addEventListener('DOMContentLoaded', () => { document.querySelector('#capavias').checked = true; document.querySelector('#capaenlaces').checked = true; }));
  await p.goto(base); await p.waitForSelector('#lienzo path.leaflet-interactive'); await p.waitForSelector('#leyendavias:not([hidden])', { timeout: 20000 }); await p.waitForTimeout(1200);
  ok('las casillas que llegan marcadas se aplican', await p.evaluate(() => !!capaVias && mapa.hasLayer(capaVias) && !!capaEnlaces && mapa.hasLayer(capaEnlaces)));
  const clicReal = async () => { await p.evaluate(() => document.querySelector('#ficha').hidden = true);
    const pt = await p.evaluate(() => { const r = document.querySelector('#lienzo').getBoundingClientRect(); for (const fy of [.5, .6, .4, .7, .3]) for (const fx of [.5, .55, .45, .6, .4]) { const x = r.left + r.width * fx, y = r.top + r.height * fy, e = document.elementFromPoint(x, y); if (y > 0 && y < innerHeight - 5 && e && e.tagName === 'path') return [x, y]; } return null; });
    if (!pt) return 'ningún polígono recibe el puntero'; await p.mouse.move(pt[0], pt[1]); await p.waitForTimeout(150); await p.mouse.down(); await p.waitForTimeout(60); await p.mouse.up(); await p.waitForTimeout(1300); return (await p.isVisible('#ficha h2')) ? 'abre la ficha' : 'NO abre la ficha'; };
  await p.locator('#lienzo').scrollIntoViewIfNeeded(); let r = await clicReal(); ok('con las carreteras encendidas, el clic abre la ficha del municipio', r === 'abre la ficha', r);
  await p.mouse.move(700, 500); await p.waitForTimeout(300); ok('la capa de carreteras no se queda con el puntero', await p.evaluate(() => getComputedStyle(document.querySelector('.leaflet-encima-pane canvas')).pointerEvents === 'none'));
  // Lugares poblados → Departamentos: antes el lienzo de los puntos quedaba encima y nada respondía.
  await p.click('.niveles button[data-n=lugar]'); await p.waitForFunction(() => !!capaNivelLugar && mapa.hasLayer(capaNivelLugar), null, { timeout: 30000 }); await p.waitForTimeout(800);
  await p.click('.niveles button[data-n=departamento]'); await p.waitForTimeout(1200);
  ok('al salir de «Lugares poblados» no queda un lienzo vacío encima', await p.evaluate(() => document.querySelectorAll('.leaflet-puntos-pane canvas').length === 0));
  r = await clicReal(); ok('después de «Lugares poblados», el clic abre la ficha del departamento', r === 'abre la ficha', r);
  await p.click('.niveles button[data-n=municipio]'); await p.waitForTimeout(900); r = await clicReal(); ok('y la del municipio', r === 'abre la ficha', r);
  await p.click('.niveles button[data-n=lugar]'); await p.waitForTimeout(1500); ok('al volver a «Lugares poblados» los puntos se dibujan de nuevo', await p.evaluate(() => document.querySelectorAll('.leaflet-puntos-pane canvas').length === 1));
  ok('sin errores de consola', errores.length === 0, errores.slice(0, 2).join(' | '));
  await nav.close(); console.log(fallas ? `\n${fallas} falla(s)` : '\nTodo bien'); process.exit(fallas ? 1 : 0);
})();
