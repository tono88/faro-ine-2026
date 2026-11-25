-- Esquema de FARO. Se aplica al arrancar (todas las sentencias son idempotentes).
-- De los envíos de instituciones la base guarda solo equivalencias, pendientes y AGREGADOS; los
-- registros fila por fila quedan en archivos del volumen privado, ya sin nombres ni identificadores
-- directos. Además guarda cuentas y sesiones, precios publicados, tableros, bitácora y trabajo de campo.

CREATE TABLE IF NOT EXISTS equivalencias (
    id             BIGSERIAL PRIMARY KEY,
    original       TEXT NOT NULL,
    clave          TEXT NOT NULL,
    nivel          TEXT NOT NULL,
    contexto       TEXT NOT NULL DEFAULT '',
    codigo         TEXT NOT NULL,
    institucion    TEXT NOT NULL DEFAULT '',
    confirmado_por TEXT NOT NULL DEFAULT '',
    fecha          TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS equivalencias_clave ON equivalencias (nivel, contexto, clave);

-- Equivalencias de valores de catálogos cerrados (sexo, pueblo...): valen por dimensión.
CREATE TABLE IF NOT EXISTS equivalencias_valor (
    id             BIGSERIAL PRIMARY KEY,
    dimension      TEXT NOT NULL,
    original       TEXT NOT NULL,
    clave          TEXT NOT NULL,
    valor          TEXT NOT NULL,
    institucion    TEXT NOT NULL DEFAULT '',
    confirmado_por TEXT NOT NULL DEFAULT '',
    fecha          TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Cuentas: usuarios con rol (admin, analista, consulta, operador) y sus sesiones. Las contraseñas se
-- guardan como huella Argon2; de cada sesión solo se guarda la huella de su llave.
CREATE TABLE IF NOT EXISTS usuarios (
    id             BIGSERIAL PRIMARY KEY,
    usuario        TEXT NOT NULL UNIQUE,
    nombre         TEXT NOT NULL,
    rol            TEXT NOT NULL,
    clave          TEXT NOT NULL,
    activo         BOOLEAN NOT NULL DEFAULT TRUE,
    debe_cambiar   BOOLEAN NOT NULL DEFAULT FALSE,
    creado         TIMESTAMPTZ NOT NULL DEFAULT now(),
    ultimo_ingreso TIMESTAMPTZ
);
CREATE TABLE IF NOT EXISTS sesiones (
    huella     TEXT PRIMARY KEY,
    usuario_id BIGINT NOT NULL REFERENCES usuarios (id) ON DELETE CASCADE,
    creada     TIMESTAMPTZ NOT NULL DEFAULT now(),
    expira     TIMESTAMPTZ NOT NULL
);

-- Muestreo automático de precios publicados: estado de cada tarea y precios observados.
CREATE TABLE IF NOT EXISTS muestreos (
    id         TEXT PRIMARY KEY,
    activa     BOOLEAN NOT NULL,
    cada_horas INT NOT NULL,
    ultima     TIMESTAMPTZ,
    ultimo_ok  BOOLEAN,
    detalle    TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS precios_observados (
    id        BIGSERIAL PRIMARY KEY,
    tarea     TEXT NOT NULL,
    rubro     TEXT NOT NULL,
    proveedor TEXT NOT NULL,
    producto  TEXT NOT NULL,
    marca     TEXT NOT NULL DEFAULT '',
    precio    DOUBLE PRECISION NOT NULL,
    unidad    TEXT NOT NULL DEFAULT '',
    url       TEXT NOT NULL,
    termino   TEXT NOT NULL DEFAULT '',
    observado TIMESTAMPTZ NOT NULL DEFAULT now()
);
-- Código de barras y precio de lista: los publica el catálogo completo de la tienda.
ALTER TABLE precios_observados ADD COLUMN IF NOT EXISTS ean TEXT NOT NULL DEFAULT '';
ALTER TABLE precios_observados ADD COLUMN IF NOT EXISTS precio_lista DOUBLE PRECISION;
ALTER TABLE precios_observados ADD COLUMN IF NOT EXISTS imagen TEXT NOT NULL DEFAULT '';
-- Huella visual de la foto de un producto: forma (64 bits) y color (cuadrícula 4x4, 48 valores).
CREATE TABLE IF NOT EXISTS huellas_foto (
    imagen    TEXT PRIMARY KEY,
    forma     BIGINT,
    color     REAL[],
    calculada TIMESTAMPTZ NOT NULL DEFAULT now()
);
-- Decisión de una persona sobre el precio que una tienda publica para un código de barras:
-- 'correcto' (se confirma) o 'error' (se excluye de comparaciones, búsquedas e informes).
CREATE TABLE IF NOT EXISTS decisiones_precio (
    rubro     TEXT NOT NULL,
    ean       TEXT NOT NULL,
    proveedor TEXT NOT NULL,
    decision  TEXT NOT NULL,
    usuario   TEXT NOT NULL,
    fecha     TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (rubro, ean, proveedor)
);
CREATE INDEX IF NOT EXISTS precios_observados_url ON precios_observados (proveedor, url, observado DESC);
CREATE INDEX IF NOT EXISTS precios_observados_rubro ON precios_observados (rubro, proveedor, producto, observado DESC);

CREATE TABLE IF NOT EXISTS envios (
    id            BIGSERIAL PRIMARY KEY,
    solicitud     TEXT NOT NULL,
    institucion   TEXT NOT NULL,
    archivo       TEXT NOT NULL,
    recibido      TIMESTAMPTZ NOT NULL DEFAULT now(),
    filas         BIGINT NOT NULL,
    pct_resuelto  DOUBLE PRECISION NOT NULL,
    problemas     BIGINT NOT NULL,
    reporte       JSONB NOT NULL
);

CREATE TABLE IF NOT EXISTS pendientes (
    id          BIGSERIAL PRIMARY KEY,
    envio_id    BIGINT NOT NULL REFERENCES envios (id) ON DELETE CASCADE,
    institucion TEXT NOT NULL,
    columna     TEXT NOT NULL,
    nivel       TEXT NOT NULL,
    contexto    TEXT NOT NULL DEFAULT '',
    valor       TEXT NOT NULL,
    veces       BIGINT NOT NULL,
    estado      TEXT NOT NULL,
    candidatos  JSONB NOT NULL,
    resuelto    BOOLEAN NOT NULL DEFAULT FALSE
);

-- Conteos por periodo, municipio y sexo: lo que se cruza con censos y encuestas.
CREATE TABLE IF NOT EXISTS hechos (
    envio_id  BIGINT NOT NULL REFERENCES envios (id) ON DELETE CASCADE,
    solicitud TEXT NOT NULL,
    periodo   TEXT NOT NULL,
    municipio TEXT NOT NULL,
    sexo      TEXT NOT NULL,
    n         BIGINT NOT NULL
);
CREATE INDEX IF NOT EXISTS hechos_solicitud ON hechos (solicitud, periodo);

CREATE TABLE IF NOT EXISTS conjuntos_ine (
    nombre      TEXT PRIMARY KEY,
    titulo      TEXT NOT NULL,
    licencia    TEXT NOT NULL,
    modificado  TEXT NOT NULL,
    recursos    INT NOT NULL,
    formatos    TEXT NOT NULL,
    primera_vez TIMESTAMPTZ NOT NULL DEFAULT now(),
    cambio      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS sincronizaciones (
    id       BIGSERIAL PRIMARY KEY,
    inicio   TIMESTAMPTZ NOT NULL DEFAULT now(),
    ok       BOOLEAN NOT NULL,
    conjuntos INT NOT NULL DEFAULT 0,
    nuevos   INT NOT NULL DEFAULT 0,
    cambiados INT NOT NULL DEFAULT 0,
    detalle  TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS edificaciones (
    municipio     TEXT NOT NULL,
    anio          INT NOT NULL,
    edificaciones DOUBLE PRECISION NOT NULL,
    fuente        TEXT NOT NULL DEFAULT 'Google Open Buildings 2.5D Temporal',
    PRIMARY KEY (municipio, anio)
);

-- Grupos de usuarios: qué secciones del portal puede usar cada grupo.
CREATE TABLE IF NOT EXISTS grupos (
    id          BIGSERIAL PRIMARY KEY,
    nombre      TEXT NOT NULL UNIQUE,
    descripcion TEXT NOT NULL DEFAULT '',
    secciones   TEXT[] NOT NULL DEFAULT '{}'
);
ALTER TABLE usuarios ADD COLUMN IF NOT EXISTS grupo_id BIGINT REFERENCES grupos(id) ON DELETE SET NULL;
INSERT INTO grupos (nombre, descripcion, secciones) VALUES
  ('Registros administrativos', 'Recepción y homologación de envíos de instituciones', '{mapa,proceso,recibir,pendientes,memoria,resultados,tableros,ine,capas,acerca,perfil}'),
  ('Territorio y cartografía', 'Población, crecimiento, red vial y operadores de campo', '{mapa,proceso,crecimiento,vial,operadores,alimentaria,tableros,capas,acerca,perfil}'),
  ('Precios e índices', 'Precios de referencia, canasta, rubros, balance y muestreos', '{mapa,proceso,construccion,precios,canasta,rubros,balance,muestreos,tableros,acerca,perfil}')
ON CONFLICT (nombre) DO NOTHING;

-- Tableros armados por los usuarios (a partir de una frase o editándolos).
CREATE TABLE IF NOT EXISTS tableros (
    id         BIGSERIAL PRIMARY KEY,
    titulo     TEXT NOT NULL,
    solicitud  TEXT NOT NULL DEFAULT '',
    definicion JSONB NOT NULL,
    autor      TEXT NOT NULL,
    publico    BOOLEAN NOT NULL DEFAULT TRUE,
    creado     TIMESTAMPTZ NOT NULL DEFAULT now(),
    cambiado   TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Un tablero nace como borrador (lo ven su autor y los administradores) y se publica aparte.
ALTER TABLE tableros ALTER COLUMN publico SET DEFAULT FALSE;

-- Corte de cada conteo: el mes de registro (o, si la solicitud no lo declara, el mes del hecho).
ALTER TABLE hechos ADD COLUMN IF NOT EXISTS corte TEXT NOT NULL DEFAULT '';
UPDATE hechos SET corte = periodo WHERE corte = '' AND periodo <> '';
CREATE INDEX IF NOT EXISTS hechos_corte ON hechos (solicitud, corte, envio_id);
-- Envíos vigentes: el más reciente de cada solicitud y corte (AAAA-MM). Un envío cuyos cortes
-- llegaron todos de nuevo en envíos posteriores queda reemplazado: se conserva en el historial,
-- pero ya no cuenta en las estadísticas ni muestra pendientes. Un envío sin filas en `hechos`
-- es vigente solo si es el último de su solicitud.
CREATE OR REPLACE VIEW envios_vigentes AS
SELECT e.id FROM envios e
WHERE e.id IN (SELECT max(envio_id) FROM hechos GROUP BY solicitud, corte)
   OR (NOT EXISTS (SELECT 1 FROM hechos h WHERE h.envio_id = e.id) AND e.id = (SELECT max(x.id) FROM envios x WHERE x.solicitud = e.solicitud));

-- Huella de las reglas (la solicitud) con que se analizó cada envío. Si la solicitud cambia después,
-- el envío queda marcado como «analizado con reglas anteriores»: hay que volver a subirlo.
ALTER TABLE envios ADD COLUMN IF NOT EXISTS reglas TEXT;

-- Bitácora de acciones: quién hizo qué y cuándo. Se anotan: recibir o borrar un envío, confirmar
-- una equivalencia de lugar, crear o cambiar una cuenta, publicar o retirar un tablero, crear o
-- borrar una ruta de campo y registrar una visita. No guarda datos de los archivos recibidos:
-- solo el nombre de quien actúa, la acción y a qué se aplicó.
CREATE TABLE IF NOT EXISTS bitacora (
    id      BIGSERIAL PRIMARY KEY,
    cuando  TIMESTAMPTZ NOT NULL DEFAULT now(),
    quien   TEXT NOT NULL,
    accion  TEXT NOT NULL,
    detalle TEXT NOT NULL DEFAULT ''
);
CREATE INDEX IF NOT EXISTS bitacora_cuando ON bitacora (cuando DESC);

-- Trabajo de campo. Una ruta es de un operador y lleva paradas (lugares poblados del censo) en orden.
CREATE TABLE IF NOT EXISTS rutas_campo (
    id          BIGSERIAL PRIMARY KEY,
    nombre      TEXT NOT NULL,
    operador_id BIGINT NOT NULL REFERENCES usuarios(id) ON DELETE CASCADE,
    -- Dirección del formulario de la encuesta (Microsoft Forms u otro admitido), con marcadores {lugar}, {latitud}...
    formulario  TEXT NOT NULL DEFAULT '',
    largo_km    REAL NOT NULL DEFAULT 0,
    creada_por  TEXT NOT NULL,
    creada      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS paradas_campo (
    id       BIGSERIAL PRIMARY KEY,
    ruta_id  BIGINT NOT NULL REFERENCES rutas_campo(id) ON DELETE CASCADE,
    orden    INT NOT NULL,
    lugar    TEXT NOT NULL,
    nombre   TEXT NOT NULL,
    longitud DOUBLE PRECISION NOT NULL,
    latitud  DOUBLE PRECISION NOT NULL
);
CREATE INDEX IF NOT EXISTS paradas_campo_ruta ON paradas_campo (ruta_id, orden);
-- Cada visita guarda la posición que entregó el teléfono y el resultado del cruce. La foto NO se
-- guarda: se clasifica en memoria y se descarta (puede traer personas, placas o direcciones).
CREATE TABLE IF NOT EXISTS visitas_campo (
    id          BIGSERIAL PRIMARY KEY,
    parada_id   BIGINT NOT NULL REFERENCES paradas_campo(id) ON DELETE CASCADE,
    operador    TEXT NOT NULL,
    cuando      TIMESTAMPTZ NOT NULL DEFAULT now(),
    longitud    DOUBLE PRECISION,
    latitud     DOUBLE PRECISION,
    precision_m REAL,
    distancia_m REAL,
    foto_clase  TEXT,
    foto_confianza REAL,
    coherencia  TEXT NOT NULL,
    cruce       JSONB NOT NULL,
    notas       TEXT NOT NULL DEFAULT ''
);
CREATE INDEX IF NOT EXISTS visitas_campo_parada ON visitas_campo (parada_id);
INSERT INTO grupos (nombre, descripcion, secciones) VALUES
  ('Operadores de campo', 'Encuestadores: ven su ruta, registran cada visita con su posición y abren la encuesta', '{mapa,campo,acerca,perfil}')
ON CONFLICT (nombre) DO NOTHING;

-- Recorrido de demostración: paradas reales, visitas ficticias. Se rotula en la pantalla y en la
-- tabla de visitas, y es el único en que se admite una posición simulada.
ALTER TABLE rutas_campo ADD COLUMN IF NOT EXISTS demo BOOLEAN NOT NULL DEFAULT FALSE;

-- Cuentas de institución: quien envía por una institución (PNC, RENAP…) entra con su usuario, ve lo
-- público y sube solo los archivos de las solicitudes de su institución.
ALTER TABLE usuarios ADD COLUMN IF NOT EXISTS institucion TEXT NOT NULL DEFAULT '';

-- El precio más reciente de cada producto de cada tienda, ya calculado. Antes cada consulta de «Precios
-- por rubro» lo volvía a sacar de todas las observaciones, varias veces; ahora se lee de aquí y el
-- servidor lo refresca cuando cambian las observaciones (`asegurar_ultimos`).
CREATE MATERIALIZED VIEW IF NOT EXISTS precios_ultimos AS
    SELECT DISTINCT ON (rubro, proveedor, url) * FROM precios_observados ORDER BY rubro, proveedor, url, observado DESC;
CREATE INDEX IF NOT EXISTS precios_ultimos_rubro ON precios_ultimos (rubro, precio);
CREATE INDEX IF NOT EXISTS precios_ultimos_ean ON precios_ultimos (rubro, ean) WHERE ean <> '';
