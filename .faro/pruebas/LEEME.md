# Guiones de prueba de la pantalla

Cada guion abre el portal en Chromium (Playwright) y comprueba una parte: clic real en el mapa,
permisos por rol, recorridos, trabajo de campo, carga de precios. Los resultados de cada corrida
están en `docs/20-verificacion-interfaz.md` y en `docs/08-versiones.md`.

- **Córralos solo contra una base desechable**: crean cuentas, envíos y rutas de prueba.
- Las contraseñas no están en el código: `FARO_CLAVE_PRUEBA` (la que tendrán las cuentas de prueba)
  y `FARO_CLAVE_INICIAL` (la inicial del administrador, si no es la del despliegue con Docker).
- `prueba_envio.csv` y los archivos que los guiones generan son **ficticios**: ninguno viene de una
  institución.
- Las contraseñas iniciales que los guiones ponen a las cuentas que ellos mismos crean
  (`Inicial-…`) son datos de prueba de esa base desechable, no credenciales de ningún sistema.

Ejemplo:

    FARO_CLAVE_PRUEBA=<una clave de prueba> NO_PROXY=localhost node .faro/pruebas/clic_real_mapa.js http://localhost:8082

Idiomas de la pantalla:

- `idiomas.js` cambia a inglés y a kaqchikel y comprueba que la interfaz se traduce y que los
  nombres de los departamentos, las cifras y las citas **no cambian** (37 comprobaciones).
- `FARO_IDIOMA=en node .faro/pruebas/verificacion_interfaz.js <url>` repite la revisión por tamaño
  de pantalla con la pantalla en inglés (no reescribe el informe ni las capturas).
