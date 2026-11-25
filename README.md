<p align="center"><img src="docs/identidad/faro-logo.svg" alt="FARO · Que cada dato llegue a puerto" width="420"></p>

# FARO · Que cada dato llegue a puerto

Datatón INE 2026 · Categoría 2: Tecnológica y ciencias de datos.
Equipo **La Sociedad de Almas**: Estuardo Sandoval · Ana Echeverría · Alejandro Sandoval. Versión 0.18.0.

FARO es un portal para que el INE **pida, reciba y homologue** registros administrativos de otras
instituciones con un formato único, y para aprovechar esos registros junto con el censo, el
territorio y los precios. Corre completo en una computadora, con software libre.

**Para empezar:** [qué se hizo, en una página](docs/00-que-se-hizo.md) ·
[antes y ahora, punto por punto](docs/22-antes-y-ahora.md) ·
[hallazgos con datos reales](docs/09-logros.md) ·
[manual de usuario gráfico (PDF)](entregables/FARO-manual-de-usuario.pdf) ·
[manual en texto](docs/14-manual-de-usuario.md) ·
[conformidad legal](docs/23-conformidad-legal.md) ·
[conformidad con la nota conceptual](docs/26-conformidad-nota-conceptual.md) ·
[presentación (PDF)](entregables/FARO-presentacion.pdf)

## Misión, visión y objetivos

**Misión.** Dar al INE una forma única, verificable y respetuosa de las personas para pedir, recibir
y entender los registros de otras instituciones, y convertirlos en estadística útil junto al censo,
el territorio y los precios.

**Visión.** Un Sistema Estadístico Nacional en el que cada institución entrega sus datos una sola
vez, en un formato común, y el país los aprovecha el mismo día, con la confianza de que nada se
adivinó y todo se puede comprobar.

**Objetivos.**

1. Un estándar para pedir y recibir registros administrativos.
2. Homologar sin adivinar y recordar cada decisión.
3. Proteger a las personas: solo agregados, sin datos personales.
4. Unir registros, censo, territorio y precios en un mismo mapa.
5. Que todo resultado tenga fuente y se pueda repetir.
6. Funcionar con software libre, en un servidor del INE.

Logotipo, colores, tipografía y voz: [`docs/25-identidad.md`](docs/25-identidad.md).

## Los problemas que atiende

El INE planteó varias necesidades en el lanzamiento del Datatón y en las presentaciones de sus
direcciones. Todas tienen la misma raíz: **hay datos que ya existen, pero no llegan en una forma
que se pueda usar**. FARO las atiende una por una.

| Problema | Quién lo planteó | Qué pasa hoy |
|---|---|---|
| **No hay un estándar para enviar registros al INE** | Gerencia | Cada institución manda su propio formato. El mismo municipio llega como "Puerto de San José", "El puerto", "PTO. SAN JOSE" o "San José" (que existe en Escuintla y en Petén); RENAP lo manda como dos números. Cada envío se limpia a mano y lo aprendido se pierde con la persona que lo hizo |
| **Los archivos traen datos personales** | Hallado en los archivos reales | Llegan CUI, nombres y direcciones que el INE no necesita para la estadística y debe proteger |
| **No se sabe qué falta de cada institución** | Coordinación Regional | El avance de la recopilación se lleva aparte, por delegación |
| **El crecimiento urbano no se mide entre censos** | Gerencia | Se pidió detectarlo con imágenes de satélite |
| **Hay caminos sin mapear** | Gerencia | No se sabe qué lugares poblados quedan sin conexión ni dónde falta dibujar un tramo |
| **Los operadores de campo quedan lejos de su puesto** | Gerencia | La asignación no optimiza distancias, y no queda constancia de dónde se hizo cada visita |
| **Los datos del censo no están a la mano por lugar poblado** | Coordinación Regional | Están en decenas de cuadros de Excel, uno por tema |
| **Reunir precios es lento y difícil de respaldar** | Precios e Índices de Referencia; IPP; IPMC | Cotizaciones y catálogos se juntan a mano; comparar el mismo producto entre proveedores obliga a interpretar descripciones |
| **Faltan herramientas propias de cálculo y de tableros** | DESA; Hoja de Balance | Se pidieron tableros con software libre y una calculadora de derivados |
| **Las fuentes oficiales traen errores que nadie avisa** | Hallado por FARO | Códigos de municipio que no existen en las proyecciones, límites intercambiados |

Cómo responde FARO a cada uno, con su cifra: [Antes y ahora](#antes-y-ahora). La lista completa de
necesidades y el estado de cada una, incluido lo que no se hizo: [`docs/01`](docs/01-necesidades-ine.md).

## A quién está dirigido

| Usuario | Qué hace con FARO |
|---|---|
| **Analistas del INE** (Coordinación del SEN, Censos y Encuestas, Índices y Estadísticas Continuas, Precios e Índices de Referencia) | Escriben las solicitudes, reciben los envíos, deciden lo que el sistema no pudo resolver, consultan territorio y precios, arman tableros |
| **Enlaces de las instituciones que envían datos** (RENAP, PNC, SE-CONRED y las demás del Sistema Estadístico Nacional) | Entran con la cuenta de su institución y suben solo sus archivos; ven en el momento qué pasó y qué falló |
| **Coordinación regional y delegaciones** | Ven el avance de la recepción y los datos del censo por municipio y lugar poblado; coordinan rutas de campo |
| **Operadores de campo** | Siguen su ruta en el teléfono y registran cada visita |
| **Personal de consulta y directivos** | Ven lo recibido y los tableros, sin poder cambiar nada |
| **Cualquier persona** | Sin cuenta: el mapa, los tableros publicados, las cifras agregadas, los precios publicados y las capas para descargar, cada una con su fuente |

## FARO en cifras

Todas salen de datos reales procesados durante el evento y se pueden comprobar en el portal.

| | |
|---|---|
| **51,719** filas de RENAP, PNC y SE-CONRED validadas en menos de 3 segundos | **100 %** de los lugares de RENAP y PNC con código oficial; 99.9 % en SE-CONRED |
| **82 → 14** columnas: lo que entra y lo que sale del archivo de defunciones, sin datos personales | **20,036** lugares poblados del Censo 2018, verificados código por código |
| **93,297 ha** construidas en 2020 (eran 30,722 en 1975), para los 340 municipios | **707** lugares poblados sin conexión vial y **83** enlaces que probablemente faltan |
| **85,907** precios publicados por 18 tiendas, leídos por el propio portal | **16,291** productos comparables entre supermercados por su código de barras |
| **29** indicadores por municipio en el mapa y los tableros | **94** pruebas automáticas, 24 secciones revisadas en seis tamaños de pantalla |

## Qué hace

### 1. Registros administrativos (el núcleo)

- **Solicitud.** El INE declara una vez qué columnas pide a cada institución, con qué catálogo,
  cuáles son datos personales y cuál es la fecha de registro (`datos/solicitudes/*.toml`). La
  institución descarga su plantilla.
- **Recepción.** Cada envío se valida contra la solicitud. Reconoce columnas con otro nombre
  ("DEPTO.", "Muni") e informa fila por fila fechas inválidas, vacíos y valores fuera de catálogo.
- **Homologación.** Los lugares se llevan al código oficial. **El sistema no adivina:** asigna solo
  con coincidencia exacta y única, o con una equivalencia que una persona ya confirmó. Lo demás lo
  propone con candidatos.
- **Memoria.** Cada decisión queda guardada (qué texto, qué código, quién, cuándo). El siguiente
  envío no vuelve a preguntar. Vale para lugares y para valores de catálogo (sexo, pueblo, tipo de
  vehículo).
- **Datos personales.** Solo salen las columnas pedidas; el nombre se elimina y el CUI se cambia
  por una llave que no se puede revertir. El archivo original no se guarda.
- **Historial.** Un reenvío reemplaza su propio mes de registro y nada más; el mismo archivo no se
  cuenta dos veces.
- **Avance.** Qué ha llegado de cada institución, de qué periodo y con qué calidad, por departamento.

Secciones: *Recibir envío*, *Revisar pendientes*, *Equivalencias*, *Resultados y avance*.

### 2. Territorio

- **Mapa en tres niveles** (departamento, municipio, lugar poblado) con 29 indicadores, buscador,
  ficha y reporte imprimible por municipio.
- **Censo 2018**: población, hogares y viviendas por lugar poblado; pirámide de edades; quince
  indicadores de condiciones de vida por municipio (alfabetismo, internet, agua, drenaje,
  electricidad, leña, techo, piso, empleo, educación, basura, tenencia) (`faro censo-indicadores`).
- **Población 2015–2030** del INE por municipio, como denominador de tasas. Para mirar más allá de
  2030 hay dos curvas, rotuladas como no oficiales: exponencial a corto plazo y logística a largo plazo.
- **Crecimiento urbano**: superficie construida de 1975 a 2020 (GHSL, que se obtiene de Landsat y
  Sentinel-2), con animación sobre la foto satelital y proyección con su margen medido
  (`faro construido`).
- **Uso del suelo**: superficie residencial y no residencial por municipio y por kilómetro
  (`faro uso-suelo`).
- **Red vial como grafo**: lugares poblados que no quedan conectados y el enlace más corto que
  probablemente falta (`faro vial`).
- **Tipo y superficie de las vías**: kilómetros por municipio de carreteras principales,
  secundarias, locales y de terracería, y cuánta red tiene la superficie anotada; las carreteras
  se ven en el mapa con su tipo y nombre.
- **Seguridad alimentaria**: inseguridad alimentaria aguda por departamento y desnutrición crónica
  en escolares por municipio (SESAN).

Secciones: *Mapa*, *Población y crecimiento*, *Red vial*, *Seguridad alimentaria*.

### 3. Trabajo de campo

- **Asignación de operadores** a puestos con la menor distancia total (método húngaro, exacto).
- **Rutas.** Quien coordina arma la ruta de cada operador con un asistente de cinco pasos
  (operador, paradas, encuesta, revisar y crear, seguimiento); el portal ordena las paradas.
- **Recorrido de demostración**, para ver el proceso paso a paso sin preparar nada: lugares reales
  de Sololá y visitas ficticias, rotuladas.
- **Visitas desde el teléfono.** El operador entra con su usuario, ve sus paradas y registra cada
  visita: el teléfono entrega la posición, una foto del lugar se clasifica y se descarta, y se abre
  la encuesta de **Microsoft Forms** con el lugar, la posición y el número de visita ya escritos.
- **Cruce de cada visita** con cuatro evidencias independientes: posición, foto (¿vivienda o
  comercio?), superficie construida del kilómetro y techo más frecuente del lugar según el censo.
  Si algo no coincide, la visita se marca para revisar; nunca se rechaza sola.

Secciones: *Operadores de campo*, *Mi ruta de campo*. Detalle: [`docs/24`](docs/24-clasificador-de-fotos.md).

### 4. Precios e índices

- **Muestreos automáticos**: 26 tareas leen las páginas públicas de 18 tiendas y guardan cada
  precio con su dirección y fecha. Seis formas de lectura cubren las plataformas más usadas.
- **Precios por rubro**: el mismo producto entre tiendas por su código de barras; las diferencias
  de 50 % o más se explican (oferta, valor atípico) o esperan la confirmación de una persona.
- **Precio de referencia**: media, mediana, moda e intervalo de confianza, con informe imprimible.
- **Materiales de construcción** contra la canasta de 89 artículos del IPMC; 116 proveedores.
- **Canasta básica** del INE junto a los precios publicados.
- **Hoja de Balance de Alimentos**: calculadora de derivados con las tasas de la FAO para
  Guatemala, contrastadas con las cuentas 2024 del INE.

Secciones: *Materiales de construcción*, *Precios de referencia*, *Canasta básica*, *Precios por
rubro*, *Hoja de balance*, *Muestreos automáticos*.

### 5. Tableros, capas y reproducibilidad

- **Tableros.** Se escribe qué se quiere ver («mapa y tabla de desnutrición en Sololá, los 5 más
  altos») y el portal arma el tablero con reglas a la vista. Se edita en un lienzo con paleta de
  visualizaciones, panel de propiedades, deshacer y arrastrar. Los **filtros** (territorio y
  condiciones) se aplican a todo el tablero. Un tablero nace como **borrador** y se **publica**: lo
  publicado lo ve cualquier visitante, con su enlace, y puede mover los filtros sin cambiarlo.
- **21 capas publicadas** en GeoJSON y CSV con dirección estable (`/capas`), para un SIG o un portal.
- **Procedencia.** Cada descarga queda con dirección, fecha, licencia y huella (`faro fuentes`).
- **Cada etapa se repite fuera del portal** con su guion (`scripts/reproducir/`).
- **Antes y ahora**: qué se tenía, qué hace FARO y cómo escala, punto por punto.

Secciones: *Tableros*, *Antes y ahora*, *Proceso*, *Capas publicadas*, *Portal del INE*, *Acerca y fuentes*.

### 6. Cuentas, seguridad y cumplimiento

- **Cuentas** con contraseña, cinco roles (administrador, analista, consulta, operador de campo e
  institución) y **grupos** que definen a qué secciones entra cada persona.
- **Cuentas de institución.** El enlace de PNC, RENAP o SE-CONRED entra con su usuario, ve todo lo
  público y sube solo los archivos de las solicitudes de su institución; no ve lo que enviaron las
  demás. El servidor lo exige en cada pedido, no solo el menú.
- **Recorridos guiados.** El botón «Recorrido guiado», siempre en el encabezado, y un botón en cada
  una de las 24 secciones explican la pantalla paso a paso, señalando cada elemento. Sin biblioteca
  externa; con teclado y en teléfono.
- **Protección estadística.** Los conteos menores que 3 no se muestran a quien no es analista, con
  supresión complementaria para que no se obtengan por resta.
- **Bitácora** de quién hizo qué y cuándo; los administradores la consultan en «Usuarios».
- **Seguridad revisada**: auditoría y correcciones en [`docs/21`](docs/21-seguridad.md).
- **Conformidad legal**: análisis norma por norma en [`docs/23`](docs/23-conformidad-legal.md)
  (Ley del INE, acceso a la información, RENAP, contrataciones, firma electrónica, derecho de
  autor, delitos informáticos y licencias de datos). Es un análisis técnico: debe revisarlo la
  asesoría jurídica del INE.

## Antes y ahora

Los diez puntos principales; los 27 están en [`docs/22`](docs/22-antes-y-ahora.md) y en el portal.

| Antes | Con FARO | Cómo escala |
|---|---|---|
| Cada institución envía con su formato; no hay estándar | Solicitud única y validación al recibir: **51,719 filas** reales en menos de 3 s | Una institución nueva es un archivo de solicitud más |
| Los lugares se corrigen a mano en cada entrega | Código oficial solo sin duda; lo decidido se recuerda: **100 %** en RENAP y PNC | La memoria crece con cada envío |
| Los archivos traen CUI, nombres y direcciones | De **82** columnas salen **14**, sin datos personales | La regla va en la solicitud |
| Se pidió detectar crecimiento urbano con imágenes | Serie 1975–2020 de los 340 municipios: **30,722 → 93,297 ha** | Una capa nueva se procesa con un comando |
| Hay carreteras sin mapear | Grafo de 144,866 km: **707** lugares sin conexión, **83** enlaces probables | Se repite con cada corte de OpenStreetMap |
| Operadores asignados lejos de su casa | Reparto exacto de distancia mínima; ruta, posición y encuesta en el teléfono | Funciona en cualquier teléfono, sin instalar nada |
| Se pidió registrar informantes y precios con alertas | **85,907** precios de 18 tiendas, con alertas de cambios | Una tienda nueva es una tarea más |
| Comparar productos obliga a interpretar descripciones | **16,291** productos iguales por código de barras | Cada tienda que publica el código se suma sola |
| Se pidieron tableros con software libre | Tableros a partir de una frase, con filtros y publicación | Todo indicador nuevo queda disponible |
| Los datos solo pueden usarse con fines estadísticos | Solo agregados, conteos pequeños protegidos, bitácora | El umbral lo fija el INE sin programar |

## Problemas encontrados y resueltos

En los datos:

- **Los códigos de RENAP son los del INE partidos en dos columnas**, y solo valen cuando el país es
  Guatemala. Con eso, los 387 «códigos inexistentes» bajaron a uno.
- **Un archivo mensual trae hechos de más de un siglo** (inscripciones tardías desde 1914): el
  reemplazo se hace por mes de registro, no por mes del hecho.
- **El libro de proyecciones del INE trae 16 municipios con un código que no existe**; FARO los
  ubica con el diccionario del propio libro.
- **La capa de límites trae intercambiados Sanarate y Sansare**, y dos figuras que son lagos.
- **Declarar `-1` como «sin dato» anulaba el `1`** (departamento de Guatemala): ahora los números
  se comparan con su signo.
- **Precios de relleno** (9,999,999) que algunas tiendas publican en productos no disponibles: se descartan.

En el propio portal:

- **El margen de las proyecciones no cumplía lo que decía** (cubría 45 % de los casos): se cambió
  por el error realmente observado en 1,020 casos.
- **Los filtros de un tablero no se aplicaban al tablero ya creado**: ahora son una barra propia
  que actúa sobre todas las piezas, también en la vista publicada.
- **El clic sobre el mapa dejó de abrir la ficha** en algunas pantallas, y más tarde después de ver
  «Lugares poblados» (un lienzo vacío quedaba encima): corregido las dos veces y cubierto por
  pruebas con clic real.
- **Los menús por grupo no ocultaban bien sus botones**: corregido.
- **Los pendientes mostraban observaciones de envíos analizados con reglas anteriores**: ahora
  cada pendiente dice si viene de un envío así, y el envío se vuelve a subir.
- **Comentarios del código que describían un comportamiento anterior**: 45 corregidos tras una revisión.

La lista completa, con la evidencia de cada uno, está en [`docs/09-logros.md`](docs/09-logros.md).

## Cómo desplegarlo

Necesita [Git](https://git-scm.com) y [Docker](https://docs.docker.com/get-docker/) con Docker
Compose (viene incluido en Docker Desktop). No hace falta instalar nada más.

**1. Clone el repositorio y entre a su carpeta.**

```bash
git clone https://github.com/tono88/faro-ine-2026.git
```

```bash
cd faro-ine-2026
```

**2. Cree su archivo de configuración.**

```bash
cp .env.example .env
```

Abra `.env` y cambie las tres claves (`POSTGRES_PASSWORD`, `FARO_TOKEN` y `FARO_SAL`) por valores
propios. En Windows, `iniciar.bat` hace los pasos 2 y 3 y genera las claves.

**3. Construya y levante el portal.**

```bash
docker compose up -d --build
```

La primera vez tarda varios minutos, porque compila el programa. Las siguientes arranca en segundos.

**4. Entre al portal.**

Abra <http://localhost:8082>.

**Usuario y contraseña iniciales:**

| Usuario | Contraseña |
|---|---|
| `admin` | `admin` |

Al entrar, el portal le pide cambiar la contraseña en «Mi perfil» y **no deja hacer ningún cambio
hasta que la cambie**. Los demás usuarios se crean en «Usuarios», con su rol y su grupo.

Sin iniciar sesión se ven el mapa, los tableros publicados, las cifras y las fuentes. Las secciones
que muestran lo recibido de las instituciones («Recibir envío», «Revisar pendientes»,
«Equivalencias», «Operadores de campo») y los paneles para cargar archivos solo aparecen con sesión
y con permiso.

La contraseña inicial no está escrita en el programa: la pone la instalación (`FARO_CLAVE_INICIAL`,
que el despliegue con Docker deja en `admin`). Si quiere otra, cámbiela en `.env` **antes** del
primer arranque. Mientras no se cambie, esa cuenta no puede ver nada interno ni hacer cambios.

Si el puerto está ocupado, cambie `FARO_PUERTO` en `.env` (`iniciar.bat`, en Windows, busca uno libre).

Configuración (archivo `.env`):

| Variable | Para qué | Valor por omisión |
|---|---|---|
| `POSTGRES_PASSWORD`, `FARO_TOKEN`, `FARO_SAL` | Clave de la base, clave de servicio y secreto de las llaves que sustituyen al CUI | Hay que definirlas |
| `FARO_CLAVE_INICIAL` | Contraseña inicial de `admin`; solo se usa al crear la base | `admin` |
| `FARO_PUERTO` | Puerto en esta máquina | 8082 |
| `FARO_ESCUCHA` | Desde dónde se atiende | `127.0.0.1` (solo esta máquina) |
| `FARO_HTTPS` | `1` si se publica detrás de un proxy con HTTPS | 0 |
| `FARO_UMBRAL_CELDA` | Conteo mínimo que se publica a quien no es analista | 3 |
| `FARO_FORMULARIOS` | Servicios admitidos para la encuesta de campo | Microsoft Forms |
| `FARO_SYNC_HORAS` | Cada cuánto se consulta el portal del INE | 24 |

**Para el trabajo de campo el portal debe publicarse con HTTPS**: los teléfonos solo entregan su
posición a páginas seguras. Instalación paso a paso: [`docs/15`](docs/15-instalacion-y-dependencias.md).

Para detenerlo:

```bash
docker compose down
```

Para actualizarlo a la última versión:

```bash
git pull
```

```bash
docker compose up -d --build
```

La línea de comandos también va dentro del contenedor; para ver sus 29 subcomandos:

```bash
docker compose exec faro faro --help
```

Los comandos que regeneran tablas y documentos (`faro verificar`, `faro citas`, `faro mejoras`,
`faro vial`…) escriben en la carpeta del repositorio: se corren desde un clon con Rust instalado
(pasos en [`docs/15`](docs/15-instalacion-y-dependencias.md)), no dentro del contenedor.

**Instalación comprobada.** El 25-nov-2026 se siguió exactamente esta sección desde un clon limpio:
la imagen se construyó en 4 minutos y el portal quedó arriba con sus datos. `iniciar.bat` y los
guiones `.ps1` de Windows no se han podido probar.

Quien quiera compilarlo sin Docker encuentra los pasos en [`docs/15`](docs/15-instalacion-y-dependencias.md).

## Cómo se comprueba

- **Pruebas automáticas** (`cargo test`, para quien compile el código): 94 pruebas de las reglas (homologación, recepción, curvas, rutas, cruce de visitas, protección de celdas).
- **`faro verificar`** vuelve a calcular las cifras y escribe [`docs/19`](docs/19-verificacion-estadistica.md):
  consistencia de las tablas, prueba hacia atrás de las proyecciones, curvas de población,
  intervalos de confianza, umbrales de precios.
- **Instalación desde cero**: clon limpio, pasos de este README y un comando (25-nov-2026).
- **Revisión independiente** de fuentes, documentos, seguridad y secreto estadístico el día 3; lo
  que encontró y cómo se corrigió está en [`docs/26`](docs/26-conformidad-nota-conceptual.md).
- **Guiones de navegador** (`.faro/pruebas/`): recorren el portal como lo haría una persona, en
  laptop, tableta y teléfono ([`docs/20`](docs/20-verificacion-interfaz.md)).
- **Seguridad**: auditoría y correcciones en [`docs/21`](docs/21-seguridad.md).

## Estados de homologación

| Estado | Significado | ¿Se asigna código? |
|---|---|---|
| `exacto` | Coincide con un único nombre oficial (sin importar tildes, mayúsculas o abreviaturas) | Sí |
| `memoria` | Equivalencia ya confirmada por una persona | Sí |
| `sugerido` | Un candidato muy parecido y sin competencia | No, falta confirmar |
| `ambiguo` | Varios lugares con ese nombre y falta el nivel superior | No |
| `revisar` | Hay candidatos, ninguno claro | No |
| `sin_coincidencia` | Nada parecido en el catálogo | No |
| `fuera_de_contexto` | El nombre existe, pero en otro departamento o municipio | No, se muestra dónde existe |
| `codigo_invalido` | Llegó un código que no existe | No; se busca en la lista y queda aprendido |
| `fuera_de_catalogo` | Valor no previsto de una variable (sexo, pueblo...) | No; se elige el valor permitido y queda aprendido |

## Datos

**La fuente principal es el Instituto Nacional de Estadística (INE) de Guatemala.** FARO usa siete
conjuntos de su portal de datos abiertos ([datos.ine.gob.gt](https://datos.ine.gob.gt), el portal
de datos de [www.ine.gob.gt](https://www.ine.gob.gt)); las demás fuentes son abiertas y lo
complementan. Cada fila dice cuándo se descargó. La cita completa de cada una (37 en total) está en
[`docs/11`](docs/11-fuentes-y-citas.md) y en la sección «Acerca y fuentes» del portal.

| Fuente | Uso | Estado |
|---|---|---|
| **INE**, Censo 2018, "Listado de lugares poblados" ([datos.ine.gob.gt](https://datos.ine.gob.gt/es/dataset/censo-2018-lugares-poblados), CC BY), descargado el 23-nov-2026 | Catálogo oficial de lugares con código y coordenadas: fuente principal | Cargado: 20,036 lugares poblados, 19,723 con coordenadas (`faro catalogo importar`) |
| `datos/catalogo/` | 22 departamentos, 340 municipios y 20,036 lugares poblados con código INE y alias | Incluido; verificado código por código contra el listado oficial ([detalle](datos/catalogo/LEEME.md)) |
| **INE**, "Estimaciones y proyecciones de la población total por municipio, 2015–2030" ([datos.ine.gob.gt](https://datos.ine.gob.gt/es/dataset/proyecciones-de-poblacion-2015-2030), ODC-BY), descargado el 24-nov-2026 | Población por municipio y año: denominador de tasas y crecimiento poblacional | Incluido resumido en `datos/referencia/poblacion_municipio.csv` ([detalle](datos/referencia/LEEME.md)) |
| © Colaboradores de OpenStreetMap, exportación de carreteras de HOT en [HDX](https://data.humdata.org/dataset/hotosm_gtm_roads) (ODbL 1.0), corte del 3-nov-2026, descargado el 24-nov-2026 | Red vial como grafo | Resultados derivados en `datos/referencia/red_vial_*` (ODbL); el archivo original no se versiona |
| CONRED / OCHA, límites administrativos de Guatemala ([HDX: cod-ab-gtm](https://data.humdata.org/dataset/cod-ab-gtm), CC BY-IGO), descargado el 24-nov-2026 | Figuras de los municipios para los mapas y para sumar por municipio | Capa simplificada y verificada en `datos/referencia/municipios.geojson` |
| Comisión Europea, JRC, GHSL GHS-BUILT-S R2023A (CC BY 4.0; se cita también Pesaresi et al., 2024, como pide la fuente), descargado el 24-nov-2026 | Superficie construida 1975–2020 para crecimiento urbano | Resumida por municipio en `datos/referencia/superficie_construida_municipio.csv` |
| **INE**, Índice de Precios de Materiales de Construcción ([datos.ine.gob.gt](https://datos.ine.gob.gt/es/dataset/indice-de-precios-de-materiales-de-construccion-ipmc), CC BY), descargado el 24-nov-2026 | Catálogo oficial de 89 artículos, con índice y variación | `datos/catalogo_precios/ipmc_articulos.csv` |
| Páginas públicas de 116 negocios de materiales de construcción, consultadas el 24-nov-2026 | Perfil de proveedores y precios publicados | `datos/referencia/proveedores_construccion.csv` y `precios_construccion.csv`, con la dirección de cada página |
| **INE**, Precios de referencia ([datos.ine.gob.gt](https://datos.ine.gob.gt/es/dataset/precios-de-referencia), ODC-BY), descargado el 24-nov-2026 | Antecedentes: 4,990 precios emitidos de 2016 a 2023 | `datos/catalogo_precios/precios_referencia_ine.csv` |
| SESAN y Grupo Técnico de la CIF Guatemala, análisis de inseguridad alimentaria aguda, octubre 2026 – febrero 2027 ([informe](https://portal.sesan.gob.gt/wp-content/uploads/2026/11/CIF_Guatemala-ct2026-Feb2027.pdf)), descargado el 24-nov-2026 | Personas por fase de la CIF en cada departamento | `datos/referencia/cif_departamentos.csv` |
| SESAN, SIINSAN, "Retardo en talla, Censo 2024" ([libro](https://portal.siinsan.gob.gt/wp-content/uploads/VULNERABILIDAD_DC_MUNICIPAL_QUINTO_CENSO_2024_2.xlsx)), descargado el 24-nov-2026 | Desnutrición crónica en escolares por municipio | `datos/referencia/censo_talla_municipio.csv` |
| **INE**, Censo 2018, cuadros por municipio y lugar poblado (mismo conjunto del listado de lugares; CC BY), descargados el 24-nov-2026 | Población, hogares y viviendas por lugar poblado; quince indicadores de condiciones de vida por municipio; material del techo por lugar poblado | `datos/referencia/censo2018_lugar_poblado.csv`, `censo2018_indicadores_municipio.csv` y `censo2018_techos_lugar_poblado.csv` |
| Comisión Europea, JRC, GHSL GHS-BUILT-S R2023A, componente no residencial (CC BY 4.0), descargado el 24-nov-2026 | Superficie residencial y no residencial | `datos/referencia/uso_construido_municipio.csv` y `uso_construido_bloques.csv` |
| NASA GIBS (mosaicos anuales Landsat) y EOX (mosaico Sentinel-2 sin nubes de 2016, CC BY 4.0) | Foto de fondo de la animación del crecimiento urbano y capa satelital del mapa | No se descargan: las pide el navegador de quien mira. Son referencia visual; no se mide sobre ellas |
| MIT CSAIL, modelo Places365-ResNet18 (pesos CC BY; código MIT), descargado el 24-nov-2026 | Decir si la foto de una visita de campo parece una vivienda o un comercio | `datos/modelos/` ([procedencia](datos/modelos/PROCEDENCIA.md)) |
| **INE**, Precios promedio de la canasta básica alimentaria ([datos.ine.gob.gt](https://datos.ine.gob.gt/es/dataset/precios-promedio-canasta-basica-alimentaria); el conjunto no declara licencia), descargado el 24-nov-2026 | Precio oficial de 34 productos para compararlo con el publicado | `datos/catalogo_precios/cba_precios.csv` |
| **INE**, Hoja de Balance de Alimentos, actualización 2024 (datos.ine.gob.gt; no declara licencia), descargada el 24-nov-2026 | Cuentas de utilización y suministro para contrastar las tasas | `datos/referencia/hba_2024_cuentas.csv` |
| **INE**, diccionario de variables de estadísticas vitales y catálogo del portal (105 conjuntos), 23 y 24-nov-2026 | Códigos «Extranjero» e «Ignorado»; conjuntos vigilados | `datos/crudo/inventario_ine.csv` |
| FAO, factores técnicos de conversión, sección Guatemala (términos de uso de la FAO: no comercial, citando la fuente), descargado el 24-nov-2026 | Calculadora de derivados | `datos/estandar/tasas_extraccion_fao_guatemala.csv` (transcripción traducida) |
| Páginas públicas de 18 tiendas en línea, consultadas el 24-nov-2026 | Precios publicados por rubro | `datos/referencia/precios_publicados.csv` (85,907 precios, cada uno con su dirección) |
| `datos/solicitudes/` | Solicitudes de datos para RENAP (defunciones, nacimientos), PNC (detenidos) y SE-CONRED (emergencias) | Incluido |
| Archivos de ejemplo de instituciones compartidos por el INE (23-nov-2026) | Diseño y prueba de las solicitudes | **No están en el repositorio** (contienen datos personales) |
| Google Research, Open Buildings 2.5D Temporal (CC BY 4.0 / ODbL) | Edificaciones por año 2016–2023 | Opcional: script listo (`scripts/`); la serie que se usa es GHSL ([opciones](docs/10-imagenes-satelitales.md)) |
| `datos/estandar/` | Núcleo común de variables y mapa de registros e instituciones (borrador del equipo) | Incluido |
| `datos/ejemplos/` | Envío, solicitud, serie de edificaciones, operadores y puestos **ficticios** para demostración ([detalle](datos/ejemplos/LEEME.md)) | Incluido |

El repositorio no contiene datos de personas. Los ejemplos son inventados. Cada descarga quedó
anotada en [`datos/crudo/manifiesto.jsonl`](datos/crudo/manifiesto.jsonl) (88 descargas) con su
dirección exacta, fecha y hora, licencia y huella SHA-256; `faro fuentes citar` las lista.

## Arquitectura y tecnologías

```
 navegador (laptop, tableta, teléfono)      una sola página, sin marcos, con Leaflet para los mapas
        │  HTTP: JSON, CSV, GeoJSON
        ▼
 faro-servidor (Rust, axum)                 sesión y permisos · recepción · territorio · precios ·
        │                                   tableros · trabajo de campo · capas · bitácora
        ├── faro-homologa                   leer, validar, homologar, memoria, quitar datos personales
        ├── faro-territorio                 población, crecimiento, red vial, asignación, precios, balance
        ├── faro-fuentes                    descargas con constancia, muestreos, fotos, clasificador local
        └── PostgreSQL 16                   solo conteos, equivalencias, cuentas, tableros, rutas, bitácora
 faro (línea de comandos)                   los mismos módulos: regenera cada tabla y cada informe
```

| Capa | Tecnología |
|---|---|
| Núcleo, servidor y línea de comandos | **Rust** (edición 2021): `axum` y `tokio` para el servidor, `tokio-postgres`, `clap`, `calamine` (Excel), `tract-onnx` (modelo local) |
| Base de datos | **PostgreSQL 16** |
| Pantalla | HTML, CSS y JavaScript sin marcos; **Leaflet 1.9.4** |
| Despliegue | **Docker Compose** (dos contenedores) |
| Pruebas y entregables | `cargo test`; Playwright con Chromium |

Todo es de código abierto y sin costo de licencias. Detalle, base de datos y razones de cada
elección: [`docs/03`](docs/03-arquitectura.md). Bibliotecas y licencias:
[`docs/27`](docs/27-componentes-de-terceros.md).

## Estructura

```
crates/faro-homologa/   núcleo: lectura, solicitud, homologación, memoria, recepción
crates/faro-fuentes/    descargas con procedencia, muestreo de precios, fotos
crates/faro-territorio/ población, crecimiento, red vial, asignación, precios, balance
crates/faro-servidor/   servidor web, base de datos y pantalla
crates/faro-cli/        ejecutable `faro`
datos/catalogo/         catálogo oficial de lugares (código, nombre, nivel, padre)
datos/catalogo_precios/ catálogos de precios (IPMC, canasta, precios de referencia)
datos/estandar/         solicitudes tipo, tareas de muestreo, fuentes y comparativo
datos/referencia/       tablas derivadas de fuentes oficiales y abiertas
datos/solicitudes/      qué pide el INE a cada institución
datos/modelos/          modelo del clasificador de fotos y su procedencia
datos/ejemplos/         archivos ficticios de muestra, rotulados
scripts/                guiones para repetir cada etapa, exportar el modelo, preparar la demostración y generar los PDF
docs/                   documentación (00 a 28)
entregables/            presentación, manual de usuario gráfico y documento técnico, en PDF, y capturas
LICENCIA.md             autoría del equipo y licencia al INE
```

## Documentación

| | |
|---|---|
| [00 Qué se hizo](docs/00-que-se-hizo.md) | [13 Comparación de productos por foto](docs/13-fotos.md) |
| [01 Necesidades del INE](docs/01-necesidades-ine.md) | [14 Manual de usuario](docs/14-manual-de-usuario.md) |
| [02 Investigación](docs/02-investigacion.md) | [15 Instalación y dependencias](docs/15-instalacion-y-dependencias.md) |
| [03 Arquitectura y X-Road](docs/03-arquitectura.md) | [16 Guion del video](docs/16-guion-video.md) |
| [04 Plan](docs/04-plan.md) | [17 Presentación de 7 minutos](docs/17-presentacion-7-minutos.md) |
| [05 Mapa institucional](docs/05-mapa-institucional.md) | [18 Ficha técnica](docs/18-ficha-tecnica.md) |
| [06 Hallazgos con archivos reales](docs/06-hallazgos-datos-reales.md) | [19 Verificación estadística](docs/19-verificacion-estadistica.md) |
| [07 Modelos](docs/07-modelos.md) | [20 Verificación de interfaz](docs/20-verificacion-interfaz.md) |
| [08 Versiones](docs/08-versiones.md) | [21 Seguridad](docs/21-seguridad.md) |
| [09 Logros y hallazgos](docs/09-logros.md) | [22 Antes y ahora](docs/22-antes-y-ahora.md) |
| [10 Imágenes satelitales y uso del suelo](docs/10-imagenes-satelitales.md) | [23 Conformidad legal y protección estadística](docs/23-conformidad-legal.md) |
| [11 Fuentes y citas](docs/11-fuentes-y-citas.md) | [24 Clasificador de fotos](docs/24-clasificador-de-fotos.md) |
| [12 Flujo de proceso](docs/12-proceso.md) | [25 Identidad: logotipo, lema, misión y visión](docs/25-identidad.md) |
| [26 Conformidad con la nota conceptual](docs/26-conformidad-nota-conceptual.md) | [27 Componentes de terceros y licencias](docs/27-componentes-de-terceros.md) |
| [28 Guía para explicar y defender el código](docs/28-defensa-del-codigo.md) | [Autoría y licencia](LICENCIA.md) |
| [Documento técnico (PDF)](entregables/FARO-documento-tecnico.pdf) | [Manual de usuario gráfico (PDF)](entregables/FARO-manual-de-usuario.pdf) |
| [Presentación para el jurado (PDF)](entregables/FARO-presentacion.pdf) | [Presentación editable (PowerPoint)](entregables/FARO-presentacion.pptx) |

## Entregables

| Lo que pide la nota conceptual | Dónde está |
|---|---|
| Repositorio con el código y la documentación | <https://github.com/tono88/faro-ine-2026> (historial desde el 23-nov-2026) |
| Presentación en PDF para la exposición | [`entregables/FARO-presentacion.pdf`](entregables/FARO-presentacion.pdf) (y la misma, editable, en [PowerPoint](entregables/FARO-presentacion.pptx)); guion hablado en [`docs/17`](docs/17-presentacion-7-minutos.md) |
| Video demostrativo de 2 a 3 minutos | Guion y plan de grabación en [`docs/16`](docs/16-guion-video.md); el video se entrega por el medio que indique el INE |
| Además | [Manual de usuario gráfico](entregables/FARO-manual-de-usuario.pdf) · [documento técnico](entregables/FARO-documento-tecnico.pdf) · [matriz de conformidad](docs/26-conformidad-nota-conceptual.md) |

## Limitaciones y posibles mejoras

### Limitaciones

- **Los umbrales son parámetros, no normas del INE**: cuándo un nombre es «muy parecido», cuándo un
  precio es «extremo», qué conteo se protege, a qué distancia una visita está «lejos». Deben
  fijarlos las unidades responsables.
- **Distancias en línea recta.** La asignación de operadores y las rutas no usan tiempo de viaje.
- **La red vial mide el mapa**: un hueco puede ser un camino sin dibujar o un paso en lancha. El
  tipo de vía está en todas, pero la superficie solo en 38 % de la red; no es el inventario oficial
  de la Dirección General de Caminos.
- **Crecimiento urbano a 100 m y cada cinco años.** GHSL mide superficie construida, no viviendas.
  Landsat no distingue techos: para eso hace falta imagen de dron o de menos de un metro.
- **El clasificador de fotos se probó con 16 fotos.** Es una prueba de humo; hay que medirlo con
  fotos de campo del INE. No conoce la arquitectura local y lo dudoso no cuenta en contra.
- **Las curvas de población después de 2030 no son proyecciones oficiales** y solo se pudieron
  probar a cinco años.
- **La protección de celdas no compara tablas entre sí**: falta una herramienta de control de
  divulgación para un uso real.
- **FARO no lee las respuestas de Microsoft Forms**: se unen con las visitas por su número.
- **Precios de construcción**: reglas para 42 de 89 artículos; faltan ferreterías del interior, que
  no publican precios, y los históricos de Guatecompras.
- **Texto libre sin codificar**: causa de muerte, delito y ocupación quedan como texto.
- **Intercambio entre sistemas**: hoy se suben archivos; el diseño prevé recibir lo mismo por
  X-Road ([arquitectura](docs/03-arquitectura.md)).
- **No probado en Windows**: `iniciar.bat` y los guiones `.ps1`.
- **Los muestreos no leen `robots.txt`** ni se revisaron los términos de uso de cada tienda; vienen
  apagados y se detienen si una tienda los rechaza.
- **«Registros por 100 mil habitantes» no es una tasa anual**: suma todos los periodos recibidos.
- **La salida fila por fila de cada envío** (sin nombres ni documentos, con una llave en lugar del
  CUI) queda en un volumen privado del servidor hasta que el envío se elimina: es microdato sin
  identificadores directos, no anónimo, y el INE debe decidir cuánto se conserva.

### Posibles mejoras

1. **Recibir por X-Road** los mismos envíos, sin archivos, con el mismo contrato de contenido.
2. **Publicar con HTTPS** y firmar electrónicamente las constancias de recepción.
3. **Medir el clasificador de fotos** con fotos de campo del INE y, si hace falta, ajustarlo.
4. **Más registros administrativos** ya verificados y con código de municipio: MINEDUC, MSPAS, SAT,
   INAB, SEGEPLAN, incidentes de CONRED y afectaciones de MAGA.
5. **Históricos de Guatecompras** (OCDS) para precios de referencia, desde una red que el sitio admita.
6. **Completar las reglas del IPMC** (hoy 42 de 89 artículos) y sumar cotizaciones directas.
7. **Control de divulgación entre tablas** y codificación del texto libre (causa de muerte, delito).
8. **Tiempos de viaje** en lugar de distancias en línea recta para rutas y asignación.
9. **Respaldo y restauración** de la base con un comando, y recuperación de contraseña.
10. **Seguridad**: bloqueo por origen, HMAC en la llave de pareo, envío en una sola transacción.

## Componentes de terceros

La lista completa, con versión y licencia de cada componente, está en
[`docs/27-componentes-de-terceros.md`](docs/27-componentes-de-terceros.md) (se genera de `Cargo.lock`).
Cada uno conserva su licencia original.

- **Bibliotecas de Rust** de código abierto (MIT o Apache-2.0), 21 usadas directamente: `serde`,
  `serde_json`, `toml`, `csv`, `strsim`, `unicode-normalization`, `chrono`, `clap`, `ureq`, `sha2`,
  `calamine`, `zip`, `regex`, `weezl`, `argon2`, `rand_core`, `axum`, `tokio`, `tokio-postgres`,
  `image` y `tract-onnx`. Versiones exactas en `Cargo.lock`.
- **Mapas**: Leaflet 1.9.4 (BSD de 2 cláusulas), incluido en `crates/faro-servidor/web/vendor/` con
  su licencia. Mapa base opcional: teselas de OpenStreetMap (© colaboradores, ODbL).
- **Tipografía**: Noto Sans (SIL Open Font License 1.1), que el navegador pide a Google Fonts si
  hay internet.
- **Base de datos**: PostgreSQL 16 (licencia PostgreSQL). Imágenes de Docker oficiales `rust`,
  `debian` y `postgres`.
- **Solo para pruebas y para generar los entregables** (no forman parte del portal): Playwright y
  Chromium; `pptxgenjs` (MIT) para la presentación en PowerPoint;
  `pdftotext` para leer un informe de SESAN; PyTorch para exportar el modelo una vez.
- **Modelo de fotos**: Places365-ResNet18, de MIT CSAIL, usado sin cambios (pesos CC BY; código MIT).
  Cita: Zhou, Lapedriza, Khosla, Oliva y Torralba (2017), *Places: A 10 million Image Database for
  Scene Recognition*, IEEE TPAMI.
- **Fotos satelitales de fondo**: NASA GIBS (Landsat) y EOX (Sentinel-2, CC BY 4.0).

Todo el código de FARO se escribió durante el evento: homologación, método húngaro, grafo vial,
lectores de Shapefile y GeoTIFF, muestreo de precios, diseñador de tableros y lo demás.

## Servicios en la nube

FARO no depende de ningún servicio de pago: corre completo en una computadora con Docker. Usa
internet solo para:

- descargar fuentes abiertas (portal del INE, HDX, Comisión Europea) y leer páginas públicas de tiendas;
- mostrar, en el navegador de quien mira, el mapa base de OpenStreetMap, las fotos satelitales de
  NASA y EOX y la tipografía de Google Fonts (todos opcionales);
- bajar la foto de un producto para comparar empaques en el informe de precio (se guarda una huella,
  no la foto);
- abrir la encuesta de Microsoft Forms del INE desde el teléfono del operador (opcional; las
  respuestas no pasan por FARO).

El desarrollo usó GitHub y un asistente de IA en la nube (ver abajo). El guion opcional
`scripts/open_buildings_por_municipio.js` es para Google Earth Engine, un servicio externo que FARO
no usa: la serie de crecimiento urbano que se muestra es la de GHSL.

## Uso de inteligencia artificial

- **En el desarrollo.** El código y la documentación se escribieron con un asistente de IA (Claude,
  de Anthropic), en una sesión en la nube, bajo la dirección y la revisión del equipo: el equipo
  definió qué construir, revisó cada resultado, corrigió el rumbo y probó el portal. **Los cambios
  del repositorio los registró el asistente desde esa sesión; por eso figuran con su firma.** Las
  reglas y las decisiones que el equipo debe poder defender están en
  [`docs/28`](docs/28-defensa-del-codigo.md).
- **Con los archivos de ejemplo de las instituciones.** Los archivos que compartió el INE se
  procesaron con FARO en ese mismo entorno de trabajo privado para diseñar y probar las solicitudes.
  No están en el repositorio ni en ningún documento: solo se conservaron conteos
  ([`docs/06`](docs/06-hallazgos-datos-reales.md)).
- **En el portal.** Una sola pieza usa un modelo entrenado: el clasificador de fotos del trabajo de
  campo (Places365), que corre dentro del servidor, sin nube, y cuyo resultado solo marca una
  visita para revisión. Todo lo demás son reglas fijas y a la vista: el mismo dato de entrada da
  siempre el mismo resultado.

## Licencia y propiedad intelectual

Los integrantes conservan el reconocimiento de la autoría y otorgan al INE una licencia no
exclusiva, gratuita, perpetua e irrevocable para usar, reproducir, modificar, adaptar y distribuir
la herramienta y su documentación para fines institucionales, como fijan los términos del Datatón.
El texto está en [`LICENCIA.md`](LICENCIA.md). Los componentes y los datos de terceros conservan sus
licencias ([`docs/27`](docs/27-componentes-de-terceros.md), [`docs/11`](docs/11-fuentes-y-citas.md)).

## Versiones

Cada versión se publica como *release* de GitHub; el detalle de cada una está en
[`docs/08-versiones.md`](docs/08-versiones.md).
