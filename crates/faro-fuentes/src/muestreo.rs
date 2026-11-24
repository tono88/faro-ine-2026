//! Muestreo de precios publicados en tiendas en línea, a través de sus búsquedas públicas.
//!
//! Solo se usan direcciones abiertas, sin sesión ni claves, con pocas consultas y una pausa entre
//! ellas. Cada precio se guarda con el texto del producto tal como lo publica la tienda, su
//! dirección y el momento de la consulta: es un precio publicado, no una cotización.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

/// Una tarea de muestreo: qué tienda, qué términos y cada cuánto.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tarea {
    pub id: String,
    pub titulo: String,
    /// Rubro al que alimenta (construccion, alimentos...).
    pub rubro: String,
    /// Tecnología del catálogo de la tienda. Por ahora: "vtex".
    pub tipo: String,
    pub proveedor: String,
    /// Dirección base de la tienda, sin barra final.
    pub base: String,
    pub terminos: Vec<String>,
    /// Cada cuántas horas se repite cuando está activa.
    #[serde(default = "semanal")]
    pub cada_horas: i32,
    /// Resultados por término (máximo 50).
    #[serde(default = "treinta")]
    pub max_resultados: usize,
    /// Las tareas vienen apagadas: las enciende una persona.
    #[serde(default)]
    pub activa: bool,
}
fn semanal() -> i32 {
    168
}
fn treinta() -> usize {
    30
}

#[derive(Debug, Deserialize)]
struct Archivo {
    #[serde(rename = "tarea", default)]
    tareas: Vec<Tarea>,
}

pub fn cargar_tareas(ruta: &Path) -> Result<Vec<Tarea>, String> {
    let texto = std::fs::read_to_string(ruta).map_err(|e| format!("{}: {e}", ruta.display()))?;
    let a: Archivo = toml::from_str(&texto).map_err(|e| format!("{}: {e}", ruta.display()))?;
    for t in &a.tareas {
        if !t.base.starts_with("https://") || t.base.ends_with('/') {
            return Err(format!("tarea {}: la dirección base debe empezar con https:// y no terminar en /", t.id));
        }
        if t.tipo != "vtex" {
            return Err(format!("tarea {}: tipo {:?} no admitido (por ahora solo vtex)", t.id, t.tipo));
        }
    }
    Ok(a.tareas)
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Oferta {
    pub producto: String,
    pub marca: String,
    pub precio: f64,
    pub unidad: String,
    pub url: String,
    pub termino: String,
}

/// Interpreta la respuesta de la búsqueda pública de un catálogo VTEX. Solo toma productos con
/// precio mayor que cero y existencia.
pub fn ofertas_vtex(respuesta: &Value, termino: &str) -> Vec<Oferta> {
    respuesta
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|p| {
            let articulo = p["items"].as_array()?.first()?;
            let oferta = &articulo["sellers"].as_array()?.first()?["commertialOffer"];
            let precio = oferta["Price"].as_f64().filter(|x| *x > 0.0)?;
            if oferta["AvailableQuantity"].as_i64() == Some(0) {
                return None;
            }
            Some(Oferta {
                producto: p["productName"].as_str()?.split_whitespace().collect::<Vec<_>>().join(" "),
                marca: p["brand"].as_str().unwrap_or("").to_string(),
                precio,
                unidad: articulo["measurementUnit"].as_str().unwrap_or("").to_string(),
                url: p["link"].as_str()?.to_string(),
                termino: termino.to_string(),
            })
        })
        .collect()
}

fn codificar(texto: &str) -> String {
    texto.bytes().map(|b| if b.is_ascii_alphanumeric() { (b as char).to_string() } else { format!("%{b:02X}") }).collect()
}

/// Ejecuta una tarea: una consulta por término, con un segundo de pausa entre consultas.
/// Devuelve las ofertas y, por término, cuántas se hallaron o el error.
pub fn ejecutar(tarea: &Tarea) -> (Vec<Oferta>, Vec<String>) {
    let agente = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(40)).try_proxy_from_env(true).user_agent("faro-muestreo/0.1 (precios publicados; consulta de baja frecuencia)").build();
    let (mut ofertas, mut bitacora) = (Vec::new(), Vec::new());
    let tope = tarea.max_resultados.clamp(1, 50);
    for (k, termino) in tarea.terminos.iter().enumerate() {
        if k > 0 {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        let url = format!("{}/api/catalog_system/pub/products/search?ft={}&_from=0&_to={}", tarea.base, codificar(termino), tope - 1);
        match agente.get(&url).call().map_err(|e| e.to_string()).and_then(|r| r.into_json::<Value>().map_err(|e| e.to_string())) {
            Ok(v) => {
                let halladas = ofertas_vtex(&v, termino);
                bitacora.push(format!("{termino}: {}", halladas.len()));
                ofertas.extend(halladas);
            }
            Err(e) => bitacora.push(format!("{termino}: no respondió ({})", e.chars().take(80).collect::<String>())),
        }
    }
    (ofertas, bitacora)
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn interpreta_la_busqueda_de_un_catalogo_vtex() {
        let r: Value = serde_json::from_str(
            r#"[
              {"productName":"CEMENTO  UGC 42.5KG","brand":"Cementos Progreso","link":"https://tienda.example/cemento-ugc/p",
               "items":[{"measurementUnit":"un","sellers":[{"commertialOffer":{"Price":83.04,"AvailableQuantity":120}}]}]},
              {"productName":"Agotado","brand":"X","link":"https://tienda.example/a/p","items":[{"measurementUnit":"un","sellers":[{"commertialOffer":{"Price":10.0,"AvailableQuantity":0}}]}]},
              {"productName":"Sin precio","brand":"X","link":"https://tienda.example/b/p","items":[{"sellers":[{"commertialOffer":{"Price":0}}]}]},
              {"productName":"Incompleto"}
            ]"#,
        )
        .unwrap();
        let o = ofertas_vtex(&r, "cemento");
        assert_eq!(o, vec![Oferta { producto: "CEMENTO UGC 42.5KG".into(), marca: "Cementos Progreso".into(), precio: 83.04, unidad: "un".into(), url: "https://tienda.example/cemento-ugc/p".into(), termino: "cemento".into() }]);
        assert!(ofertas_vtex(&serde_json::json!({"error": "x"}), "t").is_empty());
        assert_eq!(codificar("piso cerámico"), "piso%20cer%C3%A1mico");
    }

    #[test]
    fn las_tareas_se_validan_al_cargar() {
        let dir = std::env::temp_dir().join(format!("faro-muestreo-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let ruta = dir.join("m.toml");
        std::fs::write(&ruta, "[[tarea]]\nid = \"a\"\ntitulo = \"A\"\nrubro = \"alimentos\"\ntipo = \"vtex\"\nproveedor = \"P\"\nbase = \"https://tienda.example\"\nterminos = [\"arroz\"]\n").unwrap();
        let t = cargar_tareas(&ruta).unwrap();
        assert_eq!((t[0].cada_horas, t[0].max_resultados, t[0].activa), (168, 30, false));
        std::fs::write(&ruta, "[[tarea]]\nid = \"a\"\ntitulo = \"A\"\nrubro = \"x\"\ntipo = \"vtex\"\nproveedor = \"P\"\nbase = \"http://inseguro.example\"\nterminos = []\n").unwrap();
        assert!(cargar_tareas(&ruta).is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
