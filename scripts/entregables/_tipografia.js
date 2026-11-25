// Tipografía de la guía de diseño del INE (Ubuntu Sans, Ubuntu Font Licence 1.0), incrustada en
// los documentos que se generan: así el PDF sale igual con o sin internet.
const fs = require('fs'), path = require('path');
const DIR = path.join(__dirname, '..', '..', 'crates', 'faro-servidor', 'web', 'vendor', 'ubuntu-sans');
const CSS = [400, 500, 600, 700, 800].map(p => `@font-face { font-family: "Ubuntu Sans"; font-weight: ${p}; font-style: normal; src: url(data:font/woff2;base64,${fs.readFileSync(path.join(DIR, `ubuntu-sans-${p}.woff2`)).toString('base64')}) format("woff2"); }`).join('\n');
// Logotipos del INE (guía de diseño gráfico del Datatón, noviembre de 2026), como imagen incrustada.
const ine = nombre => 'data:image/svg+xml;base64,' + fs.readFileSync(path.join(__dirname, '..', '..', 'docs', 'identidad', 'ine', nombre + '.svg')).toString('base64');
// Carta de presentación del equipo (emblema, nombres y QR de LinkedIn), como SVG para poner dentro de una página.
const carta = () => fs.readFileSync(path.join(__dirname, '..', '..', 'docs', 'identidad', 'equipo', 'carta-de-presentacion.svg'), 'utf8');
// Texto de un documento sin el pie del equipo (para los PDF que juntan varios documentos y lo ponen una vez).
const sinPie = t => { const i = t.indexOf('<!-- equipo -->'); return i >= 0 ? t.slice(0, i) : t; };
module.exports = { CSS, ine, carta, sinPie };
