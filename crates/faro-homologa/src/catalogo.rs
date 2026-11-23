//! Catálogo oficial de lugares (departamento, municipio, lugar poblado) con código INE.
use crate::normalizar::clave;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Nivel {
    #[default]
    Departamento,
    Municipio,
    LugarPoblado,
}

impl std::str::FromStr for Nivel {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s.trim().to_lowercase().as_str() {
            "departamento" => Ok(Nivel::Departamento),
            "municipio" => Ok(Nivel::Municipio),
            "lugar_poblado" => Ok(Nivel::LugarPoblado),
            otro => Err(format!("nivel desconocido: {otro}")),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Lugar {
    pub codigo: String,
    pub nombre: String,
    pub nivel: Nivel,
    /// Código del nivel superior (departamento del municipio, municipio del lugar poblado).
    #[serde(default)]
    pub padre: String,
    /// Otros nombres de uso oficial o común, separados por `|`.
    #[serde(default)]
    pub alias: String,
    /// Centroide (grados decimales, WGS 84). Solo lo traen los lugares poblados.
    #[serde(default)]
    pub longitud: Option<f64>,
    #[serde(default)]
    pub latitud: Option<f64>,
    #[serde(skip)]
    pub clave: String,
    /// Claves de comparación: la del nombre y las de sus alias.
    #[serde(skip)]
    pub claves: Vec<String>,
}

#[derive(Debug, Default)]
pub struct Catalogo {
    lugares: Vec<Lugar>,
    por_codigo: HashMap<String, usize>,
}

impl Catalogo {
    pub fn agregar(&mut self, mut lugar: Lugar) {
        lugar.clave = clave(&lugar.nombre);
        lugar.claves = std::iter::once(lugar.nombre.as_str())
            .chain(lugar.alias.split('|'))
            .map(clave)
            .filter(|c| !c.is_empty())
            .collect();
        self.por_codigo.insert(lugar.codigo.clone(), self.lugares.len());
        self.lugares.push(lugar);
    }

    /// Lee un CSV con columnas `codigo,nombre,nivel,padre` y, opcionales, `alias,longitud,latitud`.
    pub fn cargar_csv(&mut self, ruta: &Path) -> Result<usize, String> {
        let mut lector = csv::Reader::from_path(ruta).map_err(|e| format!("{}: {e}", ruta.display()))?;
        let mut n = 0;
        for fila in lector.deserialize::<Lugar>() {
            self.agregar(fila.map_err(|e| format!("{}: {e}", ruta.display()))?);
            n += 1;
        }
        Ok(n)
    }

    /// Carga todos los `.csv` de una carpeta.
    pub fn cargar_carpeta(ruta: &Path) -> Result<Self, String> {
        let mut catalogo = Catalogo::default();
        let mut archivos: Vec<_> = std::fs::read_dir(ruta)
            .map_err(|e| format!("{}: {e}", ruta.display()))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|x| x == "csv"))
            .collect();
        archivos.sort();
        for a in &archivos {
            catalogo.cargar_csv(a)?;
        }
        if catalogo.lugares.is_empty() {
            return Err(format!("{}: no hay catálogos .csv", ruta.display()));
        }
        Ok(catalogo)
    }

    pub fn por_codigo(&self, codigo: &str) -> Option<&Lugar> {
        self.por_codigo.get(codigo).map(|&i| &self.lugares[i])
    }

    /// Lugares de un nivel; si `padre` viene, solo los que cuelgan de ese código.
    pub fn de_nivel<'a>(&'a self, nivel: Nivel, padre: Option<&'a str>) -> impl Iterator<Item = &'a Lugar> {
        self.lugares
            .iter()
            .filter(move |l| l.nivel == nivel && padre.map_or(true, |p| l.padre == p))
    }

    pub fn len(&self) -> usize {
        self.lugares.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lugares.is_empty()
    }
}
