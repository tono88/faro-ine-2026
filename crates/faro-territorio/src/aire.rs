//! Calidad del aire: promedio mensual por estación y contaminante, con la regla de
//! representatividad que planteó el INE (23-nov-2026, "idea general"):
//! un día con más de 4 horas sin dato no cuenta; un mes con más de 2 días que no cuentan se descarta.
//! Ambos umbrales son parámetros.
//!
//! Formato de entrada (INSIVUMEH): primera fila con el nombre de cada estación sobre sus columnas,
//! segunda fila con el contaminante, y una fila por hora (Mes, Fecha, Hora, valores...). "N.D." = sin dato.
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MesAire {
    pub estacion: String,
    pub contaminante: String,
    pub mes: String,
    pub dias_con_registro: usize,
    pub dias_validos: usize,
    pub dias_no_validos: usize,
    pub horas_con_dato: usize,
    /// Promedio de los promedios diarios de los días válidos. Vacío si el mes se descarta.
    pub promedio: Option<f64>,
    pub estado: &'static str,
}

pub fn promedios_mensuales(filas: &[Vec<String>], max_horas_sin_dato: usize, max_dias_no_validos: usize) -> Result<Vec<MesAire>, String> {
    if filas.len() < 3 {
        return Err("se esperaban dos filas de encabezado y datos".into());
    }
    // La estación se escribe una vez y abarca las columnas siguientes hasta la próxima.
    let mut series: Vec<(usize, String, String)> = vec![];
    let mut estacion = String::new();
    for (i, contaminante) in filas[1].iter().enumerate().skip(3) {
        if let Some(nombre) = filas[0].get(i).filter(|n| !n.is_empty()) {
            estacion = nombre.clone();
        }
        if !contaminante.is_empty() && !estacion.is_empty() {
            series.push((i, estacion.clone(), contaminante.split('(').next().unwrap_or("").trim().replace(' ', "")));
        }
    }
    if series.is_empty() {
        return Err("no se encontraron columnas de estación/contaminante".into());
    }
    // (serie, mes, día) -> (suma, horas con dato)
    let mut dias: BTreeMap<(usize, String, String), (f64, usize)> = BTreeMap::new();
    for fila in &filas[2..] {
        let Some(fecha) = fila.get(1).and_then(|f| f.get(..10)).filter(|f| f.len() == 10) else { continue };
        for (k, (i, _, _)) in series.iter().enumerate() {
            let celda = dias.entry((k, fecha[..7].to_string(), fecha.to_string())).or_insert((0.0, 0));
            if let Some(v) = fila.get(*i).and_then(|v| v.trim().parse::<f64>().ok()).filter(|v| v.is_finite() && *v >= 0.0) {
                celda.0 += v;
                celda.1 += 1;
            }
        }
    }
    let mut meses: BTreeMap<(usize, String), Vec<(f64, usize)>> = BTreeMap::new();
    for ((k, mes, _), d) in dias {
        meses.entry((k, mes)).or_default().push(d);
    }
    Ok(meses
        .into_iter()
        .map(|((k, mes), d)| {
            let validos: Vec<f64> = d.iter().filter(|(_, h)| 24usize.saturating_sub(*h) <= max_horas_sin_dato).map(|(s, h)| s / *h as f64).collect();
            let no_validos = d.len() - validos.len();
            let horas: usize = d.iter().map(|(_, h)| h).sum();
            let (promedio, estado) = if horas == 0 {
                (None, "sin_datos")
            } else if no_validos > max_dias_no_validos {
                (None, "descartado")
            } else {
                (Some(validos.iter().sum::<f64>() / validos.len() as f64), "valido")
            };
            MesAire { estacion: series[k].1.clone(), contaminante: series[k].2.clone(), mes, dias_con_registro: d.len(), dias_validos: validos.len(), dias_no_validos: no_validos, horas_con_dato: horas, promedio, estado }
        })
        .collect())
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn datos(faltan_por_dia: &[usize]) -> Vec<Vec<String>> {
        let s = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        let mut f = vec![s(&["Año", "2025", "", "ESTACION A", ""]), s(&["Mes", "Fecha", "Hora", "PM 10 (ug/m3)", "PM2.5 (ug/m3)"])];
        for (d, faltan) in faltan_por_dia.iter().enumerate() {
            for h in 0..24 {
                let v = if h < *faltan { "N.D.".to_string() } else { "10".to_string() };
                f.push(vec!["enero".into(), format!("2025-01-{:02}", d + 1), format!("{h:02}:00:00"), v, "N.D.".into()]);
            }
        }
        f
    }

    #[test]
    fn dia_con_cuatro_horas_sin_dato_cuenta_y_con_cinco_no() {
        let r = promedios_mensuales(&datos(&[0, 4, 5]), 4, 2).unwrap();
        let pm10 = &r[0];
        assert_eq!((pm10.estacion.as_str(), pm10.contaminante.as_str()), ("ESTACION A", "PM10"));
        assert_eq!((pm10.dias_validos, pm10.dias_no_validos, pm10.estado), (2, 1, "valido"));
        assert_eq!(pm10.promedio, Some(10.0));
        // La segunda columna hereda el nombre de la estación y no tiene ningún dato.
        assert_eq!((r[1].estacion.as_str(), r[1].contaminante.as_str(), r[1].estado), ("ESTACION A", "PM2.5", "sin_datos"));
    }

    #[test]
    fn mes_con_mas_de_dos_dias_no_validos_se_descarta() {
        assert_eq!(promedios_mensuales(&datos(&[0, 0, 9, 9]), 4, 2).unwrap()[0].estado, "valido");
        let r = promedios_mensuales(&datos(&[0, 0, 9, 9, 24]), 4, 2).unwrap();
        assert_eq!((r[0].estado, r[0].promedio), ("descartado", None));
    }
}
