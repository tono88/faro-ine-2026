// Lista los componentes de terceros con su licencia (docs/27-componentes-de-terceros.md), como pide la nota
// conceptual del Datatón: las bibliotecas de Rust salen de `cargo metadata` (versiones exactas de Cargo.lock);
// lo que no es de Rust (Leaflet, tipografía, base de datos, modelo, herramientas de prueba) se declara aquí.
// Uso: node scripts/entregables/terceros.js
const { execSync } = require('child_process'); const fs = require('fs'); const path = require('path');
const raiz = path.resolve(__dirname, '../..');
const meta = JSON.parse(execSync('cargo metadata --format-version 1 --locked', { cwd: raiz, maxBuffer: 1 << 28 }).toString());
const propios = new Set(meta.workspace_members), porId = new Map(meta.packages.map(p => [p.id, p]));
// Para qué se usa cada dependencia directa (lo demás llega por ellas).
const USO = { serde: 'Leer y escribir estructuras (JSON, TOML, CSV)', serde_json: 'JSON del API y de los catálogos', toml: 'Solicitudes de datos y reglas declarativas', csv: 'Lectura y escritura de CSV', strsim: 'Parecido entre nombres, solo para proponer (nunca para asignar)',
  'unicode-normalization': 'Quitar tildes y normalizar texto', chrono: 'Fechas', clap: 'Línea de comandos `faro`', ureq: 'Descargas HTTP de fuentes abiertas y muestreos', sha2: 'Huellas SHA-256 de descargas y llaves de pareo', calamine: 'Lectura de Excel (xlsx, xls, ods)', zip: 'Abrir archivos comprimidos de las fuentes',
  regex: 'Extracción de precios y limpieza de texto', weezl: 'Descompresión LZW del lector propio de GeoTIFF', argon2: 'Huella de contraseñas', axum: 'Servidor web', tokio: 'Ejecución asíncrona del servidor', 'tokio-postgres': 'Conexión a PostgreSQL', image: 'Abrir fotos (huella visual y clasificador)',
  'tract-onnx': 'Ejecutar el modelo Places365 dentro del servidor, sin nube', rand: 'Sal de contraseñas e identificadores de sesión', rand_core: 'Azar del sistema para la sal de las contraseñas y los identificadores de sesión', hex: 'Texto hexadecimal de huellas', 'tower-http': 'Límites y cabeceras del servidor', tempfile: 'Carpeta temporal donde se procesa un envío y se borra', flate2: 'Descompresión (deflate/gzip)', hmac: 'Llaves de pareo con secreto' };
const directas = new Map();
for (const nodo of meta.resolve.nodes) if (propios.has(nodo.id)) for (const d of nodo.deps) { const p = porId.get(d.pkg); if (!propios.has(p.id)) { const e = directas.get(p.name) || { p, de: new Set() }; e.de.add(porId.get(nodo.id).name); directas.set(p.name, e); } }
const terceros = meta.packages.filter(p => !propios.has(p.id) && p.source);
const porLicencia = new Map(); for (const p of terceros) { const l = (p.license || 'sin declarar').replace(/\s*\/\s*/g, ' OR '); porLicencia.set(l, (porLicencia.get(l) || 0) + 1); }
const sinPermisiva = terceros.filter(p => !/MIT|Apache|BSD|ISC|Zlib|Unicode|Unlicense|CC0|0BSD|BSL|MPL|CDLA-Permissive/i.test(p.license || ''));
const fila = c => '| ' + c.join(' | ') + ' |';
const md = `# Componentes de terceros y sus licencias

*Generado con \`node scripts/entregables/terceros.js\` a partir de \`Cargo.lock\`. No editar a mano.*

La nota conceptual del Datatón pide que las librerías y componentes de terceros conserven sus
licencias originales y se listen en la documentación. Todo lo que FARO usa es de código abierto o de
uso libre; **no hay ningún componente de pago**. El código propio de FARO (los cinco *crates* de
\`crates/\`, la pantalla y los guiones) se escribió durante el evento; su licencia está en
[\`LICENCIA.md\`](../LICENCIA.md).

## 1. Bibliotecas de Rust usadas directamente (${directas.size})

${fila(['Biblioteca', 'Versión', 'Licencia', 'Para qué se usa', 'En'])}
|---|---|---|---|---|
${[...directas.values()].sort((a, b) => a.p.name.localeCompare(b.p.name)).map(({ p, de }) => fila([`[\`${p.name}\`](https://crates.io/crates/${p.name})`, p.version, p.license || 'sin declarar', USO[p.name] || '', [...de].map(x => x.replace('faro-', '')).sort().join(', ')])).join('\n')}

## 2. Bibliotecas que llegan por las anteriores

Las dependencias directas traen otras ${terceros.length - directas.size} bibliotecas (${terceros.length} en total contando las propias de cada sistema operativo, con sus
versiones exactas en \`Cargo.lock\`). Por licencia declarada:

${fila(['Licencia', 'Bibliotecas'])}
|---|---|
${[...porLicencia.entries()].sort((a, b) => b[1] - a[1]).map(([l, n]) => fila([l, n])).join('\n')}

${sinPermisiva.length ? 'Bibliotecas cuya licencia no es una de las permisivas habituales: ' + sinPermisiva.map(p => `\`${p.name}\` (${p.license || 'sin declarar'})`).join(', ') + '.' : 'Todas declaran una licencia permisiva (MIT, Apache-2.0, BSD, ISC, Zlib, Unicode, MPL-2.0 u otra equivalente): ninguna obliga a publicar el código de FARO.'}

La lista completa, con versión y licencia de cada una, se obtiene con \`cargo metadata --format-version 1\`
o con \`cargo tree\`.

## 3. Componentes que no son de Rust

${fila(['Componente', 'Versión', 'Licencia', 'Dónde está y para qué'])}
|---|---|---|---|
${[['[Leaflet](https://leafletjs.com)', '1.9.4', 'BSD de 2 cláusulas', 'Incluido en `crates/faro-servidor/web/vendor/` con su archivo de licencia: mapas de la pantalla'],
  ['[Noto Sans](https://fonts.google.com/noto/specimen/Noto+Sans)', '—', 'SIL Open Font License 1.1', 'Tipografía; el navegador la pide a Google Fonts si hay internet (si no, usa la del sistema)'],
  ['[PostgreSQL](https://www.postgresql.org)', '16', 'Licencia PostgreSQL (permisiva)', 'Base de datos; imagen oficial `postgres:16`'],
  ['Imágenes de Docker `rust:1-bookworm` y `debian:bookworm-slim`', '—', 'Software libre (Debian; Rust: MIT o Apache-2.0)', 'Compilar y ejecutar el portal'],
  ['[Places365-ResNet18](http://places2.csail.mit.edu) (MIT CSAIL)', '—', 'Pesos: CC BY; código: MIT', '`datos/modelos/` ([procedencia](../datos/modelos/PROCEDENCIA.md)): clasificador de fotos del trabajo de campo, sin cambios'],
  ['[Playwright](https://playwright.dev) y Chromium', '—', 'Apache-2.0 / BSD', 'Solo para las pruebas de pantalla y para generar los PDF; no forman parte del portal'],
  ['[PyTorch](https://pytorch.org) y torchvision', '—', 'BSD de 3 cláusulas', 'Solo en `scripts/modelo/exportar.py`, para convertir el modelo a ONNX una vez; no forman parte del portal']].map(fila).join('\n')}

## 4. Servicios que el navegador consulta (opcionales, gratuitos)

No son componentes del portal: los pide el navegador de quien mira, si hay internet y si la persona
enciende esa capa. Ningún dato del INE ni de las instituciones sale hacia ellos.

${fila(['Servicio', 'Qué entrega', 'Condiciones'])}
|---|---|---|
${[['Mapa base de OpenStreetMap', 'Teselas del mapa de fondo', '© colaboradores de OpenStreetMap (ODbL); política de uso de teselas de la OSMF'],
  ['NASA GIBS', 'Mosaicos anuales Landsat (WELD) de 1985, 1990, 1998 y 2000', 'Datos de la NASA, de uso libre con reconocimiento'],
  ['EOX IT Services', 'Mosaico Sentinel-2 sin nubes de 2016', 'CC BY 4.0 (contiene datos Copernicus Sentinel modificados, 2016)'],
  ['Microsoft Forms del INE', 'La encuesta de campo, si el INE la configura', 'Cuenta institucional del INE; las respuestas no pasan por FARO']].map(fila).join('\n')}

Los **datos** (INE, OpenStreetMap, Comisión Europea, SESAN, FAO y los demás) tienen su propia lista,
con institución, fecha de descarga y licencia: [\`11-fuentes-y-citas.md\`](11-fuentes-y-citas.md).
`;
fs.writeFileSync(path.join(raiz, 'docs/27-componentes-de-terceros.md'), md);
console.log(`${directas.size} directas, ${terceros.length} en total → docs/27-componentes-de-terceros.md`);
const faltan = [...directas.keys()].filter(n => !USO[n]); if (faltan.length) console.log('sin descripción de uso:', faltan.join(', '));
