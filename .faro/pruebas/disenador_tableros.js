// Diseñador de tableros: de la frase al lienzo, filtros que se aplican al tablero ya creado,
// deshacer, pedido con palabras sobre un tablero abierto, borrador → publicado y vista sin sesión.
// Uso: NO_PROXY=localhost node .faro/pruebas/disenador_tableros.js http://localhost:8095 <carpeta de capturas>
const { chromium } = require('playwright');
const { entrar } = require('./_sesion.js');
(async () => {
  const base = process.argv[2] || 'http://localhost:8082', dir = process.argv[3] || '.';
  const nav = await chromium.launch(); const ctx = await nav.newContext({ viewport: { width: 1536, height: 864 } }); const p = await ctx.newPage();
  const errores = []; p.on('pageerror', e => errores.push(e.message)); p.on('console', m => { if (m.type() === 'error') errores.push(m.text()); });
  let fallas = 0; const ok = (t, c, extra = '') => { console.log((c ? '  ok  ' : ' FALLA ') + t + (extra ? ' — ' + extra : '')); if (!c) fallas++; };
  await p.goto(base); await p.waitForSelector('#lienzo path.leaflet-interactive'); console.log('sesión:', await entrar(p));
  await p.click('nav > button[data-s=tableros]'); await p.fill('#tabtexto', 'Mapa y tabla de desnutrición y población en Sololá, los 5 municipios más altos'); await p.click('#tabarmar'); await p.waitForSelector('#tabvista .pieza svg path');
  const piezas = () => p.locator('#tabvista .pieza[data-i]').count();
  ok('la frase arma el tablero y abre el diseñador', (await piezas()) === 4 && (await p.inputValue('#tfdep')) === '07' && await p.isVisible('#tabprops') && await p.isVisible('#tabviz'));
  const filasTabla = async () => p.locator('#tabvista .pieza table').first().locator('tbody tr td:nth-child(2)').allTextContents();
  const antes = await filasTabla();
  // Filtro de territorio sobre el tablero ya creado: otro departamento cambia todas las piezas y el título.
  await p.selectOption('#tfdep', '14'); await p.waitForFunction(() => /Quich/.test(document.querySelector('#tabvista > p').textContent));
  const despues = await filasTabla();
  ok('cambiar el departamento cambia las piezas', antes.join() !== despues.join() && despues.length === 5, despues.join(', '));
  ok('el título sigue al territorio', /Quiché$/.test(await p.inputValue('#tabtitulo')), await p.inputValue('#tabtitulo'));
  // Condición sobre un indicador.
  await p.click('#tfmas'); await p.selectOption('#tabfiltros .cond select[data-k=indicador]', 'poblacion_actual'); await p.fill('#tabfiltros .cond input', '60000'); await p.locator('#tabfiltros .cond input').dispatchEvent('change');
  await p.waitForFunction(() => /solo municipios con/.test(document.querySelector('#tabvista > p').textContent));
  const resumen = await p.textContent('#tabvista > p'); const conFiltro = await filasTabla();
  ok('la condición deja solo los municipios que la cumplen', /\((\d+) de 21\)/.test(resumen) && conFiltro.length <= 5, resumen.slice(0, 150));
  ok('el mapa marca en gris los que quedan fuera', /en gris/.test(await p.locator('#tabvista .pieza svg + small').first().textContent()));
  // Deshacer devuelve el filtro; rehacer lo repone.
  for (let i = 0; i < 3; i++) await p.click('#tabdeshacer'); await p.waitForTimeout(250); const sinCond = await p.locator('#tabfiltros .cond').count();
  await p.click('#tabrehacer'); await p.waitForTimeout(150); const conCond = await p.locator('#tabfiltros .cond').count(); await p.click('#tabdeshacer'); await p.waitForTimeout(150);
  ok('deshacer quita la condición y rehacer la repone', sinCond === 0 && conCond === 1);
  // Paleta: agregar una dispersión y cambiarle el indicador desde «Datos».
  await p.click('#tabviz button[data-tipo=dispersion]'); await p.waitForSelector('#tabvista .pieza.sel svg circle');
  await p.fill('#tabbuscar', 'internet'); await p.click('#tabdatos button[data-ind=censo_internet_pct]'); await p.waitForTimeout(250);
  ok('la dispersión informa la correlación', /Pearson r = -?\d\.\d\d/.test(await p.textContent('#tabvista .pieza.sel')), (await p.textContent('#tabvista .pieza.sel small')).slice(0, 110));
  // Ancho y orden de las piezas.
  await p.click('#tpancho button[data-v="4"]'); ok('la pieza cambia de ancho', await p.locator('#tabvista .pieza.sel.a4').count() === 1);
  await p.click('#tabvista .pieza.sel .pz-herr button[data-a=izq]'); ok('la pieza se mueve', (await p.locator('#tabvista .pieza.sel').getAttribute('data-i')) === '3');
  // Pedido con palabras sobre el tablero abierto.
  await p.fill('#tabpedido', 'ahora solo Totonicapán y agrega una tabla de internet'); await p.click('#tabaplicar'); await p.waitForFunction(() => /Totonicap/.test(document.querySelector('#tabvista > p').textContent));
  ok('el pedido con palabras se aplica al tablero abierto', (await p.inputValue('#tfdep')) === '08' && (await piezas()) === 6, (await p.textContent('#tabentendido')).replace(/\s+/g, ' ').slice(0, 200));
  await p.screenshot({ path: `${dir}/95-disenador.png` });
  // Guardar deja un borrador; publicar lo pone a la vista.
  await p.click('#tabguardar'); await p.waitForFunction(() => /guardado n\.º \d+/.test(document.querySelector('#tabestado').textContent)); const id = (await p.textContent('#tabestado')).match(/n\.º (\d+)/)[1];
  ok('se guarda como borrador', /^Borrador/.test(await p.textContent('#tabestado')) && p.url().endsWith('#tablero=' + id));
  const anon = await (await nav.newContext({ viewport: { width: 1536, height: 864 } })).newPage(); await anon.goto(base); await anon.waitForSelector('#lienzo path.leaflet-interactive');
  const cod = r => anon.evaluate(async u => (await fetch(u)).status, r);
  ok('un borrador no se ve sin sesión', (await cod('/api/tableros/' + id)) === 400 && !(await anon.evaluate(async () => (await (await fetch('/api/tableros')).json()).tableros.some(t => !t.publico))));
  await p.click('#tabpublicar'); await p.waitForFunction(() => /^Publicado/.test(document.querySelector('#tabestado').textContent));
  ok('publicado: se lee sin sesión', (await cod('/api/tableros/' + id)) === 200);
  // Vista pública por enlace: sin paneles de edición y con filtros que no cambian lo publicado.
  await anon.goto(base + '/#tablero=' + id); await anon.reload(); await anon.waitForSelector('#tabvista .pieza[data-i]');
  ok('el enlace abre la vista publicada', await anon.isVisible('#tabfiltros') && !(await anon.isVisible('#tabprops')) && !(await anon.isVisible('#tabguardar')) && (await anon.locator('#tabvista .pieza[data-i]').count()) === 6);
  await anon.selectOption('#tfdep', '07'); await anon.waitForFunction(() => /Sololá/.test(document.querySelector('#tabvista > p').textContent));
  const guardado = await anon.evaluate(async i => (await (await fetch('/api/tableros/' + i)).json()).definicion.territorio.codigo, id);
  ok('el visitante filtra sin cambiar lo publicado', guardado === '08', 'guardado: ' + guardado);
  await anon.screenshot({ path: `${dir}/96-tablero-publicado.png` });
  ok('guardar sin sesión se rechaza', (await anon.evaluate(async () => (await fetch('/api/tableros', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ definicion: { titulo: 'x', territorio: { nivel: 'pais' }, piezas: [{ tipo: 'tarjeta', indicador: 'poblacion_actual' }] } }) })).status)) === 401);
  // Galería y limpieza.
  await p.click('#tabvolver'); await p.waitForSelector(`#tabpublicados button[data-ver="${id}"]`); ok('aparece entre los publicados', true);
  await p.screenshot({ path: `${dir}/97-galeria.png` });
  p.once('dialog', d => d.accept()); await p.click(`#tabpublicados button[data-borrar="${id}"]`); await p.waitForFunction(i => !document.querySelector(`#tabpublicados button[data-ver="${i}"]`), id);
  ok('sin errores de consola', errores.length === 0, errores.slice(0, 3).join(' | '));
  await nav.close(); console.log(fallas ? `\n${fallas} falla(s)` : '\nTodo bien'); process.exit(fallas ? 1 : 0);
})();
