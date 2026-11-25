// Comprueba los diccionarios de la interfaz (datos/estandar/idiomas/<código>.json) contra la
// plantilla de textos de la pantalla y dice cuánto cubre cada idioma.
//
//   node scripts/idiomas/comprobar.js              comprueba y resume
//   node scripts/idiomas/comprobar.js --completar  además agrega a cada diccionario, vacías, las
//                                                  claves que le faltan (así quien traduce ve lo
//                                                  que queda) y lo deja en el orden de la plantilla
//   node scripts/idiomas/comprobar.js --podar      además quita las claves que ya no están en la pantalla
//
// Una traducción es válida si conserva las mismas marcas (<1>…</1>, <1/>), la misma cantidad de
// números (#) y los mismos datos ({0}) que su clave. En kaqchikel, además, cada término debe estar
// en el glosario con fuente (cak_glosario.csv): ahí no se admite nada sin procedencia.
const fs = require('fs'), path = require('path');
const DIR = path.join(__dirname, '..', '..', 'datos', 'estandar', 'idiomas');
const completar = process.argv.includes('--completar'), podar = process.argv.includes('--podar');
const plantilla = JSON.parse(fs.readFileSync(path.join(DIR, 'plantilla.json'), 'utf8')).textos;
const enPlantilla = new Set(plantilla);
const marcas = t => (t.match(/<\/?\d+\/?>/g) || []).sort().join(), datos = t => (t.match(/\{\d\}/g) || []).sort().join();
const numeros = t => (t.match(/#/g) || []).length;
// Anidado correcto de las marcas de la traducción.
function bienAnidada(t) { const pila = []; for (const m of t.matchAll(/<(\/?)(\d+)(\/?)>/g)) { if (m[3]) continue; if (!m[1]) pila.push(m[2]); else if (pila.pop() !== m[2]) return false; } return pila.length === 0; }
const glosario = fs.existsSync(path.join(DIR, 'cak_glosario.csv')) ? new Set(fs.readFileSync(path.join(DIR, 'cak_glosario.csv'), 'utf8').split('\n').slice(1).map(l => (l.match(/^(?:"[^"]*"|[^,]*),("(?:[^"]|"")*"|[^,]*)/) || [])[1]).filter(Boolean).map(c => c.replace(/^"|"$/g, '').replace(/""/g, '"'))) : null;
let fallas = 0;
for (const archivo of fs.readdirSync(DIR).filter(a => /^[a-z]{2,3}\.json$/.test(a)).sort()) {
  const ruta = path.join(DIR, archivo), dic = JSON.parse(fs.readFileSync(ruta, 'utf8')), textos = dic.textos || {};
  const malas = [], sobrantes = [], sinFuente = [];
  let hechas = 0;
  for (const [k, v] of Object.entries(textos)) {
    if (!enPlantilla.has(k)) sobrantes.push(k);
    if (typeof v !== 'string' || !v.trim()) continue;
    if (marcas(k) !== marcas(v) || datos(k) !== datos(v) || numeros(k) !== numeros(v) || !bienAnidada(v) || /<(?!\/?\d+\/?>)/.test(v)) { malas.push(k); continue; }
    if (enPlantilla.has(k)) hechas++;
    if (archivo === 'cak.json' && glosario && !glosario.has(v.replace(/…$/, '')) && !glosario.has(v)) sinFuente.push(`${k} → ${v}`);
  }
  const faltan = plantilla.filter(k => !(typeof textos[k] === 'string' && textos[k].trim()));
  console.log(`${archivo}: ${dic.nombre} (${dic.estado}) · ${hechas} de ${plantilla.length} textos de la pantalla (${(hechas * 100 / plantilla.length).toFixed(1)} %) · faltan ${faltan.length} · sobran ${sobrantes.length} · inválidas ${malas.length}${sinFuente.length ? ' · sin fuente ' + sinFuente.length : ''}`);
  for (const k of malas.slice(0, 20)) console.log('   inválida: ' + k.slice(0, 110));
  for (const k of sinFuente.slice(0, 20)) console.log('   sin fuente en el glosario: ' + k);
  fallas += malas.length + sinFuente.length;
  if (completar || podar) {
    const nuevo = {};
    for (const k of plantilla) nuevo[k] = typeof textos[k] === 'string' ? textos[k] : '';
    if (!podar) for (const k of sobrantes) nuevo[k] = textos[k];
    if (!completar) for (const k of Object.keys(nuevo)) if (!(k in textos)) delete nuevo[k];
    dic.textos = nuevo;
    fs.writeFileSync(ruta, JSON.stringify(dic, null, 1) + '\n');
  }
}
process.exit(fallas ? 1 : 0);
