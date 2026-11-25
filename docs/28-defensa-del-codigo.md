# Guía para explicar y defender el código

*FARO · Que cada dato llegue a puerto.*

La nota conceptual permite usar asistentes de inteligencia artificial y pide que **el equipo pueda
explicar y defender el código presentado**. Esta guía es el mapa para hacerlo: dónde está cada
cosa, cómo funciona cada regla importante, qué decisiones se tomaron y por qué. Sirve para los
tres minutos de preguntas y para quien revise el repositorio.

## 1. Dónde está cada cosa

| Quiero ver… | Archivo | Función o pieza clave |
|---|---|---|
| Cómo se decide el código de un lugar | `crates/faro-homologa/src/homologar.rs` | `Homologador::homologar`, estados (`Estado`), `UMBRAL_CANDIDATO`, `UMBRAL_SUGERIDO`, `MARGEN` |
| Cómo se comparan los nombres | `crates/faro-homologa/src/normalizar.rs` | `basico`, `clave`, `similitud`, tabla `ABREVIATURAS` |
| Qué se le pide a una institución | `crates/faro-homologa/src/solicitud.rs` y `datos/solicitudes/*.toml` | `Solicitud`, `Columna` (tipo, catálogo, sensible, pareo, corte, país) |
| Cómo se quita el dato personal | `crates/faro-homologa/src/recepcion.rs` | `recibir`, `seudonimo` |
| Dónde se guarda lo que una persona decide | `crates/faro-homologa/src/memoria.rs`; tablas `equivalencias*` | `Memoria` |
| Asignación de operadores | `crates/faro-territorio/src/asignacion.rs` | `asignar` (método húngaro) |
| Red vial y enlaces que faltan | `crates/faro-territorio/src/vial.rs`, `shp.rs` | `Red::construir`, `clasificar`, `enlaces_probables` |
| Proyecciones y sus márgenes | `crates/faro-territorio/src/crecimiento.rs`, `estadistica.rs` | `analizar`, `extrapolar`, `ERROR_OBSERVADO_P80/P95`, `ic_mediana` |
| Superficie construida desde el satélite | `crates/faro-territorio/src/construido.rs`, `geotiff.rs` | lector propio de GeoTIFF |
| Precio de referencia y diferencias | `crates/faro-territorio/src/precios.rs` | `analizar`, `explicar_diferencia` |
| Lectura de catálogos de tiendas | `crates/faro-fuentes/src/muestreo.rs`; `datos/estandar/muestreos.toml` | `ejecutar`, una función por tipo de tienda |
| Comparar productos por foto | `crates/faro-fuentes/src/foto.rs` | `huella`, `se_ven_iguales` |
| Clasificador de fachadas | `crates/faro-fuentes/src/fachada.rs` | reglas `MINIMO_ENTRE_LOS_DOS`, `PARTE_MINIMA` |
| Trabajo de campo | `crates/faro-servidor/src/campo.rs` | `ordenar_ruta`, `cruzar` |
| Tableros desde una frase | `crates/faro-servidor/src/tablero.rs` | `interpretar`, `INDICADORES`, `validar` |
| Quién puede hacer qué | `crates/faro-servidor/src/main.rs` | `sesion_de`, `autorizar_en`, `quien_envia`, `personal_del_ine`, `solo_admin` |
| Secreto estadístico | `crates/faro-servidor/src/main.rs` | `umbral_de_celda`, `ve_conteos_exactos`, `api_registros_municipio` |
| La base de datos | `crates/faro-servidor/src/esquema.sql` | 20 tablas, 2 vistas |
| La pantalla | `crates/faro-servidor/web/index.html` | una función por sección (objeto `cargar`), `RECORRIDOS`, `puede` |
| Cada comando | `crates/faro-cli/src/main.rs` | un subcomando por tabla o informe |

## 2. Las reglas, explicadas en un minuto cada una

**Homologación de lugares.** El texto se normaliza (minúsculas, sin tildes, sin palabras vacías,
abreviaturas declaradas como «Sn» → «San») y se busca en el catálogo **dentro de su contexto** (el
departamento). Si hay una coincidencia exacta y única, se asigna. Si una persona ya confirmó ese
texto, se usa su decisión. En cualquier otro caso **no se asigna**: se calcula el parecido solo
para ordenar las propuestas (candidato desde 0.80; sugerido desde 0.93 y con ventaja de 0.03 sobre
el segundo) y decide una persona. Los números (códigos) se prueban con las lecturas posibles y
solo valen si dan un lugar que existe.

**Datos personales.** La solicitud marca qué columnas son sensibles. Las que no se piden no pasan;
las sensibles se eliminan o se sustituyen por una llave que se obtiene con SHA-256 y un secreto de
la instalación (`FARO_SAL`), de modo que el mismo CUI da la misma llave sin poder volver al CUI.
El archivo se procesa en una carpeta temporal que se borra siempre.

**Corte y reemplazo.** Cada envío se parte por su mes de registro. Un envío nuevo del mismo corte
reemplaza al anterior en las estadísticas (vista `envios_vigentes`); el anterior queda en el
historial como reemplazado. Así nada se cuenta dos veces.

**Asignación de operadores.** Es el problema clásico de asignación: se arma la matriz de distancias
operador–puesto (fórmula del haversine) y se resuelve con el método húngaro, que da el reparto con
la menor suma de kilómetros; no es una aproximación. La distancia es en línea recta.

**Red vial.** Cada tramo de OpenStreetMap es una arista; los tramos que comparten un punto se unen
con una estructura de conjuntos disjuntos (unión–búsqueda). El componente más grande es la red
principal. Para cada lugar poblado se mide la distancia al camino más cercano, y para cada tramo
suelto junto a un lugar se busca el hueco más corto hasta la red principal: ese es el «enlace
probable». Mide el mapa, no el terreno.

**Crecimiento y proyecciones.** Para la superficie construida se ajusta una recta por mínimos
cuadrados desde el año 2000. El margen que se muestra **no es el teórico**: es el error observado
al proyectar diez años hacia atrás en 340 municipios y tres décadas (8 de cada 10 veces, 20 % o
menos; 95 de cada 100, 47 % o menos). Para la población se usan las proyecciones del INE; más allá
de 2030 se ofrece una curva exponencial a corto plazo y una logística a largo plazo, rotuladas como
no oficiales.

**Precio de referencia.** Sigue la regla de la Dirección de Precios e Índices de Referencia: se
calculan media, mediana y moda; si hay precios extremos se recomienda la mediana, si no la media.
El intervalo de confianza de la mediana se calcula por estadísticos de orden (sin suponer
normalidad). El informe avisa si la mayoría de las fuentes son secundarias.

**Diferencias entre tiendas.** Para el mismo código de barras con 50 % o más de diferencia: si el
precio regular explica la diferencia, es una oferta; si hay tres tiendas o más y una sola queda a más de
tres veces de la mediana mientras las demás coinciden, esa es atípica; en otro caso queda por confirmar y lo decide una persona.

**Lectura de catálogos.** Solo direcciones públicas, sin sesión ni claves, con una pausa de 0.6 s
entre consultas y un tope por tarea. Si un sitio bloquea a clientes automáticos, no se insiste ni
se evade: se anota como no disponible. Se guarda el precio, el texto del producto, la dirección y
la fecha.

**Fotos de producto.** Huella de forma (transformada del coseno de la imagen en grises: 64 bits) y
de color (cuadrícula de 4×4). Dos fotos se consideran iguales solo si coinciden ambas. Calibrada
con pares que comparten código de barras: cero falsos positivos.

**Clasificador de fachadas.** Un modelo de escenas ya entrenado (Places365-ResNet18) corre dentro
del servidor. Se suman las probabilidades de las categorías de vivienda y de comercio; solo se
afirma algo si entre las dos reúnen al menos 25 % y una de ellas tiene al menos 65 % de esa suma.
Si no, dice «no concluyente». La foto no se guarda y el resultado solo marca la visita para
revisión: decide el supervisor.

**Tableros desde una frase.** No es un modelo de lenguaje: la frase se normaliza y se buscan en
ella términos del catálogo (indicadores, territorios, tipos de pieza, «los 5 más altos»). La
pantalla muestra qué se entendió y qué no.

**Permisos.** Cada pedido que cambia algo pasa por `autorizar_en(sección)`: exige sesión, rol que
pueda cambiar, que el grupo incluya la sección y que la contraseña inicial ya se haya cambiado.
Una cuenta de institución solo pasa `quien_envia`, que compara su institución con la de la
solicitud. Lo que llegó de las instituciones solo lo lee el personal del INE.

**Secreto estadístico.** A la base solo llegan conteos. Al servirlos, los menores que el umbral se
sustituyen por «protegido» para quien no es analista; si en una fila queda un solo valor oculto, se
oculta un segundo para que no se obtenga por resta.

**Idiomas de la pantalla.** El portal sigue escrito en español. `web/idiomas.js` recorre lo que se
ve y busca cada frase en un diccionario cuya clave es la propia frase en español; antes de buscar
cambia los números por `#` (y los devuelve tal cual) y las etiquetas por marcas numeradas (y las
reutiliza). Si la frase no está, no se toca: por eso un nombre de municipio o de producto nunca
cambia. El kaqchikel es un borrador: cada rótulo consta en una hoja de revisión y cada palabra en
una lista con su fuente; `cargo test` rechaza una palabra que no esté ahí.

## 3. Decisiones que conviene saber defender

| Pregunta | Respuesta |
|---|---|
| ¿Por qué no asignar automáticamente cuando el parecido es muy alto? | Porque una cifra oficial no puede depender de un parecido de texto: «San José» y «San José Pinula» se parecen mucho y son municipios distintos. El costo de una decisión humana se paga una sola vez |
| ¿Por qué Rust? | Un ejecutable único, sin instalar dependencias en la máquina del INE, seguro en memoria; el mismo código sirve al portal y a la línea de comandos |
| ¿Por qué una sola página sin marcos? | Nada que compilar ni mantener del lado del navegador; va dentro del ejecutable y funciona sin internet |
| ¿Por qué lectores propios de Shapefile y GeoTIFF? | Para no exigir GDAL; y porque la biblioteca disponible no leía las teselas vacías del GHSL |
| ¿Por qué una recta y no una exponencial para lo construido? | Se midió: la recta tuvo error mediano de 8.9 % frente a 14.2 % |
| ¿Por qué el margen de proyección es tan ancho? | Porque es el error real medido. El margen teórico era más angosto y acertaba 45 % de las veces cuando prometía 95 %; se sustituyó |
| ¿Por qué el umbral de celda es 3? | Es un valor por omisión razonable y configurable (`FARO_UMBRAL_CELDA`); la norma la fija el INE |
| ¿Por qué admin/admin? | Para que quien instala pueda entrar. El portal obliga a cambiarla y bloquea todo cambio hasta entonces; por omisión solo atiende en la propia máquina |
| ¿Dónde están las claves? | En `.env`, que no se versiona; `.env.example` trae textos de relleno y `iniciar.bat` las genera |
| ¿Por qué no se traducen los datos? | Porque traducir un dato es alterarlo: un nombre oficial tiene un código, y la Ley de Idiomas Nacionales (art. 11) pide respetar la escritura propia de los nombres de lugares. Además, lo que se descarga debe ser idéntico para todos |
| ¿Quién tradujo al kaqchikel y qué tan confiable es? | Es un **borrador sin revisar**. El equipo no habla kaqchikel: los 680 rótulos los armó el asistente usando solo palabras que aparecen en textos escritos por hablantes (la memoria de traducción de Mozilla y el diccionario de la Comunidad Lingüística Kaqchikel de la ALMG), y un programa comprueba que cada palabra conste en esas fuentes. Eso evita palabras inventadas, pero no garantiza la gramática: 570 rótulos son composiciones. Por eso la pantalla lo avisa, hay una hoja de revisión rótulo por rótulo y el siguiente paso es la asesoría de una persona experta |
| ¿Por qué no un servicio de traducción? | Sería una dependencia externa y de pago, enviaría textos fuera del INE y podría dar hoy una traducción y mañana otra. Un diccionario fijo se revisa, se corrige y funciona sin internet |
| ¿Qué pasa si la tienda cambia su página? | La tarea de muestreo falla y lo dice; no se inventa un precio. Cada tipo de tienda tiene su función y su prueba |

## 4. Cómo se comprueba que funciona

| Qué | Cómo |
|---|---|
| Reglas del núcleo | `cargo test`: 96 pruebas (homologación, recepción, curvas, rutas, cruce de visitas, precios, protección de celdas, diccionarios de idiomas) |
| La pantalla | 21 guiones en `.faro/pruebas/` con Chromium: clic real en el mapa, teléfono con posición simulada, permisos por rol, recorridos, carga de precios, idiomas (los datos no cambian al traducir) |
| Seis tamaños de pantalla × 24 secciones | `.faro/pruebas/verificacion_interfaz.js` → [`20-verificacion-interfaz.md`](20-verificacion-interfaz.md) |
| Estadística | `faro verificar` → [`19-verificacion-estadistica.md`](19-verificacion-estadistica.md) |
| Instalación | Clon limpio → `.env` → `docker compose up -d --build` → portal arriba (probado el 25-nov-2026) |
| Archivos reales | RENAP, PNC y SE-CONRED pasados localmente; solo conteos en [`06-hallazgos-datos-reales.md`](06-hallazgos-datos-reales.md) |

## 5. Uso de inteligencia artificial, con precisión

- **Para programar:** un asistente de IA (Claude, de Anthropic) escribió código y documentación
  bajo la dirección del equipo, que definió qué construir, revisó los resultados, corrigió el rumbo
  y probó con los archivos reales. Los commits hechos con el asistente lo indican en su mensaje.
- **En el portal:** no hay modelos de lenguaje ni servicios externos que decidan. La única pieza
  entrenada es el clasificador de fachadas, que es local y no decide.
- **Lo que el equipo debe poder decir sin mirar el código:** las reglas de la sección 2 y las
  decisiones de la sección 3.

<!-- equipo -->
---

<p align="center"><img src="identidad/equipo/carta-de-presentacion.svg" alt="Equipo La Sociedad de Almas: Estuardo Sandoval, Ana Echeverría y Alejandro Sandoval, con el código QR del perfil de LinkedIn de cada uno" width="760"></p>

<p align="center"><a href="https://www.linkedin.com/in/estuardoantoniosandoval/">Estuardo Sandoval</a> · <a href="https://www.linkedin.com/in/analuciaecheverria/">Ana Echeverría</a> · <a href="https://www.linkedin.com/in/alejandro-jos%C3%A9-sandoval-acevedo-998781aa/">Alejandro Sandoval</a><br>Equipo La Sociedad de Almas · Datatón INE 2026</p>
