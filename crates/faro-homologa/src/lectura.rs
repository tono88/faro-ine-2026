//! Lectura de envíos en CSV o Excel a una tabla de texto, tal como llegan de las instituciones:
//! con títulos encima del encabezado, varias hojas y fechas en formato de Excel.
use crate::normalizar::basico;
use calamine::{Data, Reader};
use std::path::Path;

#[derive(Debug, Default, Clone)]
pub struct Tabla {
    pub encabezados: Vec<String>,
    pub filas: Vec<Vec<String>>,
    /// Fila (base 1) donde se encontró el encabezado.
    pub fila_encabezado: usize,
    pub hojas: Vec<String>,
}

fn celda(c: &Data) -> String {
    match c {
        Data::Empty => String::new(),
        Data::String(s) => s.trim().to_string(),
        Data::Float(f) if f.fract() == 0.0 && f.abs() < 1e15 => format!("{}", *f as i64),
        Data::Float(f) => f.to_string(),
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(d) => match d.as_datetime() {
            Some(dt) if d.as_f64() < 1.0 => dt.format("%H:%M:%S").to_string(),
            Some(dt) if dt.time() == chrono::NaiveTime::MIN => dt.format("%Y-%m-%d").to_string(),
            Some(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            None => d.to_string(),
        },
        Data::DateTimeIso(s) | Data::DurationIso(s) => s.clone(),
        Data::Error(_) => String::new(),
    }
}

/// Entre las primeras filas, la que más nombres de columna esperados contiene.
/// Así se saltan títulos, totales y encabezados de dos niveles.
pub fn fila_de_encabezado(filas: &[Vec<String>], esperados: &[String]) -> usize {
    let esperados: Vec<String> = esperados.iter().map(|e| basico(e)).collect();
    let mut mejor = (0usize, 0usize);
    for (i, fila) in filas.iter().take(25).enumerate() {
        let n = fila.iter().filter(|c| !c.is_empty() && esperados.contains(&basico(c))).count();
        if n > mejor.1 {
            mejor = (i, n);
        }
    }
    mejor.0
}

fn armar(crudas: Vec<Vec<String>>, esperados: &[String], tabla: &mut Tabla) {
    let h = fila_de_encabezado(&crudas, esperados);
    let Some(encabezados) = crudas.get(h) else { return };
    if tabla.encabezados.is_empty() {
        tabla.encabezados = encabezados.clone();
        tabla.fila_encabezado = h + 1;
    }
    let ancho = tabla.encabezados.len();
    for mut fila in crudas.into_iter().skip(h + 1) {
        if fila.iter().all(|c| c.is_empty()) {
            continue;
        }
        fila.resize(ancho, String::new());
        tabla.filas.push(fila);
    }
}

/// Lee un envío. `hoja`: nombre de la hoja, "todas" para unirlas, o `None` para la primera.
/// `esperados`: nombres y alias de las columnas pedidas (para ubicar el encabezado).
pub fn leer(ruta: &Path, hoja: Option<&str>, esperados: &[String]) -> Result<Tabla, String> {
    let ext = ruta.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let mut tabla = Tabla::default();
    if matches!(ext.as_str(), "xlsx" | "xlsm" | "xls" | "ods") {
        let mut libro = calamine::open_workbook_auto(ruta).map_err(|e| format!("{}: {e}", ruta.display()))?;
        let nombres = libro.sheet_names().to_vec();
        let elegidas: Vec<String> = match hoja {
            Some("todas") => nombres,
            Some(h) => {
                if !nombres.iter().any(|n| n == h) {
                    return Err(format!("{}: no existe la hoja {h:?} (hay: {})", ruta.display(), nombres.join(", ")));
                }
                vec![h.to_string()]
            }
            None => nombres.into_iter().take(1).collect(),
        };
        for nombre in elegidas {
            let rango = libro.worksheet_range(&nombre).map_err(|e| format!("{nombre}: {e}"))?;
            let crudas: Vec<Vec<String>> = rango.rows().map(|f| f.iter().map(celda).collect()).collect();
            armar(crudas, esperados, &mut tabla);
            tabla.hojas.push(nombre);
        }
    } else {
        let bytes = std::fs::read(ruta).map_err(|e| format!("{}: {e}", ruta.display()))?;
        // Archivos de Windows en español suelen venir en Latin-1.
        let texto = String::from_utf8(bytes.clone()).unwrap_or_else(|_| bytes.iter().map(|&b| b as char).collect());
        let texto = texto.trim_start_matches('\u{feff}');
        let muestra: String = texto.lines().take(20).collect::<Vec<_>>().join("\n");
        let sep = [b';', b'\t', b'|'].into_iter().find(|s| muestra.matches(*s as char).count() > muestra.matches(',').count()).unwrap_or(b',');
        let mut lector = csv::ReaderBuilder::new().has_headers(false).flexible(true).delimiter(sep).from_reader(texto.as_bytes());
        let crudas: Vec<Vec<String>> = lector
            .records()
            .map(|r| r.map(|r| r.iter().map(|c| c.trim().to_string()).collect()).map_err(|e| e.to_string()))
            .collect::<Result<_, _>>()?;
        armar(crudas, esperados, &mut tabla);
    }
    if tabla.encabezados.is_empty() {
        return Err(format!("{}: el archivo no tiene filas", ruta.display()));
    }
    Ok(tabla)
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn salta_titulos_y_totales_hasta_el_encabezado_real() {
        // Forma del reporte de emergencias: totales, dos filas de títulos y luego el encabezado.
        let filas = vec![s(&["", "Totales", "", "447"]), s(&["", "Información", "", ""]), s(&["", "", "", ""]), s(&["No.", "Fecha y hora", "Departamento", "Municipio"]), s(&["1", "2025-01-01", "Quiche", "Sacapulas"])];
        assert_eq!(fila_de_encabezado(&filas, &s(&["fecha y hora", "departamento", "municipio"])), 3);
    }

    #[test]
    fn csv_con_punto_y_coma_y_latin1() {
        let ruta = std::env::temp_dir().join(format!("faro-lect-{}.csv", std::process::id()));
        std::fs::write(&ruta, b"Reporte mensual\ndepartamento;municipio\nSolol\xe1;Panajachel\n").unwrap();
        let t = leer(&ruta, None, &s(&["departamento", "municipio"])).unwrap();
        assert_eq!(t.fila_encabezado, 2);
        assert_eq!(t.filas, vec![s(&["Sololá", "Panajachel"])]);
        std::fs::remove_file(ruta).unwrap();
    }
}
