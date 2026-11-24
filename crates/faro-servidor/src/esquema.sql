-- Esquema de FARO. Se aplica al arrancar (todas las sentencias son idempotentes).
-- La base guarda equivalencias, bitácora y AGREGADOS. Los registros fila por fila quedan
-- en archivos del volumen privado, ya sin nombres ni identificadores directos.

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

-- Cuentas: usuarios con rol (admin, analista, consulta) y sus sesiones. Las contraseñas se
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
