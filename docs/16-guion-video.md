# Video demostrativo: clip, guion de la narradora y plan de montaje

*FARO · Que cada dato llegue a puerto.* Equipo La Sociedad de Almas.

La nota conceptual pide un **video demostrativo de 2 a 3 minutos, como respaldo en caso de fallas
técnicas durante la demostración en vivo**. No es la presentación: es el portal funcionando.

## 1. Qué hay y qué falta

| Pieza | Dónde | Estado |
|---|---|---|
| Clip de fondo, sin voz | [`entregables/FARO-video-demo.mp4`](../entregables/FARO-video-demo.mp4) · 1920×1080, 30 cuadros por segundo, H.264, **2:55**, con pista de audio en silencio | Hecho |
| Subtítulos con los tiempos reales | [`entregables/FARO-video-demo.srt`](../entregables/FARO-video-demo.srt) | Hecho |
| Línea de tiempo por escena (con el texto de cada una) | [`entregables/FARO-video-demo-tiempos.json`](../entregables/FARO-video-demo-tiempos.json) | Hecho |
| Narración de Ana Lucía | Sección 3 de este documento | **Por grabar y montar** (postproducción del equipo) |

El clip se grabó con el portal en marcha y con los **archivos reales que compartió el INE**
(RENAP, PNC y SE-CONRED), en una base temporal. **En pantalla solo aparecen conteos y nombres de
lugares**: nunca se abre el contenido de un archivo, y la lista con valores por fila que trae el
resultado queda tapada por la banda de cifras. La base temporal se borró al terminar.

## 2. La idea: un caso resuelto, cinco frentes, tres pilares

1. **Gancho (14 s).** El problema en una frase: los datos existen, pero no llegan en forma útil.
2. **Un caso real de principio a fin (38 s).** RENAP sube 10,034 defunciones con su propia cuenta;
   en segundos quedan homologadas y sin datos personales; lo dudoso lo decide una persona, una vez.
3. **Barrido por los otros frentes (68 s).** Territorio, campo, precios y tableros, cada uno con
   el nombre de la dirección del INE a la que sirve.
4. **Los pilares (47 s).** Varios frentes convergen en una sola herramienta; es **autocontenida**
   (sin licencias, sin servicios de pago, sin depender de una IA externa); crece sin programar y
   está comprobada.
5. **Cierre (5 s).** Logotipo y lema.

Por qué así (resultado de la búsqueda de buenas prácticas del 25-nov; se consultaron los resúmenes
de las fuentes, no su texto completo, así que conviene abrirlas antes de citarlas):

- Mostrar **un caso resuelto**, no un recorrido por todas las funciones, es el consejo que más se
  repite para videos de producto y de hackatón (<https://www.twine.net/blog/demo-video-production/>).
- Lo más importante va en los **primeros 20 segundos**, y la atención cae después de los dos
  minutos (<https://wistia.com/blog/optimal-video-length>).
- El guion se escribe **contando palabras**: unas 140 a 150 por minuto
  (<https://sozai.app/es/speaking-rate-by-language/>). Aquí son 327 palabras en 2:55: hay aire para
  que se vea la pantalla.
- La independencia tecnológica se plantea como **reducción de riesgo para la institución**, sin
  atacar a nadie (marco «Public Money, Public Code»: <https://publiccode.eu/>).

## 3. Guion de la narradora (Ana Lucía)

Voz natural, de explicar a un colega; sin prisa. Las cifras se dicen completas. Cada bloque empieza
medio segundo después de su marca de tiempo y debe terminar antes de la siguiente.

| Tiempo | Escena | En pantalla | Narración |
|---|---|---|---|
| 0:00–0:14 | Apertura | Tarjeta azul con el problema; luego el logotipo y el lema | «El INE ya tiene los datos que necesita. Pero cada institución los envía a su manera, y no llegan en una forma que se pueda usar. Para eso construimos FARO.» |
| 0:14–0:40 | Un caso real | RENAP entra con su cuenta («Usted envía por RENAP»), elige la solicitud, sube el archivo y pulsa «Validar y homologar». Banda: 10,034 registros · 100 % · 82 → 15 columnas · 0 datos personales guardados | «Este es un archivo real de RENAP: diez mil defunciones, con nombres y documentos de identidad. La institución entra con su cuenta y lo sube tal como sale de su sistema. En segundos, FARO reconoce las columnas, pone el código oficial a cada lugar y deja fuera los datos personales. El archivo original no se guarda.» |
| 0:40–0:52 | La decisión | Sale RENAP, entra la analista; «Revisar pendientes»; elige «1312 La Democracia» para «La De Mocracia» | «Y cuando hay duda, FARO no adivina: propone, una persona decide una sola vez, y esa decisión queda aprendida para todas las instituciones.» |
| 0:52–1:14 | Frente 2 · Territorio y censo | Busca Panajachel y abre su ficha; animación del crecimiento de Guatemala sobre la foto satelital; red vial | «Lo mismo, en cada frente del INE. Para Censos y Encuestas y las delegaciones: el país hasta el lugar poblado, con población, censo y crecimiento urbano visto desde el satélite. Y la red vial como un grafo: setecientos siete lugares sin conexión mapeada.» |
| 1:14–1:30 | Frente 3 · Trabajo de campo | Un teléfono: la ruta de demostración, «Registrar visita», posición, foto y resultado | «Para el trabajo de campo: cada operador ve su ruta en el teléfono, y cada visita se comprueba con ubicación y foto.» |
| 1:30–1:48 | Frente 4 · Precios e índices | Busca «arroz»; mismo producto en varias tiendas; diferencias de 50 % o más | «Para Precios e Índices: más de ochenta y cinco mil precios leídos de dieciocho tiendas, comparados por código de barras, con su informe de precio de referencia.» |
| 1:48–2:03 | Frente 5 · Tableros | Escribe una frase, arma el tablero, lo guarda y lo publica | «Y para quien necesita comunicar: un tablero se pide con palabras y se publica con un enlace.» |
| 2:03–2:13 | Convergencia | Los cinco frentes, con sus direcciones, confluyen en el faro | «No son cinco herramientas: es una sola. Atacar el problema desde varios frentes los hace converger en una solución integral.» |
| 2:13–2:36 | Autocontenido | Tres ceros (licencias, servicios de pago, llamadas a una IA externa); luego la tabla de riesgo | «Y tomamos una decisión de ingeniería: FARO es autocontenido. Software libre, en un servidor del INE: sin licencias, sin servicios de pago y sin depender de una inteligencia artificial externa que mañana puede cambiar de precio o desaparecer. Los datos se quedan en la institución, y el mismo dato da siempre el mismo resultado.» |
| 2:36–2:50 | Crece sin programar | El archivo de solicitud de una institución; 51,719 registros, 94 pruebas, 1 comando | «Crece sin programar: sumar una institución es escribir un archivo. Y está comprobado: más de cincuenta mil registros reales, noventa y cuatro pruebas automáticas, y se instala con un comando.» |
| 2:50–2:55 | Cierre | Logotipo, lema, equipo y repositorio | «FARO: que cada dato llegue a puerto.» |

**Lo que se dice y lo que no.** «Sin depender de una inteligencia artificial externa» se refiere al
portal en funcionamiento, y es exacto. No decir «no usamos inteligencia artificial»: el equipo usó
un asistente de IA para programar y está declarado en el README; además, el clasificador de fotos
es un modelo, aunque local. Tampoco nombrar ni aludir a otros equipos.

## 4. Montaje (postproducción)

- **Voz.** Grabarla siguiendo el clip o el `.srt`. Si un bloque queda largo, recortar palabras
  antes que acelerar. Nivelar a volumen de voz hablada y dejar el final en silencio un segundo.
- **Subtítulos.** El `.srt` ya está sincronizado con el clip; si la voz se aparta, ajustar sus
  tiempos. Para dejarlos fijos en la imagen:

  ```bash
  ffmpeg -i FARO-video-demo.mp4 -i narracion.wav -vf "subtitles=FARO-video-demo.srt:force_style='FontName=Noto Sans,FontSize=11,BorderStyle=3,Outline=1,Shadow=0,MarginV=10'" -map 0:v -map 1:a -c:v libx264 -crf 20 -c:a aac -shortest FARO-video-final.mp4
  ```

- **Si Ana Lucía aparece en imagen.** El mejor lugar es la esquina **superior derecha** durante las
  escenas del portal (los rótulos van abajo a la izquierda y la banda de cifras ocupa la franja
  inferior de 0:27 a 0:40). En las tarjetas azules conviene quitarla o ponerla a la derecha, que
  queda libre, salvo en la de convergencia y en la del teléfono.
- **Música.** Opcional y baja; debe ser de uso libre y hay que anotar su licencia en el README.
- **Formato de entrega.** MP4 (H.264 y AAC). El clip pesa 16 MB.

## 5. Cómo se grabó y cómo repetirlo

Todo sale de un guion; no hay edición a mano.

```bash
# 1. Una instalación nueva, preparada para demostrar (cuentas y archivos de ejemplo ficticios)
node scripts/entregables/preparar_demo.js http://localhost:8082 <contraseña>
# 2. El clip, los subtítulos y la línea de tiempo
node scripts/entregables/grabar_video.js http://localhost:8082 <contraseña> [config.json]
```

- `grabar_video.js` abre el portal a 1280×720 con escala 1.5 (sale a 1920×1080), captura cada
  cuadro que cambia y lo une con `ffmpeg`. Lleva el **guion** (`GUION`: texto y duración de cada
  escena): al cambiar un texto se ajusta la duración de su escena y se vuelve a grabar.
- Sin configuración usa las cuentas de `preparar_demo.js` y sube el archivo de ejemplo ficticio.
  Con un `config.json` se cambia la cuenta de la institución, la solicitud, el archivo que se sube
  en cámara, el pendiente que se decide y los textos del caso. Así se hizo el clip entregado, con
  los archivos reales; esa configuración no está en el repositorio porque apunta a archivos que
  tampoco lo están.
- La escena del teléfono se graba aparte (412×860) y se compone sobre un fondo.
- Requiere Node, Playwright con Chromium y `ffmpeg`. Internet solo para la foto satelital de la
  animación; sin él, corre sin foto.

## 6. Reglas que cumple el clip

- Ningún dato personal en pantalla: solo conteos, nombres de lugares y valores de catálogo.
- Las contraseñas se escriben en campos ocultos.
- El trabajo de campo usa la **ruta de demostración** del propio portal, rotulada como tal.
- La foto de la visita es una foto de prueba de Wikimedia Commons (chamo estudio, CC BY 2.0), con
  su crédito en pantalla.
- La foto satelital es de NASA GIBS (Landsat) y de EOX (Sentinel-2, 2016, CC BY 4.0); el mapa de la
  ruta, de © colaboradores de OpenStreetMap. Sus créditos se ven en el propio portal.
- Las cifras dichas coinciden con el portal y con [`22-antes-y-ahora.md`](22-antes-y-ahora.md).

## 7. Piezas cortas (opcionales)

Con los tiempos de la sección 3 se recortan clips por tema sin volver a grabar:

```bash
ffmpeg -ss 0:14 -to 0:52 -i FARO-video-demo.mp4 -c:v libx264 -crf 20 -an clip-registros.mp4
ffmpeg -ss 2:13 -to 2:36 -i FARO-video-demo.mp4 -c:v libx264 -crf 20 -an clip-autocontenido.mp4
```

## 8. Antes de entregar

- [ ] La voz cabe en cada bloque y el video sigue durando menos de 3:00.
- [ ] Se entiende sin sonido (subtítulos) y sin subtítulos (voz).
- [ ] No aparece ningún dato personal ni ninguna contraseña (revisado en el clip; revisar otra vez
      si se agrega material).
- [ ] Las cifras dichas coinciden con el portal.
- [ ] MP4 con H.264 y AAC, entregado por el medio que indique el INE.
