# Verificación de ciberseguridad

Revisión de todo el portal hecha el 24-nov-2026 en dos pasos: una auditoría independiente del
código y de la instancia en marcha (rutas, sesiones, inyección, archivos, peticiones salientes,
abuso de recursos, cabeceras, secretos y privacidad) y, después, la corrección de lo encontrado.
Cada corrección tiene una prueba automática en `.faro/pruebas/tableros_grupos_seguridad.js`.

## Lo que la auditoría encontró bien desde el inicio

- **Contraseñas** guardadas con Argon2id y sal propia; verificar cuesta lo mismo si el usuario no existe.
- **Sesiones**: llave de 32 bytes del generador del sistema; en la base solo queda su huella SHA-256;
  caducan a las 12 horas; al entrar siempre se emite una nueva; salir la invalida en el servidor.
- **Inyección SQL**: no hay. Todas las consultas usan parámetros (se probó con comillas y `pg_sleep`).
- **Recorrido de rutas**: no hay. `/ejemplos`, `/scripts`, `/capas` y las plantillas devuelven 404
  ante `../` en todas sus codificaciones.
- **Roles**: toda ruta que cambia datos exige sesión; un usuario de consulta recibe 403 en todas.
- **XSS**: lo que viene de la base, de archivos subidos y de tiendas se escapa antes de pintarse.
- **Privacidad**: solo salen las columnas pedidas; las marcadas para eliminar no se guardan; las de
  identidad salen como llave no reversible; los resultados guardan únicamente conteos.
- **Contenedor**: el servidor no corre como administrador y la base no publica su puerto.

## Hallazgos y qué se hizo

| # | Gravedad | Hallazgo | Estado | Qué se hizo |
|---|---|---|---|---|
| 1 | Alta | La cuenta inicial `admin` / `admin` servía indefinidamente; el puerto se abría a toda la red | **Corregido** | Con la contraseña inicial no se puede hacer ningún cambio hasta cambiarla (403). Por omisión el portal solo se atiende en la propia máquina (`FARO_ESCUCHA`) |
| 2 | Alta | El archivo original de una institución podía quedar en disco si la lectura fallaba o se cortaba la conexión | **Corregido** | Una guarda borra el temporal siempre: error, pánico o conexión cortada |
| 3 | Alta | Una sola conexión a la base y consultas pesadas sin sesión: se podía dejar el portal sin responder | **Mitigado** | Tope de 30 s por consulta y de tres consultas pesadas a la vez (las demás reciben "intente de nuevo"). Queda pendiente un grupo de conexiones |
| 4 | Alta | Se aceptaban cuerpos de hasta 300 MB en cualquier ruta | **Corregido** | Tope general de 20 MB; solo la recepción de envíos admite archivos grandes |
| 5 | Media | No había freno a los intentos de contraseña | **Corregido** | Tras cinco fallos seguidos el usuario queda bloqueado 30 s, y el bloqueo se duplica (hasta 15 min); máximo tres verificaciones a la vez |
| 6 | Media | Contraseñas débiles; cambiarla no cerraba otras sesiones | **Corregido** | Mínimo 10 caracteres, dos tipos de carácter, sin el usuario ni palabras comunes; al cambiarla se cierran las demás sesiones |
| 7 | Media | Pendientes, envíos y equivalencias se podían leer sin sesión | **Corregido** | Esas lecturas piden sesión. Los datos de referencia y las capas siguen abiertos, como corresponde a datos públicos |
| 8 | Media | Sin cabeceras de seguridad; la galleta no llevaba `Secure` | **Corregido** | Política de contenido, `X-Frame-Options: DENY`, `nosniff`, `Referrer-Policy`; `Secure` con `FARO_HTTPS=1` |
| 9 | Media | Sin sesión se podía hacer que el servidor bajara y decodificara fotos | **Corregido** | Solo con sesión se bajan fotos nuevas; tope de 6,000 px y 96 MB al decodificar |
| 10 | Media-baja | Direcciones de fotos y submapas de tiendas seguidas sin validar | **Corregido en parte** | Las fotos no siguen redirecciones; los submapas deben ser del mismo sitio. Falta rechazar direcciones internas al resolver nombres |
| 11 | Baja | Una tienda podría publicar enlaces que no sean `https` | **Corregido** | Solo se guardan direcciones `https` |
| 12 | Baja | La única defensa contra peticiones falsificadas era `SameSite=Strict` | **Corregido** | Se rechaza toda petición que cambie datos y venga de otro origen |
| 13 | Baja | La clave de servicio se comparaba con `==` | **Corregido** | Comparación de huellas en tiempo constante |
| 14 | Baja | El reporte de validación podía citar el valor de una columna sensible | **Corregido** | De una columna sensible nunca se cita el valor |
| 15 | Baja | La llave de pareo usa SHA-256 con sal en lugar de HMAC | **Pendiente** | Cambiarla altera las llaves ya emitidas: se decide antes de recibir datos reales |
| 16 | Baja | Un error de sincronización mostraba detalles internos | **Corregido** | El detalle va a la bitácora del servidor; al público, un mensaje general |
| 17 | Baja | Las celdas del CSV que empiezan con `=`, `+`, `-`, `@` se abrirían como fórmulas | **Corregido** | Se neutralizan con un apóstrofo |
| 18 | Baja | El guardado de un envío no es una sola transacción | **Pendiente** | Si se corta a medias puede quedar incompleto; se detecta en el avance |
| 19 | Informativa | Claves de `crear_env.ps1` con un generador no criptográfico | **Corregido** (sin probar en Windows) | Usa el generador criptográfico del sistema |
| 20 | Informativa | La tipografía se pide a Google Fonts | **Pendiente** | Servirla desde el propio portal evita esa petición a un tercero |

## Lo que debe hacer quien lo instale

1. Entrar con `admin` / `admin` y **cambiar la contraseña**: hasta entonces no podrá hacer cambios.
2. Dejar `FARO_ESCUCHA` como está (solo esta máquina) o, para abrirlo a la red, ponerlo detrás de un
   proxy con HTTPS y activar `FARO_HTTPS=1`.
3. Usar claves propias en `.env` (`iniciar.bat` las genera).
4. Asignar a cada persona el rol y el grupo mínimos que necesite.

## Pruebas que lo respaldan

`tableros_grupos_seguridad.js` comprueba en cada corrida: cabeceras presentes; lecturas sensibles
cerradas sin sesión; rechazo de peticiones de otro origen; bloqueo tras intentos fallidos; rechazo de
contraseñas débiles; recorrido de rutas; permisos por grupo (403 fuera de su sección). No se hizo una
prueba de penetración externa ni se corrió `cargo audit`: conviene hacerlo antes de un uso real.
