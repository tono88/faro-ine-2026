//! Red vial como grafo: qué lugares poblados no tienen un camino mapeado cerca, cuáles quedan en un
//! tramo desconectado de la red principal y por dónde es más probable que falte el enlace.
//!
//! Los caminos (OpenStreetMap) se unen donde comparten un punto. Con eso se forman los
//! **componentes conexos** del grafo: el mayor es la red principal; los demás son tramos sueltos.
//! Un tramo suelto junto a un lugar poblado casi siempre significa que falta dibujar un enlace,
//! no que la gente esté aislada. Para cada tramo suelto se busca el **hueco más corto** hasta la
//! red principal: ese es el enlace probable que conviene verificar en campo o en imagen.
//!
//! Cuidado de interpretación: esto mide el mapa, no el terreno. Las distancias son en línea recta.
use crate::asignacion::{distancia_km, Coordenada};
use serde::Serialize;
use std::collections::HashMap;

/// Lado de la cuadrícula de búsqueda, en grados (≈ 1.1 km).
const CELDA: f64 = 0.01;
/// Cada cuántos metros se agrega un punto intermedio en los tramos largos, para que la distancia
/// a un camino no dependa de cuántos puntos dibujó quien lo mapeó.
const PASO_M: f64 = 50.0;

#[derive(Debug, Clone, Copy)]
pub struct Parametros {
    /// Un lugar "tiene camino" si hay una vía mapeada a esta distancia o menos de su centroide.
    pub cerca_m: f64,
    /// Solo se proponen enlaces de hasta esta longitud.
    pub max_enlace_km: f64,
}

impl Default for Parametros {
    fn default() -> Self {
        Parametros { cerca_m: 500.0, max_enlace_km: 5.0 }
    }
}

struct PuntoIndice {
    donde: Coordenada,
    nodo: u32,
}

/// La red vial como grafo. Cada punto de vía es un nodo; `padre` guarda, para cada nodo, el nodo
/// que identifica a su componente (los nodos unidos entre sí comparten el mismo). `principal` es
/// el componente con más kilómetros de vía.
pub struct Red {
    nodos: Vec<Coordenada>,
    padre: Vec<u32>,
    /// Kilómetros de vía de cada componente (indexado por su nodo raíz).
    km: Vec<f64>,
    indice: HashMap<(i32, i32), Vec<PuntoIndice>>,
    principal: u32,
    /// Líneas leídas con dos puntos o más (vías de OpenStreetMap). No es lo mismo que «tramo suelto»,
    /// que en este módulo es un componente del grafo no unido a la red principal.
    pub tramos: usize,
}

fn celda(c: Coordenada) -> (i32, i32) {
    ((c.longitud / CELDA).floor() as i32, (c.latitud / CELDA).floor() as i32)
}

impl Red {
    /// Arma el grafo a partir de líneas (cada una, una lista de puntos longitud–latitud).
    pub fn construir<'a>(lineas: impl Iterator<Item = &'a [(f64, f64)]>) -> Red {
        let mut ids: HashMap<(i64, i64), u32> = HashMap::new();
        let mut red = Red { nodos: vec![], padre: vec![], km: vec![], indice: HashMap::new(), principal: 0, tramos: 0 };
        let mut segmentos: Vec<(u32, u32, f64)> = Vec::new();
        for linea in lineas {
            let mut previo: Option<u32> = None;
            for &(lon, lat) in linea {
                // Dos vías están unidas si comparten un punto (mismo nodo de OpenStreetMap).
                let llave = ((lon * 1e7).round() as i64, (lat * 1e7).round() as i64);
                let id = *ids.entry(llave).or_insert_with(|| {
                    red.nodos.push(Coordenada { longitud: lon, latitud: lat });
                    (red.nodos.len() - 1) as u32
                });
                if let Some(p) = previo.filter(|p| *p != id) {
                    segmentos.push((p, id, distancia_km(red.nodos[p as usize], red.nodos[id as usize])));
                }
                previo = Some(id);
            }
            if linea.len() > 1 {
                red.tramos += 1;
            }
        }
        red.padre = (0..red.nodos.len() as u32).collect();
        for &(a, b, _) in &segmentos {
            let (ra, rb) = (red.raiz_comprimiendo(a), red.raiz_comprimiendo(b));
            if ra != rb {
                red.padre[ra as usize] = rb;
            }
        }
        for i in 0..red.nodos.len() as u32 {
            red.raiz_comprimiendo(i);
        }
        red.km = vec![0.0; red.nodos.len()];
        for &(a, b, largo) in &segmentos {
            red.km[red.padre[a as usize] as usize] += largo;
            let (pa, pb) = (red.nodos[a as usize], red.nodos[b as usize]);
            let pasos = ((largo * 1000.0 / PASO_M).ceil() as usize).max(1);
            for k in 0..pasos {
                let t = k as f64 / pasos as f64;
                let donde = Coordenada { longitud: pa.longitud + t * (pb.longitud - pa.longitud), latitud: pa.latitud + t * (pb.latitud - pa.latitud) };
                red.indice.entry(celda(donde)).or_default().push(PuntoIndice { donde, nodo: a });
            }
            red.indice.entry(celda(pb)).or_default().push(PuntoIndice { donde: pb, nodo: b });
        }
        red.principal = (0..red.nodos.len() as u32).filter(|i| red.padre[*i as usize] == *i).max_by(|a, b| red.km[*a as usize].total_cmp(&red.km[*b as usize])).unwrap_or(0);
        red
    }

    fn raiz_comprimiendo(&mut self, mut i: u32) -> u32 {
        let mut r = i;
        while self.padre[r as usize] != r {
            r = self.padre[r as usize];
        }
        while self.padre[i as usize] != r {
            let siguiente = self.padre[i as usize];
            self.padre[i as usize] = r;
            i = siguiente;
        }
        r
    }

    fn componente(&self, nodo: u32) -> u32 {
        self.padre[nodo as usize]
    }

    pub fn nodos(&self) -> usize {
        self.nodos.len()
    }

    /// Cantidad de componentes (tramos de red no unidos entre sí).
    pub fn componentes(&self) -> usize {
        (0..self.nodos.len()).filter(|i| self.padre[*i] as usize == *i && self.km[*i] > 0.0).count()
    }

    pub fn km_total(&self) -> f64 {
        self.km.iter().sum()
    }

    pub fn km_principal(&self) -> f64 {
        self.km.get(self.principal as usize).copied().unwrap_or(0.0)
    }

    /// ¿El punto de vía más cercano a `c` (a pocos metros) es de la red principal? `None` si no hay vía ahí.
    pub fn es_principal(&self, c: Coordenada) -> Option<bool> {
        self.mas_cercano(c, 0.03, |_| true).map(|(_, _, comp)| comp == self.principal)
    }

    /// Punto de vía más cercano a `c` dentro de `max_km` que cumpla `acepta` (según su componente).
    fn mas_cercano(&self, c: Coordenada, max_km: f64, acepta: impl Fn(u32) -> bool) -> Option<(f64, Coordenada, u32)> {
        let (cx, cy) = celda(c);
        // Una celda mide ~1.1 km de alto y algo menos de ancho: este radio cubre `max_km` con holgura.
        let radio_max = (max_km / 1.0).ceil() as i32 + 1;
        let mut mejor: Option<(f64, Coordenada, u32)> = None;
        for radio in 0..=radio_max {
            for dx in -radio..=radio {
                for dy in -radio..=radio {
                    if dx.abs() != radio && dy.abs() != radio {
                        continue; // solo el anillo nuevo
                    }
                    for p in self.indice.get(&(cx + dx, cy + dy)).map(Vec::as_slice).unwrap_or(&[]) {
                        let comp = self.componente(p.nodo);
                        if !acepta(comp) {
                            continue;
                        }
                        let d = distancia_km(c, p.donde);
                        if d <= max_km && mejor.as_ref().map_or(true, |m| d < m.0) {
                            mejor = Some((d, p.donde, comp));
                        }
                    }
                }
            }
            // Lo hallado es definitivo cuando los anillos ya revisados cubren esa distancia.
            if mejor.as_ref().is_some_and(|m| m.0 <= radio as f64 * 0.95) {
                break;
            }
        }
        mejor
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Conexion {
    /// Hay una vía cerca y pertenece a la red principal.
    Conectado,
    /// Hay una vía cerca, pero ese tramo no está unido a la red principal en el mapa.
    TramoSuelto,
    /// No hay ninguna vía mapeada cerca del lugar.
    SinCamino,
}

impl Conexion {
    pub fn texto(self) -> &'static str {
        match self {
            Conexion::Conectado => "conectado",
            Conexion::TramoSuelto => "tramo_suelto",
            Conexion::SinCamino => "sin_camino",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LugarVial {
    pub codigo: String,
    pub conexion: Conexion,
    /// Metros a la vía mapeada más cercana (dentro del radio de búsqueda); vacío si no hay ninguna.
    pub distancia_m: Option<f64>,
    /// Punto de esa vía.
    pub via_longitud: Option<f64>,
    pub via_latitud: Option<f64>,
    /// Kilómetros que suma el tramo al que pertenece esa vía.
    pub tramo_km: Option<f64>,
    #[serde(skip)]
    componente: Option<u32>,
}

/// Clasifica cada lugar según la vía más cercana a su centroide.
pub fn clasificar(red: &Red, lugares: &[(String, Coordenada)], p: &Parametros) -> Vec<LugarVial> {
    lugares
        .iter()
        .map(|(codigo, donde)| {
            // Primero una vía de la red principal cerca; si no, cualquier vía hasta el radio de enlace.
            let cerca = p.cerca_m / 1000.0;
            let hallado = red.mas_cercano(*donde, cerca, |c| c == red.principal).or_else(|| red.mas_cercano(*donde, p.max_enlace_km.max(cerca), |_| true));
            let conexion = match &hallado {
                Some((d, _, comp)) if *d <= cerca => {
                    if *comp == red.principal { Conexion::Conectado } else { Conexion::TramoSuelto }
                }
                _ => Conexion::SinCamino,
            };
            LugarVial {
                codigo: codigo.clone(),
                conexion,
                distancia_m: hallado.as_ref().map(|h| h.0 * 1000.0),
                via_longitud: hallado.as_ref().map(|h| h.1.longitud),
                via_latitud: hallado.as_ref().map(|h| h.1.latitud),
                tramo_km: hallado.as_ref().map(|h| red.km[h.2 as usize]),
                componente: hallado.as_ref().map(|h| h.2),
            }
        })
        .collect()
}

/// Un enlace que probablemente falta en el mapa: el hueco más corto entre un tramo suelto (o un
/// lugar sin camino) y la red principal.
#[derive(Debug, Clone, Serialize)]
pub struct Enlace {
    /// Siempre "tramo_suelto": une un tramo desconectado con la red principal. Los lugares sin camino
    /// no generan enlace; su distancia y su punto de vía más cercano van en `LugarVial`.
    pub tipo: &'static str,
    pub desde_longitud: f64,
    pub desde_latitud: f64,
    pub hasta_longitud: f64,
    pub hasta_latitud: f64,
    pub km: f64,
    /// Kilómetros de vía que quedarían unidos a la red.
    pub tramo_km: f64,
    /// Lugares poblados que dependen de este enlace.
    pub lugares: Vec<String>,
}

/// Enlaces probables, del que más lugares conecta por kilómetro al que menos.
pub fn enlaces_probables(red: &Red, clasificados: &[LugarVial], p: &Parametros) -> Vec<Enlace> {
    let mut por_componente: HashMap<u32, Vec<String>> = HashMap::new();
    let mut enlaces = Vec::new();
    for l in clasificados {
        match (l.conexion, l.componente) {
            (Conexion::TramoSuelto, Some(c)) => por_componente.entry(c).or_default().push(l.codigo.clone()),
            // Lugar sin vía cerca: el enlace va del punto de vía más cercano hacia el lugar. Se anota
            // en la tabla de lugares (distancia y punto); aquí no hay hueco entre tramos que buscar.
            _ => {}
        }
    }
    // Nodos de cada tramo suelto que tiene lugares.
    let mut nodos_de: HashMap<u32, Vec<u32>> = HashMap::new();
    for i in 0..red.nodos.len() as u32 {
        let c = red.componente(i);
        if por_componente.contains_key(&c) {
            nodos_de.entry(c).or_default().push(i);
        }
    }
    for (comp, lugares) in por_componente {
        let mut mejor: Option<(f64, Coordenada, Coordenada)> = None;
        for &n in nodos_de.get(&comp).map(Vec::as_slice).unwrap_or(&[]) {
            let desde = red.nodos[n as usize];
            let tope = mejor.as_ref().map_or(p.max_enlace_km, |m| m.0);
            if let Some((d, hasta, _)) = red.mas_cercano(desde, tope, |c| c == red.principal) {
                if mejor.as_ref().map_or(true, |m| d < m.0) {
                    mejor = Some((d, desde, hasta));
                }
            }
        }
        if let Some((km, desde, hasta)) = mejor {
            let mut lugares = lugares;
            lugares.sort();
            enlaces.push(Enlace { tipo: "tramo_suelto", desde_longitud: desde.longitud, desde_latitud: desde.latitud, hasta_longitud: hasta.longitud, hasta_latitud: hasta.latitud, km, tramo_km: red.km[comp as usize], lugares });
        }
    }
    enlaces.sort_by(|a, b| {
        let valor = |e: &Enlace| e.lugares.len() as f64 / e.km.max(0.01);
        valor(b).total_cmp(&valor(a)).then(a.lugares.cmp(&b.lugares))
    });
    enlaces
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn c(longitud: f64, latitud: f64) -> Coordenada {
        Coordenada { longitud, latitud }
    }

    /// Red principal: una carretera larga de oeste a este con un ramal que comparte un punto.
    /// Tramo suelto: una calle corta al norte, a ~330 m de la carretera, sin unión dibujada.
    fn lineas() -> Vec<Vec<(f64, f64)>> {
        vec![
            vec![(-91.00, 15.000), (-90.95, 15.000), (-90.90, 15.000)],
            vec![(-90.95, 15.000), (-90.95, 14.980)],
            vec![(-90.925, 15.003), (-90.925, 15.006)],
        ]
    }

    fn red() -> Red {
        let l = lineas();
        Red::construir(l.iter().map(Vec::as_slice))
    }

    #[test]
    fn las_vias_que_comparten_un_punto_forman_un_solo_tramo() {
        let r = red();
        assert_eq!((r.tramos, r.componentes()), (3, 2));
        assert!(r.km_principal() > 12.0 && r.km_total() - r.km_principal() < 0.5);
    }

    #[test]
    fn clasifica_conectado_tramo_suelto_y_sin_camino() {
        let r = red();
        let lugares = vec![
            ("junto a la carretera".to_string(), c(-90.97, 15.001)),
            ("a mitad de un tramo largo".to_string(), c(-90.975, 15.0005)),
            ("en la calle suelta".to_string(), c(-90.925, 15.0065)),
            ("lejos de todo".to_string(), c(-90.80, 15.30)),
            ("a 2 km de la carretera".to_string(), c(-90.97, 15.018)),
        ];
        let k = clasificar(&r, &lugares, &Parametros::default());
        assert_eq!(k.iter().map(|l| l.conexion).collect::<Vec<_>>(), vec![Conexion::Conectado, Conexion::Conectado, Conexion::TramoSuelto, Conexion::SinCamino, Conexion::SinCamino]);
        assert!(k[1].distancia_m.unwrap() < 80.0, "la distancia es a la vía, no a sus extremos: {:?}", k[1].distancia_m);
        assert_eq!(k[3].distancia_m, None);
        assert!((k[4].distancia_m.unwrap() - 2000.0).abs() < 60.0, "{:?}", k[4].distancia_m);
    }

    #[test]
    fn propone_el_hueco_mas_corto_hasta_la_red_principal() {
        let r = red();
        let lugares = vec![("en la calle suelta".to_string(), c(-90.925, 15.0065)), ("otro en la misma calle".to_string(), c(-90.925, 15.0058))];
        let p = Parametros::default();
        let e = enlaces_probables(&r, &clasificar(&r, &lugares, &p), &p);
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].lugares.len(), 2);
        assert!((e[0].km - 0.333).abs() < 0.03, "{}", e[0].km);
        assert_eq!((e[0].desde_latitud, e[0].hasta_latitud), (15.003, 15.0), "del extremo sur de la calle al punto de enfrente en la carretera");
        // Con un tope menor que el hueco, no se propone nada.
        assert!(enlaces_probables(&r, &clasificar(&r, &lugares, &p), &Parametros { max_enlace_km: 0.2, ..p }).is_empty());
    }
}
