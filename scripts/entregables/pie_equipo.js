// Deja al final del README y de cada documento de docs/ la carta de presentación del equipo
// (emblema, nombres y código QR del perfil de LinkedIn de cada integrante).
//   node scripts/entregables/pie_equipo.js
// Se puede correr las veces que haga falta: si el pie ya está, lo reemplaza. Hay que volver a
// correrlo después de regenerar un documento (faro citas, faro mejoras, terceros.js,
// verificacion_interfaz.js), porque esos comandos escriben el archivo completo.
const fs = require('fs'), path = require('path');
const RAIZ = path.join(__dirname, '..', '..'), MARCA = '<!-- equipo -->';
const pie = base => `${MARCA}\n---\n\n<p align="center"><img src="${base}identidad/equipo/carta-de-presentacion.svg" alt="Equipo La Sociedad de Almas: Estuardo Sandoval, Ana Echeverría y Alejandro Sandoval, con el código QR del perfil de LinkedIn de cada uno" width="760"></p>\n\n<p align="center"><a href="https://www.linkedin.com/in/estuardoantoniosandoval/">Estuardo Sandoval</a> · <a href="https://www.linkedin.com/in/analuciaecheverria/">Ana Echeverría</a> · <a href="https://www.linkedin.com/in/alejandro-jos%C3%A9-sandoval-acevedo-998781aa/">Alejandro Sandoval</a><br>Equipo La Sociedad de Almas · Datatón INE 2026</p>\n`;
const poner = (archivo, base) => { let t = fs.readFileSync(archivo, 'utf8'); const i = t.indexOf(MARCA); if (i >= 0) t = t.slice(0, i); fs.writeFileSync(archivo, t.replace(/\s+$/, '') + '\n\n' + pie(base)); };
poner(path.join(RAIZ, 'README.md'), 'docs/');
const docs = fs.readdirSync(path.join(RAIZ, 'docs')).filter(a => /^\d\d-.*\.md$/.test(a));
for (const a of docs) poner(path.join(RAIZ, 'docs', a), '');
console.log(`carta de presentación al final de README.md y de ${docs.length} documentos`);
module.exports = { MARCA };
