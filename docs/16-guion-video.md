# Guion y plan de grabación del video (2 a 3 minutos)

*FARO · Que cada dato llegue a puerto.* Equipo La Sociedad de Almas.

La nota conceptual pide un **video demostrativo de 2 a 3 minutos, como respaldo en caso de fallas
técnicas durante la demostración en vivo**. Por eso el video sigue el mismo orden que la
demostración de la presentación ([`17`](17-presentacion-7-minutos.md)): si el portal falla frente
al jurado, se proyecta el video y se narra encima.

**Idea central:** FARO convierte archivos dispares en datos oficiales, sin adivinar, y lo demuestra.
**Duración objetivo:** 2:50 (límite: 3:00). **Formato:** captura de pantalla a 1920×1080, con voz
y con subtítulos (para que se entienda también sin sonido).

## 1. Preparación (una vez, antes de grabar)

Todo lo que aparece en el video es de demostración: **ningún archivo real de una institución**.

| Paso | Cómo |
|---|---|
| 1. Instalación nueva | `docker compose down -v` (borra la base de esa máquina) y `docker compose up -d --build` |
| 2. Datos y cuentas de demostración | `node scripts/entregables/preparar_demo.js http://localhost:8082 <contraseña>` crea una cuenta de cada rol, sube los archivos de ejemplo ficticios, deja dos nombres por decidir en «Revisar pendientes», publica un tablero y carga la ruta de demostración |
| 3. Navegador | Ventana de 1920×1080, zoom al 100 %, sin marcadores ni otras pestañas; modo claro |
| 4. Internet | Encendido: la animación usa la foto satelital (si no hay, corre sin foto) |
| 5. Archivo para subir | Descargarlo del propio portal: «Recibir envío» → «Descargar archivo de ejemplo (ficticio)» |
| 6. Teléfono | Para la escena de campo basta la ventana del navegador en tamaño de teléfono (390×780). Con un teléfono real hace falta HTTPS |

Cuentas que deja el guion de preparación (todas con la contraseña que se le indique):
`admin` (administrador), `mlopez` (analista), `jperez` (consulta), `campo01` (operador),
`enlace.pnc`, `enlace.renap` y `enlace.conred` (instituciones).

## 2. Guion, escena por escena

| # | Tiempo | Cuenta | En pantalla (acciones exactas) | Voz | Subtítulo |
|---|---|---|---|---|---|
| 0 | 0:00–0:10 | — | Portada: logotipo y lema sobre azul. Corte al mapa de municipios | «Cada institución le envía al INE sus registros a su manera: otros nombres, otros códigos, otros formatos. FARO es la herramienta para pedirlos, recibirlos y entenderlos.» | FARO · Que cada dato llegue a puerto |
| 1 | 0:10–0:38 | `enlace.pnc` | Iniciar sesión → llega a «Recibir envío» con el rótulo «Usted envía por PNC» → elegir el archivo → «Validar y homologar» → aparece el resultado | «Cada institución entra con su cuenta y sube solo lo suyo, tal como sale de su sistema. En segundos el archivo queda validado, con códigos oficiales y sin datos personales: los nombres y documentos no se guardan.» | Recibir: el archivo llega como es y sale estándar |
| 2 | 0:38–1:00 | `mlopez` | «Revisar pendientes» → en «Mixco zona 1» elegir la propuesta → desaparece de la lista → «Equivalencias» muestra quién la confirmó | «Con una regla que no se rompe: el sistema no adivina. Lo dudoso se propone, lo decide una persona una sola vez, y queda aprendido para todas las instituciones, con nombre y fecha.» | El sistema no adivina: propone, y una persona decide |
| 3 | 1:00–1:24 | (sin sesión) | «Mapa» → buscar «Panajachel» → ficha con censo y pirámide → «Población y crecimiento» → tocar Guatemala → animación sobre la foto satelital → «Red vial» → «Ver en el mapa» de un enlace | «Con eso, el país se consulta hasta el lugar poblado: proyecciones del INE y Censo 2018 de veinte mil lugares. El crecimiento urbano se ve desde el satélite, de 1975 a 2020, y la red vial se trata como un grafo: 707 lugares sin conexión y 83 enlaces que probablemente faltan.» | Territorio: del país al lugar poblado |
| 4 | 1:24–1:44 | `admin` (ventana de teléfono) | «Mi ruta de campo» → «Ver recorrido de demostración» → paradas en orden → «Registrar visita» → pasos de posición, foto y resultado | «En campo, cada operador ve su ruta en el teléfono. Al llegar, el teléfono entrega la posición y la foto confirma que es una vivienda. Si algo no coincide, el supervisor lo ve. La foto no se guarda.» | Campo: cada visita, comprobada |
| 5 | 1:44–2:08 | (sin sesión) | «Precios por rubro» → escribir «arroz» → «Buscar» → pestaña «Mismo producto en varias tiendas» → «Diferencias de 50 % o más» → «Hacer informe de precio de referencia» | «Para precios, FARO lee lo que publican dieciocho tiendas: más de ochenta y cinco mil precios, cada uno con su dirección y su fecha. El código de barras compara el mismo producto sin interpretar nombres, y de una búsqueda sale el informe de precio de referencia.» | Precios: 85,907, cada uno con su evidencia |
| 6 | 2:08–2:28 | `mlopez` | «Tableros» → escribir «mapa y tabla de población en Sololá» → «Armar tablero» → cambiar el filtro → «Publicar» → abrir el enlace en una ventana sin sesión | «¿Hace falta un tablero? Se escribe con palabras, se filtra y se publica: cualquiera lo abre con un enlace. Sin licencias.» | Tableros: de una frase a un enlace público |
| 7 | 2:28–2:50 | (sin sesión) | Botón «Recorrido guiado» → dos pasos → «Antes y ahora» → «Acerca y fuentes» → cierre con logotipo y lema | «Cada pantalla se explica sola, cada cifra tiene su fuente, y FARO se revisa a sí mismo. Software libre, sin servicios de pago, y se levanta con un comando. FARO: que cada dato llegue a puerto.» | Software libre · un comando · github.com/tono88/faro-ine-2026 |

La voz suma unas 300 palabras: a ritmo normal caben en 2:40, con margen para las transiciones.

## 3. Reglas para grabar

- **Nunca** mostrar archivos reales de instituciones, ni la carpeta de descargas con sus nombres.
- No mostrar contraseñas: escribirlas con el campo oculto, o entrar antes de empezar la toma.
- Mover el puntero despacio y dejar un segundo quieto antes de cada clic.
- Si una pantalla tarda, cortar en la edición: el video demuestra el flujo, no mide tiempos.
- Las cifras que se dicen (707, 83, 85,907, 18 tiendas, 20 mil lugares) están en la sección «Antes y
  ahora» y en [`09-logros.md`](09-logros.md): si cambian ahí, cambian en el guion.
- Lo que se muestra de trabajo de campo es la **ruta de demostración**, rotulada como tal.
- Créditos al final: fuentes de las imágenes satelitales (NASA GIBS; Sentinel-2 sin nubes de EOX,
  CC BY 4.0), © colaboradores de OpenStreetMap, datos del INE.

## 4. Dos formas de producirlo

| Forma | Qué hace falta | Resultado |
|---|---|---|
| **A. Grabación automática** | Un guion de Playwright que recorre las ocho escenas con el portal en marcha y graba la ventana; `ffmpeg` une las tomas y quema los subtítulos de la tabla | Video sin voz, con subtítulos, reproducible con un comando. La voz se graba aparte y se monta encima |
| **B. Grabación manual** | Grabador de pantalla (OBS o el del sistema) y una persona siguiendo la tabla | Video con voz en una sola toma por escena |

Lo recomendable es A para tener un video de respaldo exacto y repetible, y sumarle la voz del
equipo. En la máquina donde se preparó esta entrega hay `ffmpeg` y Playwright; no hay sintetizador
de voz, así que la narración la graba una persona.

## 5. Videos cortos opcionales (30 a 40 segundos cada uno)

Con las mismas escenas se pueden cortar piezas por tema, para redes o para responder una pregunta
del jurado: **Recibir y homologar** (escenas 1 y 2), **Territorio** (3), **Trabajo de campo** (4),
**Precios** (5), **Tableros** (6). Cada una abre y cierra con el logotipo y el lema.

## 6. Lista de comprobación antes de entregar el video

- [ ] Dura entre 2:00 y 3:00.
- [ ] Se entiende sin sonido (subtítulos) y sin subtítulos (voz).
- [ ] No aparece ningún dato real de una institución ni ninguna contraseña.
- [ ] Las cifras dichas coinciden con el portal.
- [ ] Abre y cierra con el logotipo y el lema.
- [ ] Está en un formato que el INE pueda abrir (MP4, H.264) y pesa menos de lo que admita el
      medio de entrega.
