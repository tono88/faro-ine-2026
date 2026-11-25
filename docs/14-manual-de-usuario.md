# Manual de usuario

*FARO · Que cada dato llegue a puerto.*

FARO reúne en una sola pantalla lo que hoy está repartido en correos, hojas de cálculo y portales:
**pedir y recibir registros de otras instituciones, entenderlos sin adivinar, y aprovecharlos junto a
los datos del propio INE**. Se abre en el navegador (<http://localhost:8082>), funciona en laptop,
tableta y celular, y cada cifra dice de dónde sale.

> **Versión gráfica.** Este mismo manual, con una página por sección y la captura de cada pantalla
> marcada paso a paso, está en [`entregables/FARO-manual-de-usuario.pdf`](../entregables/FARO-manual-de-usuario.pdf).

## Qué ve cada quien

| Quién | Qué ve en el menú |
|---|---|
| Cualquier visitante, sin cuenta | Mapa, tableros publicados, territorio (población, red vial, seguridad alimentaria), precios (menos los muestreos), portal del INE, capas y fuentes |
| Cuenta de una institución | Lo anterior, más «Recibir envío» para subir los archivos de su institución |
| Operador de campo | Lo público, más «Mi ruta de campo» |
| Personal del INE (consulta, analista) | Además, según su grupo: los registros administrativos («Recibir envío», «Revisar pendientes», «Equivalencias», «Resultados y avance»), «Operadores de campo», «Antes y ahora», «Proceso», «Muestreos automáticos» y el panel para armar tableros (analista) |
| Administrador | Todo, y «Usuarios» |

## Recorridos guiados

En cualquier pantalla, el botón amarillo **«Recorrido guiado»** del encabezado explica lo que tiene
delante: señala cada parte y dice para qué sirve. Cada sección tiene además su propio botón, «Ver
cómo funciona esta pantalla», junto al título. Se avanza con «Siguiente» o con las flechas del
teclado; Esc lo cierra. Desde el primer paso se puede abrir el recorrido general del portal.

## Idioma de la pantalla

El botón del globo, en el encabezado, cambia el idioma de la pantalla: **español**, **inglés** o
**kaqchikel**. No hace falta iniciar sesión y la elección se recuerda en ese navegador.

- Cambian el menú, los títulos, los botones, las explicaciones y los recorridos guiados.
- **No cambian los datos**: los nombres de lugares, instituciones y productos, las cifras y las
  citas se muestran como los publica su fuente. Lo que se descarga es igual en cualquier idioma.
- El **kaqchikel es un borrador sin revisar**: más de un tercio de los rótulos (menú, botones,
  títulos) está en kaqchikel, armado con vocabulario escrito por hablantes. Puede tener errores:
  falta la revisión de una persona experta. Lo demás aparece en español, y la pantalla lo avisa.
- Para corregir una traducción o sumar otro idioma no hace falta programar: ver
  [`29-idiomas-e-inclusion.md`](29-idiomas-e-inclusion.md).

## Primer ingreso

Entre con `admin` / `admin`. El portal le pedirá cambiar la contraseña en «Mi perfil» y, por
seguridad, no dejará hacer cambios hasta que lo haga. Después cree a las personas en «Usuarios».

| Rol | Puede |
|---|---|
| Administrador | Todo, incluido manejar usuarios y grupos |
| Analista | Recibir envíos, decidir pendientes, manejar precios, muestreos y tableros |
| Consulta | Ver |
| Operador de campo | Ver su ruta y registrar sus visitas; nada más |
| Institución | Ver lo público y subir solo los archivos de las solicitudes de su institución |

**Cuentas de institución.** Al crear un usuario con el rol «Institución» se elige por cuál envía
(aparecen las que tienen alguna solicitud). Esa persona, al entrar, llega a «Recibir envío»: un
rótulo dice «Usted envía por PNC», la lista trae solo sus solicitudes y el historial solo sus
envíos. No ve pendientes ni equivalencias, ni lo que enviaron otras instituciones.

Los **grupos** limitan a qué secciones entra cada quien (por ejemplo, «Precios e índices» no ve la
recepción de registros, y «Operadores de campo» ve solo el mapa y su ruta). El menú de cada persona
muestra solo lo suyo.

**Lo que ve quien no tiene cuenta:** el mapa, los tableros publicados, las cifras agregadas y las
fuentes. No ve en el menú las secciones de recepción ni los paneles para cargar archivos. Los conteos menores que 3 aparecen como «protegido». Lo que llegó de las instituciones
(envíos y pendientes) pide sesión.

## Lo que resuelve, sección por sección

### Registros administrativos: de un archivo cualquiera a un dato oficial
1. **Recibir envío.** Elija la solicitud y suba el Excel o CSV tal como lo tiene la institución. En
   segundos sabe qué filas sirven, cuáles no y por qué. El archivo original no se guarda.
   Cada archivo queda en el historial; en las estadísticas cuenta el más reciente de cada corte
   (mes de registro). Volver a subir el mismo archivo reemplaza al anterior; un envío se puede eliminar.
2. **Revisar pendientes.** Lo que el sistema no quiso adivinar. Usted elige una vez; queda aprendido
   para todas las instituciones, con su nombre y fecha.
3. **Equivalencias.** La memoria institucional: todo lo aprendido, y la carga de tablas de códigos propias.
4. **Resultados y avance.** Qué se pidió, qué llegó, con qué calidad y cómo va cada departamento.

Si un pendiente o un envío lleva la marca **«reglas anteriores»**, se analizó antes de un cambio en
la solicitud: vuelva a subir ese archivo y sus observaciones se actualizan.

### Territorio: el país hasta el lugar poblado
- **Mapa** en tres niveles con buscador, ficha, vista aislada y reporte imprimible. La ficha de un
  municipio trae población, Censo 2018 por lugar poblado, pirámide de edades y crecimiento construido.
- **Población y crecimiento**, con el cálculo repetible paso a paso y márgenes comprobados contra lo
  ocurrido. Al tocar un municipio se ve la animación de su mancha urbana sobre la foto satelital
  (corre una vez; la casilla «Repetir sin parar» la deja en bucle). Para mirar la población después
  de 2030 escriba un año: verá la curva exponencial (corto plazo) y la logística (largo plazo),
  rotuladas como no oficiales.
- **Condiciones de vida** del Censo 2018 en el mapa y en la ficha: quince indicadores por municipio.
- **Red vial**: lugares sin conexión y enlaces que probablemente faltan en el mapa.
- **Operadores de campo**: reparto con la menor distancia total, y armado de rutas (ver abajo).
- **Seguridad alimentaria**: datos de SESAN por departamento y municipio.

### Precios: evidencia para cada precio
- **Precios por rubro** (con su menú): más de 85 mil precios publicados; el mismo producto entre
  tiendas por código de barras; diferencias grandes explicadas o por confirmar; **informe de precio
  de referencia** listo para imprimir, con foto, intervalo de confianza y fuentes.
- **Materiales de construcción** frente a la canasta del IPMC; **precios de referencia** desde
  cotizaciones; **canasta básica** con su histórico; **hoja de balance** con tasas oficiales.
- **Muestreos automáticos**: el portal vuelve a leer las tiendas solo, cuando usted decida.

### Trabajo de campo: de la ruta a la visita comprobada

**Para verlo sin preparar nada:** en «Mi ruta de campo», «Cargar recorrido de demostración» trae seis
lugares poblados reales de Sololá con dos visitas ficticias ya registradas (una que coincide y otra
lejos del punto). En las demás paradas puede registrar una visita paso a paso; si su dispositivo no
entrega la posición, la demostración ofrece una simulada. Todo queda rotulado como demostración.

**Quien coordina** (en «Operadores de campo», un asistente de cinco pasos):
1. Cree a cada encuestador en «Usuarios» con el rol *Operador de campo* y póngalo en el grupo
   «Operadores de campo».
2. En el asistente, paso 1, elija el operador y póngale nombre a la ruta; en el paso 2 elija el
   municipio y marque los lugares poblados. El portal ordena las paradas.
3. Si la encuesta está en Microsoft Forms, pegue su dirección rellenada previamente y cambie los
   valores de muestra por los marcadores `{lugar}`, `{latitud}`, `{longitud}`, `{visita}` (la
   pantalla lo explica paso a paso).
4. Revise el resumen y cree la ruta (paso 4).
5. En «Seguimiento» (paso 5) revise las visitas: cada una dice a qué distancia del punto se hizo, qué mostró la foto y si
   conviene revisarla. «Descargar visitas» da la tabla para unirla con las respuestas de Forms.

**El operador** (en «Mi ruta de campo», desde su teléfono):
1. Ve sus paradas en orden, en la lista y en el mapa. «Cómo llegar» abre la ruta.
2. Al llegar toca **Registrar visita**. El teléfono entrega la posición (pide permiso la primera vez).
3. Toma una foto de la fachada, sin personas. Se reduce en el teléfono, se clasifica y se descarta.
4. Toca **Abrir la encuesta**: Forms se abre con el lugar, la posición y el número de visita ya escritos.

El portal debe estar publicado con HTTPS: los teléfonos no entregan la posición a páginas sin
conexión segura.

### Tableros: de una frase a un tablero publicado
1. **Escriba lo que necesita** —«mapa y tabla de desnutrición y población en Sololá, los 5 más
   altos»— y el portal arma el tablero y le dice qué entendió.
2. **Edítelo en el lienzo.** A la izquierda, las visualizaciones (tarjeta, barras, tabla, mapa,
   dispersión, pirámide, serie, texto) y los datos; a la derecha, las propiedades de la pieza
   elegida. Arrastre una pieza sobre otra para moverla. Hay deshacer y rehacer.
3. **Filtre.** La barra de filtros actúa sobre todo el tablero: departamento, municipio y
   condiciones («población de 50,000 o más»). También puede pedirlo con palabras: «solo Quiché».
4. **Guarde y publique.** «Guardar» deja un borrador que solo ven usted y los administradores.
   «Publicar» lo pone a la vista de cualquier visitante, con un enlace para compartir. Quien lo
   abre puede mover los filtros sin cambiar lo publicado.

### Datos y transparencia
**Antes y ahora** (qué se tenía, qué hace FARO y cómo escala, punto por punto), **portal del INE**
(conjuntos vigilados), **capas publicadas** para cualquier SIG, **proceso** (qué hace cada etapa y
quién) y **acerca y fuentes** (cada dato con su cita, y el resumen de protección estadística).

## Cuatro garantías

- **No adivina.** Asigna un código solo con coincidencia exacta y única, o con una decisión humana registrada.
- **No inventa.** Todo dato es real, con fuente, fecha y licencia; lo calculado se puede repetir con un comando.
- **Se revisa a sí mismo.** Verificación estadística, de interfaz y de seguridad, con sus informes en `docs/19` a `docs/21`.
- **Protege a las personas.** Solo agregados, conteos pequeños ocultos, fotos que no se guardan y una bitácora de quién hizo qué, que los administradores consultan en «Usuarios» (`docs/23`).

<!-- equipo -->
---

<p align="center"><img src="identidad/equipo/carta-de-presentacion.svg" alt="Equipo La Sociedad de Almas: Estuardo Sandoval, Ana Echeverría y Alejandro Sandoval, con el código QR del perfil de LinkedIn de cada uno" width="760"></p>

<p align="center"><a href="https://www.linkedin.com/in/estuardoantoniosandoval/">Estuardo Sandoval</a> · <a href="https://www.linkedin.com/in/analuciaecheverria/">Ana Echeverría</a> · <a href="https://www.linkedin.com/in/alejandro-jos%C3%A9-sandoval-acevedo-998781aa/">Alejandro Sandoval</a><br>Equipo La Sociedad de Almas · Datatón INE 2026</p>
