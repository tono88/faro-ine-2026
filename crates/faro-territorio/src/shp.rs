//! Lectura mínima de Shapefile (líneas) y de un campo de su tabla dBASE, lo justo para leer la
//! exportación de carreteras de OpenStreetMap. Formato público de ESRI; sin dependencias.
use std::io::Read;

fn leer_exacto<R: Read>(r: &mut R, n: usize) -> Result<Vec<u8>, String> {
    let mut b = vec![0u8; n];
    r.read_exact(&mut b).map_err(|e| format!("archivo truncado: {e}"))?;
    Ok(b)
}
fn i32le(b: &[u8]) -> i32 {
    i32::from_le_bytes([b[0], b[1], b[2], b[3]])
}
fn f64le(b: &[u8]) -> f64 {
    f64::from_le_bytes(b[..8].try_into().expect("8 bytes"))
}

/// Líneas de un `.shp`: por cada registro, sus partes; cada parte es una lista de (longitud, latitud).
/// Los registros sin geometría quedan como lista vacía, para conservar el orden de la tabla.
pub fn leer_lineas<R: Read>(mut shp: R) -> Result<Vec<Vec<Vec<(f64, f64)>>>, String> {
    let cabecera = leer_exacto(&mut shp, 100)?;
    if cabecera[..4] != [0, 0, 0x27, 0x0a] {
        return Err("no es un archivo .shp".into());
    }
    let mut registros = Vec::new();
    loop {
        let mut enc = [0u8; 8];
        match shp.read_exact(&mut enc) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.to_string()),
        }
        let largo = u32::from_be_bytes([enc[4], enc[5], enc[6], enc[7]]) as usize * 2;
        let c = leer_exacto(&mut shp, largo)?;
        let tipo = if c.len() >= 4 { i32le(&c) } else { 0 };
        // 3 = línea; 13 y 23 son líneas con altura o medida (las coordenadas X, Y van igual).
        if !matches!(tipo, 3 | 13 | 23) {
            if tipo != 0 {
                return Err(format!("el registro {} no es una línea (tipo {tipo})", registros.len() + 1));
            }
            registros.push(Vec::new());
            continue;
        }
        if c.len() < 44 {
            return Err(format!("registro {} truncado", registros.len() + 1));
        }
        let (n_partes, n_puntos) = (i32le(&c[36..]) as usize, i32le(&c[40..]) as usize);
        let fin = 44 + 4 * n_partes + 16 * n_puntos;
        if c.len() < fin {
            return Err(format!("registro {} truncado", registros.len() + 1));
        }
        let inicios: Vec<usize> = (0..n_partes).map(|k| i32le(&c[44 + 4 * k..]) as usize).collect();
        let base = 44 + 4 * n_partes;
        let punto = |k: usize| (f64le(&c[base + 16 * k..]), f64le(&c[base + 16 * k + 8..]));
        let partes = (0..n_partes)
            .map(|k| {
                let hasta = inicios.get(k + 1).copied().unwrap_or(n_puntos).min(n_puntos);
                (inicios[k].min(hasta)..hasta).map(punto).collect()
            })
            .collect();
        registros.push(partes);
    }
    Ok(registros)
}

/// Valores de un campo de texto de un `.dbf`, en el orden de los registros.
pub fn leer_campo<R: Read>(mut dbf: R, campo: &str) -> Result<Vec<String>, String> {
    let c = leer_exacto(&mut dbf, 32)?;
    let registros = u32::from_le_bytes([c[4], c[5], c[6], c[7]]) as usize;
    let (largo_cabecera, largo_registro) = (u16::from_le_bytes([c[8], c[9]]) as usize, u16::from_le_bytes([c[10], c[11]]) as usize);
    if largo_cabecera < 33 || largo_registro == 0 {
        return Err("no es un archivo .dbf".into());
    }
    let descriptores = leer_exacto(&mut dbf, largo_cabecera - 32)?;
    let (mut desde, mut ubicacion) = (1usize, None);
    for d in descriptores.chunks(32) {
        if d[0] == 0x0d || d.len() < 32 {
            break;
        }
        let nombre = String::from_utf8_lossy(d[..11].split(|b| *b == 0).next().unwrap_or(&[])).to_string();
        if nombre.eq_ignore_ascii_case(campo) {
            ubicacion = Some((desde, d[16] as usize));
        }
        desde += d[16] as usize;
    }
    let (desde, largo) = ubicacion.ok_or(format!("la tabla no tiene el campo {campo}"))?;
    let mut valores = Vec::with_capacity(registros);
    let mut fila = vec![0u8; largo_registro];
    for _ in 0..registros {
        dbf.read_exact(&mut fila).map_err(|e| format!("tabla truncada: {e}"))?;
        valores.push(String::from_utf8_lossy(&fila[desde..desde + largo]).trim().to_string());
    }
    Ok(valores)
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// Arma un .shp con una línea de dos partes y un registro vacío.
    fn shp_de_prueba() -> Vec<u8> {
        let mut b = vec![0u8; 100];
        b[..4].copy_from_slice(&[0, 0, 0x27, 0x0a]);
        let mut contenido = Vec::new();
        contenido.extend(3i32.to_le_bytes());
        contenido.extend([0u8; 32]);
        contenido.extend(2i32.to_le_bytes());
        contenido.extend(3i32.to_le_bytes());
        contenido.extend(0i32.to_le_bytes());
        contenido.extend(2i32.to_le_bytes());
        for (x, y) in [(-90.5, 14.6), (-90.4, 14.7), (-91.0, 15.0)] {
            contenido.extend(f64::to_le_bytes(x));
            contenido.extend(f64::to_le_bytes(y));
        }
        for (n, c) in [(1u32, contenido), (2u32, 0i32.to_le_bytes().to_vec())] {
            b.extend(n.to_be_bytes());
            b.extend(((c.len() / 2) as u32).to_be_bytes());
            b.extend(c);
        }
        b
    }

    #[test]
    fn lee_lineas_con_partes_y_conserva_registros_vacios() {
        let r = leer_lineas(&shp_de_prueba()[..]).unwrap();
        assert_eq!(r.len(), 2);
        assert_eq!(r[0], vec![vec![(-90.5, 14.6), (-90.4, 14.7)], vec![(-91.0, 15.0)]]);
        assert!(r[1].is_empty());
        assert!(leer_lineas(&b"no es shp"[..]).is_err());
    }

    #[test]
    fn lee_un_campo_de_la_tabla() {
        let mut b = vec![0u8; 32];
        b[4..8].copy_from_slice(&2u32.to_le_bytes());
        b[8..10].copy_from_slice(&(32u16 + 64 + 1).to_le_bytes());
        b[10..12].copy_from_slice(&(1u16 + 3 + 8).to_le_bytes());
        for (nombre, largo) in [("id", 3u8), ("highway", 8u8)] {
            let mut d = [0u8; 32];
            d[..nombre.len()].copy_from_slice(nombre.as_bytes());
            d[11] = b'C';
            d[16] = largo;
            b.extend(d);
        }
        b.push(0x0d);
        b.extend(b" w1 track   ");
        b.extend(b" w2 primary ");
        assert_eq!(leer_campo(&b[..], "highway").unwrap(), vec!["track", "primary"]);
        assert!(leer_campo(&b[..], "surface").is_err());
    }
}
