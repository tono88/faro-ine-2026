//! Precio de referencia a partir de cotizaciones, como lo describe la Dirección de Precios e
//! Índices de Referencia del INE: se calculan media, mediana y moda y se elige según cómo se
//! comporten los precios.
//!
//! - Si hay precios extremos, la **mediana** (no se deja afectar por ellos).
//! - Si los precios son parecidos, la **media**.
//! - La **moda** se informa cuando un precio se repite.
//!
//! Qué es "extremo" y qué es "parecido" son parámetros con valores por defecto declarados aquí;
//! la Dirección puede fijar los suyos. También se revisa la recomendación de que más de la mitad
//! de las fuentes sean cotizaciones directas (fuentes primarias).
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Parametros {
    /// Un precio es extremo si queda a más de este múltiplo del rango intercuartílico fuera de
    /// los cuartiles (regla de Tukey; 1.5 es el valor usual).
    pub factor_extremos: f64,
    /// Los precios son "parecidos" si su coeficiente de variación (%) no pasa de este valor.
    pub cv_parecidos_pct: f64,
    /// Proporción mínima de fuentes primarias que se recomienda superar (%).
    pub minimo_primarias_pct: f64,
    /// Con menos cotizaciones que estas no se recomienda un precio.
    pub minimo_cotizaciones: usize,
    /// Un precio que supera este múltiplo de la mediana (o queda por debajo de la mediana entre
    /// este número) se aparta para revisar: suele ser otro producto u otra presentación.
    pub max_razon: f64,
}

impl Default for Parametros {
    fn default() -> Self {
        Parametros { factor_extremos: 1.5, cv_parecidos_pct: 15.0, minimo_primarias_pct: 50.0, minimo_cotizaciones: 3, max_razon: 3.0 }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Cotizacion {
    pub producto: String,
    pub precio: f64,
    #[serde(default)]
    pub fuente: String,
    /// "primaria" (cotización directa de un proveedor) o "secundaria" (Guatecompras, catálogos...).
    #[serde(default)]
    pub tipo_fuente: String,
    #[serde(default)]
    pub unidad: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PrecioReferencia {
    pub producto: String,
    pub unidad: String,
    pub cotizaciones: usize,
    pub minimo: f64,
    pub maximo: f64,
    pub media: f64,
    pub mediana: f64,
    /// Precio que más se repite, si alguno se repite.
    pub moda: Option<f64>,
    pub coeficiente_variacion_pct: f64,
    /// Precios extremos, con su fuente.
    pub extremos: Vec<(String, f64)>,
    pub fuentes_primarias_pct: f64,
    /// "media", "mediana" o "" si no se recomienda ninguno.
    pub medida: String,
    pub precio_referencia: Option<f64>,
    /// Por qué se eligió esa medida y qué conviene revisar.
    pub notas: Vec<String>,
}

fn cuantil(ordenados: &[f64], q: f64) -> f64 {
    let pos = q * (ordenados.len() - 1) as f64;
    let (i, f) = (pos.floor() as usize, pos.fract());
    if i + 1 < ordenados.len() { ordenados[i] + f * (ordenados[i + 1] - ordenados[i]) } else { ordenados[i] }
}

/// Analiza las cotizaciones de un producto. Ignora precios no positivos (los cuenta en las notas).
pub fn analizar(producto: &str, cotizaciones: &[&Cotizacion], p: &Parametros) -> PrecioReferencia {
    let validas: Vec<&&Cotizacion> = cotizaciones.iter().filter(|c| c.precio.is_finite() && c.precio > 0.0).collect();
    let mut notas = Vec::new();
    if validas.len() < cotizaciones.len() {
        notas.push(format!("{} cotización(es) sin precio válido no se usaron.", cotizaciones.len() - validas.len()));
    }
    let mut unidades: Vec<&str> = validas.iter().map(|c| c.unidad.trim()).filter(|u| !u.is_empty()).collect();
    unidades.sort_unstable_by_key(|u| u.to_lowercase());
    unidades.dedup_by_key(|u| u.to_lowercase());
    if unidades.len() > 1 {
        notas.push(format!("Hay cotizaciones en unidades distintas ({}): no son comparables sin convertirlas.", unidades.join(", ")));
    }
    let mut precios: Vec<f64> = validas.iter().map(|c| c.precio).collect();
    precios.sort_by(f64::total_cmp);
    let n = precios.len();
    let mut r = PrecioReferencia {
        producto: producto.to_string(),
        unidad: unidades.first().map(|u| u.to_string()).unwrap_or_default(),
        cotizaciones: n,
        minimo: precios.first().copied().unwrap_or(0.0),
        maximo: precios.last().copied().unwrap_or(0.0),
        media: 0.0,
        mediana: 0.0,
        moda: None,
        coeficiente_variacion_pct: 0.0,
        extremos: vec![],
        fuentes_primarias_pct: 0.0,
        medida: String::new(),
        precio_referencia: None,
        notas,
    };
    if n == 0 {
        r.notas.push("Sin cotizaciones válidas.".into());
        return r;
    }
    r.media = precios.iter().sum::<f64>() / n as f64;
    r.mediana = cuantil(&precios, 0.5);
    let varianza = if n > 1 { precios.iter().map(|x| (x - r.media).powi(2)).sum::<f64>() / (n - 1) as f64 } else { 0.0 };
    r.coeficiente_variacion_pct = 100.0 * varianza.sqrt() / r.media;
    // Moda: precio (al centavo) más repetido; si hay empate entre varios, no hay una moda.
    let mut veces: BTreeMap<i64, usize> = BTreeMap::new();
    for x in &precios {
        *veces.entry((x * 100.0).round() as i64).or_default() += 1;
    }
    let maximo = veces.values().copied().max().unwrap_or(0);
    let mas_repetidos: Vec<i64> = veces.iter().filter(|(_, v)| **v == maximo).map(|(k, _)| *k).collect();
    if maximo >= 2 && mas_repetidos.len() == 1 {
        r.moda = Some(mas_repetidos[0] as f64 / 100.0);
    }
    let (q1, q3) = (cuantil(&precios, 0.25), cuantil(&precios, 0.75));
    let (bajo, alto) = (q1 - p.factor_extremos * (q3 - q1), q3 + p.factor_extremos * (q3 - q1));
    let fuera_de_razon = |x: f64| r.mediana > 0.0 && (x > r.mediana * p.max_razon || x < r.mediana / p.max_razon);
    r.extremos = validas.iter().filter(|c| (n >= 4 && (c.precio < bajo || c.precio > alto)) || (n >= 3 && fuera_de_razon(c.precio))).map(|c| (c.fuente.clone(), c.precio)).collect();
    let primarias = validas.iter().filter(|c| c.tipo_fuente.trim().eq_ignore_ascii_case("primaria")).count();
    r.fuentes_primarias_pct = 100.0 * primarias as f64 / n as f64;

    if n < p.minimo_cotizaciones {
        r.notas.push(format!("Solo hay {n} cotización(es); se piden al menos {} para recomendar un precio.", p.minimo_cotizaciones));
        return r;
    }
    if !r.extremos.is_empty() {
        r.medida = "mediana".into();
        r.notas.push(format!("Hay {} precio(s) extremo(s): se usa la mediana, que no se deja afectar por ellos.", r.extremos.len()));
    } else if r.coeficiente_variacion_pct <= p.cv_parecidos_pct {
        r.medida = "media".into();
        r.notas.push(format!("Los precios son parecidos (variación de {:.1} %): se usa la media.", r.coeficiente_variacion_pct));
    } else {
        r.medida = "mediana".into();
        r.notas.push(format!("Los precios son dispersos (variación de {:.1} %) aunque ninguno es extremo: se usa la mediana.", r.coeficiente_variacion_pct));
    }
    r.precio_referencia = Some(if r.medida == "media" { r.media } else { r.mediana });
    if r.fuentes_primarias_pct <= p.minimo_primarias_pct {
        r.notas.push(format!("Solo {:.0} % de las fuentes son cotizaciones directas; se recomienda más de {:.0} %.", r.fuentes_primarias_pct, p.minimo_primarias_pct));
    }
    r
}

/// Lee un CSV `producto, precio, fuente, tipo_fuente, unidad` y analiza cada producto.
pub fn analizar_csv<R: std::io::Read>(entrada: R, p: &Parametros) -> Result<Vec<PrecioReferencia>, String> {
    let mut lector = csv::ReaderBuilder::new().trim(csv::Trim::All).from_reader(entrada);
    let cab = lector.headers().map_err(|e| e.to_string())?.iter().map(|c| c.trim_start_matches('\u{feff}').to_lowercase()).collect::<csv::StringRecord>();
    lector.set_headers(cab);
    let mut todas: Vec<Cotizacion> = Vec::new();
    for (n, fila) in lector.deserialize::<Cotizacion>().enumerate() {
        todas.push(fila.map_err(|e| format!("fila {}: {e}", n + 2))?);
    }
    let mut por_producto: BTreeMap<String, Vec<&Cotizacion>> = BTreeMap::new();
    for c in &todas {
        por_producto.entry(c.producto.clone()).or_default().push(c);
    }
    Ok(por_producto.iter().map(|(producto, lista)| analizar(producto, lista, p)).collect())
}

/// Por qué el mismo producto (mismo código de barras) tiene precios muy distintos entre tiendas.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Diferencia {
    /// `oferta`: el precio regular quita la diferencia. `atipico`: una sola tienda se aparta del
    /// resto. `por_confirmar`: ninguna regla la explica y debe decidir una persona.
    pub clase: &'static str,
    /// Tienda señalada cuando la clase es `atipico`.
    pub tienda: Option<String>,
    pub explicacion: String,
    /// Máximo ÷ mínimo con los precios publicados y con los precios regulares (de lista).
    pub razon: f64,
    pub razon_regular: f64,
}

/// Desde esta razón (máximo ÷ mínimo) una diferencia entre tiendas se revisa: 1.5 = 50 % más.
pub const RAZON_A_REVISAR: f64 = 1.5;

/// Explica una diferencia de precio entre tiendas para un mismo código de barras.
/// `precios`: (tienda, precio publicado, precio de lista si la tienda lo publica).
/// Reglas, en orden:
/// 1. Si con los precios regulares (el de lista donde lo hay) la razón baja de 1.5, es una oferta.
/// 2. Si hay tres tiendas o más, una sola queda a más de 3 veces de la mediana (o por debajo de
///    un tercio) y las demás coinciden entre sí, esa tienda tiene un precio atípico.
/// 3. En otro caso no hay regla: se pide confirmación.
pub fn explicar_diferencia(precios: &[(String, f64, Option<f64>)]) -> Option<Diferencia> {
    let razon_de = |v: &[f64]| v.iter().cloned().fold(f64::MIN, f64::max) / v.iter().cloned().fold(f64::MAX, f64::min);
    let publicados: Vec<f64> = precios.iter().map(|p| p.1).collect();
    if precios.len() < 2 || publicados.iter().any(|p| *p <= 0.0) {
        return None;
    }
    let razon = razon_de(&publicados);
    if razon < RAZON_A_REVISAR {
        return None;
    }
    let regulares: Vec<f64> = precios.iter().map(|p| p.2.filter(|l| *l > p.1).unwrap_or(p.1)).collect();
    let razon_regular = razon_de(&regulares);
    let pct = |r: f64| (r - 1.0) * 100.0;
    if razon_regular < RAZON_A_REVISAR {
        let en_oferta: Vec<String> = precios.iter().filter(|p| p.2.is_some_and(|l| l > p.1)).map(|p| format!("{} (regular Q{:.2})", p.0, p.2.unwrap_or(0.0))).collect();
        return Some(Diferencia { clase: "oferta", tienda: None, razon, razon_regular, explicacion: format!("Se explica por oferta en {}: con precios regulares la diferencia baja de {:.0} % a {:.0} %.", en_oferta.join(" y "), pct(razon), pct(razon_regular)) });
    }
    let mut orden = regulares.clone();
    orden.sort_by(|a, b| a.total_cmp(b));
    let mediana = if orden.len() % 2 == 1 { orden[orden.len() / 2] } else { (orden[orden.len() / 2 - 1] + orden[orden.len() / 2]) / 2.0 };
    let lejos = |x: f64| x > 3.0 * mediana || x < mediana / 3.0;
    let apartadas: Vec<usize> = (0..precios.len()).filter(|i| lejos(regulares[*i])).collect();
    if precios.len() >= 3 && apartadas.len() == 1 {
        let resto: Vec<f64> = (0..precios.len()).filter(|i| *i != apartadas[0]).map(|i| regulares[i]).collect();
        if razon_de(&resto) < RAZON_A_REVISAR {
            let p = &precios[apartadas[0]];
            return Some(Diferencia { clase: "atipico", tienda: Some(p.0.clone()), razon, razon_regular, explicacion: format!("{} publica Q{:.2}, más de tres veces distinto de la mediana (Q{:.2}), mientras las otras {} tiendas coinciden entre sí: probable error de publicación. Confirme.", p.0, p.1, mediana, resto.len()) });
        }
    }
    Some(Diferencia { clase: "por_confirmar", tienda: None, razon, razon_regular, explicacion: format!("Ninguna regla la explica: no hay oferta declarada y la diferencia es de {:.0} %{}. Puede ser la política de precios de cada formato de tienda o un error: confirme.", pct(razon), if precios.len() == 2 { " entre solo dos tiendas" } else { "" }) })
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn cot(precio: f64, tipo: &str) -> Cotizacion {
        Cotizacion { producto: "laptop".into(), precio, fuente: format!("proveedor {precio}"), tipo_fuente: tipo.into(), unidad: "unidad".into() }
    }
    fn analiza(lista: &[Cotizacion]) -> PrecioReferencia {
        analizar("laptop", &lista.iter().collect::<Vec<_>>(), &Parametros::default())
    }

    #[test]
    fn precios_parecidos_usan_la_media() {
        let r = analiza(&[cot(4500.0, "primaria"), cot(4800.0, "primaria"), cot(5000.0, "primaria"), cot(4500.0, "secundaria"), cot(5200.0, "secundaria")]);
        assert_eq!((r.medida.as_str(), r.precio_referencia), ("media", Some(4800.0)));
        assert_eq!((r.mediana, r.moda, r.extremos.len()), (4800.0, Some(4500.0), 0));
        assert!((r.fuentes_primarias_pct - 60.0).abs() < 1e-9);
        assert_eq!(r.notas.len(), 1);
    }

    #[test]
    fn un_precio_extremo_lleva_a_la_mediana_y_no_la_mueve() {
        let r = analiza(&[cot(4500.0, "primaria"), cot(4800.0, "primaria"), cot(5000.0, "primaria"), cot(4500.0, "primaria"), cot(20000.0, "primaria")]);
        assert_eq!((r.medida.as_str(), r.precio_referencia), ("mediana", Some(4800.0)));
        assert_eq!(r.extremos, vec![("proveedor 20000".to_string(), 20000.0)]);
        assert!(r.media > 7000.0);
    }

    #[test]
    fn avisa_si_faltan_cotizaciones_directas_o_son_muy_pocas() {
        let r = analiza(&[cot(10.0, "secundaria"), cot(10.5, "secundaria"), cot(11.0, "primaria")]);
        assert!(r.notas.iter().any(|n| n.contains("cotizaciones directas")));
        let pocas = analiza(&[cot(10.0, "primaria"), cot(0.0, "primaria")]);
        assert_eq!((pocas.cotizaciones, pocas.precio_referencia, pocas.medida.as_str()), (1, None, ""));
        assert_eq!(analiza(&[]).cotizaciones, 0);
    }

    #[test]
    fn lee_el_csv_por_producto_y_detecta_unidades_mezcladas() {
        let csv = "Producto,Precio,Fuente,Tipo_fuente,Unidad\nA,10,x,primaria,libra\nA,11,y,primaria,libra\nA,12,z,secundaria,quintal\nB,5,x,primaria,unidad\n";
        let r = analizar_csv(csv.as_bytes(), &Parametros::default()).unwrap();
        assert_eq!(r.len(), 2);
        assert_eq!((r[0].producto.as_str(), r[0].precio_referencia), ("A", Some(11.0)));
        assert!(r[0].notas.iter().any(|n| n.contains("unidades distintas")));
        assert_eq!(r[1].precio_referencia, None);
        assert!(analizar_csv("producto,precio\nA,caro\n".as_bytes(), &Parametros::default()).is_err());
    }

    #[test]
    fn explica_diferencias_entre_tiendas_por_reglas() {
        let p = |t: &str, x: f64, l: Option<f64>| (t.to_string(), x, l);
        // Menos de 50 %: nada que revisar.
        assert!(explicar_diferencia(&[p("A", 10.0, None), p("B", 14.0, None)]).is_none());
        // Oferta: el precio de lista de la barata coincide con la otra.
        let d = explicar_diferencia(&[p("A", 8.0, Some(30.0)), p("B", 40.0, None)]).unwrap();
        assert_eq!((d.clase, d.razon), ("oferta", 5.0));
        // Una tienda atípica entre tres.
        let d = explicar_diferencia(&[p("A", 11.69, Some(13.75)), p("B", 13.6, Some(16.0)), p("C", 215.0, None)]).unwrap();
        assert_eq!((d.clase, d.tienda.as_deref()), ("atipico", Some("C")));
        // Dos tiendas sin oferta: no se puede saber cuál está mal.
        assert_eq!(explicar_diferencia(&[p("A", 0.25, None), p("B", 16.0, None)]).unwrap().clase, "por_confirmar");
        // Tres tiendas donde ninguna se aparta tres veces: tampoco hay regla.
        assert_eq!(explicar_diferencia(&[p("A", 47.0, None), p("B", 45.5, None), p("C", 25.0, None)]).unwrap().clase, "por_confirmar");
    }
}
