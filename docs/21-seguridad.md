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
| 1 | Alta | La cuenta inicial `admin` / `admin` servía indefinidamente; el puerto se abría a toda la red | **Corregido** | Con la contraseña inicial no se puede hacer ningún cambio ni leer nada interno hasta cambiarla (403). Por omisión el portal solo se atiende en la propia máquina (`FARO_ESCUCHA`), con Docker y sin Docker |
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
| 17 | Baja | Las celdas del CSV que empiezan con `=`, `+`, `-`, `@` se abrirían como fórmulas | **Corregido** | Se neutralizan con un apóstrofo, en el CSV de precios y (desde la v0.18.0) en el de visitas de campo |
| 18 | Baja | El guardado de un envío no es una sola transacción | **Pendiente** | Si se corta a medias puede quedar incompleto; se detecta en el avance |
| 19 | Informativa | Claves de `crear_env.ps1` con un generador no criptográfico | **Corregido** (sin probar en Windows) | Usa el generador criptográfico del sistema |
| 20 | Informativa | La tipografía se pide a Google Fonts | **Pendiente** | Servirla desde el propio portal evita esa petición a un tercero |

## Cambios de la versión 0.16.0

La versión 0.16.0 agregó superficie nueva (trabajo de campo, tableros publicados, fotos). Esto es lo
que se revisó y cómo quedó:

| # | Tema | Riesgo | Qué se hizo |
|---|---|---|---|
| 21 | Ingreso | Si el usuario no existía no se calculaba Argon2 y la respuesta salía antes: el tiempo delataba qué usuarios existen. Lo halló la revisión de comentarios, que contradecía a este informe | **Corregido**: se verifica siempre, contra una huella de relleno si el usuario no existe |
| 22 | Posición y cámara | El portal bloqueaba la geolocalización para todos; el trabajo de campo la necesita | Se permite **solo al propio portal** (`Permissions-Policy: geolocation=(self), camera=(self)`); ningún contenido de terceros puede pedirla |
| 23 | Visitas de campo | La posición de una visita es la de un hogar | Solo las leen el operador que las hizo y quien coordina; no entran al mapa ni a las capas. La foto se clasifica en memoria y se descarta; tope de 4 MB y de 8,000 px al decodificar |
| 24 | Formulario de la encuesta | Una dirección cualquiera llevaría a los operadores a un sitio ajeno | Solo `https` y solo servicios de una lista (`FARO_FORMULARIOS`); se rechazan direcciones con usuario, puerto o dominios parecidos |
| 25 | Rol «operador» | Un encuestador no debe poder cambiar nada más | Su rol solo permite leer su ruta y registrar sus visitas; todo lo demás responde 403 (probado) |
| 26 | Tableros publicados | Lo publicado se lee sin sesión | Un borrador solo lo leen su autor y los administradores; publicar y retirar exige sesión y ser el autor; la definición se valida en el servidor (tipos, indicadores, tamaños) y todo texto se escapa al pintarlo |
| 27 | Conteos pequeños | Un conteo de 1 o 2 puede señalar a una persona | No se publican a quien no es analista, con supresión complementaria (`docs/23`) |
| 28 | Constancia | No había un registro general de acciones | Bitácora: envíos, equivalencias, cuentas, tableros, rutas y visitas, con quién y cuándo. Solo la leen los administradores |
| 29 | Fotos satelitales | El navegador pide imágenes a NASA y EOX | Son opcionales y solo imágenes (`img-src`); el servidor no las pide ni envía datos a esos servicios |

| 30 | Menús y paneles de carga | Sin sesión se veían secciones y controles de carga que no se podían usar | Las secciones que piden sesión no aparecen en el menú ni se abren por enlace; los paneles de carga y los botones de cambio solo se muestran con permiso. La carga de una tabla propia en la hoja de balance, que estaba abierta, ahora pide sesión. Es una mejora de claridad: el control real sigue estando en el servidor |
| 31 | Cuentas de institución | Cualquier cuenta con sesión podía leer la lista de envíos, los pendientes y las equivalencias de todas las instituciones | Rol «institucion» atado a una institución: `quien_envia()` solo admite la solicitud de su institución (403 en otro caso, y también con la contraseña inicial); `api_envios` filtra por institución; pendientes y equivalencias pasan por `personal_del_ine()` (administrador, analista, consulta). Prueba: `institucion_y_recorridos.js` |
| 32 | Contraseñas en el código | La contraseña inicial `admin` y una cadena de conexión con clave por omisión estaban escritas en el programa; los guiones de prueba traían una contraseña fija | Ninguna contraseña queda en el código: la inicial la da `FARO_CLAVE_INICIAL` (el despliegue con Docker la deja en `admin`; si falta, se genera al azar y se escribe una vez en el registro), `DATABASE_URL` es obligatoria y los guiones leen su clave del entorno |
| 33 | Contraseña inicial | Con la contraseña inicial no se podía cambiar nada, pero sí leer envíos, pendientes, equivalencias y conteos exactos | `con_sesion()` y `ve_conteos_exactos()` rechazan una sesión con contraseña inicial: hasta cambiarla solo se ve lo público |
| 34 | Servidor sin Docker | Atendía siempre en todas las interfaces | Lee `FARO_ESCUCHA` (por omisión `127.0.0.1`); la imagen de Docker la fija dentro del contenedor y el mapeo de puertos decide quién llega |
| 35 | Secreto estadístico | El mapa ocultaba la celda pequeña y su complemento, pero la ficha del municipio solo la pequeña: con el total del departamento se podía despejar la oculta | Una sola función, `celdas_ocultas()`, decide qué celdas no se publican y la usan el mapa y la ficha |
| 36 | Salida fila por fila | Al eliminar un envío quedaba en el volumen privado su salida disociada | Se borra junto con el envío |
| 37 | Suplantación en la bitácora | Una persona podía ponerse el nombre visible de otra, que es el que firma la bitácora y los tableros | El nombre no puede coincidir con el nombre ni con el usuario de otra cuenta |
| 38 | Bitácora incompleta | Las equivalencias de valores de catálogo y de productos no quedaban anotadas | Quedan anotadas; la bitácora se consulta en «Usuarios» (solo administradores) |
| 39 | Muestreos | Ante un 403 o un 429 la tarea seguía pidiendo hasta agotar su cupo | Se detiene en el primer rechazo y no insiste |

**Pendiente, además de lo anotado arriba:** el bloqueo por intentos fallidos es por usuario y no por
origen (alguien podría dejar bloqueada la cuenta `admin` a propósito); los muestreos no leen
`robots.txt`; los ingresos no se anotan en la bitácora; la llave de pareo usa SHA-256 con secreto y
no HMAC.

Pruebas: `visibilidad_sin_sesion.js`, `trabajo_de_campo.js` (permisos del operador, visitas, formulario), `disenador_tableros.js`
(borradores y publicados) y `tableros_grupos_seguridad.js`.

Siguen pendientes los puntos 3, 10, 15, 18 y 20 de la tabla anterior, y dos nuevos: publicar el
portal con HTTPS es **obligatorio** para el trabajo de campo (sin HTTPS el teléfono no entrega la
posición), y la protección de conteos no compara tablas entre sí.

## Lo que debe hacer quien lo instale

1. Entrar con `admin` y la contraseña inicial (`admin` en el despliegue con Docker, o la que haya puesto en
   `FARO_CLAVE_INICIAL`) y **cambiarla**: hasta entonces no podrá hacer cambios ni ver nada interno.
2. Dejar `FARO_ESCUCHA` como está (solo esta máquina) o, para abrirlo a la red, ponerlo detrás de un
   proxy con HTTPS y activar `FARO_HTTPS=1`.
3. Usar claves propias en `.env` (`iniciar.bat` las genera).
4. Asignar a cada persona el rol y el grupo mínimos que necesite.

## Pruebas que lo respaldan

`tableros_grupos_seguridad.js` comprueba en cada corrida: cabeceras presentes; lecturas sensibles
cerradas sin sesión; rechazo de peticiones de otro origen; bloqueo tras intentos fallidos; rechazo de
contraseñas débiles; recorrido de rutas; permisos por grupo (403 fuera de su sección). No se hizo una
prueba de penetración externa ni se corrió `cargo audit`: conviene hacerlo antes de un uso real.
