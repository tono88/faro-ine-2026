//! Superficie construida por municipio a partir de GHSL (GHS-BUILT-S, Comisión Europea, JRC):
//! mosaicos de 100 m en proyección Mollweide donde cada celda trae los metros cuadrados construidos.
//!
//! Aquí está lo geométrico: llevar los límites municipales a Mollweide y sumar las celdas cuyo
//! centro cae dentro de cada municipio. Leer el GeoTIFF queda fuera (lo hace el ejecutable).
//!
//! Cuidado de interpretación: es superficie construida vista por satélite (cualquier edificación),
//! no viviendas ni personas. Las épocas anteriores a 2015 vienen de imágenes Landsat y son menos
//! precisas; 2025 y 2030 en GHSL son proyecciones y no se usan como observación.
use crate::geo::Poligono;

/// Radio de la esfera de la proyección Mollweide mundial (ESRI:54009 usa el semieje mayor de WGS 84).
const RADIO_M: f64 = 6_378_137.0;

/// Longitud y latitud (grados) a Mollweide (metros).
pub fn mollweide(longitud: f64, latitud: f64) -> (f64, f64) {
    let (lam, phi) = (longitud.to_radians(), latitud.to_radians());
    // 2θ + sen 2θ = π sen φ, por Newton.
    let objetivo = std::f64::consts::PI * phi.sin();
    let mut t = phi;
    for _ in 0..50 {
        let d = (2.0 * t + (2.0 * t).sin() - objetivo) / (2.0 + 2.0 * (2.0 * t).cos()).max(1e-12);
        t -= d;
        if d.abs() < 1e-12 {
            break;
        }
    }
    (2.0 * 2f64.sqrt() / std::f64::consts::PI * RADIO_M * lam * t.cos(), 2f64.sqrt() * RADIO_M * t.sin())
}

/// Mollweide (metros) a longitud y latitud (grados).
pub fn mollweide_inversa(x: f64, y: f64) -> (f64, f64) {
    let t = (y / (2f64.sqrt() * RADIO_M)).clamp(-1.0, 1.0).asin();
    let latitud = ((2.0 * t + (2.0 * t).sin()) / std::f64::consts::PI).clamp(-1.0, 1.0).asin();
    let longitud = std::f64::consts::PI * x / (2.0 * 2f64.sqrt() * RADIO_M * t.cos());
    (longitud.to_degrees(), latitud.to_degrees())
}

pub fn proyectar(poligonos: &[Poligono]) -> Vec<Poligono> {
    poligonos.iter().map(|p| p.iter().map(|a| a.iter().map(|(x, y)| mollweide(*x, *y)).collect()).collect()).collect()
}

/// Un mosaico: celdas de `paso` metros; (x0, y0) es la esquina superior izquierda.
pub struct Mosaico<'a> {
    pub ancho: usize,
    pub alto: usize,
    pub x0: f64,
    pub y0: f64,
    pub paso: f64,
    pub celdas: &'a [u16],
    /// Valor que significa "sin dato".
    pub sin_dato: u16,
}

/// Suma de las celdas del mosaico cuyo centro cae dentro de los polígonos (ya en Mollweide), y
/// cuántas celdas se sumaron. Recorre por filas y, en cada una, los tramos que quedan dentro.
pub fn sumar(m: &Mosaico, poligonos: &[Poligono]) -> (f64, usize) {
    let (mut suma, mut celdas) = (0.0, 0usize);
    recorrer(m, poligonos, &mut |_, _, v| {
        suma += v as f64;
        celdas += 1;
    });
    (suma, celdas)
}

/// Llama a `f(x, y, valor)` por cada celda con dato cuyo centro (x, y en Mollweide) cae dentro
/// de los polígonos.
pub fn recorrer(m: &Mosaico, poligonos: &[Poligono], f: &mut dyn FnMut(f64, f64, u16)) {
    for p in poligonos {
        let (mut ymin, mut ymax) = (f64::INFINITY, f64::NEG_INFINITY);
        for (_, y) in p.iter().flatten() {
            (ymin, ymax) = (ymin.min(*y), ymax.max(*y));
        }
        let fila_desde = (((m.y0 - ymax) / m.paso - 0.5).ceil().max(0.0)) as usize;
        let fila_hasta = (((m.y0 - ymin) / m.paso - 0.5).floor()).min(m.alto as f64 - 1.0);
        if fila_hasta < 0.0 {
            continue;
        }
        for fila in fila_desde..=fila_hasta as usize {
            let y = m.y0 - (fila as f64 + 0.5) * m.paso;
            // Cruces de la horizontal con todos los anillos: entre cruces pares e impares se está
            // dentro (los huecos quedan fuera solos, por paridad).
            let mut cruces: Vec<f64> = Vec::new();
            for anillo in p {
                let mut j = anillo.len().wrapping_sub(1);
                for i in 0..anillo.len() {
                    let ((xi, yi), (xj, yj)) = (anillo[i], anillo[j]);
                    if (yi > y) != (yj > y) {
                        cruces.push((xj - xi) * (y - yi) / (yj - yi) + xi);
                    }
                    j = i;
                }
            }
            cruces.sort_by(f64::total_cmp);
            for par in cruces.chunks_exact(2) {
                let desde = (((par[0] - m.x0) / m.paso - 0.5).ceil().max(0.0)) as usize;
                let hasta = ((par[1] - m.x0) / m.paso - 0.5).floor().min(m.ancho as f64 - 1.0);
                if hasta < 0.0 {
                    continue;
                }
                for col in desde..=hasta as usize {
                    let v = m.celdas[fila * m.ancho + col];
                    if v != m.sin_dato {
                        f(m.x0 + (col as f64 + 0.5) * m.paso, y, v);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn mollweide_en_puntos_conocidos() {
        assert_eq!(mollweide(0.0, 0.0), (0.0, 0.0));
        let (x, y) = mollweide(180.0, 0.0);
        assert!((x - 2.0 * 2f64.sqrt() * RADIO_M).abs() < 1e-3 && y.abs() < 1e-9);
        let (_, y) = mollweide(0.0, 90.0);
        assert!((y - 2f64.sqrt() * RADIO_M).abs() < 1.0);
        let (lon, lat) = mollweide_inversa(mollweide(-90.51, 14.63).0, mollweide(-90.51, 14.63).1);
        assert!((lon + 90.51).abs() < 1e-6 && (lat - 14.63).abs() < 1e-6, "ida y vuelta: {lon}, {lat}");
        // Ciudad de Guatemala cae en el mosaico R8_C10 de GHSL (x de −9,041,000 a −8,041,000; y de 1,000,000 a 2,000,000).
        let (x, y) = mollweide(-90.51, 14.63);
        assert!((-9_041_000.0..-8_041_000.0).contains(&x) && (1_000_000.0..2_000_000.0).contains(&y), "{x}, {y}");
    }

    #[test]
    fn suma_las_celdas_cuyo_centro_cae_dentro_y_respeta_huecos_y_sin_dato() {
        // Mosaico de 4 x 4 celdas de 100 m con esquina en (0, 400); cada celda vale 10, una sin dato.
        let mut celdas = vec![10u16; 16];
        celdas[5] = 65535;
        let m = Mosaico { ancho: 4, alto: 4, x0: 0.0, y0: 400.0, paso: 100.0, celdas: &celdas, sin_dato: 65535 };
        let todo: Poligono = vec![vec![(0.0, 0.0), (400.0, 0.0), (400.0, 400.0), (0.0, 400.0), (0.0, 0.0)]];
        assert_eq!(sumar(&m, &[todo.clone()]), (150.0, 15));
        // La mitad izquierda: centros en x = 50 y 150.
        let mitad: Poligono = vec![vec![(0.0, 0.0), (200.0, 0.0), (200.0, 400.0), (0.0, 400.0), (0.0, 0.0)]];
        assert_eq!(sumar(&m, &[mitad]), (70.0, 7));
        // Con un hueco que cubre las dos celdas de abajo a la derecha.
        let con_hueco: Poligono = vec![todo[0].clone(), vec![(200.0, 0.0), (400.0, 0.0), (400.0, 100.0), (200.0, 100.0), (200.0, 0.0)]];
        assert_eq!(sumar(&m, &[con_hueco]), (130.0, 13));
        // Un polígono que se sale del mosaico solo suma lo que hay.
        let fuera: Poligono = vec![vec![(300.0, 300.0), (900.0, 300.0), (900.0, 900.0), (300.0, 900.0), (300.0, 300.0)]];
        assert_eq!(sumar(&m, &[fuera]), (10.0, 1));
    }
}
