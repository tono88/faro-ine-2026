// Arma la plantilla de textos de la pantalla (datos/estandar/idiomas/plantilla.json): la lista de
// todo lo que hay que traducir para llevar la interfaz a otro idioma.
//
//   node scripts/idiomas/extraer.js <url> <usuario administrador> <contraseña> [carpeta de informe]
//
// Junta dos cosechas, hechas con el mismo guion que traduce (web/idiomas.js), de modo que la clave
// de cada texto es exactamente la que el portal buscará en el diccionario:
//   1. En marcha: abre cada sección con y sin sesión y anota lo que aparece en pantalla.
//   2. Del código: las cadenas escritas en las tres páginas (avisos, diálogos, estados que no se
//      alcanzan en un recorrido) y los pasos de los recorridos guiados.
// Un texto visto en marcha solo entra en la plantilla si está escrito en el código de la pantalla
// o en los textos propios del servidor. Lo demás es dato (nombres de lugares, instituciones,
// productos, valores de catálogo) y no se traduce: queda fuera, listado en el informe.
const fs = require('fs'), path = require('path');
const { chromium } = require('playwright');
const [url, usuario, clave, informe] = process.argv.slice(2);
if (!url || !usuario || !clave) { console.error('uso: node scripts/idiomas/extraer.js <url> <usuario> <contraseña> [carpeta de informe]'); process.exit(2); }
const RAIZ = path.join(__dirname, '..', '..');
const WEB = path.join(RAIZ, 'crates', 'faro-servidor', 'web');
const PAGINAS = ['index.html', 'reporte.html', 'informe_precio.html'];
const HUECO = '';

// ---- Cadenas escritas en el código de una página ----
// Recorre el JavaScript y devuelve sus cadenas; en las plantillas, cada ${...} queda como HUECO
// (o como «x» si está dentro de una etiqueta) y las cadenas de dentro salen aparte.
function cadenasDe(js) {
  const salida = [];
  const previo = i => { let j = i - 1; while (j >= 0 && /\s/.test(js[j])) j--; return j < 0 ? '' : js[j]; };
  function plantilla(i) { // js[i] es el acento grave de apertura
    let s = '', enEtiqueta = false; i++;
    while (i < js.length && js[i] !== '`') {
      if (js[i] === '\\') { s += js[i + 1] === 'n' ? ' ' : js[i + 1]; i += 2; continue; }
      if (js[i] === '$' && js[i + 1] === '{') { i = expresion(i + 2, '}'); s += enEtiqueta ? 'x' : HUECO; continue; }
      if (js[i] === '<' && /[a-zA-Z/]/.test(js[i + 1] || '')) enEtiqueta = true; else if (js[i] === '>') enEtiqueta = false;
      s += js[i++];
    }
    salida.push(s);
    return i + 1;
  }
  function simple(i) {
    const q = js[i]; let s = ''; i++;
    while (i < js.length && js[i] !== q && js[i] !== '\n') { if (js[i] === '\\') { s += js[i + 1] === 'n' ? ' ' : js[i + 1]; i += 2; } else s += js[i++]; }
    salida.push(s);
    return i + 1;
  }
  function expresion(i, cierre) { // avanza hasta el cierre que corresponde; devuelve la posición siguiente
    let hondo = 0;
    while (i < js.length) {
      const c = js[i];
      if (c === '`') { i = plantilla(i); continue; }
      if (c === '"' || c === "'") { i = simple(i); continue; }
      if (c === '/' && js[i + 1] === '/') { while (i < js.length && js[i] !== '\n') i++; continue; }
      if (c === '/' && js[i + 1] === '*') { i = js.indexOf('*/', i + 2) + 2; continue; }
      if (c === '/' && /[(,=:[!&|?{};>]/.test(previo(i) || ';')) { // expresión regular
        let enClase = false; i++;
        while (i < js.length && (js[i] !== '/' || enClase) && js[i] !== '\n') { if (js[i] === '\\') i++; else if (js[i] === '[') enClase = true; else if (js[i] === ']') enClase = false; i++; }
        i++; continue;
      }
      if (c === '{' || c === '(' || c === '[') hondo++;
      else if (c === '}' || c === ')' || c === ']') { if (hondo === 0 && c === cierre) return i + 1; hondo--; }
      i++;
    }
    return i;
  }
  expresion(0, null);
  return salida;
}
const pareceTexto = s => {
  const t = s.replace(new RegExp(HUECO, 'g'), ' ').replace(/<[^>]*>/g, ' ').replace(/&[a-z]+;/g, ' ').trim();
  if (!/\p{L}{3}/u.test(t)) return false;
  if (/^(https?:|\/|#|\.|\[|data:|application\/|text\/|[a-z-]+:[^ ])/.test(s.trim()) && !/</.test(s)) return false; // rutas, selectores, estilos
  if (/^[a-z0-9_.:#\-\[\]="'>+~*, ()]+$/.test(s) && !/ [a-záéíóúñ]{3,} /.test(' ' + s + ' ')) return false;        // identificadores y selectores
  if (/^(Arrow(Up|Down|Left|Right)|Backspace|Delete|Enter|Escape|Tab|DOMContentLoaded|FeatureCollection|(Multi)?(Polygon|Point|LineString)|NF[CD]|use strict|[A-Z]{2,10})$/.test(t)) return false; // teclas, etiquetas, formatos
  if (/^[a-z0-9 _.:#\-\[\]="'>+~*,()]+$/.test(s) && !/ (de|del|la|el|los|las|en|con|por|para|sin|que|un|una|se|no|al|su|ya|es|hay) /.test(' ' + s + ' ')) return false;  // selectores y clases
  return /\p{L}{2,} +\p{L}{2,}/u.test(t) || /^[A-ZÁÉÍÓÚÑ¿¡«]/.test(t);                                               // frase, o palabra con mayúscula
};

(async () => {
  const fuentes = {};
  for (const p of PAGINAS) fuentes[p] = fs.readFileSync(path.join(WEB, p), 'utf8');
  fuentes['idiomas.js'] = '<script>' + fs.readFileSync(path.join(WEB, 'idiomas.js'), 'utf8') + '</script>';
  // Nombres que son dato aunque el código los mencione: departamentos, municipios e instituciones.
  const nombres = new Set();
  for (const a of ['departamentos.csv', 'municipios.csv']) for (const l of fs.readFileSync(path.join(RAIZ, 'datos', 'catalogo', a), 'utf8').split('\n').slice(1)) { const n = l.split(',')[1]; if (n) nombres.add(n.trim()); }
  for (const a of fs.readdirSync(path.join(RAIZ, 'datos', 'solicitudes'))) { const m = /^institucion\s*=\s*"([^"]+)"/m.exec(fs.readFileSync(path.join(RAIZ, 'datos', 'solicitudes', a), 'utf8')); if (m) nombres.add(m[1]); }
  // Textos propios del servidor que la pantalla muestra: mensajes y rótulos del código y el comparativo.
  let servidor = fs.readFileSync(path.join(RAIZ, 'datos', 'estandar', 'mejoras.json'), 'utf8');
  const rs = d => fs.readdirSync(d, { withFileTypes: true }).flatMap(a => a.isDirectory() ? rs(path.join(d, a.name)) : a.name.endsWith('.rs') ? [path.join(d, a.name)] : []);
  for (const a of rs(path.join(RAIZ, 'crates'))) servidor += '\n' + fs.readFileSync(a, 'utf8');
  const llano = t => t.replace(/&nbsp;/g, ' ').replace(/&middot;/g, '·').replace(/&amp;/g, '&').replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&quot;/g, '"').replace(/&num;/g, '#').replace(/\\(['"`])/g, '$1').replace(/\\n/g, ' ').replace(/[ \t\r\n]+/g, ' ');
  const pantalla = llano(Object.values(fuentes).join('\n')), propio = llano(servidor);

  const nav = await chromium.launch();
  const ctx = await nav.newContext({ viewport: { width: 1360, height: 900 } });
  await ctx.addInitScript(() => { try { localStorage.setItem('faro_idioma_recoger', '1'); localStorage.setItem('faro_idioma', 'es'); localStorage.setItem('faro_recorrido_visto', '1'); } catch (_) {} });
  const p = await ctx.newPage();
  const cosecha = new Map();
  const sumar = (lista, origen) => {
    for (const [k, v] of lista) { const c = cosecha.get(k) || { n: 0, donde: [], cuerpo: 0, origen: new Set(), tipo: v.tipo }; c.n += v.n; c.cuerpo += v.cuerpo; for (const d of v.donde) if (!c.donde.includes(d) && c.donde.length < 5) c.donde.push(d); c.origen.add(origen); cosecha.set(k, c); }
  };
  const guardar = async origen => sumar(await p.evaluate(() => { const r = [...window.FARO_IDIOMA.recogidos.entries()]; window.FARO_IDIOMA.recogidos.clear(); return r; }), origen);
  // Cosecha de textos del código: se limpia, se recoge y se lee en un solo paso, para que nada que la
  // pantalla siga pintando (una tabla que termina de cargar) se cuele como si fuera texto del código.
  const deUnaVez = async (origen, fn, arg) => sumar(await p.evaluate(([f, a]) => { const F = window.FARO_IDIOMA; F.recogidos.clear(); (new Function('F', 'a', f))(F, a); const r = [...F.recogidos.entries()]; F.recogidos.clear(); return r; }, [fn, arg]), origen);
  const secciones = async () => {
    for (const s of await p.$$eval('nav > button[data-s]', bs => bs.filter(b => !b.hidden).map(b => b.dataset.s))) {
      await p.click(`nav > button[data-s="${s}"]`).catch(() => {});
      await p.waitForTimeout(1500);
      await p.evaluate(async () => { const m = document.querySelector('main'); for (let y = 0; y < m.scrollHeight; y += 700) { m.scrollTop = y; await new Promise(r => setTimeout(r, 60)); } m.scrollTop = 0; });
    }
  };
  const abrir = async () => { await p.goto(url); await p.waitForSelector('#lienzo path.leaflet-interactive', { timeout: 30000 }).catch(() => {}); await p.waitForTimeout(800); };

  // 1. En marcha, sin sesión.
  await abrir();
  await secciones();
  await p.click('nav > button[data-s="mapa"]'); await p.waitForTimeout(500);
  await p.click('#entrar').catch(() => {}); await p.waitForTimeout(300);
  await guardar('marcha');
  // 2. En marcha, con sesión de administrador.
  const entro = await p.evaluate(async ([u, c]) => (await fetch('/api/sesion', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ usuario: u, clave: c }) })).ok, [usuario, clave]);
  if (!entro) { console.error('No se pudo iniciar sesión con ese usuario.'); process.exit(1); }
  await abrir();
  await secciones();
  // Mapa: ficha de un municipio, niveles y detalle.
  await p.click('nav > button[data-s="mapa"]'); await p.waitForTimeout(700);
  await p.evaluate(() => { const c = document.querySelector('#lienzo path.leaflet-interactive'); if (c) { const r = c.getBoundingClientRect(); c.dispatchEvent(new MouseEvent('click', { bubbles: true, clientX: r.x + r.width / 2, clientY: r.y + r.height / 2 })); } });
  await p.waitForTimeout(1200);
  for (const sel of ['#fichamas', '.ficha button', '#ficha button']) { const b = await p.$(sel); if (b && await b.isVisible().catch(() => false)) { await b.click().catch(() => {}); await p.waitForTimeout(1500); break; } }
  await guardar('marcha');
  await p.evaluate(() => document.querySelectorAll('dialog[open]').forEach(d => d.close()));
  // Rubros: el primero del menú.
  const rubro = await p.$('#menurubros button'); if (rubro) { await rubro.click().catch(() => {}); await p.waitForTimeout(2500); }
  // Un recorrido en marcha (para sus botones) y todos los pasos de todos los recorridos.
  await p.evaluate(() => { try { recorrer('portal'); } catch (_) {} }); await p.waitForTimeout(500);
  await guardar('marcha');
  await p.evaluate(() => { try { cerrarRecorrido(); } catch (_) {} });
  await deUnaVez('recorrido', `for (const [cual, pasos] of Object.entries(RECORRIDOS)) for (const x of pasos) { F.recogerHtml(x.t || '', 'recorrido-' + cual); F.recogerHtml(x.x || '', 'recorrido-' + cual); }`);
  // Las páginas imprimibles.
  for (const ruta of ['/reporte?municipio=0101', '/informe-precio?q=cemento']) { await p.goto(url + ruta).catch(() => {}); await p.waitForTimeout(2500); await guardar('marcha'); }

  // 3. Del código: cada cadena pasa por el mismo guion (como HTML si trae etiquetas).
  await abrir();
  let leidas = 0;
  for (const [nombre, html] of Object.entries(fuentes)) {
    const guiones = [...html.matchAll(/<script>([\s\S]*?)<\/script>/g)].map(m => m[1]).join('\n');
    const cadenas = [...new Set(cadenasDe(guiones))].filter(pareceTexto);
    leidas += cadenas.length;
    await deUnaVez('codigo', `for (const c of a[0]) { if (/<[a-zA-Z/]/.test(c)) F.recogerHtml(c, a[1]); else F.recogerTexto(c, a[1]); }`, [cadenas, 'codigo-' + nombre]);
  }
  await nav.close();

  // ---- Clasificación ----
  const letras = k => (k.replace(/\{\d\}|<\/?\d+\/?>|&[a-z]+;|#/g, '').match(/\p{L}/gu) || []).length;
  const tramos = k => k.split(new RegExp(`${HUECO}|<\\/?\\d+\\/?>|#|\\{\\d\\}`)).map(t => llano(t).trim().replace(/^[.,;:·() ]+|[.,;:·() ]+$/g, '')).filter(t => /\p{L}{2}/u.test(t));
  const dentro = (k, donde) => { const t = tramos(k); return t.length > 0 && t.every(x => donde.includes(x)); };
  const plantilla = new Map(), datos = [];
  const soloNombres = k => { const t = tramos(k).flatMap(x => x.split(/, | · /)).map(x => x.trim()).filter(Boolean); return t.length > 0 && t.every(x => nombres.has(x)); };
  const deCodigo = c => c.origen.has('codigo') || c.origen.has('recorrido');
  for (const [k0, c] of cosecha) {
    let k = k0, molde = false;
    if (/button\[|minmax\(|^\s*x\s*$/.test(k)) continue;                                   // selectores y estilos
    if (/^[a-z]+\|/.test(k)) { plantilla.set(k, { ...c, clase: 'pantalla' }); continue; }      // con contexto: lo marcó el código (data-ctx)
    if (!k.includes(HUECO) && soloNombres(k)) { datos.push([k, c]); continue; }               // un lugar o una institución
    // Una sola palabra vista solo dentro de tablas o listas es casi siempre un valor: se deja como está.
    if (!deCodigo(c) && !/[ <]/.test(k) && c.cuerpo >= c.n) { datos.push([k, c]); continue; }
    if (k.includes(HUECO)) { let i = 0; k = k.replace(new RegExp(HUECO, 'g'), () => `{${i++}}`); molde = true; if (i > 9) continue; }
    if (molde) {
      // Un molde solo sirve si lo fijo basta para reconocerlo: con poco texto podría coincidir con un dato.
      const fijo = letras(k), puro = /^\{\d\}/.test(k) && /\{\d\}$/.test(k);
      if (fijo < 6 || (puro && fijo < 12) || /\{\d\} ?\{\d\}/.test(k)) continue; // dos datos seguidos: son trozos de frase, no un molde
      plantilla.set(k, { ...c, clase: 'molde' });
    } else if (deCodigo(c) || dentro(k, pantalla)) plantilla.set(k, { ...c, clase: 'pantalla' });
    else if (dentro(k, propio)) plantilla.set(k, { ...c, clase: 'servidor' });
    else datos.push([k, c]);
  }
  // Valores con contexto que el código marca (data-ctx) aunque no todos se vean en un recorrido.
  for (const [ctx, valores] of Object.entries({ confianza: ['alta', 'media', 'baja'] })) for (const x of valores) if (!plantilla.has(`${ctx}|${x}`)) plantilla.set(`${ctx}|${x}`, { n: 0, cuerpo: 0, donde: [], origen: new Set(['codigo']), clase: 'pantalla' });
  // Textos que la cosecha no alcanza: los que llevan un dato en medio de la frase o solo salen en estados poco comunes.
  const extra = path.join(RAIZ, 'datos', 'estandar', 'idiomas', 'plantilla_extra.json');
  if (fs.existsSync(extra)) for (const k of JSON.parse(fs.readFileSync(extra, 'utf8')).textos) if (!plantilla.has(k)) plantilla.set(k, { n: 0, cuerpo: 0, donde: [], origen: new Set(['extra']), clase: /\{\d\}/.test(k) ? 'molde' : 'pantalla' });
  const claves = [...plantilla.keys()].sort((a, b) => a.localeCompare(b, 'es'));
  const cuenta = clase => [...plantilla.values()].filter(v => v.clase === clase).length;
  fs.writeFileSync(path.join(RAIZ, 'datos', 'estandar', 'idiomas', 'plantilla.json'), JSON.stringify({
    que_es: 'Textos de la pantalla de FARO en español, tal como los busca el traductor (web/idiomas.js). Es la lista de lo que hay que traducir para sumar un idioma. La genera scripts/idiomas/extraer.js; no se edita a mano.',
    claves: { '#': 'un número (se devuelve tal cual; #2 pide el segundo)', '<1>…</1>': 'una etiqueta con texto (negrita, enlace): se conserva alrededor de su traducción', '<1/>': 'una pieza que no se traduce (campo, código, cifra)', '{0}': 'un dato (nombre de lugar, de institución, de archivo): pasa sin cambios', '&nbsp; &lt; &amp; &num;': 'espacio duro, <, & y #' },
    textos: claves,
  }, null, 1) + '\n');
  console.log(`cadenas leídas del código: ${leidas} · en la plantilla: ${claves.length} (pantalla ${cuenta('pantalla')}, servidor ${cuenta('servidor')}, moldes ${cuenta('molde')}) · datos que no se traducen: ${datos.length}`);
  if (informe) {
    fs.mkdirSync(informe, { recursive: true });
    const fila = ([k, c]) => ({ clave: k, clase: c.clase, n: c.n, cuerpo: c.cuerpo, donde: c.donde, origen: [...c.origen] });
    fs.writeFileSync(path.join(informe, 'plantilla_detalle.json'), JSON.stringify([...plantilla].map(fila), null, 1));
    fs.writeFileSync(path.join(informe, 'datos_no_traducidos.json'), JSON.stringify(datos.sort((a, b) => b[1].n - a[1].n).map(fila), null, 1));
  }
})().catch(e => { console.error(e); process.exit(1); });
