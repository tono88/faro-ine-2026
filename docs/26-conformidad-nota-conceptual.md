# Conformidad con la nota conceptual del Datatón INE 2026

*FARO · Que cada dato llegue a puerto.* Equipo La Sociedad de Almas. Revisión del 25-nov-2026.

Este documento recorre la nota conceptual del evento, **punto por punto**, y dice para cada uno
cómo lo cumple FARO y dónde está la prueba. También dice lo que no se cumple del todo y lo que el
equipo todavía debe hacer. Categoría 2: Tecnológica y ciencias de datos.

## 1. Lo esencial

| | |
|---|---|
| Objetivo de la categoría | «Desarrollar una herramienta o sistema funcional que potencie capacidades tecnológicas y de datos del INE» |
| Qué es FARO | Un portal para que el INE pida, reciba y homologue registros administrativos de otras instituciones, y los aproveche junto al censo, el territorio y los precios |
| Tema relevante para el país | El intercambio de datos entre instituciones del Sistema Estadístico Nacional: hoy cada una envía a su manera. Las necesidades las planteó el propio INE en el lanzamiento (`01-necesidades-ine.md`) |
| Equipo | Tres integrantes (la nota pide de 3 a 4) |
| Entrega | Jueves 26-nov-2026, 09:00; presentación el 27-nov (7 minutos + 3 de preguntas) |

## 2. Lineamientos de la categoría 2

| Lineamiento | Cómo se cumple | Dónde se comprueba | Estado |
|---|---|---|---|
| **Datos.** Fuente principal: al menos una base pública del portal del INE | Siete conjuntos del portal de datos abiertos del INE: lugares poblados y cuadros del Censo 2018, proyecciones de población 2015–2030, IPMC, precios de referencia, canasta básica, Hoja de Balance 2024 y diccionario de estadísticas vitales | README, sección «Datos»; `datos/estandar/fuentes.json`; «Acerca y fuentes» en el portal | Cumple |
| Otras fuentes abiertas, citadas correctamente | 39 fuentes con autor, año, título, dirección, licencia y fecha de consulta | `11-fuentes-y-citas.md` (se genera con `faro citas`) | Cumple |
| La documentación indica las bases usadas y su **fecha de descarga** | Cada fila de la tabla «Datos» lleva su fecha; el manifiesto guarda 107 descargas con dirección exacta, fecha y hora, tamaño, licencia y huella SHA-256 | `datos/crudo/manifiesto.jsonl` (versionado); `faro fuentes citar` | Cumple |
| **Tecnología.** Libre elección; se recomienda código abierto o sin costo | Rust, PostgreSQL 16, Leaflet, Docker Compose. Ninguna licencia de pago | `03-arquitectura.md`; `27-componentes-de-terceros.md` | Cumple |
| Declarar servicios de pago o en la nube | No hay servicios de pago. Están declarados los servicios gratuitos que piden el servidor y el navegador, y el uso de GitHub y de un asistente de IA en la nube durante el desarrollo | README, «Servicios en la nube»; `15-instalacion-y-dependencias.md` | Cumple |
| **Desarrollo durante el evento.** No presentar proyectos anteriores | Todo el código se escribió del 23 al 25 de noviembre. Se usan bibliotecas públicas preexistentes, que la nota permite | Historial del repositorio (primer cambio: 23-nov-2026); `08-versiones.md` | Cumple |
| **Control de versiones.** Repositorio Git creado a partir del día 1, con acceso para el jurado | Repositorio en GitHub con el historial completo y una versión publicada por cada bloque de trabajo | <https://github.com/tono88/faro-ine-2026> | Cumple; **falta dar acceso al jurado** (ver sección 8) |
| **Prototipo funcional.** Debe demostrarse en funcionamiento, local o en línea | Corre completo en una computadora con Docker; 24 secciones en uso | `docker compose up -d --build` | Cumple |
| **Reproducibilidad.** Instrucciones de instalación y ejecución, y listado de dependencias | Cuatro pasos en el README; guía detallada; lista de dependencias con licencias. **Probado desde un clon limpio el 25-nov-2026**: imagen construida en 4 minutos, portal arriba con sus datos | README, «Cómo desplegarlo»; `15`; `27` | Cumple (Windows sin probar) |
| **Documentación.** README con los cinco mínimos | Ver la sección 3 | `README.md` | Cumple |
| **Seguridad y confidencialidad.** Respetar el secreto estadístico | Solo se guardan y publican conteos; los menores que 3 se ocultan a quien no es analista, con supresión complementaria aplicada igual en todas las rutas; el archivo original no se guarda | `23-conformidad-legal.md`, sección 3; `celdas_ocultas()` en el servidor | Cumple |
| No intentar identificar personas, hogares o empresas | Nombres y documentos se eliminan o se sustituyen por una llave al recibir. El repositorio no contiene datos personales (revisado: `datos/`, `docs/`, `entregables/`, capturas) | `recepcion.rs`; `23`, sección 2.1 | Cumple |
| No incluir credenciales, contraseñas o llaves en el código | Las claves viven en `.env`, que no se versiona; `.env.example` trae textos de relleno. La contraseña inicial del administrador la da la instalación (`FARO_CLAVE_INICIAL`); `DATABASE_URL` no tiene valor por omisión; los guiones de prueba leen su clave del entorno | `compose.yaml`, `.env.example`, `21-seguridad.md` (fila 32) | Cumple, con una salvedad (ver sección 7) |
| **Inteligencia artificial.** Permitida si se declara; el equipo debe poder explicar y defender el código | Declarado con precisión: un asistente de IA para programar, en una sesión en la nube dirigida por el equipo. En el portal, reglas fijas; un solo modelo, local, que no decide | README, «Uso de inteligencia artificial»; `28-defensa-del-codigo.md` | Cumple |
| **Uso correcto de la información.** Fiel a las definiciones y metadatos del INE; no inducir a error | Las cifras del INE se muestran como las publica; lo que es cálculo propio está rotulado: curvas de población después de 2030 («no oficial»), precios publicados («no es una cotización»), informe de precio («no es un documento oficial del INE»), registros por 100 mil habitantes («no es una tasa anual»), superficie de las vías (solo 38 % anotada), clasificador de fotos (16 fotos de prueba) | Avisos en cada sección; `19-verificacion-estadistica.md` | Cumple |

## 3. Mínimos del README

| La nota pide | Sección del README |
|---|---|
| Problema que resuelve y usuarios a quienes está dirigida | «Los problemas que atiende» y «A quién está dirigido» |
| Arquitectura general y tecnologías empleadas | «Arquitectura y tecnologías» (detalle en `03`) |
| Instrucciones de instalación y uso | «Cómo desplegarlo»; el uso, en el manual gráfico y en los recorridos guiados del portal |
| Conjuntos de datos utilizados | «Datos», con fecha de descarga y licencia de cada uno |
| Limitaciones y posibles mejoras | «Limitaciones y posibles mejoras» |

## 4. Entregables

| La nota pide | Estado |
|---|---|
| Enlace al repositorio con el código fuente y la documentación | Listo: <https://github.com/tono88/faro-ine-2026> |
| Presentación en formato PDF para la exposición | Lista: [`entregables/FARO-presentacion.pdf`](../entregables/FARO-presentacion.pdf), 14 diapositivas y 2 de apéndice; guion en `17`. La misma, editable, en `entregables/FARO-presentacion.pptx` |
| Video demostrativo de 2 a 3 minutos | Clip listo: [`entregables/FARO-video-demo.mp4`](../entregables/FARO-video-demo.mp4), 2:55, grabado del portal en marcha, con subtítulos. **Falta la narración**, que el equipo graba y monta (`16-guion-video.md`) |
| (Adicionales del equipo) | Manual de usuario gráfico (37 páginas), documento técnico, esta matriz, guía para defender el código |

## 5. Términos y condiciones

| Término | Cómo se atiende |
|---|---|
| Los integrantes conservan la autoría y otorgan al INE una licencia no exclusiva, gratuita, perpetua e irrevocable | Escrito en [`LICENCIA.md`](../LICENCIA.md) |
| Las librerías y componentes de terceros conservan sus licencias y se listan en la documentación | [`27-componentes-de-terceros.md`](27-componentes-de-terceros.md): 21 bibliotecas directas y todas las que llegan por ellas, con su licencia; Leaflet, tipografía, base de datos y modelo |
| La herramienta y su documentación citan al INE como fuente de los datos | «Fuente principal: Instituto Nacional de Estadística (INE), Guatemala» en «Acerca y fuentes», en el pie del mapa, en cada sección que usa sus datos y en los reportes; README y `11` |
| Causas de descalificación: proyecto anterior, plagio, uso indebido de los datos, incumplir los lineamientos | Código escrito en el evento; fuentes citadas; ningún dato personal publicado; lineamientos revisados en este documento |

## 6. Criterios de evaluación: dónde mirar

| Criterio | Peso | Qué mostrar | Evidencia |
|---|---|---|---|
| **Funcionalidad** | 25 % | Subir un archivo y verlo homologado; decidir un pendiente; ficha de un municipio; precios; tablero publicado | 96 pruebas automáticas; 21 guiones de pantalla, con clic real; instalación desde cero comprobada; `20-verificacion-interfaz.md` |
| **Aplicabilidad institucional** | 20 % | Responde a necesidades planteadas por la Gerencia y las direcciones; se instala con un comando en un servidor del INE; agregar una institución es escribir un archivo | `01` (estado de cada necesidad, incluido lo no hecho); `22-antes-y-ahora.md` (28 puntos con cifra y cómo escala); `05-mapa-institucional.md` |
| **Calidad técnica** | 20 % | Cinco módulos con responsabilidades separadas; reglas declarativas; documentación numerada | `03`, `28`, `15`, `08`; `cargo test` |
| **Innovación** | 15 % | El sistema no adivina y recuerda; la solicitud como contrato; red vial como grafo; margen de proyección medido, no supuesto; manual que se genera del propio portal | `09-logros.md`; `19` |
| **Rigor en el uso de los datos** | 10 % | INE como fuente principal; cada cifra con fuente y fecha; errores hallados en fuentes oficiales y documentados; rótulos de «no oficial» | `11`, `19`, `06`; manifiesto |
| **Presentación y usabilidad** | 10 % | Recorrido guiado en las 24 secciones; funciona en teléfono; identidad propia; pantalla en inglés y, como borrador con fuente, en kaqchikel (la interfaz, nunca los datos) | Manual gráfico; `20`; `25-identidad.md`; `29-idiomas-e-inclusion.md` |

En caso de empate prevalece la **aplicabilidad institucional**: conviene cerrar la presentación con
«Antes y ahora» y con lo que el INE necesita para adoptarlo.

## 7. Revisión independiente del 25-nov y salvedades

El día 3 se hicieron tres revisiones independientes del repositorio (fuentes y licencias; documentos
contra código; seguridad, secreto estadístico y legalidad). Esto es lo que encontraron y lo que se
hizo.

| Hallazgo | Qué se hizo |
|---|---|
| La contraseña inicial y una cadena de conexión con clave estaban escritas en el programa; los guiones de prueba traían una contraseña fija | Se sacaron del código: las da la instalación o el entorno (`21`, fila 32) |
| Con la contraseña inicial se podía leer lo interno | Ya no: solo se ve lo público hasta cambiarla |
| Una celda protegida se podía despejar combinando la ficha del municipio con el total del departamento | Todas las rutas usan la misma lista de celdas ocultas |
| Una celda protegida se mostraba como cero o como «no llegó nada» | Ahora dice «protegido» y el total del departamento avisa cuántos municipios no cuenta |
| «Registros por 100 mil habitantes» se describía como una tasa del año del registro | Se describe como lo que es: todos los periodos recibidos sobre la población más reciente; no es una tasa anual |
| Dos licencias del INE mal declaradas (precios de referencia es ODC-BY; la canasta básica no declara licencia) y la FAO figuraba como CC BY | Corregidas en `fuentes.json`, `11` y `23` |
| El manifiesto de descargas no estaba en el repositorio | Se versiona, junto con el inventario del portal |
| Faltaban la cita de referencia de GHSL, el aviso ODbL completo y la atribución enlazada de OpenStreetMap | Agregadas |
| El informe de precio podía pasar por un documento oficial | Ahora dice que no lo es |
| Documentos del día 1 seguían describiendo cosas como pendientes | Reescritos o rotulados como plan original (`03`, `04`, `07`, `01`, `02`, `05`, `06`) |
| La bitácora estaba anunciada pero no se podía consultar en pantalla | Se consulta en «Usuarios» |
| No había licencia ni lista de componentes de terceros con sus licencias | `LICENCIA.md` y `27` |

**Salvedades que el equipo debe conocer** (no son incumplimientos, pero un jurado puede preguntar):

1. **Contraseña inicial `admin`.** Es un valor de configuración del despliegue (está en
   `compose.yaml` y en `.env.example`, no en el programa), está documentada, y el portal no deja ver
   ni cambiar nada interno hasta que se cambia. Quien quiera otra la pone en `.env` antes del primer
   arranque.
2. **Los cambios del repositorio figuran firmados por el asistente de IA**, porque el equipo trabajó
   dirigiéndolo en una sesión en la nube. Está declarado en el README. Lo que el equipo aporta y debe
   poder explicar está en `28`.
3. **Los archivos de ejemplo de las instituciones se procesaron en ese entorno de trabajo en la
   nube.** No están en el repositorio ni en ningún documento; se declara en el README y en `06`. Si
   el INE lo pide, conviene confirmar que la sesión y sus archivos se eliminaron al terminar.
4. **El portal de datos del INE** que se usa es `datos.ine.gob.gt`, el portal de datos abiertos del
   INE; la nota nombra `www.ine.gob.gt`.
5. **Windows sin probar**: `iniciar.bat` y los guiones `.ps1`.
6. **Los muestreos de precios no leen `robots.txt`** ni se revisaron los términos de cada tienda;
   vienen apagados (`23`, sección 2.10).
7. **Licencia para terceros.** `LICENCIA.md` deja escrita la licencia al INE. Si el equipo quiere
   además publicar FARO como software libre para cualquiera (por ejemplo, con licencia MIT o
   Apache-2.0), es una decisión suya que aún no está tomada.
8. **Idiomas.** El inglés lo tradujo el equipo con el asistente de IA y no tiene revisión
   profesional. El kaqchikel es un **borrador** que cubre 1.7 % de la pantalla: solo términos
   tomados de una fuente abierta revisada por hablantes, sin traducción automática. No debe
   presentarse como «el portal está en kaqchikel», sino como «está preparado para estarlo y lo
   poco que hay tiene fuente» (`29`).

## 8. Lo que falta hacer antes de entregar

- [ ] **Dar acceso al repositorio a los correos del jurado** (lo pide el lineamiento de control de
      versiones).
- [ ] **Enviar la nota de consentimiento firmada**, si el INE la pidió.
- [ ] **Grabar la narración** del video y montarla sobre el clip (`16-guion-video.md`); entregarlo
      por el medio que indique el INE.
- [ ] **Entregar** el enlace al repositorio y la presentación en PDF antes del jueves 26 a las 09:00.
- [ ] Ensayar la presentación con reloj (7 minutos) y las preguntas de `17` y `28`.
- [ ] Probar `iniciar.bat` en una máquina con Windows, si la demostración se hará ahí.
- [ ] Decidir la licencia para terceros (salvedad 7).
- [ ] (Después de la entrega) Pedir a una persona hablante que complete y revise el kaqchikel; la
      autoridad en su escritura es la Academia de Lenguas Mayas de Guatemala (`29`, sección 4).
