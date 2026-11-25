// FARO · idiomas de la interfaz.
//
// La pantalla está escrita en español y así se queda: este guion la traduce al vuelo con un
// diccionario (datos/estandar/idiomas/<código>.json) cuya clave es el propio texto en español.
// Regla de fondo: se traduce la interfaz, nunca los datos. Un texto solo cambia si está en el
// diccionario, y en el diccionario solo hay textos de la pantalla; los nombres de lugares,
// instituciones y productos, los valores de los catálogos, las cifras y las citas llegan de la
// base o de las fuentes y se muestran como los publica quien los produce.
//
// Cómo se forma la clave de un texto:
//   - Los números se cambian por «#» (así «10,034 registros» y «51 registros» son la misma frase)
//     y se devuelven tal cual a la traducción. «#2» pide un número por su posición.
//   - Las etiquetas dentro de una frase se cambian por marcas numeradas: «<1>texto</1>» si la
//     etiqueta lleva texto traducible y «<1/>» si es una pieza que no se toca (un campo, un
//     código, una cifra con identificador). La traducción reutiliza las mismas piezas.
//   - Una clave con «{0}», «{1}»… es un molde: ese tramo es un dato y pasa sin cambios.
//   - Una palabra con dos sentidos lleva contexto: el elemento se marca con data-ctx="confianza"
//     y su clave es «confianza|media» (nivel medio), distinta de «media» (el promedio).
(function () {
  'use strict';
  const ATRIBUTOS = ['title', 'placeholder', 'aria-label', 'alt'];
  const EN_LINEA = new Set(['B', 'STRONG', 'I', 'EM', 'A', 'SMALL', 'SPAN', 'U', 'SUP', 'SUB', 'ABBR', 'MARK', 'KBD']);
  const SIN_TOCAR = new Set(['SCRIPT', 'STYLE', 'TEXTAREA', 'CODE', 'PRE', 'NOSCRIPT', 'TEMPLATE', 'IFRAME', 'CANVAS']);
  const DE_BLOQUE = 'button,select,input,textarea,svg,img,table,div,p,ul,ol,canvas,label,code';
  const APARTADO = '[data-no-traducir],code,pre,textarea';
  const HTML = 'http://www.w3.org/1999/xhtml', SVG = 'http://www.w3.org/2000/svg';
  const LETRA = /\p{L}/u;
  const raiz = document.documentElement;

  let codigo = 'es', recoger = false;
  try {
    const pedido = new URLSearchParams(location.search).get('idioma');
    if (pedido && /^[a-z]{2,3}$/.test(pedido)) localStorage.setItem('faro_idioma', pedido);
    codigo = localStorage.getItem('faro_idioma') || 'es';
    recoger = localStorage.getItem('faro_idioma_recoger') === '1';
  } catch (_) {}
  if (!/^[a-z]{2,3}$/.test(codigo)) codigo = 'es';

  const exactos = new Map(), patrones = [], fallos = new Set(), recogidos = new Map();
  const hecho = new WeakMap(), hechoAtr = new WeakMap();
  let activo = false, diccionario = null;

  // ---- Claves ----
  const escapar = t => t.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/#/g, '&num;').replace(/ /g, '&nbsp;');
  const desescapar = t => t.replace(/&nbsp;/g, ' ').replace(/&num;/g, '#').replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&amp;/g, '&');
  const marcas = t => (t.match(/<\/?\d+\/?>/g) || []).sort().join();

  function normalizar(s) {
    const ini = /^[ \t\n\r]*/.exec(s)[0], resto = s.slice(ini.length), fin = /[ \t\n\r]*$/.exec(resto)[0];
    const numeros = [];
    const clave = resto.slice(0, resto.length - fin.length).replace(/[ \t\n\r]+/g, ' ')
      .replace(/<\/?\d+\/?>|&[a-z]+;|\d+(?:[.,]\d+)*/g, m => (m[0] === '<' || m[0] === '&') ? m : (numeros.push(m), '#'));
    return { clave, numeros, ini, fin };
  }
  const poner = (t, numeros) => { let i = 0; return t.replace(/#(\d+)?/g, (_, n) => numeros[n ? n - 1 : i++] ?? ''); };

  function cargar(dic) {
    diccionario = dic;
    for (const [k, v] of Object.entries(dic.textos || {})) {
      if (typeof v !== 'string' || !v.trim()) continue;
      if (!/\{\d\}/.test(k)) { exactos.set(k, v); continue; }
      // Molde: los tramos {n} son datos (texto sin etiquetas) y pasan sin cambios.
      const orden = [];
      const fuente = k.replace(/[.*+?^$()|[\]\\]/g, '\\$&').replace(/\{(\d)\}|#/g, (m, n) => { if (m === '#') return '#'; orden.push(+n); return '([^<]+?)'; });
      const ancla = k.split(/\{\d\}|#|<\/?\d+\/?>/).sort((a, b) => b.length - a.length)[0];
      try { patrones.push({ re: new RegExp('^' + fuente + '$', 'd'), orden, t: v, ancla: ancla.length >= 3 ? ancla : '' }); } catch (_) {}
    }
  }

  // Devuelve la traducción de una clave ya normalizada, con sus números, o null.
  // `sinMoldes`: dentro de tablas y listas de opciones solo valen las claves exactas; un molde podría
  // coincidir por casualidad con un dato (un nombre de producto) y cambiarlo.
  function traducirClave(clave, numeros, sinMoldes) {
    const t = exactos.get(clave);
    if (t !== undefined) return poner(t, numeros);
    if (sinMoldes || fallos.has(clave)) return null;
    for (const p of patrones) {
      if (p.ancla && !clave.includes(p.ancla)) continue;
      const m = p.re.exec(clave);
      if (!m) continue;
      // Qué números caen dentro de cada dato capturado y cuáles son de la frase.
      const antes = pos => { let n = 0; for (let i = 0; i < pos; i++) if (clave[i] === '#') n++; return n; };
      const datos = {}, usados = new Set();
      p.orden.forEach((cual, i) => {
        const [a, b] = m.indices[i + 1]; let n = antes(a);
        // Si lo capturado es a su vez un texto de la pantalla (el nombre de una sección), se traduce; un dato pasa igual.
        const trozo = clave.slice(a, b), suyo = exactos.get(trozo.trim());
        datos[cual] = (suyo === undefined || /<\/?\d+\/?>/.test(suyo) ? trozo : trozo.replace(trozo.trim(), () => suyo)).replace(/#/g, () => { usados.add(n); return numeros[n++] ?? ''; });
      });
      const propios = numeros.filter((_, i) => !usados.has(i));
      return poner(p.t, propios).replace(/\{(\d)\}/g, (_, n) => datos[n] ?? '');
    }
    if (fallos.size < 40000) fallos.add(clave);
    return null;
  }

  function anotar(clave, el, tipo, traducido) {
    if (!recoger || !clave || !LETRA.test(clave)) return;
    let r = recogidos.get(clave);
    if (!r) recogidos.set(clave, r = { n: 0, donde: [], cuerpo: 0, tipo, traducido: false });
    r.n++; if (traducido) r.traducido = true;
    if (el && el.closest) {
      const d = (el.closest('section,dialog,nav,header,[data-donde]') || {});
      const nombre = d.id || (d.dataset && d.dataset.donde) || (d.tagName || 'pagina').toLowerCase();
      if (r.donde.length < 4 && !r.donde.includes(nombre)) r.donde.push(nombre);
      if (el.closest('tbody,select,datalist')) r.cuerpo++;
    }
  }

  // Traduce un texto suelto (un aviso, un atributo). Devuelve null si no está en el diccionario.
  function texto(v, el, tipo) {
    const n = normalizar(escapar(String(v)));
    const t = traducirClave(n.clave, n.numeros);
    anotar(n.clave, el, tipo || 'texto', t != null);
    return t == null ? null : n.ini + desescapar(t.replace(/<\/?\d+\/?>/g, '')) + n.fin;
  }

  // ---- Documento ----
  const omitir = el => el.nodeType !== 1 || SIN_TOCAR.has(el.tagName) || (el.namespaceURI !== HTML && el.namespaceURI !== SVG) || el.hasAttribute('data-no-traducir') || el.isContentEditable;
  const esEnLinea = n => EN_LINEA.has(n.tagName) && !n.id && !n.hasAttribute('data-no-traducir') && !n.hasAttribute('data-ctx') && LETRA.test(n.textContent) && !n.querySelector(DE_BLOQUE);
  function esUnidad(el) {
    if (el.namespaceURI === SVG && !/^(text|tspan|title)$/.test(el.tagName)) return false;
    for (const n of el.childNodes) if (n.nodeType === 3 && LETRA.test(n.nodeValue)) return true;
    return false;
  }
  // Escribe el contenido de un elemento como clave; `piezas` recibe sus elementos hijos en orden.
  function escribir(el, piezas) {
    let s = '';
    for (const n of el.childNodes) {
      if (n.nodeType === 3) s += escapar(n.nodeValue);
      else if (n.nodeType === 1) {
        const i = piezas.push(n);
        if (esEnLinea(n)) { n.__enLinea = true; s += `<${i}>` + escribir(n, piezas) + `</${i}>`; } else { n.__enLinea = false; s += `<${i}/>`; }
      }
    }
    return s;
  }
  function colocar(el, hijos) {
    if (hijos.length === 1 && hijos[0].nodeType === 3 && el.childNodes.length === 1 && el.firstChild.nodeType === 3) { el.firstChild.nodeValue = hijos[0].nodeValue; return; }
    for (const v of [...el.childNodes]) if (v.nodeType === 3) v.remove();
    hijos.forEach((h, i) => { if (el.childNodes[i] !== h) el.insertBefore(h, el.childNodes[i] || null); });
  }
  // Reconstruye el contenido con la traducción, reutilizando los mismos elementos (no se pierden
  // sus identificadores ni sus manejadores).
  function armar(el, t, piezas) {
    const pila = [{ el, hijos: [] }], re = /<(\d+)\/>|<(\d+)>|<\/(\d+)>|[^<]+/g;
    let m;
    while ((m = re.exec(t))) {
      const cima = pila[pila.length - 1];
      if (m[1]) cima.hijos.push(piezas[m[1] - 1]);
      else if (m[2]) { const h = piezas[m[2] - 1]; cima.hijos.push(h); pila.push({ el: h, hijos: [] }); }
      else if (m[3]) { if (pila.length > 1) { const c = pila.pop(); colocar(c.el, c.hijos); } }
      else cima.hijos.push(document.createTextNode(desescapar(m[0])));
    }
    while (pila.length) { const c = pila.pop(); colocar(c.el, c.hijos); }
  }
  function atributos(el) {
    for (const a of ATRIBUTOS) {
      const v = el.getAttribute(a);
      if (!v || !LETRA.test(v)) continue;
      let m = hechoAtr.get(el);
      if (m && m[a] === v) continue;
      if (!m) hechoAtr.set(el, m = {});
      const t = texto(v, el, 'atributo');
      m[a] = t ?? v;
      if (t != null) el.setAttribute(a, t);
    }
  }
  function recorrer(el) {
    if (omitir(el)) return;
    atributos(el);
    if (!esUnidad(el)) { for (const h of el.children) recorrer(h); return; }
    const piezas = [], s = escribir(el, piezas);
    if (hecho.get(el) !== s) {
      const n = normalizar(s), ctx = el.getAttribute('data-ctx');
      if (ctx) n.clave = ctx + '|' + n.clave;
      const t = traducirClave(n.clave, n.numeros, !!el.closest('tbody,select,datalist'));
      anotar(n.clave, el, 'frase', t != null);
      if (t != null && marcas(t) === marcas(n.clave)) { armar(el, n.ini + t + n.fin, piezas); hecho.set(el, escribir(el, [])); }
      else hecho.set(el, s);
    }
    for (const p of piezas) { if (p.__enLinea) atributos(p); else recorrer(p); }
  }

  // Lo que la pantalla cambia después (tablas, avisos, fichas) se traduce al aparecer.
  const vigia = new MutationObserver(lista => {
    const raices = new Set();
    for (const m of lista) {
      if (m.type === 'attributes') { if (!m.target.closest(APARTADO) && !omitir(m.target)) atributos(m.target); continue; }
      let el = m.type === 'characterData' ? m.target.parentElement : m.target;
      if (!el || !el.isConnected) continue;
      while (el !== document.body && EN_LINEA.has(el.tagName) && el.parentElement) el = el.parentElement;
      if (!el.closest(APARTADO)) raices.add(el);
    }
    for (const el of raices) if (![...raices].some(o => o !== el && o.contains(el))) recorrer(el);
    vigia.takeRecords();
  });
  function activar() {
    if (activo) return;
    activo = true;
    const t = texto(document.title, null, 'titulo'); if (t != null) document.title = t;
    recorrer(document.body);
    vigia.observe(document.body, { subtree: true, childList: true, characterData: true, attributes: true, attributeFilter: ATRIBUTOS });
    for (const f of ['confirm', 'prompt', 'alert']) { const nativo = window[f]; window[f] = (m, ...r) => nativo.call(window, texto(m ?? '', null, 'dialogo') ?? m, ...r); }
  }

  // ---- Selector y aviso de borrador ----
  function elegir(nuevo) {
    try {
      localStorage.setItem('faro_idioma', nuevo);
      const abierta = document.querySelector('section.activa'); if (abierta) sessionStorage.setItem('faro_seccion', abierta.id);
    } catch (_) {}
    const u = new URL(location.href); u.searchParams.delete('idioma');
    if (u.href === location.href) location.reload(); else location.replace(u.href);
  }
  async function selector() {
    const boton = document.getElementById('idiomabtn'), lista = document.getElementById('idiomalista');
    if (!boton || !lista) return;
    const sigla = boton.querySelector('b'); if (sigla) sigla.textContent = codigo.toUpperCase();
    let r; try { r = await (await fetch('/api/idiomas')).json(); } catch (_) { return; }
    lista.textContent = '';
    for (const i of r.idiomas) {
      const b = document.createElement('button');
      b.type = 'button'; b.setAttribute('role', 'option'); b.setAttribute('aria-selected', i.codigo === codigo); b.lang = i.codigo;
      const nombre = document.createElement("span"); nombre.setAttribute("data-no-traducir", ""); nombre.textContent = i.nombre; b.append(nombre);
      const nota = document.createElement('small');
      nota.textContent = i.estado === 'origen' ? 'idioma de origen' : i.estado === 'borrador' ? `borrador · ${i.cobertura_pct ?? 0} % de la pantalla` : `${i.cobertura_pct ?? 0} % de la pantalla`;
      b.append(nota);
      b.onclick = () => { if (i.codigo !== codigo) elegir(i.codigo); else cerrar(); };
      lista.append(b);
    }
    const regla = document.createElement('p'); regla.textContent = 'Cambia el idioma de la pantalla. Los nombres oficiales y los datos se muestran como los publica su fuente.'; lista.append(regla);
    if (activo) recorrer(lista);
    const cerrar = () => { lista.hidden = true; boton.setAttribute('aria-expanded', 'false'); };
    boton.onclick = () => { lista.hidden = !lista.hidden; boton.setAttribute('aria-expanded', String(!lista.hidden)); if (!lista.hidden) (lista.querySelector('[aria-selected="true"]') || lista.firstChild).focus(); };
    document.addEventListener('click', e => { if (!lista.hidden && !e.target.closest('.idioma')) cerrar(); });
    document.addEventListener('keydown', e => { if (e.key === 'Escape' && !lista.hidden) { cerrar(); boton.focus(); } });
  }
  function avisoDeBorrador(dic) {
    if (!dic || dic.estado !== 'borrador' || document.getElementById('aviso-idioma')) return;
    const donde = document.querySelector('main .ancho') || document.querySelector('main') || document.body;
    const a = document.createElement('div');
    a.id = 'aviso-idioma'; a.setAttribute('data-no-traducir', ''); a.setAttribute('role', 'note');
    const n = document.createElement('b'); n.textContent = `${dic.nombre} · borrador. `;
    a.append(n, document.createTextNode(dic.aviso || 'Traducción en borrador, pendiente de revisión. Lo que aún no está traducido aparece en español.'));
    donde.prepend(a);
  }

  // ---- Arranque ----
  const estilo = document.createElement('style');
  estilo.textContent = 'html.idioma-espera body{visibility:hidden}';
  document.head.appendChild(estilo);
  if (codigo !== 'es') { raiz.classList.add('idioma-espera'); setTimeout(() => raiz.classList.remove('idioma-espera'), 3000); }
  const pedirDic = codigo === 'es' ? Promise.resolve(null) : fetch('/api/idiomas/' + codigo).then(r => r.ok ? r.json() : null).catch(() => null);
  const pagina = new Promise(r => document.readyState === 'loading' ? document.addEventListener('DOMContentLoaded', r, { once: true }) : r());
  const listo = Promise.all([pedirDic, pagina]).then(([dic]) => {
    if (dic) { cargar(dic); if (dic.estado !== 'borrador') raiz.lang = codigo; avisoDeBorrador(dic); activar(); }
    else { if (codigo !== 'es') { codigo = 'es'; try { localStorage.removeItem('faro_idioma'); } catch (_) {} } if (recoger) activar(); }
    raiz.classList.remove('idioma-espera');
    selector();
    return codigo;
  });

  // `tr(texto)` queda disponible para el resto de la página; en español devuelve lo mismo.
  window.tr = t => (activo ? texto(t, null, 'texto') : null) ?? t;
  window.FARO_IDIOMA = {
    get codigo() { return codigo; }, get diccionario() { return diccionario; }, listo, elegir, recogidos,
    // Para la herramienta que arma la plantilla: recoge las claves de un trozo de HTML.
    recogerHtml(html, donde) { const d = document.createElement('div'); d.dataset.donde = donde || 'guion'; d.innerHTML = html; const antes = recoger; recoger = true; recorrer(d); recoger = antes; },
    recogerTexto(t, donde) { const antes = recoger; recoger = true; const d = document.createElement('div'); d.dataset.donde = donde || 'guion'; texto(t, d, 'texto'); recoger = antes; },
  };
})();
