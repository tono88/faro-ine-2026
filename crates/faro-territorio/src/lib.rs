//! FARO · territorio: crecimiento urbano medido con edificaciones observadas por satélite,
//! calidad del aire y asignación de operadores de campo ([`asignacion`]).
//!
//! Entrada: una serie anual de edificaciones por territorio (ver `scripts/open_buildings_por_municipio.js`,
//! que la obtiene de Google Open Buildings 2.5D Temporal, 2016–2023).
//! Salida: cuánto creció cada territorio, a qué ritmo y cuánto tendría en un año futuro.
//!
//! Cuidado de interpretación: se cuentan edificaciones, no viviendas ni personas. Sirve para
//! priorizar actualización cartográfica, no para sustituir un conteo.
pub mod aire;
pub mod asignacion;
pub mod crecimiento;
pub use crecimiento::{analizar, leer_series, Crecimiento, Punto};
