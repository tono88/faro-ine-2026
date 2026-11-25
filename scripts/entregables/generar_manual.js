// Genera el manual de usuario gráfico (entregables/FARO-manual-de-usuario.pdf) a partir del portal en marcha.
// Cada página de sección lleva una captura tomada en el momento, con marcas numeradas sobre los mismos
// elementos que señala el recorrido guiado del portal (la lista RECORRIDOS de index.html): si el recorrido
// cambia, el manual cambia con él. Requiere Node y Playwright con Chromium.
//
// Uso: node scripts/entregables/generar_manual.js <url> <usuario administrador> <contraseña> [usuario de institución] [contraseña]
// Las capturas muestran lo que haya en esa base: para un manual sin datos reales, use una base nueva y suba
// los archivos de ejemplo (ficticios) que ofrece cada solicitud.
const { chromium } = require('playwright'); const fs = require('fs'); const path = require('path');
const [base = 'http://localhost:8082', usuario = 'admin', clave = process.env.FARO_CLAVE || '', uInst = '', cInst = ''] = process.argv.slice(2);
const raiz = path.resolve(__dirname, '../..'), salida = path.join(raiz, 'entregables', 'FARO-manual-de-usuario.pdf');
const esc = t => String(t ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const leer = f => fs.readFileSync(path.join(raiz, f), 'utf8');
const AN = 1360, AL = 1040; // tamaño de la ventana de las capturas

// Bloques del menú, con el color que los distingue en el manual (paleta de la identidad).
const BLOQUES = {
  panorama: ['Panorama', '#0080ff'], registros: ['Registros administrativos', '#004080'], territorio: ['Territorio', '#3399ff'],
  precios: ['Precios', '#c77700'], datos: ['Datos', '#4a6b8a'], cuenta: ['Cuenta', '#313131'],
};
const ROLES = { todos: ['Cualquier visitante', '#1a7f4b'], institucion: ['Institución', '#c77700'], analista: ['Analista', '#0080ff'], consulta: ['Consulta', '#4a6b8a'], operador: ['Operador de campo', '#7a4bb3'], admin: ['Administrador', '#004080'], cuenta: ['Quien tenga cuenta', '#4a6b8a'] };
// Sección, bloque, quién la usa, para qué sirve y qué la hace valiosa. Las cifras entre llaves se leen del portal al generar.
const SECCIONES = [
  ['mapa', 'panorama', ['todos'], 'Ver el país por departamento, municipio o lugar poblado, y abrir la ficha de cualquier territorio.', 'Un solo mapa reúne población, censo, vías, huella construida y registros recibidos de {municipios} municipios y casi 20 mil lugares poblados. Cada color tiene su fuente y su año.'],
  ['mejoras', 'panorama', ['todos'], 'Comparar, punto por punto, cómo se trabajaba antes y qué cambia con FARO.', '{mejoras} comparaciones con una cifra medida cada una: nada de promesas, solo lo que se puede comprobar.'],
  ['proceso', 'panorama', ['todos'], 'Entender el camino completo de un dato: de la solicitud a la estadística.', 'Cuatro flujos dibujados que dicen qué hace cada etapa y quién. Lo que el portal hace no es una caja negra.'],
  ['tableros', 'panorama', ['todos', 'analista', 'admin'], 'Armar un tablero escribiendo qué se quiere ver, filtrarlo y publicarlo con un enlace.', 'De una frase a un tablero publicado sin programar: {indicadores} indicadores, mapas, tablas y gráficas. Lo publicado lo abre cualquiera, sin cuenta.'],
  ['recibir', 'registros', ['institucion', 'analista', 'admin'], 'Subir el archivo de una institución y recibirlo validado, con códigos oficiales y sin datos personales.', 'El archivo llega como sale del sistema de la institución: FARO reconoce las columnas aunque cambien de nombre u orden. El original nunca se guarda.'],
  ['pendientes', 'registros', ['analista', 'admin'], 'Decidir a qué corresponde cada texto que el sistema no pudo resolver con certeza.', 'FARO no adivina lugares. Lo dudoso lo propone, una persona decide una vez y la decisión sirve para todos los envíos que vengan.'],
  ['memoria', 'registros', ['analista', 'admin'], 'Cargar la tabla de códigos de una institución y consultar las equivalencias confirmadas.', 'La memoria queda en el sistema, no en la cabeza de una persona: cada equivalencia dice quién la confirmó y cuándo.'],
  ['resultados', 'registros', ['todos'], 'Ver cuánto ha llegado de cada institución y qué cobertura tiene cada departamento.', 'Solo se publican conteos. Las celdas con menos de {umbral} casos se protegen para que nadie pueda ser identificado.'],
  ['crecimiento', 'territorio', ['todos'], 'Consultar la población proyectada por municipio y el crecimiento que se ve desde el satélite.', 'Proyecciones oficiales del INE 2015–2030, y un botón que repite todo el cálculo desde la fuente. Lo que va más allá se rotula como no oficial.'],
  ['vial', 'territorio', ['todos'], 'Conocer la red vial por tipo y superficie, y qué lugares poblados no tienen camino mapeado.', 'La red se trata como un grafo: por eso FARO puede señalar enlaces que probablemente faltan en el mapa, listos para verificar en campo.'],
  ['operadores', 'territorio', ['analista', 'admin'], 'Asignar operadores a puestos y armar la ruta de cada uno con un asistente de cinco pasos.', 'La asignación minimiza la distancia total y la ruta sale ordenada. Hay una demostración rotulada para aprender sin tocar datos reales.'],
  ['campo', 'territorio', ['operador'], 'Seguir la ruta del día desde el teléfono y registrar cada visita.', 'Cada visita se comprueba con ubicación y foto de la fachada. La foto se analiza y se descarta: solo queda el resultado.'],
  ['alimentaria', 'territorio', ['todos'], 'Ver la inseguridad alimentaria por departamento y la desnutrición crónica por municipio.', 'Dos fuentes oficiales de SESAN, juntas y con su nivel territorial dicho con claridad: una es por departamento, la otra por municipio.'],
  ['construccion', 'precios', ['todos', 'analista'], 'Consultar precios de materiales de construcción por artículo oficial.', 'Los precios que publican los proveedores se llevan a los artículos del índice con reglas escritas y revisables; lo dudoso lo decide una persona.'],
  ['precios', 'precios', ['todos', 'analista'], 'Calcular un precio de referencia a partir de cotizaciones y buscar los ya emitidos por el INE.', 'El método está declarado y los valores atípicos se marcan, no se esconden: el resultado se puede defender.'],
  ['canasta', 'precios', ['todos'], 'Comparar cada producto de la canasta básica: precio oficial frente a lo que publican las tiendas.', 'El precio del INE y el precio publicado hoy, lado a lado, con la tienda y la fecha de cada observación.'],
  ['rubros', 'precios', ['todos', 'analista'], 'Buscar entre los precios publicados en línea, por rubro, tienda o producto.', '{precios} precios leídos de tiendas en línea, cada uno con su dirección y su fecha. El mismo código de barras permite comparar tiendas sin suponer.'],
  ['balance', 'precios', ['todos', 'analista'], 'Calcular los derivados de un producto primario para la Hoja de Balance de Alimentos.', 'Usa las tasas de extracción de la FAO para Guatemala y las contrasta con la hoja de balance del INE. Si el INE tiene tasas propias, se cargan.'],
  ['muestreos', 'precios', ['todos', 'analista'], 'Ver y ejecutar las lecturas automáticas de precios en catálogos públicos.', 'Todo es código determinista: sin modelos externos, sin saltarse bloqueos, con la fecha y la dirección de cada lectura.'],
  ['ine', 'datos', ['todos', 'analista'], 'Vigilar los conjuntos del portal de datos abiertos del INE.', 'El INE es la fuente principal: FARO avisa cuando un conjunto cambia y deja constancia.'],
  ['capas', 'datos', ['todos'], 'Descargar todo lo que FARO produce, listo para un sistema de información geográfica.', '{capas} capas en GeoJSON o CSV, con fuente y licencia. Lo que se ve en el portal también se puede llevar.'],
  ['acerca', 'datos', ['todos'], 'Conocer qué es FARO, qué se hizo y de dónde sale cada dato.', '{fuentes} fuentes citadas con institución, fecha de descarga y licencia. Ninguna cifra queda sin origen.'],
  ['perfil', 'cuenta', ['cuenta'], 'Cambiar su nombre y su contraseña.', 'Las contraseñas se guardan con Argon2 y una contraseña inicial bloquea los cambios hasta que se sustituye.'],
  ['usuarios', 'cuenta', ['admin'], 'Crear cuentas, asignar roles, grupos e instituciones.', 'Cada quien ve solo lo suyo: roles, grupos por sección y cuentas atadas a una institución. Toda acción queda en la bitácora con nombre.'],
];
const TITULO_EXTRA = { usuarios: 'Usuarios', balance: 'Hoja de balance: derivados' };

const icono = (lado, fondo = '#0080ff') => `<svg viewBox="0 0 64 64" width="${lado}" height="${lado}" aria-hidden="true"><rect width="64" height="64" rx="14" fill="${fondo}"/><path d="M26 21 5 12v17z" fill="#ffd500" opacity=".9"/><path d="M38 21 59 12v17z" fill="#ffd500" opacity=".9"/><path d="M32 8l8 8H24z" fill="#fff"/><rect x="26" y="16" width="12" height="8" rx="1" fill="#ffe566"/><path d="M27 26h10l4 28H23z" fill="#fff"/><path d="M26.1 34h11.8l.9 6H25.2z" fill="#66b3ff"/><path d="M24.4 46h15.2l.6 4H23.8z" fill="#66b3ff"/><rect x="18" y="54" width="28" height="4" rx="2" fill="#fff"/></svg>`;
const chip = r => `<span class="chip" style="--c:${ROLES[r][1]}">${ROLES[r][0]}</span>`;

(async () => {
  if (!clave) { console.error('Falta la contraseña del administrador (tercer argumento o FARO_CLAVE).'); process.exit(2); }
  const proxy = process.env.HTTPS_PROXY; const nav = await chromium.launch(proxy ? { proxy: { server: proxy, bypass: 'localhost,127.0.0.1' } } : {});
  const ctx = await nav.newContext({ viewport: { width: AN, height: AL }, deviceScaleFactor: 1.5 }); const p = await ctx.newPage();
  p.on('dialog', d => d.accept());
  const abrir = async () => { await p.goto(base); await p.waitForSelector('#lienzo path.leaflet-interactive'); await p.waitForTimeout(700); };
  const entrar = async (u, c) => { const r = await p.evaluate(async ([u, c]) => (await fetch('/api/sesion', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ usuario: u, clave: c }) })).status, [u, c]); if (r !== 200) throw new Error(`no se pudo entrar como ${u} (${r})`); await abrir(); };
  const salir = async () => { await p.evaluate(() => fetch('/api/sesion', { method: 'DELETE' })); await abrir(); };
  const foto = async (opc = {}) => 'data:image/jpeg;base64,' + (await p.screenshot({ type: 'jpeg', quality: 84, ...opc })).toString('base64');
  // Los pasos del recorrido de una sección, con la caja de cada elemento tal como está en la pantalla.
  const pasosDe = cual => p.evaluate(c => { scrollTo(0, 0); return (RECORRIDOS[c] || []).map(q => { let el = q.en ? [...document.querySelectorAll(q.en)].find(aLaVista) : null; if (el && q.sube) el = el.closest(q.sube) || el; const k = el && el.getBoundingClientRect(); return { t: q.t, x: q.x, general: !q.en, caja: k ? [k.left, k.top, k.width, k.height] : null }; }); }, cual);

  await abrir();
  // Portada interior: la pantalla sin sesión, con las marcas del recorrido general.
  const general = { pasos: await pasosDe('portal'), img: await foto() };
  await p.click('#recorrido'); await p.click('#recglobo [data-r=sig]'); await p.waitForTimeout(400); const conGlobo = await foto(); await p.keyboard.press('Escape');
  // La misma pantalla en inglés, con la lista de idiomas abierta; después se vuelve al español.
  await p.evaluate(() => localStorage.setItem('faro_idioma', 'en')); await p.reload(); await p.waitForFunction(() => window.FARO_IDIOMA && !document.documentElement.classList.contains('idioma-espera')); await p.waitForTimeout(1800);
  await p.click('#idiomabtn'); await p.waitForTimeout(400); const conIdiomas = await foto();
  await p.evaluate(() => localStorage.setItem('faro_idioma', 'es')); await p.reload(); await p.waitForFunction(() => window.FARO_IDIOMA && !document.documentElement.classList.contains('idioma-espera')); await p.waitForTimeout(1800);
  // Teléfono.
  const tel = await (await nav.newContext({ viewport: { width: 390, height: 780 }, deviceScaleFactor: 2 })).newPage(); await tel.goto(base); await tel.waitForSelector('#lienzo path.leaflet-interactive'); await tel.waitForTimeout(700);
  const fotoTel = async () => 'data:image/jpeg;base64,' + (await tel.screenshot({ type: 'jpeg', quality: 84 })).toString('base64');
  const telMapa = await fotoTel(); await tel.click('#menu'); await tel.waitForTimeout(300); const telMenu = await fotoTel(); await tel.click('#menu'); await tel.click('#recorrido'); await tel.waitForTimeout(300); const telRec = await fotoTel(); await tel.context().close();

  // Cuenta de institución (si se dio una): su pantalla de envío.
  let inst = null;
  if (uInst) { await entrar(uInst, cInst); await p.click('nav > button[data-s=recibir]'); await p.waitForTimeout(900);
    inst = { img: await foto(), pasos: await pasosDe('recibir'), quien: await p.evaluate(() => yo.institucion), menu: await p.evaluate(() => [...document.querySelectorAll('nav > button[data-s]')].filter(b => !b.hidden).length) }; await salir(); }

  await entrar(usuario, clave);
  const datos = await p.evaluate(async () => { const j = u => fetch(u).then(x => x.json()).catch(() => null); const e = await j('/api/estado'), c = await j('/capas'), pu = await j('/api/publicados');
    return { nombres: NOMBRE_SECCION, umbral: e?.umbral_de_celda ?? 3, lugares: e?.lugares_en_catalogo, capas: (c?.capas || c || []).length, precios: (pu?.rubros || []).reduce((a, r) => a + (r.productos || 0), 0), tiendas: new Set((pu?.rubros || []).flatMap(r => String(r.tiendas || '').split(',').map(t => t.trim()).filter(Boolean))).size,
      municipios: 340 }; });
  datos.fuentes = JSON.parse(leer('datos/estandar/fuentes.json')).fuentes.length; datos.mejoras = JSON.parse(leer('datos/estandar/mejoras.json')).puntos.length;
  if (!datos.indicadores) datos.indicadores = ((leer('crates/faro-servidor/src/tablero.rs').split('pub const INDICADORES')[1] || '').split('];')[0].match(/^\s*\("/gm) || []).length || 'decenas de';
  if (!datos.tiendas) datos.tiendas = 'varias'; const num = n => typeof n === 'number' ? n.toLocaleString('es-GT') : n;
  const conCifras = t => t.replace(/\{(\w+)\}/g, (_, k) => num(datos[k]));

  const secciones = [];
  for (const [id, bloque, roles, para, virtud] of SECCIONES) {
    await p.click(`nav > button[data-s="${id}"]`); await p.waitForTimeout(id === 'rubros' || id === 'mapa' ? 2500 : 1400);
    if (id === 'rubros') await p.waitForSelector('#trubros tbody tr', { timeout: 30000 }).catch(() => {});
    // «Mi ruta de campo» se muestra con la ruta de demostración del propio portal (rotulada), que se quita después.
    let demo = false; if (id === 'campo' && await p.isVisible('#campodemo').catch(() => false)) { await p.click('#campodemo'); await p.waitForTimeout(1800); demo = true; }
    const pasos = (await pasosDe(id)); await p.waitForTimeout(150);
    secciones.push({ id, bloque, roles, para, virtud: conCifras(virtud), titulo: TITULO_EXTRA[id] || datos.nombres[id] || id, pasos, img: await foto() });
    if (demo && await p.isVisible('#campodemofuera').catch(() => false)) { await p.click('#campodemofuera'); await p.waitForTimeout(800); }
    process.stdout.write(id + ' ');
  }
  // La ficha de un municipio.
  await p.click('nav > button[data-s=mapa]'); await p.fill('#buscalugar', 'panajachel'); await p.waitForSelector('#buscalista li[data-i]'); await p.keyboard.press('Enter'); await p.waitForSelector('#ficha svg'); await p.waitForTimeout(900);
  const ficha = await foto(); await p.click('#ficha .cerrar').catch(() => {});
  console.log('\ncapturas listas');

  // ---------- Composición ----------
  // Captura con marcas: un recuadro ámbar y un número sobre cada elemento que cae dentro de la ventana.
  const marcada = (img, pasos) => { let n = 0; const usados = [];
    const marcas = pasos.map(q => { if (q.general) return ''; n++; q.n = n; const c = q.caja; if (!c || c[1] > AL - 40 || c[1] + c[3] < 20 || c[0] > AN - 20) { q.fuera = true; return ''; }
      const x0 = Math.max(4, c[0]), y0 = Math.max(4, c[1]), x1 = Math.min(AN - 4, c[0] + c[2]), y1 = Math.min(AL - 4, c[1] + c[3]);
      let mx = x0 - 2, my = y0 - 2; while (usados.some(u => Math.abs(u[0] - mx) < 40 && Math.abs(u[1] - my) < 40)) mx += 44; usados.push([mx, my]);
      const pc = (v, t) => (v / t * 100).toFixed(2) + '%';
      return `<i class="caja" style="left:${pc(x0, AN)};top:${pc(y0, AL)};width:${pc(x1 - x0, AN)};height:${pc(y1 - y0, AL)}"></i><b class="marca" style="left:${pc(mx, AN)};top:${pc(my, AL)}">${n}</b>`; }).join('');
    return `<div class="captura"><img src="${img}">${marcas}</div>`; };
  const leyenda = pasos => `<ol class="pasos">${pasos.filter(q => !q.general).map(q => `<li><b class="num">${q.n}</b><div><h4>${esc(q.t)}${q.fuera ? ' <em>más abajo o al avanzar</em>' : ''}</h4><p>${esc(q.x)}</p></div></li>`).join('')}</ol>`;
  const paginas = []; const pag = (clase, cuerpo, color) => paginas.push({ clase, cuerpo, color });
  const version = (leer('docs/08-versiones.md').match(/## (v[\d.]+)/) || [])[1] || '';
  const identidad = leer('docs/25-identidad.md'); const parrafo = t => ((identidad.split(`## ${t}`)[1] || '').split('\n## ')[0] || '').trim().replace(/\s+/g, ' ');
  const haz = `<svg class="haz" viewBox="0 0 1000 770" preserveAspectRatio="none" aria-hidden="true"><defs><linearGradient id="h1" x1="1" x2="0"><stop offset="0" stop-color="#ffd500" stop-opacity=".55"/><stop offset="1" stop-color="#ffd500" stop-opacity="0"/></linearGradient></defs><path d="M760 250 L0 20 L0 330 Z" fill="url(#h1)"/><path d="M760 250 L0 380 L0 560 Z" fill="url(#h1)" opacity=".45"/><g fill="none" stroke="#66b3ff" stroke-width="2" opacity=".35"><path d="M0 690 q60 -22 120 0 t120 0 t120 0 t120 0 t120 0 t120 0 t120 0 t120 0 t120 0"/><path d="M-40 722 q60 -22 120 0 t120 0 t120 0 t120 0 t120 0 t120 0 t120 0 t120 0 t120 0" opacity=".7"/><path d="M0 752 q60 -22 120 0 t120 0 t120 0 t120 0 t120 0 t120 0 t120 0 t120 0 t120 0" opacity=".45"/></g></svg>`;

  // 1. Portada
  pag('portada', `${haz}<div class="torre">${icono(330, '#0080ff')}</div><div class="texto"><p class="sobre">Manual de usuario</p><h1>FARO</h1><p class="lema">Que cada dato llegue a puerto.</p><p class="desc">Registros administrativos, territorio y precios para el Instituto Nacional de Estadística de Guatemala.</p><p class="pie-portada">${esc(version)} · Datatón INE 2026 · Equipo La Sociedad de Almas</p></div>`);
  // 2. Qué es
  const cifra = (n, t) => `<div class="cifra"><b>${esc(num(n))}</b><span>${t}</span></div>`;
  pag('que', `<div class="col-a"><p class="ante">Qué es FARO</p><h2>Un faro no mueve los barcos.<br>Les da una referencia fija para llegar.</h2><p class="grande">FARO hace lo mismo con los datos que cada institución ya tiene: el INE los <b>pide</b> con una solicitud clara, los <b>recibe</b> tal como salen de cada sistema, los <b>homologa</b> con los códigos oficiales sin adivinar, y los <b>aprovecha</b> junto al censo, el territorio y los precios.</p><div class="mv"><div><h3>Misión</h3><p>${esc(parrafo('Misión'))}</p></div><div><h3>Visión</h3><p>${esc(parrafo('Visión'))}</p></div></div><div class="verbos"><span>Pide</span><i>→</i><span>Recibe</span><i>→</i><span>Homologa</span><i>→</i><span>Aprovecha</span></div></div>
    <div class="col-b"><p class="ante claro">FARO en cifras</p><div class="cifras">${cifra(24, 'secciones, cada una con su recorrido guiado')}${cifra(datos.lugares, 'lugares en el catálogo oficial')}${cifra(datos.fuentes, 'fuentes citadas con fecha y licencia')}${cifra(datos.capas, 'capas listas para descargar')}${cifra(datos.precios, 'precios publicados leídos')}${cifra(0, 'datos personales guardados')}</div><p class="nota-b">Las cifras se leyeron del portal el día en que se generó este manual.</p></div>`);
  // 3. Elija su camino
  const camino = (r, quien, puede, empieza) => `<div class="camino" style="--c:${ROLES[r][1]}"><span class="inicial">${ROLES[r][0][0]}</span><h3>${ROLES[r][0]}</h3><p class="quien">${quien}</p><p>${puede}</p><p class="empieza"><b>Empiece en</b> ${empieza}</p></div>`;
  pag('elegir', `<header class="cab"><p class="ante">Antes de empezar</p><h2>Elija su camino</h2><p class="bajada">El portal muestra a cada persona lo que le corresponde. Busque su caso y vaya directo a sus páginas.</p></header><div class="caminos">
    ${camino('todos', 'No necesita cuenta.', 'Ve el mapa, los tableros publicados, las cifras agregadas, los precios y las fuentes. Puede descargar las capas.', 'Mapa · Tableros · Capas publicadas')}
    ${camino('institucion', 'Enlace de PNC, RENAP, SE-CONRED u otra institución.', 'Ve todo lo público y, al entrar, sube <b>solo los archivos de su institución</b>. No ve lo que enviaron las demás.', 'Recibir envío')}
    ${camino('analista', 'Personal técnico del INE.', 'Recibe envíos, decide pendientes, carga tablas, maneja precios y muestreos, arma tableros y coordina rutas.', 'Recibir envío · Revisar pendientes')}
    ${camino('consulta', 'Personal del INE que solo necesita ver.', 'Ve lo recibido, los pendientes y las equivalencias, sin poder cambiar nada.', 'Resultados y avance')}
    ${camino('operador', 'Quien sale a campo.', 'Ve su ruta en el teléfono y registra cada visita con ubicación y foto.', 'Mi ruta de campo')}
    ${camino('admin', 'Responsable del portal.', 'Todo lo anterior, más crear cuentas, grupos y revisar la bitácora.', 'Usuarios')}</div>
    <p class="franja">Primer ingreso de una instalación nueva: usuario <code>admin</code>, contraseña <code>admin</code>. El portal pide cambiarla y no permite hacer cambios hasta que se haga.</p>`);
  // 4. La pantalla
  pag('seccion', `<header class="cab"><p class="ante">Primeros pasos</p><h2>La pantalla en treinta segundos</h2><p class="bajada">Todo el portal tiene la misma forma: menú a la izquierda, contenido al centro, y arriba el botón de ayuda y su cuenta.</p></header><div class="cuerpo">${marcada(general.img, general.pasos)}<div class="lado">${leyenda(general.pasos)}<div class="virtud"><h4>Por qué importa</h4><p>No hay que aprender veinticuatro pantallas distintas: aprendida una, se saben todas.</p></div></div></div>`, '#0080ff');
  // 5. Recorridos
  pag('seccion', `<header class="cab"><p class="ante">Primeros pasos</p><h2>Recorridos guiados: el portal se explica solo</h2><p class="bajada">En cualquier pantalla, el botón ámbar <span class="boton-rec">Recorrido guiado</span> señala cada parte y dice para qué sirve. Este manual usa los mismos pasos.</p></header><div class="cuerpo"><div class="captura"><img src="${conGlobo}"></div><div class="lado"><ol class="pasos sin">
    <li><b class="num">?</b><div><h4>Arriba, siempre</h4><p>«Recorrido guiado» explica la pantalla que tiene abierta.</p></div></li>
    <li><b class="num">?</b><div><h4>En cada sección</h4><p>«Ver cómo funciona esta pantalla», junto al título, hace lo mismo.</p></div></li>
    <li><b class="num">→</b><div><h4>Avance a su ritmo</h4><p>Siguiente y Anterior, o las flechas del teclado. Esc lo cierra.</p></div></li>
    <li><b class="num">★</b><div><h4>¿Primera vez?</h4><p>Desde el primer paso se abre el recorrido general del portal.</p></div></li></ol><div class="virtud"><h4>Por qué importa</h4><p>Nadie necesita capacitación previa para empezar: la ayuda está donde se necesita, en el momento en que se necesita.</p></div></div></div>`, '#ffd500');
  // 5b. Idioma de la pantalla
  pag('seccion', `<header class="cab"><p class="ante">Primeros pasos</p><h2>Idioma de la pantalla: español, inglés y kaqchikel</h2><p class="bajada">El botón del globo, arriba, cambia el idioma de la pantalla en cualquier sección y sin iniciar sesión. Se traduce la interfaz, nunca los datos.</p></header><div class="cuerpo"><div class="captura"><img src="${conIdiomas}"></div><div class="lado"><ol class="pasos sin">
    <li><b class="num">1</b><div><h4>Qué cambia</h4><p>El menú, los títulos, los botones, las explicaciones y los recorridos guiados.</p></div></li>
    <li><b class="num">2</b><div><h4>Qué no cambia</h4><p>Los nombres de lugares, instituciones y productos, las cifras y las citas: se muestran como los publica su fuente. Lo que se descarga es igual en cualquier idioma.</p></div></li>
    <li><b class="num">3</b><div><h4>Kaqchikel: borrador</h4><p>Más de un tercio de los rótulos, armados con vocabulario escrito por hablantes. Falta la revisión de una persona experta: la pantalla lo avisa, y lo demás aparece en español.</p></div></li>
    <li><b class="num">★</b><div><h4>Un idioma es un archivo</h4><p>Corregir una frase o sumar otro idioma nacional no pide programar.</p></div></li></ol><div class="virtud"><h4>Por qué importa</h4><p>Un dato no llega a puerto si quien lo necesita no puede leer la pantalla. Y traducir un dato sería alterarlo: por eso solo se traduce la pantalla.</p></div></div></div>`, '#66b3ff');
  // 6. Índice visual (se llena al final, cuando se conocen las páginas)
  const indice = paginas.length; pag('indice', '');
  // Secciones
  const paginaDe = {};
  secciones.forEach((s, i) => { const [nomBloque, color] = BLOQUES[s.bloque];
    paginaDe[s.id] = paginas.length + 1;
    pag('seccion', `<header class="cab"><p class="ante" style="color:${color}">${esc(nomBloque)}</p><h2><span class="nsec" style="background:${color}">${String(i + 1).padStart(2, '0')}</span>${esc(s.titulo)}</h2><p class="bajada">${esc(s.para)}</p><div class="chips">${s.roles.map(chip).join('')}</div></header>
      <div class="cuerpo">${marcada(s.img, s.pasos)}<div class="lado">${leyenda(s.pasos)}<div class="virtud"><h4>Lo que la hace valiosa</h4><p>${esc(s.virtud)}</p></div></div></div>`, color);
    if (s.id === 'mapa') pag('seccion', `<header class="cab"><p class="ante" style="color:${color}">${esc(nomBloque)}</p><h2>La ficha de un territorio</h2><p class="bajada">Al tocar un municipio (o buscarlo por nombre) se abre su ficha sobre el mapa.</p></header><div class="cuerpo"><div class="captura"><img src="${ficha}"></div><div class="lado"><ol class="pasos sin">
      <li><b class="num">1</b><div><h4>Sus cifras</h4><p>Población, censo 2018, vías, superficie construida y registros recibidos.</p></div></li>
      <li><b class="num">2</b><div><h4>Sus gráficas</h4><p>Pirámide de edades y crecimiento, cada una con su fuente.</p></div></li>
      <li><b class="num">3</b><div><h4>Ver más</h4><p>El detalle abre la animación del crecimiento urbano sobre la foto satelital.</p></div></li>
      <li><b class="num">4</b><div><h4>Para llevar</h4><p>Un reporte del municipio listo para imprimir.</p></div></li></ol><div class="virtud"><h4>Lo que la hace valiosa</h4><p>Lo que antes pedía abrir varios archivos y portales, para un municipio, cabe en una ficha.</p></div></div></div>`, color);
    if (s.id === 'recibir') { const fuente = inst || { img: s.img, pasos: s.pasos, quien: 'su institución' };
      pag('institucion', `<div class="banda"><p class="ante claro">Para instituciones</p><h2>Su archivo, en tres pasos</h2><p>Si usted es enlace de una institución (PNC, RENAP, SE-CONRED…), esta es la única página que necesita.</p>
        <ol class="tres"><li><b>1</b><div><h4>Entre con su cuenta</h4><p>El INE le entrega un usuario atado a su institución. Al entrar por primera vez cambie la contraseña.</p></div></li><li><b>2</b><div><h4>Elija la solicitud y el archivo</h4><p>Solo verá las solicitudes de su institución. Suba el Excel o CSV tal como sale de su sistema.</p></div></li><li><b>3</b><div><h4>Validar y homologar</h4><p>En segundos sabrá cuántas filas pasaron y qué hay que corregir.</p></div></li></ol>
        <div class="tranquilo"><h4>Qué pasa con sus datos</h4><p>Nombres y documentos de identidad se eliminan o se sustituyen por una llave. El archivo original no se guarda. Usted no ve los envíos de otras instituciones, ni ellas los suyos.</p></div></div>
        <div class="demo"><div class="captura"><img src="${fuente.img}"></div><p class="pie-cap">La pantalla de una cuenta de institución${inst ? ` (envía por ${esc(inst.quien)})` : ''}: un rótulo dice por quién envía y la lista trae solo sus solicitudes. Los envíos de la captura son los archivos de ejemplo, ficticios.</p><div class="tres-datos"><div><b>Solo sus solicitudes</b><span>La lista no trae las de otras instituciones.</span></div><div><b>Solo sus envíos</b><span>El historial es el suyo, con fecha y corte.</span></div><div><b>Lo público, completo</b><span>Mapa, tableros, precios y fuentes siguen a la vista.</span></div></div></div>`); }
  });
  // Teléfono
  pag('telefono', `<header class="cab"><p class="ante">En cualquier lugar</p><h2>También en el teléfono</h2><p class="bajada">El portal se acomoda a la pantalla: el menú se guarda tras el botón ☰ y los controles crecen para el dedo. Así trabaja un operador en campo.</p></header><div class="moviles"><figure><img src="${telMapa}"><figcaption>El mapa</figcaption></figure><figure><img src="${telMenu}"><figcaption>El menú, al tocar ☰</figcaption></figure><figure><img src="${telRec}"><figcaption>El recorrido guiado</figcaption></figure>
    <div class="virtud alto"><h4>Lo que lo hace valioso</h4><p>Sin instalar nada: se abre en el navegador del teléfono, la tableta o la computadora.</p><p>Para registrar visitas con ubicación, el portal debe publicarse con HTTPS.</p></div></div>`);
  // Virtudes
  const v = (t, x) => `<div class="v"><h3>${t}</h3><p>${x}</p></div>`;
  pag('virtudes', `${haz}<header><p class="ante claro">En resumen</p><h2>Ocho razones para confiar en FARO</h2></header><div class="vs">
    ${v('No adivina', 'Asigna un código oficial solo con coincidencia exacta y única, o con la decisión registrada de una persona.')}
    ${v('No inventa', 'Todo dato es real, con fuente, fecha y licencia. Lo que es de muestra está rotulado.')}
    ${v('Cuida a las personas', 'El archivo original no se guarda, solo se publican conteos y las celdas pequeñas se protegen.')}
    ${v('Recuerda', 'Cada decisión queda en la memoria con nombre y fecha: no se corrige dos veces lo mismo.')}
    ${v('Se puede repetir', 'Cada resultado tiene un procedimiento que lo vuelve a obtener desde la fuente.')}
    ${v('Cada quien ve lo suyo', 'Roles, grupos y cuentas por institución; cada acción queda en la bitácora.')}
    ${v('Se explica solo', 'Recorridos guiados en las 24 secciones y una frase que dice qué muestra cada pantalla. También en inglés, y en kaqchikel como borrador por revisar.')}
    ${v('Queda en el INE', 'Hecho con software libre; corre en un servidor propio, sin servicios de pago. El INE recibe licencia perpetua para usarlo y modificarlo.')}</div>`);
  // Preguntas
  const q = (a, b) => `<div class="q"><h4>${a}</h4><p>${b}</p></div>`;
  pag('preguntas', `<header class="cab"><p class="ante">Si algo no sale</p><h2>Preguntas frecuentes</h2></header><div class="qs">
    ${q('No veo una sección en el menú', 'El menú muestra solo lo que su rol y su grupo permiten. Si la necesita, pídala a quien administra el portal.')}
    ${q('Olvidé mi contraseña', 'Un administrador le pone una nueva en «Usuarios». Al entrar deberá cambiarla.')}
    ${q('Mi archivo trae las columnas con otros nombres', 'No importa: FARO las reconoce por su contenido y sus nombres habituales. Las que no se piden se ignoran.')}
    ${q('Un lugar no quedó con código', 'Aparece en «Revisar pendientes» para que el personal del INE lo decida. No se asigna al azar.')}
    ${q('Subí el archivo equivocado', 'Vuelva a subir el correcto: el mismo corte reemplaza al anterior. Un administrador puede eliminar un envío.')}
    ${q('El portal dice «protegido» en una celda', 'Hay menos de ' + esc(datos.umbral) + ' casos: el número exacto se oculta para que nadie pueda ser identificado.')}
    ${q('El teléfono no da mi ubicación', 'Los navegadores solo comparten la ubicación en sitios con HTTPS. Avise a quien administra el portal.')}
    ${q('Quiero comprobar una cifra', 'Cada sección tiene su fuente; en «Acerca y fuentes» está la cita completa, y en «Proceso» el camino del dato.')}</div>
    <p class="franja">¿Más detalle? El documento técnico y la guía de instalación están en la carpeta <code>docs/</code> del repositorio. <b>Conformidad:</b> FARO cumple cada lineamiento de la nota conceptual del Datatón INE 2026 (<code>docs/26</code>) y tiene un análisis de conformidad legal, norma por norma (<code>docs/23</code>, no es un dictamen); el resumen está en el README.</p>`);
  // Contraportada
  const muestra = (c, n) => `<div class="muestra"><i style="background:${c}"></i><b>${n}</b><span>${c}</span></div>`;
  pag('contra', `${haz}<div class="centro">${icono(120, '#0080ff')}<h2>FARO</h2><p class="lema">Que cada dato llegue a puerto.</p><div class="paleta">${muestra('#004080', 'Azul oscuro')}${muestra('#0080ff', 'Azul radiante')}${muestra('#66b3ff', 'Azul claro')}${muestra('#e5f2ff', 'Cielo')}${muestra('#ffd500', 'Amarillo')}</div><p class="tipo">Paleta y tipografía (Ubuntu Sans) de la guía de diseño del INE · La torre es el catálogo oficial, fijo; la luz es el dato que sale ya entendido.</p><div class="carta">${require('./_tipografia.js').carta()}</div><p class="equipo">La Sociedad de Almas · Estuardo Sandoval · Ana Echeverría · Alejandro Sandoval<br>Datatón INE 2026 · Categoría 2</p></div>`);
  // Índice visual
  paginas[indice].cuerpo = `<header class="cab"><p class="ante">El portal de un vistazo</p><h2>Mapa del manual</h2><p class="bajada">Seis bloques, veinticuatro secciones. El color de cada bloque acompaña sus páginas.</p></header><p class="franja">Las capturas de este manual se tomaron del portal en marcha, con los archivos de ejemplo —ficticios y rotulados— que el propio portal ofrece. Los números marcan los mismos elementos que señala el recorrido guiado.</p><div class="bloques">${Object.entries(BLOQUES).map(([k, [n, c]]) => `<div class="bloque" style="--c:${c}"><h3>${n}</h3><ul>${secciones.filter(s => s.bloque === k).map(s => `<li><span>${esc(s.titulo)}</span><b>${paginaDe[s.id]}</b></li>`).join('')}</ul></div>`).join('')}</div>`;

  const html = `<!doctype html><html lang="es"><head><meta charset="utf-8"><title>FARO · Manual de usuario</title>
  <style>${require('./_tipografia.js').CSS}</style><style>
  @page { size: 279.4mm 215.9mm; margin: 0; } * { box-sizing: border-box; } html { -webkit-print-color-adjust: exact; print-color-adjust: exact; }
  body { margin: 0; font: 10.5pt/1.45 "Ubuntu Sans", "DejaVu Sans", Arial, sans-serif; color: #313131; }
  .pag { width: 279.4mm; height: 215.9mm; position: relative; overflow: hidden; break-after: page; padding: 13mm 14mm 14mm 20mm; background: #fff; }
  .pag > .lomo { position: absolute; left: 0; top: 0; bottom: 0; width: 8mm; background: var(--c, #0080ff); } .pag > .lomo::after { content: ""; position: absolute; left: 0; right: 0; top: 0; height: 26mm; background: #ffd500; }
  .contra .carta { width: 150mm; margin: 6mm auto 0; } .contra .carta svg { display: block; width: 100%; height: auto; }
  .pag > .ine { position: absolute; right: 0; bottom: 0; background: #004080; border-top-left-radius: 5mm; padding: 2.6mm 5mm 2.6mm 4.4mm; } .pag .ine img { height: 7mm; display: block; } .pag.oscura .ine { background: #0080ff; }
  .pag > footer { position: absolute; left: 20mm; right: 42mm; bottom: 6mm; display: flex; align-items: center; gap: 2.5mm; font-size: 8pt; color: #5b6672; } .pag > footer b { color: #004080; letter-spacing: 1px; } .pag > footer span.n { margin-left: auto; font-weight: 700; color: #004080; font-size: 10pt; } .pag > footer em { font-style: normal; }
  h2 { font-size: 23pt; line-height: 1.12; color: #004080; margin: 0; font-weight: 800; letter-spacing: -.2px; } h3 { margin: 0; color: #004080; } h4 { margin: 0; font-size: 10.5pt; color: #004080; } p { margin: 0; }
  .ante { font-size: 8.5pt; font-weight: 700; letter-spacing: 2.2px; text-transform: uppercase; color: #0080ff; margin: 0 0 1.5mm; } .ante.claro { color: #ffe566; }
  .cab { margin: 0 0 5mm; position: relative; } .bajada { font-size: 11.5pt; color: #5b6672; margin-top: 1.5mm; max-width: 190mm; } .cab .chips { position: absolute; right: 0; top: 0; display: flex; gap: 1.5mm; flex-wrap: wrap; justify-content: flex-end; max-width: 80mm; }
  .chip { font-size: 8pt; font-weight: 700; color: #fff; background: var(--c); border-radius: 99px; padding: .8mm 3mm; white-space: nowrap; }
  .nsec { display: inline-grid; place-items: center; width: 12mm; height: 12mm; border-radius: 3.2mm; color: #fff; font-size: 14pt; margin-right: 3mm; vertical-align: 1mm; }
  code { background: #e5f2ff; border-radius: 3px; padding: 0 4px; font-size: 10pt; color: #004080; }
  .cuerpo { display: grid; grid-template-columns: 172mm 1fr; gap: 6mm; align-items: start; }
  .captura { position: relative; border-radius: 3mm; overflow: hidden; box-shadow: 0 1.2mm 5mm rgba(7,48,107,.22); border: .3mm solid #d9dfe6; line-height: 0; } .captura img { width: 100%; display: block; }
  .captura .caja { position: absolute; border: .7mm solid #ffd500; border-radius: 1.5mm; box-shadow: 0 0 0 .3mm rgba(255,255,255,.8); }
  .captura .marca, .pasos .num { display: grid; place-items: center; width: 7mm; height: 7mm; border-radius: 50%; background: #ffd500; color: #004080; font: 800 10.5pt/1 "Ubuntu Sans", sans-serif; box-shadow: 0 0 0 .6mm #fff, 0 .6mm 2mm rgba(0,0,0,.35); } .captura .marca { position: absolute; transform: translate(-35%, -35%); }
  .pasos { list-style: none; margin: 0; padding: 0; display: grid; gap: 2.6mm; } .pasos li { display: grid; grid-template-columns: 7mm 1fr; gap: 3mm; align-items: start; } .pasos .num { box-shadow: none; } .pasos p { font-size: 9.5pt; line-height: 1.36; color: #313131; } .pasos em { font-style: normal; font-weight: 600; font-size: 7.5pt; color: #8a5a00; background: #fffbe5; border-radius: 3px; padding: 0 1.2mm; white-space: nowrap; }
  .virtud { margin-top: 4mm; background: #004080; color: #fff; border-radius: 3mm; padding: 3.2mm 4mm; border-left: 1.6mm solid #ffd500; } .virtud h4 { color: #ffe566; font-size: 8.5pt; letter-spacing: 1.6px; text-transform: uppercase; margin-bottom: 1.2mm; } .virtud p { font-size: 9.6pt; line-height: 1.38; } .virtud p + p { margin-top: 2mm; color: #b3d9ff; }
  .franja { position: absolute; left: 20mm; right: 14mm; bottom: 15mm; background: #fffbe5; border-left: 1.6mm solid #ffd500; border-radius: 2mm; padding: 3mm 4.5mm; font-size: 10.5pt; }
  .boton-rec { display: inline-block; background: #fffbe5; border: .4mm solid #ffd500; border-radius: 99px; padding: 0 3mm; font-weight: 600; color: #6b4300; font-size: 10pt; }
  /* Portada, virtudes y contraportada: fondo azul profundo con el haz del faro */
  .oscura { background: #004080; color: #fff; padding: 0; } .haz { position: absolute; inset: 0; width: 100%; height: 100%; }
  .portada .torre { position: absolute; right: 22mm; top: 38mm; filter: drop-shadow(0 4mm 10mm rgba(0,0,0,.35)); } .portada .texto { position: absolute; left: 24mm; top: 50mm; width: 150mm; }
  .portada .sobre { font-weight: 700; letter-spacing: 5px; text-transform: uppercase; color: #ffe566; font-size: 12pt; } .portada h1 { font-size: 96pt; line-height: 1; margin: 3mm 0 2mm; letter-spacing: 6px; font-weight: 800; } .portada .lema { font-size: 21pt; font-weight: 600; color: #fff; } .portada .desc { font-size: 12.5pt; color: #b3d9ff; margin-top: 9mm; max-width: 120mm; } .portada .pie-portada { margin-top: 30mm; font-size: 10pt; color: #b3d9ff; letter-spacing: .5px; border-top: .3mm solid rgba(255,255,255,.3); padding-top: 3mm; display: inline-block; }
  .que { padding: 0; display: grid; grid-template-columns: 1fr 104mm; } .que .col-a { padding: 20mm 12mm 14mm 24mm; } .que .col-b { background: #004080; color: #fff; padding: 20mm 12mm; } .que h2 { font-size: 29pt; } .que .grande { font-size: 14pt; line-height: 1.5; margin-top: 8mm; } .que .grande b { color: #0080ff; }
  .que .mv { display: grid; grid-template-columns: 1fr 1fr; gap: 6mm; margin-top: 9mm; } .que .mv div { border-top: 1mm solid #ffd500; padding-top: 2.5mm; } .que .mv h3 { font-size: 11pt; letter-spacing: 1.5px; text-transform: uppercase; } .que .mv p { font-size: 11pt; margin-top: 1.5mm; }
  .verbos { position: absolute; left: 24mm; bottom: 16mm; display: flex; align-items: center; gap: 3mm; } .verbos span { background: #e5f2ff; color: #004080; font-weight: 700; font-size: 12pt; border-radius: 99px; padding: 1.5mm 5mm; } .verbos span:last-child { background: #ffd500; } .verbos i { font-style: normal; color: #66b3ff; font-size: 14pt; font-weight: 700; }
  .cifras { display: grid; grid-template-columns: 1fr 1fr; gap: 4mm; margin-top: 4mm; } .cifra { background: rgba(255,255,255,.08); border-radius: 3mm; padding: 4mm; border-top: 1mm solid #ffd500; } .cifra b { display: block; font-size: 24pt; line-height: 1.1; font-weight: 800; } .cifra span { font-size: 9pt; color: #b3d9ff; line-height: 1.3; display: block; margin-top: 1mm; } .nota-b { font-size: 8.5pt; color: #b3d9ff; margin-top: 5mm; }
  .caminos { display: grid; grid-template-columns: repeat(3, 1fr); gap: 5mm; } .camino { border: .3mm solid #d9dfe6; border-top: 1.6mm solid var(--c); border-radius: 3mm; padding: 5mm 5mm; position: relative; min-height: 60mm; } .camino .inicial { position: absolute; right: 4mm; top: 3mm; width: 10mm; height: 10mm; border-radius: 50%; background: var(--c); color: #fff; display: grid; place-items: center; font-weight: 800; font-size: 13pt; } .camino h3 { font-size: 15pt; } .camino .quien { color: #5b6672; font-size: 9.5pt; margin: .5mm 0 2mm; } .camino p { font-size: 11pt; } .camino .empieza { margin-top: 2.5mm; font-size: 9.5pt; color: #004080; } .camino .empieza b { color: var(--c); text-transform: uppercase; font-size: 8pt; letter-spacing: 1px; display: block; }
  .bloques { display: grid; grid-template-columns: repeat(3, 1fr); gap: 5mm; } .bloque { border-radius: 3mm; background: #f4f6f9; overflow: hidden; } .bloque h3 { background: var(--c); color: #fff; padding: 2.5mm 4.5mm; font-size: 12pt; } .bloque ul { list-style: none; margin: 0; padding: 2.5mm 4.5mm 3.5mm; } .bloque li { display: flex; align-items: baseline; gap: 2mm; padding: 1.1mm 0; font-size: 10.5pt; border-bottom: .2mm dotted #b8c2cc; } .bloque li:last-child { border: 0; } .bloque li b { margin-left: auto; color: var(--c); }
  .institucion { padding: 0; display: grid; grid-template-columns: 112mm 1fr; } .institucion .banda { background: #004080; color: #fff; padding: 16mm 10mm 12mm 22mm; position: relative; } .institucion .banda::before { content: ""; position: absolute; left: 0; top: 0; bottom: 0; width: 8mm; background: #ffd500; } .institucion h2 { color: #fff; font-size: 25pt; } .institucion .banda > p { color: #b3d9ff; margin-top: 2.5mm; font-size: 11pt; }
  .tres { list-style: none; padding: 0; margin: 7mm 0 0; display: grid; gap: 4.5mm; } .tres li { display: grid; grid-template-columns: 13mm 1fr; gap: 3.5mm; } .tres li > b { width: 13mm; height: 13mm; border-radius: 50%; background: #ffd500; color: #004080; display: grid; place-items: center; font-size: 17pt; font-weight: 800; } .tres h4 { color: #fff; font-size: 12.5pt; } .tres p { color: #d9ecff; font-size: 10pt; }
  .tranquilo { margin-top: 7mm; background: rgba(255,255,255,.1); border-radius: 3mm; padding: 4mm; } .tranquilo h4 { color: #ffe566; text-transform: uppercase; font-size: 8.5pt; letter-spacing: 1.6px; } .tranquilo p { font-size: 9.8pt; margin-top: 1mm; }
  .institucion .demo { padding: 20mm 12mm 0 10mm; background: #e5f2ff; } .tres-datos { display: grid; grid-template-columns: repeat(3, 1fr); gap: 4mm; margin-top: 7mm; } .tres-datos div { background: #fff; border-radius: 3mm; border-top: 1.2mm solid #ffd500; padding: 3.5mm; } .tres-datos b { display: block; color: #004080; font-size: 11pt; } .tres-datos span { font-size: 9.5pt; color: #5b6672; } .pie-cap { font-size: 9pt; color: #5b6672; margin-top: 3mm; line-height: 1.4; }
  .moviles { display: grid; grid-template-columns: 60mm 60mm 60mm 1fr; gap: 7mm; align-items: start; } .moviles figure { margin: 0; } .moviles img { width: 100%; border-radius: 5mm; border: 1.6mm solid #004080; box-shadow: 0 2mm 6mm rgba(7,48,107,.3); display: block; } .moviles figcaption { text-align: center; font-size: 9.5pt; color: #5b6672; margin-top: 2mm; font-weight: 600; } .virtud.alto { margin-top: 0; }
  .virtudes header { position: absolute; left: 24mm; top: 20mm; } .virtudes h2, .contra h2 { color: #fff; } .vs { position: absolute; left: 24mm; right: 18mm; top: 52mm; display: grid; grid-template-columns: repeat(4, 1fr); gap: 5mm; } .v { background: rgba(7,48,107,.82); border: .3mm solid rgba(255,255,255,.22); border-top: 1.4mm solid #ffd500; border-radius: 3mm; padding: 4.5mm; min-height: 58mm; } .v h3 { color: #fff; font-size: 14pt; } .v p { color: #d9ecff; font-size: 10pt; margin-top: 2mm; }
  .qs { display: grid; grid-template-columns: 1fr 1fr; gap: 4mm 8mm; } .q { border-left: 1.2mm solid #66b3ff; padding: 1mm 0 1mm 4mm; } .q h4 { font-size: 13pt; } .q p { font-size: 11.5pt; margin-top: 1mm; } .qs { margin-top: 4mm; row-gap: 7mm; }
  .contra .centro { position: absolute; inset: 0; display: flex; flex-direction: column; align-items: center; justify-content: center; text-align: center; } .contra h2 { font-size: 46pt; letter-spacing: 5px; margin-top: 5mm; } .contra .lema { font-size: 16pt; color: #ffe566; font-weight: 600; margin-top: 1mm; }
  .paleta { display: flex; gap: 5mm; margin-top: 12mm; } .muestra { width: 30mm; } .muestra i { display: block; height: 14mm; border-radius: 2.5mm; border: .3mm solid rgba(255,255,255,.35); } .muestra b { display: block; font-size: 9pt; margin-top: 1.5mm; } .muestra span { font-size: 8pt; color: #b3d9ff; } .contra .tipo { margin-top: 8mm; font-size: 9.5pt; color: #b3d9ff; max-width: 150mm; } .contra .equipo { margin-top: 9mm; font-size: 10pt; line-height: 1.6; }
  </style></head><body>${paginas.map((g, i) => { const oscura = ['portada', 'virtudes', 'contra'].includes(g.clase), sinMarco = oscura || ['que', 'institucion'].includes(g.clase);
    return `<section class="pag ${g.clase}${oscura ? ' oscura' : ''}"${g.color ? ` style="--c:${g.color}"` : ''}>${sinMarco ? '' : '<div class="lomo"></div>'}${g.cuerpo}${sinMarco ? '' : `<footer>${icono(14)}<b>FARO</b><em>· Manual de usuario · Que cada dato llegue a puerto</em><span class="n">${i + 1}</span></footer>`}<div class="ine"><img src="${require('./_tipografia.js').ine('ine-horizontal-blanco')}" alt="Instituto Nacional de Estadística"></div></section>`; }).join('')}</body></html>`;
  const d = await ctx.newPage(); await d.setContent(html, { waitUntil: 'networkidle' }).catch(() => {}); await d.evaluate(() => document.fonts.ready);
  await d.pdf({ path: salida, width: '279.4mm', height: '215.9mm', printBackground: true, preferCSSPageSize: true });
  if (process.env.FARO_MANUAL_HTML) fs.writeFileSync(process.env.FARO_MANUAL_HTML, html);
  await nav.close(); console.log(`${paginas.length} páginas → ${salida} (${(fs.statSync(salida).size / 1048576).toFixed(1)} MB)`);
})().catch(e => { console.error(e); process.exit(1); });
