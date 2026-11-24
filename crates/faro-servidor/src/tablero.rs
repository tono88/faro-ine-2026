//! Tableros: de una solicitud escrita en lenguaje corriente a una definición de tablero.
//!
//! No interviene ningún modelo: el texto se normaliza y se buscan en él (1) indicadores por sus
//! nombres y sinónimos, (2) un territorio del catálogo oficial, (3) palabras que piden un tipo de
//! pieza (mapa, tabla, barras...) y (4) cantidades ("los 10 con más"). La misma frase produce
//! siempre el mismo tablero, y la respuesta dice qué se entendió y qué no.
use faro_homologa::normalizar::basico;
use faro_homologa::{Catalogo, Nivel};
use serde::{Deserialize, Serialize};

/// Indicadores disponibles: (id, nombre, unidad, sinónimos ya normalizados).
pub const INDICADORES: &[(&str, &str, &str, &[&str])] = &[
    ("poblacion_actual", "Población proyectada (INE)", "hab.", &["poblacion", "habitantes", "personas", "cuanta gente", "demografia"]),
    ("crecimiento_anual_pct", "Crecimiento anual de la población 2018–2030", "%", &["crecimiento poblacional", "crecimiento de la poblacion", "crecimiento demografico", "crece la poblacion", "aumento de poblacion"]),
    ("construido_ha_2020", "Superficie construida 2020 (satélite)", "ha", &["superficie construida", "area construida", "construido", "huella urbana", "mancha urbana", "hectareas construidas", "urbanizacion"]),
    ("construido_crecimiento_anual_pct", "Crecimiento anual de lo construido 2000–2020", "%", &["crecimiento urbano", "crecimiento de lo construido", "expansion urbana", "crecimiento construido"]),
    ("construido_m2_por_habitante_2020", "Metros construidos por habitante 2020", "m²/hab.", &["metros construidos por habitante", "construido por habitante", "metros por habitante"]),
    ("retardo_en_talla_pct", "Desnutrición crónica en escolares", "%", &["desnutricion", "retardo en talla", "desnutricion cronica", "talla"]),
    ("cif_fase3_o_superior_pct_departamento", "Inseguridad alimentaria aguda (dato del departamento)", "%", &["inseguridad alimentaria", "cif", "hambre", "crisis alimentaria"]),
    ("sin_conexion", "Lugares poblados sin conexión vial", "lugares", &["sin conexion", "sin camino", "red vial", "caminos", "carreteras", "aislados", "conexion vial", "incomunicados"]),
    ("lugares_poblados", "Lugares poblados del censo", "lugares", &["lugares poblados", "aldeas", "caserios", "comunidades", "cantones"]),
    ("proveedores_construccion", "Proveedores de construcción perfilados", "proveedores", &["proveedores", "ferreterias", "materiales de construccion"]),
    ("censo_alfabetismo_pct", "Alfabetismo (Censo 2018)", "%", &["alfabetismo", "analfabetismo", "saben leer", "leer y escribir", "educacion"]),
    ("censo_internet_pct", "Personas que usan internet (Censo 2018)", "%", &["internet", "conectividad", "brecha digital", "tecnologia"]),
    ("censo_agua_en_vivienda_pct", "Hogares con agua por tubería en la vivienda (Censo 2018)", "%", &["agua", "agua potable", "agua entubada", "tuberia"]),
    ("censo_drenaje_pct", "Hogares con inodoro conectado a drenaje (Censo 2018)", "%", &["drenaje", "saneamiento", "alcantarillado", "servicio sanitario"]),
    ("censo_electricidad_pct", "Hogares con red eléctrica (Censo 2018)", "%", &["electricidad", "energia electrica", "luz", "alumbrado"]),
    ("censo_lena_pct", "Hogares que cocinan con leña (Censo 2018)", "%", &["lena", "cocinan con lena"]),
    ("censo_techo_lamina_pct", "Viviendas con techo de lámina (Censo 2018)", "%", &["techo de lamina", "lamina", "techos", "techo"]),
    ("censo_techo_concreto_pct", "Viviendas con techo de concreto (Censo 2018)", "%", &["techo de concreto", "terraza", "losa"]),
    ("censo_piso_tierra_pct", "Viviendas con piso de tierra (Censo 2018)", "%", &["piso de tierra", "piso", "vivienda precaria"]),
    ("censo_maya_pct", "Población que se identifica como maya (Censo 2018)", "%", &["maya", "pueblo maya", "pueblos indigenas", "indigena"]),
];

pub const TIPOS: &[&str] = &["tarjeta", "barras", "tabla", "mapa", "piramide", "serie"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Territorio {
    /// "pais", "departamento" o "municipio".
    pub nivel: String,
    #[serde(default)]
    pub codigo: String,
    #[serde(default)]
    pub nombre: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Pieza {
    pub tipo: String,
    #[serde(default)]
    pub indicador: String,
    #[serde(default)]
    pub titulo: String,
    /// Cuántas filas o barras.
    #[serde(default = "diez")]
    pub n: usize,
    /// "desc" (los mayores primero) o "asc".
    #[serde(default = "desc")]
    pub orden: String,
}
fn diez() -> usize { 10 }
fn desc() -> String { "desc".into() }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tablero {
    pub titulo: String,
    pub territorio: Territorio,
    pub piezas: Vec<Pieza>,
}

#[derive(Debug, Serialize)]
pub struct Interpretacion {
    pub tablero: Tablero,
    /// Lo que se reconoció en la frase, dicho en claro.
    pub entendido: Vec<String>,
    /// Avisos: lo que se supuso o no se pudo atender.
    pub avisos: Vec<String>,
}

fn contiene(texto: &str, frase: &str) -> bool {
    format!(" {texto} ").contains(&format!(" {frase} "))
}

/// Convierte una solicitud en lenguaje corriente en la definición de un tablero.
pub fn interpretar(solicitud: &str, catalogo: &Catalogo) -> Interpretacion {
    let texto = basico(solicitud);
    let (mut entendido, mut avisos) = (Vec::new(), Vec::new());

    // 1. Territorio: el nombre más largo del catálogo que aparezca como palabras completas.
    let mut territorio = Territorio { nivel: "pais".into(), codigo: String::new(), nombre: "Guatemala".into() };
    let mut mejor = 0;
    for (nivel, etiqueta) in [(Nivel::Departamento, "departamento"), (Nivel::Municipio, "municipio")] {
        for l in catalogo.de_nivel(nivel, None) {
            let nombre = basico(&l.nombre);
            // "Guatemala" a secas es el país, salvo que se pida "departamento de" o "municipio de".
            if nombre == "guatemala" && !contiene(&texto, &format!("{etiqueta} de guatemala")) {
                continue;
            }
            let pide_este_nivel = contiene(&texto, &format!("{etiqueta} de {nombre}"));
            // A igual nombre gana el departamento, salvo que se diga "municipio de".
            let peso = nombre.len() * 2 + usize::from(pide_este_nivel) * 100 + usize::from(etiqueta == "departamento");
            if nombre.len() >= 4 && contiene(&texto, &nombre) && peso > mejor {
                mejor = peso;
                territorio = Territorio { nivel: etiqueta.into(), codigo: l.codigo.clone(), nombre: l.nombre.clone() };
            }
        }
    }
    entendido.push(match territorio.nivel.as_str() { "pais" => "Territorio: todo el país".to_string(), n => format!("Territorio: {n} de {} (código {})", territorio.nombre, territorio.codigo) });

    // 2. Indicadores, en el orden en que aparecen en la frase.
    let mut hallados: Vec<(usize, &str)> = INDICADORES.iter().filter_map(|(id, _, _, sinonimos)| sinonimos.iter().filter_map(|s| format!(" {texto} ").find(&format!(" {s} "))).min().map(|pos| (pos, *id))).collect();
    // "crecimiento urbano" no debe arrastrar "población" ni "construido" si no se pidieron aparte.
    hallados.sort();
    let mut indicadores: Vec<&str> = hallados.into_iter().map(|(_, id)| id).collect();
    if indicadores.contains(&"crecimiento_anual_pct") && !contiene(&texto, "habitantes") && texto.matches("poblacion").count() <= 1 {
        indicadores.retain(|i| *i != "poblacion_actual");
    }
    if indicadores.contains(&"construido_crecimiento_anual_pct") && texto.matches("construid").count() + texto.matches("urban").count() <= 1 {
        indicadores.retain(|i| *i != "construido_ha_2020");
    }
    if indicadores.is_empty() {
        avisos.push("No reconocí ningún indicador en la frase: puse la población. Puede cambiarlo en cada pieza.".into());
        indicadores.push("poblacion_actual");
    }
    fn nombre_de(id: &str) -> &'static str { INDICADORES.iter().find(|i| i.0 == id).map(|i| i.1).unwrap_or("") }
    entendido.push(format!("Indicadores: {}", indicadores.iter().map(|i| nombre_de(i)).collect::<Vec<_>>().join("; ")));

    // 3. Cantidad y orden.
    let n = texto.split(' ').filter_map(|p| p.parse::<usize>().ok()).find(|x| (2..=50).contains(x)).unwrap_or(10);
    let asc = ["menor", "menores", "menos", "mas bajo", "mas bajos", "mas bajas", "peores", "ultimos"].iter().any(|p| contiene(&texto, p));
    let orden = if asc { "asc" } else { "desc" };

    // 4. Tipos de pieza pedidos.
    let pide = |palabras: &[&str]| palabras.iter().any(|p| contiene(&texto, p));
    let mut tipos: Vec<&str> = Vec::new();
    for (tipo, palabras) in [("tarjeta", &["tarjeta", "tarjetas", "total", "totales", "resumen", "cifras", "cifra", "indicadores clave"][..]), ("mapa", &["mapa", "mapas", "cartografia"][..]),
        ("barras", &["barras", "grafica", "graficas", "grafico", "graficos", "ranking", "comparar", "comparacion", "comparativo", "top", "mayores", "menores", "primeros"][..]),
        ("tabla", &["tabla", "tablas", "lista", "listado", "detalle"][..]), ("piramide", &["piramide", "edades", "edad", "sexo"][..]), ("serie", &["serie", "evolucion", "historico", "historia", "tendencia", "a lo largo"][..])] {
        if pide(palabras) {
            tipos.push(tipo);
        }
    }
    if tipos.is_empty() {
        tipos = vec!["tarjeta", "barras", "mapa"];
        avisos.push("No se pidió un tipo de pieza: puse tarjetas, barras y mapa.".into());
    }
    entendido.push(format!("Piezas: {}{}", tipos.join(", "), if tipos.iter().any(|t| *t == "barras" || *t == "tabla") { format!(" · {n} filas, {} primero", if asc { "los menores" } else { "los mayores" }) } else { String::new() }));

    let mut piezas: Vec<Pieza> = Vec::new();
    let pieza = |tipo: &str, indicador: &str| Pieza { tipo: tipo.into(), indicador: indicador.into(), titulo: String::new(), n, orden: orden.into() };
    if tipos.contains(&"tarjeta") {
        piezas.extend(indicadores.iter().map(|i| pieza("tarjeta", i)));
    }
    for tipo in ["mapa", "barras", "tabla"] {
        if tipos.contains(&tipo) {
            // Un mapa o una gráfica por indicador (hasta tres), para no llenar la pantalla.
            piezas.extend(indicadores.iter().take(3).map(|i| pieza(tipo, i)));
        }
    }
    for tipo in ["piramide", "serie"] {
        if tipos.contains(&tipo) {
            if territorio.nivel == "municipio" {
                piezas.push(pieza(tipo, "poblacion_actual"));
            } else {
                avisos.push(format!("La {} se muestra por municipio: elija un municipio para verla.", if tipo == "piramide" { "pirámide de edades" } else { "serie de población" }));
            }
        }
    }
    if territorio.nivel == "municipio" && piezas.iter().any(|p| matches!(p.tipo.as_str(), "barras" | "tabla" | "mapa")) {
        avisos.push("En un solo municipio las barras, la tabla y el mapa lo comparan con los demás municipios de su departamento.".into());
    }
    let titulo = format!("{} · {}", indicadores.iter().take(2).map(|i| nombre_de(i).split(" (").next().unwrap_or("")).collect::<Vec<_>>().join(" y "), territorio.nombre);
    Interpretacion { tablero: Tablero { titulo, territorio, piezas }, entendido, avisos }
}

/// Revisa una definición que llega del editor: tipos e indicadores conocidos y tamaños razonables.
pub fn validar(t: &Tablero, catalogo: &Catalogo) -> Result<(), String> {
    if t.titulo.trim().is_empty() || t.titulo.chars().count() > 120 {
        return Err("el tablero necesita un título de hasta 120 caracteres".into());
    }
    if t.piezas.is_empty() || t.piezas.len() > 24 {
        return Err("un tablero lleva entre 1 y 24 piezas".into());
    }
    match t.territorio.nivel.as_str() {
        "pais" => {}
        "departamento" | "municipio" => {
            let nivel = if t.territorio.nivel == "departamento" { Nivel::Departamento } else { Nivel::Municipio };
            if catalogo.por_codigo(&t.territorio.codigo).is_none_or(|l| l.nivel != nivel) {
                return Err(format!("el código {} no es un {} del catálogo", t.territorio.codigo, t.territorio.nivel));
            }
        }
        otro => return Err(format!("nivel de territorio no admitido: {otro}")),
    }
    for p in &t.piezas {
        if !TIPOS.contains(&p.tipo.as_str()) {
            return Err(format!("tipo de pieza no admitido: {}", p.tipo));
        }
        if !INDICADORES.iter().any(|i| i.0 == p.indicador) {
            return Err(format!("indicador no conocido: {}", p.indicador));
        }
        if !(1..=50).contains(&p.n) || !matches!(p.orden.as_str(), "asc" | "desc") || p.titulo.chars().count() > 120 {
            return Err("una pieza tiene valores fuera de rango".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalogo() -> Catalogo {
        let mut c = Catalogo::default();
        let texto = "codigo,nombre,nivel,padre\n07,Sololá,departamento,\n0701,Sololá,municipio,07\n0710,Panajachel,municipio,07\n01,Guatemala,departamento,\n0101,Guatemala,municipio,01\n16,Alta Verapaz,departamento,\n";
        for l in csv::Reader::from_reader(texto.as_bytes()).deserialize::<faro_homologa::Lugar>() {
            c.agregar(l.unwrap());
        }
        c
    }

    #[test]
    fn de_una_frase_sale_un_tablero() {
        let c = catalogo();
        let i = interpretar("Quiero un mapa y una tabla de la desnutrición y la población en Sololá, los 5 más altos", &c);
        assert_eq!((i.tablero.territorio.nivel.as_str(), i.tablero.territorio.codigo.as_str()), ("departamento", "07"));
        assert_eq!(i.tablero.piezas.iter().map(|p| (p.tipo.as_str(), p.indicador.as_str(), p.n)).collect::<Vec<_>>(), vec![("mapa", "retardo_en_talla_pct", 5), ("mapa", "poblacion_actual", 5), ("tabla", "retardo_en_talla_pct", 5), ("tabla", "poblacion_actual", 5)]);
        validar(&i.tablero, &c).unwrap();
        // "municipio de" cambia el nivel; la pirámide solo existe por municipio.
        let i = interpretar("pirámide de edades y serie del municipio de Sololá", &c);
        assert_eq!((i.tablero.territorio.nivel.as_str(), i.tablero.territorio.codigo.as_str()), ("municipio", "0701"));
        assert!(i.tablero.piezas.iter().any(|p| p.tipo == "piramide") && i.tablero.piezas.iter().any(|p| p.tipo == "serie"));
    }

    #[test]
    fn lo_que_no_se_entiende_se_dice_y_lo_invalido_se_rechaza() {
        let c = catalogo();
        // Sin indicador ni tipo: valores por omisión y avisos; "Guatemala" a secas es el país.
        let i = interpretar("algo de Guatemala", &c);
        assert_eq!(i.tablero.territorio.nivel, "pais");
        assert_eq!(i.avisos.len(), 2);
        assert_eq!(i.tablero.piezas.iter().map(|p| p.tipo.as_str()).collect::<Vec<_>>(), vec!["tarjeta", "mapa", "barras"]);
        // Orden ascendente y nombres de dos palabras.
        let i = interpretar("ranking de los 3 municipios con menos lugares sin conexión en Alta Verapaz", &c);
        assert_eq!((i.tablero.territorio.codigo.as_str(), i.tablero.piezas[0].orden.as_str(), i.tablero.piezas[0].n), ("16", "asc", 3));
        // "crecimiento urbano" no arrastra la superficie construida.
        let i = interpretar("tabla del crecimiento urbano", &c);
        assert_eq!(i.tablero.piezas.iter().map(|p| p.indicador.as_str()).collect::<Vec<_>>(), vec!["construido_crecimiento_anual_pct"]);
        let mut malo = i.tablero.clone();
        malo.piezas[0].tipo = "guion".into();
        assert!(validar(&malo, &c).is_err());
        malo = i.tablero.clone();
        malo.territorio = Territorio { nivel: "municipio".into(), codigo: "9999".into(), nombre: "x".into() };
        assert!(validar(&malo, &c).is_err());
    }
}
