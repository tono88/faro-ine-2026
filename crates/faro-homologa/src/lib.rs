//! FARO · núcleo de homologación.
//!
//! Piezas:
//! - [`normalizar`]: lleva un texto libre ("El puerto, San José") a una clave comparable.
//! - [`catalogo`] + [`homologar`]: busca esa clave en el catálogo oficial del INE y en la
//!   memoria de equivalencias ya confirmadas por una persona.
//! - [`solicitud`]: describe qué columnas pide el INE a una institución y valida un envío.
//! - [`lectura`] + [`recepcion`]: leen el archivo (CSV o Excel) y producen la salida estándar.
//! - [`memoria`]: guarda las equivalencias que confirma una persona.
//! - [`censo`]: verifica el catálogo contra el listado de lugares poblados del Censo 2018.
//! - [`productos`]: homologa productos de tiendas contra los artículos del IPMC.
pub mod catalogo;
pub mod censo;
pub mod homologar;
pub mod lectura;
pub mod memoria;
pub mod normalizar;
pub mod productos;
pub mod recepcion;
pub mod solicitud;

pub use catalogo::{Catalogo, Lugar, Nivel};
pub use homologar::{Candidato, Estado, Homologador, Resultado};
pub use memoria::{Equivalencia, EquivalenciaValor, Memoria};
