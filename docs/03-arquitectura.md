# Arquitectura y tecnologías

*FARO · Que cada dato llegue a puerto.* Estado: versión 0.17.

## De un vistazo

```
                        ┌────────────────────────── navegador (laptop, tableta, teléfono) ──────────────────────────┐
                        │  Una sola página (HTML y JavaScript sin marcos) + Leaflet para los mapas                   │
                        └──────────────────────────────────────────┬───────────────────────────────────────────────┘
                                                                   │ HTTP (JSON, CSV, GeoJSON)
┌──────────────────────────────────────────────────────────────────▼───────────────────────────────────────────────┐
│ faro-servidor (Rust, axum)   68 rutas: sesión y permisos · recepción · pendientes · territorio · precios ·          │
│                              tableros · trabajo de campo · capas · bitácora          [la pantalla va dentro]        │
├──────────────────┬──────────────────────────┬──────────────────────────┬────────────────────────────────────────┤
│ faro-homologa    │ faro-territorio          │ faro-fuentes             │ PostgreSQL 16                            │
│ leer · validar · │ población · crecimiento ·│ descargas con constancia │ equivalencias · envíos · pendientes ·    │
│ homologar ·      │ red vial · asignación ·  │ · muestreo de precios ·  │ conteos (hechos) · precios · usuarios ·  │
│ memoria ·        │ precios · balance ·      │ huella de fotos ·        │ grupos · tableros · rutas y visitas ·    │
│ sin datos        │ estadística · lectores   │ clasificador de fachadas │ bitácora                                 │
│ personales       │ de Shapefile y GeoTIFF   │ (modelo local)           │ (solo agregados: ninguna fila personal)  │
└──────────────────┴──────────────────────────┴──────────────────────────┴────────────────────────────────────────┘
        ▲                                                                              
        │ los mismos módulos                                                           
┌───────┴──────────┐      ┌─────────────────────────────────────────────────────────────────────────────────────────┐
│ faro (CLI, Rust) │ ───▶ │ datos/  catálogo oficial · solicitudes · reglas · tablas de referencia con su fuente     │
│ 29 subcomandos   │      │         (cada tabla se regenera con un comando; cada descarga queda en un manifiesto)    │
└──────────────────┘      └─────────────────────────────────────────────────────────────────────────────────────────┘
```

Todo corre en una computadora con Docker: dos contenedores (el portal y la base de datos). No hay
servicios de pago ni piezas en la nube.

## Flujo del dato

```
INE escribe la SOLICITUD ─▶ la institución ENVÍA ─▶ VALIDAR ─▶ HOMOLOGAR ─▶ CONTEOS ─▶ mapa, tableros, capas
  (columnas, catálogos,      (Excel o CSV, con        estructura   código oficial   (por periodo,
   qué es dato personal,      su propia cuenta;       y contenido  + MEMORIA         territorio y
   llaves de pareo, corte)    X-Road después)                         ▲              variables)
                                                    una persona decide lo dudoso, una sola vez
```

El archivo original se procesa en una carpeta temporal y se borra. A la base llegan solo conteos;
la salida fila por fila, ya sin nombres ni identificadores, queda en un volumen privado.

## Principios

1. **El sistema no adivina.** Solo asigna un código cuando la coincidencia es exacta y única, o
   cuando una persona ya confirmó esa equivalencia. El parecido de texto sirve para *proponer*,
   nunca para asignar.
2. **Lo aprendido no se pierde.** Cada decisión queda en la memoria con quién la tomó y cuándo, y
   vale para todos los envíos que vengan.
3. **El contexto manda.** «San José» existe en Escuintla y en Petén: una equivalencia vale solo
   dentro de su contexto (el departamento, o la dimensión del catálogo).
4. **Nada inventado y todo repetible.** Cada tabla de `datos/referencia/` sale de una fuente citada
   y de un comando `faro …`; lo ficticio vive en `datos/ejemplos/` y está rotulado.
5. **Mantenible por el INE.** Solicitudes, reglas y catálogos son archivos de texto (TOML, CSV,
   JSON) que un analista lee y edita sin programar.
6. **Explicable.** Reglas fijas y a la vista: el mismo dato de entrada da siempre el mismo
   resultado. La única pieza entrenada (el clasificador de fotos) corre en el servidor y no decide.

## Componentes

| Componente | Tecnología | Qué contiene |
|---|---|---|
| `faro-homologa` | Rust | Lectura de Excel y CSV (`lectura`), solicitud declarativa (`solicitud`), normalización de texto (`normalizar`), catálogo oficial (`catalogo`), homologación (`homologar`), memoria de equivalencias (`memoria`), recepción y salida sin datos personales (`recepcion`), cuadros del censo (`censo`), productos del IPMC (`productos`) |
| `faro-territorio` | Rust | Población del INE (`poblacion`), crecimiento y proyecciones (`crecimiento`), superficie construida (`construido`, `geotiff`), red vial como grafo (`vial`, `shp`), asignación de operadores (`asignacion`), precios de referencia y diferencias (`precios`), hoja de balance (`balance`), intervalos de confianza (`estadistica`) |
| `faro-fuentes` | Rust | Portal CKAN del INE (`ckan`), descargas con manifiesto (`procedencia`), lectura de catálogos de tiendas (`muestreo`), huella visual de fotos (`foto`), clasificador de fachadas (`fachada`) |
| `faro-servidor` | Rust, axum, tokio | API, sesiones y permisos, esquema de la base (`esquema.sql`), tableros (`tablero`), trabajo de campo (`campo`) y la pantalla (`web/index.html`, incluida en el ejecutable) |
| `faro-cli` | Rust, clap | Ejecutable `faro`: los mismos módulos desde la línea de comandos, para regenerar cada tabla y cada informe |
| Base de datos | PostgreSQL 16 | 20 tablas y dos vistas (`envios_vigentes`, `precios_ultimos`) |
| Pantalla | HTML, CSS y JavaScript sin marcos; Leaflet 1.9.4 | 24 secciones, recorridos guiados, diseñador de tableros, mapa; funciona en teléfono |
| Idiomas de la pantalla | JavaScript sin marcos (`web/idiomas.js`); diccionarios JSON | Español de origen, inglés completo y kaqchikel en borrador; `datos/estandar/idiomas/`; se traduce la interfaz, no los datos |
| Despliegue | Docker Compose | `docker compose up -d --build`; probado desde un clon limpio |
| Pruebas | `cargo test`; Playwright con Chromium | 96 pruebas de reglas y 21 guiones de pantalla (`.faro/pruebas/`) |

Bibliotecas y licencias: [`27-componentes-de-terceros.md`](27-componentes-de-terceros.md).
Instalación y dependencias: [`15-instalacion-y-dependencias.md`](15-instalacion-y-dependencias.md).

## Por qué estas tecnologías

| Decisión | Razón |
|---|---|
| **Rust** para núcleo, servidor y CLI | Un solo ejecutable sin dependencias en la máquina del INE; el compilador impide errores de memoria y de concurrencia; el mismo código sirve al portal y a la línea de comandos |
| **PostgreSQL** | Libre, conocido por el INE, con vistas y funciones de agregación suficientes; no hace falta otro motor |
| **Una página sin marcos** | Nada que compilar ni actualizar del lado del navegador; se sirve dentro del ejecutable y funciona sin internet |
| **Archivos de texto para reglas** | La solicitud, las reglas del IPMC y las tareas de muestreo se pueden revisar, versionar y firmar |
| **Lectores propios de Shapefile y GeoTIFF** | Evitan instalar GDAL; el lector de GeoTIFF resuelve teselas vacías que la biblioteca disponible no leía |
| **Modelo local para las fotos** | La foto de una visita nunca sale del servidor ni se guarda |

## La base de datos

| Grupo | Tablas | Qué guardan |
|---|---|---|
| Recepción | `envios`, `pendientes`, `hechos`, vista `envios_vigentes` | Historial de envíos, lo que espera decisión y los **conteos** por corte, territorio y variables. Ninguna fila de persona |
| Memoria | `equivalencias`, `equivalencias_valor` | Cada decisión con quién la confirmó y cuándo |
| Acceso | `usuarios`, `sesiones`, `grupos`, `bitacora` | Cuentas (huella Argon2), sesiones, secciones por grupo, quién hizo qué |
| Precios | `muestreos`, `precios_observados`, vista `precios_ultimos`, `decisiones_precio`, `huellas_foto` | Lecturas de catálogos públicos, el último precio ya calculado, decisiones sobre diferencias, huella de fotos de producto |
| Territorio | `edificaciones`, `conjuntos_ine`, `sincronizaciones` | Series cargadas por el usuario y vigilancia del portal del INE |
| Tableros | `tableros` | Definición de cada tablero, borrador o publicado |
| Campo | `rutas_campo`, `paradas_campo`, `visitas_campo` | Rutas, paradas y resultado de cada visita (la foto no se guarda) |

## Acceso y protección

- Cinco roles: administrador, analista, consulta, operador de campo e institución. Los **grupos**
  limitan las secciones; una cuenta de **institución** sube solo los archivos de su institución.
- El servidor comprueba el permiso en cada pedido; la pantalla solo oculta lo que no corresponde.
- **Secreto estadístico:** solo se publican conteos; los menores que el umbral (3, configurable) se
  ocultan a quien no es analista, con supresión complementaria.
- Detalle: [`21-seguridad.md`](21-seguridad.md) y [`23-conformidad-legal.md`](23-conformidad-legal.md).

## Idiomas de la pantalla

La pantalla está escrita en español y no se reescribió para traducirla. Un guion aparte
(`web/idiomas.js`) la traduce al vuelo con un diccionario cuya clave es el propio texto en español;
los números y las etiquetas de cada frase se conservan, y lo que la pantalla pinta después se
traduce al aparecer. Decisiones:

- **Los diccionarios son datos** (`datos/estandar/idiomas/<código>.json`), como las solicitudes y las
  tareas de muestreo: sumar un idioma o corregir una frase no pide programar ni reiniciar.
- **El servidor solo los entrega** (`/api/idiomas`, `/api/idiomas/<código>`); el idioma vive en el
  navegador y no toca la base, las capas ni las descargas.
- **Solo se traduce lo que está en el diccionario**, y la lista de textos de la pantalla
  (`plantilla.json`) deja fuera lo que es dato. Dentro de las tablas no se aplican moldes.
- **Sin servicio de traducción**: un diccionario fijo da siempre el mismo resultado y funciona sin
  internet.

Detalle en [`29-idiomas-e-inclusion.md`](29-idiomas-e-inclusion.md).

## Relación con X-Road

X-Road es la carretera; FARO es el formulario de aduana.

- X-Road transporta mensajes entre servidores de seguridad de cada institución, con identidad,
  firma y bitácora. No dice qué columnas lleva el mensaje ni cómo se escriben los lugares.
- FARO define el **contrato de contenido** (la solicitud) y verifica que cada envío lo cumpla.
- Una solicitud de FARO puede publicarse como un servicio REST sobre X-Road: la institución envía
  por su servidor de seguridad y el INE valida y homologa al recibir. Mientras una institución no
  esté conectada, el mismo contrato funciona con un archivo y una cuenta de institución.

Hoy FARO recibe archivos por la pantalla; la conexión por X-Road es una mejora posible, no algo
hecho.
