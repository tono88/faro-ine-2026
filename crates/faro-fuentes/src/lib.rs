//! FARO · fuentes: descubrir y descargar datos abiertos dejando constancia de dónde,
//! cuándo y con qué huella se obtuvo cada archivo (lineamiento: citar fuente y fecha de descarga).
pub mod ckan;
pub mod foto;
pub mod muestreo;
pub mod procedencia;

pub use ckan::{Ckan, Conjunto, Recurso};
pub use procedencia::{Descarga, Manifiesto};

/// Portal de datos abiertos del INE.
pub const INE_CKAN: &str = "https://datos.ine.gob.gt/es";

pub(crate) fn agente() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .try_proxy_from_env(true)
        .timeout(std::time::Duration::from_secs(120))
        .user_agent(concat!("faro/", env!("CARGO_PKG_VERSION"), " (Dataton INE 2026; uso estadistico)"))
        .build()
}
