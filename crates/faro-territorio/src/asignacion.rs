//! Asignación de operadores de campo a puestos de trabajo minimizando la distancia total.
//!
//! Cada operador vive en un punto y cada puesto (un lugar poblado, un sector) necesita cierta
//! cantidad de operadores. Se busca el reparto en el que la suma de distancias casa–puesto es la
//! menor posible. Es el "problema de asignación" y se resuelve de forma exacta con el método
//! húngaro: no es una aproximación, no existe otro reparto con menos kilómetros en total.
//!
//! Cuidado de interpretación: la distancia es en línea recta entre coordenadas (centroides de
//! lugares poblados), no el recorrido por carretera ni el tiempo de viaje.
use serde::Serialize;

/// Tamaño máximo de la tabla operadores × plazas. Por encima conviene repartir por departamento.
pub const MAX_CELDAS: usize = 4_000_000;
const RADIO_TIERRA_KM: f64 = 6371.0088;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coordenada {
    pub longitud: f64,
    pub latitud: f64,
}

/// Distancia en línea recta sobre la esfera (fórmula del semiverseno), en kilómetros.
pub fn distancia_km(a: Coordenada, b: Coordenada) -> f64 {
    let (la, lb) = (a.latitud.to_radians(), b.latitud.to_radians());
    let (dlat, dlon) = (lb - la, (b.longitud - a.longitud).to_radians());
    let h = (dlat / 2.0).sin().powi(2) + la.cos() * lb.cos() * (dlon / 2.0).sin().powi(2);
    2.0 * RADIO_TIERRA_KM * h.sqrt().min(1.0).asin()
}

#[derive(Debug, Clone)]
pub struct Operador {
    pub id: String,
    /// Dónde vive.
    pub donde: Coordenada,
}

#[derive(Debug, Clone)]
pub struct Puesto {
    pub id: String,
    pub donde: Coordenada,
    /// Cuántos operadores necesita.
    pub operadores: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Asignado {
    pub operador: String,
    pub puesto: String,
    pub km: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Resumen {
    pub asignados: usize,
    pub total_km: f64,
    pub promedio_km: f64,
    pub maximo_km: f64,
}

impl Resumen {
    fn de(km: impl Iterator<Item = f64>) -> Self {
        let v: Vec<f64> = km.collect();
        let total: f64 = v.iter().sum();
        Resumen { asignados: v.len(), total_km: total, promedio_km: if v.is_empty() { 0.0 } else { total / v.len() as f64 }, maximo_km: v.iter().copied().fold(0.0, f64::max) }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Asignacion {
    pub asignados: Vec<Asignado>,
    /// Operadores que quedaron sin puesto (había más operadores que plazas).
    pub sin_puesto: Vec<String>,
    /// Puestos con plazas vacías y cuántas (había más plazas que operadores).
    pub sin_cubrir: Vec<(String, usize)>,
    pub resumen: Resumen,
    /// Punto de comparación: repartir en el orden en que vienen las dos listas, sin mirar distancias.
    pub en_orden_de_lista: Resumen,
}

/// Lee un CSV de puntos. Cada fila trae un identificador (columna `id`) y su ubicación: `longitud`
/// y `latitud`, o `lugar` con un código que `donde` convierte en coordenada (el centroide de un
/// lugar poblado). Devuelve además el valor de la columna `extra`, si existe.
///
/// Si alguna fila no se puede ubicar no se asigna nada: se devuelven todas las filas con problema.
fn leer_puntos<R: std::io::Read>(entrada: R, id: &str, extra: &str, donde: &dyn Fn(&str) -> Result<Coordenada, String>) -> Result<Vec<(String, Coordenada, String)>, String> {
    let mut lector = csv::ReaderBuilder::new().trim(csv::Trim::All).flexible(true).from_reader(entrada);
    let cab: Vec<String> = lector.headers().map_err(|e| e.to_string())?.iter().map(|c| c.trim_start_matches('\u{feff}').trim().to_lowercase()).collect();
    let pos = |n: &str| cab.iter().position(|c| c == n);
    let i_id = pos(id).ok_or(format!("falta la columna {id}"))?;
    let (i_lugar, i_lon, i_lat, i_extra) = (pos("lugar"), pos("longitud"), pos("latitud"), pos(extra));
    if i_lugar.is_none() && (i_lon.is_none() || i_lat.is_none()) {
        return Err("falta la ubicación: columna lugar (código de lugar poblado) o longitud y latitud".into());
    }
    let (mut puntos, mut problemas) = (Vec::new(), Vec::new());
    for (n, fila) in lector.records().enumerate() {
        let f = fila.map_err(|e| e.to_string())?;
        let celda = |i: Option<usize>| i.and_then(|i| f.get(i)).unwrap_or("");
        let nombre = celda(Some(i_id)).to_string();
        // Las coordenadas propias mandan; si no vienen, se usa el código de lugar.
        let ubicacion = match (celda(i_lon), celda(i_lat), celda(i_lugar)) {
            ("", "", "") => Err("sin ubicación".to_string()),
            ("", "", codigo) => donde(codigo),
            (lon, lat, _) => match (lon.parse::<f64>(), lat.parse::<f64>()) {
                (Ok(longitud), Ok(latitud)) if (-180.0..=180.0).contains(&longitud) && (-90.0..=90.0).contains(&latitud) => Ok(Coordenada { longitud, latitud }),
                _ => Err(format!("coordenadas no válidas: {lon:?}, {lat:?}")),
            },
        };
        match ubicacion {
            Ok(_) if nombre.is_empty() => problemas.push(format!("fila {}: falta {id}", n + 2)),
            Ok(d) => puntos.push((nombre, d, celda(i_extra).to_string())),
            Err(e) => problemas.push(format!("fila {} ({nombre}): {e}", n + 2)),
        }
    }
    if !problemas.is_empty() {
        let mas = if problemas.len() > 20 { format!("\n  ... y {} más", problemas.len() - 20) } else { String::new() };
        problemas.truncate(20);
        return Err(format!("hay filas sin ubicación utilizable:\n  {}{mas}", problemas.join("\n  ")));
    }
    Ok(puntos)
}

/// CSV de operadores: `operador` y dónde vive (`lugar`, o `longitud` y `latitud`).
pub fn leer_operadores<R: std::io::Read>(entrada: R, donde: &dyn Fn(&str) -> Result<Coordenada, String>) -> Result<Vec<Operador>, String> {
    Ok(leer_puntos(entrada, "operador", "", donde)?.into_iter().map(|(id, donde, _)| Operador { id, donde }).collect())
}

/// CSV de puestos: `puesto`, dónde queda (`lugar`, o `longitud` y `latitud`) y `operadores`
/// (cuántos necesita; 1 si la columna no viene o está vacía).
pub fn leer_puestos<R: std::io::Read>(entrada: R, donde: &dyn Fn(&str) -> Result<Coordenada, String>) -> Result<Vec<Puesto>, String> {
    leer_puntos(entrada, "puesto", "operadores", donde)?
        .into_iter()
        .map(|(id, donde, n)| {
            let operadores = if n.is_empty() { 1 } else { n.parse().map_err(|_| format!("puesto {id}: operadores debe ser un número entero, llegó {n:?}"))? };
            Ok(Puesto { id, donde, operadores })
        })
        .collect()
}

/// Método húngaro para una tabla con no más filas que columnas. Devuelve la columna de cada fila.
fn hungaro(costo: &[Vec<f64>]) -> Vec<usize> {
    let n = costo.len();
    let m = costo.first().map_or(0, Vec::len);
    debug_assert!(n <= m);
    // Índices desde 1; la columna 0 es el punto de partida de cada búsqueda.
    let (mut u, mut v) = (vec![0.0; n + 1], vec![0.0; m + 1]);
    let (mut fila_de, mut camino) = (vec![0usize; m + 1], vec![0usize; m + 1]);
    for i in 1..=n {
        fila_de[0] = i;
        let mut j0 = 0;
        let mut minimo = vec![f64::INFINITY; m + 1];
        let mut usada = vec![false; m + 1];
        loop {
            usada[j0] = true;
            let i0 = fila_de[j0];
            let (mut delta, mut j1) = (f64::INFINITY, 0);
            for j in 1..=m {
                if usada[j] {
                    continue;
                }
                let actual = costo[i0 - 1][j - 1] - u[i0] - v[j];
                if actual < minimo[j] {
                    minimo[j] = actual;
                    camino[j] = j0;
                }
                if minimo[j] < delta {
                    delta = minimo[j];
                    j1 = j;
                }
            }
            for j in 0..=m {
                if usada[j] {
                    u[fila_de[j]] += delta;
                    v[j] -= delta;
                } else {
                    minimo[j] -= delta;
                }
            }
            j0 = j1;
            if fila_de[j0] == 0 {
                break;
            }
        }
        while j0 != 0 {
            let j1 = camino[j0];
            fila_de[j0] = fila_de[j1];
            j0 = j1;
        }
    }
    let mut columna_de = vec![0; n];
    for j in 1..=m {
        if fila_de[j] != 0 {
            columna_de[fila_de[j] - 1] = j - 1;
        }
    }
    columna_de
}

/// Reparte operadores entre puestos con la menor distancia total posible.
pub fn asignar(operadores: &[Operador], puestos: &[Puesto]) -> Result<Asignacion, String> {
    // Una plaza por cada operador que necesita cada puesto.
    let plazas: Vec<usize> = puestos.iter().enumerate().flat_map(|(p, puesto)| std::iter::repeat(p).take(puesto.operadores)).collect();
    if operadores.len().saturating_mul(plazas.len()) > MAX_CELDAS {
        return Err(format!("{} operadores × {} plazas es demasiado para una sola corrida: reparta por departamento", operadores.len(), plazas.len()));
    }
    let km = |o: usize, plaza: usize| distancia_km(operadores[o].donde, puestos[plazas[plaza]].donde);
    let mut asignados = Vec::new();
    let mut plaza_ocupada = vec![false; plazas.len()];
    let mut con_puesto = vec![false; operadores.len()];
    if !operadores.is_empty() && !plazas.is_empty() {
        // El método pide no más filas que columnas: el lado con menos elementos va en las filas.
        let operadores_en_filas = operadores.len() <= plazas.len();
        let pares: Vec<(usize, usize)> = if operadores_en_filas {
            let costo: Vec<Vec<f64>> = (0..operadores.len()).map(|o| (0..plazas.len()).map(|p| km(o, p)).collect()).collect();
            hungaro(&costo).into_iter().enumerate().collect()
        } else {
            let costo: Vec<Vec<f64>> = (0..plazas.len()).map(|p| (0..operadores.len()).map(|o| km(o, p)).collect()).collect();
            hungaro(&costo).into_iter().enumerate().map(|(p, o)| (o, p)).collect()
        };
        for (o, p) in pares {
            con_puesto[o] = true;
            plaza_ocupada[p] = true;
            asignados.push(Asignado { operador: operadores[o].id.clone(), puesto: puestos[plazas[p]].id.clone(), km: km(o, p) });
        }
    }
    asignados.sort_by(|a, b| a.puesto.cmp(&b.puesto).then(a.operador.cmp(&b.operador)));
    let mut vacias = vec![0usize; puestos.len()];
    for (p, ocupada) in plaza_ocupada.iter().enumerate() {
        if !ocupada {
            vacias[plazas[p]] += 1;
        }
    }
    Ok(Asignacion {
        resumen: Resumen::de(asignados.iter().map(|a| a.km)),
        en_orden_de_lista: Resumen::de((0..operadores.len().min(plazas.len())).map(|i| km(i, i))),
        sin_puesto: operadores.iter().zip(&con_puesto).filter(|(_, c)| !**c).map(|(o, _)| o.id.clone()).collect(),
        sin_cubrir: puestos.iter().zip(vacias).filter(|(_, n)| *n > 0).map(|(p, n)| (p.id.clone(), n)).collect(),
        asignados,
    })
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn c(longitud: f64, latitud: f64) -> Coordenada {
        Coordenada { longitud, latitud }
    }
    fn op(id: &str, longitud: f64, latitud: f64) -> Operador {
        Operador { id: id.into(), donde: c(longitud, latitud) }
    }
    fn puesto(id: &str, longitud: f64, latitud: f64, operadores: usize) -> Puesto {
        Puesto { id: id.into(), donde: c(longitud, latitud), operadores }
    }

    #[test]
    fn un_grado_de_latitud_son_unos_111_km() {
        let d = distancia_km(c(-90.5, 14.0), c(-90.5, 15.0));
        assert!((d - 111.195).abs() < 0.01, "{d}");
        assert_eq!(distancia_km(c(-90.5, 14.6), c(-90.5, 14.6)), 0.0);
    }

    #[test]
    fn prefiere_el_total_menor_aunque_alguien_no_quede_en_su_puesto_mas_cercano() {
        // A está a 1 de P1 y a 2 de P2; B está a 2 de P1 y a 10 de P2 (en décimas de grado).
        // Dar a cada quien "su" más cercano manda a B lejísimos; el óptimo cruza: A→P2, B→P1.
        let operadores = [op("A", 0.1, 0.0), op("B", -0.2, 0.0)];
        let puestos = [puesto("P1", 0.0, 0.0, 1), puesto("P2", 0.3, 0.0, 1)];
        let a = asignar(&operadores, &puestos).unwrap();
        let pares: Vec<(&str, &str)> = a.asignados.iter().map(|x| (x.operador.as_str(), x.puesto.as_str())).collect();
        assert_eq!(pares, vec![("B", "P1"), ("A", "P2")]);
        assert!(a.resumen.total_km < a.en_orden_de_lista.total_km);
        assert!(a.sin_puesto.is_empty() && a.sin_cubrir.is_empty());
    }

    #[test]
    fn respeta_cuantos_operadores_pide_cada_puesto_y_dice_que_sobra_o_falta() {
        let operadores = [op("A", 0.0, 0.0), op("B", 0.01, 0.0), op("C", 1.0, 0.0)];
        let sobran = asignar(&operadores, &[puesto("P", 0.0, 0.0, 2)]).unwrap();
        assert_eq!(sobran.asignados.len(), 2);
        assert_eq!(sobran.sin_puesto, vec!["C".to_string()]);
        let faltan = asignar(&operadores, &[puesto("P", 0.0, 0.0, 2), puesto("Q", 1.0, 0.0, 3)]).unwrap();
        assert_eq!(faltan.asignados.len(), 3);
        assert_eq!(faltan.sin_cubrir, vec![("Q".to_string(), 2)]);
        assert!(faltan.sin_puesto.is_empty());
        let nada = asignar(&[], &[puesto("P", 0.0, 0.0, 1)]).unwrap();
        assert_eq!((nada.asignados.len(), nada.sin_cubrir.clone()), (0, vec![("P".to_string(), 1)]));
    }

    #[test]
    fn lee_ubicacion_por_codigo_o_por_coordenadas_y_no_adivina() {
        let donde = |codigo: &str| if codigo == "0701006" { Ok(c(-91.18, 14.77)) } else { Err(format!("el código {codigo} no existe en el catálogo")) };
        let ops = leer_operadores("operador,lugar,longitud,latitud\nA,0701006,,\nB,,-91.2,14.8\n".as_bytes(), &donde).unwrap();
        assert_eq!(ops.iter().map(|o| (o.id.as_str(), o.donde)).collect::<Vec<_>>(), vec![("A", c(-91.18, 14.77)), ("B", c(-91.2, 14.8))]);
        let puestos = leer_puestos("Puesto,Lugar,Operadores\nP,0701006,3\nQ,0701006,\n".as_bytes(), &donde).unwrap();
        assert_eq!(puestos.iter().map(|p| p.operadores).collect::<Vec<_>>(), vec![3, 1]);
        let e = leer_operadores("operador,lugar\nA,0701006\nB,9999999\nC,\n".as_bytes(), &donde).unwrap_err();
        assert!(e.contains("fila 3 (B)") && e.contains("fila 4 (C): sin ubicación"), "{e}");
        assert!(leer_operadores("operador\nA\n".as_bytes(), &donde).is_err());
        assert!(leer_puestos("puesto,lugar,operadores\nP,0701006,muchos\n".as_bytes(), &donde).is_err());
    }

    /// El menor total posible probando todos los repartos (solo sirve para tablas pequeñas).
    fn fuerza_bruta(costo: &[Vec<f64>], fila: usize, usadas: &mut Vec<bool>) -> f64 {
        if fila == costo.len() {
            return 0.0;
        }
        let mut mejor = f64::INFINITY;
        for j in 0..costo[fila].len() {
            if !usadas[j] {
                usadas[j] = true;
                mejor = mejor.min(costo[fila][j] + fuerza_bruta(costo, fila + 1, usadas));
                usadas[j] = false;
            }
        }
        mejor
    }

    #[test]
    fn el_total_es_el_minimo_posible() {
        // Tablas de varios tamaños con números seudoaleatorios fijos, comparadas con probar todo.
        let mut semilla: u64 = 20261123;
        let mut azar = || {
            semilla = semilla.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((semilla >> 33) % 1000) as f64 / 10.0
        };
        for (n, m) in [(1, 1), (2, 5), (4, 4), (5, 7), (6, 6), (7, 7)] {
            for _ in 0..20 {
                let costo: Vec<Vec<f64>> = (0..n).map(|_| (0..m).map(|_| azar()).collect()).collect();
                let columnas = hungaro(&costo);
                let mut distintas = columnas.clone();
                distintas.sort();
                distintas.dedup();
                assert_eq!(distintas.len(), n, "cada fila en una columna distinta");
                let total: f64 = columnas.iter().enumerate().map(|(i, &j)| costo[i][j]).sum();
                let minimo = fuerza_bruta(&costo, 0, &mut vec![false; m]);
                assert!((total - minimo).abs() < 1e-9, "{n}x{m}: {total} en vez de {minimo}");
            }
        }
    }
}
