// Trabajo de campo: el coordinador arma una ruta; el operador (otro usuario, en su grupo) ve su ruta,
// registra visitas con la posición que entrega el dispositivo y una foto, y recibe el enlace a la encuesta.
// Uso: NO_PROXY=localhost node .faro/pruebas/trabajo_de_campo.js http://localhost:8095 <carpeta>
// La posición se simula con la geolocalización del navegador de pruebas (no hay GPS en el contenedor).
// Opcional: dos fotos reales (una vivienda y un comercio) como 4.º y 5.º argumento, para probar el clasificador.
const { chromium } = require('playwright');
const { entrar } = require('./_sesion.js');
(async () => {
  const base = process.argv[2] || 'http://localhost:8082', dir = process.argv[3] || '.', fotoVivienda = process.argv[4], fotoComercio = process.argv[5];
  const nav = await chromium.launch(); const ctx = await nav.newContext({ viewport: { width: 1536, height: 864 } }); const p = await ctx.newPage();
  const errores = []; p.on('pageerror', e => errores.push('coord: ' + e.message));
  let fallas = 0; const ok = (t, c, extra = '') => { console.log((c ? '  ok  ' : ' FALLA ') + t + (extra ? ' — ' + extra : '')); if (!c) fallas++; };
  const pedir = pag => (ruta, cuerpo, metodo) => pag.evaluate(async ([r, c, m]) => { const x = await fetch(r, { method: m || (c ? 'POST' : 'GET'), headers: c ? { 'content-type': 'application/json' } : {}, body: c ? JSON.stringify(c) : undefined }); const t = await x.text(); let j; try { j = JSON.parse(t); } catch { j = t; } return [x.status, j]; }, [ruta, cuerpo, metodo]);
  await p.goto(base); await p.waitForSelector('#lienzo path.leaflet-interactive'); console.log('sesión:', await entrar(p)); const api = pedir(p);
  // Usuario operador, en el grupo «Operadores de campo».
  const u = 'campo' + Date.now().toString().slice(-6), clave0 = 'Inicial-2026-xy', clave = 'Campo-Prueba-2026';
  ok('se crea el operador', (await api('/api/usuarios', { usuario: u, nombre: 'Operadora de prueba', rol: 'operador', clave: clave0 }))[0] === 200);
  const uid = (await api('/api/usuarios', null, 'GET'))[1].find(x => x.usuario === u).id, grupo = (await api('/api/grupos', null, 'GET'))[1].grupos.find(g => g.nombre === 'Operadores de campo');
  ok('existe el grupo «Operadores de campo» y se le asigna', !!grupo && (await api('/api/usuarios/' + uid, { grupo_id: grupo.id }))[0] === 200, grupo && grupo.secciones.join(','));
  // El coordinador arma la ruta desde la pantalla.
  await p.click('nav > button[data-s=operadores]'); await p.waitForSelector('#campocoord:not([hidden])'); await p.waitForFunction(() => document.querySelector('#crdep').options.length > 5);
  await p.fill('#crnombre', 'Prueba de campo, Sololá'); await p.selectOption('#croperador', String(uid)); await p.selectOption('#crdep', '07'); await p.waitForFunction(() => document.querySelector('#crmun').options.length > 5); await p.selectOption('#crmun', '0701');
  await p.waitForSelector('#crlugares input'); const total = await p.locator('#crlugares input').count(); await p.fill('#crbuscar', 'san'); await p.waitForTimeout(200); const filtrados = await p.locator('#crlugares input').count(); await p.click('#crtodos');
  await p.fill('#crform', 'https://ejemplo.net/encuesta?l={lugar}'); await p.click('#crcrear'); await p.waitForSelector('#aviso.mal'); ok('un formulario fuera de los servicios admitidos se rechaza', /servicios/.test(await p.textContent('#aviso')), (await p.textContent('#aviso')).slice(0, 90));
  await p.fill('#crform', 'https://forms.office.com/r/PRUEBA?lugar={lugar}&lat={latitud}&lon={longitud}&v={visita}&op={operador}'); await p.click('#crcrear'); await p.waitForFunction(() => /Ruta creada/.test(document.querySelector('#aviso').textContent));
  const creada = await p.textContent('#aviso'); ok('la ruta se crea con las paradas marcadas y se ordena', filtrados > 1 && filtrados < total && new RegExp(filtrados + ' paradas').test(creada), `${creada} (${filtrados} de ${total} lugares)`);
  const datos = (await api('/api/campo', null, 'GET'))[1], ruta = datos.rutas.find(r => r.nombre === 'Prueba de campo, Sololá'), par = ruta.paradas;
  await p.screenshot({ path: `${dir}/101-campo-coordinacion.png`, fullPage: false });
  // El operador entra con su usuario: cambia la contraseña inicial y ve solo su menú.
  const ctxOp = await nav.newContext({ viewport: { width: 412, height: 915 }, isMobile: true, hasTouch: true, geolocation: { latitude: par[0].latitud + 0.002, longitude: par[0].longitud + 0.001, accuracy: 14 }, permissions: ['geolocation'] });
  const o = await ctxOp.newPage(); o.on('pageerror', e => errores.push('operador: ' + e.message)); await o.goto(base); await o.waitForSelector('#lienzo path.leaflet-interactive'); console.log('operador:', await entrar(o, u, clave, clave0)); const apiOp = pedir(o);
  const menu = await o.evaluate(() => [...document.querySelectorAll('nav > button[data-s]')].filter(b => getComputedStyle(b).display !== 'none').map(b => b.dataset.s));
  ok('el operador ve solo su menú', menu.join(',') === 'mapa,campo,acerca,perfil', menu.join(','));
  ok('el operador no puede crear rutas ni hacer otros cambios', (await apiOp('/api/campo/rutas', { nombre: 'x', operador_id: uid, lugares: [par[0].lugar] }))[0] === 403 && (await apiOp('/api/tableros', { definicion: { titulo: 'x', territorio: { nivel: 'pais' }, piezas: [{ tipo: 'tarjeta', indicador: 'poblacion_actual' }] } }))[0] === 403);
  await o.click('#menu'); await o.click('nav > button[data-s=campo]'); await o.waitForSelector('#campoparadas .parada');
  ok('ve su ruta con las paradas en orden', (await o.locator('#campoparadas .parada').count()) === par.length && (await o.locator('#campomapa .marca-parada').count()) === par.length);
  // Visita 1: en el lugar, con foto.
  await o.locator('#campoparadas button[data-visitar]').first().click(); await o.waitForFunction(() => /del punto/.test(document.querySelector('#cvpos').textContent));
  ok('el dispositivo entrega la posición sola', /± 14 m/.test(await o.textContent('#cvpos')), (await o.textContent('#cvpos')).trim());
  if (fotoVivienda) await o.setInputFiles('#cvfoto', fotoVivienda); else await o.setInputFiles('#cvfoto', { name: 'fachada.png', mimeType: 'image/png', buffer: await o.locator('#campomapa').screenshot() });
  await o.waitForSelector('#cvvista img');
  await o.fill('#cvnotas', 'Vivienda de un nivel.'); await o.click('#cvregistrar'); await o.waitForSelector('#cvresultado .veredicto');
  const v1 = (await o.textContent('#cvresultado')).replace(/\s+/g, ' ').trim(); ok('la visita se registra y se cruza', /Visita n\.º \d+/.test(v1) && /posición a \d+ m del punto/.test(v1), v1.slice(0, 260));
  const enlace = await o.getAttribute('#cvabrir', 'href'); ok('el enlace a la encuesta lleva el lugar, la posición y el número de visita', new RegExp(`lugar=${par[0].lugar}&lat=\\d+\\.\\d{6}&lon=-\\d+\\.\\d{6}&v=\\d+&op=${u}$`).test(enlace), enlace);
  if (fotoVivienda) ok('la foto de una vivienda cuenta a favor', /la foto parece una vivienda/.test(v1) && /ya se descartó/.test(v1));
  await o.screenshot({ path: `${dir}/102-campo-visita-movil.png` }); await o.click('#campovisita .cerrar');
  // Con una foto de comercio en el lugar correcto, la visita se marca para revisar.
  if (fotoComercio) { await o.locator('#campoparadas button[data-visitar]').first().click(); await o.waitForFunction(() => /del punto/.test(document.querySelector('#cvpos').textContent)); await o.setInputFiles('#cvfoto', fotoComercio); await o.waitForSelector('#cvvista img');
    await o.click('#cvregistrar'); await o.waitForSelector('#cvresultado .veredicto'); const vc = (await o.textContent('#cvresultado')).replace(/\s+/g, ' ');
    ok('la foto de un comercio marca la visita para revisar', /Conviene revisar/.test(vc) && /parece un comercio/.test(vc), vc.slice(0, 230)); await o.screenshot({ path: `${dir}/102b-campo-foto-comercio.png` }); await o.click('#campovisita .cerrar'); }
  // Visita 2: lejos del punto asignado.
  await ctxOp.setGeolocation({ latitude: par[1].latitud + 0.2, longitude: par[1].longitud, accuracy: 30 });
  await o.locator('#campoparadas button[data-visitar]').nth(1).click(); await o.waitForFunction(() => /del punto/.test(document.querySelector('#cvpos').textContent)); await o.click('#cvregistrar'); await o.waitForSelector('#cvresultado .veredicto.revisar');
  ok('una visita lejos del lugar se marca para revisar', /km del lugar poblado asignado/.test(await o.textContent('#cvresultado')), (await o.textContent('#cvresultado .veredicto')).replace(/\s+/g, ' ').slice(0, 200));
  await o.click('#campovisita .cerrar'); await o.waitForSelector('#campoparadas .parada.revisar'); await o.screenshot({ path: `${dir}/103-campo-ruta-movil.png`, fullPage: true });
  // Sin sesión no se ve nada; el coordinador ve las visitas y las descarga.
  const anon = await (await nav.newContext()).newPage(); await anon.goto(base); ok('sin sesión no hay acceso', (await pedir(anon)('/api/campo', null, 'GET'))[0] === 401 && (await pedir(anon)('/api/campo/visitas', { parada_id: par[0].id }))[0] === 401);
  await p.click('nav > button[data-s=mapa]'); await p.click('nav > button[data-s=operadores]'); await p.waitForFunction(() => /n\.º \d+/.test(document.querySelector('#tvisitas').textContent));
  ok('el coordinador ve las visitas con su resultado', (await p.locator('#tvisitas tbody tr').count()) >= 2 && /Conviene revisar/.test(await p.textContent('#tvisitas')));
  const csv = (await api('/api/campo/visitas.csv', null, 'GET'))[1]; ok('las visitas se descargan en CSV para unirlas con el formulario', /^visita,cuando,ruta,operador/.test(csv) && csv.includes(par[0].lugar));
  ok('queda en la bitácora', (await api('/api/bitacora', null, 'GET'))[1].some(b => b.accion === 'visita_de_campo'));
  await p.screenshot({ path: `${dir}/104-campo-visitas.png`, fullPage: true });
  // Limpieza.
  ok('la ruta se borra', (await api('/api/campo/rutas/' + ruta.id, null, 'DELETE'))[0] === 200); await api('/api/usuarios/' + uid, { activo: false });
  ok('sin errores de consola', errores.length === 0, errores.slice(0, 3).join(' | '));
  await nav.close(); console.log(fallas ? `\n${fallas} falla(s)` : '\nTodo bien'); process.exit(fallas ? 1 : 0);
})();
