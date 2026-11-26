// Guía para evaluadores arriba del menú, comparador de territorios y certificación de reportes.
// Uso: NO_PROXY=localhost FARO_CLAVE_PRUEBA=... node .faro/pruebas/comparar_y_certificar.js http://localhost:8082 [carpeta de capturas]
const { chromium } = require('playwright'); const { entrar, CLAVE } = require('./_sesion.js');
(async () => {
  const base = process.argv[2] || 'http://localhost:8082', dir = process.argv[3]; const nav = await chromium.launch();
  const ctx = await nav.newContext({ viewport: { width: 1440, height: 900 } }); await ctx.addInitScript(() => { try { localStorage.setItem('faro_recorrido_visto', '1'); } catch (_) {} });
  const p = await ctx.newPage(); const errores = []; p.on('pageerror', e => errores.push(e.message)); let fallas = 0;
  const ok = (t, c, x = '') => { console.log((c ? '  ok  ' : ' FALLA ') + t + (x ? ' — ' + x : '')); if (!c) fallas++; };
  await p.goto(base); await p.waitForSelector('#lienzo path.leaflet-interactive'); await p.waitForTimeout(600);

  console.log('Sin sesión');
  const primero = await p.evaluate(() => { const b = document.querySelector('nav > button:not([hidden])'); return { s: b.dataset.s, t: b.textContent.trim(), color: getComputedStyle(b).color }; });
  ok('lo primero del menú es la guía para evaluadores, en rojo', primero.s === 'mejoras' && primero.t === 'Guía para evaluadores' && /200, 16, 46/.test(primero.color), JSON.stringify(primero));
  ok('el encabezado dice «for» junto al logotipo del INE', await p.isVisible('header .marca .para img') && /for/.test(await p.textContent('header .marca .para')));
  await p.click('nav > button[data-s=mejoras]'); await p.waitForSelector('.mejora');
  ok('la guía trae los seis criterios con su peso', (await p.locator('.guia-criterios tbody tr').count()) === 6 && /25 %/.test(await p.textContent('.guia-criterios')));
  if (dir) await p.screenshot({ path: dir + '/guia-evaluadores.png' });
  await p.click('.guia-criterios a[href="#comparar"]'); await p.waitForSelector('#compbuscar');
  ok('un enlace de la guía lleva a la sección', (await p.evaluate(() => document.querySelector('section.activa').id)) === 'comparar');
  // Como en un teléfono: se escribe una parte del nombre y se toca la sugerencia; el campo queda libre y sin foco.
  await p.click('#compbuscar'); await p.keyboard.type('jutia', { delay: 30 }); await p.waitForSelector('.sugerencias li');
  const sug = await p.$$eval('#compbuscar + .sugerencias li', c => c.map(x => x.textContent));
  ok('al escribir aparecen sugerencias debajo del campo', sug.length > 0 && sug[0] === 'Jutiapa, Jutiapa' && await p.evaluate(() => { const c = document.querySelector('#compbuscar').getBoundingClientRect(), u = document.querySelector('#compbuscar + .sugerencias').getBoundingClientRect(); return u.top >= c.bottom - 1; }), sug.slice(0, 3).join(' | '));
  await p.dispatchEvent('#compbuscar + .sugerencias li', 'pointerdown');
  ok('al tocar una sugerencia se agrega, el campo queda vacío y suelta el foco (el teclado se cierra)', (await p.locator('#compelegidos span').count()) === 1 && (await p.inputValue('#compbuscar')) === '' && await p.evaluate(() => document.activeElement.id !== 'compbuscar' && document.querySelector('#compbuscar + .sugerencias').hidden));
  await p.click('#compbuscar'); await p.keyboard.type('comapa', { delay: 30 }); await p.keyboard.press('Enter'); await p.waitForTimeout(200);
  ok('el segundo territorio también se puede escribir y elegir (con Intro)', (await p.locator('#compelegidos span').count()) === 2 && await p.isVisible('#compresultado'));
  await p.click('#compelegidos button[data-quitar]'); await p.click('#compelegidos button[data-quitar]'); await p.waitForTimeout(150);
  const agregar = async t => { await p.fill('#compbuscar', t); await p.dispatchEvent('#compbuscar', 'input'); await p.waitForTimeout(250); };
  await agregar('Mixco, Guatemala'); ok('con un solo territorio todavía no compara', !(await p.isVisible('#compresultado')));
  await agregar('Villa Nueva, Guatemala'); await p.waitForSelector('#tcomp tbody tr');
  const filas = await p.locator('#tcomp tbody tr').count(), cab = await p.$$eval('#tcomp thead th', c => c.map(x => x.textContent.trim()));
  ok('dos municipios: tabla con indicadores y columna de diferencia', filas >= 20 && cab.length === 4 && cab[3] === 'Diferencia', `${filas} indicadores · ${cab.join(' | ')}`);
  const pob = await p.evaluate(() => { const f = geoMuni.features.map(x => x.properties), a = f.find(x => x.nombre === 'Mixco'), b = f.find(x => x.nombre === 'Villa Nueva'); return [a.poblacion_actual, b.poblacion_actual].map(v => Number(v).toLocaleString('es-GT')); });
  const fila1 = await p.$$eval('#tcomp tbody tr:first-child td b', c => c.map(x => x.textContent.trim()));
  ok('las cifras son las del mapa', fila1[0] === pob[0] && fila1[1] === pob[1], fila1.join(' / '));
  ok('el mapa pinta los dos territorios', (await p.locator('#complienzo path.leaflet-interactive').count()) === 2);
  await agregar('Sololá, Sololá'); await agregar('Cobán, Alta Verapaz'); await agregar('Flores, Petén'); await p.waitForTimeout(200);
  ok('se comparan hasta cuatro', (await p.locator('#compelegidos span').count()) === 4 && (await p.$$eval('#tcomp thead th', c => c.length)) === 5);
  if (dir) { await p.waitForTimeout(700); await p.screenshot({ path: dir + '/comparar-municipios.png' }); }
  const [descarga] = await Promise.all([p.waitForEvent('download'), p.click('#compcsv')]); const csv = require('fs').readFileSync(await descarga.path(), 'utf8');
  ok('la comparación se descarga como CSV', /indicador/.test(csv) && /Mixco, Guatemala \(0108\)/.test(csv) && csv.split('\n').length > 20);
  await p.click('#compnivel button[data-n=departamento]'); await agregar('Petén'); await agregar('Guatemala'); await p.waitForSelector('#tcomp tbody tr');
  const dep = await p.evaluate(() => { const ms = geoMuni.features.map(x => x.properties).filter(x => x.departamento_codigo === '17'); return ms.reduce((t, x) => t + (+x.poblacion_actual || 0), 0).toLocaleString('es-GT'); });
  ok('por departamento: la población es la suma de sus municipios', (await p.textContent('#tcomp tbody tr:first-child td:nth-child(2) b')).trim() === dep, dep);
  if (dir) { await p.waitForTimeout(700); await p.screenshot({ path: dir + '/comparar-departamentos.png' }); }

  console.log('Reporte sin sesión');
  await p.goto(base + '/reporte?municipio=1709'); await p.waitForSelector('.kpi'); await p.waitForTimeout(500);
  ok('sin sesión no aparece «Certificar reporte»', !(await p.isVisible('#certificar')));
  ok('sin sesión el servidor no certifica', (await p.evaluate(async () => (await fetch('/api/certificaciones', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ municipio: '1709', expediente: 'X' }) })).status)) === 401);

  console.log('Con un usuario del grupo «Acceso a la información»');
  await p.goto(base); await p.waitForSelector('#lienzo path.leaflet-interactive'); await entrar(p);
  const grupo = await p.evaluate(async () => (await (await fetch('/api/grupos')).json()).grupos.find(g => g.nombre === 'Acceso a la información'));
  ok('existe el grupo, con lo de Territorio y cartografía más «certificar»', !!grupo && grupo.secciones.includes('certificar') && ['crecimiento', 'vial', 'operadores'].every(s => grupo.secciones.includes(s)), grupo && grupo.secciones.join(','));
  const u = 'cert.' + Date.now().toString(36);
  const creado = await p.evaluate(async ([u, c, g]) => { const r = await fetch('/api/usuarios', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ usuario: u, nombre: 'Prueba de certificación ' + u, rol: 'analista', clave: c + '-0', grupo_id: g }) }); const lista = await (await fetch('/api/usuarios')).json(), j = (lista.usuarios || lista).find(x => x.usuario === u) || {};
    if (r.ok && j.id) await fetch('/api/usuarios/' + j.id, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ grupo_id: g }) }); return r.status; }, [u, CLAVE, grupo.id]);
  await p.evaluate(() => fetch('/api/sesion', { method: 'DELETE' })); const entro = await entrar(p, u, CLAVE, CLAVE + '-0');
  const yo = await p.evaluate(async () => (await (await fetch('/api/sesion')).json()).usuario);
  ok('la cuenta quedó en el grupo', creado === 200 && !/NO /.test(entro) && yo.secciones && yo.secciones.includes('certificar'), `${creado} ${entro} ${yo.secciones}`);
  await p.goto(base + '/reporte?municipio=1709'); await p.waitForSelector('.kpi'); await p.waitForSelector('#certificar:not([hidden])');
  await p.click('#certificar'); await p.fill('#certdatos [name=expediente]', '0000-2026'); await p.fill('#certdatos [name=solicitante]', 'Persona de Prueba'); await p.fill('#certdatos [name=correo]', 'prueba@example.org'); await p.fill('#certdatos [name=pedido]', 'Población y superficie construida del municipio.');
  await p.click('#certdatos button[type=submit]'); await p.waitForSelector('#constancia:not([hidden]) .sello');
  const texto = await p.textContent('#constancia'), folio = (texto.match(/FARO-\d{4}-\d{5}/) || [])[0];
  ok('se agrega la constancia con folio, expediente, solicitante y huella', !!folio && /0000-2026/.test(texto) && /Persona de Prueba/.test(texto) && /[0-9a-f]{64}/.test(texto) && /San Luis, Petén/.test(texto), folio);
  ok('la constancia dice que es un borrador, no un documento oficial', /Borrador generado por FARO/.test(texto) && /No es un documento oficial/.test(texto));
  ok('lleva la marca de agua «PRUEBA» y va sin firma', (await p.evaluate(() => getComputedStyle(document.querySelector('#constancia'), '::before').content)) === '"PRUEBA"' && /SIN FIRMA · DOCUMENTO DE PRUEBA/.test(texto));
  const correo = await p.getAttribute('#correo', 'data-enlace') || '';
  ok('«Enviar por correo» abre un mensaje ya escrito (mailto), sin servidor de correo', /^mailto:prueba%40example\.org\?subject=/.test(correo) && correo.includes(encodeURIComponent(folio)), correo.slice(0, 70));
  if (dir) { await p.screenshot({ path: dir + '/reporte-certificado.png', fullPage: false }); await p.pdf({ path: dir + '/reporte-certificado.pdf', format: 'Letter', printBackground: true }); }
  const ver = await p.evaluate(async f => (await (await fetch('/api/certificaciones/' + f)).json()), folio);
  ok('el folio se puede comprobar y los datos no han cambiado', ver.folio === folio && ver.datos_sin_cambios === true && ver.municipio === '1709' && !('solicitante' in ver), JSON.stringify(ver).slice(0, 160));
  ok('del solicitante no se guardó nada', !/Persona de Prueba|example\.org/.test(JSON.stringify(ver)));

  ok('sin errores de JavaScript', errores.length === 0, errores.slice(0, 2).join(' | '));
  await nav.close(); console.log(fallas ? `\n${fallas} falla(s)` : '\nTodo bien'); process.exit(fallas ? 1 : 0);
})().catch(e => { console.error(e); process.exit(1); });
