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

Abra <http://localhost:8082> (usuario `admin`, contraseña `admin`; cámbiela al entrar). En Windows
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
| `FARO_SYNC_HORAS` | Cada cuánto se consulta el portal del INE (0 = nunca) | 24 |

## Dependencias (todas de código abierto)

| Componente | Uso | Licencia |
|---|---|---|
| Rust: axum, tokio, tokio-postgres | Servidor web y base de datos | MIT |
| serde, serde_json, toml, csv | Lectura y escritura de datos | MIT / Apache-2.0 |
| calamine, zip | Lectura de Excel y archivos comprimidos | MIT |
| ureq | Descargas (portal del INE, tiendas, fotos) | MIT / Apache-2.0 |
| image | Lectura de fotos para la huella visual | MIT / Apache-2.0 |
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
| Tras actualizar no cambian los precios publicados | El levantamiento se recarga solo si la tabla cambió de tamaño; `docker compose down -v` lo fuerza (borra lo demás) |
