//! Huella visual de la foto de un producto, para decir si dos productos "se ven iguales".
//!
//! Se calcula en la propia máquina, sin servicios externos ni modelos entrenados:
//! 1. Se recorta el fondo (el color de las esquinas) para quedarse con el empaque.
//! 2. **Forma**: el recorte se lleva a 32×32 en grises, se le aplica la transformada del coseno
//!    y de sus 64 frecuencias más bajas sale un bit por cada una (mayor o no que la mediana).
//!    Dos fotos del mismo empaque difieren en pocos bits (distancia de Hamming).
//! 3. **Color**: el recorte se reduce a una cuadrícula de 4×4 y se guarda el color medio de cada
//!    celda (48 valores entre 0 y 1). Separa el mismo envase con otra etiqueta o sabor.
//!
//! Dos fotos se consideran iguales solo si la forma **y** el color están cerca. Los umbrales se
//! calibran con pares de productos que tienen el mismo código de barras en dos tiendas
//! (`faro fotos calibrar`). Es una ayuda para quien revisa: no sustituye al código de barras.
use image::{imageops::FilterType, DynamicImage, RgbImage};

/// Huella de una foto: forma (64 bits) y color (cuadrícula 4×4 × RGB).
#[derive(Debug, Clone, PartialEq)]
pub struct Huella {
    pub forma: u64,
    pub color: Vec<f32>,
}

/// Umbrales por omisión (ver `docs/13-fotos.md` para la calibración que los respalda).
#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct Umbrales {
    /// Máximo de bits distintos en la forma.
    pub max_bits: u32,
    /// Máxima diferencia media de color (0 a 1).
    pub max_color: f32,
}

impl Default for Umbrales {
    fn default() -> Self {
        Umbrales { max_bits: 10, max_color: 0.08 }
    }
}

/// Diferencia entre dos huellas: (bits de forma distintos, diferencia media de color).
pub fn distancia(a: &Huella, b: &Huella) -> (u32, f32) {
    let color = if a.color.len() == b.color.len() && !a.color.is_empty() { a.color.iter().zip(&b.color).map(|(x, y)| (x - y).abs()).sum::<f32>() / a.color.len() as f32 } else { 1.0 };
    ((a.forma ^ b.forma).count_ones(), color)
}

pub fn se_ven_iguales(a: &Huella, b: &Huella, u: &Umbrales) -> bool {
    let (bits, color) = distancia(a, b);
    bits <= u.max_bits && color <= u.max_color
}

/// Calcula la huella de una imagen (JPEG, PNG o WebP). `None` si no se puede leer.
pub fn huella(bytes: &[u8]) -> Option<Huella> {
    let imagen = image::load_from_memory(bytes).ok()?;
    if imagen.width() < 8 || imagen.height() < 8 {
        return None;
    }
    // Se trabaja sobre una copia pequeña: basta para la huella y es mucho más rápido.
    let chica = aplanar(&imagen.thumbnail(160, 160));
    let recorte = recortar_fondo(&chica);
    let gris = DynamicImage::ImageRgb8(recorte.clone()).resize_exact(32, 32, FilterType::Triangle).to_luma8();
    let pixeles: Vec<f64> = gris.pixels().map(|p| p.0[0] as f64).collect();
    let frecuencias = coseno_2d(&pixeles, 32);
    // Las 8×8 frecuencias más bajas; la primera (brillo medio) no entra en la mediana.
    let bajas: Vec<f64> = (0..8).flat_map(|y| (0..8).map(move |x| (x, y))).map(|(x, y)| frecuencias[y * 32 + x]).collect();
    let mut orden: Vec<f64> = bajas[1..].to_vec();
    orden.sort_by(|a, b| a.total_cmp(b));
    let mediana = orden[orden.len() / 2];
    let forma = bajas.iter().enumerate().fold(0u64, |h, (i, v)| if *v > mediana { h | (1 << i) } else { h });
    let celdas = DynamicImage::ImageRgb8(recorte).resize_exact(4, 4, FilterType::Triangle).to_rgb8();
    Some(Huella { forma, color: celdas.pixels().flat_map(|p| p.0).map(|c| c as f32 / 255.0).collect() })
}

/// Pone la imagen sobre fondo blanco (las fotos con transparencia) y la deja en RGB.
fn aplanar(imagen: &DynamicImage) -> RgbImage {
    let rgba = imagen.to_rgba8();
    RgbImage::from_fn(rgba.width(), rgba.height(), |x, y| {
        let p = rgba.get_pixel(x, y).0;
        let a = p[3] as f32 / 255.0;
        image::Rgb([0, 1, 2].map(|i| (p[i] as f32 * a + 255.0 * (1.0 - a)).round() as u8))
    })
}

/// Recorta el fondo: toma como fondo el color medio de las cuatro esquinas y se queda con el
/// rectángulo que contiene todo lo que se aparta de él. Si no hay nada distinto, deja la imagen.
fn recortar_fondo(imagen: &RgbImage) -> RgbImage {
    let (ancho, alto) = imagen.dimensions();
    let esquinas = [(0, 0), (ancho - 1, 0), (0, alto - 1), (ancho - 1, alto - 1)].map(|(x, y)| imagen.get_pixel(x, y).0);
    let fondo = [0, 1, 2].map(|i| esquinas.iter().map(|e| e[i] as i32).sum::<i32>() / 4);
    // Si las esquinas no se parecen entre sí, la foto no tiene un fondo liso: no se recorta.
    if esquinas.iter().any(|e| (0..3).any(|i| (e[i] as i32 - fondo[i]).abs() > 24)) {
        return imagen.clone();
    }
    let (mut x0, mut y0, mut x1, mut y1) = (ancho, alto, 0, 0);
    for (x, y, p) in imagen.enumerate_pixels() {
        if (0..3).any(|i| (p.0[i] as i32 - fondo[i]).abs() > 24) {
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x);
            y1 = y1.max(y);
        }
    }
    if x1 < x0 + 8 || y1 < y0 + 8 {
        return imagen.clone();
    }
    image::imageops::crop_imm(imagen, x0, y0, x1 - x0 + 1, y1 - y0 + 1).to_image()
}

/// Transformada del coseno (tipo II) en dos dimensiones de una matriz cuadrada de lado `n`.
fn coseno_2d(valores: &[f64], n: usize) -> Vec<f64> {
    let base: Vec<f64> = (0..n * n).map(|i| { let (k, x) = (i / n, i % n); (std::f64::consts::PI * (2 * x + 1) as f64 * k as f64 / (2 * n) as f64).cos() }).collect();
    // Primero por filas y luego por columnas.
    let mut filas = vec![0.0; n * n];
    for y in 0..n {
        for k in 0..n {
            filas[y * n + k] = (0..n).map(|x| valores[y * n + x] * base[k * n + x]).sum();
        }
    }
    let mut salida = vec![0.0; n * n];
    for x in 0..n {
        for k in 0..n {
            salida[k * n + x] = (0..n).map(|y| filas[y * n + x] * base[k * n + y]).sum();
        }
    }
    salida
}

/// Dirección de una foto ya reducida, donde la tienda lo permite (catálogos VTEX: `-250-250`).
pub fn direccion_liviana(url: &str) -> String {
    static IDS: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let ids = IDS.get_or_init(|| regex::Regex::new(r"(/arquivos/ids/\d+)(-\d+-\d+)?/").expect("patrón válido"));
    ids.replace(url, "$1-250-250/").into_owned()
}

/// Descarga una foto (tope de 6 MB) y calcula su huella.
pub fn huella_de_url(agente: &ureq::Agent, url: &str) -> Option<Huella> {
    use std::io::Read;
    if !url.starts_with("https://") {
        return None;
    }
    let mut bytes = Vec::new();
    agente.get(&direccion_liviana(url)).call().ok()?.into_reader().take(6 * 1024 * 1024).read_to_end(&mut bytes).ok()?;
    huella(&bytes)
}

pub fn agente() -> ureq::Agent {
    ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(25)).try_proxy_from_env(true).user_agent("faro-muestreo/0.1 (fotos publicadas; consulta de baja frecuencia)").build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageFormat, Rgb};

    /// Un "empaque" de prueba: rectángulo de un color con una franja, sobre fondo blanco y con margen.
    fn empaque(lado: u32, margen: u32, cuerpo: [u8; 3], franja: [u8; 3], vertical: bool) -> Vec<u8> {
        let img = RgbImage::from_fn(lado, lado, |x, y| {
            if x < margen || y < margen || x >= lado - margen || y >= lado - margen {
                return Rgb([255, 255, 255]);
            }
            let t = if vertical { (x - margen) as f32 / (lado - 2 * margen) as f32 } else { (y - margen) as f32 / (lado - 2 * margen) as f32 };
            Rgb(if (0.3..0.5).contains(&t) { franja } else { cuerpo })
        });
        let mut salida = std::io::Cursor::new(Vec::new());
        img.write_to(&mut salida, ImageFormat::Png).unwrap();
        salida.into_inner()
    }

    #[test]
    fn el_mismo_empaque_con_otro_margen_y_tamano_se_ve_igual() {
        let u = Umbrales::default();
        let a = huella(&empaque(200, 10, [200, 30, 30], [250, 220, 40], false)).unwrap();
        let b = huella(&empaque(320, 60, [200, 30, 30], [250, 220, 40], false)).unwrap();
        let (bits, color) = distancia(&a, &b);
        assert!(bits <= 4 && color < 0.03, "bits {bits}, color {color}");
        assert!(se_ven_iguales(&a, &b, &u));
    }

    #[test]
    fn otro_color_u_otra_forma_no_se_ven_iguales() {
        let u = Umbrales::default();
        let rojo = huella(&empaque(200, 10, [200, 30, 30], [250, 220, 40], false)).unwrap();
        let azul = huella(&empaque(200, 10, [30, 60, 200], [250, 220, 40], false)).unwrap();
        let girado = huella(&empaque(200, 10, [200, 30, 30], [250, 220, 40], true)).unwrap();
        assert!(distancia(&rojo, &azul).1 > u.max_color, "mismo envase con otra etiqueta: lo separa el color");
        assert!(distancia(&rojo, &girado).0 > u.max_bits, "otra disposición: la separa la forma");
        assert!(!se_ven_iguales(&rojo, &azul, &u) && !se_ven_iguales(&rojo, &girado, &u));
    }

    #[test]
    fn rechaza_lo_que_no_es_imagen_y_pide_la_foto_reducida() {
        assert!(huella(b"esto no es una imagen").is_none());
        assert_eq!(direccion_liviana("https://t.example/arquivos/ids/12345/foto.jpg?v=1"), "https://t.example/arquivos/ids/12345-250-250/foto.jpg?v=1");
        assert_eq!(direccion_liviana("https://t.example/arquivos/ids/12345-500-500/foto.jpg"), "https://t.example/arquivos/ids/12345-250-250/foto.jpg");
        assert_eq!(direccion_liviana("https://otra.example/foto.jpg"), "https://otra.example/foto.jpg");
    }
}
