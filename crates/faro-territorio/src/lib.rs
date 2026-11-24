//! FARO · territorio: cálculos sobre el territorio y sobre precios.
//!
//! - [`crecimiento`], [`construido`], [`geotiff`]: superficie construida vista por satélite y su proyección.
//! - [`poblacion`]: proyecciones del INE por municipio y año.
//! - [`vial`], [`shp`], [`geo`]: red vial como grafo, lectura de Shapefile y geometría.
//! - [`precios`], [`construccion`], [`balance`]: precio de referencia, comparación contra el IPMC y hoja de balance.
//! - [`asignacion`], [`aire`], [`estadistica`]: operadores de campo, calidad del aire e intervalos de confianza.
//!
//! Cuidado de interpretación: se cuenta superficie construida, no viviendas ni personas. Sirve para
//! priorizar la actualización cartográfica, no para sustituir un conteo.
pub mod aire;
pub mod asignacion;
pub mod balance;
pub mod construccion;
pub mod construido;
pub mod crecimiento;
pub mod estadistica;
pub mod geo;
pub mod geotiff;
pub mod poblacion;
pub mod precios;
pub mod shp;
pub mod vial;
pub use crecimiento::{analizar, leer_series, Crecimiento, Punto};
