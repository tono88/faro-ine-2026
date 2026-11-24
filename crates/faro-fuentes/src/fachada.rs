//! Clasifica la foto de una fachada: ¿parece una vivienda, un comercio u otra cosa?
//!
//! Usa un modelo ya entrenado por terceros, Places365-ResNet18 (MIT CSAIL), que reconoce 365 tipos
//! de escena. Corre dentro del propio servidor, sin servicios externos (`tract`, Rust puro), y la
//! foto no sale de la máquina. FARO no entrenó nada: suma las probabilidades de las escenas que son
//! vivienda y de las que son comercio, y compara.
//!
//! El resultado nunca decide. Sirve para que el cruce de una visita de campo avise al supervisor
//! cuando la foto parece un comercio. Procedencia, licencia y pruebas: `datos/modelos/PROCEDENCIA.md`
//! y `docs/24-clasificador-de-fotos.md`.
use image::imageops::FilterType;
use std::path::Path;
use tract_onnx::prelude::*;

/// Escenas de Places365 que cuentan como vivienda (nombres de `categories_places365.txt` sin el
/// prefijo `/letra/`). Se eligieron leyendo la lista de 365 categorías, antes de probar el modelo.
pub const VIVIENDA: &[&str] = &[
    // edificio o conjunto de vivienda visto desde afuera
    "apartment_building/outdoor", "beach_house", "cabin/outdoor", "chalet", "cottage", "house", "hunting_lodge/outdoor", "manufactured_home", "mansion", "residential_neighborhood", "slum", "tree_house",
    // partes exteriores de una casa
    "balcony/exterior", "driveway", "garage/outdoor", "patio", "porch", "yard",
    // interiores de vivienda
    "alcove", "artists_loft", "attic", "balcony/interior", "basement", "bathroom", "bedchamber", "bedroom", "bow_window/indoor", "childs_room", "closet", "dining_room", "garage/indoor", "home_office", "home_theater", "kitchen", "living_room",
    "nursery", "pantry", "playroom", "shower", "television_room", "utility_room",
];

/// Escenas que cuentan como comercio: locales vistos desde afuera, interiores de tiendas y lugares de comida.
pub const COMERCIO: &[&str] = &[
    "bazaar/outdoor", "beer_garden", "diner/outdoor", "flea_market/indoor", "gas_station", "general_store/outdoor", "hotel/outdoor", "inn/outdoor", "market/outdoor", "motel", "restaurant_patio", "shopfront",
    "amusement_arcade", "auto_showroom", "bakery/shop", "bazaar/indoor", "beauty_salon", "bookstore", "butchers_shop", "candy_store", "clothing_store", "delicatessen", "department_store", "drugstore", "fabric_store", "florist_shop/indoor",
    "general_store/indoor", "gift_shop", "hardware_store", "jewelry_shop", "laundromat", "market/indoor", "pet_shop", "pharmacy", "repair_shop", "shoe_shop", "shopping_mall/indoor", "supermarket", "toyshop",
    "bar", "beer_hall", "bowling_alley", "cafeteria", "coffee_shop", "discotheque", "fastfood_restaurant", "food_court", "ice_cream_parlor", "pizzeria", "pub/indoor", "restaurant", "restaurant_kitchen", "sushi_bar",
];

/// Si vivienda y comercio juntos no llegan a esta probabilidad, la foto es de otra cosa
/// (una calle, una iglesia, un paisaje) y no se dice ni lo uno ni lo otro.
pub const MINIMO_ENTRE_LOS_DOS: f32 = 0.25;
/// Parte de la probabilidad conjunta que debe llevarse el grupo ganador para que el resultado
/// cuente. `decidir` no la aplica: la aplican el cruce de visitas (`campo::CONFIANZA_MINIMA`, en el
/// servidor) y la prueba `faro fachadas`, que por debajo informan la foto como «no concluyente».
pub const PARTE_MINIMA: f32 = 0.65;

const MEDIA: [f32; 3] = [0.485, 0.456, 0.406];
const DESVIACION: [f32; 3] = [0.229, 0.224, 0.225];

/// Lo que el modelo dice de una foto.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Fachada {
    /// "vivienda", "comercio" u "otro".
    pub clase: String,
    /// Para vivienda o comercio: la parte de la probabilidad conjunta que se lleva el ganador.
    /// Para «otro»: la probabilidad de que no sea ninguno de los dos.
    pub confianza: f32,
    pub p_vivienda: f32,
    pub p_comercio: f32,
    /// Las tres escenas más probables, con su probabilidad.
    pub escenas: Vec<(String, f32)>,
}

type Corrida = Box<dyn Fn(Tensor) -> TractResult<Vec<f32>> + Send + Sync>;

/// El modelo cargado y la lista de escenas, listo para clasificar.
pub struct Clasificador {
    correr: Corrida,
    escenas: Vec<String>,
    /// Para cada escena: 1 si es vivienda, 2 si es comercio, 0 si no cuenta.
    grupo: Vec<u8>,
}

impl Clasificador {
    /// Carga `places365_resnet18.onnx` y `categories_places365.txt` de la carpeta dada.
    pub fn cargar(carpeta: &Path) -> Result<Self, String> {
        let lista = carpeta.join("categories_places365.txt");
        let escenas: Vec<String> = std::fs::read_to_string(&lista)
            .map_err(|e| format!("{}: {e}", lista.display()))?
            .lines()
            .filter_map(|l| l.split_whitespace().next())
            // "/a/airfield" -> "airfield", como en el código de los autores.
            .map(|n| n.get(3..).unwrap_or(n).to_string())
            .collect();
        if escenas.len() != 365 {
            return Err(format!("{}: se esperaban 365 escenas y hay {}", lista.display(), escenas.len()));
        }
        // Un nombre mal escrito en los grupos dejaría de contar sin avisar: mejor detenerse.
        if let Some(falta) = VIVIENDA.iter().chain(COMERCIO).find(|n| !escenas.iter().any(|e| e == *n)) {
            return Err(format!("la escena «{falta}» no está en la lista del modelo"));
        }
        let grupo = escenas.iter().map(|e| if VIVIENDA.contains(&e.as_str()) { 1 } else if COMERCIO.contains(&e.as_str()) { 2 } else { 0 }).collect();
        let ruta = carpeta.join("places365_resnet18.onnx");
        let modelo = tract_onnx::onnx().model_for_path(&ruta).and_then(|m| m.into_optimized()).and_then(|m| m.into_runnable()).map_err(|e| format!("{}: {e}", ruta.display()))?;
        let correr: Corrida = Box::new(move |entrada| {
            let salida = modelo.run(tvec!(entrada.into()))?;
            Ok(salida[0].to_plain_array_view::<f32>()?.iter().copied().collect())
        });
        Ok(Clasificador { correr, escenas, grupo })
    }

    /// Nombre del modelo, para mostrarlo en pantalla.
    pub fn descripcion(&self) -> String {
        "Places365-ResNet18 (MIT CSAIL, pesos con licencia CC BY), 365 tipos de escena".to_string()
    }

    /// Clasifica una foto (JPEG, PNG o WebP). La foto se usa en memoria y no se guarda.
    pub fn clasificar(&self, foto: &[u8]) -> Result<Fachada, String> {
        let p = self.probabilidades(foto)?;
        let (mut p_vivienda, mut p_comercio) = (0.0f32, 0.0f32);
        for (i, v) in p.iter().enumerate() {
            match self.grupo[i] {
                1 => p_vivienda += v,
                2 => p_comercio += v,
                _ => {}
            }
        }
        let mut orden: Vec<usize> = (0..p.len()).collect();
        orden.sort_by(|a, b| p[*b].total_cmp(&p[*a]));
        let escenas = orden.iter().take(3).map(|i| (self.escenas[*i].clone(), (p[*i] * 1000.0).round() / 1000.0)).collect();
        let (clase, confianza) = decidir(p_vivienda, p_comercio);
        Ok(Fachada { clase: clase.to_string(), confianza, p_vivienda, p_comercio, escenas })
    }

    /// Probabilidad de cada una de las 365 escenas.
    fn probabilidades(&self, foto: &[u8]) -> Result<Vec<f32>, String> {
        let tensor: Tensor = tract_ndarray::Array4::from_shape_vec((1, 3, 224, 224), preparar(foto)?).map_err(|e| e.to_string())?.into();
        let puntajes = (self.correr)(tensor).map_err(|e| format!("el modelo no pudo evaluar la foto: {e}"))?;
        // Softmax: de puntajes a probabilidades que suman 1.
        let maximo = puntajes.iter().copied().fold(f32::MIN, f32::max);
        let exp: Vec<f32> = puntajes.iter().map(|x| (x - maximo).exp()).collect();
        let suma: f32 = exp.iter().sum();
        Ok(exp.iter().map(|x| x / suma).collect())
    }
}

/// De las dos probabilidades sale la clase y su confianza. Usa `MINIMO_ENTRE_LOS_DOS`; el mínimo
/// de confianza (`PARTE_MINIMA`) lo aplica quien llama.
pub fn decidir(p_vivienda: f32, p_comercio: f32) -> (&'static str, f32) {
    let juntos = p_vivienda + p_comercio;
    if juntos < MINIMO_ENTRE_LOS_DOS {
        ("otro", 1.0 - juntos)
    } else if p_vivienda >= p_comercio {
        ("vivienda", p_vivienda / juntos)
    } else {
        ("comercio", p_comercio / juntos)
    }
}

/// Prepara la foto como la pide el modelo: lado corto a 256 píxeles, recorte central de 224 × 224,
/// colores entre 0 y 1 y normalizados con la media y la desviación con que se entrenó.
/// Devuelve los valores en orden canal, fila, columna.
fn preparar(foto: &[u8]) -> Result<Vec<f32>, String> {
    let mut lector = image::ImageReader::new(std::io::Cursor::new(foto)).with_guessed_format().map_err(|e| e.to_string())?;
    // Tope de tamaño: una imagen enorme comprimida no debe agotar la memoria del servidor.
    let mut limites = image::Limits::default();
    limites.max_image_width = Some(8_000);
    limites.max_image_height = Some(8_000);
    limites.max_alloc = Some(256 * 1024 * 1024);
    lector.limits(limites);
    let imagen = lector.decode().map_err(|e| format!("no se pudo leer la foto: {e}"))?.to_rgb8();
    let (ancho, alto) = imagen.dimensions();
    if ancho < 32 || alto < 32 {
        return Err("la foto es demasiado pequeña para evaluarla".into());
    }
    let (nuevo_ancho, nuevo_alto) = if ancho <= alto { (256, (256.0 * alto as f64 / ancho as f64) as u32) } else { ((256.0 * ancho as f64 / alto as f64) as u32, 256) };
    let chica = image::imageops::resize(&imagen, nuevo_ancho, nuevo_alto, FilterType::Triangle);
    let (izquierda, arriba) = ((nuevo_ancho - 224) / 2, (nuevo_alto - 224) / 2);
    let mut valores = vec![0f32; 3 * 224 * 224];
    for y in 0..224u32 {
        for x in 0..224u32 {
            let pixel = chica.get_pixel(izquierda + x, arriba + y);
            for canal in 0..3 {
                valores[canal * 224 * 224 + (y * 224 + x) as usize] = (pixel[canal] as f32 / 255.0 - MEDIA[canal]) / DESVIACION[canal];
            }
        }
    }
    Ok(valores)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_decision_es_prudente() {
        // Claramente vivienda y claramente comercio.
        assert_eq!(decidir(0.585, 0.007).0, "vivienda");
        assert!(decidir(0.585, 0.007).1 > 0.98);
        assert_eq!(decidir(0.002, 0.975).0, "comercio");
        // Ni lo uno ni lo otro: una iglesia, una calle.
        let (clase, confianza) = decidir(0.084, 0.003);
        assert_eq!(clase, "otro");
        assert!((confianza - 0.913).abs() < 0.001);
        // Reñido: gana comercio por poco, pero su parte no llega al mínimo para contar.
        let (clase, confianza) = decidir(0.374, 0.458);
        assert_eq!(clase, "comercio");
        assert!(confianza < PARTE_MINIMA);
    }

    #[test]
    fn los_grupos_no_se_pisan() {
        assert!(VIVIENDA.iter().all(|v| !COMERCIO.contains(v)));
        assert_eq!((VIVIENDA.len(), COMERCIO.len()), (41, 53));
    }

    #[test]
    fn una_foto_ilegible_se_rechaza_con_mensaje() {
        assert!(preparar(b"esto no es una imagen").is_err());
    }
}
