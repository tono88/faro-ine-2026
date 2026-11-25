// Genera la presentación para el jurado en dos formatos, con el mismo contenido: el PDF que pide la nota
// conceptual (entregables/FARO-presentacion.pdf) y un PowerPoint editable (FARO-presentacion.pptx, con textos,
// tablas y formas nativos; necesita la biblioteca pptxgenjs: si no está, se genera solo el PDF). Diapositivas
// 16:9 con capturas tomadas en el momento del portal en marcha. La diapositiva de problemas sale de la tabla
// «Los problemas que atiende» del README. La nota conceptual pide una presentación en PDF para la
// exposición de 7 minutos; el guion hablado está en docs/17-presentacion-7-minutos.md.
// Las cifras salen de datos/estandar/mejoras.json (las mismas de «Antes y ahora»), de fuentes.json y del portal.
// Requiere Node y Playwright con Chromium. Para capturas sin datos reales: scripts/entregables/preparar_demo.js.
//
// Uso: node scripts/entregables/generar_presentacion.js <url> <usuario administrador> <contraseña>
const { chromium } = require('playwright'); const fs = require('fs'); const path = require('path'); const { execSync } = require('child_process');
const INE = require('./_tipografia.js');
const [base = 'http://localhost:8082', usuario = 'admin', clave = process.env.FARO_CLAVE || ''] = process.argv.slice(2);
const raiz = path.resolve(__dirname, '../..'), salida = path.join(raiz, 'entregables', 'FARO-presentacion.pdf');
const esc = t => String(t ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const leer = f => fs.readFileSync(path.join(raiz, f), 'utf8'); const num = n => typeof n === 'number' ? n.toLocaleString('es-GT') : n;
const icono = lado => `<svg viewBox="0 0 64 64" width="${lado}" height="${lado}" aria-hidden="true"><rect width="64" height="64" rx="14" fill="#0080ff"/><path d="M26 21 5 12v17z" fill="#ffd500" opacity=".9"/><path d="M38 21 59 12v17z" fill="#ffd500" opacity=".9"/><path d="M32 8l8 8H24z" fill="#fff"/><rect x="26" y="16" width="12" height="8" rx="1" fill="#ffe566"/><path d="M27 26h10l4 28H23z" fill="#fff"/><path d="M26.1 34h11.8l.9 6H25.2z" fill="#66b3ff"/><path d="M24.4 46h15.2l.6 4H23.8z" fill="#66b3ff"/><rect x="18" y="54" width="28" height="4" rx="2" fill="#fff"/></svg>`;
const haz = `<svg class="haz" viewBox="0 0 1000 562" preserveAspectRatio="none" aria-hidden="true"><defs><linearGradient id="h1" x1="1" x2="0"><stop offset="0" stop-color="#ffd500" stop-opacity=".55"/><stop offset="1" stop-color="#ffd500" stop-opacity="0"/></linearGradient></defs><path d="M780 215 L0 10 L0 250 Z" fill="url(#h1)"/><path d="M780 215 L0 290 L0 420 Z" fill="url(#h1)" opacity=".45"/><g fill="none" stroke="#66b3ff" stroke-width="2" opacity=".35"><path d="M0 505 q60 -20 120 0 t120 0 t120 0 t120 0 t120 0 t120 0 t120 0 t120 0 t120 0"/><path d="M-40 530 q60 -20 120 0 t120 0 t120 0 t120 0 t120 0 t120 0 t120 0 t120 0 t120 0" opacity=".7"/><path d="M0 552 q60 -20 120 0 t120 0 t120 0 t120 0 t120 0 t120 0 t120 0 t120 0 t120 0" opacity=".45"/></g></svg>`;

(async () => {
  if (!clave) { console.error('Falta la contraseña del administrador (tercer argumento o FARO_CLAVE).'); process.exit(2); }
  const proxy = process.env.HTTPS_PROXY; const nav = await chromium.launch(proxy ? { proxy: { server: proxy, bypass: 'localhost,127.0.0.1' } } : {});
  const ctx = await nav.newContext({ viewport: { width: 1366, height: 800 }, deviceScaleFactor: 1.5 }); const p = await ctx.newPage(); p.on('dialog', d => d.accept());
  const abrir = async () => { await p.goto(base); await p.waitForSelector('#lienzo path.leaflet-interactive'); await p.waitForTimeout(700); };
  await abrir(); const r = await p.evaluate(async ([u, c]) => (await fetch('/api/sesion', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ usuario: u, clave: c }) })).status, [usuario, clave]);
  if (r !== 200) { console.error(`No se pudo entrar como ${usuario} (${r}).`); process.exit(1); } await abrir();
  const tomas = {}; const foto = async (n, pagina = p) => { tomas[n] = 'data:image/jpeg;base64,' + (await pagina.screenshot({ type: 'jpeg', quality: 86 })).toString('base64'); };
  // Cada captura va en su propio intento: si una pantalla no está disponible, esa diapositiva sale sin imagen y el resto sigue.
  const toma = async (n, pasos) => { try { await pasos(); await foto(n); process.stdout.write(n + ' '); } catch (e) { console.log(`\n  (sin captura «${n}»: ${String(e.message).split('\n')[0]})`); } };
  const ir = async s => { await p.click(`nav > button[data-s=${s}]`); await p.waitForTimeout(1100); await p.evaluate(() => scrollTo(0, 0)); };
  await toma('recibir', async () => { await ir('recibir'); await p.waitForSelector('#envios tbody tr'); await p.evaluate(() => document.querySelector('#envios').scrollIntoView({ block: 'center' })); await p.waitForTimeout(300); });
  await toma('pendientes', async () => { await ir('pendientes'); await p.waitForTimeout(600); });
  await toma('ficha', async () => { await ir('mapa'); await p.fill('#buscalugar', 'panajachel'); await p.waitForSelector('#buscalista li[data-i]'); await p.keyboard.press('Enter'); await p.waitForSelector('#ficha svg'); await p.waitForTimeout(900); });
  await p.click('#ficha .cerrar').catch(() => {});
  await toma('animacion', async () => { await p.evaluate(() => detalle('0101')); await p.waitForSelector('#animlienzo'); await p.waitForFunction(() => /Ver de nuevo/.test(document.querySelector('#animplay').textContent), null, { timeout: 45000 }); await p.waitForTimeout(1200); });
  await p.click('#detalle .cerrar').catch(() => {});
  await toma('vial', async () => { await ir('vial'); await p.waitForSelector('#tvialenl tbody tr'); });
  await toma('operadores', async () => { await ir('operadores'); });
  await toma('rubros', async () => { await ir('rubros'); await p.waitForSelector('#trubros tbody tr td a', { timeout: 30000 }); await p.waitForFunction(() => !document.querySelector('#rubrovistas .gira'), null, { timeout: 30000 }); await p.click('#rubrovistas button[data-v=iguales]'); await p.waitForTimeout(700); await p.evaluate(() => document.querySelector('#rubrovistas').scrollIntoView({ block: 'start' })); await p.evaluate(() => scrollBy(0, -330)); await p.waitForTimeout(200); });
  await toma('tablero', async () => { await ir('tableros'); await p.waitForSelector('#tabejemplos button'); await p.click('#tabejemplos button'); await p.waitForSelector('#tabvista .pieza svg', { timeout: 20000 }); await p.waitForTimeout(900); });
  await toma('mejoras', async () => { await ir('mejoras'); await p.waitForSelector('.mejora'); });
  await toma('recorrido', async () => { await ir('recibir'); await p.click('#recorrido'); await p.click('#recglobo [data-r=sig]'); await p.waitForTimeout(400); });
  await p.keyboard.press('Escape');
  // El teléfono del operador: «Mi ruta de campo» con la ruta de demostración, si está cargada.
  const tel = await (await nav.newContext({ viewport: { width: 390, height: 780 }, deviceScaleFactor: 2, storageState: await ctx.storageState() })).newPage();
  try { await tel.goto(base); await tel.waitForSelector('#lienzo path.leaflet-interactive'); await tel.click('#menu'); await tel.click('nav > button[data-s=campo]'); await tel.waitForTimeout(1800); await tel.evaluate(() => scrollTo(0, 0)); await foto('telefono', tel); process.stdout.write('telefono '); } catch (e) { console.log('\n  (sin captura «telefono»)'); }
  const vivo = await p.evaluate(async () => { const j = u => fetch(u).then(x => x.json()).catch(() => null); const e = await j('/api/estado'), c = await j('/capas'); return { lugares: e?.lugares_en_catalogo, umbral: e?.umbral_de_celda ?? 3, capas: (c?.capas || []).length }; });
  console.log('\ncapturas listas');

  // Logotipo y fondo de las diapositivas oscuras, como imágenes para el PowerPoint.
  const lienzo = await ctx.newPage();
  const png = async (cuerpo, ancho, alto, transparente) => { await lienzo.setViewportSize({ width: ancho, height: alto }); await lienzo.setContent(`<body style="margin:0;background:${transparente ? 'transparent' : '#004080'}">${cuerpo}</body>`); return 'image/png;base64,' + (await lienzo.screenshot({ omitBackground: !!transparente })).toString('base64'); };
  const inePng = await png(`<img src="${INE.ine('ine-horizontal-blanco')}" style="width:1236px;height:400px;display:block">`, 1236, 400, true);
  const logoPng = await png(icono(256), 256, 256, true), fondoPng = await png(haz.replace('class="haz"', 'style="width:1280px;height:720px;display:block"'), 1280, 720, false);
  await lienzo.close();

  // ---------- Contenido: una sola lista de diapositivas, de la que salen el PDF y el PowerPoint ----------
  const mejoras = JSON.parse(leer('datos/estandar/mejoras.json')).puntos, M = tema => mejoras.find(m => m.tema === tema) || { cifra: '—', cifra_de: '' };
  const fuentes = JSON.parse(leer('datos/estandar/fuentes.json')).fuentes;
  const version = (leer('docs/08-versiones.md').match(/## (v[\d.]+)/) || [])[1] || '';
  const git = c => { try { return execSync(c, { cwd: raiz }).toString().trim(); } catch (_) { return ''; } };
  const commits = git('git rev-list --count HEAD'), primero = git("git log --reverse --format=%ad --date=format:%d-%m-%Y | head -1");
  const pruebas = (() => { let n = 0; const rec = d => fs.readdirSync(d, { withFileTypes: true }).forEach(x => { const r = path.join(d, x.name); if (x.isDirectory()) rec(r); else if (x.name.endsWith('.rs')) n += (fs.readFileSync(r, 'utf8').match(/#\[test\]/g) || []).length; }); rec(path.join(raiz, 'crates')); return n; })();
  // Los problemas salen de la tabla «Los problemas que atiende» del README: si cambia ahí, cambia aquí.
  const problemas = ((leer('README.md').split('## Los problemas que atiende')[1] || '').split('\n## ')[0].match(/^\| \*\*.*$/gm) || []).map(l => { const c = l.split('|').map(x => x.trim()); return { t: c[1].replace(/\*\*/g, ''), quien: c[2], hoy: c[3].split(/\. (?=[A-ZÁÉÍÓÚ«"])/)[0].replace(/\.$/, '') + '.' }; });
  const cifraDe = (tema, texto) => [M(tema).cifra, texto];
  const D = [
    { tipo: 'portada', seccion: 'Apertura', notas: 'Nombre, lema y equipo. «FARO es una herramienta para que el INE pida, reciba y homologue los registros de otras instituciones, y los aproveche junto al censo, el territorio y los precios.»' },
    { tipo: 'problemas', seccion: 'Apertura', crit: 'Aplicabilidad institucional', ante: 'Los problemas que atiende, con las palabras del INE', titulo: 'Datos que existen, pero no llegan en forma útil', bajada: 'Diez necesidades planteadas por el INE o halladas en los archivos reales. Todas tienen la misma raíz, y FARO las atiende una por una.', items: problemas,
      pie: 'Un ejemplo del primero: el mismo municipio llega como «Puerto de San José», «El puerto», «PTO. SAN JOSE» o «San José» (que existe en Escuintla y en Petén). Estado de cada necesidad: docs/01.',
      notas: 'No es un solo problema: son diez, y todos tienen la misma raíz. Los datos ya existen, pero no llegan en una forma que se pueda usar. Nombrar tres o cuatro y decir quién los planteó. Ejemplo rápido: el mismo municipio llega escrito de cuatro maneras.' },
    { tipo: 'tarjetas', seccion: 'Apertura', crit: 'Aplicabilidad institucional', ante: 'Cinco frentes, una raíz', titulo: 'Cinco frentes que convergen en una solución', bajada: 'Los procesos del INE son un todo. Atacar el problema desde varios frentes los hace converger en una sola herramienta, con un solo catálogo y una sola memoria.', items: [
      ['1 · Registros administrativos', `Para la Coordinación del SEN y las instituciones. Pedir, recibir y homologar sin adivinar: ${M('Un estándar para pedir y recibir datos').cifra} registros reales.`],
      ['2 · Territorio y censo', `Para Censos y Encuestas y las delegaciones. Del país al lugar poblado: ${num(vivo.lugares) || '20,036'} lugares, crecimiento urbano y red vial.`],
      ['3 · Trabajo de campo', 'Para operadores, cartografía y supervisión. Rutas ordenadas y visitas comprobadas con posición y foto.'],
      ['4 · Precios e índices', `Para Precios e Índices de Referencia, IPP, IPMC y HBA. ${M('Informantes, productos y precios').cifra} precios con su evidencia.`],
      ['5 · Tableros y datos abiertos', `Para DESA y quien comunica. Tableros desde una frase y ${num(vivo.capas) || 21} capas para descargar.`],
      ['Una sola herramienta', 'Un catálogo oficial, una memoria de decisiones, un mapa y una bitácora para los cinco frentes. Se instala una vez.']],
      notas: 'Cada dirección del INE planteó una necesidad distinta. No hicimos cinco herramientas: hicimos una, porque comparten el catálogo de lugares, la memoria de decisiones y las reglas de protección. Nombrar a cada dirección al señalar su frente.' },
    { tipo: 'verbos', seccion: 'Apertura', crit: 'Innovación', ante: 'La respuesta', titulo: 'Un solo camino para el dato, y una regla fija', items: [['Pide', 'Una solicitud declarativa dice qué columnas, qué catálogos y qué es dato personal. Agregar una institución es escribir un archivo, sin programar.'], ['Recibe', 'El Excel o CSV llega como sale de cada sistema. En segundos queda validado; el original no se guarda.'], ['Homologa', 'Código oficial solo con coincidencia exacta y única. Lo dudoso se propone, lo decide una persona una vez y queda aprendido.'], ['Aprovecha', 'Registros, censo, territorio y precios en un mismo mapa, en tableros publicables y en capas para cualquier SIG.']],
      regla: ['El sistema no adivina.', 'Y no inventa: cada dato tiene fuente, fecha y licencia; cada resultado se puede repetir con un comando.'], notas: 'Pide, recibe, homologa, aprovecha. La regla que no se rompe: el sistema no adivina. Lo dudoso lo propone y lo decide una persona, una sola vez.' },
    { tipo: 'demo', seccion: 'Demostración', n: 1, crit: 'Funcionalidad', titulo: 'De un archivo cualquiera a un dato oficial', bajada: 'Para la Coordinación del SEN y las instituciones que envían. Cada una entra con su cuenta y sube solo lo suyo; lo dudoso se propone.', caps: ['recibir', 'pendientes'],
      puntos: ['Columnas reconocidas aunque cambien de nombre u orden.', 'Nombres y documentos de identidad se eliminan o se sustituyen por una llave.', 'Una decisión sirve para todos los envíos que vengan, de cualquier institución.'],
      cifras: [cifraDe('Un estándar para pedir y recibir datos', 'filas reales de RENAP, PNC y SE-CONRED validadas en segundos'), cifraDe('Lugares escritos de mil maneras', 'de los lugares de RENAP y PNC con código oficial; 99.9 % en SE-CONRED'), cifraDe('Datos personales que no deben viajar', 'columnas en defunciones de RENAP: ninguna con nombre ni CUI')],
      notas: 'EN VIVO: entrar como enlace de PNC, subir el archivo de ejemplo, ver el resultado. Entrar como analista, decidir un pendiente. Si el portal falla, narrar sobre estas capturas.' },
    { tipo: 'demo', seccion: 'Demostración', n: 2, crit: 'Funcionalidad', titulo: 'El país, hasta el lugar poblado', bajada: 'Para Censos y Encuestas y las delegaciones. Población del INE, Censo 2018, crecimiento urbano desde el satélite y la red vial como un grafo.', caps: ['ficha', 'animacion'],
      puntos: ['Ficha de cada municipio con censo, edades y un reporte para imprimir.', 'Crecimiento urbano de 1975 a 2020 sobre la foto satelital de cada época.', 'Lugares sin conexión y enlaces que probablemente faltan en el mapa.'],
      cifras: [[num(vivo.lugares) || '20,036', 'lugares en el catálogo oficial, verificados uno a uno contra el censo'], cifraDe('Caminos que faltan en el mapa', 'lugares poblados sin conexión vial mapeada; 83 enlaces probables'), cifraDe('Errores en fuentes oficiales, detectados solos', 'códigos de municipio corregidos en fuentes oficiales, con la razón')],
      notas: 'EN VIVO: buscar Panajachel, abrir la ficha; animación del crecimiento de Guatemala; red vial, ver un enlace en el mapa.' },
    { tipo: 'demo', seccion: 'Demostración', n: 3, crit: 'Aplicabilidad institucional', titulo: 'Del escritorio al campo', bajada: 'Para operadores, cartografía y supervisión. Un asistente arma la ruta; en el teléfono, cada visita se comprueba con posición y foto.', caps: ['operadores', 'telefono'], movil: true,
      puntos: ['Asignación que minimiza la distancia total (método húngaro).', 'La foto de la fachada se analiza en el servidor y se descarta.', 'La encuesta del INE se abre con el lugar ya escrito.'],
      cifras: [cifraDe('Operadores cerca de su puesto', 'recorrido total en el archivo de muestra (ficticio)'), cifraDe('Ruta, posición y encuesta en el teléfono', 'evidencias por visita: posición, foto, satélite y censo'), cifraDe('La foto confirma que es una vivienda', 'fachadas acertadas, 0 errores, 5 no concluyentes (prueba de humo)')],
      notas: 'EN VIVO, en ventana de teléfono: «Mi ruta de campo» con la ruta de demostración; registrar una visita paso a paso.' },
    { tipo: 'demo', seccion: 'Demostración', n: 4, crit: 'Funcionalidad', titulo: 'Precios con evidencia', bajada: 'Para Precios e Índices de Referencia, IPP, IPMC y Hoja de Balance. Lo que las tiendas publican, leído con reglas fijas y con su evidencia.', caps: ['rubros'],
      puntos: ['El código de barras compara el mismo producto sin interpretar nombres.', 'Diferencias de 50 % o más: una regla dice si es oferta o atípico; lo demás lo confirma una persona.', 'De una búsqueda sale el informe de precio de referencia, listo para imprimir.'],
      cifras: [cifraDe('Informantes, productos y precios', 'precios publicados por 18 tiendas en seis rubros'), cifraDe('El mismo producto entre tiendas', 'productos comparables por código de barras'), cifraDe('Derivados de la Hoja de Balance', 'tasas de la FAO coinciden con las cuentas del INE')],
      notas: 'EN VIVO: buscar «arroz», ver el mismo producto en varias tiendas, las diferencias de 50 % o más y el informe.' },
    { tipo: 'demo', seccion: 'Demostración', n: 5, crit: 'Presentación y usabilidad', titulo: 'Tableros y un portal que se explica solo', bajada: 'Para DESA y quien necesita comunicar. Una frase produce un tablero que se publica con un enlace; cada pantalla tiene su recorrido guiado.', caps: ['tablero', 'recorrido'],
      puntos: ['Los tableros publicados los abre cualquiera, sin cuenta.', 'Botón «Recorrido guiado» siempre a la vista, en las 24 secciones.', 'Manual gráfico generado desde el portal; funciona en teléfono y se lee también en inglés.'],
      cifras: [cifraDe('Tableros sin licencias', 'indicadores y 8 tipos de pieza'), [String(num(vivo.capas) || M('Resultados que otros pueden usar y repetir').cifra), 'capas descargables en GeoJSON y CSV'], ['6 × 24', 'tamaños de pantalla por secciones, sin hallazgos']],
      notas: 'EN VIVO: escribir una frase, armar el tablero, publicarlo y abrir el enlace sin sesión. Pulsar «Recorrido guiado».' },
    { tipo: 'tabla', seccion: 'Por qué FARO', crit: 'Aplicabilidad institucional', ante: 'Una decisión de ingeniería', titulo: 'Autocontenido: el riesgo que el INE no corre', bajada: 'Software libre en un servidor del INE: sin licencias, sin servicios de pago y sin llamadas a una inteligencia artificial externa para funcionar.', filas: [
      ['Si mañana cambia el precio', 'Con un servicio externo, el costo crece con cada consulta. Con FARO no hay pago por uso: Q 0 en licencias'],
      ['Si el servicio o el modelo se retira', 'Una herramienta que depende de él deja de funcionar. FARO sigue igual: todo está en el servidor'],
      ['Si hay que explicar un resultado', 'Reglas fijas y a la vista: el mismo dato da siempre el mismo resultado, con constancia de cada paso'],
      ['Si los datos no pueden salir', 'No viajan a un tercero: se quedan en la institución. El archivo original ni siquiera se guarda'],
      ['Si cambia el equipo que lo mantiene', 'Código abierto y documentado, con licencia perpetua para el INE: cualquiera puede darle soporte']],
      pie: 'Lo externo es opcional y gratuito (mapa base, foto satelital, tipografía): sin internet, el portal funciona con los datos que trae.',
      notas: 'No atacar a nadie: las API de inteligencia artificial son útiles. Nuestra elección es que el INE pueda repetir el mismo resultado dentro de cinco años sin depender de un tercero. SI PREGUNTAN POR LA IA: sí usamos un asistente de IA para programar, y está declarado en el README; lo que no depende de ninguna IA externa es el portal en funcionamiento. La única pieza entrenada es el clasificador de fotos, que corre en el propio servidor y no decide.' },
    { tipo: 'tarjetas', seccion: 'Por qué FARO', crit: 'Calidad técnica', ante: 'Configurable y escalable', titulo: 'Crece sin programar', items: [
      ['Una institución nueva', 'Es un archivo de solicitud: qué columnas, qué catálogos y qué es dato personal. Hoy hay cuatro; la quinta no pide código.'],
      ['Una tienda o un rubro nuevo', 'Es una tarea más en una lista: dirección, rubro y términos. Hoy son 26 tareas de 18 tiendas.'],
      ['Las reglas son parámetros', 'Umbral de celdas protegidas, cuándo un precio es extremo, a qué distancia una visita está lejos: los fija el INE.'],
      ['Cada quien ve lo suyo', 'Cinco roles, grupos por sección y una cuenta por institución. Sumar una delegación es crear un grupo.'],
      ['De una laptop a un servidor', 'El mismo comando lo instala en ambos. El contrato de datos es el mismo si mañana se recibe por X-Road.'],
      ['Todo se regenera', 'Cada tabla y cada documento sale de un comando. Una fuente nueva queda anotada con su fecha y su huella.']],
      notas: 'Escalar no es reescribir: es agregar archivos de configuración. Mostrar, si hay tiempo, el archivo de solicitud de una institución.' },
    { tipo: 'tarjetas', seccion: 'Por qué FARO', crit: 'Rigor en el uso de los datos', ante: 'Comprobado, no prometido', titulo: 'Se revisa a sí mismo y dice sus límites', items: [
      ['El INE es la fuente principal', `Censo 2018, proyecciones de población 2015–2030, IPMC, precios de referencia, canasta básica y Hoja de Balance 2024. ${fuentes.length} fuentes citadas con fecha de descarga y licencia.`],
      ['Secreto estadístico', `Solo se publican conteos. Los menores que ${vivo.umbral} se ocultan a quien no es analista, con supresión complementaria. El archivo original no se guarda.`],
      ['Verificación estadística', 'Los márgenes de proyección se probaron contra lo ocurrido en 1,020 casos. El inicial acertaba 45 % cuando prometía 95 %: se corrigió.'],
      ['Nada pasa por oficial si no lo es', 'Las curvas de población después de 2030, los precios publicados y el clasificador de fotos están rotulados como cálculo propio.'],
      ['Seguridad auditada', 'Contraseñas con Argon2, roles, grupos y cuentas por institución; bitácora de quién hizo qué. Lo pendiente está escrito.'],
      ['Conformidad legal analizada', 'Ley del INE, acceso a la información, RENAP, delitos informáticos y licencias, norma por norma. No es un dictamen jurídico.']],
      notas: 'El INE es la fuente principal. Solo conteos, con las celdas pequeñas protegidas. FARO se midió a sí mismo y corrigió su margen de proyección. Lo que no es oficial está rotulado.' },
    { tipo: 'antes', seccion: 'Cierre', crit: 'Aplicabilidad institucional', ante: 'Antes y ahora', titulo: `${mejoras.length} mejoras, cada una con su cifra`, bajada: 'Nuestra propuesta: un piloto con una institución, en un servidor del INE, durante un mes. Todo lo necesario ya está en el repositorio.', items: ['Un estándar para pedir y recibir datos', 'Datos personales que no deben viajar', 'Crecimiento urbano desde el satélite', 'El censo al alcance de cada delegación', 'Informantes, productos y precios', 'Sin depender de terceros'].map(t => ({ tema: t, cifra: M(t).cifra, de: M(t).cifra_de })),
      adoptar: ['un servidor con Docker', 'escribir sus solicitudes', 'crear sus usuarios'], notas: 'La sección «Antes y ahora» del portal trae cada punto con su cifra. LA PETICIÓN: un piloto acotado, con una institución, en un servidor del INE, durante un mes. Para adoptarlo, el INE necesita un servidor con Docker, escribir sus solicitudes y crear sus usuarios.' },
    { tipo: 'cierre', seccion: 'Cierre', notas: 'MENCIÓN FINAL DE INCLUSIÓN (una frase): la pantalla también se lee en inglés y está preparada para los idiomas nacionales; el kaqchikel ya cubre más de un tercio de los rótulos como BORRADOR, armado solo con vocabulario escrito por hablantes (Mozilla y la Academia de Lenguas Mayas); falta que lo revise una persona experta, y así lo dice la propia pantalla. Se traduce la interfaz, nunca los datos. Luego cerrar con el lema: «Que cada dato llegue a puerto». Preguntas: ver docs/17, docs/28 y docs/29.' },
    { tipo: 'tabla', seccion: 'Apéndice para las preguntas', crit: 'Calidad técnica', ante: 'Lo que pide la nota conceptual', titulo: 'Cumple cada lineamiento, con su prueba', filas: [
      ['Datos del INE como fuente principal', 'Siete conjuntos del portal de datos abiertos del INE; las demás fuentes, abiertas y citadas con fecha de descarga (`docs/11`)'],
      ['Código abierto, sin costo de licencias', 'Rust, PostgreSQL y Leaflet. Ningún servicio de pago. Componentes y licencias en `docs/27`'],
      ['Desarrollado durante el evento', `Más de ${Math.floor((+commits - 1) / 5) * 5} cambios en el repositorio desde el ${primero}; el historial es la evidencia`],
      ['Prototipo funcional y reproducible', `Se levanta con un comando (\`docker compose up\`); ${pruebas} pruebas automáticas y pruebas de pantalla con clic real`],
      ['Documentación', 'README con problema, usuarios, arquitectura, instalación, datos, limitaciones y mejoras; manual gráfico y documento técnico'],
      ['Secreto estadístico y sin credenciales', 'Solo conteos, con las celdas pequeñas protegidas; las claves las pone cada instalación y no están en el código'],
      ['Inteligencia artificial declarada', 'Asistente de IA para programar, declarado. En el portal, reglas fijas; un solo modelo, local, que nunca decide'],
      ['Propiedad intelectual', 'Autoría del equipo y licencia perpetua al INE, por escrito en `LICENCIA.md`']],
      pie: 'Matriz completa, lineamiento por lineamiento y criterio por criterio: docs/26-conformidad-nota-conceptual.md.', notas: 'Cada lineamiento de la nota conceptual tiene su prueba en el repositorio. La matriz completa está en docs/26.' },
    { tipo: 'dos', seccion: 'Apéndice para las preguntas', crit: 'Calidad técnica', ante: 'Con honestidad', titulo: 'Lo que todavía no hace, y lo que sigue',
      izq: ['Limitaciones', ['Los umbrales (parecido, precio extremo, celda protegida) son parámetros: debe fijarlos el INE.', 'Distancias en línea recta, no en tiempo de viaje.', 'El clasificador de fotos se probó con 16 fotos: falta medirlo con fotos de campo.', 'Reglas del IPMC para 42 de 89 artículos.', 'La instalación en Windows no se ha probado.']],
      der: ['Posibles mejoras', ['Recibir los mismos envíos por X-Road, sin archivos.', 'Publicar con HTTPS y firmar las constancias.', 'Históricos de Guatecompras para precios de referencia.', 'Más registros: MINEDUC, MSPAS, SAT, INAB, SEGEPLAN.', 'Control de divulgación entre tablas y codificación de texto libre.']],
      notas: 'Decir las limitaciones antes de que las pregunten, y lo que sigue.' },
  ];
  const EQUIPO = 'Estuardo Sandoval · Ana Echeverría · Alejandro Sandoval', REPO = 'github.com/tono88/faro-ine-2026', LEMA = 'Que cada dato llegue a puerto.';
const INCLUSION = 'También en inglés · kaqchikel en borrador, por revisar con hablantes · se traduce la pantalla, nunca los datos';
  const DESC = 'Una herramienta para que el INE pida, reciba y homologue los registros de otras instituciones, y los aproveche junto al censo, el territorio y los precios.';

  // ---------- PDF ----------
  const conCodigo = t => esc(t).replace(/`([^`]+)`/g, '<code>$1</code>');
  const img = (n, clase = '') => tomas[n] ? `<div class="cap ${clase}"><img src="${tomas[n]}"></div>` : `<div class="cap vacia ${clase}">Demostración en vivo</div>`;
  const cabeza = d => `<header><p class="ante">${esc(d.ante || `Demostración ${d.n}`)}</p><h2>${esc(d.titulo)}</h2>${d.bajada ? `<p class="bajada">${esc(d.bajada)}</p>` : ''}</header>`;
  const cuerpo = d => ({
    portada: () => `${haz}<div class="torre">${icono(300)}</div><div class="texto"><p class="sobre">Datatón INE 2026 · Categoría 2: Tecnológica y ciencias de datos</p><h1>FARO</h1><p class="lema">${LEMA}</p><p class="desc">${DESC}</p><p class="equipo">Equipo <b>La Sociedad de Almas</b> · ${EQUIPO}</p></div>`,
    problemas: () => `${cabeza(d)}<ol class="probs">${d.items.map((x, i) => `<li><b class="num">${i + 1}</b><div><h4>${esc(x.t)}</h4><p>${esc(x.hoy)} <span>${esc(x.quien)}</span></p></div></li>`).join('')}</ol><p class="pie-nota">${conCodigo(d.pie.replace('docs/01', '`docs/01`'))}</p>`,
    verbos: () => `${cabeza(d)}<div class="verbos">${d.items.map(([v, t], i) => `<div class="verbo${i === 3 ? ' fin' : ''}"><b>${v}</b><p>${t}</p></div>`).join('<i>→</i>')}</div><div class="regla"><b>${d.regla[0]}</b> ${d.regla[1]}</div>`,
    demo: () => `${cabeza(d)}<div class="demo"><div class="caps">${img(d.caps[0], d.caps.length === 1 ? 'ancha' : '')}${d.caps[1] ? img(d.caps[1], d.movil ? 'movil' : '') : ''}</div><div class="lado"><ul class="puntos">${d.puntos.map(x => `<li>${esc(x)}</li>`).join('')}</ul><div class="cifras">${d.cifras.map(([v, t]) => `<div class="cifra"><b>${esc(v)}</b><span>${esc(t)}</span></div>`).join('')}</div></div></div>`,
    tarjetas: () => `${cabeza(d)}<div class="tarjs">${d.items.map(([t, x]) => `<div class="tarj"><h3>${esc(t)}</h3><p>${esc(x)}</p></div>`).join('')}</div>`,
    tabla: () => `${cabeza(d)}<table class="cumple"><tbody>${d.filas.map(([t, x]) => `<tr><td class="si">✓</td><td><b>${esc(t)}</b></td><td>${conCodigo(x)}</td></tr>`).join('')}</tbody></table><p class="pie-nota">${esc(d.pie)}</p>`,
    antes: () => `${cabeza(d)}<div class="antes">${d.items.map(m => `<div class="ant"><b>${esc(m.cifra)}</b><h4>${esc(m.tema)}</h4><p>${esc(m.de)}</p></div>`).join('')}</div><div class="adoptar"><b>Para adoptarlo, el INE necesita:</b>${d.adoptar.map(x => `<span>${esc(x)}</span>`).join('')}</div>`,
    dos: () => `${cabeza(d)}<div class="dos">${[d.izq, d.der].map(([t, l]) => `<div><h3>${esc(t)}</h3><ul class="lista">${l.map(x => `<li>${esc(x)}</li>`).join('')}</ul></div>`).join('')}</div>`,
    cierre: () => `${haz}<div class="centro">${icono(110)}<h2>FARO</h2><p class="lema">${LEMA}</p><p class="repo">${REPO} · ${esc(version)}</p><p class="equipo">La Sociedad de Almas · ${EQUIPO}</p><p class="inclusion">${INCLUSION}</p><p class="gracias">Gracias. ¿Preguntas?</p></div>`,
  })[d.tipo]();
  const html = `<!doctype html><html lang="es"><head><meta charset="utf-8"><title>FARO · Presentación</title>
  <style>${require('./_tipografia.js').CSS}</style><style>
  @page { size: 338.67mm 190.5mm; margin: 0; } * { box-sizing: border-box; } html { -webkit-print-color-adjust: exact; print-color-adjust: exact; }
  body { margin: 0; font: 13pt/1.42 "Ubuntu Sans", "DejaVu Sans", Arial, sans-serif; color: #313131; }
  .d { width: 338.67mm; height: 190.5mm; position: relative; overflow: hidden; break-after: page; padding: 13mm 18mm 15mm 24mm; background: #fff; }
  .d > .lomo { position: absolute; left: 0; top: 0; bottom: 0; width: 9mm; background: #0080ff; } .d > .lomo::after { content: ""; position: absolute; left: 0; right: 0; top: 0; height: 30mm; background: #ffd500; }
  .d > /* Cintillo inferior de la guía de diseño del INE: su logotipo en la esquina inferior derecha, en un recuadro con la esquina superior izquierda curva. */
  .d .ine { position: absolute; right: 0; bottom: 0; background: #004080; border-top-left-radius: 6mm; padding: 3mm 6mm 3mm 5mm; } .d .ine img { height: 8.5mm; display: block; } .d.oscura .ine { background: #0080ff; }
  footer { position: absolute; left: 24mm; right: 50mm; bottom: 5.5mm; display: flex; align-items: center; gap: 3mm; font-size: 9pt; color: #5b6672; } footer b { color: #004080; letter-spacing: 1px; } footer .crit { margin-left: auto; background: #e5f2ff; color: #004080; border-radius: 99px; padding: .6mm 3.5mm; font-weight: 600; font-size: 8.5pt; } footer .n { font-weight: 700; color: #004080; font-size: 10.5pt; margin-left: 4mm; }
  h2 { font-size: 27pt; line-height: 1.12; color: #004080; margin: 0; font-weight: 800; letter-spacing: -.3px; max-width: 270mm; } h3 { margin: 0; color: #004080; font-size: 15pt; } h4 { margin: 0; color: #004080; } p { margin: 0; }
  .ante { font-size: 10pt; font-weight: 700; letter-spacing: 2.4px; text-transform: uppercase; color: #c77700; margin-bottom: 1.5mm; } .bajada { font-size: 13.5pt; color: #5b6672; margin-top: 1.5mm; max-width: 290mm; } header { margin-bottom: 5mm; }
  code { background: #e5f2ff; border-radius: 3px; padding: 0 4px; font-size: .9em; color: #004080; } .pie-nota { position: absolute; left: 24mm; right: 18mm; bottom: 14mm; font-size: 10.5pt; color: #5b6672; }
  .oscura { background: #004080; color: #fff; padding: 0; } .haz { position: absolute; inset: 0; width: 100%; height: 100%; }
  .portada .torre { position: absolute; right: 26mm; top: 30mm; filter: drop-shadow(0 4mm 10mm rgba(0,0,0,.35)); } .portada .texto { position: absolute; left: 26mm; top: 34mm; width: 185mm; }
  .portada .sobre { font-weight: 700; letter-spacing: 2.6px; text-transform: uppercase; color: #ffe566; font-size: 11pt; } .portada h1 { font-size: 104pt; line-height: 1; margin: 4mm 0 1mm; letter-spacing: 7px; font-weight: 800; } .lema { font-size: 23pt; font-weight: 600; } .portada .desc { font-size: 14.5pt; color: #b3d9ff; margin-top: 9mm; max-width: 160mm; } .portada .equipo { margin-top: 14mm; font-size: 11.5pt; color: #b3d9ff; border-top: .3mm solid rgba(255,255,255,.3); padding-top: 3mm; display: inline-block; } .portada .equipo b { color: #fff; }
  .problema { display: grid; grid-template-columns: 118mm 1fr; gap: 14mm; align-items: center; margin-top: 4mm; } .problema .lista { gap: 5.5mm; } .mismo { background: #f4f6f9; border-radius: 4mm; padding: 6mm; text-align: center; } .rot { font-size: 9.5pt; font-weight: 700; letter-spacing: 1.6px; text-transform: uppercase; color: #5b6672; margin-bottom: 3mm; }
  .formas { display: grid; gap: 2.2mm; } .formas span { background: #fff; border: .3mm solid #d9dfe6; border-radius: 2.5mm; padding: 2.6mm 4mm; font-size: 15.5pt; font-weight: 600; color: #313131; } .formas small { font-weight: 400; font-size: 9.5pt; color: #5b6672; } .flecha { font-size: 22pt; color: #ffd500; line-height: 1.2; font-weight: 800; }
  .oficial { background: #004080; color: #fff; border-radius: 2.5mm; padding: 3mm; font-size: 15pt; font-weight: 600; } .oficial b { color: #ffe566; font-size: 19pt; margin-right: 2mm; } .oficial small { display: block; font-size: 9pt; color: #b3d9ff; font-weight: 400; }
  .lista { margin: 0; padding: 0; list-style: none; display: grid; gap: 3.6mm; } .lista li { padding-left: 7mm; position: relative; font-size: 15pt; line-height: 1.35; } .lista li::before { content: ""; position: absolute; left: 0; top: 2.6mm; width: 3.4mm; height: 3.4mm; border-radius: 50%; background: #ffd500; } .lista b { color: #004080; }
  .verbos { display: flex; align-items: stretch; gap: 3mm; margin-top: 4mm; } .verbos > i { align-self: center; font-style: normal; font-size: 22pt; color: #66b3ff; font-weight: 800; } .verbo { flex: 1; background: #e5f2ff; border-radius: 4mm; padding: 8mm 6mm 9mm; border-top: 1.6mm solid #0080ff; } .verbo.fin { border-top-color: #ffd500; background: #fffbe5; } .verbo b { display: block; font-size: 25pt; color: #004080; margin-bottom: 3mm; } .verbo p { font-size: 12.8pt; line-height: 1.42; }
  .regla { margin-top: 11mm; background: #004080; color: #fff; border-radius: 4mm; padding: 8mm 9mm; font-size: 17pt; border-left: 2.4mm solid #ffd500; } .regla b { color: #ffe566; font-size: 20pt; margin-right: 2mm; }
  .demo { display: grid; grid-template-columns: 200mm 1fr; gap: 9mm; align-items: start; } .caps { position: relative; padding-bottom: 7mm; }
  /* La segunda captura va encima de la primera, abajo a la derecha: así la principal se lee desde lejos. */
  .caps .cap + .cap { position: absolute; right: -5mm; bottom: 0; width: 47%; border: 1mm solid #fff; box-shadow: 0 1.5mm 7mm rgba(7,48,107,.4); } .caps .cap + .cap.movil { width: 21%; right: -3mm; border: 1.2mm solid #004080; border-radius: 4mm; } .cap { border-radius: 2.5mm; overflow: hidden; border: .3mm solid #d9dfe6; box-shadow: 0 1.2mm 5mm rgba(7,48,107,.2); line-height: 0; } .cap img { width: 100%; display: block; }
  .cap.vacia { display: grid; place-items: center; min-height: 60mm; background: #e5f2ff; color: #004080; font-weight: 700; line-height: 1.3; }
  .puntos { margin: 0 0 4mm; padding: 0; list-style: none; display: grid; gap: 2mm; } .puntos li { padding-left: 6mm; position: relative; font-size: 11.8pt; line-height: 1.32; } .puntos li::before { content: "✓"; position: absolute; left: 0; color: #1a7f4b; font-weight: 800; }
  .cifras { display: grid; gap: 2.2mm; } .cifra { background: #004080; color: #fff; border-radius: 3mm; padding: 2.2mm 4mm; border-left: 1.6mm solid #ffd500; } .cifra b { display: block; font-size: 19pt; line-height: 1.12; font-weight: 800; } .cifra span { font-size: 9.3pt; color: #b3d9ff; line-height: 1.3; display: block; }
  .tarjs { display: grid; grid-template-columns: repeat(3, 1fr); gap: 5mm; } .tarj { border: .3mm solid #d9dfe6; border-top: 1.6mm solid #0080ff; border-radius: 3mm; padding: 5mm 5.5mm; min-height: 62mm; } .tarj:nth-child(even) { border-top-color: #ffd500; } .tarj h3 { font-size: 15pt; margin-bottom: 2mm; } .tarj p { font-size: 12.2pt; line-height: 1.42; }
  .cumple { border-collapse: collapse; width: 100%; } .cumple td { padding: 2.5mm 3mm; border-bottom: .25mm solid #d9dfe6; font-size: 12.6pt; vertical-align: top; line-height: 1.35; } .cumple td:nth-child(2) { width: 82mm; color: #004080; } .cumple .si { width: 9mm; color: #1a7f4b; font-weight: 800; font-size: 15pt; }
  .antes { display: grid; grid-template-columns: repeat(3, 1fr); gap: 5mm; } .ant { background: #f4f6f9; border-radius: 3mm; padding: 6mm 6mm; border-left: 1.6mm solid #ffd500; min-height: 48mm; } .ant b { display: block; font-size: 29pt; color: #004080; line-height: 1.1; font-weight: 800; } .ant h4 { font-size: 13.5pt; margin: 2mm 0 1mm; } .ant p { font-size: 11.5pt; color: #5b6672; line-height: 1.35; }
  .adoptar { margin-top: 7mm; display: flex; align-items: center; gap: 4mm; font-size: 14pt; } .adoptar b { color: #004080; } .adoptar span { background: #004080; color: #fff; border-radius: 99px; padding: 1.5mm 5mm; font-weight: 600; font-size: 12.5pt; }
  .dos { display: grid; grid-template-columns: 1fr 1fr; gap: 12mm; } .dos h3 { font-size: 17pt; margin-bottom: 4mm; padding-bottom: 2mm; border-bottom: 1mm solid #ffd500; display: inline-block; } .dos .lista li { font-size: 15pt; } .dos .lista { gap: 5mm; }
  .cierre .centro { position: absolute; inset: 0; display: flex; flex-direction: column; align-items: center; justify-content: center; text-align: center; } .cierre h2 { color: #fff; font-size: 54pt; letter-spacing: 6px; margin-top: 5mm; max-width: none; } .cierre .lema { color: #ffe566; font-size: 20pt; } .cierre .repo { margin-top: 10mm; font-size: 13pt; color: #b3d9ff; } .cierre .equipo { margin-top: 2mm; font-size: 12pt; color: #b3d9ff; } .cierre .inclusion { margin-top: 7mm; font-size: 11.5pt; color: #ffe566; } .cierre .gracias { margin-top: 7mm; font-size: 20pt; font-weight: 700; }
    .probs { list-style: none; margin: 1mm 0 0; padding: 0; display: grid; grid-template-columns: 1fr 1fr; grid-template-rows: repeat(5, auto); grid-auto-flow: column; gap: 2.8mm 13mm; } .probs li { display: grid; grid-template-columns: 9mm 1fr; gap: 3.5mm; align-items: start; }
  .probs .num { width: 9mm; height: 9mm; border-radius: 50%; background: #ffd500; color: #004080; display: grid; place-items: center; font-weight: 800; font-size: 13pt; } .probs h4 { font-size: 13.5pt; line-height: 1.25; }
  .probs p span { font-weight: 600; font-size: 8.5pt; color: #5b6672; background: #e5f2ff; border-radius: 99px; padding: .3mm 2.5mm; margin-left: 1mm; white-space: nowrap; vertical-align: 1px; } .probs p { font-size: 11.3pt; line-height: 1.35; margin-top: .4mm; }
</style></head><body>${D.map((d, i) => { const oscura = d.tipo === 'portada' || d.tipo === 'cierre'; return `<section class="d ${oscura ? d.tipo + ' oscura' : ''}">${oscura ? '' : '<div class="lomo"></div>'}${cuerpo(d)}${oscura ? '' : `<footer>${icono(16)}<b>FARO</b><span>· Que cada dato llegue a puerto</span>${d.crit ? `<span class="crit">Criterio: ${esc(d.crit)}</span>` : ''}<span class="n">${i + 1} / ${D.length}</span></footer>`}<div class="ine"><img src="${INE.ine('ine-horizontal-blanco')}" alt="Instituto Nacional de Estadística"></div></section>`; }).join('')}</body></html>`;
  const hoja = await ctx.newPage(); await hoja.setContent(html, { waitUntil: 'networkidle' }).catch(() => {}); await hoja.evaluate(() => document.fonts.ready);
  await hoja.pdf({ path: salida, width: '338.67mm', height: '190.5mm', printBackground: true, preferCSSPageSize: true });
  await nav.close(); console.log(`${D.length} diapositivas → ${salida} (${(fs.statSync(salida).size / 1048576).toFixed(1)} MB)`);

  // ---------- PowerPoint editable (mismo contenido; textos, tablas y formas nativos) ----------
  let Pptx; try { Pptx = require('pptxgenjs'); } catch (_) { console.log('Sin la biblioteca pptxgenjs (npm install pptxgenjs): se generó solo el PDF.'); return; }
  const TEMA = { name: 'FARO', headFontFace: 'Ubuntu Sans', bodyFontFace: 'Ubuntu Sans', colors: { dk1: '313131', lt1: 'FFFFFF', dk2: '004080', lt2: 'E5F2FF', accent1: '0080FF', accent2: 'FFD500', accent3: '66B3FF', accent4: '1A7F4B', accent5: '5B6672', accent6: '8A5A00', hlink: '0080FF', folHlink: '004080' } };
  const pres = new Pptx(); pres.layout = 'LAYOUT_WIDE'; pres.title = 'FARO · Que cada dato llegue a puerto'; pres.author = 'La Sociedad de Almas'; pres.company = 'La Sociedad de Almas'; pres.subject = 'Datatón INE 2026, categoría 2';
  pres.theme = { headFontFace: TEMA.headFontFace, bodyFontFace: TEMA.bodyFontFace };
  const C = pres.SchemeColor, AZUL = C.text2, CIELO = C.background2, AMBAR = C.accent2, GRIS = C.accent5, BLANCO = C.background1, TINTA = C.text1;
  pres.defineSlideMaster({ title: 'OSCURA', background: { data: fondoPng }, objects: [
    { placeholder: { options: { name: 'title', type: 'title', x: 0.9, y: 2.0, w: 7.6, h: 1.6, fontSize: 96, bold: true, color: BLANCO, align: 'left', valign: 'middle', margin: 0, charSpacing: 6 }, text: '' } },
    { placeholder: { options: { name: 'body', type: 'body', x: 0.9, y: 3.7, w: 7.6, h: 0.6, fontSize: 26, bold: true, color: BLANCO, align: 'left', valign: 'middle', margin: 0 }, text: '' } },
    { rect: { x: 11.33, y: 6.85, w: 2.0, h: 0.65, fill: { color: '0080FF' } } }, { image: { x: 11.56, y: 6.99, w: 1.16, h: 0.375, data: inePng } }] });
  pres.defineSlideMaster({ title: 'CONTENIDO', background: { color: 'FFFFFF' }, objects: [
    { image: { x: 0.6, y: 6.98, w: 0.3, h: 0.3, data: logoPng } },
    { rect: { x: 11.33, y: 6.85, w: 2.0, h: 0.65, fill: { color: '004080' } } }, { image: { x: 11.56, y: 6.99, w: 1.16, h: 0.375, data: inePng } },
    { text: { text: 'FARO · Que cada dato llegue a puerto', options: { x: 1.0, y: 6.98, w: 5, h: 0.3, fontSize: 10, color: GRIS, valign: 'middle', margin: 0 } } },
    { placeholder: { options: { name: 'title', type: 'title', x: 0.6, y: 0.72, w: 12.13, h: 0.75, fontSize: 36, bold: true, color: AZUL, align: 'left', valign: 'middle', margin: 0 }, text: '' } },
    { placeholder: { options: { name: 'body', type: 'body', x: 0.6, y: 1.47, w: 12.13, h: 0.5, fontSize: 15, color: GRIS, align: 'left', valign: 'top', margin: 0 }, text: '' } }],
    slideNumber: { x: 10.45, y: 6.98, w: 0.7, h: 0.3, fontSize: 11, bold: true, color: AZUL, align: 'right' } });
  const caja = (s, texto, o) => s.addText(texto, { isTextBox: true, margin: 0, valign: 'top', ...o });
  const tarjeta = (s, x, y, w, h, color, nombre) => s.addShape(pres.ShapeType.roundRect, { x, y, w, h, rectRadius: 0.08, fill: { color }, line: { type: 'none' }, objectName: nombre });
  const imagen = (s, n, x, y, w, h, nombre) => { if (!tomas[n]) return caja(s, 'Demostración en vivo', { x, y, w, h, align: 'center', valign: 'middle', fontSize: 16, bold: true, color: AZUL, fill: { color: CIELO }, objectName: nombre });
    s.addShape(pres.ShapeType.rect, { x: x - 0.02, y: y - 0.02, w: w + 0.04, h: h + 0.04, fill: { color: BLANCO }, line: { color: 'D9DFE6', width: 0.75 }, objectName: nombre + ' (marco)' }); s.addImage({ data: tomas[n].replace(/^data:/, ''), x, y, w, h, altText: nombre, objectName: nombre }); };
  let seccion = '';
  D.forEach((d, i) => { if (d.seccion !== seccion) { seccion = d.seccion; pres.addSection({ title: seccion }); }
    const oscura = d.tipo === 'portada' || d.tipo === 'cierre', s = pres.addSlide({ masterName: oscura ? 'OSCURA' : 'CONTENIDO', sectionTitle: seccion });
    if (d.notas) s.addNotes(d.notas + (d.crit ? `\n\nCriterio de evaluación que apoya: ${d.crit}.` : ''));
    if (oscura) { s.addText('FARO', { placeholder: 'title' }); s.addText(LEMA, { placeholder: 'body' }); s.addImage({ data: logoPng, x: 9.4, y: 1.7, w: 2.9, h: 2.9, altText: 'Logotipo de FARO: un faro', objectName: 'Logotipo' });
      if (d.tipo === 'portada') { caja(s, 'DATATÓN INE 2026 · CATEGORÍA 2: TECNOLÓGICA Y CIENCIAS DE DATOS', { x: 0.9, y: 1.55, w: 8, h: 0.35, fontSize: 12, bold: true, color: AMBAR, charSpacing: 2, objectName: 'Evento' });
        caja(s, DESC, { x: 0.9, y: 4.55, w: 6.6, h: 1.0, fontSize: 16, color: CIELO, objectName: 'Descripción' }); caja(s, [{ text: 'Equipo La Sociedad de Almas', options: { bold: true, color: BLANCO } }, { text: '  ·  ' + EQUIPO, options: { color: CIELO } }], { x: 0.9, y: 5.9, w: 9, h: 0.35, fontSize: 13, objectName: 'Equipo' }); }
      else { caja(s, `${REPO} · ${version}`, { x: 0.9, y: 4.55, w: 8, h: 0.35, fontSize: 16, color: CIELO, objectName: 'Repositorio' }); caja(s, 'La Sociedad de Almas · ' + EQUIPO, { x: 0.9, y: 4.95, w: 9, h: 0.35, fontSize: 13, color: CIELO, objectName: 'Equipo' }); caja(s, INCLUSION, { x: 0.9, y: 5.4, w: 9.5, h: 0.3, fontSize: 12, color: AMBAR, objectName: 'Inclusión' }); caja(s, 'Gracias. ¿Preguntas?', { x: 0.9, y: 5.95, w: 8, h: 0.5, fontSize: 26, bold: true, color: AMBAR, objectName: 'Cierre' }); }
      return; }
    s.addText(d.titulo, { placeholder: 'title' }); if (d.bajada) s.addText(d.bajada, { placeholder: 'body' });
    caja(s, (d.ante || `Demostración ${d.n}`).toUpperCase(), { x: 0.6, y: 0.4, w: 9, h: 0.3, fontSize: 11, bold: true, color: C.accent6, charSpacing: 2, objectName: 'Antetítulo' });
    if (d.crit) caja(s, `Criterio: ${d.crit}`, { x: 7.4, y: 6.98, w: 4.5, h: 0.3, fontSize: 10, bold: true, color: AZUL, align: 'right', valign: 'middle', objectName: 'Criterio de evaluación' });
    const Y = d.bajada ? 2.2 : 1.75;
    if (d.tipo === 'problemas') { d.items.forEach((x, k) => { const x0 = 0.6 + (k < 5 ? 0 : 6.2), y0 = 2.25 + (k % 5) * 0.93;
        caja(s, String(k + 1), { shape: pres.ShapeType.ellipse, x: x0, y: y0, w: 0.42, h: 0.42, fill: { color: AMBAR }, align: 'center', valign: 'middle', fontSize: 14, bold: true, color: AZUL, objectName: `Número ${k + 1}` });
        caja(s, [{ text: x.t, options: { bold: true, color: AZUL, fontSize: 14.5, breakLine: true } }, { text: x.hoy, options: { color: TINTA, fontSize: 13, breakLine: true } }, { text: 'Lo planteó: ' + x.quien, options: { color: GRIS, fontSize: 10, italic: true } }], { x: x0 + 0.58, y: y0 - 0.04, w: 5.3, h: 0.9, objectName: `Problema ${k + 1}` }); }); }
    if (d.tipo === 'verbos') { d.items.forEach(([v, t], k) => { const x = 0.6 + k * 3.085; tarjeta(s, x, Y, 2.87, 2.75, CIELO, `Tarjeta ${v}`);
        caja(s, v, { x: x + 0.22, y: Y + 0.2, w: 2.43, h: 0.5, fontSize: 26, bold: true, color: k === 3 ? C.accent6 : AZUL, objectName: v }); caja(s, t, { x: x + 0.22, y: Y + 0.8, w: 2.43, h: 1.85, fontSize: 16, color: TINTA, objectName: `${v}: detalle` });
        if (k < 3) caja(s, '›', { x: x + 2.87, y: Y + 1.05, w: 0.215, h: 0.6, fontSize: 28, bold: true, color: C.accent3, align: 'center', valign: 'middle', objectName: 'Flecha' }); });
      tarjeta(s, 0.6, Y + 3.05, 12.13, 1.25, AZUL, 'Regla (fondo)'); caja(s, [{ text: d.regla[0] + '  ', options: { bold: true, color: AMBAR, fontSize: 24 } }, { text: d.regla[1], options: { color: BLANCO, fontSize: 16 } }], { x: 0.95, y: Y + 3.05, w: 11.4, h: 1.25, valign: 'middle', objectName: 'Regla' }); }
    if (d.tipo === 'demo') { imagen(s, d.caps[0], 0.6, Y, 7.55, 4.42, `Captura: ${d.caps[0]}`); if (d.caps[1]) { d.movil ? imagen(s, d.caps[1], 6.75, Y + 1.15, 1.6, 3.2, `Captura: ${d.caps[1]}`) : imagen(s, d.caps[1], 4.75, Y + 2.3, 3.6, 2.11, `Captura: ${d.caps[1]}`); }
      caja(s, d.puntos.map((t, k) => ({ text: t, options: { bullet: true, breakLine: k < d.puntos.length - 1, paraSpaceAfter: 5 } })), { x: 8.75, y: Y, w: 3.98, h: 1.8, fontSize: 14, color: TINTA, objectName: 'Puntos' });
      d.cifras.forEach(([v, t], k) => { const y = Y + 1.93 + k * 0.9; tarjeta(s, 8.75, y, 3.98, 0.82, AZUL, `Cifra ${k + 1} (fondo)`); caja(s, [{ text: v, options: { bold: true, fontSize: 22, color: BLANCO, breakLine: true } }, { text: t, options: { fontSize: 11, color: CIELO } }], { x: 8.92, y: y + 0.03, w: 3.66, h: 0.76, valign: 'middle', objectName: `Cifra ${k + 1}` }); }); }
    if (d.tipo === 'tarjetas') d.items.forEach(([t, x], k) => { const x0 = 0.6 + (k % 3) * 4.11, y0 = Y + Math.floor(k / 3) * 2.5; tarjeta(s, x0, y0, 3.91, 2.32, CIELO, `Tarjeta ${k + 1} (fondo)`);
      caja(s, [{ text: t, options: { bold: true, color: AZUL, fontSize: 17, breakLine: true, paraSpaceAfter: 4 } }, { text: x, options: { color: TINTA, fontSize: 16 } }], { x: x0 + 0.22, y: y0 + 0.18, w: 3.47, h: 2.0, objectName: `Tarjeta ${k + 1}` }); });
    if (d.tipo === 'tabla') { s.addTable(d.filas.map(([t, x]) => [{ text: '✓', options: { bold: true, color: C.accent4, align: 'center', fontSize: 18 } }, { text: t, options: { bold: true, color: AZUL } }, { text: x.replace(/`/g, '') }]), { x: 0.6, y: Y, w: 12.13, colW: [0.55, 3.75, 7.83], fontSize: 15, color: TINTA, valign: 'middle', rowH: d.bajada ? 0.72 : 0.56, border: [{ type: 'none' }, { type: 'none' }, { type: 'solid', pt: 0.75, color: 'D9DFE6' }, { type: 'none' }], margin: [0.03, 0.08, 0.03, 0.08], autoPage: false, objectName: 'Lineamientos' });
      caja(s, d.pie, { x: 0.6, y: 6.5, w: 12.13, h: 0.3, fontSize: 11, color: GRIS, objectName: 'Nota' }); }
    if (d.tipo === 'antes') { d.items.forEach((m, k) => { const x0 = 0.6 + (k % 3) * 4.11, y0 = Y + Math.floor(k / 3) * 2.02; tarjeta(s, x0, y0, 3.91, 1.87, CIELO, `Mejora ${k + 1} (fondo)`);
        caja(s, [{ text: m.cifra, options: { bold: true, color: AZUL, fontSize: m.cifra.length > 12 ? 24 : 36, breakLine: true } }, { text: m.tema, options: { bold: true, color: AZUL, fontSize: 16, breakLine: true } }, { text: m.de, options: { color: GRIS, fontSize: 12 } }], { x: x0 + 0.22, y: y0 + 0.1, w: 3.47, h: 1.67, valign: 'middle', objectName: `Mejora ${k + 1}` }); });
      caja(s, 'Para adoptarlo, el INE necesita:', { x: 0.6, y: Y + 4.2, w: 4.0, h: 0.5, fontSize: 16, bold: true, color: AZUL, valign: 'middle', objectName: 'Adopción' });
      d.adoptar.forEach((t, k) => caja(s, t, { shape: pres.ShapeType.roundRect, rectRadius: 0.25, x: 4.7 + k * 2.7, y: Y + 4.2, w: 2.55, h: 0.5, fill: { color: AZUL }, color: BLANCO, fontSize: 14, bold: true, align: 'center', valign: 'middle', objectName: `Requisito ${k + 1}` })); }
    if (d.tipo === 'dos') [d.izq, d.der].forEach(([t, l], k) => { const x0 = 0.6 + k * 6.17; tarjeta(s, x0, Y, 5.96, 4.75, CIELO, `${t} (fondo)`); caja(s, t, { x: x0 + 0.3, y: Y + 0.22, w: 5.36, h: 0.5, fontSize: 22, bold: true, color: k ? C.accent6 : AZUL, objectName: t });
      caja(s, l.map((x, n) => ({ text: x, options: { bullet: true, breakLine: n < l.length - 1, paraSpaceAfter: 8 } })), { x: x0 + 0.3, y: Y + 0.85, w: 5.36, h: 3.75, fontSize: 18, color: TINTA, objectName: `${t}: lista` }); });
  });
  const pptx = salida.replace(/\.pdf$/, '.pptx'); await pres.writeFile({ fileName: pptx });
  // pptxgenjs no escribe los colores del tema: se ponen aquí, para que el archivo se pueda cambiar de colores en un solo lugar.
  const JSZip = require(require.resolve('jszip', { paths: [require.resolve('pptxgenjs')] })), zip = await JSZip.loadAsync(fs.readFileSync(pptx)), parte = 'ppt/theme/theme1.xml';
  const esquema = `<a:clrScheme name="${TEMA.name}">` + Object.entries(TEMA.colors).map(([k, v]) => `<a:${k}><a:srgbClr val="${v}"/></a:${k}>`).join('') + '</a:clrScheme>';
  zip.file(parte, (await zip.file(parte).async('string')).replace(/<a:clrScheme\b[\s\S]*?<\/a:clrScheme>/, () => esquema).replace(/(<a:(?:theme|fontScheme)\b[^>]*?\bname=")[^"]*"/g, (_, a) => `${a}${TEMA.name}"`));
  // Se vuelve a empaquetar solo con archivos (sin entradas de carpeta, que algunos servicios en la nube no abren).
  const limpio = new JSZip(); for (const [nombre, f] of Object.entries(zip.files)) if (!f.dir) limpio.file(nombre, await f.async('nodebuffer'), { createFolders: false });
  fs.writeFileSync(pptx, await limpio.generateAsync({ type: 'nodebuffer', compression: 'DEFLATE' }));
  console.log(`${D.length} diapositivas → ${pptx} (${(fs.statSync(pptx).size / 1048576).toFixed(1)} MB), editable`);
})().catch(e => { console.error(e); process.exit(1); });
