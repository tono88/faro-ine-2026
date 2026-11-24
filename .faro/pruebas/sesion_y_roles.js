// HISTÓRICO (v0.9.0): solo funciona en una base recién creada, porque entra con admin / admin y cambia esa
// contraseña. No forma parte de la batería actual: sesión, roles y grupos se prueban en tableros_grupos_seguridad.js.
const { chromium } = require('playwright');
(async () => {
  const [,, base, dir] = process.argv;
  const b = await chromium.launch(); const ctx = await b.newContext({ viewport: { width: 1440, height: 900 } }); const p = await ctx.newPage();
  const errores = []; p.on('pageerror', e => errores.push(String(e).slice(0, 200)));
  const t = async sel => (await p.textContent(sel)).replace(/\s+/g, ' ');
  const entrar = async (u, c) => { await p.click('#entrar'); await p.fill('#iusuario', u); await p.fill('#iclave', c); await p.click('#formingreso button[type=submit]'); await p.waitForTimeout(1500); };
  await p.goto(base); await p.waitForSelector('#lienzo path.leaflet-interactive');
  // sin sesión: una operación de cambio pide entrar
  const sin = await p.evaluate(() => fetch('/api/muestreos/alimentos-walmart', {method:'POST', headers:{'content-type':'application/json'}, body:'{"activa":true}'}).then(r => r.status)); console.log('sin sesión, cambio:', sin);
  await entrar('admin', 'incorrecta'); console.log('clave mala:', await t('#ingerror')); await p.click('#ingreso .cerrar');
  await entrar('admin', 'admin'); console.log('cabecera:', await t('#acceso'), '| aviso clave visible:', await p.isVisible('#aviso-clave'), '| menú usuarios visible:', await p.isVisible('nav button[data-s=usuarios]'));
  await p.click('nav button[data-s=perfil]'); await p.fill('#pactual', 'admin'); await p.fill('#pnueva', 'corta'); await p.fill('#pnueva2', 'corta'); await p.click('#pclave'); await p.waitForTimeout(700); console.log('clave corta:', await t('#aviso'));
  await p.fill('#pnueva', 'clave-nueva-2026'); await p.fill('#pnueva2', 'clave-nueva-2026'); await p.click('#pclave'); await p.waitForTimeout(1500); console.log('cambio:', await t('#aviso'), '| aviso clave visible:', await p.isVisible('#aviso-clave'));
  await p.click('nav button[data-s=usuarios]'); await p.waitForSelector('#tusuarios tbody tr');
  await p.fill('#uusuario', 'lectora'); await p.fill('#unombre', 'Persona de consulta'); await p.selectOption('#urol', 'consulta'); await p.fill('#uclave', 'consulta-2026'); await p.click('#ucrear'); await p.waitForTimeout(1500);
  console.log('usuarios:', (await p.locator('#tusuarios tbody tr').allTextContents()).map(x => x.replace(/\s+/g, ' ').slice(0, 60)));
  await p.locator('#tusuarios input[data-act]').first().uncheck(); await p.waitForTimeout(900); console.log('desactivarse a sí mismo:', await t('#aviso'));
  await p.screenshot({ path: dir + '/30-usuarios.png' });
  // ficha dentro del mapa
  await p.click('nav button[data-s=mapa]'); await p.waitForTimeout(500); await p.locator('#lienzo path.leaflet-interactive').nth(300).click({force:true}); await p.waitForSelector('#ficha .mini');
  const cajas = await p.evaluate(() => { const f = document.querySelector('#ficha').getBoundingClientRect(), m = document.querySelector('#lienzo').getBoundingClientRect(); return {dentro: f.top >= m.top && f.bottom <= m.bottom + 1 && f.right <= m.right, fichaAlto: Math.round(f.height), mapaAlto: Math.round(m.height), desborda: document.querySelector('#ficha').scrollHeight > document.querySelector('#ficha').clientHeight + 2}; });
  console.log('ficha:', JSON.stringify(cajas), '|', (await t('#ficha')).match(/Inseguridad alimentaria[^.]*\./)?.[0]); await p.screenshot({ path: dir + '/31-ficha.png' });
  await p.selectOption('#indicador', 'cif_pct'); await p.waitForTimeout(500); console.log('leyenda CIF:', await t('#leyenda'));
  await p.click('nav button[data-s=alimentaria]'); await p.waitForSelector('#tcif tbody tr td b'); console.log('CIF:', await t('#ciftarjetas'), '|', (await t('#tcif tbody tr:first-child')).slice(0, 110));
  await p.click('#salir'); await p.waitForTimeout(700); await entrar('lectora', 'consulta-2026');
  const cons = await p.evaluate(() => fetch('/api/muestreos/alimentos-walmart', {method:'POST', headers:{'content-type':'application/json'}, body:'{"activa":true}'}).then(r => r.json())); console.log('consulta intenta cambiar:', JSON.stringify(cons), '| ve Usuarios:', await p.isVisible('nav button[data-s=usuarios]'));
  const adm = await p.evaluate(() => fetch('/api/usuarios').then(r => r.status)); console.log('consulta lista usuarios:', adm);
  console.log('errores js:', JSON.stringify(errores)); await b.close();
})();
