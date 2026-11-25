// Deja una instalación NUEVA de FARO lista para demostrar, grabar el video o generar los PDF, sin usar ningún
// dato real: cambia la contraseña inicial del administrador, crea una cuenta de cada rol, sube los archivos de
// ejemplo (ficticios) que el propio portal ofrece para cada solicitud, deja unos nombres dudosos para «Revisar
// pendientes», publica un tablero y carga la ruta de demostración. Todo lo que crea queda rotulado como ejemplo.
// Se puede repetir: lo que ya existe no se duplica. Requiere Node y Playwright con Chromium.
//
// Uso: node scripts/entregables/preparar_demo.js <url> <contraseña para las cuentas de demostración>
//      (la contraseña: al menos 10 caracteres, con letras y números o signos; no use una real)
// Para empezar de cero antes de grabar: docker compose down -v && docker compose up -d
const { chromium } = require('playwright');
const [url = 'http://localhost:8082', clave = process.env.FARO_CLAVE || ''] = process.argv.slice(2);
const INICIAL = 'Inicial-Demo-2026';
const CUENTAS = [['mlopez', 'María López', 'analista', ''], ['jperez', 'Jorge Pérez', 'consulta', ''], ['campo01', 'Luis Tzoc', 'operador', ''],
  ['enlace.pnc', 'Enlace PNC', 'institucion', 'PNC'], ['enlace.renap', 'Enlace RENAP', 'institucion', 'RENAP'], ['enlace.conred', 'Enlace SE-CONRED', 'institucion', 'SE-CONRED']];
(async () => {
  if (clave.length < 10) { console.error('Indique la contraseña de demostración (segundo argumento o FARO_CLAVE), de 10 caracteres o más.'); process.exit(2); }
  const nav = await chromium.launch(), p = await nav.newPage({ viewport: { width: 1366, height: 850 } }); p.on('dialog', d => d.accept());
  await p.goto(url); await p.waitForSelector('#lienzo path.leaflet-interactive');
  const hecho = [];
  // Entrar con la contraseña ya puesta o, si es la primera vez, con la inicial y cambiarla.
  const entrar = (usuario, inicial) => p.evaluate(async ([u, c, ini]) => { const pedir = (r, b) => fetch(r, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(b) });
    if ((await pedir('/api/sesion', { usuario: u, clave: c })).ok) return 'ya tenía la contraseña de demostración';
    const r = await pedir('/api/sesion', { usuario: u, clave: ini }); if (!r.ok) return 'NO ENTRÓ: ' + (await r.text());
    const c2 = await pedir('/api/perfil', { clave_actual: ini, clave_nueva: c }); return c2.ok ? 'contraseña inicial cambiada' : 'NO CAMBIÓ: ' + (await c2.text()); }, [usuario, clave, inicial]);
  const salir = () => p.evaluate(() => fetch('/api/sesion', { method: 'DELETE' }));
  const admin = await entrar('admin', 'admin'); hecho.push('admin: ' + admin); if (/^NO/.test(admin)) { console.error(admin); process.exit(1); }

  // Cuentas de cada rol.
  hecho.push(...await p.evaluate(async ([cuentas, ini]) => { const ya = new Set((await fetch('/api/usuarios').then(x => x.json())).map(u => u.usuario)), out = [];
    for (const [usuario, nombre, rol, institucion] of cuentas) { if (ya.has(usuario)) { out.push(usuario + ': ya existía'); continue; }
      const r = await fetch('/api/usuarios', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ usuario, nombre, rol, clave: ini, institucion }) }); out.push(`${usuario} (${rol}${institucion ? ', ' + institucion : ''}): ${r.ok ? 'creada' : await r.text()}`); }
    return out; }, [CUENTAS, INICIAL]));
  // Ruta de demostración del trabajo de campo (rotulada por el propio portal) y un tablero publicado.
  hecho.push('ruta de demostración: ' + await p.evaluate(async () => { const r = await fetch('/api/campo/demo', { method: 'POST' }); return r.ok ? 'cargada' : (await r.text()).slice(0, 80); }));
  const publicados = await p.evaluate(async () => (await fetch('/api/tableros').then(x => x.json()).catch(() => [])));
  if (!JSON.stringify(publicados).includes('"publico":true')) { await p.reload(); await p.waitForSelector('#lienzo path.leaflet-interactive');
    await p.click('nav > button[data-s=tableros]'); await p.waitForSelector('#tabejemplos button'); await p.click('#tabejemplos button'); await p.waitForSelector('#tabvista .pieza', { timeout: 20000 });
    await p.click('#tabguardar'); await p.waitForTimeout(900); await p.click('#tabpublicar'); await p.waitForTimeout(900); hecho.push('tablero de ejemplo: guardado y publicado'); } else hecho.push('tablero publicado: ya existía');
  await salir();

  // Cada institución entra con su cuenta y sube su archivo de ejemplo (ficticio). PNC sube además uno con nombres
  // de municipio dudosos, hecho con la misma plantilla, para que haya algo que decidir en «Revisar pendientes».
  for (const [usuario, , rol, institucion] of CUENTAS.filter(c => c[2] === 'institucion')) {
    const e = await entrar(usuario, INICIAL); if (/^NO/.test(e)) { hecho.push(`${usuario}: ${e}`); continue; }
    hecho.push(...await p.evaluate(async inst => { const out = [], propios = new Set((await fetch('/api/envios').then(x => x.json())).map(x => x.archivo));
      const subir = async (id, nombre, texto) => { if (propios.has(nombre)) return `${nombre}: ya estaba`; const f = new FormData(); f.append('archivo', new Blob([texto], { type: 'text/csv' }), nombre);
        const r = await fetch('/api/envios/' + encodeURIComponent(id), { method: 'POST', body: f }); const j = await r.json().catch(() => ({})); return `${nombre}: ${r.ok ? `recibido, ${j.porcentaje_resuelto} % de lugares resueltos, ${j.recepcion?.pendientes?.length ?? 0} por revisar` : JSON.stringify(j).slice(0, 100)}`; };
      for (const s of (await fetch('/api/solicitudes').then(x => x.json())).filter(s => s.institucion === inst)) {
        const texto = await fetch(`/api/solicitudes/${encodeURIComponent(s.id)}/plantilla.csv`).then(x => x.text()); out.push(await subir(s.id, `ejemplo_ficticio_${s.id}.csv`, texto));
        if (inst === 'PNC') { const L = texto.trim().split('\n'), im = L[0].split(',').indexOf('municipio'), base = L[1].split(',');
          if (im >= 0) out.push(await subir(s.id, 'ejemplo_ficticio_con_nombres_dudosos.csv', [L[0], ...L.slice(1), ...['Mixco zona 1', 'Amatitlan lago', 'Villa Nva', 'Sn Juan Sacatepequez'].map(m => { const f = [...base]; f[im] = m; return f.join(','); })].join('\n') + '\n')); } }
      return out; }, institucion));
    await salir();
  }
  // Las demás cuentas quedan con la contraseña de demostración ya puesta, para entrar sin pasos extra frente a la cámara.
  for (const [usuario, , rol] of CUENTAS.filter(c => c[2] !== 'institucion')) { hecho.push(`${usuario}: ${await entrar(usuario, INICIAL)}`); await salir(); }
  const estado = await p.evaluate(() => fetch('/api/estado').then(x => x.json())); await nav.close();
  console.log(hecho.map(x => '  ' + x).join('\n'));
  console.log(`\nListo: ${estado.envios} envíos de ejemplo, ${estado.pendientes} valores por revisar.`);
  console.log('Cuentas de demostración (todas con la contraseña indicada):\n  admin (administrador) · ' + CUENTAS.map(c => `${c[0]} (${c[2]}${c[3] ? ' de ' + c[3] : ''})`).join(' · '));
})().catch(e => { console.error(e); process.exit(1); });
