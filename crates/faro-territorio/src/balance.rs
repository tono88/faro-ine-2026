//! Hoja de Balance de Alimentos: productos derivados a partir de un producto primario.
//!
//! Entrada: una tabla de tasas (`primario, derivado, utilizacion_pct, tasa_extraccion_pct`) y la
//! cantidad del producto primario. De esa cantidad, `utilizacion_pct` se destina a cada derivado y
//! `tasa_extraccion_pct` es cuánto derivado sale de lo destinado:
//! `derivado = cantidad × utilización ÷ 100 × tasa de extracción ÷ 100`.
//! Las tasas las define el INE; el cálculo solo las aplica y avisa de lo que no cuadra.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Tasa {
    pub primario: String,
    pub derivado: String,
    pub utilizacion_pct: f64,
    pub tasa_extraccion_pct: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Derivado {
    pub derivado: String,
    pub utilizacion_pct: f64,
    pub tasa_extraccion_pct: f64,
    /// Cantidad del primario destinada a este derivado.
    pub destinado: f64,
    /// Cantidad de derivado que resulta.
    pub cantidad: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Balance {
    pub primario: String,
    pub cantidad: f64,
    pub derivados: Vec<Derivado>,
    pub utilizacion_total_pct: f64,
    /// Parte del primario que no se destina a ningún derivado de la tabla.
    pub sin_destinar: f64,
    pub avisos: Vec<String>,
}

/// Lee la tabla de tasas (CSV con encabezado). Rechaza porcentajes negativos o no numéricos.
pub fn leer_tasas<R: std::io::Read>(entrada: R) -> Result<Vec<Tasa>, String> {
    let mut tasas = Vec::new();
    for (i, fila) in csv::ReaderBuilder::new().trim(csv::Trim::All).from_reader(entrada).deserialize::<Tasa>().enumerate() {
        let t = fila.map_err(|e| format!("fila {}: {e}", i + 2))?;
        if t.primario.is_empty() || t.derivado.is_empty() {
            return Err(format!("fila {}: falta el producto primario o el derivado", i + 2));
        }
        if !(t.utilizacion_pct >= 0.0 && t.tasa_extraccion_pct >= 0.0) {
            return Err(format!("fila {}: los porcentajes no pueden ser negativos", i + 2));
        }
        tasas.push(t);
    }
    if tasas.is_empty() {
        return Err("la tabla no tiene filas".into());
    }
    Ok(tasas)
}

/// Productos primarios de la tabla, sin repetir y en el orden en que aparecen.
pub fn primarios(tasas: &[Tasa]) -> Vec<String> {
    let mut vistos: Vec<String> = Vec::new();
    for t in tasas {
        if !vistos.contains(&t.primario) {
            vistos.push(t.primario.clone());
        }
    }
    vistos
}

/// Aplica las tasas de un producto primario a una cantidad.
pub fn calcular(tasas: &[Tasa], primario: &str, cantidad: f64) -> Result<Balance, String> {
    if !(cantidad >= 0.0) || !cantidad.is_finite() {
        return Err("la cantidad debe ser un número igual o mayor que cero".into());
    }
    let propias: Vec<&Tasa> = tasas.iter().filter(|t| t.primario.eq_ignore_ascii_case(primario)).collect();
    if propias.is_empty() {
        return Err(format!("la tabla no tiene tasas para {primario:?}"));
    }
    let derivados: Vec<Derivado> = propias.iter().map(|t| {
        let destinado = cantidad * t.utilizacion_pct / 100.0;
        Derivado { derivado: t.derivado.clone(), utilizacion_pct: t.utilizacion_pct, tasa_extraccion_pct: t.tasa_extraccion_pct, destinado, cantidad: destinado * t.tasa_extraccion_pct / 100.0 }
    }).collect();
    let total: f64 = derivados.iter().map(|d| d.utilizacion_pct).sum();
    let mut avisos = Vec::new();
    if total > 100.0 + 1e-9 {
        avisos.push(format!("la utilización suma {total:.1} %: se destina más producto del que hay"));
    } else if total < 100.0 - 1e-9 {
        avisos.push(format!("la utilización suma {total:.1} %: queda {:.1} % sin destinar a derivados de la tabla", 100.0 - total));
    }
    for d in derivados.iter().filter(|d| d.tasa_extraccion_pct > 100.0) {
        avisos.push(format!("{}: tasa de extracción de {:.1} %, mayor que 100 % (revise si el derivado gana peso, por ejemplo por agua añadida)", d.derivado, d.tasa_extraccion_pct));
    }
    let mut nombres: Vec<&str> = derivados.iter().map(|d| d.derivado.as_str()).collect();
    nombres.sort_unstable();
    if nombres.windows(2).any(|w| w[0] == w[1]) {
        avisos.push("hay un derivado repetido en la tabla".into());
    }
    Ok(Balance { primario: propias[0].primario.clone(), cantidad, sin_destinar: (cantidad * (100.0 - total) / 100.0).max(0.0), utilizacion_total_pct: total, derivados, avisos })
}

#[cfg(test)]
mod tests {
    use super::*;

    const TABLA: &str = "primario,derivado,utilizacion_pct,tasa_extraccion_pct\nA,harina,80,75\nA,salvado,15,20\nB,jugo,60,120\nB,pulpa,50,40\n";

    #[test]
    fn aplica_utilizacion_y_tasa_de_extraccion() {
        let tasas = leer_tasas(TABLA.as_bytes()).unwrap();
        assert_eq!(primarios(&tasas), vec!["A", "B"]);
        let b = calcular(&tasas, "a", 1000.0).unwrap();
        assert_eq!(b.derivados.iter().map(|d| (d.derivado.as_str(), d.destinado, d.cantidad)).collect::<Vec<_>>(), vec![("harina", 800.0, 600.0), ("salvado", 150.0, 30.0)]);
        assert_eq!((b.utilizacion_total_pct, b.sin_destinar), (95.0, 50.0));
        assert_eq!(b.avisos.len(), 1);
    }

    #[test]
    fn avisa_de_lo_que_no_cuadra_y_rechaza_entradas_malas() {
        let tasas = leer_tasas(TABLA.as_bytes()).unwrap();
        let b = calcular(&tasas, "B", 10.0).unwrap();
        assert!(b.avisos.iter().any(|a| a.contains("110.0 %")) && b.avisos.iter().any(|a| a.contains("mayor que 100")));
        assert_eq!(b.sin_destinar, 0.0);
        assert!(calcular(&tasas, "C", 1.0).is_err());
        assert!(calcular(&tasas, "A", -1.0).is_err());
        assert!(leer_tasas("primario,derivado,utilizacion_pct,tasa_extraccion_pct\nA,x,-5,10\n".as_bytes()).is_err());
        assert!(leer_tasas("primario,derivado,utilizacion_pct,tasa_extraccion_pct\n".as_bytes()).is_err());
    }
}
