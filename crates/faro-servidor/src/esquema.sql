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
