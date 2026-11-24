// Prueba: seguridad (cabeceras, lecturas con sesión, origen ajeno, freno de intentos, contraseñas débiles),
// tableros a pedido (frase → tablero → editar → guardar → borrar) y grupos (menú y permisos por sección).
// Uso: NO_PROXY=localhost node tableros_grupos_seguridad.js http://localhost:8082 carpeta
const { chromium } = require('playwright'); const { entrar, CLAVE } = require('./_sesion');
(async () => {
  const base = process.argv[2], dir = process.argv[3]; const b = await chromium.launch(); let fallos = 0;
  const ok = (nombre, cond, detalle = '') => { console.log((cond ? 'ok   ' : 'FALLA') + ' ' + nombre + (detalle ? ' — ' + detalle : '')); if (!cond) fallos++; };
  const ctx = await b.newContext({ viewport: { width: 1440, height: 900 } }); const p = await ctx.newPage(); const errores = []; p.on('pageerror', e => errores.push(String(e).slice(0, 200)));
  await p.goto(base); await p.waitForSelector('#lienzo path.leaflet-interactive');
  // ---- Seguridad, sin sesión ----
  const cab = (await ctx.request.get(base + '/')).headers();
  ok('cabeceras de seguridad', cab['x-frame-options'] === 'DENY' && cab['x-content-type-options'] === 'nosniff' && /frame-ancestors 'none'/.test(cab['content-security-policy'] || ''), (cab['content-security-policy'] || '').slice(0, 60));
  for (const ruta of ['/api/pendientes', '/api/envios', '/api/equivalencias', '/api/usuarios', '/api/grupos']) ok('sin sesión no se lee ' + ruta, [401, 403].includes((await ctx.request.get(base + ruta)).status()));
  ok('una petición que viene de otro sitio se rechaza', (await ctx.request.post(base + '/api/precios', { headers: { origin: 'https://otro.example', 'content-type': 'text/plain' }, data: 'x' })).status() === 403);
  let ultimo = 0; for (let i = 0; i < 7; i++) ultimo = (await ctx.request.post(base + '/api/sesion', { data: { usuario: 'zz-no-existe', clave: 'mala' + i } })).status();
  ok('tras varios intentos fallidos el usuario queda bloqueado', ultimo === 429, 'último estado ' + ultimo);
  ok('recorrido de rutas', (await ctx.request.get(base + '/ejemplos/..%2f..%2fCargo.toml')).status() === 404);
  // ---- Con sesión de administrador ----
  console.log('     ingreso:', await entrar(p));
  const api = (ruta, cuerpo, metodo = 'POST') => p.evaluate(async ([r, c, m]) => { const x = await fetch(r, { method: m, headers: c ? { 'content-type': 'application/json' } : {}, body: c ? JSON.stringify(c) : undefined }); return [x.status, await x.json().catch(() => null)]; }, [ruta, cuerpo, metodo]);
  const u = 'zz' + Date.now().toString(36);
  ok('contraseña débil rechazada', (await api('/api/usuarios', { usuario: u, nombre: 'Prueba', rol: 'analista', clave: 'aaaaaaaaaa' }))[0] === 400);
  ok('contraseña igual al usuario rechazada', (await api('/api/usuarios', { usuario: u, nombre: 'Prueba', rol: 'analista', clave: u + '-1' }))[0] === 400);
  // ---- Tableros ----
  await p.click('nav > button[data-s=tableros]'); await p.fill('#tabtexto', 'Mapa y tabla de desnutrición y población en Sololá, los 5 municipios más altos'); await p.click('#tabarmar'); await p.waitForSelector('#tabvista .pieza svg path');
  console.log('     ' + (await p.textContent('#tabentendido')).replace(/\s+/g, ' ').trim().slice(0, 230));
  ok('la frase produce el tablero pedido', (await p.locator('#tabvista .pieza').count()) === 4 && (await p.inputValue('#tablugar')) === '07' && (await p.locator('#tabvista .pieza table tbody tr').first().locator('..').locator('tr').count()) === 5, (await p.textContent('#tabvista h2')));
  await p.screenshot({ path: `${dir}/90-tablero.png`, fullPage: false });
  await p.selectOption('#tabpiezas tbody tr:first-child select[data-k=tipo]', 'barras'); await p.click('#tabagregar'); await p.waitForTimeout(300);
  ok('el editor cambia y agrega piezas', (await p.locator('#tabvista .pieza').count()) === 5 && (await p.locator('#tabvista .barrah').count()) >= 5);
  await p.click('#tabguardar'); await p.waitForFunction(() => /n\.º \d+/.test(document.querySelector('#tabestado').textContent)); const id = (await p.textContent('#tabestado')).match(/\d+/)[0];
  ok('se guarda y aparece en la lista', (await p.locator(`#ttableros button[data-abrir="${id}"]`).count()) === 1);
  ok('una definición inválida se rechaza', (await api('/api/tableros', { definicion: { titulo: 'x', territorio: { nivel: 'pais' }, piezas: [{ tipo: 'guion', indicador: 'poblacion_actual', n: 5, orden: 'desc' }] } }))[0] === 400);
  // ---- Grupos ----
  const gr = (await api('/api/grupos', null, 'GET'))[1].grupos.find(g => g.nombre === 'Precios e índices');
  const cu = await api('/api/usuarios', { usuario: u, nombre: 'Prueba de grupo', rol: 'analista', clave: 'Inicial-2026-xy' }); ok('usuario de prueba creado', cu[0] === 200, JSON.stringify(cu[1]));
  const uid = (await api('/api/usuarios', null, 'GET'))[1].find(x => x.usuario === u).id;
  ok('se asigna al grupo', (await api('/api/usuarios/' + uid, { grupo_id: gr.id }))[0] === 200);
  const ctx2 = await b.newContext({ viewport: { width: 1440, height: 900 } }); const p2 = await ctx2.newPage(); await p2.goto(base); await p2.waitForSelector('#lienzo path.leaflet-interactive');
  console.log('     ingreso del usuario del grupo:', await entrar(p2, u, 'Propia-2026-xy', 'Inicial-2026-xy'));
  const menu = await p2.locator('nav > button[data-s]').evaluateAll(bs => bs.filter(x => !x.hidden).map(x => x.dataset.s));
  ok('su menú solo trae las secciones del grupo', menu.includes('rubros') && menu.includes('muestreos') && !menu.includes('recibir') && !menu.includes('pendientes') && !menu.includes('usuarios'), menu.join(','));
  const fuera = await p2.evaluate(async () => (await fetch('/api/confirmar', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ nivel: 'municipio', contexto: '', valor: 'x', codigo: '0101', institucion: 'x' }) })).status);
  const dentro = await p2.evaluate(async () => (await fetch('/api/publicados/decision', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ rubro: 'supermercado', ean: '0000000000000', proveedor: 'prueba', decision: '' }) })).status);
  ok('fuera de su grupo no puede cambiar nada (403) y dentro sí (200)', fuera === 403 && dentro === 200, `${fuera} / ${dentro}`);
  await p2.screenshot({ path: `${dir}/91-menu-de-grupo.png` });
  // ---- Limpieza ----
  await api('/api/tableros/' + id, null, 'DELETE'); await api('/api/usuarios/' + uid, { activo: false });
  ok('sin errores de JavaScript', errores.length === 0, JSON.stringify(errores));
  console.log(fallos ? `FALLOS: ${fallos}` : 'todo bien'); await b.close(); process.exit(fallos ? 1 : 0);
})();
