//! Lector mínimo de GeoTIFF en teselas con compresión LZW y enteros de 16 bits, clásico o BigTIFF:
//! lo que hace falta para los mosaicos de GHSL. Admite teselas "vacías" (sin bytes), que los
//! productores usan para zonas sin dato y que otros lectores rechazan.
use std::collections::HashMap;

/// Una imagen leída: celdas por filas desde la esquina superior izquierda.
pub struct Raster {
    pub ancho: usize,
    pub alto: usize,
    pub celdas: Vec<u16>,
    /// Esquina superior izquierda y tamaño de celda, en las unidades de la proyección.
    pub x0: f64,
    pub y0: f64,
    pub paso: f64,
    pub sin_dato: u16,
}

struct Archivo<'a> {
    b: &'a [u8],
    menor_primero: bool,
    grande: bool,
    /// etiqueta -> (tipo, cantidad, posición del valor o del desplazamiento)
    etiquetas: HashMap<u16, (u16, usize, usize)>,
}

impl<'a> Archivo<'a> {
    fn entero(&self, pos: usize, bytes: usize) -> Result<u64, String> {
        let t = self.b.get(pos..pos + bytes).ok_or("archivo truncado")?;
        let mut v = 0u64;
        for k in 0..bytes {
            let byte = if self.menor_primero { t[bytes - 1 - k] } else { t[k] };
            v = (v << 8) | byte as u64;
        }
        Ok(v)
    }

    fn abrir(b: &'a [u8]) -> Result<Self, String> {
        let menor_primero = match b.get(..2) {
            Some(b"II") => true,
            Some(b"MM") => false,
            _ => return Err("no es un archivo TIFF".into()),
        };
        let mut a = Archivo { b, menor_primero, grande: false, etiquetas: HashMap::new() };
        a.grande = match a.entero(2, 2)? {
            42 => false,
            43 => true,
            _ => return Err("no es un archivo TIFF".into()),
        };
        let (directorio, n, largo) = if a.grande { let d = a.entero(8, 8)? as usize; (d + 8, a.entero(d, 8)? as usize, 20) } else { let d = a.entero(4, 4)? as usize; (d + 2, a.entero(d, 2)? as usize, 12) };
        for k in 0..n {
            let p = directorio + k * largo;
            let (etiqueta, tipo) = (a.entero(p, 2)? as u16, a.entero(p + 2, 2)? as u16);
            let (cantidad, valor) = if a.grande { (a.entero(p + 4, 8)? as usize, p + 12) } else { (a.entero(p + 4, 4)? as usize, p + 8) };
            a.etiquetas.insert(etiqueta, (tipo, cantidad, valor));
        }
        Ok(a)
    }

    /// Posición de los datos de una etiqueta y tamaño de cada elemento.
    fn datos(&self, etiqueta: u16) -> Result<Option<(u16, usize, usize, usize)>, String> {
        let Some(&(tipo, cantidad, valor)) = self.etiquetas.get(&etiqueta) else { return Ok(None) };
        let tam = match tipo {
            1 | 2 | 6 | 7 => 1,
            3 | 8 => 2,
            4 | 9 | 11 => 4,
            5 | 10 | 12 | 16 | 17 | 18 => 8,
            _ => return Err(format!("etiqueta {etiqueta}: tipo {tipo} no admitido")),
        };
        let cabe = if self.grande { 8 } else { 4 };
        let pos = if cantidad * tam <= cabe { valor } else { self.entero(valor, cabe)? as usize };
        Ok(Some((tipo, cantidad, pos, tam)))
    }

    fn enteros(&self, etiqueta: u16) -> Result<Vec<u64>, String> {
        let Some((_, cantidad, pos, tam)) = self.datos(etiqueta)? else { return Ok(vec![]) };
        (0..cantidad).map(|k| self.entero(pos + k * tam, tam)).collect()
    }

    fn uno(&self, etiqueta: u16, nombre: &str) -> Result<usize, String> {
        self.enteros(etiqueta)?.first().map(|v| *v as usize).ok_or(format!("falta {nombre}"))
    }

    fn decimales(&self, etiqueta: u16) -> Result<Vec<f64>, String> {
        let Some((tipo, cantidad, pos, tam)) = self.datos(etiqueta)? else { return Ok(vec![]) };
        if tipo != 12 {
            return Err(format!("etiqueta {etiqueta}: se esperaban decimales"));
        }
        (0..cantidad).map(|k| self.entero(pos + k * tam, 8).map(f64::from_bits)).collect()
    }
}

/// Lee un GeoTIFF completo ya cargado en memoria: una banda de enteros de 16 bits, en teselas, con
/// LZW o sin compresión. Las teselas vacías quedan con el valor «sin dato». Devuelve error si el
/// archivo usa otra variante del formato.
pub fn leer(bytes: &[u8]) -> Result<Raster, String> {
    let a = Archivo::abrir(bytes)?;
    let (ancho, alto) = (a.uno(256, "el ancho")?, a.uno(257, "el alto")?);
    if a.uno(258, "los bits por celda")? != 16 || a.enteros(277)?.first().copied().unwrap_or(1) != 1 {
        return Err("se esperaba una banda de enteros de 16 bits".into());
    }
    let compresion = a.enteros(259)?.first().copied().unwrap_or(1);
    if compresion != 5 && compresion != 1 {
        return Err(format!("compresión {compresion} no admitida (solo LZW o sin compresión)"));
    }
    let predictor = a.enteros(317)?.first().copied().unwrap_or(1);
    if predictor > 2 {
        return Err(format!("predictor {predictor} no admitido"));
    }
    let (ta, tl) = (a.uno(322, "el ancho de tesela (¿archivo en franjas?)")?, a.uno(323, "el alto de tesela")?);
    let (posiciones, largos) = (a.enteros(324)?, a.enteros(325)?);
    let (por_fila, filas) = (ancho.div_ceil(ta), alto.div_ceil(tl));
    if ta == 0 || tl == 0 || posiciones.len() != por_fila * filas || largos.len() != posiciones.len() {
        return Err("tabla de teselas inconsistente".into());
    }
    let (escala, ancla) = (a.decimales(33550)?, a.decimales(33922)?);
    if escala.len() < 2 || ancla.len() < 6 || (escala[0] - escala[1]).abs() > 1e-6 || ancla[0] != 0.0 || ancla[1] != 0.0 {
        return Err("georreferencia ausente o no esperada".into());
    }
    // GDAL anota el valor "sin dato" como texto.
    let sin_dato = match a.datos(42113)? {
        Some((_, cantidad, pos, _)) => String::from_utf8_lossy(bytes.get(pos..pos + cantidad).ok_or("archivo truncado")?).trim_matches(|c: char| c == '\0' || c.is_whitespace()).parse::<f64>().map(|v| v as u16).unwrap_or(u16::MAX),
        None => u16::MAX,
    };
    let mut celdas = vec![sin_dato; ancho * alto];
    let mut tesela = vec![0u8; ta * tl * 2];
    for (k, (&pos, &largo)) in posiciones.iter().zip(&largos).enumerate() {
        if largo == 0 {
            continue; // tesela vacía: queda como sin dato
        }
        let crudo = bytes.get(pos as usize..(pos + largo) as usize).ok_or("tesela fuera del archivo")?;
        if compresion == 5 {
            let mut d = weezl::decode::Configuration::with_tiff_size_switch(weezl::BitOrder::Msb, 8).build();
            let (mut entrada, mut escrito) = (crudo, 0);
            while escrito < tesela.len() {
                let r = d.decode_bytes(entrada, &mut tesela[escrito..]);
                entrada = &entrada[r.consumed_in..];
                escrito += r.consumed_out;
                match r.status {
                    Ok(weezl::LzwStatus::Ok) => {}
                    Ok(_) => break,
                    Err(e) => return Err(format!("tesela {k}: {e}")),
                }
            }
            if escrito < tesela.len() {
                return Err(format!("tesela {k}: datos incompletos"));
            }
        } else {
            let n = tesela.len();
            tesela.copy_from_slice(crudo.get(..n).ok_or(format!("tesela {k}: datos incompletos"))?);
        }
        let (c0, f0) = ((k % por_fila) * ta, (k / por_fila) * tl);
        for f in 0..tl.min(alto - f0) {
            let mut previo = 0u16;
            for c in 0..ta {
                let i = (f * ta + c) * 2;
                let mut v = if a.menor_primero { u16::from_le_bytes([tesela[i], tesela[i + 1]]) } else { u16::from_be_bytes([tesela[i], tesela[i + 1]]) };
                if predictor == 2 {
                    v = v.wrapping_add(previo); // cada celda se guarda como diferencia con la anterior
                    previo = v;
                }
                if c0 + c < ancho {
                    celdas[(f0 + f) * ancho + c0 + c] = v;
                }
            }
        }
    }
    Ok(Raster { ancho, alto, celdas, x0: ancla[3], y0: ancla[4], paso: escala[0], sin_dato })
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// TIFF clásico de 4 x 2 celdas en dos teselas de 2 x 2: la primera con LZW y predictor, la
    /// segunda vacía.
    fn tiff_de_prueba() -> Vec<u8> {
        // Tesela 1 (celdas 100, 103 / 7, 7) como diferencias por fila, en bytes de menor a mayor.
        let diferencias: Vec<u8> = [100u16, 3, 7, 0].iter().flat_map(|v| v.to_le_bytes()).collect();
        let lzw = weezl::encode::Encoder::with_tiff_size_switch(weezl::BitOrder::Msb, 8).encode(&diferencias).unwrap();
        let mut b = vec![b'I', b'I', 42, 0, 0, 0, 0, 0];
        let p_lzw = b.len();
        b.extend(&lzw);
        let p_escala = b.len();
        for v in [100.0f64, 100.0, 0.0] {
            b.extend(v.to_le_bytes());
        }
        let p_ancla = b.len();
        for v in [0.0f64, 0.0, 0.0, -9_041_000.0, 2_000_000.0, 0.0] {
            b.extend(v.to_le_bytes());
        }
        let p_pos = b.len();
        b.extend((p_lzw as u32).to_le_bytes());
        b.extend(0u32.to_le_bytes());
        let p_largos = b.len();
        b.extend((lzw.len() as u32).to_le_bytes());
        b.extend(0u32.to_le_bytes());
        let p_nd = b.len();
        b.extend(b"65535\0");
        let directorio = b.len();
        b[4..8].copy_from_slice(&(directorio as u32).to_le_bytes());
        let entradas: [(u16, u16, u32, u32); 13] = [(256, 3, 1, 4), (257, 3, 1, 2), (258, 3, 1, 16), (259, 3, 1, 5), (277, 3, 1, 1), (317, 3, 1, 2), (322, 3, 1, 2), (323, 3, 1, 2),
            (324, 4, 2, p_pos as u32), (325, 4, 2, p_largos as u32), (33550, 12, 3, p_escala as u32), (33922, 12, 6, p_ancla as u32), (42113, 2, 6, p_nd as u32)];
        b.extend((entradas.len() as u16).to_le_bytes());
        for (etiqueta, tipo, cantidad, valor) in entradas {
            b.extend(etiqueta.to_le_bytes());
            b.extend(tipo.to_le_bytes());
            b.extend(cantidad.to_le_bytes());
            b.extend(valor.to_le_bytes());
        }
        b.extend(0u32.to_le_bytes());
        b
    }

    #[test]
    fn lee_teselas_lzw_con_predictor_y_teselas_vacias() {
        let r = leer(&tiff_de_prueba()).unwrap();
        assert_eq!((r.ancho, r.alto, r.x0, r.y0, r.paso, r.sin_dato), (4, 2, -9_041_000.0, 2_000_000.0, 100.0, 65535));
        assert_eq!(r.celdas, vec![100, 103, 65535, 65535, 7, 7, 65535, 65535]);
        assert!(leer(b"no es tiff").is_err());
    }
}
