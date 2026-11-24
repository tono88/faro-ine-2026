//! Estadística de apoyo para dar, junto a cada cifra, cuánta confianza merece.
//! Todo es exacto y determinista: no se usan remuestreos al azar.

/// Intervalo de confianza de la mediana sin suponer ninguna distribución (estadísticos de orden).
/// `ordenados` debe venir de menor a mayor. Devuelve (inferior, superior, cobertura real) con una
/// cobertura de al menos `nivel` (por ejemplo 0.95), o `None` si hay muy pocos datos para lograrla
/// (con menos de 6 valores no se puede llegar a 95 %).
pub fn ic_mediana(ordenados: &[f64], nivel: f64) -> Option<(f64, f64, f64)> {
    let n = ordenados.len();
    if n < 2 {
        return None;
    }
    // P(X <= k) con X ~ Binomial(n, 1/2), acumulada término a término.
    let mut termino = 0.5f64.powi(n as i32);
    let mut acumulada = 0.0;
    let mut j = 0; // cuántos valores se dejan fuera en cada cola
    for k in 0..n / 2 {
        acumulada += termino;
        if acumulada > (1.0 - nivel) / 2.0 {
            break;
        }
        j = k + 1;
        termino *= (n - k) as f64 / (k + 1) as f64;
    }
    if j == 0 {
        return None;
    }
    // Cobertura exacta de [x_(j), x_(n-j+1)] = 1 - 2·P(X <= j-1).
    let mut t = 0.5f64.powi(n as i32);
    let mut cola = 0.0;
    for k in 0..j {
        cola += t;
        t *= (n - k) as f64 / (k + 1) as f64;
    }
    Some((ordenados[j - 1], ordenados[n - j], 1.0 - 2.0 * cola))
}

/// Intervalo de Wilson al 95 % para una proporción (aciertos de un total). Sirve también cuando
/// los aciertos son cero: da el techo razonable de la proporción verdadera.
pub fn wilson(aciertos: usize, total: usize) -> Option<(f64, f64)> {
    if total == 0 {
        return None;
    }
    let (n, p, z) = (total as f64, aciertos as f64 / total as f64, 1.959964f64);
    let centro = (p + z * z / (2.0 * n)) / (1.0 + z * z / n);
    let mitad = z * (p * (1.0 - p) / n + z * z / (4.0 * n * n)).sqrt() / (1.0 + z * z / n);
    Some(((centro - mitad).max(0.0), (centro + mitad).min(1.0)))
}

/// Cuantil 97.5 % de la t de Student (para intervalos al 95 % con pocos datos). Con 3 grados de
/// libertad vale 3.18, no 2: usar 2 con series cortas da márgenes demasiado estrechos.
pub fn t_975(grados: usize) -> f64 {
    const TABLA: [f64; 30] = [12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179, 2.160, 2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064, 2.060, 2.056, 2.052, 2.048, 2.045, 2.042];
    match grados {
        0 => f64::INFINITY,
        g if g <= 30 => TABLA[g - 1],
        _ => 1.96,
    }
}

/// Percentil por interpolación lineal (p entre 0 y 1) de valores ya ordenados.
pub fn percentil(ordenados: &[f64], p: f64) -> Option<f64> {
    if ordenados.is_empty() {
        return None;
    }
    let pos = p.clamp(0.0, 1.0) * (ordenados.len() - 1) as f64;
    let (i, f) = (pos.floor() as usize, pos.fract());
    Some(if i + 1 < ordenados.len() { ordenados[i] * (1.0 - f) + ordenados[i + 1] * f } else { ordenados[i] })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intervalo_de_la_mediana_por_estadisticos_de_orden() {
        // Con 5 valores no se alcanza 95 %; con 6 sí, usando el mínimo y el máximo (96.9 %).
        assert!(ic_mediana(&[1.0, 2.0, 3.0, 4.0, 5.0], 0.95).is_none());
        let (a, b, c) = ic_mediana(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0], 0.95).unwrap();
        assert_eq!((a, b), (1.0, 6.0));
        assert!((c - 0.96875).abs() < 1e-9);
        // Con 20 valores, la tabla clásica da los puestos 6 y 15 (cobertura 95.9 %).
        let v: Vec<f64> = (1..=20).map(f64::from).collect();
        let (a, b, c) = ic_mediana(&v, 0.95).unwrap();
        assert_eq!((a, b), (6.0, 15.0));
        assert!((c - 0.9586).abs() < 1e-3);
    }

    #[test]
    fn wilson_y_percentiles() {
        let (bajo, alto) = wilson(0, 300).unwrap();
        assert!(bajo < 1e-9 && (alto - 0.0126).abs() < 5e-4, "techo con 0 de 300: {alto}");
        let (bajo, alto) = wilson(29, 300).unwrap();
        assert!((bajo - 0.068).abs() < 2e-3 && (alto - 0.135).abs() < 2e-3, "{bajo} {alto}");
        assert!(wilson(1, 0).is_none());
        assert_eq!(percentil(&[10.0, 20.0, 30.0], 0.5), Some(20.0));
        assert_eq!(percentil(&[10.0, 20.0], 0.25), Some(12.5));
    }
}
