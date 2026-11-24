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

Para empezar de cero (borra envíos, equivalencias y usuarios): `docker compose down -v`.

El portal funciona sin internet con los datos que trae. Necesita internet para: sincronizar con el
portal del INE, ejecutar muestreos, bajar fotos de productos, el mapa base de OpenStreetMap y la
tipografía (si no hay, usa la del sistema).

## Sin Docker

Requisitos: Rust estable (1.82 o posterior; probado con 1.97) y PostgreSQL 16.

```bash
cargo test                       # todas las pruebas
cargo build --release
export DATABASE_URL="host=localhost user=faro password=... dbname=faro"
export FARO_TOKEN=... FARO_SAL=... FARO_PUERTO=8082
./target/release/faro-servidor   # desde la raíz del repositorio (lee datos/)
./target/release/faro --help     # línea de comandos
```

## Variables

| Variable | Para qué | Por omisión |
|---|---|---|
| `POSTGRES_PASSWORD` | Clave de la base (Docker) | — |
| `FARO_TOKEN` | Clave de servicio para procesos automáticos | — |
| `FARO_SAL` | Sal para seudonimizar identificadores | — |
| `FARO_PUERTO` | Puerto de la pantalla | 8082 |
| `FARO_ESCUCHA` | Dirección donde se atiende; `0.0.0.0` lo abre a la red (úselo detrás de un proxy con HTTPS) | 127.0.0.1 |
| `FARO_HTTPS` | `1` si el portal está detrás de HTTPS: la galleta de sesión solo viaja cifrada | 0 |
| `FARO_SYNC_HORAS` | Cada cuánto se consulta el portal del INE (0 = nunca) | 24 |
| `FARO_UMBRAL_CELDA` | Conteo mínimo que se publica a quien no es analista (0 = sin protección) | 3 |
| `FARO_FORMULARIOS` | Servicios donde puede estar la encuesta de campo, separados por coma | Microsoft Forms |
| `FARO_MODELOS` | Carpeta del modelo de fotos (sin Docker) | `datos/modelos` |

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
| Noto Sans (Google Fonts, opcional) | Tipografía | OFL |

Las versiones exactas están en `Cargo.lock`. No se usan servicios de pago. Servicios en línea
consultados en ejecución (todos públicos y sin clave): portal de datos abiertos del INE, páginas de
las tiendas muestreadas y teselas de OpenStreetMap si se enciende el mapa base.

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
