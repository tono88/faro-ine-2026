//! Población por municipio y año según las "Estimaciones y proyecciones de la población total
//! por municipio según sexo y edad, 2015–2030" del INE (portal de datos abiertos).
//!
//! El libro trae una fila por municipio, sexo y edad simple, y una columna por año. Aquí se
//! resume a una fila por municipio y año (total, hombres, mujeres) y se calcula cuánto crece
//! cada municipio entre dos años. Son proyecciones oficiales: FARO no estima población.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PoblacionAnual {
    /// Código INE del municipio (4 dígitos).
    pub municipio: String,
    pub anio: i32,
    pub total: i64,
    pub hombres: i64,
    pub mujeres: i64,
}

/// Resume las filas del libro (encabezado `DEPTO, MUPIO, SEXO, EDAD, 2015, ..., 2030`).
/// SEXO: 0 = ambos, 1 = hombres, 2 = mujeres. Si hombres + mujeres no da el total publicado
/// para algún municipio y año, devuelve el error en vez de elegir una de las dos cifras.
pub fn resumir(filas: &[Vec<String>]) -> Result<Vec<PoblacionAnual>, String> {
    let pos_enc = filas.iter().take(20).position(|f| f.iter().any(|c| c.trim().eq_ignore_ascii_case("MUPIO"))).ok_or("no parece el libro de proyecciones: falta la columna MUPIO")?;
    let enc: Vec<String> = filas[pos_enc].iter().map(|c| c.trim().to_uppercase()).collect();
    let col = |n: &str| enc.iter().position(|c| c == n).ok_or(format!("falta la columna {n}"));
    let (i_muni, i_sexo) = (col("MUPIO")?, col("SEXO")?);
    let anios: Vec<(usize, i32)> = enc.iter().enumerate().filter_map(|(i, c)| c.parse::<i32>().ok().filter(|a| (1900..=2100).contains(a)).map(|a| (i, a))).collect();
    if anios.is_empty() {
        return Err("el libro no tiene columnas de año".into());
    }
    // (municipio, año) -> [ambos, hombres, mujeres]
    let mut suma: BTreeMap<(String, i32), [i64; 3]> = BTreeMap::new();
    for (n, f) in filas.iter().enumerate().skip(pos_enc + 1) {
        let celda = |i: usize| f.get(i).map(|c| c.trim()).unwrap_or("");
        if celda(i_muni).is_empty() {
            continue;
        }
        let municipio = format!("{:0>4}", celda(i_muni));
        let sexo: usize = celda(i_sexo).parse().ok().filter(|s| *s <= 2).ok_or(format!("fila {}: SEXO no es 0, 1 ni 2: {:?}", n + 1, celda(i_sexo)))?;
        for &(i, anio) in &anios {
            let v: f64 = celda(i).parse().map_err(|_| format!("fila {}: {anio} no es un número: {:?}", n + 1, celda(i)))?;
            suma.entry((municipio.clone(), anio)).or_default()[sexo] += v.round() as i64;
        }
    }
    suma.into_iter()
        .map(|((municipio, anio), [total, hombres, mujeres])| {
            if total != hombres + mujeres {
                return Err(format!("{municipio} en {anio}: hombres ({hombres}) + mujeres ({mujeres}) no da el total publicado ({total})"));
            }
            Ok(PoblacionAnual { municipio, anio, total, hombres, mujeres })
        })
        .collect()
}

/// Códigos de municipio y su nombre según el diccionario que acompaña al libro (hoja de
/// variables: una sección `MUPIO` con filas `código, etiqueta`).
pub fn diccionario_municipios(filas: &[Vec<String>]) -> BTreeMap<String, String> {
    let mut mapa = BTreeMap::new();
    let mut dentro = false;
    for f in filas {
        let celda = |i: usize| f.get(i).map(|c| c.trim()).unwrap_or("");
        match celda(0) {
            "" => {}
            variable => dentro = variable.eq_ignore_ascii_case("MUPIO") && celda(1).chars().all(|c| c.is_ascii_digit()),
        }
        if dentro && !celda(1).is_empty() && celda(1).chars().all(|c| c.is_ascii_digit()) && !celda(2).is_empty() {
            mapa.insert(format!("{:0>4}", celda(1)), celda(2).to_string());
        }
    }
    mapa
}

/// Cambia códigos de municipio según `equivalencias` (código del libro → código INE).
pub fn recodificar(filas: &mut [PoblacionAnual], equivalencias: &BTreeMap<String, String>) {
    for p in filas {
        if let Some(nuevo) = equivalencias.get(&p.municipio) {
            p.municipio = nuevo.clone();
        }
    }
}

/// Un paso del proceso, para mostrarlo a quien quiera repetirlo o auditarlo.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Paso {
    pub titulo: String,
    pub detalle: String,
}

fn paso(pasos: &mut Vec<Paso>, titulo: &str, detalle: String) {
    pasos.push(Paso { titulo: titulo.to_string(), detalle });
}

/// Todo el proceso, del libro publicado a la tabla municipio–año, dejando constancia de cada paso.
///
/// `es_municipio`: ¿el código existe en el catálogo oficial? `ubicar`: dado un código que no
/// existe y el nombre que le da el diccionario, devuelve el código oficial solo si la coincidencia
/// es exacta y única (si no, error: no se adivina).
pub fn procesar(libro: &[Vec<String>], diccionario: Option<&[Vec<String>]>, es_municipio: &dyn Fn(&str) -> bool, ubicar: &dyn Fn(&str, &str) -> Result<String, String>) -> Result<(Vec<PoblacionAnual>, Vec<Paso>), String> {
    let mut pasos = Vec::new();
    paso(&mut pasos, "Leer el libro", format!("{} filas: una por municipio, sexo y edad simple, con una columna por año.", libro.len().saturating_sub(1)));
    let mut filas = resumir(libro)?;
    let municipios: std::collections::BTreeSet<String> = filas.iter().map(|p| p.municipio.clone()).collect();
    let anios: std::collections::BTreeSet<i32> = filas.iter().map(|p| p.anio).collect();
    paso(&mut pasos, "Sumar edades", format!("{} municipios × {} años ({}–{}). En cada municipio y año, hombres + mujeres da exactamente el total publicado.", municipios.len(), anios.len(), anios.first().copied().unwrap_or(0), anios.last().copied().unwrap_or(0)));
    let ajenos: Vec<String> = municipios.iter().filter(|m| !es_municipio(m)).cloned().collect();
    if ajenos.is_empty() {
        paso(&mut pasos, "Cruzar con el catálogo", "Todos los códigos de municipio del libro existen en el catálogo oficial.".into());
    } else {
        paso(&mut pasos, "Cruzar con el catálogo", format!("{} códigos de municipio del libro no existen en el catálogo oficial: {}.", ajenos.len(), ajenos.join(", ")));
        let nombres = diccionario_municipios(diccionario.ok_or(format!("el libro trae {} códigos de municipio que no existen ({}...) y no se dio su diccionario para ubicarlos por nombre", ajenos.len(), ajenos[0]))?);
        let mut equivalencias: BTreeMap<String, String> = BTreeMap::new();
        for codigo in &ajenos {
            let nombre = nombres.get(codigo).ok_or(format!("{codigo}: el diccionario no lo nombra"))?;
            let oficial = ubicar(codigo, nombre)?;
            if municipios.contains(&oficial) || equivalencias.values().any(|v| *v == oficial) {
                return Err(format!("{codigo} ({nombre}) correspondería a {oficial}, que ya tiene datos propios en el libro"));
            }
            equivalencias.insert(codigo.clone(), oficial);
        }
        paso(&mut pasos, "Ubicar por nombre", format!("El diccionario del propio libro nombra cada uno; se ubicó por coincidencia exacta y única dentro de su departamento: {}.", equivalencias.iter().map(|(a, b)| format!("{a} {} → {b}", nombres[a])).collect::<Vec<_>>().join("; ")));
        recodificar(&mut filas, &equivalencias);
        filas.sort_by(|a, b| (&a.municipio, a.anio).cmp(&(&b.municipio, b.anio)));
    }
    let p = Poblacion::new(filas.clone());
    let totales = anios.iter().filter(|a| [anios.first(), anios.last()].contains(&Some(a)) || **a == 2018).filter_map(|a| p.total("", *a).map(|n| format!("{a}: {n}"))).collect::<Vec<_>>().join("; ");
    paso(&mut pasos, "Tabla final", format!("{} filas (municipio, año, total, hombres, mujeres). Población del país — {totales}.", filas.len()));
    Ok((filas, pasos))
}

pub fn leer(ruta: &Path) -> Result<Vec<PoblacionAnual>, String> {
    let mut lector = csv::Reader::from_path(ruta).map_err(|e| format!("{}: {e}", ruta.display()))?;
    lector.deserialize().collect::<Result<_, _>>().map_err(|e| format!("{}: {e}", ruta.display()))
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CambioPoblacion {
    pub codigo: String,
    pub anio_inicial: i32,
    pub anio_final: i32,
    pub poblacion_inicial: i64,
    pub poblacion_final: i64,
    pub cambio: i64,
    pub cambio_pct: f64,
    /// Tasa de crecimiento anual compuesta, en porcentaje.
    pub tasa_anual_pct: f64,
}

/// Población consultable por territorio y año. Un código de 2 dígitos suma su departamento.
#[derive(Debug, Default)]
pub struct Poblacion {
    por_municipio: BTreeMap<(String, i32), PoblacionAnual>,
}

impl Poblacion {
    pub fn new(filas: Vec<PoblacionAnual>) -> Self {
        Poblacion { por_municipio: filas.into_iter().map(|p| ((p.municipio.clone(), p.anio), p)).collect() }
    }

    pub fn is_empty(&self) -> bool {
        self.por_municipio.is_empty()
    }

    pub fn len(&self) -> usize {
        self.por_municipio.len()
    }

    pub fn fila(&self, municipio: &str, anio: i32) -> Option<&PoblacionAnual> {
        self.por_municipio.get(&(municipio.to_string(), anio))
    }

    pub fn anios(&self) -> Option<(i32, i32)> {
        let mut a = self.por_municipio.keys().map(|(_, a)| *a);
        let primero = a.next()?;
        Some(a.fold((primero, primero), |(min, max), x| (min.min(x), max.max(x))))
    }

    /// Población total de un municipio (4 dígitos), un departamento (2) o el país ("").
    pub fn total(&self, codigo: &str, anio: i32) -> Option<i64> {
        if codigo.len() == 4 {
            return self.por_municipio.get(&(codigo.to_string(), anio)).map(|p| p.total);
        }
        let mut n = None;
        for ((m, a), p) in &self.por_municipio {
            if *a == anio && m.starts_with(codigo) {
                *n.get_or_insert(0) += p.total;
            }
        }
        n
    }

    pub fn municipios(&self) -> Vec<String> {
        let mut m: Vec<String> = self.por_municipio.keys().map(|(m, _)| m.clone()).collect();
        m.dedup();
        m
    }

    /// Cambio de población de un territorio entre dos años.
    pub fn cambio(&self, codigo: &str, desde: i32, hasta: i32) -> Option<CambioPoblacion> {
        let (a, b) = (self.total(codigo, desde)?, self.total(codigo, hasta)?);
        if a <= 0 || hasta <= desde {
            return None;
        }
        let razon = b as f64 / a as f64;
        Some(CambioPoblacion {
            codigo: codigo.to_string(),
            anio_inicial: desde,
            anio_final: hasta,
            poblacion_inicial: a,
            poblacion_final: b,
            cambio: b - a,
            cambio_pct: (razon - 1.0) * 100.0,
            tasa_anual_pct: (razon.powf(1.0 / (hasta - desde) as f64) - 1.0) * 100.0,
        })
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    fn libro() -> Vec<Vec<String>> {
        vec![
            s(&["DEPTO", "MUPIO", "SEXO", "EDAD", "2018", "2020"]),
            s(&["01", "0101", "0", "0", "30", "33"]),
            s(&["01", "0101", "0", "1", "70", "88"]),
            s(&["01", "0101", "1", "0", "14", "16"]),
            s(&["01", "0101", "1", "1", "36", "44"]),
            s(&["01", "0101", "2", "0", "16", "17"]),
            s(&["01", "0101", "2", "1", "34", "44"]),
            s(&["1", "102", "0", "0", "10", "10"]),
            s(&["1", "102", "1", "0", "4", "5"]),
            s(&["1", "102", "2", "0", "6", "5"]),
        ]
    }

    #[test]
    fn suma_edades_por_municipio_anio_y_sexo() {
        let p = resumir(&libro()).unwrap();
        assert_eq!(p.len(), 4);
        assert_eq!(p[0], PoblacionAnual { municipio: "0101".into(), anio: 2018, total: 100, hombres: 50, mujeres: 50 });
        assert_eq!(p[3], PoblacionAnual { municipio: "0102".into(), anio: 2020, total: 10, hombres: 5, mujeres: 5 });
    }

    #[test]
    fn si_los_sexos_no_suman_el_total_lo_dice() {
        let mut malo = libro();
        malo[5][4] = "17".into();
        assert!(resumir(&malo).unwrap_err().contains("0101 en 2018"));
        assert!(resumir(&[s(&["a", "b"])]).is_err());
    }

    #[test]
    fn lee_el_diccionario_y_recodifica() {
        let d = diccionario_municipios(&[s(&["SEXO", "0", "Ambos sexos"]), s(&["", "1", "Hombres"]), s(&["MUPIO", "0101", "Guatemala"]), s(&["", "0123", "Santa Catarina Pinula"]), s(&["EDAD", "0", "0 años"])]);
        assert_eq!(d, BTreeMap::from([("0101".to_string(), "Guatemala".to_string()), ("0123".to_string(), "Santa Catarina Pinula".to_string())]));
        let mut p = vec![PoblacionAnual { municipio: "0123".into(), anio: 2018, total: 1, hombres: 1, mujeres: 0 }];
        recodificar(&mut p, &BTreeMap::from([("0123".to_string(), "0102".to_string())]));
        assert_eq!(p[0].municipio, "0102");
    }

    #[test]
    fn el_proceso_completo_deja_constancia_de_cada_paso() {
        let mut l = libro();
        for f in l.iter_mut().skip(7) {
            f[1] = "0123".into(); // el libro trae un código que no existe
        }
        let dic = vec![s(&["MUPIO", "0101", "Guatemala"]), s(&["", "0123", "Santa Catarina Pinula"])];
        let es = |c: &str| c == "0101" || c == "0102";
        let ubicar = |_: &str, nombre: &str| if nombre == "Santa Catarina Pinula" { Ok("0102".to_string()) } else { Err("no coincide".to_string()) };
        let (filas, pasos) = procesar(&l, Some(&dic), &es, &ubicar).unwrap();
        assert_eq!(filas.iter().map(|f| f.municipio.as_str()).collect::<Vec<_>>(), vec!["0101", "0101", "0102", "0102"]);
        assert_eq!(pasos.iter().map(|p| p.titulo.as_str()).collect::<Vec<_>>(), vec!["Leer el libro", "Sumar edades", "Cruzar con el catálogo", "Ubicar por nombre", "Tabla final"]);
        assert!(pasos[3].detalle.contains("0123 Santa Catarina Pinula → 0102"));
        assert!(procesar(&l, None, &es, &ubicar).is_err(), "sin diccionario no se adivina");
        assert_eq!(procesar(&libro(), None, &es, &ubicar).unwrap().1.len(), 4);
    }

    #[test]
    fn cambio_y_tasa_por_municipio_y_por_departamento() {
        let p = Poblacion::new(resumir(&libro()).unwrap());
        assert_eq!(p.anios(), Some((2018, 2020)));
        let c = p.cambio("0101", 2018, 2020).unwrap();
        assert_eq!((c.poblacion_inicial, c.poblacion_final, c.cambio), (100, 121, 21));
        assert!((c.tasa_anual_pct - 10.0).abs() < 1e-9 && (c.cambio_pct - 21.0).abs() < 1e-9);
        assert_eq!(p.total("01", 2020), Some(131));
        assert_eq!(p.total("", 2018), Some(110));
        assert_eq!(p.total("0199", 2018), None);
        assert!(p.cambio("0101", 2020, 2018).is_none());
    }
}
