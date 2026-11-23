//! "Listado de lugares poblados" del Censo 2018 (INE): la referencia oficial de códigos de
//! departamento, municipio y lugar poblado, con el centroide de cada lugar.
//!
//! Sirve para dos cosas: verificar el catálogo código por código y cargar los lugares poblados
//! con sus coordenadas. El archivo se descarga con `faro fuentes descargar censo-2018-lugares-poblados`.
use crate::catalogo::{Catalogo, Nivel};
use crate::normalizar::{basico, clave};
use std::collections::{BTreeMap, BTreeSet};

/// Guatemala cabe holgadamente en este rectángulo; una coordenada fuera de él es un error.
const LONGITUD: (f64, f64) = (-92.5, -88.0);
const LATITUD: (f64, f64) = (13.5, 18.0);

#[derive(Debug, Clone, PartialEq)]
pub struct LugarCenso {
    pub departamento: String,
    pub departamento_nombre: String,
    pub municipio: String,
    pub municipio_nombre: String,
    /// Siete dígitos: municipio (4) + correlativo del lugar (3).
    pub codigo: String,
    pub nombre: String,
    /// Centroide. El listado no lo trae para "Otros lugares poblados".
    pub longitud: Option<f64>,
    pub latitud: Option<f64>,
}

fn numero(celda: &str, ancho: usize) -> Option<String> {
    let t = celda.trim();
    (!t.is_empty() && t.len() <= ancho && t.chars().all(|c| c.is_ascii_digit())).then(|| format!("{t:0>ancho$}"))
}

fn coordenada(celda: &str, (min, max): (f64, f64), que: &str, fila: usize) -> Result<Option<f64>, String> {
    let t = celda.trim();
    if t.is_empty() {
        return Ok(None);
    }
    let v: f64 = t.parse().map_err(|_| format!("fila {fila}: {que} no es un número: {t:?}"))?;
    if !(min..=max).contains(&v) {
        return Err(format!("fila {fila}: {que} {v} queda fuera de Guatemala"));
    }
    Ok(Some(v))
}

/// Interpreta las filas del libro (ver [`crate::lectura::leer_crudo`]). Columnas, en orden:
/// código y nombre de departamento, de municipio y de lugar poblado; longitud y latitud.
///
/// No corrige nada: si un código no cuelga de su nivel superior, se repite o una coordenada
/// queda fuera del país, devuelve el error con el número de fila.
pub fn interpretar(filas: &[Vec<String>]) -> Result<Vec<LugarCenso>, String> {
    let encabezado = filas.iter().take(15).any(|f| f.iter().any(|c| basico(c) == "lugar poblado"));
    if !encabezado {
        return Err("no parece el Listado de lugares poblados: falta el encabezado \"Lugar poblado\"".into());
    }
    let mut lugares = Vec::new();
    let mut vistos = BTreeSet::new();
    for (i, f) in filas.iter().enumerate() {
        let fila = i + 1;
        let celda = |k: usize| f.get(k).map(String::as_str).unwrap_or("");
        // Títulos, encabezados y la nota de fuente no empiezan con un código.
        let (Some(departamento), Some(municipio), Some(codigo)) = (numero(celda(0), 2), numero(celda(2), 4), numero(celda(4), 7)) else {
            continue;
        };
        if !municipio.starts_with(&departamento) || !codigo.starts_with(&municipio) {
            return Err(format!("fila {fila}: el código {codigo} no cuelga de {municipio} / {departamento}"));
        }
        if !vistos.insert(codigo.clone()) {
            return Err(format!("fila {fila}: el código {codigo} está repetido"));
        }
        let nombre = celda(5).trim().to_string();
        if nombre.is_empty() || celda(1).trim().is_empty() || celda(3).trim().is_empty() {
            return Err(format!("fila {fila}: falta un nombre para el código {codigo}"));
        }
        let (longitud, latitud) = (coordenada(celda(6), LONGITUD, "la longitud", fila)?, coordenada(celda(7), LATITUD, "la latitud", fila)?);
        if longitud.is_some() != latitud.is_some() {
            return Err(format!("fila {fila}: el código {codigo} trae solo una de las dos coordenadas"));
        }
        lugares.push(LugarCenso {
            departamento,
            departamento_nombre: celda(1).trim().to_string(),
            municipio,
            municipio_nombre: celda(3).trim().to_string(),
            codigo,
            nombre,
            longitud,
            latitud,
        });
    }
    if lugares.is_empty() {
        return Err("el listado no tiene filas de lugares poblados".into());
    }
    Ok(lugares)
}

/// Un código cuyo nombre en el catálogo no es el del listado.
#[derive(Debug, Clone, PartialEq)]
pub struct Diferencia {
    pub codigo: String,
    pub en_catalogo: String,
    pub en_listado: String,
    /// El nombre del listado ya era un alias del catálogo: el sistema lo reconocía.
    pub cubierto_por_alias: bool,
    /// Solo cambian tildes, mayúsculas o palabras como "de" y "la".
    pub solo_grafia: bool,
}

/// Resultado de comparar un nivel del catálogo con el listado, código por código.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Verificacion {
    pub en_listado: usize,
    pub iguales: usize,
    /// Códigos del listado que el catálogo no tiene (código, nombre).
    pub faltan: Vec<(String, String)>,
    /// Códigos del catálogo que el listado no tiene (código, nombre).
    pub sobran: Vec<(String, String)>,
    pub diferencias: Vec<Diferencia>,
}

impl Verificacion {
    /// Los códigos son los mismos en ambos lados (los nombres pueden diferir).
    pub fn codigos_coinciden(&self) -> bool {
        self.faltan.is_empty() && self.sobran.is_empty()
    }
}

/// Nombres oficiales de un nivel según el listado: código → nombre.
pub fn oficiales(lugares: &[LugarCenso], nivel: Nivel) -> BTreeMap<String, String> {
    lugares
        .iter()
        .map(|l| match nivel {
            Nivel::Departamento => (l.departamento.clone(), l.departamento_nombre.clone()),
            Nivel::Municipio => (l.municipio.clone(), l.municipio_nombre.clone()),
            Nivel::LugarPoblado => (l.codigo.clone(), l.nombre.clone()),
        })
        .collect()
}

pub fn verificar(catalogo: &Catalogo, lugares: &[LugarCenso], nivel: Nivel) -> Verificacion {
    let oficiales = oficiales(lugares, nivel);
    let mut v = Verificacion { en_listado: oficiales.len(), ..Default::default() };
    for (codigo, nombre) in &oficiales {
        match catalogo.por_codigo(codigo).filter(|l| l.nivel == nivel) {
            None => v.faltan.push((codigo.clone(), nombre.clone())),
            Some(l) if l.nombre == *nombre => v.iguales += 1,
            Some(l) => v.diferencias.push(Diferencia {
                codigo: codigo.clone(),
                en_catalogo: l.nombre.clone(),
                en_listado: nombre.clone(),
                cubierto_por_alias: l.claves.contains(&clave(nombre)),
                solo_grafia: l.clave == clave(nombre),
            }),
        }
    }
    v.sobran = catalogo.de_nivel(nivel, None).filter(|l| !oficiales.contains_key(&l.codigo)).map(|l| (l.codigo.clone(), l.nombre.clone())).collect();
    v
}

/// Alias de un lugar al adoptar el nombre oficial: los que ya tenía (menos el que pasó a ser el
/// nombre) más el nombre anterior, si no se compara igual que alguno de los demás.
pub fn alias_al_adoptar(oficial: &str, anterior: &str, alias: &str) -> String {
    let mut lista: Vec<&str> = alias.split('|').map(str::trim).filter(|a| !a.is_empty() && *a != oficial).collect();
    let anterior = anterior.trim();
    let ya_cubierto = std::iter::once(oficial).chain(lista.iter().copied()).any(|n| clave(n) == clave(anterior));
    if !anterior.is_empty() && !ya_cubierto {
        lista.push(anterior);
    }
    lista.join("|")
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::catalogo::Lugar;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    /// Forma del libro real: títulos, encabezado en dos filas, datos y nota de fuente.
    fn libro() -> Vec<Vec<String>> {
        vec![
            s(&["Lugares poblados de la República de Guatemala. Censo 2018"]),
            s(&[]),
            s(&["Código", "Departamento", "Código", "Municipio", "Código", "Lugar poblado", "Centroide del lugar poblado", ""]),
            s(&["", "", "", "", "", "", "Longitud", "Latitud"]),
            s(&["1", "Guatemala", "101", "Guatemala", "101001", "Zona Municipal 1", "-90.512", "14.637021"]),
            s(&["9", "Quetzaltenango", "903", "San Juan Olintepeque", "903002", "La Cumbre", "-91.51", "14.89"]),
            s(&["22", "Jutiapa", "2217", "Quesada", "2217999", "Otros Lugares Poblados", "", ""]),
            s(&["Fuente: INE, XII Censo Nacional de Población y VII de Vivienda - 2018"]),
        ]
    }

    #[test]
    fn lee_codigos_con_ceros_y_coordenadas() {
        let l = interpretar(&libro()).unwrap();
        assert_eq!(l.len(), 3);
        assert_eq!((l[0].departamento.as_str(), l[0].municipio.as_str(), l[0].codigo.as_str()), ("01", "0101", "0101001"));
        assert_eq!((l[0].longitud, l[0].latitud), (Some(-90.512), Some(14.637021)));
        assert_eq!(l[2].codigo, "2217999");
        assert_eq!((l[2].longitud, l[2].latitud), (None, None));
    }

    #[test]
    fn rechaza_lo_que_no_cuadra_en_vez_de_corregirlo() {
        let mut otro = libro();
        otro[5][4] = "904002".into(); // lugar que no cuelga de su municipio
        assert!(interpretar(&otro).unwrap_err().contains("fila 6"));
        let mut fuera = libro();
        fuera[4][7] = "41.6".into(); // latitud fuera del país
        assert!(interpretar(&fuera).unwrap_err().contains("fuera de Guatemala"));
        let mut repetido = libro();
        repetido[5] = repetido[4].clone();
        assert!(interpretar(&repetido).unwrap_err().contains("repetido"));
        assert!(interpretar(&[s(&["1", "Guatemala", "101", "Guatemala", "101001", "Zona 1"])]).is_err());
    }

    #[test]
    fn verifica_codigo_por_codigo() {
        let mut c = Catalogo::default();
        for (codigo, nombre, alias) in [("0101", "Guatemala", ""), ("0903", "Olintepeque", ""), ("2217", "Quesada", ""), ("9999", "Inventado", "")] {
            c.agregar(Lugar { codigo: codigo.into(), nombre: nombre.into(), nivel: Nivel::Municipio, padre: codigo[..2].into(), alias: alias.into(), ..Default::default() });
        }
        let v = verificar(&c, &interpretar(&libro()).unwrap(), Nivel::Municipio);
        assert_eq!((v.en_listado, v.iguales), (3, 2));
        assert_eq!(v.sobran, vec![("9999".to_string(), "Inventado".to_string())]);
        assert!(v.faltan.is_empty() && !v.codigos_coinciden());
        assert_eq!(v.diferencias.len(), 1);
        assert_eq!((v.diferencias[0].en_listado.as_str(), v.diferencias[0].cubierto_por_alias, v.diferencias[0].solo_grafia), ("San Juan Olintepeque", false, false));
    }

    #[test]
    fn el_nombre_anterior_queda_como_alias() {
        assert_eq!(alias_al_adoptar("San Juan Olintepeque", "Olintepeque", ""), "Olintepeque");
        assert_eq!(alias_al_adoptar("San Juan Alotenango", "Alotenango", "San Juan Alotenango"), "Alotenango");
        // Si solo cambia la grafía no hace falta alias: se comparan igual.
        assert_eq!(alias_al_adoptar("Santa Lucía la Reforma", "Santa Lucía La Reforma", ""), "");
        // Los alias que ya estaban se conservan tal cual.
        assert_eq!(alias_al_adoptar("San José", "San José", "Puerto San José|Puerto de San José"), "Puerto San José|Puerto de San José");
    }
}
