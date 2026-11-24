// Entrada compartida por los guiones: inicia sesión como administrador. Si la base es nueva
// (admin / admin), cambia la contraseña inicial, que el portal ya exige antes de permitir cambios.
const CLAVE = 'Faro-Prueba-2026';
async function entrar(p, usuario = 'admin', clave = CLAVE, inicial = 'admin') {
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
