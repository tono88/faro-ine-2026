//! Homologación de productos contra un catálogo oficial de artículos (el IPMC del INE): decidir
//! si "Cemento UGC Progreso 4060 PSI saco 42.5 kg" de una tienda es el artículo 12 "Cemento gris,
//! saco 42.5 kg", y llevar su precio a la unidad del catálogo.
//!
//! Criterios (los mismos que en lugares: no adivinar):
//! - **Comparable**: coinciden las palabras del artículo y todas sus medidas (diámetro, peso,
//!   calibre, dimensiones) y se sabe convertir la presentación a la unidad oficial.
//! - **Revisar**: coinciden las palabras pero el texto no dice alguna medida, o no se sabe en qué
//!   presentación se vende. Decide una persona.
//! - **Distinto**: coinciden las palabras pero una medida es otra (varilla de 1/2" cuando el
//!   artículo es de 3/8"). No se compara.
//!
//! Lo que una persona decide queda en la memoria (dimensión `ipmc`) y no se vuelve a preguntar.
use crate::memoria::Memoria;
use crate::normalizar::basico;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::OnceLock;

/// Dimensión de la memoria donde se guardan las decisiones de producto → artículo.
pub const DIMENSION: &str = "ipmc";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Presentacion {
    /// Palabras que identifican la presentación en la unidad publicada o en el texto
    /// (vacío = vale cuando no se reconoce otra).
    #[serde(default)]
    pub contiene: Vec<String>,
    /// Por cuánto se multiplica el precio publicado para llevarlo a la unidad oficial.
    #[serde(default = "uno")]
    pub factor: f64,
    /// Si la unidad oficial es por longitud o peso: clase de medida del texto entre la que se
    /// divide el precio (ej. "ft" para pasar de lámina de 12 pies a pie lineal).
    #[serde(default)]
    pub divide_por: Option<String>,
    /// Supuesto de la conversión, si lo hay (se muestra junto al precio convertido).
    #[serde(default)]
    pub nota: String,
}
fn uno() -> f64 {
    1.0
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Regla {
    pub codigo: u32,
    /// Grupos de palabras: de cada grupo debe aparecer al menos una.
    pub requiere: Vec<Vec<String>>,
    /// Si aparece alguna, no es este artículo.
    #[serde(default)]
    pub excluye: Vec<String>,
    /// Grupos de medidas (ej. `["in:3/8"]`, `["dim:14x19x39", "dim:15x20x40"]`): de cada grupo
    /// debe aparecer una.
    #[serde(default)]
    pub medidas: Vec<Vec<String>>,
    #[serde(default)]
    pub presentaciones: Vec<Presentacion>,
    /// Si el artículo oficial tiene una característica que el texto de una tienda no permite
    /// comprobar ("de primera", "nacional", "tipo económico"): motivo por el que siempre lo
    /// revisa una persona.
    #[serde(default)]
    pub revisar: Option<String>,
    #[serde(default)]
    pub nota: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Reglas {
    #[serde(rename = "articulo")]
    pub articulos: Vec<Regla>,
}

impl Reglas {
    pub fn cargar(ruta: &Path) -> Result<Self, String> {
        let texto = std::fs::read_to_string(ruta).map_err(|e| format!("{}: {e}", ruta.display()))?;
        toml::from_str(&texto).map_err(|e| format!("{}: {e}", ruta.display()))
    }
}

fn patrones() -> &'static [(Regex, &'static str)] {
    static P: OnceLock<Vec<(Regex, &'static str)>> = OnceLock::new();
    P.get_or_init(|| {
        let n = r"(\d+(?:\.\d+)?)";
        [
            (format!(r"{n}\s*x\s*{n}(?:\s*x\s*{n})?"), "dim"),
            (format!(r"{n}\s*(?:kg|kgs|kilos?|kilogramos?)\b"), "kg"),
            (format!(r"{n}\s*(?:lbs?|libras?)\b"), "lb"),
            (format!(r"{n}?\s*(?:qq|quintal(?:es)?)\b"), "qq"),
            (format!(r"{n}\s*psi\b"), "psi"),
            (r"(?:calibre|cal\.?|ga\.?)\s*(\d{1,2})\b".to_string(), "cal"),
            (r"(?:#|no\.?|awg)\s*(\d{1,2})\b".to_string(), "num"),
            (r"(?:chapa|ch\.?)\s*(\d{1,2})\b".to_string(), "chapa"),
            (r"(1/4|1/2|3/4)\s*(?:de\s+)?(?:galon(?:es)?|gal)\b".to_string(), "gal"),
            (r"\b(?:cubetas?|cub)\b()".to_string(), "cubeta"),
            (format!(r"{n}?\s*(?:galon(?:es)?|gal)\b"), "gal"),
            (r#"(\d+\s+\d+/\d+|\d+/\d+|\d+(?:\.\d+)?)\s*(?:"|''|”|pulgadas?|pulg\.?|plg\.?)"#.to_string(), "in"),
            (format!(r"{n}\s*(?:'|´|’|pies?\b|ft\b)"), "ft"),
            (format!(r"{n}\s*(?:mts?|metros?|m)\b"), "m"),
            (r"\b(\d+/\d+)\b".to_string(), "in"), // "varilla 3/8 corrugada": una fracción suelta es un diámetro en pulgadas
        ]
        .into_iter()
        .map(|(p, clase)| (Regex::new(&p).expect("patrón válido"), clase))
        .collect()
    })
}

fn numero(t: &str) -> String {
    match t.parse::<f64>() {
        Ok(v) => format!("{v}"),
        Err(_) => t.split_whitespace().collect::<Vec<_>>().join(" "),
    }
}

/// Medidas que trae un texto, como `clase:valor`: `in:3/8`, `kg:42.5`, `cal:26`, `dim:14x19x39`,
/// `ft:20`, `psi:3000`, `gal:1`. Las dimensiones se ordenan de menor a mayor.
pub fn medidas(texto: &str) -> Vec<String> {
    let mut t: String = crate::normalizar::reparar_codificacion(texto).to_lowercase().replace(',', ".").replace('×', "x");
    // Sin tildes, pero conservando comillas, barras y puntos, que aquí significan algo.
    t = t.chars().map(|c| match c { 'á' => 'a', 'é' => 'e', 'í' => 'i', 'ó' => 'o', 'ú' => 'u', otro => otro }).collect();
    let mut halladas = Vec::new();
    for (patron, clase) in patrones() {
        let mut resto = String::with_capacity(t.len());
        let mut fin = 0;
        for c in patron.captures_iter(&t) {
            let todo = c.get(0).expect("coincidencia");
            let valor = if *clase == "dim" {
                let mut lados: Vec<f64> = (1..=3).filter_map(|k| c.get(k)).filter_map(|m| m.as_str().parse().ok()).collect();
                lados.sort_by(f64::total_cmp);
                lados.iter().map(|v| format!("{v}")).collect::<Vec<_>>().join("x")
            } else {
                match c.get(1).map(|m| m.as_str()).filter(|m| !m.is_empty()) {
                    Some(m) => numero(m),
                    None if *clase == "cubeta" => String::new(),
                    None => "1".into(),
                }
            };
            halladas.push(format!("{clase}:{valor}"));
            resto.push_str(&t[fin..todo.start()]);
            resto.push(' ');
            fin = todo.end();
        }
        resto.push_str(&t[fin..]);
        t = resto; // lo ya interpretado no se vuelve a leer con otro patrón
    }
    halladas.sort();
    halladas.dedup();
    halladas
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Estado {
    Comparable,
    /// Lo confirmó una persona.
    Memoria,
    Revisar,
    Distinto,
    /// Una persona indicó que no corresponde a ningún artículo.
    Descartado,
    SinArticulo,
}

impl Estado {
    pub fn texto(self) -> &'static str {
        match self {
            Estado::Comparable => "comparable",
            Estado::Memoria => "memoria",
            Estado::Revisar => "revisar",
            Estado::Distinto => "distinto",
            Estado::Descartado => "descartado",
            Estado::SinArticulo => "sin_articulo",
        }
    }
    /// ¿El precio entra en la comparación?
    pub fn cuenta(self) -> bool {
        matches!(self, Estado::Comparable | Estado::Memoria)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Resultado {
    pub estado: Estado,
    pub codigo: Option<u32>,
    /// Precio llevado a la unidad oficial del artículo (solo si se pudo convertir).
    pub precio_oficial: Option<f64>,
    pub medidas: Vec<String>,
    pub motivo: String,
}

fn clase(medida: &str) -> &str {
    medida.split(':').next().unwrap_or("")
}

fn tiene_palabra(texto: &str, palabra: &str) -> bool {
    let p = basico(palabra);
    if p.contains(' ') { texto.contains(&p) } else { texto.split(' ').any(|t| t == p) }
}

/// Evalúa un producto contra una regla. `None` si las palabras no corresponden al artículo.
fn evaluar(regla: &Regla, texto: &str, unidad: &str, halladas: &[String], precio: f64, confirmado: bool) -> Option<Resultado> {
    if !confirmado && (!regla.requiere.iter().all(|g| g.iter().any(|p| tiene_palabra(texto, p))) || regla.excluye.iter().any(|p| tiene_palabra(texto, p))) {
        return None;
    }
    let (mut faltan, mut otras) = (Vec::new(), Vec::new());
    for grupo in &regla.medidas {
        if grupo.iter().any(|m| halladas.contains(m)) {
            continue;
        }
        match halladas.iter().find(|h| grupo.iter().any(|m| clase(m) == clase(h))) {
            Some(distinta) => otras.push(format!("{distinta} en vez de {}", grupo.join(" o "))),
            None => faltan.push(grupo.join(" o ")),
        }
    }
    // Presentación: la primera cuyas palabras aparecen en la unidad publicada o en el texto.
    let donde = format!("{} {texto}", basico(unidad));
    let presentacion = regla.presentaciones.iter().find(|p| !p.contiene.is_empty() && p.contiene.iter().any(|c| tiene_palabra(&donde, c))).or_else(|| regla.presentaciones.iter().find(|p| p.contiene.is_empty()));
    let precio_oficial = presentacion.and_then(|p| match &p.divide_por {
        None => Some(precio * p.factor),
        Some(k) => halladas.iter().find(|h| clase(h) == k).and_then(|h| h.split(':').nth(1)?.parse::<f64>().ok()).filter(|v| *v > 0.0).map(|v| precio * p.factor / v),
    });
    let (estado, motivo) = if !otras.is_empty() && !confirmado {
        (Estado::Distinto, format!("medida distinta: {}", otras.join("; ")))
    } else if precio_oficial.is_none() {
        (Estado::Revisar, "no se reconoce la presentación para llevar el precio a la unidad oficial".to_string())
    } else if confirmado {
        (Estado::Memoria, "confirmado por una persona".to_string())
    } else if !faltan.is_empty() {
        (Estado::Revisar, format!("el texto no indica: {}", faltan.join("; ")))
    } else if let Some(motivo) = &regla.revisar {
        (Estado::Revisar, motivo.clone())
    } else {
        let supuesto = presentacion.map(|p| p.nota.as_str()).filter(|n| !n.is_empty()).map(|n| format!(" ({n})")).unwrap_or_default();
        (Estado::Comparable, format!("coinciden palabras, medidas y presentación{supuesto}"))
    };
    Some(Resultado { estado, codigo: Some(regla.codigo), precio_oficial: precio_oficial.filter(|_| estado != Estado::Distinto), medidas: halladas.to_vec(), motivo })
}

/// Homologa un producto publicado (texto, unidad y precio) contra las reglas del catálogo.
pub fn homologar(reglas: &Reglas, memoria: &Memoria, producto: &str, unidad: &str, precio: f64) -> Resultado {
    let texto = basico(producto);
    let halladas = medidas(&format!("{producto} {unidad}"));
    let nada = |estado, motivo: &str| Resultado { estado, codigo: None, precio_oficial: None, medidas: halladas.clone(), motivo: motivo.to_string() };
    // Lo que una persona ya decidió manda.
    if let Some(decidido) = memoria.buscar_valor(DIMENSION, producto) {
        return match decidido.parse::<u32>().ok().and_then(|c| reglas.articulos.iter().find(|r| r.codigo == c)) {
            Some(regla) => evaluar(regla, &texto, unidad, &halladas, precio, true).unwrap_or_else(|| nada(Estado::Revisar, "confirmado, pero sin regla aplicable")),
            None => nada(Estado::Descartado, "una persona indicó que no corresponde a ningún artículo"),
        };
    }
    let orden = |e: Estado| match e {
        Estado::Comparable => 0,
        Estado::Revisar => 1,
        _ => 2,
    };
    reglas
        .articulos
        .iter()
        .filter_map(|r| evaluar(r, &texto, unidad, &halladas, precio, false))
        .min_by_key(|r| orden(r.estado))
        .unwrap_or_else(|| nada(Estado::SinArticulo, "ninguna regla del catálogo corresponde"))
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn reglas() -> Reglas {
        toml::from_str(
            r#"
            [[articulo]]
            codigo = 12
            requiere = [["cemento"], ["gris", "ugc", "uso general"]]
            excluye = ["blanco", "estructural", "libra"]
            medidas = [["kg:42.5"]]
            presentaciones = [{ contiene = [] }]

            [[articulo]]
            codigo = 21
            requiere = [["varilla", "hierro"], ["corrugada", "corrugado"]]
            medidas = [["in:3/8"]]
            presentaciones = [{ contiene = ["quintal", "qq"] }]

            [[articulo]]
            codigo = 61
            requiere = [["block"], ["pomez"]]
            medidas = [["dim:14x19x39", "dim:15x20x40"]]
            presentaciones = [{ contiene = ["ciento"] }, { contiene = [], factor = 100.0 }]

            [[articulo]]
            codigo = 26
            requiere = [["lamina"], ["galvanizada", "galvanizado"]]
            medidas = [["cal:26"]]
            presentaciones = [{ contiene = ["pie lineal"] }, { contiene = [], divide_por = "ft" }]
            "#,
        )
        .unwrap()
    }

    #[test]
    fn lee_medidas_escritas_de_muchas_formas() {
        assert_eq!(medidas("Cemento UGC Progreso — 4060 PSI · Saco 42.5 kg"), vec!["kg:42.5", "psi:4060"]);
        assert_eq!(medidas("Varilla corrugada 3/8\" x 20' grado 40"), vec!["ft:20", "in:3/8"]);
        assert_eq!(medidas("varilla 3/8 corrugada legítima"), vec!["in:3/8"]);
        assert_eq!(medidas("Block de pómez 40 x 20 x 15 cm"), vec!["dim:15x20x40"]);
        assert_eq!(medidas("Lámina galvanizada calibre 26 de 12 pies"), vec!["cal:26", "ft:12"]);
        assert_eq!(medidas("Tubo PVC 1 1/2\" 250 PSI"), vec!["in:1 1/2", "psi:250"]);
        assert_eq!(medidas("Pintura látex blanca galón"), vec!["gal:1"]);
        assert_eq!(medidas("Pintura látex Clásica blanco mate 1/4 de galón"), vec!["gal:1/4"]);
        assert_eq!(medidas("Pintura látex mate clásica blanco cub"), vec!["cubeta:"]);
        assert_eq!(medidas("Tubo industrial cuadrado 1\" x 6 m Chapa 20"), vec!["chapa:20", "in:1", "m:6"]);
        assert_eq!(medidas("CABLE THHN FLEXIMAX No.12 AZUL METRO"), vec!["num:12"]);
        assert!(medidas("Pala punta redonda").is_empty());
    }

    #[test]
    fn comparable_revisar_y_distinto() {
        let (r, m) = (reglas(), Memoria::default());
        let c = homologar(&r, &m, "Cemento UGC Progreso — 4060 PSI · Saco 42.5 kg", "saco", 83.0);
        assert_eq!((c.estado, c.codigo, c.precio_oficial), (Estado::Comparable, Some(12), Some(83.0)));
        let sin_peso = homologar(&r, &m, "Cemento gris uso general", "unidad", 80.0);
        assert_eq!((sin_peso.estado, sin_peso.codigo), (Estado::Revisar, Some(12)));
        assert!(sin_peso.motivo.contains("kg:42.5"));
        let otra = homologar(&r, &m, "Varilla corrugada 1/2\" legítima", "quintal", 400.0);
        assert_eq!((otra.estado, otra.precio_oficial), (Estado::Distinto, None));
        assert_eq!(homologar(&r, &m, "Cemento blanco 42.5 kg", "saco", 140.0).estado, Estado::SinArticulo, "la palabra excluida lo aparta");
        assert_eq!(homologar(&r, &m, "Martillo de uña", "unidad", 50.0).estado, Estado::SinArticulo);
    }

    #[test]
    fn lleva_el_precio_a_la_unidad_oficial() {
        let (r, m) = (reglas(), Memoria::default());
        // Block por unidad -> por ciento; la medida nominal 15x20x40 equivale a la real 14x19x39.
        let b = homologar(&r, &m, "Block pómez 15x20x40 de 25 kg", "unidad", 4.90);
        assert_eq!((b.estado, b.precio_oficial.map(|p| (p * 100.0).round() / 100.0)), (Estado::Comparable, Some(490.0)));
        // Lámina de 12 pies -> por pie lineal.
        let l = homologar(&r, &m, "Lámina galvanizada calibre 26 de 12 pies", "lámina", 120.0);
        assert_eq!((l.estado, l.precio_oficial), (Estado::Comparable, Some(10.0)));
        // Sin largo no se puede convertir: se revisa.
        assert_eq!(homologar(&r, &m, "Lámina galvanizada calibre 26", "lámina", 120.0).estado, Estado::Revisar);
        // Varilla por unidad: no hay conversión declarada a quintal.
        assert_eq!(homologar(&r, &m, "Varilla corrugada 3/8\"", "unidad", 30.0).estado, Estado::Revisar);
    }

    #[test]
    fn lo_que_decide_una_persona_queda_aprendido() {
        let r = reglas();
        let mut m = Memoria::default();
        m.confirmar_valor(DIMENSION, "Cemento gris uso general", "12", "", "analista").unwrap();
        m.confirmar_valor(DIMENSION, "Cemento gris libra", "ninguno", "", "analista").unwrap();
        let c = homologar(&r, &m, "CEMENTO GRIS USO GENERAL", "unidad", 80.0);
        assert_eq!((c.estado, c.codigo, c.precio_oficial), (Estado::Memoria, Some(12), Some(80.0)));
        assert_eq!(homologar(&r, &m, "Cemento gris libra", "libra", 1.6).estado, Estado::Descartado);
    }
}
