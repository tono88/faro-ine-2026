//! Solicitud de datos: el INE declara qué columnas necesita de una institución
//! (y cuáles sirven para el pareo), y cada envío se valida contra esa declaración.
use crate::normalizar::basico;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tipo {
    Texto,
    Entero,
    Decimal,
    /// AAAA-MM-DD o DD/MM/AAAA.
    Fecha,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Columna {
    pub nombre: String,
    pub tipo: Tipo,
    #[serde(default)]
    pub requerida: bool,
    /// Otros nombres con los que las instituciones mandan esta columna ("depto", "DEPARTAMENTO").
    #[serde(default)]
    pub alias: Vec<String>,
    /// La columna participa en el pareo con otras fuentes.
    #[serde(default)]
    pub pareo: bool,
    /// Nivel geográfico a homologar contra el catálogo INE: departamento | municipio | lugar_poblado.
    #[serde(default)]
    pub homologar: Option<String>,
    /// Columna que da el contexto de la homologación (ej. el departamento de un municipio).
    #[serde(default)]
    pub contexto: Option<String>,
    /// Valores permitidos (catálogo cerrado), comparados sin tildes ni mayúsculas.
    #[serde(default)]
    pub valores: Vec<String>,
    #[serde(default)]
    pub descripcion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Solicitud {
    pub id: String,
    pub titulo: String,
    pub institucion: String,
    #[serde(default)]
    pub periodicidad: String,
    #[serde(rename = "columna")]
    pub columnas: Vec<Columna>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Problema {
    /// Fila del archivo (1 = primera fila de datos); 0 = problema de estructura.
    pub fila: usize,
    pub columna: String,
    pub detalle: String,
}

#[derive(Debug, Default, Serialize)]
pub struct Reporte {
    pub filas: usize,
    /// Nombre pedido -> nombre con el que llegó.
    pub columnas_reconocidas: HashMap<String, String>,
    pub columnas_sobrantes: Vec<String>,
    pub problemas: Vec<Problema>,
    /// Problemas adicionales no listados (se listan los primeros 200).
    pub problemas_omitidos: usize,
}

impl Reporte {
    pub fn aceptado(&self) -> bool {
        self.problemas.is_empty()
    }
}

const MAX_PROBLEMAS: usize = 200;

impl Solicitud {
    pub fn cargar(ruta: &Path) -> Result<Self, String> {
        let texto = std::fs::read_to_string(ruta).map_err(|e| format!("{}: {e}", ruta.display()))?;
        toml::from_str(&texto).map_err(|e| format!("{}: {e}", ruta.display()))
    }

    /// Para cada columna pedida, índice de la columna del archivo que la contiene.
    pub fn mapear(&self, encabezados: &[String]) -> (HashMap<String, usize>, Vec<String>) {
        let normal: Vec<String> = encabezados.iter().map(|e| basico(e)).collect();
        let mut mapa = HashMap::new();
        let mut usados = vec![false; encabezados.len()];
        for c in &self.columnas {
            let nombres = std::iter::once(&c.nombre).chain(c.alias.iter()).map(|n| basico(n));
            for n in nombres {
                if let Some(i) = normal.iter().position(|e| *e == n) {
                    mapa.insert(c.nombre.clone(), i);
                    usados[i] = true;
                    break;
                }
            }
        }
        let sobrantes = encabezados.iter().zip(&usados).filter(|(_, u)| !**u).map(|(e, _)| e.clone()).collect();
        (mapa, sobrantes)
    }

    /// Valida estructura y contenido de un envío.
    pub fn validar(&self, encabezados: &[String], filas: impl Iterator<Item = Vec<String>>) -> Reporte {
        let (mapa, sobrantes) = self.mapear(encabezados);
        let mut rep = Reporte { columnas_sobrantes: sobrantes, ..Default::default() };
        let anotar = |rep: &mut Reporte, fila: usize, columna: &str, detalle: String| {
            if rep.problemas.len() < MAX_PROBLEMAS {
                rep.problemas.push(Problema { fila, columna: columna.to_string(), detalle });
            } else {
                rep.problemas_omitidos += 1;
            }
        };
        for c in &self.columnas {
            match mapa.get(&c.nombre) {
                Some(&i) => {
                    rep.columnas_reconocidas.insert(c.nombre.clone(), encabezados[i].clone());
                }
                None if c.requerida => anotar(&mut rep, 0, &c.nombre, "falta la columna requerida".into()),
                None => {}
            }
        }
        for (n, fila) in filas.enumerate() {
            rep.filas += 1;
            for c in &self.columnas {
                let Some(&i) = mapa.get(&c.nombre) else { continue };
                let valor = fila.get(i).map(|v| v.trim()).unwrap_or("");
                if valor.is_empty() {
                    if c.requerida {
                        anotar(&mut rep, n + 1, &c.nombre, "valor vacío en columna requerida".into());
                    }
                    continue;
                }
                if !tipo_valido(c.tipo, valor) {
                    anotar(&mut rep, n + 1, &c.nombre, format!("{valor:?} no es {:?}", c.tipo).to_lowercase());
                } else if !c.valores.is_empty() && !c.valores.iter().any(|v| basico(v) == basico(valor)) {
                    anotar(&mut rep, n + 1, &c.nombre, format!("{valor:?} no está en los valores permitidos"));
                }
            }
        }
        rep
    }
}

fn tipo_valido(tipo: Tipo, valor: &str) -> bool {
    match tipo {
        Tipo::Texto => true,
        Tipo::Entero => valor.parse::<i64>().is_ok(),
        Tipo::Decimal => valor.replace(',', "").parse::<f64>().is_ok(),
        Tipo::Fecha => ["%Y-%m-%d", "%d/%m/%Y"].iter().any(|f| chrono::NaiveDate::parse_from_str(valor, f).is_ok()),
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn solicitud() -> Solicitud {
        toml::from_str(
            r#"
            id = "prueba"
            titulo = "Prueba"
            institucion = "X"
            [[columna]]
            nombre = "departamento"
            tipo = "texto"
            requerida = true
            alias = ["depto", "DEPTO."]
            [[columna]]
            nombre = "fecha_hecho"
            tipo = "fecha"
            requerida = true
            [[columna]]
            nombre = "sexo"
            tipo = "texto"
            valores = ["hombre", "mujer", "ignorado"]
            "#,
        )
        .unwrap()
    }

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn reconoce_alias_de_columnas_y_reporta_sobrantes() {
        let rep = solicitud().validar(&s(&["Depto.", "FECHA_HECHO", "obs"]), vec![s(&["Escuintla", "2025-01-31", "x"])].into_iter());
        assert!(rep.aceptado(), "{:?}", rep.problemas);
        assert_eq!(rep.columnas_reconocidas["departamento"], "Depto.");
        assert_eq!(rep.columnas_sobrantes, vec!["obs"]);
    }

    #[test]
    fn detecta_columna_faltante_fecha_mala_vacio_y_valor_fuera_de_catalogo() {
        let rep = solicitud().validar(
            &s(&["departamento", "fecha_hecho", "sexo"]),
            vec![s(&["Escuintla", "31/01/2025", "Mujer"]), s(&["", "2025-13-01", "N/A"])].into_iter(),
        );
        let detalles: Vec<_> = rep.problemas.iter().map(|p| (p.fila, p.columna.as_str())).collect();
        assert_eq!(detalles, vec![(2, "departamento"), (2, "fecha_hecho"), (2, "sexo")]);
        let rep = solicitud().validar(&s(&["departamento"]), std::iter::empty());
        assert_eq!(rep.problemas[0].detalle, "falta la columna requerida");
    }
}
