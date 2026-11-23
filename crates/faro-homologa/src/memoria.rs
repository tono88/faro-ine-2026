//! Memoria de equivalencias: cada vez que una persona confirma que "El puerto" es el
//! municipio 0509, queda grabado y el siguiente envío ya no pregunta.
//!
//! Es un archivo JSONL de solo agregar: sirve de bitácora auditable (quién, cuándo, de qué
//! institución) y se puede versionar o compartir entre instituciones.
use crate::catalogo::Nivel;
use crate::normalizar::clave;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Equivalencia {
    /// Texto tal como llegó.
    pub original: String,
    pub clave: String,
    pub nivel: Nivel,
    /// Código del nivel superior en el que vale la equivalencia ("" = en cualquier parte).
    #[serde(default)]
    pub contexto: String,
    /// Código INE al que equivale.
    pub codigo: String,
    #[serde(default)]
    pub institucion: String,
    #[serde(default)]
    pub confirmado_por: String,
    pub fecha: String,
}

#[derive(Debug, Default)]
pub struct Memoria {
    ruta: Option<PathBuf>,
    indice: HashMap<(Nivel, String, String), String>,
    total: usize,
}

impl Memoria {
    /// Abre (o prepara) un archivo de memoria. Si no existe, empieza vacía.
    pub fn abrir(ruta: &Path) -> Result<Self, String> {
        let mut m = Memoria { ruta: Some(ruta.to_path_buf()), ..Default::default() };
        if ruta.exists() {
            let texto = std::fs::read_to_string(ruta).map_err(|e| format!("{}: {e}", ruta.display()))?;
            for (n, linea) in texto.lines().enumerate() {
                if linea.trim().is_empty() {
                    continue;
                }
                let e: Equivalencia = serde_json::from_str(linea)
                    .map_err(|e| format!("{} línea {}: {e}", ruta.display(), n + 1))?;
                m.indexar(&e);
            }
        }
        Ok(m)
    }

    fn indexar(&mut self, e: &Equivalencia) {
        // La confirmación más reciente gana: permite corregir una equivalencia equivocada.
        self.indice.insert((e.nivel, e.contexto.clone(), e.clave.clone()), e.codigo.clone());
        self.total += 1;
    }

    /// Busca primero dentro del contexto dado y luego una equivalencia sin contexto.
    pub fn buscar(&self, nivel: Nivel, contexto: Option<&str>, clave: &str) -> Option<&str> {
        if let Some(c) = contexto {
            if let Some(cod) = self.indice.get(&(nivel, c.to_string(), clave.to_string())) {
                return Some(cod);
            }
        }
        self.indice.get(&(nivel, String::new(), clave.to_string())).map(String::as_str)
    }

    /// Graba una equivalencia confirmada por una persona.
    pub fn confirmar(
        &mut self,
        original: &str,
        nivel: Nivel,
        contexto: Option<&str>,
        codigo: &str,
        institucion: &str,
        confirmado_por: &str,
    ) -> Result<Equivalencia, String> {
        let e = Equivalencia {
            original: original.to_string(),
            clave: clave(original),
            nivel,
            contexto: contexto.unwrap_or("").to_string(),
            codigo: codigo.to_string(),
            institucion: institucion.to_string(),
            confirmado_por: confirmado_por.to_string(),
            fecha: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        };
        if let Some(ruta) = &self.ruta {
            if let Some(dir) = ruta.parent() {
                std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            }
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(ruta)
                .map_err(|e| format!("{}: {e}", ruta.display()))?;
            writeln!(f, "{}", serde_json::to_string(&e).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        }
        self.indexar(&e);
        Ok(e)
    }

    pub fn len(&self) -> usize {
        self.total
    }

    pub fn is_empty(&self) -> bool {
        self.total == 0
    }
}
