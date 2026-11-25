# Presentación ante el jurado (7 minutos + 3 de preguntas)

Equipo **La Sociedad de Almas** · lema: **«Que cada dato llegue a puerto.»** Abrir y cerrar con él.
Misión, visión y objetivos: [`25-identidad.md`](25-identidad.md).

**La presentación en PDF que pide la nota conceptual está en
[`entregables/FARO-presentacion.pdf`](../entregables/FARO-presentacion.pdf)** (13 diapositivas 16:9,
con capturas tomadas del portal). Se regenera con
`node scripts/entregables/generar_presentacion.js <url> <usuario> <contraseña>`. Este documento es el
guion hablado.

## El mensaje en una frase

**FARO hace que el INE reciba de cualquier institución datos que puede usar el mismo día, sin
adivinar nada, y deja prueba de cada paso.**

## Estructura

| Minuto | Tema | Idea que debe quedar |
|---|---|---|
| 0:00–0:40 | El problema, con las palabras de la Gerencia | No hay un estándar para enviar datos; «Puerto de San José» llega escrito de diez maneras |
| 0:40–2:00 | Demostración 1: pedir, recibir, homologar | No adivina; lo que una persona decide queda aprendido para siempre |
| 2:00–2:50 | Demostración 2: territorio | Del país al lugar poblado; la mancha urbana sobre la foto satelital; red vial como grafo |
| 2:50–3:40 | Demostración 3: trabajo de campo, en un teléfono | El operador ve su ruta, el teléfono da la posición, la foto se comprueba y se abre Microsoft Forms ya lleno |
| 3:40–4:40 | Demostración 4: precios | 85 mil precios reales; código de barras; reglas para diferencias; informe de referencia |
| 4:40–5:20 | Demostración 5: tableros | Una frase produce un tablero; se filtra, se publica y cualquiera lo abre con un enlace |
| 5:20–6:10 | Por qué confiar | Se verifica a sí mismo; protege los conteos pequeños; conformidad legal analizada |
| 6:10–7:00 | Antes y ahora, y adopción | La sección «Antes y ahora» en pantalla: 27 puntos con su cifra; se instala con un comando |

## Las diapositivas y el reloj

Cada diapositiva lleva al pie el criterio de evaluación al que responde.

| # | Diapositiva | Minuto | Criterio que apoya | Qué se dice |
|---|---|---|---|---|
| 1 | Portada | 0:00 | — | Nombre, lema, equipo |
| 2 | El problema, con las palabras del INE | 0:10 | Aplicabilidad (20 %) | El mismo municipio llega escrito de cuatro maneras; cinco necesidades del INE |
| 3 | La respuesta | 0:40 | Innovación (15 %) | Pide, recibe, homologa, aprovecha; el sistema no adivina |
| 4 | Demostración 1: registros | 1:00 | Funcionalidad (25 %) | **En vivo**: una institución sube su archivo; el analista decide un pendiente |
| 5 | Demostración 2: territorio | 2:00 | Funcionalidad | **En vivo**: ficha de un municipio, animación, red vial |
| 6 | Demostración 3: campo | 2:50 | Aplicabilidad | **En vivo**, en ventana de teléfono: ruta y visita |
| 7 | Demostración 4: precios | 3:40 | Funcionalidad | **En vivo**: búsqueda, mismo código de barras, informe |
| 8 | Demostración 5: tableros y recorridos | 4:40 | Presentación y usabilidad (10 %) | **En vivo**: una frase → tablero publicado; botón «Recorrido guiado» |
| 9 | Por qué confiar | 5:20 | Rigor en el uso de los datos (10 %) | INE fuente principal, secreto estadístico, verificación |
| 10 | Lo que pide la nota conceptual | 5:50 | Calidad técnica (20 %) | Ocho lineamientos con su prueba |
| 11 | Antes y ahora | 6:10 | Aplicabilidad | Seis cifras; qué necesita el INE para adoptarlo |
| 12 | Lo que todavía no hace | 6:40 | Calidad técnica | Limitaciones y mejoras, dichas por el equipo |
| 13 | Cierre | 6:55 | — | Lema y preguntas |

Las diapositivas 4 a 8 traen capturas: si el portal falla, se narra sobre ellas o se proyecta el
video ([`16-guion-video.md`](16-guion-video.md)). Para la demostración en vivo, dejar el portal
preparado con `scripts/entregables/preparar_demo.js` y las ventanas ya abiertas.

**Apoyo para quien presenta:** la sección «Antes y ahora» del portal (y `docs/22`) trae, para cada
punto, qué se tenía y quién lo planteó, la cifra de FARO y cómo escala. Sirve de guion y de
respaldo en las preguntas.

## Lo que nos distingue (decirlo explícitamente)

1. **Responde a las cuatro necesidades de la Gerencia y a las propuestas de las direcciones**, con el estado de cada una a la vista, incluido lo que no se hizo (`docs/01-necesidades-ine.md`).
2. **Nada inventado**: cada dato tiene fuente, fecha, licencia y huella; cada tabla se regenera con un comando.
3. **Encontró errores en datos oficiales y en sí mismo**: 16 códigos de municipio inexistentes en las proyecciones; y una prueba hacia atrás mostró que su propio margen de proyección acertaba 45 % de las veces cuando prometía 95 %: se corrigió y ahora cumple.
4. **Explicable de principio a fin**: reglas fijas y a la vista; el mismo dato da siempre el mismo resultado. La única pieza entrenada es el clasificador de fotos, que corre en el servidor y nunca decide.
5. **Listo para usarse**: usuarios, roles, grupos, una cuenta por institución que solo sube lo suyo, recorridos guiados en cada pantalla, manual gráfico, bitácora, auditoría de seguridad atendida, y funciona en celular.
6. **Cumple y lo demuestra**: secreto estadístico (solo agregados, conteos pequeños protegidos), datos personales disociados antes de guardar, y un análisis norma por norma que dice también lo que falta (`docs/23`).
7. **Llega hasta el campo**: de la asignación del operador a la visita comprobada con posición, foto, satélite y censo.

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
