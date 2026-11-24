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
    /// Otras formas de escribir un valor permitido: `{ "m" = "hombre", "masculino" = "hombre" }`.
    #[serde(default)]
    pub equivalencias: HashMap<String, String>,
    /// Variable común a la que pertenece el catálogo (sexo, pueblo...). Las equivalencias que una
    /// persona confirma se guardan por dimensión y valen para todas las solicitudes que la usan.
    /// Si no se indica, es el nombre de la columna.
    #[serde(default)]
    pub dimension: Option<String>,
    /// Columnas del archivo que traen el mismo lugar escrito de otra forma (ej. el nombre, cuando
    /// la columna principal trae un código propio de la institución). Se usan solo si la
    /// principal no se pudo resolver.
    #[serde(default)]
    pub respaldo: Vec<String>,
    /// Valores que en origen significan "sin dato" (ej. "98", "99", "999"). Salen vacíos.
    #[serde(default)]
    pub sin_dato: Vec<String>,
    #[serde(default)]
    pub descripcion: String,
    /// Dato personal: "eliminar" (no sale del envío) o "seudonimizar" (se sustituye por una
    /// llave no reversible, útil para parear sin conocer la identidad).
    #[serde(default)]
    pub sensible: Option<String>,
}

impl Columna {
    /// Dimensión del catálogo cerrado de esta columna.
    pub fn dimension(&self) -> &str {
        self.dimension.as_deref().unwrap_or(&self.nombre)
    }

    /// ¿El valor es uno de los que la institución usa para decir "sin dato"?
    pub fn es_sin_dato(&self, valor: &str) -> bool {
        let v = basico(valor);
        !self.sin_dato.is_empty() && self.sin_dato.iter().any(|s| basico(s) == v)
    }

    /// Valor canónico de un catálogo cerrado: "M", "masculino" y "Hombre" -> "hombre".
    /// `None` si la columna no tiene catálogo o el valor no pertenece a él.
    pub fn canonico(&self, valor: &str) -> Option<&str> {
        let v = basico(valor);
        if let Some(c) = self.valores.iter().find(|c| basico(c) == v) {
            return Some(c);
        }
        let destino = self.equivalencias.iter().find(|(k, _)| basico(k) == v).map(|(_, d)| basico(d))?;
        self.valores.iter().find(|c| basico(c) == destino).map(String::as_str)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Solicitud {
    pub id: String,
    pub titulo: String,
    pub institucion: String,
    #[serde(default)]
    pub periodicidad: String,
    /// Hoja de Excel a leer; "todas" une todas las hojas. Por defecto, la primera.
    #[serde(default)]
    pub hoja: Option<String>,
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

    /// Nombres y alias de todas las columnas pedidas (para ubicar el encabezado en el archivo).
    pub fn nombres_esperados(&self) -> Vec<String> {
        self.columnas.iter().flat_map(|c| std::iter::once(c.nombre.clone()).chain(c.alias.iter().cloned())).collect()
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
        self.validar_con(encabezados, filas, &|_, _| false)
    }

    /// Igual que [`Solicitud::validar`]; `aprendido` dice si un valor fuera de catálogo ya tiene
    /// una equivalencia confirmada por una persona (entonces no es un problema).
    pub fn validar_con(&self, encabezados: &[String], filas: impl Iterator<Item = Vec<String>>, aprendido: &dyn Fn(&Columna, &str) -> bool) -> Reporte {
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
                if valor.is_empty() || crate::homologar::es_ignorado(valor) || c.es_sin_dato(valor) {
                    if c.requerida && valor.is_empty() {
                        anotar(&mut rep, n + 1, &c.nombre, "valor vacío en columna requerida".into());
                    }
                    continue;
                }
                if c.homologar.is_some() {
                    continue; // lo geográfico lo juzga la homologación, no el tipo
                }
                if !tipo_valido(c.tipo, valor) {
                    anotar(&mut rep, n + 1, &c.nombre, format!("{valor:?} no es {:?}", c.tipo).to_lowercase());
                } else if !c.valores.is_empty() && c.canonico(valor).is_none() && !aprendido(c, valor) {
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
        Tipo::Fecha => fecha_iso(valor).is_some(),
    }
}

/// Lleva una fecha a AAAA-MM-DD. Acepta AAAA-MM-DD, DD/MM/AAAA y fecha con hora.
pub fn fecha_iso(valor: &str) -> Option<String> {
    let solo_fecha = valor.trim().split([' ', 'T']).next().unwrap_or("");
    ["%Y-%m-%d", "%d/%m/%Y", "%d-%m-%Y"]
        .iter()
        .find_map(|f| chrono::NaiveDate::parse_from_str(solo_fecha, f).ok())
        .map(|d| d.format("%Y-%m-%d").to_string())
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
            equivalencias = { "M" = "hombre", "masculino" = "hombre", "F" = "mujer" }
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
    fn equivalencias_de_valores_llevan_al_canonico() {
        let sol = solicitud();
        let sexo = &sol.columnas[2];
        assert_eq!(sexo.canonico("MASCULINO"), Some("hombre"));
        assert_eq!(sexo.canonico(" f "), Some("mujer"));
        assert_eq!(sexo.canonico("Mujer"), Some("mujer"));
        assert_eq!(sexo.canonico("X"), None);
    }

    #[test]
    fn detecta_columna_faltante_fecha_mala_vacio_y_valor_fuera_de_catalogo() {
        let rep = solicitud().validar(
            &s(&["departamento", "fecha_hecho", "sexo"]),
            vec![s(&["Escuintla", "31/01/2025", "Mujer"]), s(&["", "2025-13-01", "X"])].into_iter(),
        );
        let detalles: Vec<_> = rep.problemas.iter().map(|p| (p.fila, p.columna.as_str())).collect();
        assert_eq!(detalles, vec![(2, "departamento"), (2, "fecha_hecho"), (2, "sexo")]);
        let rep = solicitud().validar(&s(&["departamento"]), std::iter::empty());
        assert_eq!(rep.problemas[0].detalle, "falta la columna requerida");
    }
}
