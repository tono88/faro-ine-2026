// Graba el video demostrativo de FARO (entregables/FARO-video-demo.mp4) recorriendo el portal en marcha:
// un caso resuelto de principio a fin y un barrido por los demás frentes, con rótulos, tarjetas de transición
// y el cursor a la vista. No lleva voz: la narración (el guion de abajo, GUION) se graba aparte y se monta
// encima; junto al video se escribe un archivo de subtítulos (.srt) con los tiempos reales de cada escena.
//
// Cómo captura: abre Chromium a 1280×720 con escala 1.5 (sale a 1920×1080 nítido), pide a Chromium cada
// cuadro que cambia y los une con ffmpeg. La escena del teléfono se graba aparte y se compone sobre un fondo.
// Requiere Node, Playwright con Chromium y ffmpeg. Antes: una instalación preparada para demostrar
// (scripts/entregables/preparar_demo.js, o los archivos propios ya subidos por la pantalla).
//
// Uso: node scripts/entregables/grabar_video.js <url> <contraseña de las cuentas de demostración> [config.json]
// La configuración opcional cambia cuentas, solicitud, archivo que se sube en cámara y textos (ver CFG).
// Regla: en cámara nunca se abre el contenido de un archivo; el portal solo muestra conteos.
const { chromium } = require('playwright'); const fs = require('fs'); const os = require('os'); const path = require('path'); const { execFileSync } = require('child_process');
const [url = 'http://localhost:8082', clave = process.env.FARO_CLAVE || '', rutaConfig] = process.argv.slice(2);
const raiz = path.resolve(__dirname, '../..'), leer = f => fs.readFileSync(path.join(raiz, f), 'utf8');
const CFG = { admin: 'admin', analista: 'mlopez', institucion: 'enlace.pnc', solicitud: 'pnc-detenidos-v1', archivo: null, nombreArchivo: null,
  caso: ['Un caso de principio a fin', 'Archivo de ejemplo (ficticio) de PNC'], vozCaso: null, pendiente: null, buscar: 'Panajachel', municipio: '0101', producto: 'arroz',
  frase: 'mapa y tabla de desnutrición y población en Sololá', fotoCampo: null, creditoFoto: '', salida: path.join(raiz, 'entregables', 'FARO-video-demo.mp4'), trabajo: path.join(os.tmpdir(), 'faro-video'),
  ...(rutaConfig ? JSON.parse(fs.readFileSync(rutaConfig, 'utf8')) : {}) };
const esc = t => String(t ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const dormir = ms => new Promise(r => setTimeout(r, ms));
const mejoras = JSON.parse(leer('datos/estandar/mejoras.json')).puntos, M = t => (mejoras.find(m => m.tema === t) || {}).cifra || '';
const pruebas = (() => { let n = 0; const rec = d => fs.readdirSync(d, { withFileTypes: true }).forEach(x => { const r = path.join(d, x.name); if (x.isDirectory()) rec(r); else if (x.name.endsWith('.rs')) n += (fs.readFileSync(r, 'utf8').match(/#\[test\]/g) || []).length; }); rec(path.join(raiz, 'crates')); return n; })();

// ---------- Guion de la narración: una entrada por escena, con su duración en segundos ----------
// Ritmo: unas 140 palabras por minuto, con aire para que se vea la pantalla. Si cambia un texto, ajuste la
// duración de su escena (palabras ÷ 2.3 ≈ segundos de voz) y vuelva a grabar.
const GUION = [
  { id: 'apertura', dur: 13, voz: 'El INE ya tiene los datos que necesita. Pero cada institución los envía a su manera, y no llegan en una forma que se pueda usar. Para eso construimos FARO.' },
  { id: 'caso', dur: 24.5, voz: CFG.vozCaso || 'Este es el archivo de una institución, tal como sale de su sistema. La institución entra con su propia cuenta y lo sube. En segundos, FARO reconoce las columnas, pone el código oficial a cada lugar y deja fuera los datos personales. El archivo original no se guarda.' },
  { id: 'decision', dur: 10.5, voz: 'Y cuando hay duda, FARO no adivina: propone, una persona decide una sola vez, y esa decisión queda aprendida para todas las instituciones.' },
  { id: 'territorio', dur: 21, voz: 'Lo mismo, en cada frente del INE. Para Censos y Encuestas y las delegaciones: el país hasta el lugar poblado, con población, censo y crecimiento urbano visto desde el satélite. Y la red vial como un grafo: setecientos siete lugares sin conexión mapeada.' },
  { id: 'campo', dur: 15, voz: 'Para el trabajo de campo: cada operador ve su ruta en el teléfono, y cada visita se comprueba con ubicación y foto.' },
  { id: 'precios', dur: 18, voz: 'Para Precios e Índices: más de ochenta y cinco mil precios leídos de dieciocho tiendas, comparados por código de barras, con su informe de precio de referencia.' },
  { id: 'tableros', dur: 13, voz: 'Y para quien necesita comunicar: un tablero se pide con palabras y se publica con un enlace.' },
  { id: 'convergencia', dur: 9, voz: 'No son cinco herramientas: es una sola. Atacar el problema desde varios frentes los hace converger en una solución integral.' },
  { id: 'autonomia', dur: 23, voz: 'Y tomamos una decisión de ingeniería: FARO es autocontenido. Software libre, en un servidor del INE: sin licencias, sin servicios de pago y sin depender de una inteligencia artificial externa que mañana puede cambiar de precio o desaparecer. Los datos se quedan en la institución, y el mismo dato da siempre el mismo resultado.' },
  { id: 'escala', dur: 12, voz: `Crece sin programar: sumar una institución es escribir un archivo. Y está comprobado: más de cincuenta mil registros reales, ${pruebas} pruebas automáticas, y se instala con un comando.` },
  { id: 'guia', dur: 5.5, voz: 'Para quien evalúa: la guía para evaluadores, al inicio del menú.' },
  { id: 'cierre', dur: 7, voz: 'Y en más de un idioma. FARO: que cada dato llegue a puerto.' },
];
const G = id => GUION.find(g => g.id === id);

// ---------- Lo que se inyecta en la página: cursor, rótulos, bandas y tarjetas ----------
const ICONO = `<svg viewBox="0 0 64 64" aria-hidden="true"><rect width="64" height="64" rx="14" fill="#0080ff"/><path d="M26 21 5 12v17z" fill="#ffd500" opacity=".9"/><path d="M38 21 59 12v17z" fill="#ffd500" opacity=".9"/><path d="M32 8l8 8H24z" fill="#fff"/><rect x="26" y="16" width="12" height="8" rx="1" fill="#ffe566"/><path d="M27 26h10l4 28H23z" fill="#fff"/><path d="M26.1 34h11.8l.9 6H25.2z" fill="#66b3ff"/><path d="M24.4 46h15.2l.6 4H23.8z" fill="#66b3ff"/><rect x="18" y="54" width="28" height="4" rx="2" fill="#fff"/></svg>`;
const INE_BLANCO = require('./_tipografia.js').ine('ine-horizontal-blanco');
const CARTA = require('./_tipografia.js').carta();
const HAZ = `<div class="vz-ine"><img src="${INE_BLANCO}" alt="Instituto Nacional de Estadística"></div><svg class="vz-haz" viewBox="0 0 1000 562" preserveAspectRatio="none" aria-hidden="true"><defs><linearGradient id="vzh" x1="1" x2="0"><stop offset="0" stop-color="#ffd500" stop-opacity=".5"/><stop offset="1" stop-color="#ffd500" stop-opacity="0"/></linearGradient></defs><path d="M790 250 L0 30 L0 270 Z" fill="url(#vzh)"/><path d="M790 250 L0 320 L0 450 Z" fill="url(#vzh)" opacity=".45"/><g fill="none" stroke="#66b3ff" stroke-width="2" opacity=".3"><path d="M0 505 q60 -20 120 0 t120 0 t120 0 t120 0 t120 0 t120 0 t120 0 t120 0 t120 0"/><path d="M-40 532 q60 -20 120 0 t120 0 t120 0 t120 0 t120 0 t120 0 t120 0 t120 0 t120 0" opacity=".7"/></g></svg>`;
// La última lista del resultado de un envío trae el valor de algunas filas («Fila N · variable · "valor"»).
// En cámara esa lista no se dibuja en ningún momento (ni mientras aparece la banda que además la cubre):
// en el video solo van conteos.
const CSS = `
#resultado ul:last-of-type { visibility: hidden !important; }
.vz, .vz * { box-sizing: border-box; font-family: "Ubuntu Sans", "Segoe UI", system-ui, sans-serif; }
.vz { position: fixed; z-index: 2147483000; pointer-events: none; }
#vz-cursor { position: fixed; z-index: 2147483647; left: 0; top: 0; right: auto; bottom: auto; inset: 0 auto auto 0; margin: 0; border: 0; padding: 0; background: transparent; overflow: visible; width: 26px; height: 26px; pointer-events: none; transform: translate(640px, 360px); filter: drop-shadow(0 2px 3px rgba(0,0,0,.45)); }
#vz-cursor { transition: opacity .3s; } #vz-cursor.dedo svg { display: none; } #vz-cursor.dedo { width: 34px; height: 34px; margin: -17px 0 0 -17px; border-radius: 50%; background: rgba(245,166,35,.55); border: 2px solid #fff; }
.vz-onda { position: fixed; z-index: 2147483646; width: 16px; height: 16px; margin: -8px 0 0 -8px; border-radius: 50%; border: 3px solid #ffd500; pointer-events: none; animation: vz-onda .55s ease-out forwards; }
@keyframes vz-onda { to { transform: scale(4.2); opacity: 0; } }
#vz-rotulo { left: 26px; bottom: 26px; max-width: 760px; background: rgba(7,48,107,.95); color: #fff; border-radius: 14px; padding: 12px 20px 13px; box-shadow: 0 10px 30px rgba(7,48,107,.4); opacity: 0; transform: translateY(16px); transition: opacity .45s, transform .45s; }
#vz-rotulo.ver { opacity: 1; transform: none; } #vz-rotulo small { display: block; font-size: 11.5px; font-weight: 700; letter-spacing: 2.2px; text-transform: uppercase; color: #ffe566; } #vz-rotulo b { display: block; font-size: 21px; line-height: 1.2; margin-top: 2px; } #vz-rotulo span { display: block; font-size: 14px; color: #cce6ff; margin-top: 3px; }
#vz-banda { left: 0; right: 0; bottom: 0; height: 190px; background: #004080; border-top: 4px solid #ffd500; padding: 34px 40px 0; align-items: flex-start; display: flex; gap: 22px; justify-content: center; opacity: 0; transform: translateY(30px); transition: opacity .5s, transform .5s; }
#vz-banda.ver { opacity: 1; transform: none; } #vz-banda div { background: rgba(255,255,255,.1); border-top: 4px solid #ffd500; border-radius: 12px; padding: 12px 22px; color: #fff; min-width: 250px; } #vz-banda b { display: block; font-size: 34px; font-weight: 800; line-height: 1.1; } #vz-banda span { font-size: 14px; color: #cce6ff; }
#vz-tarjeta { inset: 0; background: #004080; color: #fff; opacity: 0; transition: opacity .55s; overflow: hidden; } #vz-tarjeta.ver { opacity: 1; } #vz-tarjeta.velo { background: rgba(7,48,107,.93); }
.vz-haz { position: absolute; inset: 0; width: 100%; height: 100%; }
.vz-ine { position: absolute; right: 0; bottom: 0; z-index: 2; background: #0080ff; border-top-left-radius: 22px; padding: 12px 24px 12px 20px; } .vz-ine img { height: 34px; display: block; }
.vz-cierre-marca { display: flex; align-items: center; gap: 20px; text-align: left; } .vz-cierre-logo { flex: 0 0 78px; width: 78px; height: 78px; } .vz-cierre-logo svg { display: block; width: 78px; height: 78px; filter: drop-shadow(0 6px 14px rgba(0,0,0,.35)); } .vz-carta { width: 800px; margin-top: 26px; } .vz-carta svg { display: block; width: 100%; height: auto; }
.vz-c { position: absolute; inset: 0; display: flex; flex-direction: column; justify-content: center; padding: 0 90px; } .vz-c.centro { align-items: center; text-align: center; }
.vz-k { font-size: 15px; font-weight: 700; letter-spacing: 3.5px; text-transform: uppercase; color: #ffe566; } .vz-t { font-size: 50px; font-weight: 800; line-height: 1.12; margin-top: 8px; max-width: 980px; } .vz-s { font-size: 23px; color: #cce6ff; margin-top: 14px; max-width: 900px; line-height: 1.4; }
.vz-sale { opacity: 0; transform: translateY(18px); animation: vz-sale .7s ease-out forwards; } @keyframes vz-sale { to { opacity: 1; transform: none; } }
.vz-logo svg { width: 150px; height: 150px; filter: drop-shadow(0 8px 20px rgba(0,0,0,.35)); } .vz-marca { font-size: 92px; font-weight: 800; letter-spacing: 8px; line-height: 1; margin-top: 16px; } .vz-lema { font-size: 30px; font-weight: 600; color: #ffe566; margin-top: 8px; }
.vz-fila { display: flex; gap: 22px; margin-top: 34px; } .vz-pilar { flex: 1; background: rgba(255,255,255,.09); border-top: 5px solid #ffd500; border-radius: 16px; padding: 24px 26px 26px; } .vz-pilar b { display: block; font-size: 54px; font-weight: 800; line-height: 1; } .vz-pilar h4 { margin: 10px 0 6px; font-size: 22px; } .vz-pilar p { margin: 0; font-size: 16.5px; color: #cce6ff; line-height: 1.4; }
.vz-riesgo { margin-top: 30px; border-collapse: collapse; width: 100%; } .vz-riesgo th, .vz-riesgo td { background: transparent !important; border: 0; } .vz-riesgo th { text-align: left; font-size: 13px; letter-spacing: 2px; text-transform: uppercase; color: #ffe566 !important; padding: 0 18px 10px 0; } .vz-riesgo td { padding: 15px 18px 15px 0; border-top: 1px solid rgba(255,255,255,.22); font-size: 19.5px; vertical-align: top; line-height: 1.35; } .vz-riesgo td:first-child { font-weight: 700; width: 30%; } .vz-riesgo td:nth-child(2) { color: #cce6ff; width: 33%; } .vz-riesgo td { color: #fff; } .vz-riesgo td:last-child { color: #fff; } .vz-riesgo td:last-child::before { content: "✓ "; color: #7bd89f; font-weight: 800; }
.vz-frentes { position: absolute; left: 90px; top: 205px; display: grid; gap: 13px; width: 560px; } .vz-frente { background: rgba(255,255,255,.1); border-radius: 12px; padding: 11px 18px; border-left: 0; } .vz-frente b { font-size: 21px; } .vz-frente span { display: block; font-size: 14px; color: #cce6ff; }
.vz-lineas { position: absolute; left: 650px; top: 205px; width: 300px; height: 420px; } .vz-lineas path { fill: none; stroke: #ffd500; stroke-width: 3; stroke-dasharray: 420; stroke-dashoffset: 420; animation: vz-traza 1s ease-out forwards; } @keyframes vz-traza { to { stroke-dashoffset: 0; } }
.vz-una { position: absolute; left: 950px; top: 290px; text-align: center; width: 250px; } .vz-una svg { width: 130px; height: 130px; } .vz-una b { display: block; font-size: 44px; letter-spacing: 4px; margin-top: 8px; } .vz-una span { font-size: 17px; color: #ffe566; font-weight: 600; }
.vz-dos { display: grid; grid-template-columns: 1.05fr 1fr; gap: 40px; margin-top: 28px; align-items: start; } .vz-codigo { background: #002b57; border: 1px solid rgba(255,255,255,.18); border-radius: 14px; padding: 18px 22px; font: 15px/1.55 "DejaVu Sans Mono", Consolas, monospace; color: #d9ecff; white-space: pre; overflow: hidden; } .vz-codigo i { color: #ffe566; font-style: normal; } .vz-codigo u { color: #8fd3a8; text-decoration: none; }
.vz-cifras { display: grid; gap: 14px; } .vz-cifras div { background: rgba(255,255,255,.09); border-left: 0; border-top: 4px solid #ffd500; border-radius: 12px; padding: 12px 20px; } .vz-cifras b { font-size: 40px; font-weight: 800; line-height: 1.1; display: block; } .vz-cifras span { font-size: 16px; color: #cce6ff; }
.vz-pie { position: absolute; left: 90px; bottom: 44px; font-size: 15px; color: #cce6ff; }
`;
const INYECTADO = `(() => { const css = ${JSON.stringify(CSS)}, flecha = '<svg viewBox="0 0 24 24" width="26" height="26"><path d="M4 2l15 11-6.6.9 3.9 7.3-2.7 1.4-3.9-7.3L4 20z" fill="#fff" stroke="#004080" stroke-width="1.6" stroke-linejoin="round"/></svg>';
  const listo = () => { if (document.getElementById('vz-cursor')) return; const s = document.createElement('style'); s.textContent = css; document.head.append(s);
    const c = document.createElement('div'); c.id = 'vz-cursor'; c.innerHTML = flecha; c.setAttribute('popover', 'manual'); document.documentElement.append(c); const subir = () => { try { c.hidePopover(); } catch (_) {} try { c.showPopover(); } catch (_) {} }; subir();
    // Un diálogo modal se dibuja encima de todo: el cursor vuelve a ponerse arriba cada vez que se abre uno.
    const abrir = HTMLDialogElement.prototype.showModal; HTMLDialogElement.prototype.showModal = function () { const r = abrir.apply(this, arguments); subir(); return r; };
    for (const id of ['vz-rotulo', 'vz-banda', 'vz-tarjeta']) { const d = document.createElement('div'); d.id = id; d.className = 'vz'; document.documentElement.append(d); }
    addEventListener('mousemove', e => { c.style.transform = 'translate(' + e.clientX + 'px,' + e.clientY + 'px)'; }, true);
    addEventListener('mousedown', e => { const o = document.createElement('div'); o.className = 'vz-onda'; o.style.left = e.clientX + 'px'; o.style.top = e.clientY + 'px'; document.documentElement.append(o); setTimeout(() => o.remove(), 600); }, true); };
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', listo); else listo();
  const el = id => document.getElementById(id);
  window.vz = { dedo: () => el('vz-cursor').classList.add('dedo'),
    rotulo: (k, t, s) => { const r = el('vz-rotulo'); r.innerHTML = '<small>' + k + '</small><b>' + t + '</b>' + (s ? '<span>' + s + '</span>' : ''); r.classList.add('ver'); }, sinRotulo: () => el('vz-rotulo').classList.remove('ver'),
    banda: h => { const b = el('vz-banda'); b.innerHTML = h; b.classList.add('ver'); }, sinBanda: () => el('vz-banda').classList.remove('ver'),
    tarjeta: (h, velo) => { const t = el('vz-tarjeta'); t.innerHTML = h; t.classList.toggle('velo', !!velo); t.classList.add('ver'); el('vz-cursor').style.opacity = 0; }, sinTarjeta: () => { el('vz-tarjeta').classList.remove('ver'); el('vz-cursor').style.opacity = 1; } }; })();`;

// ---------- Tarjetas ----------
const sale = (i, h) => `<div class="vz-sale" style="animation-delay:${i}s">${h}</div>`;
const T = {
  apertura: () => `${HAZ}<div class="vz-c">${sale(0.3, '<div class="vz-t">El INE ya tiene los datos que el país necesita.</div>')}${sale(4.2, '<div class="vz-s" style="font-size:30px;color:#fff">Pero cada institución los envía a su manera…</div>')}${sale(7.2, '<div class="vz-s" style="font-size:30px;color:#ffe566;font-weight:600">…y no llegan en una forma que se pueda usar.</div>')}</div>`,
  marca: () => `${HAZ}<div class="vz-c centro">${sale(0.1, `<div class="vz-logo">${ICONO}</div>`)}${sale(0.35, '<div class="vz-marca">FARO</div>')}${sale(0.7, '<div class="vz-lema">Que cada dato llegue a puerto.</div>')}</div>`,
  frente: (n, t, para) => `<div class="vz-c">${sale(0, `<div class="vz-k">Frente ${n} de 5</div>`)}${sale(0.12, `<div class="vz-t">${esc(t)}</div>`)}${sale(0.3, `<div class="vz-s">Para ${esc(para)}</div>`)}</div>`,
  convergencia: () => { const F = [['Registros administrativos', 'Coordinación del SEN y las instituciones que envían'], ['Territorio y censo', 'Censos y Encuestas · Coordinación Regional'], ['Trabajo de campo', 'Operadores, cartografía y supervisión'], ['Precios e índices', 'Precios e Índices de Referencia · IPP · IPMC · HBA'], ['Tableros y datos abiertos', 'DESA y quien necesita comunicar']];
    return `${HAZ}<div class="vz-c" style="justify-content:flex-start;padding-top:56px">${sale(0, '<div class="vz-k">Cinco frentes, una raíz</div>')}${sale(0.15, '<div class="vz-t" style="font-size:36px;max-width:none">Varios frentes que convergen en una solución integral</div>')}</div>
      <div class="vz-frentes">${F.map((f, i) => sale(0.6 + i * 0.45, `<div class="vz-frente"><b>${f[0]}</b><span>${f[1]}</span></div>`)).join('')}</div>
      <svg class="vz-lineas" viewBox="0 0 300 420">${[36, 114, 192, 270, 348].map((y, i) => `<path d="M0 ${y} C 150 ${y}, 150 175, 300 175" style="animation-delay:${3 + i * 0.25}s"/>`).join('')}</svg>
      <div class="vz-una">${sale(4.4, `${ICONO}<b>FARO</b><span>una sola herramienta</span>`)}</div>`; },
  autonomia: () => `${HAZ}<div class="vz-c">${sale(0, '<div class="vz-k">Una decisión de ingeniería</div>')}${sale(0.15, '<div class="vz-t">Autocontenido por diseño</div>')}<div class="vz-fila">
      ${sale(0.9, '<div class="vz-pilar"><b>Q 0</b><h4>en licencias</h4><p>Todo es software libre: Rust, PostgreSQL y Leaflet.</p></div>')}${sale(2.6, '<div class="vz-pilar"><b>0</b><h4>servicios de pago</h4><p>Corre completo en un servidor del INE. Funciona sin internet con los datos que trae.</p></div>')}${sale(4.3, '<div class="vz-pilar"><b>0</b><h4>llamadas a una IA externa</h4><p>Reglas a la vista: el mismo dato da siempre el mismo resultado.</p></div>')}</div></div>`,
  riesgo: () => `${HAZ}<div class="vz-c">${sale(0, '<div class="vz-k">Análisis de riesgo</div>')}${sale(0.15, '<div class="vz-t" style="font-size:42px">El riesgo que el INE no tiene que correr</div>')}${sale(0.6, `<table class="vz-riesgo"><thead><tr><th>Si mañana…</th><th>Con dependencia externa</th><th>Con FARO</th></tr></thead><tbody>
      <tr><td>cambia el precio</td><td>el costo crece con cada consulta</td><td>no hay pago por uso</td></tr><tr><td>el servicio o el modelo se retira</td><td>la herramienta deja de funcionar</td><td>sigue funcionando: todo está en el servidor</td></tr><tr><td>hay que explicar un resultado</td><td>la respuesta puede cambiar de un día a otro</td><td>mismo dato, mismo resultado, con constancia</td></tr><tr><td>los datos no pueden salir</td><td>viajan a un tercero</td><td>se quedan en la institución</td></tr></tbody></table>`)}</div>`,
  escala: () => { const toml = leer('datos/solicitudes/pnc-detenidos-v1.toml').split('\n').filter(l => l.trim() && !l.trim().startsWith('#')).slice(0, 13).map(l => esc(l.length > 62 ? l.slice(0, 60) + '…' : l).replace(/^(\[\[?[^\]]+\]\]?)/, '<i>$1</i>').replace(/^(\w+)(\s*=)/, '<u>$1</u>$2')).join('\n');
    return `${HAZ}<div class="vz-c">${sale(0, '<div class="vz-k">Configurable, escalable y comprobado</div>')}${sale(0.15, '<div class="vz-t" style="font-size:42px">Crece sin programar</div>')}<div class="vz-dos">${sale(0.7, `<div><div class="vz-codigo">${toml}</div><div class="vz-s" style="font-size:17px;margin-top:10px">Sumar una institución es escribir un archivo como este.</div></div>`)}
      <div class="vz-cifras">${sale(5.2, `<div><b>${esc(M('Un estándar para pedir y recibir datos'))}</b><span>registros reales de RENAP, PNC y SE-CONRED procesados</span></div>`)}${sale(6.6, `<div><b>${pruebas}</b><span>pruebas automáticas, más pruebas de pantalla con clic real</span></div>`)}${sale(8, '<div><b>1 comando</b><span>para instalarlo: comprobado desde un clon limpio</span></div>')}</div></div></div>`; },
  cierre: () => `${HAZ}<div class="vz-c centro">${sale(0.1, `<div class="vz-cierre-marca"><div class="vz-cierre-logo">${ICONO}</div><div><div class="vz-marca" style="font-size:54px;line-height:1;margin:0">FARO</div><div class="vz-lema" style="font-size:22px;margin-top:6px">Que cada dato llegue a puerto.</div></div></div>`)}${sale(1.0, `<div class="vz-carta">${CARTA}</div>`)}${sale(2.0, '<div class="vz-s" style="font-size:17px;margin-top:20px;color:#ffe566">También en inglés · kaqchikel en borrador, por revisar con hablantes</div>')}${sale(2.4, '<div class="vz-s" style="font-size:16px;margin-top:8px">Datatón INE 2026 · github.com/tono88/faro-ine-2026</div>')}</div>`,
};

// ---------- Grabación ----------
async function grabar(nombre, ventana, escala, sesion, escenas, antes) {
  const proxy = process.env.HTTPS_PROXY, nav = await chromium.launch({ args: [`--force-device-scale-factor=${escala}`, `--window-size=${ventana[0]},${ventana[1]}`, '--hide-scrollbars'], ...(proxy ? { proxy: { server: proxy, bypass: 'localhost,127.0.0.1' } } : {}) });
  const ctx = await nav.newContext({ viewport: null }), p = await ctx.newPage(); p.on('dialog', d => d.accept()); await p.addInitScript(INYECTADO);
  const abrir = async () => { await p.goto(url); await p.waitForSelector('#lienzo path.leaflet-interactive'); await p.waitForTimeout(900); };
  await abrir(); if (sesion) { const r = await p.evaluate(async ([u, c]) => (await fetch('/api/sesion', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ usuario: u, clave: c }) })).status, [sesion, clave]); if (r !== 200) throw new Error(`no se pudo entrar como ${sesion} (${r})`); await abrir(); }
  const dir = path.join(CFG.trabajo, nombre); fs.rmSync(dir, { recursive: true, force: true }); fs.mkdirSync(dir, { recursive: true });
  const cdp = await ctx.newCDPSession(p), marcos = []; let ultimo = 0, n = 0;
  cdp.on('Page.screencastFrame', f => { cdp.send('Page.screencastFrameAck', { sessionId: f.sessionId }).catch(() => {}); const t = f.metadata.timestamp; if (t - ultimo < 1 / 31) return; ultimo = t; const a = path.join(dir, String(++n).padStart(6, '0') + '.jpg'); fs.writeFileSync(a, Buffer.from(f.data, 'base64')); marcos.push([t, a]); });
  // Ayudas de movimiento: el cursor viaja con aceleración y frenado, y cada clic deja una onda.
  let pos = { x: ventana[0] / 2, y: ventana[1] / 2 };
  const mover = async (x, y) => { const d = Math.hypot(x - pos.x, y - pos.y), pasos = Math.max(10, Math.min(46, Math.round(d / 13))); for (let i = 1; i <= pasos; i++) { const k = i / pasos, e = k < .5 ? 2 * k * k : 1 - Math.pow(-2 * k + 2, 2) / 2; await p.mouse.move(pos.x + (x - pos.x) * e, pos.y + (y - pos.y) * e); await dormir(11); } pos = { x, y }; };
  const A = { p, dormir, vz: (f, ...a) => p.evaluate(([f, a]) => window.vz[f](...a), [f, a]),
    ver: async (sel, bloque = 'center') => { const l = typeof sel === 'string' ? p.locator(sel).first() : sel; await l.evaluate((e, b) => e.scrollIntoView({ behavior: 'smooth', block: b }), bloque); await dormir(650); return l; },
    sobre: async (sel, opc = {}) => { const l = typeof sel === 'string' ? p.locator(sel).first() : sel; if (opc.ver !== false) { const c0 = await l.boundingBox(); if (!c0 || c0.y < 70 || c0.y + c0.height > ventana[1] - 30) await A.ver(l, opc.bloque); } const c = await l.boundingBox(); await mover(c.x + Math.min(c.width / 2, opc.dx ?? 1e9), c.y + c.height / 2); return l; },
    clic: async (sel, opc) => { await A.sobre(sel, opc); await dormir(170); await p.mouse.down(); await dormir(70); await p.mouse.up(); await dormir(120); },
    escribir: async (sel, texto, ms = 38) => { await A.clic(sel); await p.keyboard.type(texto, { delay: ms }); },
    tarjeta: async (html, velo) => { await A.vz('tarjeta', html, velo); await dormir(600); }, sinTarjeta: async () => { await A.vz('sinTarjeta'); await dormir(600); },
    entrar: async usuario => { await A.clic('#entrar'); await p.waitForSelector('#ingreso[open]'); await p.fill('#iusuario', ''); await p.fill('#iclave', ''); await p.focus('#iusuario'); await p.keyboard.type(usuario, { delay: 34 }); await p.keyboard.press('Tab'); await p.keyboard.type(clave, { delay: 22 }); await dormir(200); await p.keyboard.press('Enter'); await p.waitForSelector('#salir'); await dormir(500); } };
  if (antes) { await antes(A); await dormir(700); }
  const marcas = []; await cdp.send('Page.startScreencast', { format: 'jpeg', quality: 90, everyNthFrame: 1 }); await p.mouse.move(pos.x, pos.y); const t0 = Date.now();
  for (const [id, accion] of escenas) { const ini = Date.now(), dur = G(id).dur * 1000;
    try { await accion(A); } catch (e) { const captura = path.join(CFG.trabajo, `error-${id}.png`); await p.screenshot({ path: captura }).catch(() => {}); await nav.close(); throw new Error(`la escena «${id}» falló (captura: ${captura}): ${String(e.message).split('\n')[0]}`); }
    const usado = Date.now() - ini; if (usado < dur) await dormir(dur - usado); marcas.push({ id, inicio: (ini - t0) / 1000, fin: (Date.now() - t0) / 1000, sobra: (dur - usado) / 1000 }); }
  await p.mouse.move(pos.x + 1, pos.y); await dormir(120); const fin = Date.now(); await cdp.send('Page.stopScreencast'); await nav.close();
  // Los cuadros se unen con su duración real y se llevan a 30 cuadros por segundo.
  const lista = path.join(dir, 'lista.txt'), dura = (fin - t0) / 1000, base = marcos[0][0];
  fs.writeFileSync(lista, 'ffconcat version 1.0\n' + marcos.map(([t, a], i) => `file '${a}'\nduration ${Math.max(0.01, (i + 1 < marcos.length ? marcos[i + 1][0] : base + dura) - t).toFixed(4)}`).join('\n') + `\nfile '${marcos[marcos.length - 1][1]}'\n`);
  const mp4 = path.join(CFG.trabajo, nombre + '.mp4');
  execFileSync('ffmpeg', ['-y', '-loglevel', 'error', '-f', 'concat', '-safe', '0', '-i', lista, '-vf', 'fps=30,format=yuv420p', '-c:v', 'libx264', '-preset', 'medium', '-crf', '17', '-t', dura.toFixed(3), mp4]);
  console.log(`  ${nombre}: ${marcos.length} cuadros, ${dura.toFixed(1)} s · ` + marcas.map(m => `${m.id} ${(m.fin - m.inicio).toFixed(1)} s${m.sobra < 0 ? ` (¡se pasó ${(-m.sobra).toFixed(1)} s!)` : ''}`).join(' · '));
  return { mp4, dura, marcas };
}

// Una imagen de 1920×1080 a partir de HTML (fondo y bisel de la escena del teléfono).
async function imagen(html, archivo, transparente) { const nav = await chromium.launch({ args: ['--force-device-scale-factor=1.5', '--window-size=1280,720', '--hide-scrollbars'] }), p = await (await nav.newContext({ viewport: null })).newPage();
  await p.goto(url); await p.waitForSelector('#lienzo'); // se parte del portal para tener su tipografía
  await p.evaluate(([css, html, transparente]) => { document.head.querySelectorAll('style').forEach(x => x.remove()); const s = document.createElement('style'); s.textContent = css + ` html, body { margin: 0; width: 1280px; height: 720px; overflow: hidden; display: block; background: ${transparente ? 'transparent' : '#004080'}; }`; document.head.append(s); document.body.innerHTML = `<div class="vz" style="inset:0;color:#fff">${html}</div>`; }, [CSS, html, !!transparente]);
  await p.waitForTimeout(1500); await p.screenshot({ path: archivo, omitBackground: !!transparente }); await nav.close(); }

(async () => {
  if (!clave) { console.error('Falta la contraseña de las cuentas de demostración (segundo argumento o FARO_CLAVE).'); process.exit(2); }
  fs.mkdirSync(CFG.trabajo, { recursive: true });
  const frente = async (A, n, t, para, rotulo) => { await A.vz('sinRotulo'); await A.vz('sinBanda'); await A.tarjeta(T.frente(n, t, para), true); await dormir(1250); return async () => { await A.sinTarjeta(); await A.vz('rotulo', `Frente ${n} · ${t}`, rotulo[0], rotulo[1] || ''); }; };

  // --- Parte A: apertura, el caso de principio a fin y el territorio ---
  console.log('Grabando…');
  const a = await grabar('a', [1280, 720], 1.5, null, [
    ['apertura', async A => { await A.vz('tarjeta', T.apertura()); await dormir(9600); await A.vz('tarjeta', T.marca()); await dormir(2900); }],

    ['caso', async A => { const { p } = A; await A.sinTarjeta(); await A.vz('rotulo', 'Frente 1 · Registros administrativos', CFG.caso[0], CFG.caso[1]);
      await A.entrar(CFG.institucion); await p.waitForSelector('#enviapor:not([hidden])'); await A.sobre('#enviapor', { dx: 260 }); await dormir(1500);
      await A.sobre('#sol'); await p.selectOption('#sol', CFG.solicitud); await dormir(900);
      const arch = CFG.archivo ? { name: CFG.nombreArchivo || path.basename(CFG.archivo), mimeType: 'application/octet-stream', buffer: fs.readFileSync(CFG.archivo) }
        : { name: `ejemplo_ficticio_${CFG.solicitud}.csv`, mimeType: 'text/csv', buffer: Buffer.from(await p.evaluate(s => fetch(`/api/solicitudes/${encodeURIComponent(s)}/plantilla.csv`).then(x => x.text()), CFG.solicitud)) };
      await A.sobre('#archivo', { dx: 70 }); await dormir(250); await p.setInputFiles('#archivo', arch); await dormir(1100);
      const t = Date.now(), respuesta = p.waitForResponse(r => r.url().includes('/api/envios/') && r.request().method() === 'POST', { timeout: 120000 }); await A.clic('#enviar');
      const j = await (await respuesta).json().catch(() => ({})), seg = (Date.now() - t) / 1000; await p.waitForSelector('#resultado:not([hidden])'); await A.vz('sinRotulo');
      // El resultado trae, más abajo, una lista con el valor de algunas filas: en cámara solo se muestran los conteos.
      // La pantalla se acomoda para que esa lista quede debajo de la banda de cifras, que es opaca (lo que se desplaza es <main>).
      await p.evaluate(() => { const r = document.querySelector('#resultado'), corte = r.querySelector('h2 ~ h2, ul'), y = corte ? corte.getBoundingClientRect().top - (innerHeight - 190) : r.getBoundingClientRect().top - 90; document.querySelector('main').scrollBy({ top: y, behavior: 'smooth' }); }); await dormir(900);
      const rep = j.recepcion?.reporte || {}, reconocidas = Object.keys(rep.columnas_reconocidas || {}).length, entran = reconocidas + (rep.columnas_sobrantes || []).length, salen = reconocidas - (j.recepcion?.columnas_eliminadas || []).length;
      await A.vz('banda', `<div><b>${Number(rep.filas || 0).toLocaleString('es-GT')}</b><span>registros validados en ${seg < 1.5 ? 'un segundo' : Math.round(seg) + ' segundos'}</span></div><div><b>${Math.round(j.porcentaje_resuelto ?? 0)} %</b><span>de los lugares con código oficial</span></div>${entran && salen ? `<div><b>${entran} → ${salen}</b><span>columnas: ninguna con nombre ni documento</span></div>` : ''}<div><b>0</b><span>datos personales guardados</span></div>`); await A.sobre('#resultado h2', { ver: false, dx: 300 }); }],
    ['decision', async A => { const { p } = A; await A.vz('sinBanda'); await p.evaluate(() => document.querySelector('main').scrollTo({ top: 0, behavior: 'smooth' })); await dormir(450); await A.clic('#salir', { ver: false }); await p.evaluate(() => document.querySelector('main').scrollTo(0, 0)); await dormir(400); await A.entrar(CFG.analista); await A.clic('nav > button[data-s=pendientes]'); await p.waitForSelector('#tpend tbody tr'); await A.vz('rotulo', 'El sistema no adivina', 'Propone, y una persona decide una sola vez', 'La decisión queda con nombre y fecha, y sirve para todas las instituciones');
      const fila = CFG.pendiente ? p.locator('#tpend tbody tr', { hasText: CFG.pendiente }).first() : p.locator('#tpend tbody tr:has(button[data-c])').first(); await A.sobre(fila.locator('td').first(), { dx: 60 }); await dormir(900); await A.clic(fila.locator('button[data-c]').first()); await dormir(1200); }],
    ['territorio', async A => { const { p } = A; const fin = await frente(A, 2, 'Territorio y censo', 'Censos y Encuestas y las delegaciones', ['Del país al lugar poblado', 'Proyecciones del INE, Censo 2018 y crecimiento urbano desde el satélite']); await p.click('nav > button[data-s=mapa]'); await p.evaluate(() => document.querySelector('main').scrollTo(0, 0)); await dormir(300); await fin();
      await A.escribir('#buscalugar', CFG.buscar, 55); await p.waitForSelector('#buscalista li[data-i]'); await dormir(350); await p.keyboard.press('Enter'); await p.waitForSelector('#ficha svg'); await A.sobre('#ficha svg', { ver: false }); await dormir(2600);
      await p.evaluate(c => detalle(c), CFG.municipio); await p.waitForSelector('#animlienzo'); await A.sobre('#animlienzo', { ver: false }); await dormir(7200); await p.click('#detalle .cerrar'); await p.click('#ficha .cerrar').catch(() => {});
      await A.clic('nav > button[data-s=vial]'); await p.waitForSelector('#vialtarjetas .tarjeta'); await A.vz('rotulo', 'Frente 2 · Territorio y censo', 'La red vial, tratada como un grafo', 'Lugares sin conexión mapeada y enlaces que probablemente faltan'); await A.sobre('#vialtarjetas .tarjeta', { ver: false }); await dormir(1500); await A.ver('#tvialenl', 'center'); }],
  ], async A => { await A.p.evaluate(() => { const t = document.getElementById('vz-tarjeta'); t.style.transition = 'none'; t.classList.add('ver'); t.innerHTML = ''; requestAnimationFrame(() => { t.style.transition = ''; }); }); });

  // --- Parte del teléfono: el trabajo de campo, con la ruta de demostración ---
  const tel = await grabar('telefono', [412, 860], 2, CFG.admin, [
    ['campo', async A => { const { p } = A; await A.vz('dedo'); await A.clic('#menu', { ver: false }); await dormir(350); await A.clic('nav > button[data-s=campo]'); await p.waitForSelector('#campoparadas button[data-visitar]', { timeout: 20000 }); await dormir(1300);
      await A.ver('#campomapa', 'start'); await dormir(1400); const boton = p.locator('#campoparadas button[data-visitar]', { hasText: /^Registrar visita$/ }).first(); await A.clic(boton); await p.waitForSelector('#campovisita[open]'); await dormir(900);
      if (await p.isVisible('#cvsimular')) { await A.clic('#cvsimular', { ver: false }); await dormir(700); } await A.clic('#cvsig', { ver: false }); await dormir(500);
      if (CFG.fotoCampo) { await p.setInputFiles('#cvfoto', CFG.fotoCampo); await p.waitForFunction(() => document.querySelector('#cvvista')?.textContent.trim().length > 3, null, { timeout: 30000 }).catch(() => {}); await dormir(1500); }
      await A.clic('#cvsig', { ver: false }); await dormir(500); await A.clic('#cvregistrar', { ver: false }); await p.waitForFunction(() => document.querySelector('#cvresultado')?.textContent.trim().length > 3, null, { timeout: 30000 }).catch(() => {}); await dormir(600); await A.ver('#cvresultado', 'center'); }],
  ]);

  // --- Parte B: precios, tableros y los pilares ---
  const b = await grabar('b', [1280, 720], 1.5, CFG.analista, [
    ['precios', async A => { const { p } = A; await dormir(1700); await p.click('nav > button[data-s=rubros]'); await p.waitForSelector('#trubros tbody tr td a', { timeout: 30000 }); await p.evaluate(() => document.querySelector('main').scrollTo(0, 0)); await A.sinTarjeta(); await A.vz('rotulo', 'Frente 4 · Precios e índices', 'Lo que las tiendas publican, con su evidencia', 'Cada precio con su dirección y su fecha; reglas fijas, sin interpretar nombres');
      await A.escribir('#rubroq', CFG.producto, 70); await A.clic('#rubrobuscar'); await p.waitForSelector('#trubros tbody tr td a'); await p.waitForFunction(() => !document.querySelector('#rubrovistas .gira'), null, { timeout: 30000 }).catch(() => {}); await A.ver('#rubrovistas', 'start'); await dormir(1200);
      await A.clic('#rubrovistas button[data-v=iguales]', { ver: false }); await dormir(2600); await A.clic('#rubrovistas button[data-v=diferencias]', { ver: false }); await dormir(1200); }],
    ['tableros', async A => { const { p } = A; const fin = await frente(A, 5, 'Tableros y datos abiertos', 'DESA y quien necesita comunicar', ['De una frase a un tablero publicado', 'Sin licencias: lo publicado lo abre cualquiera, con un enlace']); await p.click('nav > button[data-s=tableros]'); await p.waitForSelector('#tabejemplos button'); await p.evaluate(() => document.querySelector('main').scrollTo(0, 0)); await fin();
      await A.escribir('#tabtexto', CFG.frase, 30); await A.clic('#tabarmar'); await p.waitForSelector('#tabvista .pieza svg', { timeout: 20000 }); await dormir(700); await A.ver('#tabvista', 'start'); await dormir(2000); await p.evaluate(() => document.querySelector('main').scrollTo({ top: 0, behavior: 'smooth' })); await dormir(600); await A.clic('#tabguardar', { ver: false }); await dormir(700); await A.clic('#tabpublicar', { ver: false }); await dormir(900); }],
    ['convergencia', async A => { await A.vz('sinRotulo'); await A.tarjeta(T.convergencia()); }],
    ['autonomia', async A => { await A.vz('tarjeta', T.autonomia()); await dormir(11000); await A.vz('tarjeta', T.riesgo()); }],
    ['escala', async A => { await A.vz('tarjeta', T.escala()); }],
    // Mención final: la guía para evaluadores, lo primero del menú, en rojo.
    ['guia', async A => { const { p } = A; await p.click('nav > button[data-s=mejoras]'); await p.waitForSelector('.guia-criterios'); await p.evaluate(() => document.querySelector('main').scrollTo(0, 0)); await A.sinTarjeta(); await A.vz('rotulo', 'Para quien evalúa', 'Guía para evaluadores', 'Lo primero del menú, en rojo: qué revisar en cada criterio y dónde está'); await A.sobre('nav > button[data-s=mejoras]'); }],
    ['cierre', async A => { await A.vz('sinRotulo'); await A.vz('tarjeta', T.cierre()); }],
  ], async A => { await A.vz('tarjeta', T.frente(4, 'Precios e índices', 'Precios e Índices de Referencia, IPP, IPMC y Hoja de Balance'), true); });

  // --- Composición: el teléfono sobre un fondo, y las tres partes con fundidos ---
  console.log('Componiendo…');
  const fondo = path.join(CFG.trabajo, 'fondo.png'), bisel = path.join(CFG.trabajo, 'bisel.png'), X = 830, Y = 34, W = 306, H = 638; // en píxeles de 1280×720
  await imagen(`${HAZ}<div class="vz-c" style="right:500px;padding-right:0;justify-content:center"><div class="vz-k">Frente 3 de 5</div><div class="vz-t">Trabajo de campo</div><div class="vz-s">Para operadores, cartografía y supervisión</div><div class="vz-s" style="font-size:19px;margin-top:26px;color:#fff">Cada operador ve su ruta en el teléfono.<br>Cada visita se comprueba con ubicación y foto.<br>La foto se analiza en el servidor y no se guarda.</div></div><div class="vz-pie" style="max-width:640px;line-height:1.4">Ruta de demostración del propio portal, rotulada como tal.${CFG.creditoFoto ? '<br>' + esc(CFG.creditoFoto) : ''}</div>`, fondo);
  await imagen(`<svg width="1280" height="720" viewBox="0 0 1280 720"><path fill-rule="evenodd" fill="#002b57" stroke="#66b3ff" stroke-width="1.5" d="M${X - 12 + 34} ${Y - 12} h${W + 24 - 68} a34 34 0 0 1 34 34 v${H + 24 - 68} a34 34 0 0 1 -34 34 h-${W + 24 - 68} a34 34 0 0 1 -34 -34 v-${H + 24 - 68} a34 34 0 0 1 34 -34 z M${X + 22} ${Y} h${W - 44} a22 22 0 0 1 22 22 v${H - 44} a22 22 0 0 1 -22 22 h-${W - 44} a22 22 0 0 1 -22 -22 v-${H - 44} a22 22 0 0 1 22 -22 z"/></svg>`, bisel, true);
  const telComp = path.join(CFG.trabajo, 'telefono_compuesto.mp4'), k = 1.5;
  execFileSync('ffmpeg', ['-y', '-loglevel', 'error', '-loop', '1', '-framerate', '30', '-i', fondo, '-i', tel.mp4, '-loop', '1', '-framerate', '30', '-i', bisel, '-filter_complex', `[1:v]scale=${W * k}:${H * k}:flags=lanczos[t];[0:v][t]overlay=${X * k}:${Y * k}[f];[f][2:v]overlay=0:0,format=yuv420p[v]`, '-map', '[v]', '-t', tel.dura.toFixed(3), '-c:v', 'libx264', '-preset', 'medium', '-crf', '17', '-r', '30', telComp]);
  const F = 0.5, total = a.dura + tel.dura + b.dura - 2 * F; fs.mkdirSync(path.dirname(CFG.salida), { recursive: true });
  execFileSync('ffmpeg', ['-y', '-loglevel', 'error', '-i', a.mp4, '-i', telComp, '-i', b.mp4, '-f', 'lavfi', '-i', 'anullsrc=channel_layout=stereo:sample_rate=48000', '-filter_complex', `[0:v][1:v]xfade=transition=fade:duration=${F}:offset=${(a.dura - F).toFixed(3)}[x];[x][2:v]xfade=transition=fade:duration=${F}:offset=${(a.dura + tel.dura - 2 * F).toFixed(3)},format=yuv420p[v]`,
    '-map', '[v]', '-map', '3:a', '-t', total.toFixed(3), '-c:v', 'libx264', '-preset', 'slow', '-crf', '20', '-r', '30', '-pix_fmt', 'yuv420p', '-movflags', '+faststart', '-c:a', 'aac', '-b:a', '96k', CFG.salida]);

  // --- Subtítulos con los tiempos reales, y la línea de tiempo para quien narra ---
  const linea = [...a.marcas, ...tel.marcas.map(m => ({ ...m, inicio: m.inicio + a.dura - F, fin: m.fin + a.dura - F })), ...b.marcas.map(m => ({ ...m, inicio: m.inicio + a.dura + tel.dura - 2 * F, fin: m.fin + a.dura + tel.dura - 2 * F }))];
  const reloj = (s, sep = ',') => { const ms = Math.round(s * 1000), h = Math.floor(ms / 3600000), m = Math.floor(ms % 3600000 / 60000), x = Math.floor(ms % 60000 / 1000); return `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}:${String(x).padStart(2, '0')}${sep}${String(ms % 1000).padStart(3, '0')}`; };
  let n = 0; const srt = linea.flatMap(m => { const palabras = G(m.id).voz.split(/\s+/), trozos = []; for (let i = 0; i < palabras.length; i += 11) trozos.push(palabras.slice(i, i + 11).join(' '));
    const ini = m.inicio + 0.6, voz = Math.min(m.fin - m.inicio - 0.9, palabras.length / 2.35); let t = ini; return trozos.map(x => { const d = voz * x.split(' ').length / palabras.length, r = `${++n}\n${reloj(t)} --> ${reloj(t + d - 0.05)}\n${x}\n`; t += d; return r; }); }).join('\n');
  fs.writeFileSync(CFG.salida.replace(/\.mp4$/, '.srt'), srt);
  fs.writeFileSync(CFG.salida.replace(/\.mp4$/, '-tiempos.json'), JSON.stringify(linea.map(m => ({ escena: m.id, inicio: +m.inicio.toFixed(2), fin: +m.fin.toFixed(2), palabras: G(m.id).voz.split(/\s+/).length, voz: G(m.id).voz })), null, 1) + '\n');
  console.log(`\n${reloj(total, '.').slice(3, 8)} de video → ${CFG.salida} (${(fs.statSync(CFG.salida).size / 1048576).toFixed(1)} MB)`);
  for (const m of linea) console.log(`  ${reloj(m.inicio, '.').slice(3, 10)}–${reloj(m.fin, '.').slice(3, 10)}  ${m.id.padEnd(13)} ${G(m.id).voz.split(/\s+/).length} palabras`);
})().catch(e => { console.error(e); process.exit(1); });
