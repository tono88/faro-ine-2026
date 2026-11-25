# Conformidad legal y protección estadística

**Qué es este documento.** Un análisis técnico del equipo sobre cómo lo construido en FARO se ajusta a
las normas que tocan su materia. **No es un dictamen jurídico**: lo debe revisar la asesoría jurídica
del INE antes de usarse como respaldo. Los textos se abrieron y leyeron el 24-nov-2026; cada cita
lleva su enlace, y al final se dice qué no se pudo comprobar en una fuente oficial.

Abreviaturas de las fuentes más citadas:

- **Ley del INE**: Decreto Ley 3-85 y su reglamento, edición del INE —
  <https://www.ine.gob.gt/ine/wp-content/uploads/2022/01/Ley_Reglamento_INE_dic21.pdf>
- **LAIP**: Decreto 57-2008, Ley de Acceso a la Información Pública —
  <https://idpp.gob.gt/images/Biblioteca-virtual/Leyes_y_Reglamentos/LEY%20DE%20ACCESO%20A%20LA%20INFORMACION%20PUBLICA%20DECRETO%2057-2008.pdf>

## 1. De un vistazo

| Norma | Qué exige, en lo que toca a FARO | Cómo lo atiende FARO | Estado |
|---|---|---|---|
| Ley del INE, arts. 3, 5 y 24 | El INE produce las estadísticas oficiales y las entidades deben darle la información que pida | La solicitud declarativa es la forma de ese pedido; cada envío se valida contra ella | Cumple |
| Ley del INE, art. 25; reglamento, art. 24 | Secreto estadístico: uso solo estadístico y publicación sin nombres | Salida sin datos personales, conteos pequeños protegidos, originales que no se guardan | Cumple, con dos decisiones del INE (sección 4) |
| Constitución, arts. 24, 30 y 31 | Inviolabilidad de documentos, publicidad de los actos y derecho a conocer lo que conste de uno en registros estatales | FARO no conserva datos que identifiquen a una persona; lo que publica son agregados | Cumple |
| LAIP, arts. 9, 22, 30, 31 y 32 | Datos personales y sensibles: minimización, seguridad, y difusión solo con disociación previa | Se pide lo necesario, se marca lo sensible, se disocia antes de guardar | Cumple |
| LAIP, arts. 64 y 65 | Sanciona distribuir, alterar o destruir datos personales | Cuentas por rol y grupo, bitácora, borrado de envíos con constancia | Cumple |
| Ley del RENAP (D. 90-2005), arts. 4, 6 y 61 | El CUI es el identificador único; la dirección de residencia no es pública | El CUI se cambia por una llave no reversible; las direcciones nunca pasan | Cumple |
| Ley de Contrataciones (D. 57-92, reformada por D. 9-2015), art. 8 | El INE elabora y publica precios e índices de referencia | Precio de referencia con su respaldo estadístico e informe imprimible | Apoya la función; ver nota |
| D. 47-2008, arts. 5, 11 y 13 | Las comunicaciones electrónicas valen y se pueden conservar en formato electrónico | Constancia de cada recepción con fecha, huella del archivo y resultado | Cumple en lo básico; sin firma electrónica |
| D. 5-2021, arts. 2, 30 y 31 | Intercambio de información entre dependencias sin convenios adicionales | Un formato común para el intercambio; aplica de forma obligatoria solo al Ejecutivo | Compatible |
| Ley de Derecho de Autor (D. 33-98), arts. 15, 16, 24, 30 y 35 | El programa es una obra protegida; los datos en sí no | Código escrito en el evento, dependencias de licencia libre, autoría del equipo y licencia perpetua al INE (`LICENCIA.md`) | Cumple |
| Código Penal, arts. 274 "A" a "G" | Delitos informáticos: obtener datos sin autorización, alterar registros | Solo se leen páginas públicas, sin saltar bloqueos ni sesiones | Cumple |
| Licencias de los datos (ODbL, CC BY 4.0, portal del INE) | Atribuir, indicar cambios y, en ODbL, compartir igual la base derivada | Cada descarga guarda su licencia; el mapa y las capas citan la fuente | Cumple |
| Principios de la ONU (A/RES/68/261) y Código Regional de la CEPAL | Registros administrativos como fuente válida; confidencialidad | Es el propósito del proyecto | Alineado |

## 2. Norma por norma

### 2.1 Ley Orgánica del INE (Decreto Ley 3-85)

**Obligación de informar (arts. 3, 5 y 24).** La ley dice que las entidades «están obligadas a
suministrar la información que le sea requerida por autoridad competente». FARO no crea esa
obligación: le da forma. La solicitud (`datos/solicitudes/*.toml`) es el pedido escrito —qué
columnas, con qué catálogo, cada cuánto— y la recepción deja constancia de qué llegó y qué no.

**Secreto estadístico (art. 25; reglamento, art. 24).** Los datos que recibe el INE «no hacen fe en
juicio, ni pueden utilizarse para fines tributarios, investigaciones judiciales o cualquier otro
propósito que no sea de carácter estadístico». Lo que FARO hace para respetarlo:

- **No conserva el archivo original.** Se procesa en una carpeta temporal y se borra al terminar,
  también si el proceso falla (la estructura `Temporal` de `api_enviar`, en `crates/faro-servidor/src/main.rs`).
- **Disocia antes de guardar.** Las columnas marcadas `sensible` en la solicitud no pasan: el nombre
  se elimina y el CUI se cambia por una llave (SHA-256 con un secreto de la instalación, `FARO_SAL`).
  Sin el secreto no se puede volver al CUI; **con el secreto, alguien podría probar CUI uno por uno**:
  por eso el secreto se guarda como una contraseña y la llave nunca se publica. En el archivo de defunciones
  de RENAP, de 82 columnas pasan 15.
- **La base guarda conteos, no personas.** La tabla `hechos` solo tiene cuántos casos hubo por
  municipio, periodo y categoría.
- **Queda una salida fila por fila, ya disociada, fuera de la base.** Cada envío deja en el volumen
  privado del servidor (`datos/privado/envios/<n>/homologado.csv`) las columnas pedidas, sin nombres
  ni documentos y con la llave en lugar del CUI: fecha, municipio, sexo, edad y las demás variables
  de la solicitud. Es microdato sin identificadores directos, **no anónimo**: lo protege el secreto
  estadístico, no se sirve por ninguna ruta del portal, no se versiona y se borra al eliminar el
  envío. El INE debe decidir cuánto tiempo se conserva y quién accede a ese volumen.
- **Protege las celdas pequeñas** (sección 3).
- **No repite datos personales en la documentación.** En `docs/` y en el repositorio solo hay conteos.

**Sanciones (arts. 37 a 40; reglamento, art. 31).** Violar el secreto estadístico es infracción
grave. Los controles que ayudan a prevenirla y a deslindar responsabilidades están en la sección 2.4.

### 2.2 Constitución Política (arts. 24, 30 y 31)

El artículo 31 da a toda persona el «derecho de conocer lo que de ella conste en archivos, fichas o
cualquier otra forma de registros estatales». FARO no arma un registro de personas: no hay en su
base un dato que permita buscar a alguien (la salida fila por fila del párrafo anterior tampoco
lleva nombre ni documento). El artículo 30 exceptúa de la publicidad los datos
dados «bajo garantía de confidencia», que es el caso de lo que las instituciones entregan al INE.
Fuente: <https://www.constituteproject.org/constitution/Guatemala_1993.pdf?lang=es> (copia no oficial).

### 2.3 Ley de Acceso a la Información Pública (Decreto 57-2008)

| Artículo | Qué dice | En FARO |
|---|---|---|
| 9 | Define datos personales, datos sensibles y hábeas data. Aclara que «los datos impersonales no identificables, como aquellos de carácter demográfico recolectados para mantener estadísticas, no se sujetan al régimen de hábeas data» | Lo que FARO guarda y publica es de esa clase |
| 22 | Es confidencial lo que otra ley declare así y los datos sensibles | El detalle de los envíos, los pendientes y las equivalencias solo lo ve el personal del INE con sesión; cada institución ve únicamente sus propios envíos. Es público el avance agregado: cuántos archivos y filas ha enviado cada institución y con qué porcentaje de lugares resueltos |
| 30 | «Administrar datos personales sólo cuando éstos sean adecuados, pertinentes y no excesivos» | La solicitud pide columnas concretas; lo que no se pidió se descarta al recibir |
| 31 y 32 | Difundir datos personales exige consentimiento, salvo por «razones estadísticas, científicas o de interés general previstas en ley, previo procedimiento por el cual no puedan asociarse los datos personales con el individuo», o entre entidades del Estado | Ese procedimiento previo es la disociación de la recepción |
| 64 y 65 | Sancionan comercializar, distribuir, alterar o destruir datos personales | Sección 2.4 |

### 2.4 Controles de acceso y constancia

Sirven a la vez a la Ley del INE, a la LAIP y al Código Penal:

- Cuentas personales con contraseña (Argon2), cinco roles (administrador, analista, consulta,
  operador de campo e institución) y grupos que limitan las secciones (`docs/21-seguridad.md`).
- **Mínimo privilegio por institución**: una cuenta de rol «institución» sube únicamente los
  archivos de las solicitudes de su institución y ve únicamente sus propios envíos; pendientes y
  equivalencias quedan para el personal del INE. El servidor lo comprueba en cada pedido.
- La contraseña inicial bloquea los cambios **y la lectura de todo lo interno** (envíos, pendientes,
  equivalencias, conteos exactos) hasta que se cambia. No está escrita en el código: la da la
  instalación.
- El detalle de lo que llegó de las instituciones (envíos, pendientes) pide sesión del personal del
  INE. Lo que sí es público es el avance agregado por institución (archivos, filas y porcentaje
  resuelto), que alimenta la sección «Resultados y avance».
- El nombre con que una persona firma la bitácora y los tableros no puede ser el de otra cuenta.
- **Bitácora** (tabla `bitacora`): quién publicó un tablero, creó una ruta o registró una visita, y
  cuándo. Las equivalencias guardan quién las confirmó. Solo los administradores la leen.
- Borrar un envío es una operación explícita, de analista o administrador, que elimina sus conteos.

### 2.5 Ley del RENAP (Decreto 90-2005)

El CUI es el identificador único de la persona (arts. 4 y 61). El RENAP da información a las
entidades autorizadas, «excepto cuando pueda ser utilizada para afectar el honor o la intimidad del
ciudadano», y la dirección de residencia no es pública (art. 6, literales h y j). En FARO el CUI nunca se guarda: se sustituye por una
llave calculada con una sal secreta (`FARO_SAL`), que permite saber que dos registros son de la
misma persona sin saber de quién. Las direcciones no pasan.
Fuente: <https://www.renap.gob.gt/sites/default/files/informacion-publica/1_decreto_90-2005_ley_del_registro_nacional_de_las_personas.pdf>

### 2.6 Ley de Contrataciones del Estado (Decreto 57-92, reformado por Decreto 9-2015)

El artículo 8 encarga al INE que «elaborará y publicará mensualmente en GUATECOMPRAS y en su página
web, los precios, salarios e índices que se requieran». Las secciones de precios de FARO apoyan esa
función: reúnen precios con su fuente y su fecha, calculan media, mediana, moda e intervalo de
confianza, y producen el informe. **FARO no publica precios oficiales**: prepara el insumo para que
el INE decida.
Fuentes: comparativo del texto antes y después de la reforma
(<https://www.icefi.org/sites/default/files/comparacion_texto_lce_antes_y_despues_de_la_reforma.pdf>)
y reglamento del INE de 2023
(<https://www.ine.gob.gt/wp-content/uploads/2023/09/12-09-2023-Reglamento-precios-de-referencia-contrato-abierto.pdf>).

### 2.7 Comunicaciones y firmas electrónicas (Decreto 47-2008)

«Las comunicaciones electrónicas serán admisibles como medios de prueba» (arts. 5 y 11) y pueden
conservarse en formato electrónico (art. 13). Cada recepción deja fecha, institución, nombre del
archivo, huella y resultado; cada descarga de una fuente deja dirección, fecha, licencia y huella
SHA-256 en `datos/crudo/manifiesto.jsonl` (versionado en el repositorio). **Lo que falta:** FARO no firma electrónicamente las
constancias. Si el INE quiere que valgan como acuse formal, hay que agregar firma electrónica
avanzada con un prestador autorizado.
Fuente: <https://www.5b.com.gt/assets/docs/DecretoLey_47_2008.pdf> (escaneo del Diario Oficial en un sitio privado).

### 2.8 Simplificación de trámites (Decreto 5-2021)

Para el intercambio de información entre dependencias, «no será necesaria la suscripción de acuerdos
o convenios adicionales» (arts. 30 y 31). La ley obliga al Organismo Ejecutivo; las entidades
descentralizadas, como el INE, pueden aplicarla. FARO es compatible con esa vía: lo único que una
institución necesita para enviar es la solicitud y su plantilla. El decreto no menciona X-Road ni
una plataforma de interoperabilidad concreta.
Fuente: <https://mineco.gob.gt/images/simplificacion_tramites/decreto-numero-5-2021.pdf>

### 2.9 Derecho de autor (Decreto 33-98)

El programa de ordenador se protege como obra literaria (arts. 15 y 30). Las bases de datos se
protegen solo por la originalidad de su selección o disposición, y «esta protección no se extenderá
a los datos o material contenido en las compilaciones» (art. 16).

- Todo el código de FARO se escribió durante el evento. Como fijan los términos de la categoría 2,
  los integrantes **conservan el reconocimiento de la autoría** y otorgan al INE una **licencia no
  exclusiva, gratuita, perpetua e irrevocable** para usar, reproducir, modificar, adaptar y
  distribuir la herramienta y su documentación para fines institucionales. Está escrito en
  [`LICENCIA.md`](../LICENCIA.md). (No es una cesión de derechos: esa es la regla de la categoría 1.)
- Las dependencias son de licencia libre (MIT o Apache 2.0 en su mayoría); la lista completa, con la
  licencia de cada una, está en [`27-componentes-de-terceros.md`](27-componentes-de-terceros.md).
- Leaflet y la tipografía se usan con sus licencias (BSD-2 y OFL).
- Los datos de terceros conservan su licencia y su cita (`datos/estandar/fuentes.json`).

Fuente: <https://www.wipo.int/wipolex/es/legislation/details/16159>

### 2.10 Delitos informáticos (Código Penal, arts. 274 "A" a "G")

Castigan, entre otros, a quien «sin autorización, utilice u obtenga para sí o para otro, datos
contenidos en registros informáticos, bancos de datos o archivos electrónicos». Los muestreos de
precios de FARO:

- leen solo páginas que cualquier visitante ve, sin cuenta ni clave;
- no saltan bloqueos, captchas ni inicios de sesión: los sitios que los ponen quedan fuera y se
  anota (Kemik, Platino, Fayco, PriceSmart, entre otros);
- hacen una consulta a la vez, con una pausa de 0.6 s entre consultas y un tope por tarea, y se
  identifican como cliente automático (`faro-muestreo`), sin hacerse pasar por un navegador;
- si una tienda responde 403 o 429 (no admite la consulta o son demasiadas), la tarea se detiene y
  no insiste;
- vienen **apagados**: los enciende una persona con permiso.

**Lo que no se hizo y debe saberse:**

- FARO **no lee `robots.txt`** ni revisó los términos de uso de cada tienda. Antes de encender un
  muestreo de forma periódica, el INE debería revisarlos o, mejor, acordar la consulta con cada
  tienda.
- Para el informe de precio, el servidor **baja la foto del producto** para calcular su huella
  (64 bits de forma y 48 valores de color); guarda la huella y la dirección, no la foto.
- El repositorio trae el levantamiento del 24-nov (`precios_publicados.csv`, 85,907 precios con su
  dirección). Son hechos (qué precio publicó qué tienda y cuándo), no el catálogo ni sus fotos; aun
  así, redistribuirlo a gran escala es una decisión que el INE debe tomar con esa salvedad.

Fuente: <https://mcd.gob.gt/wp-content/uploads/2013/07/Actualizaci%C3%B3n_del_Codigo_Penal_Decreto_17-73.pdf>
(copia con reformas hasta 2012).

**Ley Marco de Ciberseguridad (Decreto 26-2026).** Según notas de la Agencia Guatemalteca de
Noticias, se publicó el 17-nov-2026, entra en vigor 30 días después y agrega ocho delitos. No se
pudo leer el texto oficial. Las medidas de `docs/21-seguridad.md` (cabeceras, control de origen,
límites de intentos, escucha local por omisión, HTTPS configurable) van en la dirección que esa ley
anuncia, pero la conformidad concreta debe revisarse cuando el texto esté disponible.
Fuente: <https://agn.gt/presidente-sanciona-la-ley-marco-de-ciberseguridad>

### 2.11 Protección de datos personales

No se halló una ley general de protección de datos personales vigente en Guatemala; según la prensa,
la iniciativa 6464 seguía en estudio en comisión en julio de 2026. FARO aplica de todos modos los
principios que esas leyes suelen pedir: finalidad (solo estadística), minimización (solo lo
solicitado), disociación (antes de guardar) y seguridad (controles de acceso).

### 2.12 Trabajo de campo (rutas, posición y foto)

Es lo más nuevo y lo más delicado, porque la posición de una visita es la de un hogar:

- **La foto no se guarda.** Se reduce en el teléfono, se clasifica en el servidor y se descarta. En
  la base queda solo la clase («vivienda», «comercio», «otro») y su confianza.
- **La posición se pide al momento de registrar la visita**, con el permiso que da el propio
  operador en su teléfono; no hay seguimiento continuo.
- **Las visitas las ven el operador que las hizo y quien coordina.** No se publican ni entran al mapa.
- **Las respuestas de la encuesta no pasan por FARO**: van a Microsoft Forms, en la cuenta del INE.
  El enlace que FARO abre lleva el número de visita, el lugar, el operador y la posición, para que el
  formulario llegue ya lleno: esos datos sí viajan a Microsoft Forms, dentro de la cuenta del INE.

Lo que debe definir el INE: el aviso al informante y al operador sobre el uso de la posición y de
la foto, y cuánto tiempo se conservan las visitas.

### 2.13 Licencias de los datos usados

| Fuente | Licencia | Qué pide | En FARO |
|---|---|---|---|
| OpenStreetMap (red vial, mapa base) | ODbL 1.0 | Aviso de fuente y licencia en lo que se publique; si se publica una base derivada, compartirla con la misma licencia | El mapa dice «© colaboradores de OpenStreetMap (ODbL)» con enlace a la página de derechos; las tablas y capas `red_vial_*` se publican con ODbL |
| GHSL (Comisión Europea) | CC BY 4.0 | Atribuir, indicar los cambios y citar la publicación de referencia (Pesaresi et al., 2024) | Citado en `fuentes.json`; las tablas dicen que FARO suma las celdas por municipio |
| Mosaico Sentinel-2 de 2016 de EOX | CC BY 4.0 | Atribución con su texto («contiene datos Copernicus Sentinel modificados, 2016») | La capa y la animación la llevan. Los mosaicos de 2018 en adelante son de uso no comercial y **no se usan** |
| Límites COD-AB (CONRED / OCHA) | CC BY-IGO | Atribuir e indicar los cambios | Las capas dicen «FARO, a partir de CONRED / OCHA» y qué se cambió |
| FAO, factores técnicos de conversión | Términos de uso de la FAO: estudio, investigación y productos o servicios **no comerciales**, citando la fuente. No es CC BY (esa licencia cubre solo sus bases estadísticas) | Citar; una adaptación o traducción con otros fines se consulta con la FAO | La tabla es una transcripción traducida, con la cita y esta salvedad junto al archivo |
| SESAN (CIF y Censo de Talla) | No declaran licencia | — | Se citan como fuente oficial; se usan cifras, no se redistribuyen los documentos |
| Portal de datos abiertos del INE | La de cada conjunto (50 con CC BY, 22 con ODC-BY y 33 sin licencia declarada, de 105) | Citar | El manifiesto guarda la licencia de cada descarga |
| Imágenes Landsat (NASA GIBS) | Uso libre | Reconocer a NASA GIBS/ESDIS | El pie de cada cuadro de la animación nombra la foto |
| SESAN (CIF y Censo de Talla) | Publicación oficial | Citar | Citada; la licencia no se pudo verificar |

### 2.14 Referentes internacionales

- **Principios Fundamentales de las Estadísticas Oficiales** (ONU, A/RES/68/261). El principio 5
  admite los registros administrativos como fuente; el 6 pide que los datos individuales sean
  «estrictamente confidenciales» y se usen «exclusivamente para fines estadísticos».
  <https://undocs.org/es/A/RES/68/261>
- **Código Regional de Buenas Prácticas en Estadísticas** (CEPAL, 2023). Pide transferir registros
  administrativos «salvaguardando su confidencialidad y seguridad» (criterio 13.3) y explorar
  fuentes como las páginas web (13.6), que es lo que hacen los muestreos de precios.

## 3. Protección estadística: qué se publica y qué no

**Regla general.** FARO publica agregados: cuántos casos por municipio, departamento, periodo y
categoría. Nunca una fila de una persona.

**El riesgo que queda.** Un conteo muy pequeño puede señalar a alguien: «una defunción por tal
causa en tal municipio en tal mes» puede ser reconocible para sus vecinos.

**Lo que hace FARO** (`umbral_de_celda()` y `api_registros_municipio` en el servidor):

1. **Umbral de frecuencia.** Un conteo de 1 o 2 no se muestra a quien no tiene sesión de analista o
   administrador. En su lugar dice «menos de 3 o protegido». El umbral es `FARO_UMBRAL_CELDA`
   (3 por omisión; 0 lo desactiva).
2. **Supresión complementaria.** Si en un departamento queda oculta una sola celda, se oculta también
   la siguiente más pequeña. Sin eso, la celda oculta se obtendría restando del total del
   departamento, que sí se publica.
3. **Desgloses.** Si el número de hombres o el de mujeres de un departamento está bajo el umbral, se
   oculta el desglose por sexo y se deja el total.
4. **Quién ve los números exactos.** Analistas y administradores con sesión, que son quienes
   trabajan el dato y están sujetos al secreto estadístico.

**Lo que todavía no hace** (y conviene decidir antes de un uso real):

- No compara tablas entre sí: con dos cortes distintos del mismo dato, alguien podría deducir una
  celda por diferencia. Los institutos lo resuelven con una herramienta de control de divulgación
  (por ejemplo τ-ARGUS) o con redondeo controlado.
- El umbral de 3 es un valor habitual, no una norma del INE. Si su política interna pide 5, se
  cambia en la configuración.
- La llave que sustituye al CUI usa SHA-256 con una sal; pasarla a HMAC está en la lista de
  pendientes de seguridad.

**Precios.** Los precios publicados en las páginas de las tiendas no son datos entregados al INE
bajo secreto: son públicos, y por eso se muestran con el nombre de la tienda y el enlace. Las
cotizaciones que una persona cargue para un precio de referencia sí son de informantes: FARO las
procesa sin guardarlas y el informe se imprime para uso interno.

## 4. Lo que debe decidir el INE

1. El umbral de protección de celdas y si se exige también a los analistas.
2. Si las constancias de recepción necesitan firma electrónica avanzada.
3. El aviso de privacidad del trabajo de campo y el plazo de conservación de las visitas.
4. El instrumento con cada institución (convenio, o la vía del Decreto 5-2021 donde aplique).
5. La revisión de este análisis por su asesoría jurídica.

## 5. Lo que no se pudo verificar

- **Ley de Contrataciones, texto consolidado oficial.** Los sitios del Congreso, del MINFIN y de
  Guatecompras no respondieron a consultas automáticas. El artículo 8 se cita por el comparativo de
  ICEFI y por el reglamento del INE de 2023; no se descartan reformas posteriores.
- **Decreto 26-2026.** Solo notas de prensa oficial; no se sabe si reforma los artículos 274 "A" a "G".
- **Iniciativa 6464.** Sin fuente oficial sobre su estado después de julio de 2026.
- **Constitución y Decreto 47-2008.** Se leyeron en copias no oficiales.
- **Código Penal.** Copia con reformas hasta el Decreto 31-2012.
- **Código Regional de la CEPAL.** Se leyó en una presentación de la CEPAL alojada por la ONU; el
  enunciado literal del principio de confidencialidad no se pudo confirmar.
- **GSBPM, X-Road y la licencia de las publicaciones de SESAN.** No verificados.
