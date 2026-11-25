// Cuentas de institución (solo suben y ven lo de su institución) y recorridos guiados de todas las secciones.
// Uso: node institucion_y_recorridos.js <url> [carpeta de capturas]
const { chromium } = require('playwright');
const { entrar, CLAVE } = require('./_sesion.js');
const path = require('path');
const url = process.argv[2] || 'http://localhost:8082', dir = process.argv[3];
let fallos = 0;
const ok = (c, t) => { console.log((c ? '  bien  ' : '  FALLA ') + t); if (!c) fallos++; };
(async () => {
  const nav = await chromium.launch();
  const p = await nav.newPage({ viewport: { width: 1366, height: 850 } });
  const errores = []; p.on('pageerror', e => errores.push(e.message));
  await p.goto(url); await p.waitForSelector('#lienzo path.leaflet-interactive');

  console.log('Recorridos sin sesión');
  ok(await p.isVisible('#recorrido'), 'el botón «Recorrido guiado» está en el encabezado');
  await p.click('#recorrido'); await p.waitForSelector('#recglobo');
  ok(/paso 1 de \d/.test(await p.textContent('#recglobo .cuenta')), 'abre el recorrido de la sección a la vista (mapa)');
  const foco = await p.evaluate(() => { const f = document.querySelector('#recfoco').getBoundingClientRect(); return f.width > 50 && f.height > 20; });
  ok(foco, 'el foco rodea un elemento');
  let n = 0; while (await p.$('#recglobo') && n++ < 12) { const dentro = await p.evaluate(() => { const g = document.querySelector('#recglobo').getBoundingClientRect(); return g.top >= 0 && g.left >= 0 && g.bottom <= innerHeight + 1 && g.right <= innerWidth + 1; }); if (!dentro) { ok(false, 'el globo se sale de la pantalla en el paso ' + n); } await p.click('#recglobo [data-r=sig]'); }
  ok(!(await p.$('#recglobo')) && n >= 3, `«Siguiente» recorre los pasos y «Terminar» cierra (${n} pasos)`);
  await p.click('#recorrido'); await p.keyboard.press('Escape'); ok(!(await p.$('#recglobo')), 'Esc cierra el recorrido');
  await p.click('#recorrido'); await p.click('#recglobo [data-r=portal]'); ok(/Recorrido general/.test(await p.textContent('#recglobo .cuenta')), 'desde el primer paso se llega al recorrido general'); await p.keyboard.press('Escape');
  await p.click('#lienzo', { position: { x: 300, y: 300 } }); ok(true, 'tras cerrar, el mapa vuelve a recibir clics');

  console.log('Recorrido de cada sección, como administrador');
  console.log('  sesión: ' + await entrar(p));
  const secciones = await p.evaluate(() => [...document.querySelectorAll('nav > button[data-s]')].filter(b => !b.hidden).map(b => b.dataset.s));
  const sin = [];
  for (const s of secciones) {
    await p.click(`nav > button[data-s="${s}"]`); await p.waitForTimeout(500);
    const boton = await p.$(`section#${s} button.recorrer`);
    if (!boton || !(await boton.isVisible())) { sin.push(s + ' (sin botón)'); continue; }
    await boton.click(); if (!(await p.$('#recglobo'))) { sin.push(s + ' (no abre)'); continue; }
    const total = +(await p.textContent('#recglobo .cuenta')).match(/de (\d+)/)[1];
    for (let i = 0; i < total; i++) await p.keyboard.press('ArrowRight');
    if (await p.$('#recglobo')) { sin.push(s + ' (no termina)'); await p.keyboard.press('Escape'); }
    if (dir && ['mapa', 'recibir', 'tableros'].includes(s)) { await boton.click(); await p.click('#recglobo [data-r=sig]'); await p.waitForTimeout(350); await p.screenshot({ path: path.join(dir, `recorrido_${s}.png`) }); await p.keyboard.press('Escape'); }
  }
  ok(sin.length === 0, `las ${secciones.length} secciones del menú tienen botón y recorrido que abre y termina` + (sin.length ? ': ' + sin.join(', ') : ''));

  console.log('Cuenta de institución');
  const sufijo = Date.now() % 100000, usuario = 'pnc' + sufijo, otra = 'renap' + sufijo;
  const r = await p.evaluate(async ([u, o]) => {
    const pedir = (ruta, cuerpo) => fetch(ruta, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(cuerpo) }).then(async x => [x.status, await x.text()]);
    const sols = await fetch('/api/solicitudes').then(x => x.json());
    const inst = [...new Set(sols.map(s => s.institucion))];
    return { inst, sols: sols.map(s => [s.id, s.institucion]),
      mala: await pedir('/api/usuarios', { usuario: u + 'x', nombre: 'Sin institución', rol: 'institucion', clave: 'Inicial-Prueba-1', institucion: 'No existe' }),
      una: await pedir('/api/usuarios', { usuario: u, nombre: 'Enlace PNC', rol: 'institucion', clave: 'Inicial-Prueba-1', institucion: 'PNC' }),
      dos: await pedir('/api/usuarios', { usuario: o, nombre: 'Enlace RENAP', rol: 'institucion', clave: 'Inicial-Prueba-1', institucion: 'RENAP' }) };
  }, [usuario, otra]);
  ok(r.inst.includes('PNC') && r.inst.includes('RENAP'), 'hay solicitudes de PNC y RENAP: ' + r.inst.join(', '));
  ok(r.mala[0] === 400, 'no se crea una cuenta de institución sin una institución con solicitud');
  ok(r.una[0] === 200 && r.dos[0] === 200, 'se crean las cuentas de PNC y RENAP');
  await p.click('nav > button[data-s=usuarios]'); await p.waitForFunction(u => document.querySelector('#tusuarios')?.textContent.includes(u), usuario);
  ok(await p.evaluate(u => [...document.querySelectorAll('#tusuarios tr')].some(f => f.textContent.includes(u) && f.querySelector('select[data-inst]')?.value === 'PNC'), usuario), 'la tabla de usuarios muestra su institución');
  await p.selectOption('#urol', 'institucion'); ok(await p.isVisible('#uinst') && (await p.$$eval('#uinst option', o => o.length)) >= 2, 'al crear, el rol «Institución» pide elegir la institución');
  if (dir) await p.screenshot({ path: path.join(dir, 'usuarios_institucion.png'), fullPage: true });
  await p.evaluate(() => fetch('/api/sesion', { method: 'DELETE' }));

  const solPNC = r.sols.find(s => s[1] === 'PNC')[0], solRENAP = r.sols.find(s => s[1] === 'RENAP')[0];
  // Con la contraseña inicial todavía no puede enviar.
  await p.reload(); await p.waitForSelector('#lienzo path.leaflet-interactive');
  const subir = (sol) => p.evaluate(async s => { const f = new FormData(); f.append('archivo', new Blob(['departamento,municipio,fecha\nGuatemala,Mixco,2026-01-05\n'], { type: 'text/csv' }), 'prueba_institucion.csv'); const x = await fetch('/api/envios/' + encodeURIComponent(s), { method: 'POST', body: f }); return [x.status, (await x.text()).slice(0, 160)]; }, sol);
  await p.evaluate(async u => { await fetch('/api/sesion', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ usuario: u, clave: 'Inicial-Prueba-1' }) }); }, usuario);
  const conInicial = await p.evaluate(async () => [(await fetch('/api/envios')).status, (await fetch('/api/sesion').then(x => x.json())).usuario?.debe_cambiar]);
  ok(conInicial[0] === 403 && conInicial[1] === true, `con la contraseña inicial tampoco puede leer sus envíos (${conInicial[0]})`);
  let e = await subir(solPNC); ok(e[0] === 403 && /contraseña/.test(e[1]), 'con la contraseña inicial no puede enviar: ' + e[1]);
  await p.evaluate(() => fetch('/api/sesion', { method: 'DELETE' }));
  console.log('  sesión: ' + await entrar(p, usuario, CLAVE, 'Inicial-Prueba-1'));
  const menu = await p.evaluate(() => [...document.querySelectorAll('nav > button[data-s]')].filter(b => getComputedStyle(b).display !== 'none').map(b => b.dataset.s));
  ok(menu.includes('recibir') && !menu.includes('pendientes') && !menu.includes('memoria') && !menu.includes('operadores') && !menu.includes('campo') && !menu.includes('usuarios'), 'su menú trae «Recibir envío» y no pendientes, equivalencias, operadores ni usuarios');
  ok(menu.includes('mapa') && menu.includes('rubros') && menu.includes('tableros'), 'sigue viendo el contenido abierto');
  ok(!['resultados', 'proceso', 'muestreos'].some(s => menu.includes(s)), 'no ve las secciones internas del INE');
  ok(/envía por PNC/.test(await p.textContent('#acceso')), 'el encabezado dice por quién envía');
  await p.click('nav > button[data-s=recibir]'); await p.waitForTimeout(500);
  const opciones = await p.$$eval('#sol option', o => o.map(x => x.textContent));
  ok(opciones.length >= 1 && opciones.every(o => o.startsWith('PNC')), 'la lista de solicitudes trae solo las de PNC: ' + opciones.length);
  ok(await p.isVisible('#enviapor') && /PNC/.test(await p.textContent('#enviapor')), 'un rótulo dice «Usted envía por PNC»');
  e = await subir(solRENAP); ok(e[0] === 403, 'no puede subir un archivo de RENAP: ' + e[1]);
  e = await subir(solPNC); ok(e[0] !== 403 && e[0] !== 401, `sí puede subir el de PNC (respuesta ${e[0]})`);
  const vistos = await p.evaluate(async () => { const g = async r => { const x = await fetch(r); return [x.status, x.ok ? await x.json() : null]; };
    const [se, env] = await g('/api/envios'); return { se, inst: [...new Set((env || []).map(x => x.institucion))], pend: (await g('/api/pendientes'))[0], equi: (await g('/api/equivalencias'))[0], usu: (await g('/api/usuarios'))[0],
      borrar: (await fetch('/api/envios/1', { method: 'DELETE' })).status, decidir: (await fetch('/api/confirmar', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ nivel: 'municipio', valor: 'x', contexto: '', codigo: '0101' }) })).status }; });
  ok(vistos.se === 200 && vistos.inst.every(i => i === 'PNC'), 'en el historial ve solo envíos de PNC: ' + JSON.stringify(vistos.inst));
  ok(vistos.pend === 403 && vistos.equi === 403 && vistos.usu === 403, 'no ve pendientes, equivalencias ni usuarios (403)');
  ok(vistos.borrar === 403 && vistos.decidir === 403, `no puede borrar envíos ni decidir equivalencias (${vistos.borrar}, ${vistos.decidir})`);
  await p.reload(); await p.waitForSelector('#lienzo path.leaflet-interactive'); await p.click('nav > button[data-s=recibir]'); await p.waitForTimeout(600);
  const nombre = await p.evaluate(async propio => { const c = n => fetch('/api/perfil', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ nombre: n }) }).then(x => x.status); return [await c('Administrador'), await c('admin'), await c(propio)]; }, 'Enlace de prueba ' + sufijo);
  ok(nombre[0] === 400 && nombre[1] === 400 && nombre[2] === 200, `no puede tomar el nombre ni el usuario de otra cuenta (${nombre.join(', ')})`);
  ok((await p.$$('#envios button[data-borrar-envio]')).length === 0, 'su historial no ofrece «Eliminar»');
  if (dir) await p.screenshot({ path: path.join(dir, 'institucion_recibir.png'), fullPage: true });
  // Limpieza: el envío de prueba se borra con la cuenta de administrador.
  await p.evaluate(() => fetch('/api/sesion', { method: 'DELETE' })); await p.reload(); await p.waitForSelector('#lienzo path.leaflet-interactive'); await entrar(p);
  const limpio = await p.evaluate(async () => { const env = await fetch('/api/envios').then(x => x.json()); let n = 0; for (const x of env.filter(x => x.archivo === 'prueba_institucion.csv')) { await fetch('/api/envios/' + x.id, { method: 'DELETE' }); n++; } return n; });
  console.log(`  limpieza: ${limpio} envío(s) de prueba borrado(s)`);

  console.log('Teléfono');
  const t = await nav.newPage({ viewport: { width: 390, height: 780 } }); await t.goto(url); await t.waitForSelector('#lienzo path.leaflet-interactive');
  ok(await t.isVisible('#recorrido'), 'el botón se ve en un teléfono');
  await t.click('#recorrido'); await t.waitForSelector('#recglobo');
  ok(await t.evaluate(() => { const g = document.querySelector('#recglobo').getBoundingClientRect(); return g.left >= 0 && g.right <= innerWidth + 1 && g.bottom <= innerHeight + 1 && document.documentElement.scrollWidth <= innerWidth + 1; }), 'el globo cabe y la página no se ensancha');
  if (dir) await t.screenshot({ path: path.join(dir, 'recorrido_telefono.png') });
  ok(errores.length === 0, 'sin errores de JavaScript' + (errores.length ? ': ' + errores.slice(0, 3).join(' | ') : ''));
  await nav.close();
  console.log(fallos ? `\n${fallos} FALLA(S)` : '\nTodo bien.'); process.exit(fallos ? 1 : 0);
})();
