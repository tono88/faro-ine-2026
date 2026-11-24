//! FARO · núcleo de homologación.
//!
//! Tres piezas:
//! - [`normalizar`]: lleva un texto libre ("El puerto, San José") a una clave comparable.
//! - [`catalogo`] + [`homologar`]: busca esa clave en el catálogo oficial del INE y en la
//!   memoria de equivalencias ya confirmadas por una persona.
//! - [`solicitud`]: describe qué columnas pide el INE a una institución y valida un envío.
pub mod catalogo;
pub mod censo;
pub mod homologar;
pub mod lectura;
pub mod memoria;
pub mod normalizar;
pub mod recepcion;
pub mod solicitud;

pub use catalogo::{Catalogo, Lugar, Nivel};
pub use homologar::{Candidato, Estado, Homologador, Resultado};
pub use memoria::{Equivalencia, EquivalenciaValor, Memoria};
