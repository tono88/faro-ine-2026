// Entrada compartida por los guiones: inicia sesión como administrador. Si la base es nueva, cambia la
// contraseña inicial, que el portal ya exige antes de permitir cambios.
// Ninguna contraseña vive en el código: la de prueba se da en FARO_CLAVE_PRUEBA (10 caracteres o más, con
// letras y números) y la inicial del administrador en FARO_CLAVE_INICIAL (la misma que usó el servidor).
// Estos guiones crean cuentas y envíos de prueba: córralos solo contra una base desechable.
const CLAVE = process.env.FARO_CLAVE_PRUEBA;
if (!CLAVE) { console.error('Defina FARO_CLAVE_PRUEBA con la contraseña que usarán las cuentas de prueba (base desechable).'); process.exit(2); }
async function entrar(p, usuario = 'admin', clave = CLAVE, inicial = process.env.FARO_CLAVE_INICIAL || 'admin') {
  const r = await p.evaluate(async ([u, c, ini]) => {
    const pedir = (ruta, cuerpo) => fetch(ruta, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(cuerpo) });
    let r = await pedir('/api/sesion', { usuario: u, clave: c });
    if (r.ok) return 'ya tenía contraseña propia';
    r = await pedir('/api/sesion', { usuario: u, clave: ini });
    if (!r.ok) return 'NO ENTRÓ: ' + (await r.text());
    const c2 = await pedir('/api/perfil', { clave_actual: ini, clave_nueva: c });
    return c2.ok ? 'contraseña inicial cambiada' : 'NO CAMBIÓ: ' + (await c2.text());
  }, [usuario, clave, inicial]);
  await p.reload(); await p.waitForSelector('#lienzo path.leaflet-interactive'); await p.waitForTimeout(600);
  return r;
}
module.exports = { entrar, CLAVE };
