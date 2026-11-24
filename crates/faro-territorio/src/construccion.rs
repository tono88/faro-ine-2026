//! Comparación de precios de materiales de construcción por artículo oficial (IPMC): a partir de
//! productos de tiendas ya homologados, un precio por proveedor y el precio de referencia.
use crate::precios::{analizar, Cotizacion, Parametros, PrecioReferencia};
use serde::Serialize;
use std::collections::BTreeMap;

/// Un producto publicado por una tienda, ya comparado contra el catálogo.
#[derive(Debug, Clone, Serialize)]
pub struct Observacion {
    pub proveedor: String,
    pub producto: String,
    pub url: String,
    pub fecha: String,
    pub precio: f64,
    pub unidad: String,
    /// Artículo oficial al que corresponde, si alguno.
    pub codigo: Option<u32>,
    /// comparable | memoria | revisar | distinto | descartado | sin_articulo
    pub estado: String,
    /// Precio en la unidad oficial del artículo.
    pub precio_oficial: Option<f64>,
    pub motivo: String,
}

impl Observacion {
    pub fn cuenta(&self) -> bool {
        matches!(self.estado.as_str(), "comparable" | "memoria") && self.precio_oficial.is_some()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PrecioProveedor {
    pub proveedor: String,
    /// Mediana de los productos comparables de ese proveedor para el artículo.
    pub precio: f64,
    pub productos: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResumenArticulo {
    pub codigo: u32,
    pub por_proveedor: Vec<PrecioProveedor>,
    pub referencia: PrecioReferencia,
    /// Diferencia entre el proveedor más caro y el más barato, en porcentaje del más barato.
    pub dispersion_pct: Option<f64>,
    pub por_revisar: usize,
    pub distintos: usize,
}

fn mediana(v: &mut [f64]) -> f64 {
    v.sort_by(f64::total_cmp);
    let n = v.len();
    if n % 2 == 1 { v[n / 2] } else { (v[n / 2 - 1] + v[n / 2]) / 2.0 }
}

/// Resume por artículo. Cada proveedor pesa una vez, aunque publique muchas variantes.
pub fn resumir(observaciones: &[Observacion], p: &Parametros) -> Vec<ResumenArticulo> {
    let mut por_articulo: BTreeMap<u32, Vec<&Observacion>> = BTreeMap::new();
    for o in observaciones {
        if let Some(c) = o.codigo {
            por_articulo.entry(c).or_default().push(o);
        }
    }
    por_articulo
        .into_iter()
        .map(|(codigo, lista)| {
            let mut precios: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
            for o in lista.iter().filter(|o| o.cuenta()) {
                precios.entry(&o.proveedor).or_default().push(o.precio_oficial.unwrap_or(0.0));
            }
            let mut por_proveedor: Vec<PrecioProveedor> = precios.into_iter().map(|(proveedor, mut v)| PrecioProveedor { proveedor: proveedor.to_string(), productos: v.len(), precio: mediana(&mut v) }).collect();
            por_proveedor.sort_by(|a, b| a.precio.total_cmp(&b.precio));
            // Catálogos en línea: para la Dirección de Precios son fuentes secundarias.
            let cotizaciones: Vec<Cotizacion> = por_proveedor.iter().map(|x| Cotizacion { producto: codigo.to_string(), precio: x.precio, fuente: x.proveedor.clone(), tipo_fuente: "secundaria".into(), unidad: String::new() }).collect();
            let referencia = analizar(&codigo.to_string(), &cotizaciones.iter().collect::<Vec<_>>(), p);
            let dispersion_pct = match (por_proveedor.first(), por_proveedor.last()) {
                (Some(a), Some(b)) if por_proveedor.len() > 1 && a.precio > 0.0 => Some(100.0 * (b.precio - a.precio) / a.precio),
                _ => None,
            };
            ResumenArticulo { codigo, por_proveedor, referencia, dispersion_pct, por_revisar: lista.iter().filter(|o| o.estado == "revisar").count(), distintos: lista.iter().filter(|o| o.estado == "distinto").count() }
        })
        .collect()
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn o(proveedor: &str, precio: f64, estado: &str) -> Observacion {
        Observacion { proveedor: proveedor.into(), producto: "p".into(), url: String::new(), fecha: String::new(), precio, unidad: String::new(), codigo: Some(12), estado: estado.into(), precio_oficial: (estado == "comparable").then_some(precio), motivo: String::new() }
    }

    #[test]
    fn cada_proveedor_pesa_una_vez_y_solo_cuenta_lo_comparable() {
        let obs = vec![o("A", 80.0, "comparable"), o("A", 84.0, "comparable"), o("A", 300.0, "distinto"), o("B", 83.0, "comparable"), o("C", 86.0, "comparable"), o("C", 70.0, "revisar")];
        let r = resumir(&obs, &Parametros::default());
        assert_eq!(r.len(), 1);
        let a = &r[0];
        assert_eq!(a.por_proveedor.iter().map(|x| (x.proveedor.as_str(), x.precio, x.productos)).collect::<Vec<_>>(), vec![("A", 82.0, 2), ("B", 83.0, 1), ("C", 86.0, 1)]);
        assert_eq!((a.por_revisar, a.distintos, a.referencia.cotizaciones), (1, 1, 3));
        assert!((a.referencia.precio_referencia.unwrap() - 83.666).abs() < 0.01 && a.referencia.medida == "media");
        assert!((a.dispersion_pct.unwrap() - 4.878).abs() < 0.01);
    }
}
