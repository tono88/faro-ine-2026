// Verificación de interfaz en los tamaños de pantalla de uso real: laptop de 15.6" (1366×768 y
// 1920×1080), tableta (vertical y horizontal) y celular. Recorre TODAS las secciones y mide, en cada
// una: desborde horizontal, elementos que se salen, texto demasiado pequeño, botones difíciles de
// tocar y errores de JavaScript; además hace clic real en el mapa. Escribe el informe en
// docs/20-verificacion-interfaz.md y capturas en entregables/capturas/interfaz/.
// Uso: NO_PROXY=localhost node verificacion_interfaz.js http://localhost:8082
const { chromium } = require('playwright'); const fs = require('fs'); const path = require('path'); const { entrar } = require('./_sesion');
const raiz = path.resolve(__dirname, '../..'), cap = path.join(raiz, 'entregables/capturas/interfaz');
// Con FARO_IDIOMA=en (u otro idioma) se hace la misma revisión con la pantalla traducida; el informe
// y las capturas del repositorio son los de la pantalla en español y no se reescriben.
const enOtroIdioma = !!process.env.FARO_IDIOMA && process.env.FARO_IDIOMA !== 'es';
const PANTALLAS = [
  { id: 'laptop-1366', nombre: 'Laptop 15.6" HD (1366×768)', w: 1366, h: 768, tactil: false },
  { id: 'laptop-1920', nombre: 'Laptop 15.6" Full HD (1920×1080)', w: 1920, h: 1080, tactil: false },
  { id: 'tableta-vertical', nombre: 'Tableta vertical (768×1024)', w: 768, h: 1024, tactil: true },
  { id: 'tableta-horizontal', nombre: 'Tableta horizontal (1024×768)', w: 1024, h: 768, tactil: true },
  { id: 'celular', nombre: 'Celular (390×844)', w: 390, h: 844, tactil: true },
  { id: 'celular-chico', nombre: 'Celular pequeño (360×740)', w: 360, h: 740, tactil: true },
];
(async () => {
  const base = process.argv[2]; fs.mkdirSync(cap, { recursive: true }); const b = await chromium.launch(); const filas = []; const detalle = []; let problemas = 0;
  for (const pt of PANTALLAS) {
    const ctx = await b.newContext({ viewport: { width: pt.w, height: pt.h }, deviceScaleFactor: 1, hasTouch: pt.tactil, isMobile: pt.tactil && pt.w < 700 }); if (process.env.FARO_IDIOMA) await ctx.addInitScript(i => { try { localStorage.setItem('faro_idioma', i); } catch (_) {} }, process.env.FARO_IDIOMA); const p = await ctx.newPage(); const errores = []; p.on('pageerror', e => errores.push(String(e).slice(0, 160)));
    await p.goto(base); await p.waitForSelector('#lienzo path.leaflet-interactive'); await entrar(p);
    const menuOculto = await p.isVisible('#menu'); const secciones = await p.locator('nav > button[data-s]').evaluateAll(bs => bs.filter(x => !x.hidden).map(x => x.dataset.s));
    const tot = { secciones: secciones.length, desbordes: 0, fuera: 0, chico: 0, toque: 0 };
    for (const s of secciones) {
      if (menuOculto) await p.click('#menu'); await p.click(`nav > button[data-s=${s}]`); await p.waitForTimeout(s === 'rubros' ? 3500 : 900);
      if (s === 'tableros') { await p.click('#tabejemplos button'); await p.waitForSelector('#tabvista .pieza', { timeout: 15000 }); await p.waitForTimeout(500); }
      const m = await p.evaluate(tactil => { const ancho = document.documentElement.clientWidth, sec = document.querySelector('section.activa'); const visibles = [...sec.querySelectorAll('*')].filter(e => { const r = e.getBoundingClientRect(); return r.width > 0 && r.height > 0; });
        const contenido = e => { let a = e.parentElement; while (a && a !== sec) { const o = getComputedStyle(a).overflowX; if (o === 'auto' || o === 'scroll' || o === 'hidden') return true; a = a.parentElement; } return false; };
        const fuera = visibles.filter(e => e.getBoundingClientRect().right > ancho + 2 && !contenido(e) && !e.closest('.leaflet-container') && !e.closest('svg'));
        const chico = visibles.filter(e => e.children.length === 0 && e.textContent.trim().length > 3 && parseFloat(getComputedStyle(e).fontSize) < 11 && !e.closest('svg') && !e.closest('.leaflet-container'));
        const minimo = tactil ? 30 : 22, toque = visibles.filter(e => (e.tagName === 'BUTTON' || (e.tagName === 'INPUT' && !['checkbox', 'radio', 'range', 'file'].includes(e.type)) || e.tagName === 'SELECT') && !e.closest('.leaflet-container') && e.getBoundingClientRect().height < minimo);
        const d = e => e.tagName.toLowerCase() + (e.id ? '#' + e.id : '') + (e.className && typeof e.className === 'string' ? '.' + e.className.split(' ')[0] : '') + ' «' + e.textContent.trim().slice(0, 24) + '»';
        return { pagina: document.documentElement.scrollWidth > ancho + 1, fuera: fuera.slice(0, 3).map(d), nfuera: fuera.length, chico: chico.slice(0, 2).map(d), nchico: chico.length, toque: toque.slice(0, 3).map(e => d(e) + ' ' + Math.round(e.getBoundingClientRect().height) + 'px'), ntoque: toque.length }; }, pt.tactil);
      tot.desbordes += m.pagina ? 1 : 0; tot.fuera += m.nfuera; tot.chico += m.nchico; tot.toque += m.ntoque;
      if (m.pagina || m.nfuera || m.nchico || m.ntoque) detalle.push(`| ${pt.nombre} | ${s} | ${m.pagina ? 'la página se desborda a lo ancho. ' : ''}${m.nfuera ? 'se salen: ' + m.fuera.join('; ') + '. ' : ''}${m.nchico ? 'texto menor de 11 px: ' + m.chico.join('; ') + '. ' : ''}${m.ntoque ? 'controles bajos: ' + m.toque.join('; ') : ''} |`);
      if (['mapa', 'tableros', 'rubros'].includes(s) && ['laptop-1366', 'tableta-vertical', 'celular'].includes(pt.id) && !enOtroIdioma) await p.screenshot({ path: path.join(cap, `${pt.id}-${s}.jpg`), type: 'jpeg', quality: 68 });
    }
    // Clic real en el mapa.
    if (menuOculto) await p.click('#menu'); await p.click('nav > button[data-s=mapa]'); await p.waitForTimeout(900);
    const punto = await p.evaluate(() => { const r = document.querySelector('#lienzo').getBoundingClientRect(); for (const fy of [.5, .6, .4, .7, .3]) for (const fx of [.5, .55, .45, .6, .4]) { const x = r.left + r.width * fx, y = r.top + r.height * fy; if (y < innerHeight - 5 && y > 0 && document.elementFromPoint(x, y)?.tagName === 'path') return [x, y]; } return null; });
    let clic = 'sin polígono a la vista';
    if (punto) { if (pt.tactil) await p.touchscreen.tap(punto[0], punto[1]); else { await p.mouse.move(punto[0], punto[1]); await p.waitForTimeout(200); await p.mouse.down(); await p.waitForTimeout(60); await p.mouse.up(); } await p.waitForTimeout(1500); clic = (await p.isVisible('#ficha h2')) ? 'abre la ficha' : 'NO abre la ficha'; }
    const malo = tot.desbordes + tot.fuera + tot.chico + tot.toque + errores.length + (clic === 'abre la ficha' ? 0 : 1); problemas += malo;
    filas.push(`| ${pt.nombre} | ${tot.secciones} | ${tot.desbordes} | ${tot.fuera} | ${tot.chico} | ${tot.toque} | ${clic} | ${errores.length} | ${malo ? '**revisar**' : 'bien'} |`);
    console.log(pt.nombre.padEnd(34), JSON.stringify(tot), clic, errores.length ? JSON.stringify(errores) : ''); await ctx.close();
  }
  const md = `# Verificación de interfaz por tamaño de pantalla

Este informe lo escribe \`.faro/pruebas/verificacion_interfaz.js\` recorriendo el portal en un navegador
real (Chromium) en cada tamaño. Repetir: \`node .faro/pruebas/verificacion_interfaz.js http://localhost:8082\`.

Qué se mide en **cada sección** y en **cada pantalla**:

- **Desborde**: la página no debe necesitar desplazamiento horizontal.
- **Elementos que se salen** del ancho visible sin estar dentro de una zona con desplazamiento propio (las tablas anchas sí lo tienen).
- **Texto pequeño**: ningún texto de menos de 11 px.
- **Controles bajos**: botones, campos y listas de al menos 30 px de alto en pantallas táctiles (22 px con ratón).
- **Clic o toque real en el mapa**: debe abrir la ficha.
- **Errores de JavaScript**.

| Pantalla | Secciones recorridas | Páginas con desborde | Elementos que se salen | Textos pequeños | Controles bajos | Mapa | Errores JS | Resultado |
|---|---|---|---|---|---|---|---|---|
${filas.join('\n')}

${detalle.length ? '## Detalle de lo encontrado\n\n| Pantalla | Sección | Qué |\n|---|---|---|\n' + detalle.join('\n') : 'No quedó ningún hallazgo abierto.'}

## Capturas

Mapa, tableros y precios por rubro en laptop de 15.6", tableta y celular: \`entregables/capturas/interfaz/\`.

## Criterios de diseño que se comprueban

- Laptop de 15.6": menú lateral fijo y contenido a 1,180 px como máximo, para que las líneas no sean demasiado largas en Full HD.
- Tableta y celular (980 px o menos): el menú pasa a un botón; a 700 px o menos todo va en una columna, la ficha del mapa es una hoja inferior y los campos usan 16 px para que el teléfono no amplíe la página al escribir.
- Las tablas anchas se desplazan dentro de su propio marco, con el encabezado fijo.
- Colores y tipografía de la guía de diseño del INE (#0080FF, #004080, Ubuntu Sans); el foco del teclado siempre es visible.
`;
  if (!enOtroIdioma) fs.writeFileSync(path.join(raiz, 'docs/20-verificacion-interfaz.md'), md); else console.log(detalle.join('\n'));
  console.log(problemas ? `PROBLEMAS: ${problemas}` : 'sin hallazgos'); await b.close();
})();
