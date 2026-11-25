// Tipografía de la guía de diseño del INE (Ubuntu Sans, Ubuntu Font Licence 1.0), incrustada en
// los documentos que se generan: así el PDF sale igual con o sin internet.
const fs = require('fs'), path = require('path');
const DIR = path.join(__dirname, '..', '..', 'crates', 'faro-servidor', 'web', 'vendor', 'ubuntu-sans');
const CSS = [400, 500, 600, 700, 800].map(p => `@font-face { font-family: "Ubuntu Sans"; font-weight: ${p}; font-style: normal; src: url(data:font/woff2;base64,${fs.readFileSync(path.join(DIR, `ubuntu-sans-${p}.woff2`)).toString('base64')}) format("woff2"); }`).join('\n');
// Logotipos del INE (guía de diseño gráfico del Datatón, noviembre de 2026), como imagen incrustada.
const ine = nombre => 'data:image/svg+xml;base64,' + fs.readFileSync(path.join(__dirname, '..', '..', 'docs', 'identidad', 'ine', nombre + '.svg')).toString('base64');
module.exports = { CSS, ine };
