// Idiomas de la interfaz: español (origen), inglés y kaqchikel (borrador con fuente).
// Comprueba que la pantalla se traduce y que los datos NO: nombres de lugares, cifras y citas
// salen iguales en los tres idiomas.
// Uso: node idiomas.js <url> [carpeta de capturas]
const { chromium } = require('playwright');
const { entrar } = require('./_sesion.js');
const url = process.argv[2] || 'http://localhost:8082', dir = process.argv[3];
let fallos = 0;
const ok = (c, t) => { console.log((c ? '  bien  ' : '  FALLA ') + t); if (!c) fallos++; };
(async () => {
  const nav = await chromium.launch();
  const ctx = await nav.newContext({ viewport: { width: 1366, height: 850 } });
  await ctx.addInitScript(() => { try { localStorage.setItem('faro_recorrido_visto', '1'); } catch (_) {} });
  const p = await ctx.newPage();
  const errores = []; p.on('pageerror', e => errores.push(e.message));
  await p.goto(url); await p.waitForSelector('#lienzo path.leaflet-interactive');
  console.log('  sesión: ' + await entrar(p));
  const pedir = ruta => p.evaluate(async r => { const x = await fetch(r); return { estado: x.status, cuerpo: await x.json().catch(() => null) }; }, ruta);
  const menu = s => p.textContent(`nav > button[data-s="${s}"]`).then(t => t.trim());
  const ir = async s => { await p.click(`nav > button[data-s="${s}"]`); await p.waitForTimeout(1300); };
  // Lo que es dato en una sección: primera columna de una tabla y las cifras de las tarjetas.
  const datos = async () => {
    await ir('vial'); await p.waitForSelector('#tvialdep tbody tr');
    const deptos = await p.$$eval('#tvialdep tbody tr td:first-child', c => c.map(x => x.textContent.trim()));
    await ir('recibir');
    const cifras = await p.$$eval('#tarjetas b', c => c.map(x => x.textContent.trim()));
    await ir('acerca'); await p.waitForSelector('#citas *');
    const citas = (await p.textContent('#citas')).replace(/\s+/g, ' ').trim();
    return { deptos, cifras, citas };
  };
  const cambiar = async codigo => {
    await p.click('#idiomabtn'); await p.waitForSelector('#idiomalista:not([hidden]) button');
    await Promise.all([p.waitForNavigation(), p.click(`#idiomalista button[lang="${codigo}"]`)]);
    await p.waitForFunction(() => window.FARO_IDIOMA && !document.documentElement.classList.contains('idioma-espera')); await p.waitForTimeout(900);
  };

  console.log('El servicio de idiomas');
  const lista = (await pedir('/api/idiomas')).cuerpo;
  const de = c => lista.idiomas.find(i => i.codigo === c) || {};
  ok(de('es').estado === 'origen' && de('en').estado === 'completo' && de('cak').estado === 'borrador', 'español es el origen, inglés está completo y kaqchikel es borrador');
  ok(de('en').cobertura_pct >= 99, `inglés cubre ${de('en').cobertura_pct} % de los ${lista.textos_de_la_pantalla} textos de la pantalla`);
  ok(de('cak').cobertura_pct > 0 && de('cak').cobertura_pct < 50, `kaqchikel declara su cobertura real (${de('cak').cobertura_pct} %)`);
  ok(lista.idiomas.map(i => i.codigo).join() === 'es,en,cak', 'orden: origen, completos, borradores');
  ok((await pedir('/api/idiomas/zz')).estado === 400 && (await pedir('/api/idiomas/EN')).estado === 400 && (await pedir('/api/idiomas/..%2Fmejoras')).estado === 400, 'un código que no existe o no es un código se rechaza');
  const en = (await pedir('/api/idiomas/en')).cuerpo;
  ok(Object.values(en.textos).every(v => v.trim()), 'el diccionario se sirve sin entradas vacías');

  console.log('En español (por omisión)');
  ok(await menu('mapa') === 'Mapa' && await p.getAttribute('html', 'lang') === 'es', 'el menú está en español');
  ok((await p.textContent('#idiomabtn')).trim() === 'ES', 'el selector dice ES');
  ok(!(await p.$('#aviso-idioma')), 'sin aviso de borrador');
  const es = await datos();
  ok(es.deptos.length >= 22 && es.citas.length > 500 && es.cifras.length >= 4, `datos de referencia leídos (${es.deptos.length} departamentos, citas de ${es.citas.length} caracteres)`);

  console.log('En inglés');
  await ir('vial');
  await cambiar('en');
  ok(await p.getAttribute('html', 'lang') === 'en', 'la página se declara en inglés');
  ok(await p.evaluate(() => document.querySelector('section.activa').id) === 'vial', 'al cambiar de idioma se vuelve a la sección en la que se estaba');
  ok(await menu('mapa') === 'Map' && await menu('recibir') === 'Receive submission' && await menu('vial') === 'Road network', 'el menú está en inglés');
  ok(/May every piece of data reach port/.test(await p.textContent('header')), 'el lema del encabezado está en inglés');
  ok((await p.textContent('#idiomabtn')).trim() === 'EN', 'el selector dice EN');
  ok(await p.getAttribute('#recorrido', 'aria-label') === 'Guided tour' && /Sign out/.test(await p.textContent('#salir')), 'atributos y botones traducidos');
  const ing = await datos();
  ok(JSON.stringify(ing.deptos) === JSON.stringify(es.deptos), 'los nombres de los departamentos NO cambian');
  ok(JSON.stringify(ing.cifras) === JSON.stringify(es.cifras), 'las cifras NO cambian');
  ok(ing.citas === es.citas, 'las citas bibliográficas NO cambian');
  await ir('vial');
  const titulos = await p.$$eval('#tvialdep thead th', c => c.map(x => x.textContent.trim()));
  ok(titulos[0] === 'Department' && !titulos.some(t => /^(Lugares|Kilómetros|Vías)/.test(t)), `los encabezados de la tabla sí cambian (${titulos.slice(0, 3).join(' · ')})`);
  await ir('usuarios'); await p.waitForSelector('#tusuarios tbody tr');
  ok(await p.$eval('#tusuarios thead', t => /Role/.test(t.textContent)) && await p.$eval('#tusuarios tbody select[data-rol]', s => /Administrator/.test(s.textContent)), 'los roles se muestran en inglés');
  await ir('mapa'); await p.click('#recorrido'); await p.waitForSelector('#recglobo');
  const globo = await p.textContent('#recglobo');
  ok(/Next/.test(globo) && !/ (el|los|las|una|para|que) /.test(globo), 'el recorrido guiado está en inglés');
  await p.keyboard.press('Escape');
  // Lo que la pantalla pinta después también se traduce: el aviso flotante.
  await p.evaluate(() => aviso('Guardado.')); await p.waitForTimeout(150);
  ok((await p.textContent('#aviso')).trim() === 'Saved.', 'un aviso que aparece después también se traduce');
  ok(await p.evaluate(() => tr('Cancelar') === 'Cancel' && tr('Santa Catarina Pinula') === 'Santa Catarina Pinula'), 'tr() traduce un rótulo y deja igual un nombre');
  // Cuánto de lo que se ve fuera de las tablas quedó en español.
  const resto = await p.evaluate(() => { const es = / (de|del|la|las|los|el|que|para|con|por|una|sus?) /; let total = 0, sin = 0; const m = []; for (const s of document.querySelectorAll('section h2, section h3, section .nota, section label, section button, nav button, header')) { const t = s.textContent.replace(/\s+/g, ' ').trim(); if (!t) continue; total++; if (es.test(' ' + t.toLowerCase() + ' ')) { sin++; if (m.length < 6) m.push(t.slice(0, 70)); } } return { total, sin, m }; });
  ok(resto.sin / resto.total < 0.03, `títulos, notas y botones aún en español: ${resto.sin} de ${resto.total}${resto.sin ? ' (' + resto.m.join(' | ') + ')' : ''}`);
  if (dir) await p.screenshot({ path: `${dir}/idiomas-ingles.png` });

  console.log('En teléfono, en inglés');
  await p.setViewportSize({ width: 360, height: 760 }); await p.waitForTimeout(400);
  ok(await p.evaluate(() => document.querySelector('header').scrollWidth <= innerWidth + 1 && document.documentElement.scrollWidth <= innerWidth + 1), 'el encabezado cabe en 360 px');
  await p.click('#idiomabtn');
  ok(await p.evaluate(() => { const r = document.querySelector('#idiomalista').getBoundingClientRect(); return r.left >= 0 && r.right <= innerWidth && r.height > 100; }), 'la lista de idiomas cabe en el teléfono');
  await p.keyboard.press('Escape');
  ok(await p.evaluate(() => document.querySelector('#idiomalista').hidden), 'Esc cierra la lista');
  await p.setViewportSize({ width: 1366, height: 850 });

  console.log('En kaqchikel (borrador)');
  await cambiar('cak');
  ok(await p.isVisible('#aviso-idioma') && /borrador/.test(await p.textContent('#aviso-idioma')), 'avisa que es un borrador pendiente de revisión por hablantes');
  ok(await p.getAttribute('html', 'lang') === 'es', 'la página no se declara en kaqchikel mientras casi todo siga en español');
  ok(/Titz'apïx molojri'ïl/.test(await p.textContent('#salir')), 'los rótulos con fuente salen en kaqchikel («Cerrar sesión»)');
  ok(await menu('mapa') === 'Mapa', 'lo que no tiene fuente sigue en español');
  const cak = await datos();
  ok(JSON.stringify(cak.deptos) === JSON.stringify(es.deptos) && cak.citas === es.citas && JSON.stringify(cak.cifras) === JSON.stringify(es.cifras), 'los datos tampoco cambian en kaqchikel');
  if (dir) await p.screenshot({ path: `${dir}/idiomas-kaqchikel.png` });

  console.log('De vuelta al español, y por enlace');
  await cambiar('es');
  ok(await menu('mapa') === 'Mapa' && !(await p.$('#aviso-idioma')), 'vuelve al español');
  await p.goto(url + '/?idioma=en'); await p.waitForFunction(() => window.FARO_IDIOMA && !document.documentElement.classList.contains('idioma-espera')); await p.waitForTimeout(700);
  ok(await menu('mapa') === 'Map', 'un enlace con ?idioma=en abre el portal en inglés');
  await p.goto(url + '/reporte?municipio=0101'); await p.waitForTimeout(2500);
  ok(/Municipal report|municipal report/.test(await p.textContent('body')), 'el reporte imprimible también sale en inglés');
  await p.evaluate(() => localStorage.setItem('faro_idioma', 'es'));

  ok(errores.length === 0, 'sin errores de JavaScript' + (errores.length ? ': ' + errores.slice(0, 3).join(' | ') : ''));
  await nav.close();
  console.log(fallos ? `\n${fallos} comprobación(es) fallaron` : '\nTodo bien');
  process.exit(fallos ? 1 : 0);
})().catch(e => { console.error(e); process.exit(1); });
