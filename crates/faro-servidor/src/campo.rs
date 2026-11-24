//! Trabajo de campo: el orden de la ruta de un operador, el enlace al formulario de la encuesta y el
//! cruce de cada visita con lo que ya se sabe del lugar.
//!
//! Nada de esto decide por el supervisor. El cruce compara cuatro cosas independientes (la posición
//! que entrega el teléfono, la foto, lo que ve el satélite en ese kilómetro y lo que declaró el censo
//! en ese lugar poblado) y dice si coinciden o si conviene revisar la visita, con sus razones.
use serde::Serialize;

/// Hasta esta distancia del punto del lugar poblado la visita se toma como hecha en el lugar.
/// El punto del censo es el centro de la aldea o caserío, no la vivienda: por eso el margen es amplio.
pub const CERCA_M: f64 = 1_000.0;
/// Más allá de esta distancia la visita se marca para revisar.
pub const LEJOS_M: f64 = 3_000.0;
/// Confianza mínima para tomar en cuenta lo que dice el clasificador de la foto.
pub const CONFIANZA_MINIMA: f32 = 0.5;

/// Distancia en metros entre dos puntos (longitud, latitud) sobre la esfera (fórmula del haversine).
pub fn distancia_m(a: (f64, f64), b: (f64, f64)) -> f64 {
    let (lat1, lat2) = (a.1.to_radians(), b.1.to_radians());
    let (dlat, dlon) = (lat2 - lat1, (b.0 - a.0).to_radians());
    let h = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    2.0 * 6_371_008.8 * h.sqrt().min(1.0).asin()
}

/// Largo del recorrido, en metros, visitando los puntos en ese orden (en línea recta).
pub fn largo_m(puntos: &[(f64, f64)], orden: &[usize]) -> f64 {
    orden.windows(2).map(|p| distancia_m(puntos[p[0]], puntos[p[1]])).sum()
}

/// Ordena las paradas para acortar el recorrido: parte de la primera, va siempre a la más cercana
/// que falte y después mejora el resultado deshaciendo cruces (2-opt). Es una buena ruta, no la
/// óptima demostrada: con decenas de paradas el óptimo exacto no se calcula en un tiempo razonable.
pub fn ordenar_ruta(puntos: &[(f64, f64)]) -> Vec<usize> {
    let n = puntos.len();
    if n < 3 {
        return (0..n).collect();
    }
    let mut orden = vec![0usize];
    let mut faltan: Vec<usize> = (1..n).collect();
    while !faltan.is_empty() {
        let actual = puntos[*orden.last().unwrap_or(&0)];
        let (k, _) = faltan.iter().enumerate().map(|(k, i)| (k, distancia_m(actual, puntos[*i]))).fold((0, f64::MAX), |m, x| if x.1 < m.1 { x } else { m });
        orden.push(faltan.swap_remove(k));
    }
    // 2-opt: invertir un tramo si con eso el recorrido queda más corto. La salida (índice 0) no se mueve.
    let mut mejoro = true;
    let mut vueltas = 0;
    while mejoro && vueltas < 60 {
        mejoro = false;
        vueltas += 1;
        for i in 1..n - 1 {
            for j in i + 1..n {
                let antes = distancia_m(puntos[orden[i - 1]], puntos[orden[i]]) + if j + 1 < n { distancia_m(puntos[orden[j]], puntos[orden[j + 1]]) } else { 0.0 };
                let despues = distancia_m(puntos[orden[i - 1]], puntos[orden[j]]) + if j + 1 < n { distancia_m(puntos[orden[i]], puntos[orden[j + 1]]) } else { 0.0 };
                if despues + 1e-6 < antes {
                    orden[i..=j].reverse();
                    mejoro = true;
                }
            }
        }
    }
    orden
}

/// Revisa la dirección del formulario de la encuesta: debe ser https y de un servicio admitido.
pub fn formulario_admitido(url: &str, admitidos: &[String]) -> Result<(), String> {
    if url.is_empty() {
        return Ok(());
    }
    let resto = url.strip_prefix("https://").ok_or("la dirección del formulario debe empezar por https://")?;
    let servidor = resto.split(['/', '?', '#']).next().unwrap_or("").to_lowercase();
    if servidor.contains('@') || servidor.contains(':') || url.chars().count() > 1500 || url.chars().any(|c| c.is_control() || c == ' ') {
        return Err("la dirección del formulario no es válida".into());
    }
    if admitidos.iter().any(|a| servidor == *a || servidor.ends_with(&format!(".{a}"))) {
        Ok(())
    } else {
        Err(format!("el formulario debe estar en uno de estos servicios: {}", admitidos.join(", ")))
    }
}

/// Llena la dirección del formulario: cada marcador `{nombre}` se cambia por su valor, ya codificado
/// para viajar en una dirección. Los marcadores que no se conocen se dejan como están.
pub fn enlace_formulario(plantilla: &str, valores: &[(&str, String)]) -> String {
    let codificar = |t: &str| t.bytes().map(|b| if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) { (b as char).to_string() } else { format!("%{b:02X}") }).collect::<String>();
    valores.iter().fold(plantilla.to_string(), |url, (marcador, valor)| url.replace(&format!("{{{marcador}}}"), &codificar(valor)))
}

/// Lo que se sabe de una visita por caminos independientes.
#[derive(Debug, Default, Clone)]
pub struct Evidencia {
    /// Metros entre la posición del teléfono y el punto del lugar poblado asignado.
    pub distancia_m: Option<f64>,
    /// Radio de error que informa el teléfono, en metros.
    pub precision_m: Option<f64>,
    /// Clase que dio el clasificador de la foto ("vivienda", "comercio" u "otro") y su confianza.
    pub foto: Option<(String, f32)>,
    /// Hectáreas construidas en el bloque de 1 km donde cayó la posición (GHSL 2020).
    pub construido_ha: Option<f64>,
    /// De esas, cuántas son no residenciales (GHSL, componente NRES).
    pub no_residencial_ha: Option<f64>,
    /// ¿Hay datos del satélite cargados? Si no los hay, no decir que "el satélite no ve construcción".
    pub hay_satelite: bool,
    /// Material de techo más frecuente en el lugar poblado y su porcentaje (Censo 2018).
    pub techo: Option<(String, f64)>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct Cruce {
    /// "coherente", "revisar" o "sin_datos".
    pub coherencia: &'static str,
    /// Lo que coincide.
    pub a_favor: Vec<String>,
    /// Lo que no coincide: por esto se pide revisar.
    pub dudas: Vec<String>,
    /// Contexto que no inclina la balanza.
    pub notas: Vec<String>,
}

fn metros(d: f64) -> String {
    if d < 1_000.0 { format!("{d:.0} m") } else { format!("{:.1} km", d / 1_000.0) }
}

/// Cruza las evidencias de una visita. Cada regla es simple y está a la vista; ninguna rechaza la
/// visita: «revisar» es un aviso para el supervisor.
pub fn cruzar(e: &Evidencia) -> Cruce {
    let (mut a_favor, mut dudas, mut notas) = (Vec::new(), Vec::new(), Vec::new());
    match e.distancia_m {
        None => dudas.push("el teléfono no entregó la posición: no se puede confirmar dónde se hizo la visita".to_string()),
        Some(d) => {
            let margen = e.precision_m.unwrap_or(0.0);
            if d <= CERCA_M + margen {
                a_favor.push(format!("posición a {} del punto del lugar poblado", metros(d)));
            } else if d <= LEJOS_M + margen {
                notas.push(format!("posición a {} del punto del lugar poblado: dentro de lo esperable en un lugar disperso", metros(d)));
            } else {
                dudas.push(format!("la posición está a {} del lugar poblado asignado", metros(d)));
            }
            if margen > 200.0 {
                notas.push(format!("el teléfono informó una precisión baja (± {})", metros(margen)));
            }
        }
    }
    match &e.foto {
        None => notas.push("sin foto clasificada".to_string()),
        Some((clase, confianza)) if *confianza < CONFIANZA_MINIMA => notas.push(format!("la foto no es concluyente (lo más parecido: {clase}, {:.0} %)", confianza * 100.0)),
        Some((clase, confianza)) => match clase.as_str() {
            "vivienda" => a_favor.push(format!("la foto parece una vivienda ({:.0} %)", confianza * 100.0)),
            "comercio" => dudas.push(format!("la foto parece un comercio o un local, no una vivienda ({:.0} %)", confianza * 100.0)),
            _ => dudas.push(format!("la foto no muestra una vivienda ({:.0} % de otra cosa)", confianza * 100.0)),
        },
    }
    if e.distancia_m.is_some() && e.hay_satelite {
        match e.construido_ha {
            None => dudas.push("el satélite no registra construcción en ese kilómetro (GHSL 2020)".to_string()),
            Some(ha) => {
                let nres = e.no_residencial_ha.unwrap_or(0.0);
                if ha > 0.0 && nres / ha > 0.5 {
                    dudas.push(format!("ese kilómetro es mayormente no residencial según el satélite ({:.0} % de {ha:.1} ha construidas)", 100.0 * nres / ha));
                } else {
                    a_favor.push(format!("el satélite registra {ha:.1} ha construidas en ese kilómetro, {:.0} % residencial", 100.0 * (1.0 - nres / ha.max(1e-9))));
                }
            }
        }
    }
    if let Some((material, pct)) = &e.techo {
        notas.push(format!("techo más frecuente en el lugar según el Censo 2018: {material} ({pct:.0} % de las viviendas)"));
    }
    let coherencia = if !dudas.is_empty() { "revisar" } else if a_favor.is_empty() { "sin_datos" } else { "coherente" };
    Cruce { coherencia, a_favor, dudas, notas }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_ruta_se_ordena_y_no_empeora() {
        // Cuatro esquinas de un cuadrado dadas en zigzag: el orden bueno las recorre por el borde.
        let puntos = [(-90.50, 14.60), (-90.40, 14.70), (-90.40, 14.60), (-90.50, 14.70)];
        let orden = ordenar_ruta(&puntos);
        assert_eq!(orden[0], 0, "la salida no se mueve");
        let mut vistos = orden.clone();
        vistos.sort();
        assert_eq!(vistos, vec![0, 1, 2, 3], "cada parada aparece una vez");
        assert!(largo_m(&puntos, &orden) < largo_m(&puntos, &[0, 1, 2, 3]) - 1_000.0);
        // Un grado de latitud mide unos 111 km.
        assert!((distancia_m((-90.0, 14.0), (-90.0, 15.0)) - 111_195.0).abs() < 100.0);
        assert_eq!(ordenar_ruta(&puntos[..2]), vec![0, 1]);
    }

    #[test]
    fn el_formulario_se_valida_y_se_llena() {
        let admitidos = vec!["forms.office.com".to_string(), "forms.cloud.microsoft".to_string()];
        assert!(formulario_admitido("https://forms.office.com/r/AbC123?r1={lugar}", &admitidos).is_ok());
        assert!(formulario_admitido("https://customer.forms.office.com/r/x", &admitidos).is_ok());
        assert!(formulario_admitido("", &admitidos).is_ok());
        // http, otro servidor, o un servidor disfrazado con usuario: se rechazan.
        assert!(formulario_admitido("http://forms.office.com/r/x", &admitidos).is_err());
        assert!(formulario_admitido("https://forms.office.com.ejemplo.net/r/x", &admitidos).is_err());
        assert!(formulario_admitido("https://forms.office.com@ejemplo.net/r/x", &admitidos).is_err());
        let url = enlace_formulario("https://forms.office.com/r/x?lugar={lugar}&n={nombre}&lat={latitud}&otro={desconocido}", &[("lugar", "0701005".into()), ("nombre", "San José & Anexos".into()), ("latitud", "14.7712".into())]);
        assert_eq!(url, "https://forms.office.com/r/x?lugar=0701005&n=San%20Jos%C3%A9%20%26%20Anexos&lat=14.7712&otro={desconocido}");
    }

    #[test]
    fn el_cruce_dice_por_que() {
        // Todo coincide.
        let c = cruzar(&Evidencia { distancia_m: Some(240.0), precision_m: Some(12.0), foto: Some(("vivienda".into(), 0.83)), construido_ha: Some(6.2), no_residencial_ha: Some(0.1), hay_satelite: true, techo: Some(("lámina".into(), 71.0)) });
        assert_eq!((c.coherencia, c.a_favor.len(), c.dudas.len()), ("coherente", 3, 0));
        // Lejos, con foto de comercio y en zona industrial: tres dudas distintas.
        let c = cruzar(&Evidencia { distancia_m: Some(8_400.0), precision_m: Some(20.0), foto: Some(("comercio".into(), 0.71)), construido_ha: Some(30.0), no_residencial_ha: Some(22.0), hay_satelite: true, techo: None });
        assert_eq!((c.coherencia, c.dudas.len()), ("revisar", 3));
        assert!(c.dudas[0].contains("8.4 km"));
        // Una foto dudosa no acusa a nadie: queda como nota.
        let c = cruzar(&Evidencia { distancia_m: Some(1_800.0), precision_m: None, foto: Some(("comercio".into(), 0.31)), construido_ha: Some(1.0), no_residencial_ha: None, hay_satelite: true, techo: None });
        assert_eq!((c.coherencia, c.dudas.len(), c.notas.len()), ("coherente", 0, 2));
        // Sin posición no hay cómo confirmar el lugar; sin capa de satélite no se afirma nada del satélite.
        let c = cruzar(&Evidencia::default());
        assert_eq!((c.coherencia, c.dudas.len()), ("revisar", 1));
        let c = cruzar(&Evidencia { distancia_m: Some(100.0), hay_satelite: false, ..Evidencia::default() });
        assert_eq!((c.coherencia, c.a_favor.len()), ("coherente", 1));
        // Con satélite cargado y sin bloque construido cerca, sí es una duda.
        let c = cruzar(&Evidencia { distancia_m: Some(100.0), hay_satelite: true, ..Evidencia::default() });
        assert_eq!(c.coherencia, "revisar");
    }
}
