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
    /// Cómo se lee la tienda:
    /// - `vtex`: búsqueda pública por términos (`terminos`).
    /// - `vtex-catalogo`: recorre el catálogo por categorías (`categorias` filtra las de primer nivel).
    /// - `sitemap`: lee el mapa del sitio (`sitemap`) y los datos estructurados de cada página de producto.
    /// - `magento`: recorre el listado paginado de productos (`listado`).
    pub tipo: String,
    pub proveedor: String,
    /// Dirección base de la tienda, sin barra final.
    pub base: String,
    #[serde(default)]
    pub terminos: Vec<String>,
    /// `vtex-catalogo`: nombres (o parte) de las categorías de primer nivel a recorrer; vacío = todas.
    #[serde(default)]
    pub categorias: Vec<String>,
    /// `sitemap`: dirección del mapa del sitio.
    #[serde(default)]
    pub sitemap: Option<String>,
    /// `sitemap`: texto que distingue las páginas de producto.
    #[serde(default = "patron_producto")]
    pub patron: String,
    /// Tope de consultas por corrida (por omisión 400; nunca más de 3,000).
    #[serde(default = "consultas_por_omision")]
    pub max_consultas: usize,
    /// `magento`: dirección del listado de productos (se le agrega `?p=N`).
    #[serde(default)]
    pub listado: Option<String>,
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
fn consultas_por_omision() -> usize {
    400
}
fn patron_producto() -> String {
    "/producto/".into()
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
        let falta = match t.tipo.as_str() {
            "vtex" if t.terminos.is_empty() => Some("terminos"),
            "sitemap" if !t.sitemap.as_deref().is_some_and(|u| u.starts_with("https://")) => Some("sitemap (https)"),
            "magento" if !t.listado.as_deref().is_some_and(|u| u.starts_with("https://")) => Some("listado (https)"),
            "vtex" | "vtex-catalogo" | "sitemap" | "magento" | "shopify" | "woocommerce" => None,
            otro => return Err(format!("tarea {}: tipo {otro:?} no admitido (vtex, vtex-catalogo, sitemap, magento, shopify o woocommerce)", t.id)),
        };
        if let Some(campo) = falta {
            return Err(format!("tarea {}: falta {campo}", t.id));
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
    /// Término buscado o categoría de la tienda.
    pub termino: String,
    /// Código de barras, si la tienda lo publica.
    pub ean: String,
    /// Precio de lista (antes de oferta), si es distinto del precio.
    pub precio_lista: Option<f64>,
    /// Dirección de la foto del producto, si la tienda la publica.
    pub imagen: String,
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
                ean: articulo["ean"].as_str().map(|e| e.trim()).filter(|e| e.len() >= 8 && e.chars().all(|c| c.is_ascii_digit())).unwrap_or("").to_string(),
                precio_lista: oferta["ListPrice"].as_f64().filter(|l| *l > precio),
                imagen: articulo["images"][0]["imageUrl"].as_str().unwrap_or("").to_string(),
            })
        })
        .collect()
}

fn codificar(texto: &str) -> String {
    texto.bytes().map(|b| if b.is_ascii_alphanumeric() { (b as char).to_string() } else { format!("%{b:02X}") }).collect()
}

/// Producto descrito en los datos estructurados (JSON-LD, schema.org) de una página.
pub fn oferta_de_jsonld(html: &str, url: &str) -> Option<Oferta> {
    static GUION: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let guion = GUION.get_or_init(|| regex::Regex::new(r#"(?is)<script[^>]*application/ld\+json[^>]*>(.*?)</script>"#).expect("patrón válido"));
    fn buscar(v: &Value) -> Option<&Value> {
        match v {
            Value::Array(a) => a.iter().find_map(buscar),
            Value::Object(o) => {
                let es_producto = match &o.get("@type") {
                    Some(Value::String(t)) => t == "Product",
                    Some(Value::Array(t)) => t.iter().any(|x| x == "Product"),
                    _ => false,
                };
                if es_producto { Some(v) } else { o.get("@graph").and_then(buscar) }
            }
            _ => None,
        }
    }
    let numero = |v: &Value| v.as_f64().or_else(|| v.as_str()?.replace(',', "").parse().ok());
    guion.captures_iter(html).find_map(|c| {
        let dato: Value = serde_json::from_str(c.get(1)?.as_str().trim()).ok()?;
        let p = buscar(&dato)?;
        let oferta = match &p["offers"] {
            Value::Array(a) => a.first()?,
            otro => otro,
        };
        let precio = numero(&oferta["price"]).or_else(|| numero(&oferta["lowPrice"])).filter(|x| *x > 0.0)?;
        let texto = |v: &Value| v.as_str().map(|t| t.split_whitespace().collect::<Vec<_>>().join(" ")).unwrap_or_default();
        let marca = match &p["brand"] {
            Value::Object(o) => o.get("name").map(texto).unwrap_or_default(),
            otro => texto(otro),
        };
        let imagen = match &p["image"] {
            Value::String(u) => u.clone(),
            Value::Array(a) => a.first().map(|x| x.as_str().map(String::from).or_else(|| x["url"].as_str().map(String::from)).unwrap_or_default()).unwrap_or_default(),
            Value::Object(o) => o.get("url").and_then(|u| u.as_str()).unwrap_or("").to_string(),
            _ => String::new(),
        };
        let ean = ["gtin13", "gtin", "gtin12", "gtin8"].iter().map(|k| texto(&p[*k])).find(|e| e.len() >= 8 && e.chars().all(|c| c.is_ascii_digit())).unwrap_or_default();
        Some(Oferta { producto: texto(&p["name"]), marca, precio, unidad: String::new(), url: url.to_string(), termino: texto(&p["category"]), ean, precio_lista: None, imagen }).filter(|o| !o.producto.is_empty())
    })
}

/// Direcciones de un mapa del sitio (`<loc>`); dice también si es un índice de otros mapas.
pub fn direcciones_del_mapa(xml: &str) -> (bool, Vec<String>) {
    static LOC: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let loc = LOC.get_or_init(|| regex::Regex::new(r"(?is)<loc>\s*(.*?)\s*</loc>").expect("patrón válido"));
    (xml.contains("<sitemapindex"), loc.captures_iter(xml).map(|c| c[1].replace("&amp;", "&")).collect())
}

/// Productos de una página de listado de una tienda Magento: nombre, dirección y precio final.
pub fn ofertas_de_listado_magento(html: &str) -> Vec<Oferta> {
    static ENLACE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    static PRECIO: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let enlace = ENLACE.get_or_init(|| regex::Regex::new(r#"(?is)class="product-item-link"\s+href="([^"]+)"[^>]*>(.*?)</a>"#).expect("patrón válido"));
    let precio = PRECIO.get_or_init(|| regex::Regex::new(r#"data-price-amount="([0-9.]+)""#).expect("patrón válido"));
    static FOTO: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let foto = FOTO.get_or_init(|| regex::Regex::new(r#"(?is)<img[^>]*class="product-image-photo"[^>]*?\ssrc="([^"]+)""#).expect("patrón válido"));
    // Cada producto es un bloque; dentro, el primer precio es el final.
    html.split("product-item-info").skip(1).filter_map(|bloque| {
        let e = enlace.captures(bloque)?;
        let p: f64 = precio.captures(bloque)?[1].parse().ok().filter(|x| *x > 0.0)?;
        let nombre = e[2].replace("&quot;", "\"").replace("&amp;", "&").replace("&#039;", "'").split_whitespace().collect::<Vec<_>>().join(" ");
        Some(Oferta { producto: nombre, marca: String::new(), precio: p, unidad: String::new(), url: e[1].to_string(), termino: String::new(), ean: String::new(), precio_lista: None, imagen: foto.captures(bloque).map(|c| c[1].to_string()).unwrap_or_default() }).filter(|o| !o.producto.is_empty())
    }).collect()
}

/// Productos de una página del catálogo público de una tienda Shopify (`/products.json`).
pub fn ofertas_shopify(respuesta: &Value, base: &str) -> Vec<Oferta> {
    let numero = |v: &Value| v.as_str().and_then(|t| t.parse::<f64>().ok()).or_else(|| v.as_f64());
    respuesta["products"].as_array().into_iter().flatten().filter_map(|p| {
        let v = p["variants"].as_array()?.first()?;
        let precio = numero(&v["price"]).filter(|x| *x > 0.0)?;
        let codigo = v["barcode"].as_str().unwrap_or("").trim();
        Some(Oferta {
            producto: p["title"].as_str()?.split_whitespace().collect::<Vec<_>>().join(" "),
            marca: p["vendor"].as_str().unwrap_or("").to_string(),
            precio,
            unidad: String::new(),
            url: format!("{base}/products/{}", p["handle"].as_str()?),
            termino: p["product_type"].as_str().unwrap_or("").to_string(),
            ean: if codigo.len() >= 8 && codigo.chars().all(|c| c.is_ascii_digit()) { codigo.to_string() } else { String::new() },
            precio_lista: numero(&v["compare_at_price"]).filter(|l| *l > precio),
            imagen: p["images"][0]["src"].as_str().unwrap_or("").to_string(),
        })
    }).collect()
}

/// Convierte las entidades HTML de un texto (`&amp;`, `&quot;`, `&#8243;`...) en sus caracteres.
pub fn sin_entidades(texto: &str) -> String {
    static NUMERICA: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let numerica = NUMERICA.get_or_init(|| regex::Regex::new(r"&#(\d{1,7});").expect("patrón válido"));
    let t = numerica.replace_all(texto, |c: &regex::Captures| c[1].parse::<u32>().ok().and_then(char::from_u32).map(String::from).unwrap_or_default());
    t.replace("&quot;", "\"").replace("&lt;", "<").replace("&gt;", ">").replace("&nbsp;", " ").replace("&amp;", "&")
}

/// Productos de una página de la API pública de tienda de WooCommerce (`/wp-json/wc/store/v1/products`).
/// Los precios vienen como enteros con los decimales corridos (`currency_minor_unit`).
pub fn ofertas_woocommerce(respuesta: &Value) -> Vec<Oferta> {
    respuesta.as_array().into_iter().flatten().filter_map(|p| {
        let precios = &p["prices"];
        let divisor = 10f64.powi(precios["currency_minor_unit"].as_i64().unwrap_or(2) as i32);
        let numero = |v: &Value| v.as_str().and_then(|t| t.parse::<f64>().ok()).map(|x| x / divisor);
        let precio = numero(&precios["price"]).filter(|x| *x > 0.0)?;
        Some(Oferta {
            producto: sin_entidades(p["name"].as_str()?).split_whitespace().collect::<Vec<_>>().join(" "),
            marca: String::new(),
            precio,
            unidad: String::new(),
            url: p["permalink"].as_str()?.to_string(),
            termino: p["categories"][0]["name"].as_str().unwrap_or("").replace("&amp;", "&"),
            ean: String::new(),
            precio_lista: numero(&precios["regular_price"]).filter(|l| *l > precio),
            imagen: p["images"][0]["src"].as_str().unwrap_or("").to_string(),
        })
    }).collect()
}

/// Desde este valor un "precio" publicado se toma como relleno y se descarta.
const PRECIO_DE_RELLENO: f64 = 999_999.0;

/// Máximo de consultas que hace una tarea en una corrida, sea cual sea su configuración.
const MAX_CONSULTAS: usize = 3000;

/// Ejecuta una tarea con pausas entre consultas. Devuelve las ofertas y una bitácora breve.
pub fn ejecutar(tarea: &Tarea) -> (Vec<Oferta>, Vec<String>) {
    let agente = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(40)).try_proxy_from_env(true).user_agent("faro-muestreo/0.1 (precios publicados; consulta de baja frecuencia)").build();
    let (mut ofertas, mut bitacora): (Vec<Oferta>, Vec<String>) = (Vec::new(), Vec::new());
    let mut consultas = 0usize;
    let mut pedir = |url: &str| -> Result<String, String> {
        if consultas >= tarea.max_consultas.clamp(1, MAX_CONSULTAS) {
            return Err("se alcanzó el máximo de consultas por corrida".into());
        }
        if consultas > 0 {
            std::thread::sleep(std::time::Duration::from_millis(600));
        }
        consultas += 1;
        agente.get(url).call().map_err(|e| e.to_string().chars().take(90).collect::<String>())?.into_string().map_err(|e| e.to_string())
    };
    let json = |texto: String| serde_json::from_str::<Value>(&texto).map_err(|e| e.to_string());
    match tarea.tipo.as_str() {
        "vtex" => {
            let tope = tarea.max_resultados.clamp(1, 50);
            for termino in &tarea.terminos {
                let url = format!("{}/api/catalog_system/pub/products/search?ft={}&_from=0&_to={}", tarea.base, codificar(termino), tope - 1);
                match pedir(&url).and_then(json) {
                    Ok(v) => {
                        let halladas = ofertas_vtex(&v, termino);
                        bitacora.push(format!("{termino}: {}", halladas.len()));
                        ofertas.extend(halladas);
                    }
                    Err(e) => bitacora.push(format!("{termino}: no respondió ({e})")),
                }
            }
        }
        "vtex-catalogo" => match pedir(&format!("{}/api/catalog_system/pub/category/tree/2", tarea.base)).and_then(json) {
            Err(e) => bitacora.push(format!("categorías: no respondió ({e})")),
            Ok(arbol) => {
                // Las tiendas escriben la misma categoría con y sin tildes o mayúsculas.
                let plano = |t: &str| t.to_lowercase().chars().map(|c| match c { 'á' => 'a', 'é' => 'e', 'í' => 'i', 'ó' => 'o', 'ú' | 'ü' => 'u', o => o }).collect::<String>();
                let quiere = |nombre: &str| tarea.categorias.is_empty() || tarea.categorias.iter().any(|c| plano(nombre).contains(&plano(c)));
                // Cada categoría de primer nivel se recorre por sus subcategorías: la tienda no entrega
                // más allá del producto 2,500 de una misma consulta, y así no se pierde nada.
                let categorias: Vec<(String, Vec<String>)> = arbol.as_array().into_iter().flatten().filter_map(|c| {
                    let (id, nombre) = (c["id"].as_i64()?, c["name"].as_str()?.to_string());
                    let hijas: Vec<String> = c["children"].as_array().into_iter().flatten().filter_map(|h| Some(format!("{id}/{}", h["id"].as_i64()?))).collect();
                    Some((nombre, if hijas.is_empty() { vec![id.to_string()] } else { hijas }))
                }).filter(|(n, _)| quiere(n)).collect();
                let por_categoria = (tarea.max_resultados.max(50) / categorias.len().max(1)).max(50);
                'categorias: for (nombre, rutas) in &categorias {
                    let mut de_esta = 0;
                    for ruta in rutas {
                        for desde in (0..2500).step_by(50) {
                            if de_esta >= por_categoria {
                                break;
                            }
                            let url = format!("{}/api/catalog_system/pub/products/search?fq=C:/{ruta}/&_from={desde}&_to={}", tarea.base, desde + 49);
                            match pedir(&url).and_then(json) {
                                Ok(v) => {
                                    let recibidos = v.as_array().map_or(0, Vec::len);
                                    let halladas = ofertas_vtex(&v, nombre);
                                    de_esta += halladas.len();
                                    ofertas.extend(halladas);
                                    if recibidos < 50 {
                                        break;
                                    }
                                }
                                Err(e) if e.contains("máximo de consultas") => {
                                    bitacora.push(format!("{nombre}: {de_esta} (se alcanzó el máximo de consultas)"));
                                    break 'categorias;
                                }
                                // Una subcategoría que falla no detiene las demás.
                                Err(_) => break,
                            }
                        }
                    }
                    bitacora.push(format!("{nombre}: {de_esta}"));
                }
            }
        },
        "sitemap" => {
            let raiz = tarea.sitemap.clone().unwrap_or_default();
            let mut paginas: Vec<String> = Vec::new();
            match pedir(&raiz) {
                Err(e) => bitacora.push(format!("mapa del sitio: no respondió ({e})")),
                Ok(xml) => {
                    let (indice, direcciones) = direcciones_del_mapa(&xml);
                    if indice {
                        for sub in direcciones.iter().take(15) {
                            if let Ok(x) = pedir(sub) {
                                paginas.extend(direcciones_del_mapa(&x).1);
                            }
                        }
                    } else {
                        paginas = direcciones;
                    }
                }
            }
            paginas.retain(|u| u.contains(&tarea.patron) && u.starts_with("https://"));
            let total = paginas.len();
            // Si hay más páginas que el cupo, se toma una de cada tantas para cubrir todo el catálogo.
            let cupo = tarea.max_resultados.max(1);
            let paso = (total / cupo).max(1);
            let mut sin_datos = 0;
            for url in paginas.iter().step_by(paso).take(cupo) {
                match pedir(url) {
                    Ok(html) => match oferta_de_jsonld(&html, url) {
                        Some(o) => ofertas.push(o),
                        None => sin_datos += 1,
                    },
                    Err(_) => sin_datos += 1,
                }
            }
            bitacora.push(format!("{total} páginas de producto en el mapa; {} con precio; {sin_datos} sin datos estructurados o sin respuesta", ofertas.len()));
        }
        "magento" => {
            let listado = tarea.listado.clone().unwrap_or_default();
            let union = if listado.contains('?') { '&' } else { '?' };
            for pagina in 1..=(tarea.max_resultados / 24).max(1) {
                match pedir(&format!("{listado}{union}p={pagina}")) {
                    Ok(html) => {
                        let halladas = ofertas_de_listado_magento(&html);
                        if halladas.is_empty() {
                            break;
                        }
                        ofertas.extend(halladas);
                    }
                    Err(e) => {
                        bitacora.push(format!("página {pagina}: no respondió ({e})"));
                        break;
                    }
                }
                if ofertas.len() >= tarea.max_resultados {
                    break;
                }
            }
            bitacora.push(format!("listado: {} productos", ofertas.len()));
        }
        "shopify" | "woocommerce" => {
            let (por_pagina, shopify) = if tarea.tipo == "shopify" { (250, true) } else { (100, false) };
            for pagina in 1..=(tarea.max_resultados / por_pagina).max(1) {
                let url = if shopify { format!("{}/products.json?limit=250&page={pagina}", tarea.base) } else { format!("{}/wp-json/wc/store/v1/products?per_page=100&page={pagina}", tarea.base) };
                match pedir(&url).and_then(json) {
                    Ok(v) => {
                        let recibidos = if shopify { v["products"].as_array().map_or(0, Vec::len) } else { v.as_array().map_or(0, Vec::len) };
                        ofertas.extend(if shopify { ofertas_shopify(&v, &tarea.base) } else { ofertas_woocommerce(&v) });
                        if recibidos < por_pagina {
                            break;
                        }
                    }
                    Err(e) => {
                        bitacora.push(format!("página {pagina}: no respondió ({e})"));
                        break;
                    }
                }
            }
            bitacora.push(format!("catálogo: {} productos con precio", ofertas.len()));
        }
        otro => bitacora.push(format!("tipo {otro:?} no admitido")),
    }
    // Algunas tiendas publican 9,999,999 como precio de relleno de un producto no disponible: no es un precio.
    let antes = ofertas.len();
    ofertas.retain(|o| o.precio < PRECIO_DE_RELLENO);
    if ofertas.len() < antes {
        bitacora.push(format!("{} con precio de relleno, descartados", antes - ofertas.len()));
    }
    // Un mismo producto puede salir en dos categorías o páginas: se deja una vez.
    let mut vistos = std::collections::HashSet::new();
    ofertas.retain(|o| vistos.insert(o.url.clone()));
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
        assert_eq!(o, vec![Oferta { producto: "CEMENTO UGC 42.5KG".into(), marca: "Cementos Progreso".into(), precio: 83.04, unidad: "un".into(), url: "https://tienda.example/cemento-ugc/p".into(), termino: "cemento".into(), ean: String::new(), precio_lista: None, imagen: String::new() }]);
        let con_codigo: Value = serde_json::from_str(r#"[{"productName":"Arroz 1 kg","brand":"M","link":"https://t.example/a/p","items":[{"ean":"7401234567890","measurementUnit":"un","sellers":[{"commertialOffer":{"Price":9.5,"ListPrice":11.0,"AvailableQuantity":3}}]}]}]"#).unwrap();
        let o = &ofertas_vtex(&con_codigo, "Abarrotes")[0];
        assert_eq!((o.ean.as_str(), o.precio_lista), ("7401234567890", Some(11.0)));
        assert!(ofertas_vtex(&serde_json::json!({"error": "x"}), "t").is_empty());
        assert_eq!(codificar("piso cerámico"), "piso%20cer%C3%A1mico");
    }

    #[test]
    fn lee_datos_estructurados_mapas_del_sitio_y_listados() {
        let html = r#"<html><script type="application/ld+json">{"@context":"https://schema.org","@graph":[{"@type":"BreadcrumbList"},
          {"@type":"Product","name":"Acetaminofén  500 mg x 10 tabletas","brand":{"@type":"Brand","name":"Genérico"},"gtin13":"7401111222333","category":"Analgésicos",
           "offers":{"@type":"Offer","price":"12.50","priceCurrency":"GTQ"}}]}</script></html>"#;
        let o = oferta_de_jsonld(html, "https://farmacia.example/producto/acetaminofen").unwrap();
        assert_eq!((o.producto.as_str(), o.marca.as_str(), o.precio, o.ean.as_str(), o.termino.as_str()), ("Acetaminofén 500 mg x 10 tabletas", "Genérico", 12.5, "7401111222333", "Analgésicos"));
        assert!(oferta_de_jsonld("<html>sin datos</html>", "u").is_none());
        assert!(oferta_de_jsonld(r#"<script type="application/ld+json">{"@type":"Product","name":"Sin precio"}</script>"#, "u").is_none());

        let (indice, urls) = direcciones_del_mapa("<urlset><url><loc> https://f.example/producto/a </loc></url><url><loc>https://f.example/contacto?x=1&amp;y=2</loc></url></urlset>");
        assert_eq!((indice, urls), (false, vec!["https://f.example/producto/a".to_string(), "https://f.example/contacto?x=1&y=2".to_string()]));
        assert!(direcciones_del_mapa("<sitemapindex><sitemap><loc>https://f.example/s1.xml</loc></sitemap></sitemapindex>").0);

        let listado = r#"<li class="item product product-item"><div class="product-item-info"><a class="product-item-link" href="https://t.example/toma-rj11.html"> Toma RJ11
            blanco </a><span data-price-amount="18.95" data-price-type="finalPrice"></span><span data-price-amount="25" data-price-type="oldPrice"></span></div></li>
            <li><div class="product-item-info"><a class="product-item-link" href="https://t.example/sin-precio.html">Sin precio</a></div></li>"#;
        let o = ofertas_de_listado_magento(listado);
        assert_eq!(o.len(), 1);
        assert_eq!((o[0].producto.as_str(), o[0].precio, o[0].url.as_str()), ("Toma RJ11 blanco", 18.95, "https://t.example/toma-rj11.html"));
    }

    #[test]
    fn lee_catalogos_shopify_y_woocommerce() {
        let s: Value = serde_json::from_str(r#"{"products":[{"title":"Audífonos  X","vendor":"JBL","handle":"audifonos-x","product_type":"Audio","variants":[{"price":"2399.00","compare_at_price":"2999.00","barcode":"050036394437"}]},
            {"title":"Sin precio","handle":"sin","variants":[{"price":"0.00"}]}]}"#).unwrap();
        let o = ofertas_shopify(&s, "https://t.example");
        assert_eq!(o.len(), 1);
        assert_eq!((o[0].producto.as_str(), o[0].precio, o[0].precio_lista, o[0].url.as_str(), o[0].ean.as_str()), ("Audífonos X", 2399.0, Some(2999.0), "https://t.example/products/audifonos-x", "050036394437"));
        assert_eq!(sin_entidades("15.6&#8243; FHD &amp; W11 &#8211; US"), "15.6″ FHD & W11 – US");
        let w: Value = serde_json::from_str(r#"[{"name":"Laptop HP 15 &amp; funda","permalink":"https://m.example/producto/laptop","prices":{"price":"405500","regular_price":"450000","currency_minor_unit":2},"categories":[{"name":"Laptops"}]}]"#).unwrap();
        let o = ofertas_woocommerce(&w);
        assert_eq!((o[0].producto.as_str(), o[0].precio, o[0].precio_lista, o[0].termino.as_str()), ("Laptop HP 15 & funda", 4055.0, Some(4500.0), "Laptops"));
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
