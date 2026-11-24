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
    /// Margen de la proyección al 95 %. Es el mayor de dos valores: (a) cuantil de la t de Student ×
    /// error estándar del ajuste, ampliado por la distancia al periodo observado; (b) el error observado
    /// en la prueba hacia atrás: proyección × `ERROR_OBSERVADO_P95` × años proyectados ÷ 10.
    pub proyeccion_margen: f64,
    /// Margen en el que cayó lo observado 8 de cada 10 veces en la prueba hacia atrás.
    pub proyeccion_margen_80: f64,
    /// Segunda proyección, con tasa de crecimiento constante (recta sobre el logaritmo): es el
    /// modelo natural de lo que crece en proporción a su tamaño. Se muestra al lado de la recta
    /// para que quien analiza vea cuánto cambia la cifra según el supuesto. Vacía si la serie tiene ceros.
    pub proyeccion_tasa_constante: Option<f64>,
    /// Margen de 8 de cada 10 casos de esa segunda proyección (error observado en la prueba hacia atrás).
    pub margen_tasa_constante_80: Option<f64>,
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

/// Error relativo (respecto de lo proyectado) que se observó al proyectar diez años con la recta de
/// cinco épocas, en 340 municipios y tres décadas (1,020 casos; `faro verificar` lo recalcula y
/// avisa si cambia): 8 de cada 10 veces el error fue de 20 % o menos; 95 de cada 100, de 47 % o menos.
pub const ERROR_OBSERVADO_P80: f64 = 0.20;
pub const ERROR_OBSERVADO_P95: f64 = 0.47;
/// Lo mismo para la proyección de tasa constante: en los mismos 1,020 casos su error fue de 24 % o
/// menos en 8 de cada 10. En conjunto acierta menos que la recta (error mediano de 14 % frente a
/// 9 %), por eso la recta es la cifra principal y esta es la de contraste.
pub const ERROR_TASA_CONSTANTE_P80: f64 = 0.24;

/// Analiza una serie: cambio, tasa anual, recta de tendencia y proyección al año pedido con sus
/// márgenes. Devuelve `None` si hay menos de 3 años distintos o si el primer valor no es positivo.
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
    // Error de predicción de una regresión simple: crece al alejarse del periodo observado. Se
    // multiplica por el cuantil 97.5 % de la t de Student con n − 2 grados de libertad (3.18 con
    // cinco puntos); el factor 2 solo sirve para series largas.
    let margen = crate::estadistica::t_975(s.len() - 2) * error * (1.0 + 1.0 / n + (x - mx).powi(2) / sxx).sqrt();
    // El error del ajuste subestima la incertidumbre cuando la serie es suave pero no lineal: en la
    // prueba hacia atrás solo contenía lo observado 1 de cada 4 veces. Por eso el margen nunca es
    // menor que el error realmente observado al proyectar diez años (ver `ERROR_OBSERVADO_*`).
    let proyeccion = (corte + pendiente * x).max(0.0);
    let horizonte = ((anio_proyectado - ultimo.anio).max(0) as f64 / 10.0).max(0.1);
    let margen = margen.max(proyeccion * ERROR_OBSERVADO_P95 * horizonte);
    let margen_80 = proyeccion * ERROR_OBSERVADO_P80 * horizonte;
    // Proyección de tasa constante: misma recta, ajustada al logaritmo de la serie.
    let tasa_constante = s.iter().all(|p| p.edificaciones > 0.0).then(|| {
        let ly = s.iter().map(|p| p.edificaciones.ln()).sum::<f64>() / n;
        let b = s.iter().map(|p| (p.anio as f64 - mx) * (p.edificaciones.ln() - ly)).sum::<f64>() / sxx;
        (ly + b * (x - mx)).exp()
    });
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
        proyeccion_tasa_constante: tasa_constante,
        margen_tasa_constante_80: tasa_constante.map(|t| t * ERROR_TASA_CONSTANTE_P80 * horizonte),
        confianza: confianza.to_string(),
    })
}

/// Hasta cuántos años más allá del último dato se prefiere la curva exponencial. Después, si la
/// serie deja ver un techo, se prefiere la logística.
pub const HORIZONTE_CORTO: i32 = 10;

/// Una población llevada más allá del último año de su serie con las dos curvas clásicas.
///
/// - **Exponencial** (tasa constante): sirve a corto plazo, cuando el ritmo reciente no ha tenido
///   tiempo de cambiar.
/// - **Logística** (crecimiento que se frena al acercarse a un techo): sirve a largo plazo, cuando
///   los recursos —suelo, agua, empleo— son limitados y el ritmo no puede sostenerse.
///
/// Las dos parten del último valor de la serie, para no dar un salto. Ninguna es una proyección
/// oficial: la oficial es la del INE, hecha por componentes demográficos, y llega hasta donde llega
/// la serie.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Extrapolacion {
    pub anio: i32,
    /// Años más allá del último dato de la serie.
    pub horizonte: i32,
    /// Tasa anual compuesta de los últimos cinco años de la serie, en %.
    pub tasa_reciente_pct: f64,
    /// Último valor creciendo a esa tasa constante.
    pub exponencial: f64,
    /// Valor de la logística ajustada a toda la serie. `None` si la serie no se frena (no deja ver
    /// un techo) o si la población baja.
    pub logistica: Option<f64>,
    /// Techo de la logística: el valor al que la curva se acerca sin pasarlo.
    pub techo: Option<f64>,
    /// "exponencial" o "logistica": la que corresponde al horizonte pedido.
    pub sugerida: &'static str,
    /// Valor de la curva sugerida.
    pub valor: f64,
}

/// Ajusta una curva logística a la serie y devuelve (techo, ritmo). Para cada techo candidato la
/// logística se vuelve una recta —ln(y ÷ (techo − y)) = a + ritmo × año— que se ajusta por mínimos
/// cuadrados; se queda con el techo que deja menos error en habitantes. Devuelve `None` cuando el
/// mejor techo es el mayor de los probados: la serie no se frena y no hay techo que estimar.
fn ajustar_logistica(serie: &[(i32, f64)]) -> Option<(f64, f64)> {
    let maximo = serie.iter().map(|p| p.1).fold(0.0, f64::max);
    let n = serie.len() as f64;
    let media_t = serie.iter().map(|p| p.0 as f64).sum::<f64>() / n;
    const PASOS: usize = 400;
    const MAYOR: f64 = 20.0;
    let mut mejor: Option<(f64, f64, f64)> = None; // (error, techo, ritmo)
    for k in 0..=PASOS {
        // Techos candidatos de 1.01 a 20 veces el máximo observado, más juntos cerca del máximo.
        let techo = maximo * 1.01 * (MAYOR / 1.01f64).powf(k as f64 / PASOS as f64);
        let z: Vec<f64> = serie.iter().map(|p| (p.1 / (techo - p.1)).ln()).collect();
        let media_z = z.iter().sum::<f64>() / n;
        let sxx: f64 = serie.iter().map(|p| (p.0 as f64 - media_t).powi(2)).sum();
        let sxz: f64 = serie.iter().zip(&z).map(|(p, z)| (p.0 as f64 - media_t) * (z - media_z)).sum();
        let ritmo = sxz / sxx;
        let a = media_z - ritmo * media_t;
        let error: f64 = serie.iter().map(|p| (techo / (1.0 + (-(a + ritmo * p.0 as f64)).exp()) - p.1).powi(2)).sum();
        if mejor.is_none_or(|m| error < m.0) {
            mejor = Some((error, techo, ritmo));
        }
    }
    let (_, techo, ritmo) = mejor?;
    (ritmo > 0.0 && techo < maximo * MAYOR * 0.98).then_some((techo, ritmo))
}

/// Lleva una serie anual de población (año, habitantes) hasta `anio`, más allá de su último dato.
/// Los años sin habitantes se dejan fuera (un municipio creado después del inicio de la serie trae
/// cero en sus primeros años). Devuelve `None` si quedan menos de seis años o si `anio` no es
/// posterior al último.
pub fn extrapolar(serie: &[(i32, f64)], anio: i32) -> Option<Extrapolacion> {
    let mut s: Vec<(i32, f64)> = serie.iter().copied().filter(|p| p.1.is_finite() && p.1 > 0.0).collect();
    s.sort_by_key(|p| p.0);
    s.dedup_by_key(|p| p.0);
    let (ultimo_anio, ultimo) = *s.last()?;
    if s.len() < 6 || anio <= ultimo_anio {
        return None;
    }
    let horizonte = anio - ultimo_anio;
    // Tasa reciente: los últimos cinco años de la serie (o los que haya).
    let base = s.iter().rev().find(|p| p.0 <= ultimo_anio - 5).copied().unwrap_or(s[0]);
    let tasa = (ultimo / base.1).powf(1.0 / (ultimo_anio - base.0) as f64) - 1.0;
    let exponencial = ultimo * (1.0 + tasa).powi(horizonte);
    // La logística describe un crecimiento que se frena: si la población baja, no aplica.
    let ajuste = if tasa > 0.0 { ajustar_logistica(&s) } else { None };
    let logistica = ajuste.map(|(techo, ritmo)| {
        // Se desplaza la curva para que pase por el último dato de la serie.
        let centro = ultimo_anio as f64 + (techo / ultimo - 1.0).ln() / ritmo;
        techo / (1.0 + (-ritmo * (anio as f64 - centro)).exp())
    });
    let (sugerida, valor) = match logistica {
        Some(l) if horizonte > HORIZONTE_CORTO => ("logistica", l),
        _ => ("exponencial", exponencial),
    };
    let redondo = |x: f64| x.round();
    Some(Extrapolacion { anio, horizonte, tasa_reciente_pct: (tasa * 10_000.0).round() / 100.0, exponencial: redondo(exponencial), logistica: logistica.map(redondo), techo: ajuste.map(|a| redondo(a.0)), sugerida, valor: redondo(valor) })
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

    #[test]
    fn la_exponencial_sirve_a_corto_plazo_y_la_logistica_a_largo() {
        // Serie logística exacta: techo 100,000, ritmo 0.08, centro en 2020.
        let curva = |t: i32| 100_000.0 / (1.0 + (-0.08 * (t as f64 - 2020.0)).exp());
        let serie: Vec<(i32, f64)> = (2015..=2030).map(|t| (t, curva(t))).collect();
        let corto = extrapolar(&serie, 2035).unwrap();
        assert_eq!((corto.horizonte, corto.sugerida), (5, "exponencial"));
        let largo = extrapolar(&serie, 2060).unwrap();
        assert_eq!(largo.sugerida, "logistica");
        // Recupera el techo con el que se hizo la serie y da el valor de la curva.
        assert!((largo.techo.unwrap() - 100_000.0).abs() < 1_500.0, "techo {:?}", largo.techo);
        assert!((largo.logistica.unwrap() - curva(2060)).abs() < 600.0);
        // A largo plazo la exponencial se pasa del techo; la logística no.
        assert!(largo.exponencial > 100_000.0 && largo.valor < 100_000.0);
    }

    #[test]
    fn sin_freno_no_hay_techo_y_si_baja_no_hay_logistica() {
        // Crece siempre 2 % al año: no hay techo que estimar; se sugiere la exponencial a cualquier plazo.
        let serie: Vec<(i32, f64)> = (2015..=2030).map(|t| (t, 50_000.0 * 1.02f64.powi(t - 2015))).collect();
        let e = extrapolar(&serie, 2055).unwrap();
        assert_eq!((e.logistica, e.sugerida), (None, "exponencial"));
        assert!((e.tasa_reciente_pct - 2.0).abs() < 0.01);
        assert!((e.exponencial - 50_000.0 * 1.02f64.powi(40)).abs() < 2.0);
        // Una población que baja sigue bajando a su ritmo; la logística no aplica.
        let baja: Vec<(i32, f64)> = (2015..=2030).map(|t| (t, 20_000.0 * 0.99f64.powi(t - 2015))).collect();
        let e = extrapolar(&baja, 2045).unwrap();
        assert!(e.logistica.is_none() && e.exponencial < 20_000.0 * 0.99f64.powi(15));
        // No se extrapola hacia atrás ni con series cortas.
        assert!(extrapolar(&serie, 2030).is_none() && extrapolar(&serie[..4], 2040).is_none());
        // Un municipio creado después del inicio de la serie trae cero al principio: se empieza en su primer año con población.
        let mut nuevo = serie.clone();
        nuevo[0].1 = 0.0;
        assert!((extrapolar(&nuevo, 2035).unwrap().exponencial - extrapolar(&serie, 2035).unwrap().exponencial).abs() < 1.0);
    }
}
