# Presentación ante el jurado (7 minutos + 3 de preguntas)

Equipo **La Sociedad de Almas** · lema: **«Que cada dato llegue a puerto.»** Abrir y cerrar con él.
Misión, visión y objetivos: [`25-identidad.md`](25-identidad.md).

**La presentación en PDF que pide la nota conceptual está en
[`entregables/FARO-presentacion.pdf`](../entregables/FARO-presentacion.pdf)** (14 diapositivas 16:9 y dos de apéndice,
con capturas tomadas del portal). La misma presentación está en PowerPoint, **editable**, en
[`entregables/FARO-presentacion.pptx`](../entregables/FARO-presentacion.pptx): textos, tablas y formas
nativos, colores en el tema y las notas del orador de cada diapositiva. Las dos se regeneran con
`node scripts/entregables/generar_presentacion.js <url> <usuario> <contraseña>` (el PowerPoint
necesita la biblioteca `pptxgenjs`). Este documento es el
guion hablado.

## El mensaje en una frase

**FARO hace que el INE reciba de cualquier institución datos que puede usar el mismo día, sin
adivinar nada, y deja prueba de cada paso.**

## Cómo está armada (v0.20.0)

La presentación es para la final (27-nov): 7 minutos y 3 de preguntas. **No es el video**: el video
es un respaldo de la demostración ([`16`](16-guion-video.md)). Sigue el orden que mejor funciona
ante un jurado: problema → a quién sirve → respuesta → demostración → por qué adoptarla → petición.

**Qué se cambió respecto de la versión anterior, y por qué:**

| Antes | Ahora | Por qué |
|---|---|---|
| El problema era un solo ejemplo | Los diez problemas del README, con quién planteó cada uno | El portal atiende muchas necesidades, no una |
| No decía a quién sirve cada parte | «Cinco frentes que convergen en una solución», con la dirección del INE de cada uno | La aplicabilidad institucional pesa 20 % y desempata |
| No había un diferenciador explícito | «Autocontenido: el riesgo que el INE no corre» | Es lo que separa a FARO de una solución que depende de servicios externos |
| «Configurable y escalable» estaba implícito | «Crece sin programar», con seis ejemplos concretos | El jurado pregunta cómo se mantiene y cómo crece |
| Cerraba con las limitaciones | Cierra con «Antes y ahora» y una petición: un piloto | Se recuerda el final; las limitaciones pasan al apéndice, listas para las preguntas |
| La matriz de la nota conceptual iba en el cuerpo | Va en el apéndice | Es evidencia para consultar, no para exponer |

## Las diapositivas y el reloj

Cada diapositiva lleva al pie el criterio de evaluación al que responde. En el PowerPoint, cada una
trae en las notas del orador lo que hay que decir.

| # | Diapositiva | Minuto | Criterio que apoya | Qué se dice |
|---|---|---|---|---|
| 1 | Portada | 0:00 | — | Nombre, lema, equipo |
| 2 | Datos que existen, pero no llegan en forma útil | 0:10 | Aplicabilidad (20 %) | Diez necesidades con la misma raíz; nombrar tres y quién las planteó |
| 3 | Cinco frentes que convergen en una solución | 0:40 | Aplicabilidad | Cada dirección y su frente; no son cinco herramientas, es una |
| 4 | Un solo camino para el dato, y una regla fija | 1:00 | Innovación (15 %) | Pide, recibe, homologa, aprovecha; el sistema no adivina |
| 5–9 | Demostración (registros, territorio, campo, precios, tableros) | 1:20–4:20 | Funcionalidad (25 %) y usabilidad (10 %) | **En vivo**, tres minutos. Las diapositivas son el respaldo si el portal falla |
| 10 | Autocontenido: el riesgo que el INE no corre | 4:20 | Aplicabilidad | Sin licencias, sin servicios de pago, sin IA externa para funcionar; cinco riesgos evitados |
| 11 | Crece sin programar | 5:00 | Calidad técnica (20 %) | Una institución nueva es un archivo; las reglas son parámetros |
| 12 | Se revisa a sí mismo y dice sus límites | 5:35 | Rigor (10 %) | INE fuente principal, secreto estadístico, verificación |
| 13 | Antes y ahora, y la petición | 6:10 | Aplicabilidad | Seis cifras; **un piloto con una institución, en un servidor del INE, un mes** |
| 14 | Cierre | 6:50 | — | **Mención de inclusión, una frase** (ver abajo); lema y preguntas |
| 15 | *Apéndice:* cumple cada lineamiento | preguntas | Calidad técnica | Ocho lineamientos con su prueba |
| 16 | *Apéndice:* lo que todavía no hace | preguntas | Calidad técnica | Limitaciones y mejoras, dichas por el equipo |

**Sobre el pilar de autonomía.** Se plantea como reducción de riesgo para el INE, sin atacar a
nadie: «las API de inteligencia artificial son útiles; nuestra elección es que el INE pueda repetir
el mismo resultado dentro de cinco años sin depender de un tercero». Lo que es exacto decir: el
portal, para funcionar, no llama a ninguna IA ni servicio de pago. Lo que **no** hay que decir: «no
usamos inteligencia artificial», porque el equipo programó con un asistente de IA (declarado en el
README) y el clasificador de fotos es un modelo, aunque local.

**La mención de inclusión (v0.20.0).** Va al final, en una frase, sobre la diapositiva de cierre,
que la lleva escrita: «La pantalla también se lee en inglés, y más de un tercio de sus rótulos ya
está en kaqchikel como borrador, armado con vocabulario escrito por hablantes y pendiente de
revisión experta. Se traduce la pantalla, nunca los datos». Si hay diez segundos, mostrarlo en
vivo: el botón del globo, kaqchikel, el menú cambia y aparece el aviso de borrador; señalar que
los nombres de los municipios no cambiaron. Lo que **no** hay que decir: «el portal está traducido
al kaqchikel». Es un borrador de 680 rótulos (36 %) que ninguna persona hablante ha revisado; lo
defendible es que no tiene palabras inventadas (cada una consta en la memoria de Mozilla o en el
diccionario de la Academia de Lenguas Mayas), que la pantalla dice que es un borrador y que hay
una hoja de revisión lista para una persona experta ([`29`](29-idiomas-e-inclusion.md)).

**Ensayo.** Tres pasadas con reloj. Decidir quién maneja el portal y quién habla, y en qué momento
se cambia al video si la demostración falla. Dejar el portal preparado con
`scripts/entregables/preparar_demo.js` y las ventanas ya abiertas.

## Lo que nos distingue (decirlo explícitamente)

1. **Responde a las cuatro necesidades de la Gerencia y a las propuestas de las direcciones**, con el estado de cada una a la vista, incluido lo que no se hizo (`docs/01-necesidades-ine.md`).
2. **Nada inventado**: cada dato tiene fuente, fecha, licencia y huella; cada tabla se regenera con un comando.
3. **Encontró errores en datos oficiales y en sí mismo**: 16 códigos de municipio inexistentes en las proyecciones; y una prueba hacia atrás mostró que su propio margen de proyección acertaba 45 % de las veces cuando prometía 95 %: se corrigió y ahora cumple.
4. **Explicable de principio a fin**: reglas fijas y a la vista; el mismo dato da siempre el mismo resultado. La única pieza entrenada es el clasificador de fotos, que corre en el servidor y nunca decide.
5. **Listo para usarse**: usuarios, roles, grupos, una cuenta por institución que solo sube lo suyo, recorridos guiados en cada pantalla, manual gráfico, bitácora, auditoría de seguridad atendida, y funciona en celular.
6. **Cumple y lo demuestra**: secreto estadístico (solo agregados, conteos pequeños protegidos), datos personales disociados antes de guardar, y un análisis norma por norma que dice también lo que falta (`docs/23`).
7. **Llega hasta el campo**: de la asignación del operador a la visita comprobada con posición, foto, satélite y censo.
8. **Incluye sin fingir**: la pantalla se lee en inglés, y en kaqchikel tiene un borrador de más de un tercio de sus rótulos, armado con vocabulario escrito por hablantes y rotulado como pendiente de revisión experta. Se traduce la pantalla, nunca los datos.

## Cifras para citar

- 20,036 lugares poblados con población, hogares y viviendas del Censo 2018; 340 municipios; 22 departamentos.
- 707 lugares sin conexión vial mapeada y 83 enlaces probables (51 km).
- 85,907 precios de 18 tiendas; 16 mil productos comparables por código de barras; 3 de cada 4 diferencias grandes son ofertas.
- Tasas de extracción de la FAO que coinciden con la Hoja de Balance 2024 del INE (trigo: 72 %).
- 1,020 proyecciones comprobadas contra lo ocurrido; 6 tamaños de pantalla × 24 secciones; 20 hallazgos de seguridad, 16 corregidos.
- 29 indicadores por municipio, quince de ellos del Censo 2018 (alfabetismo 81.5 %, internet 29.3 %, piso de tierra 26.5 %).
- 93,297 hectáreas construidas en 2020; solo 1.6 % es no residencial.
- Curvas de población contra el INE: error mediano de 1.05 % (exponencial) y 0.17 % (logística) a cinco años.
- Clasificador de fotos: 11 aciertos, 0 errores y 5 no concluyentes en 16 fachadas.
- Idiomas: 1,873 textos de la pantalla en inglés; 680 rótulos en kaqchikel (36 %), en borrador: 110 idénticos a la fuente y 570 compuestos con sus palabras, sin revisión de hablantes; 2,394 textos que eran datos quedaron fuera del diccionario; ningún dato traducido.

## Preguntas probables y respuesta corta

| Pregunta | Respuesta |
|---|---|
| ¿Usa inteligencia artificial? | Casi todo son reglas fijas y a la vista; hasta los tableros «en lenguaje natural» son reglas que dicen qué entendieron. La excepción es el clasificador de fotos del trabajo de campo: un modelo de escenas ya entrenado (Places365, MIT) que corre en el propio servidor, no guarda la foto y solo marca una visita para revisión. Se usó un asistente de IA para programar, y está declarado. |
| ¿Usaron Landsat? | Sí, de dos formas indirectas: las cifras de superficie construida vienen del GHSL, que la Comisión Europea obtiene de Landsat y Sentinel-2; y la animación se dibuja sobre mosaicos Landsat de NASA. Landsat ve cuadros de 30 m: no distingue techos. El techo lo tomamos del Censo 2018, que lo preguntó hogar por hogar. |
| ¿Por qué una recta y no una exponencial? | Lo medimos. Para la superficie construida la recta acertó más (error mediano de 8.9 % frente a 14.2 %). Para la población usamos las proyecciones del INE; después de 2030 ofrecemos exponencial a corto plazo y logística a largo plazo, rotuladas como no oficiales. |
| ¿Cumple con la ley? | Hicimos el análisis norma por norma (`docs/23`): secreto estadístico de la Ley del INE, acceso a la información, RENAP, contrataciones, delitos informáticos y licencias. Dice también lo que falta, como la firma electrónica de las constancias. Debe revisarlo la asesoría jurídica del INE. |
| ¿Cómo sabe dónde se hizo una encuesta? | El teléfono del operador entrega su posición al registrar la visita. El portal la compara con el punto del lugar poblado, con lo construido en ese kilómetro y con la foto. Si algo no coincide, avisa; decide el supervisor. |
| ¿Y los datos personales? | Los archivos reales se procesan en un temporal que se borra siempre; la salida no lleva nombres ni identificadores; en el repositorio no hay ninguno. |
| ¿Cómo se relaciona con X-Road? | X-Road transporta; FARO define qué se pide y comprueba lo que llega. Se complementan. |
| ¿Qué tan difícil es agregar una institución? | Un archivo de solicitud con sus columnas. Sin programar. |
| ¿Está en idiomas mayas? | En kaqchikel hay un **borrador** de 680 rótulos (36 % de la pantalla: menú, botones, títulos). No hablamos kaqchikel, así que lo armamos solo con palabras que aparecen en textos escritos por hablantes: la memoria de traducción de Mozilla y el diccionario de la Comunidad Lingüística Kaqchikel de la Academia de Lenguas Mayas. Un programa comprueba que no haya ninguna palabra fuera de esas fuentes. Lo que no podemos garantizar es la gramática: por eso la pantalla dice que es un borrador y hay una hoja de revisión lista para una persona experta. Otro idioma se suma igual: es un archivo. |
| ¿Tradujeron los datos? | No. Se traduce la pantalla; los nombres de lugares, las cifras y las citas salen como los publica su fuente, y lo que se descarga es idéntico en cualquier idioma. Hay una prueba automática que lo comprueba. |
| ¿Quién tradujo? | El equipo, con el asistente de IA declarado en el README: el inglés sobre un glosario fijo; el kaqchikel, solo con vocabulario de las fuentes citadas. Quedaron como diccionarios en el repositorio; el portal no llama a ningún servicio de traducción. Ninguno tiene revisión profesional, y el kaqchikel está rotulado como borrador. |
| ¿Y si se equivoca al homologar? | Solo asigna con coincidencia exacta y única o con una decisión humana registrada, que se puede revertir. |
| ¿Sus proyecciones son confiables? | Lo medimos: al proyectar diez años, 8 de cada 10 veces el error fue de 20 % o menos. Ese es el margen que el portal muestra. |
| ¿Los precios publicados sirven de referencia? | Como antecedente: son fuentes secundarias y el informe lo dice. La regla de la Dirección de Precios pide más de 50 % de cotizaciones directas. |
| ¿Es seguro? | Se auditó dos veces, la segunda por una revisión independiente del código el día 3: lo encontrado se corrigió (entre otras cosas, una forma de despejar una celda protegida por resta) y lo pendiente está escrito en `docs/21`. |
| ¿Hay contraseñas en el código? | No. Las claves viven en `.env`, que no se versiona. La contraseña inicial del administrador la da la instalación (`admin` en el despliegue con Docker, como dice el README) y el portal no deja ver ni cambiar nada interno hasta que se cambia. |
| ¿De quién es el código? | Del equipo, que conserva la autoría; el INE recibe una licencia no exclusiva, gratuita, perpetua e irrevocable, como fijan los términos (`LICENCIA.md`). |
| ¿Por qué los cambios del repositorio los firma un asistente? | Porque el equipo trabajó dirigiendo un asistente de IA en una sesión en la nube, que es quien registra los cambios; está declarado en el README. El equipo definió qué construir, revisó, corrigió y probó con los archivos reales, y puede explicar cada regla (`docs/28`). |
| ¿Cumple lo que pide la nota conceptual? | Lineamiento por lineamiento, con la prueba de cada uno: `docs/26`. |
| ¿Qué no hace todavía? | Guatecompras (bloquea clientes automáticos), comparación con censos anteriores (no hay fuente descargable), el ICC y el Manual del IPC. |
| ¿Qué necesita el INE para adoptarlo? | Un servidor con Docker, definir sus solicitudes y crear sus usuarios. |
