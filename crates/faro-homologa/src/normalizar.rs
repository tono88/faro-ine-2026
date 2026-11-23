//! Normalización de texto para comparar nombres escritos de muchas formas.
use unicode_normalization::UnicodeNormalization;

/// Palabras que no distinguen un lugar de otro.
const VACIAS: &[&str] = &["de", "del", "la", "las", "el", "los", "y"];

/// Abreviaturas frecuentes en registros administrativos.
const ABREVIATURAS: &[(&str, &str)] = &[
    ("sn", "san"),
    ("sta", "santa"),
    ("sto", "santo"),
    ("pto", "puerto"),
    ("nva", "nueva"),
    ("nvo", "nuevo"),
    ("gral", "general"),
    ("cd", "ciudad"),
    ("mpio", "municipio"),
    ("depto", "departamento"),
    ("guate", "guatemala"),
    ("xela", "quetzaltenango"),
];

/// Minúsculas, sin tildes ni signos, espacios colapsados. Conserva la ñ como "n".
pub fn basico(texto: &str) -> String {
    let sin_tildes: String = texto
        .nfd()
        .filter(|c| !('\u{0300}'..='\u{036f}').contains(c))
        .collect();
    let mut salida = String::with_capacity(sin_tildes.len());
    for c in sin_tildes.chars() {
        if c.is_alphanumeric() {
            salida.extend(c.to_lowercase());
        } else {
            salida.push(' ');
        }
    }
    salida.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Clave de comparación: `basico` + abreviaturas expandidas + sin palabras vacías.
///
/// "Puerto de San José", "PTO. SAN JOSE" y "puerto san josé" dan la misma clave.
pub fn clave(texto: &str) -> String {
    let base = basico(texto);
    let fichas: Vec<&str> = base
        .split(' ')
        .map(|f| {
            ABREVIATURAS
                .iter()
                .find(|(abrev, _)| *abrev == f)
                .map(|(_, completa)| *completa)
                .unwrap_or(f)
        })
        .filter(|f| !f.is_empty() && !VACIAS.contains(f))
        .collect();
    if fichas.is_empty() {
        base // "La Libertad" no debe quedar vacía si todo fueran palabras vacías
    } else {
        fichas.join(" ")
    }
}

/// Similitud entre dos claves, de 0 a 1. Toma lo mejor de comparar el texto tal cual
/// y con las palabras ordenadas ("San José, Puerto" contra "Puerto San José").
pub fn similitud(a: &str, b: &str) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    if a == b {
        return 1.0;
    }
    let directa = strsim::jaro_winkler(a, b);
    let ordenar = |s: &str| {
        let mut f: Vec<&str> = s.split(' ').collect();
        f.sort_unstable();
        f.join(" ")
    };
    let (oa, ob) = (ordenar(a), ordenar(b));
    if oa == ob {
        return 0.99;
    }
    directa.max(strsim::jaro_winkler(&oa, &ob)).max(contencion(a, b))
}

/// Si todas las palabras del nombre más corto están en el más largo ("puerto san jose"
/// contiene "san jose"), es un candidato a revisar aunque las letras difieran mucho.
fn contencion(a: &str, b: &str) -> f64 {
    let (fa, fb): (Vec<&str>, Vec<&str>) = (a.split(' ').collect(), b.split(' ').collect());
    let (corto, largo) = if fa.len() <= fb.len() { (&fa, &fb) } else { (&fb, &fa) };
    if corto.iter().all(|f| largo.contains(f)) {
        0.80 + 0.10 * corto.len() as f64 / largo.len() as f64
    } else {
        0.0
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn variantes_de_puerto_san_jose_dan_la_misma_clave() {
        for v in ["Puerto de San José", "PTO. SAN JOSE", "puerto san josé", "  Puerto   San  José "] {
            assert_eq!(clave(v), "puerto san jose", "falló con {v:?}");
        }
    }

    #[test]
    fn conserva_nombres_formados_por_articulos() {
        assert_eq!(clave("La Libertad"), "libertad");
        assert_eq!(clave("El"), "el");
    }

    #[test]
    fn orden_de_palabras_no_importa() {
        assert!(similitud(&clave("San José, Puerto"), &clave("Puerto San José")) > 0.98);
        assert!(similitud("escuintla", "quetzaltenango") < 0.7);
    }

    #[test]
    fn nombre_contenido_en_otro_queda_como_candidato_no_como_igual() {
        let s = similitud(&clave("Puerto de San José"), &clave("San José"));
        assert!((0.80..0.93).contains(&s), "{s}");
    }
}
