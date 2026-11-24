use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Punto {
    pub anio: i32,
    pub edificaciones: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Crecimiento {
    pub codigo: String,
    pub anio_inicial: i32,
    pub anio_final: i32,
    pub edificaciones_inicial: f64,
    pub edificaciones_final: f64,
    pub cambio_absoluto: f64,
    /// Tasa de crecimiento anual compuesta, en porcentaje.
    pub tasa_anual_pct: f64,
    /// Edificaciones nuevas por año según la recta de mínimos cuadrados.
    pub tendencia_por_anio: f64,
    /// Qué tan bien explica la recta la serie (0 a 1). Bajo = serie irregular, proyección poco fiable.
    pub r2: f64,
    pub anio_proyectado: i32,
    pub proyeccion: f64,
    /// Margen de la proyección al 95 %: cuantil de la t de Student × error estándar del ajuste, ampliado por la distancia al periodo observado.
    pub proyeccion_margen: f64,
    /// Margen en el que cayó lo observado 8 de cada 10 veces en la prueba hacia atrás.
    pub proyeccion_margen_80: f64,
    /// "alta", "media" o "baja" según r2 y cantidad de años.
    pub confianza: String,
}

#[derive(Deserialize)]
struct Fila {
    codigo: String,
    anio: i32,
    edificaciones: f64,
}

/// Lee un CSV `codigo,anio,edificaciones` y agrupa por código.
pub fn leer_series(ruta: &Path) -> Result<BTreeMap<String, Vec<Punto>>, String> {
    let mut lector = csv::Reader::from_path(ruta).map_err(|e| format!("{}: {e}", ruta.display()))?;
    let mut series: BTreeMap<String, Vec<Punto>> = BTreeMap::new();
    for fila in lector.deserialize::<Fila>() {
        let f = fila.map_err(|e| format!("{}: {e}", ruta.display()))?;
        series.entry(f.codigo).or_default().push(Punto { anio: f.anio, edificaciones: f.edificaciones });
    }
    Ok(series)
}

/// Analiza una serie. Devuelve `None` si hay menos de 3 años o valores no positivos al inicio.
/// Error relativo (respecto de lo proyectado) que se observó al proyectar diez años con la recta de
/// cinco épocas, en 340 municipios y tres décadas (1,020 casos; `faro verificar` lo recalcula y
/// avisa si cambia): 8 de cada 10 veces el error fue de 20 % o menos; 95 de cada 100, de 47 % o menos.
pub const ERROR_OBSERVADO_P80: f64 = 0.20;
pub const ERROR_OBSERVADO_P95: f64 = 0.47;

pub fn analizar(codigo: &str, serie: &[Punto], anio_proyectado: i32) -> Option<Crecimiento> {
    let mut s: Vec<Punto> = serie.iter().copied().filter(|p| p.edificaciones.is_finite()).collect();
    s.sort_by_key(|p| p.anio);
    s.dedup_by_key(|p| p.anio);
    if s.len() < 3 {
        return None;
    }
    let (primero, ultimo) = (s[0], s[s.len() - 1]);
    if primero.edificaciones <= 0.0 || ultimo.anio == primero.anio {
        return None;
    }
    let n = s.len() as f64;
    let mx = s.iter().map(|p| p.anio as f64).sum::<f64>() / n;
    let my = s.iter().map(|p| p.edificaciones).sum::<f64>() / n;
    let sxx: f64 = s.iter().map(|p| (p.anio as f64 - mx).powi(2)).sum();
    let sxy: f64 = s.iter().map(|p| (p.anio as f64 - mx) * (p.edificaciones - my)).sum();
    let syy: f64 = s.iter().map(|p| (p.edificaciones - my).powi(2)).sum();
    let pendiente = sxy / sxx;
    let corte = my - pendiente * mx;
    let residuos: f64 = s.iter().map(|p| (p.edificaciones - (corte + pendiente * p.anio as f64)).powi(2)).sum();
    let r2 = if syy > 0.0 { 1.0 - residuos / syy } else { 1.0 };
    let error = (residuos / (n - 2.0)).sqrt();
    let x = anio_proyectado as f64;
    // Error de predicción de una regresión simple: crece al alejarse del periodo observado.
    // Con pocas épocas el factor correcto no es 2 sino el cuantil de la t de Student (3.18 con cinco puntos).
    let margen = crate::estadistica::t_975(s.len() - 2) * error * (1.0 + 1.0 / n + (x - mx).powi(2) / sxx).sqrt();
    // El error del ajuste subestima la incertidumbre cuando la serie es suave pero no lineal: en la
    // prueba hacia atrás solo contenía lo observado 1 de cada 4 veces. Por eso el margen nunca es
    // menor que el error realmente observado al proyectar diez años (ver `ERROR_OBSERVADO_*`).
    let proyeccion = (corte + pendiente * x).max(0.0);
    let horizonte = ((anio_proyectado - ultimo.anio).max(0) as f64 / 10.0).max(0.1);
    let margen = margen.max(proyeccion * ERROR_OBSERVADO_P95 * horizonte);
    let margen_80 = proyeccion * ERROR_OBSERVADO_P80 * horizonte;
    let anios = (ultimo.anio - primero.anio) as f64;
    let tasa = ((ultimo.edificaciones / primero.edificaciones).powf(1.0 / anios) - 1.0) * 100.0;
    let confianza = if r2 >= 0.9 && s.len() >= 6 { "alta" } else if r2 >= 0.7 { "media" } else { "baja" };
    Some(Crecimiento {
        codigo: codigo.to_string(),
        anio_inicial: primero.anio,
        anio_final: ultimo.anio,
        edificaciones_inicial: primero.edificaciones,
        edificaciones_final: ultimo.edificaciones,
        cambio_absoluto: ultimo.edificaciones - primero.edificaciones,
        tasa_anual_pct: tasa,
        tendencia_por_anio: pendiente,
        r2,
        anio_proyectado,
        proyeccion,
        proyeccion_margen: margen,
        proyeccion_margen_80: margen_80,
        confianza: confianza.to_string(),
    })
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn serie(valores: &[f64]) -> Vec<Punto> {
        valores.iter().enumerate().map(|(i, v)| Punto { anio: 2016 + i as i32, edificaciones: *v }).collect()
    }

    #[test]
    fn crecimiento_lineal_exacto() {
        // +500 por año desde 10 000: la recta es exacta y la proyección a 2030 es 17 000.
        let c = analizar("0501", &serie(&[10000., 10500., 11000., 11500., 12000., 12500., 13000., 13500.]), 2030).unwrap();
        assert!((c.tendencia_por_anio - 500.0).abs() < 1e-6);
        assert!((c.proyeccion - 17000.0).abs() < 1e-3);
        // Aunque la recta sea exacta, el margen no baja del error observado al proyectar (aquí, 7 años).
        assert!(c.r2 > 0.999 && (c.proyeccion_margen - c.proyeccion * ERROR_OBSERVADO_P95 * 0.7).abs() < 1e-6 && c.proyeccion_margen_80 < c.proyeccion_margen);
        assert_eq!(c.confianza, "alta");
        assert!((c.tasa_anual_pct - 4.38).abs() < 0.01, "{}", c.tasa_anual_pct);
    }

    #[test]
    fn serie_irregular_baja_la_confianza_y_abre_el_margen() {
        let c = analizar("x", &serie(&[100., 180., 90., 200., 110., 190., 100., 210.]), 2030).unwrap();
        assert_eq!(c.confianza, "baja");
        assert!(c.proyeccion_margen > 100.0);
    }

    #[test]
    fn series_insuficientes_no_se_proyectan() {
        assert!(analizar("x", &serie(&[100., 110.]), 2030).is_none());
        assert!(analizar("x", &serie(&[0., 110., 120.]), 2030).is_none());
    }

    #[test]
    fn el_orden_de_los_anios_no_importa() {
        let mut s = serie(&[100., 110., 120., 130.]);
        s.reverse();
        let c = analizar("x", &s, 2025).unwrap();
        assert_eq!((c.anio_inicial, c.anio_final), (2016, 2019));
        assert!((c.proyeccion - 190.0).abs() < 1e-6);
    }
}
