# Instalación y dependencias

## Lo más simple: Docker

Requisitos: Docker Desktop (Windows o macOS) o Docker Engine con Compose (Linux); 4 GB de memoria
libres y unos 3 GB de disco. La primera construcción tarda de 5 a 10 minutos.

```bash
git clone https://github.com/tono88/faro-ine-2026.git
cd faro-ine-2026
cp .env.example .env        # cambie las tres claves
docker compose up -d --build
```

Abra <http://localhost:8082> (usuario `admin`, contraseña `admin`). El portal exige cambiarla en «Mi perfil» antes de permitir cualquier cambio: al menos 10 caracteres, con letras y números o signos. En Windows
puede usar `iniciar.bat`, que crea `.env` y busca un puerto libre. Para actualizar a una versión nueva:

```bash
git pull
docker compose up -d --build
```

**Después de entrar:** en «Usuarios» cree una cuenta para cada persona del INE y una cuenta de rol
«Institución» para el enlace de cada institución que envía datos (PNC, RENAP, SE-CONRED…): esa
cuenta solo podrá subir los archivos de su institución. El botón «Recorrido guiado» del encabezado
explica cada pantalla.

Para empezar de cero (borra envíos, equivalencias y usuarios): `docker compose down -v`.

El portal funciona sin internet con los datos que trae. Necesita internet para: sincronizar con el
portal del INE, ejecutar muestreos, bajar fotos de productos, el mapa base de OpenStreetMap y la
tipografía (si no hay, usa la del sistema).

**Usuario y contraseña iniciales: `admin` / `admin`.** El portal obliga a cambiar la contraseña
antes de permitir cambios y antes de mostrar nada interno. Sin sesión no aparecen las secciones de
recepción ni los paneles de carga. La contraseña inicial no está en el código: la da
`FARO_CLAVE_INICIAL` (el despliegue con Docker la deja en `admin`; puede poner otra en `.env` antes
del primer arranque).

## Sin Docker

Requisitos: Rust estable (1.82 o posterior; probado con 1.97) y PostgreSQL 16.

```bash
# Una vez: crear el usuario y la base (como administrador de PostgreSQL)
createuser --pwprompt faro
createdb --owner faro faro

cargo test                       # todas las pruebas
cargo build --release
export DATABASE_URL="host=localhost user=faro password=... dbname=faro"   # obligatoria: no tiene valor por omisión
export FARO_TOKEN=... FARO_SAL=... FARO_PUERTO=8082
export FARO_CLAVE_INICIAL=...    # contraseña inicial de «admin»; si falta, se genera una al azar y se escribe en el registro
./target/release/faro-servidor   # desde la raíz del repositorio (lee datos/)
./target/release/faro --help     # línea de comandos
```

Sin Docker el servidor también atiende solo en esta máquina (`127.0.0.1`). Para abrirlo a la red,
`FARO_ESCUCHA=0.0.0.0`, siempre detrás de un proxy con HTTPS. Los comandos que regeneran tablas y
documentos (`faro verificar`, `faro citas`, `faro mejoras`…) escriben en la carpeta del repositorio:
se corren así, desde un clon, no dentro del contenedor.

## Variables

| Variable | Para qué | Por omisión |
|---|---|---|
| `POSTGRES_PASSWORD` | Clave de la base (Docker) | — |
| `FARO_TOKEN` | Clave de servicio para procesos automáticos | — |
| `FARO_SAL` | Sal para seudonimizar identificadores | — |
| `FARO_CLAVE_INICIAL` | Contraseña inicial de `admin`; solo se usa al crear la base | `admin` con Docker; al azar sin Docker |
| `FARO_PUERTO` | Puerto de la pantalla | 8082 |
| `FARO_ESCUCHA` | Dirección donde se atiende; `0.0.0.0` lo abre a la red (úselo detrás de un proxy con HTTPS) | 127.0.0.1 |
| `FARO_HTTPS` | `1` si el portal está detrás de HTTPS: la galleta de sesión solo viaja cifrada | 0 |
| `FARO_SYNC_HORAS` | Cada cuánto se consulta el portal del INE (0 = nunca) | 24 |
| `FARO_UMBRAL_CELDA` | Conteo mínimo que se publica a quien no es analista (0 = sin protección) | 3 |
| `FARO_FORMULARIOS` | Servicios donde puede estar la encuesta de campo, separados por coma | Microsoft Forms |
| `FARO_MODELOS` | Carpeta del modelo de fotos (sin Docker) | `datos/modelos` |

**Idiomas de la pantalla.** No hay nada que instalar ni configurar: los diccionarios van en
`datos/estandar/idiomas/` (dentro de la imagen de Docker) y el botón del globo los ofrece. Para
corregir una traducción se edita el archivo del idioma y se recarga la página; para sumar un idioma
se agrega un archivo ([`29-idiomas-e-inclusion.md`](29-idiomas-e-inclusion.md)). Con Docker, el
cambio se hace en la carpeta del repositorio y se reconstruye la imagen, o se monta esa carpeta
como volumen.

**Trabajo de campo y HTTPS.** Los teléfonos solo entregan su posición a páginas con HTTPS. Para que
los operadores registren visitas, publique el portal detrás de un proxy con certificado (Caddy,
Nginx o el que use el INE), con `FARO_ESCUCHA=0.0.0.0` y `FARO_HTTPS=1`. En la propia máquina
(`localhost`) funciona sin HTTPS, que es como se probó.

## Dependencias (todas de código abierto)

| Componente | Uso | Licencia |
|---|---|---|
| Rust: axum, tokio, tokio-postgres | Servidor web y base de datos | MIT |
| serde, serde_json, toml, csv | Lectura y escritura de datos | MIT / Apache-2.0 |
| calamine, zip | Lectura de Excel y archivos comprimidos | MIT |
| ureq | Descargas (portal del INE, tiendas, fotos) | MIT / Apache-2.0 |
| image | Lectura de fotos para la huella visual y el clasificador | MIT / Apache-2.0 |
| tract-onnx | Corre el modelo de fotos dentro del servidor (Rust puro, sin tarjeta gráfica) | MIT / Apache-2.0 |
| Places365-ResNet18 (incluido en `datos/modelos/`, 45 MB) | Modelo de escenas: ¿vivienda o comercio? | Pesos CC BY; código MIT |
| regex, strsim, unicode-normalization, chrono, clap, weezl, rand_core, sha2 | Texto, parecido de nombres, fechas, CLI, LZW, azar, huellas | MIT / Apache-2.0 |
| argon2 | Contraseñas | MIT / Apache-2.0 |
| PostgreSQL 16 | Base de datos | PostgreSQL |
| Leaflet 1.9.4 (incluido en el repositorio) | Mapa | BSD-2 |
| Ubuntu Sans (incluida en el portal) | Tipografía de la guía de diseño del INE | Ubuntu Font Licence 1.0 |

Las versiones exactas están en `Cargo.lock` y la lista completa, con la licencia de cada biblioteca,
en [`27-componentes-de-terceros.md`](27-componentes-de-terceros.md). No se usan servicios de pago.
Servicios en línea consultados en ejecución (todos públicos y sin clave):

| Quién lo pide | Servicio | Para qué | ¿Opcional? |
|---|---|---|---|
| El servidor | Portal de datos abiertos del INE | Vigilar si un conjunto cambió | Sí (`FARO_SYNC_HORAS=0` lo apaga) |
| El servidor | Páginas públicas de las tiendas | Muestreos de precios y fotos de producto para el informe | Sí: las tareas vienen apagadas |
| El navegador | Teselas de OpenStreetMap | Mapa base | Sí: una casilla |
| El navegador | NASA GIBS y EOX | Foto satelital de la animación y del mapa | Sí |
| El teléfono del operador | Microsoft Forms del INE | Abrir la encuesta de campo | Sí: solo si el INE la configura |

## Si algo falla

| Síntoma | Causa y arreglo |
|---|---|
| El puerto 8082 está ocupado | Cambie `FARO_PUERTO` en `.env` y vuelva a levantar |
| "Inicie sesión para continuar" | La acción cambia datos: entre con un usuario analista o administrador |
| Los muestreos quedan "sin precios" | El contenedor no tiene salida a internet o la tienda no respondió; vea el detalle en la fila |
| No aparecen fotos en el informe | Sin internet no se pueden bajar; el informe funciona igual con código de barras y nombre |
| «Sin posición» al registrar una visita | La página no usa HTTPS, o el operador no dio permiso de ubicación en su teléfono |
| «El clasificador de fotos no está instalado» | Falta la carpeta `datos/modelos`; la visita se registra igual, sin evaluar la foto |
| Un pendiente dice «reglas anteriores» | Ese envío se analizó antes de un cambio en la solicitud: vuelva a subir el archivo |
| La animación no muestra la foto satelital | El navegador no tiene internet; la animación funciona igual sin foto |
| Tras actualizar no cambian los precios publicados | El levantamiento se recarga solo si la tabla cambió de tamaño; `docker compose down -v` lo fuerza (borra lo demás) |

<!-- equipo -->
---

<p align="center"><img src="identidad/equipo/carta-de-presentacion.svg" alt="Equipo La Sociedad de Almas: Estuardo Sandoval, Ana Echeverría y Alejandro Sandoval, con el código QR del perfil de LinkedIn de cada uno" width="760"></p>

<p align="center"><a href="https://www.linkedin.com/in/estuardoantoniosandoval/">Estuardo Sandoval</a> · <a href="https://www.linkedin.com/in/analuciaecheverria/">Ana Echeverría</a> · <a href="https://www.linkedin.com/in/alejandro-jos%C3%A9-sandoval-acevedo-998781aa/">Alejandro Sandoval</a><br>Equipo La Sociedad de Almas · Datatón INE 2026</p>
