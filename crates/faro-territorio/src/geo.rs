//! Geometría mínima para capas: punto dentro de polígono y simplificación de contornos.
use crate::asignacion::Coordenada;

/// Un anillo es una lista cerrada de (longitud, latitud). El primero de un polígono es el contorno
/// y los demás son huecos.
pub type Anillo = Vec<(f64, f64)>;
pub type Poligono = Vec<Anillo>;

fn en_anillo(anillo: &[(f64, f64)], x: f64, y: f64) -> bool {
    let mut dentro = false;
    let mut j = anillo.len().wrapping_sub(1);
    for i in 0..anillo.len() {
        let ((xi, yi), (xj, yj)) = (anillo[i], anillo[j]);
        if (yi > y) != (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi {
            dentro = !dentro;
        }
        j = i;
    }
    dentro
}

/// ¿El punto cae dentro de alguno de los polígonos (respetando huecos)?
pub fn contiene(poligonos: &[Poligono], c: Coordenada) -> bool {
    poligonos.iter().any(|p| p.first().is_some_and(|contorno| en_anillo(contorno, c.longitud, c.latitud)) && !p.iter().skip(1).any(|hueco| en_anillo(hueco, c.longitud, c.latitud)))
}

/// Caja (min_lon, min_lat, max_lon, max_lat) que encierra los polígonos.
pub fn caja(poligonos: &[Poligono]) -> (f64, f64, f64, f64) {
    let mut c = (f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
    for (x, y) in poligonos.iter().flatten().flatten() {
        c = (c.0.min(*x), c.1.min(*y), c.2.max(*x), c.3.max(*y));
    }
    c
}

fn distancia_a_segmento(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let largo2 = dx * dx + dy * dy;
    let t = if largo2 == 0.0 { 0.0 } else { (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / largo2).clamp(0.0, 1.0) };
    ((p.0 - a.0 - t * dx).powi(2) + (p.1 - a.1 - t * dy).powi(2)).sqrt()
}

/// Simplifica una línea (Douglas–Peucker): quita puntos que se apartan menos de `tolerancia`
/// (en grados) de la línea que queda. Conserva los extremos.
pub fn simplificar(linea: &[(f64, f64)], tolerancia: f64) -> Vec<(f64, f64)> {
    if linea.len() < 3 {
        return linea.to_vec();
    }
    let mut conservar = vec![false; linea.len()];
    conservar[0] = true;
    conservar[linea.len() - 1] = true;
    let mut pila = vec![(0usize, linea.len() - 1)];
    while let Some((a, b)) = pila.pop() {
        let (mut lejos, mut k) = (0.0, a);
        for i in a + 1..b {
            let d = distancia_a_segmento(linea[i], linea[a], linea[b]);
            if d > lejos {
                (lejos, k) = (d, i);
            }
        }
        if lejos > tolerancia {
            conservar[k] = true;
            pila.push((a, k));
            pila.push((k, b));
        }
    }
    linea.iter().zip(conservar).filter(|(_, c)| *c).map(|(p, _)| *p).collect()
}

/// Simplifica cada anillo; descarta los que quedan sin área (menos de 4 puntos).
pub fn simplificar_poligonos(poligonos: &[Poligono], tolerancia: f64) -> Vec<Poligono> {
    poligonos
        .iter()
        .filter_map(|p| {
            let anillos: Vec<Anillo> = p.iter().enumerate().filter_map(|(i, a)| Some(simplificar(a, tolerancia)).filter(|s| s.len() >= 4 || (i == 0 && a.len() >= 4 && s.len() >= 4))).collect();
            // Si el contorno desapareció, el polígono entero se omite.
            (p.first().map(|c| simplificar(c, tolerancia).len() >= 4).unwrap_or(false)).then_some(anillos)
        })
        .collect()
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn c(longitud: f64, latitud: f64) -> Coordenada {
        Coordenada { longitud, latitud }
    }

    #[test]
    fn punto_dentro_fuera_y_en_un_hueco() {
        let cuadro: Poligono = vec![vec![(0.0, 0.0), (4.0, 0.0), (4.0, 4.0), (0.0, 4.0), (0.0, 0.0)], vec![(1.0, 1.0), (2.0, 1.0), (2.0, 2.0), (1.0, 2.0), (1.0, 1.0)]];
        let p = vec![cuadro];
        assert!(contiene(&p, c(3.0, 3.0)));
        assert!(!contiene(&p, c(1.5, 1.5)), "dentro del hueco");
        assert!(!contiene(&p, c(5.0, 3.0)));
        assert_eq!(caja(&p), (0.0, 0.0, 4.0, 4.0));
    }

    #[test]
    fn simplificar_quita_lo_que_no_cambia_la_forma() {
        let linea = vec![(0.0, 0.0), (1.0, 0.001), (2.0, 0.0), (3.0, 1.0), (4.0, 0.0)];
        assert_eq!(simplificar(&linea, 0.01), vec![(0.0, 0.0), (2.0, 0.0), (3.0, 1.0), (4.0, 0.0)]);
        assert_eq!(simplificar(&linea, 5.0), vec![(0.0, 0.0), (4.0, 0.0)]);
        let cuadro: Poligono = vec![vec![(0.0, 0.0), (2.0, 0.0001), (4.0, 0.0), (4.0, 4.0), (0.0, 4.0), (0.0, 0.0)]];
        assert_eq!(simplificar_poligonos(&[cuadro], 0.01)[0][0].len(), 5);
    }
}
