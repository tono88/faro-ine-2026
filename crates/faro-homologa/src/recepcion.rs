//! Recepción de un envío: validar, homologar y producir la **salida estándar**.
//!
//! La salida solo contiene las columnas pedidas en la solicitud, con sus nombres canónicos.
//! Lo que la institución manda de más no pasa; los datos personales se eliminan o se
//! sustituyen por una llave no reversible según lo declare la solicitud.
use crate::homologar::{es_ignorado, Estado, Homologador, Resultado};
use crate::lectura::Tabla;
use crate::solicitud::{fecha_iso, Columna, Reporte, Solicitud, Tipo};
use crate::Nivel;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, Serialize)]
pub struct Pendiente {
    pub columna: String,
    pub nivel: Nivel,
    /// Código del nivel superior con el que se buscó ("" si no había).
    pub contexto: String,
    pub valor: String,
    pub veces: usize,
    pub resultado: Resultado,
}

#[derive(Debug, Default, Serialize)]
pub struct Recepcion {
    pub reporte: Reporte,
    #[serde(skip)]
    pub encabezados: Vec<String>,
    #[serde(skip)]
    pub filas: Vec<Vec<String>>,
    pub pendientes: Vec<Pendiente>,
    /// columna -> estado -> filas.
    pub homologacion: BTreeMap<String, BTreeMap<&'static str, usize>>,
    /// Columnas con catálogo cerrado: columna -> valores que no se reconocieron -> veces.
    pub valores_no_reconocidos: BTreeMap<String, BTreeMap<String, usize>>,
    pub columnas_eliminadas: Vec<String>,
    pub columnas_seudonimizadas: Vec<String>,
}

impl Recepcion {
    /// Porcentaje de valores geográficos no vacíos que quedaron con código.
    pub fn porcentaje_resuelto(&self) -> f64 {
        let (mut ok, mut total) = (0usize, 0usize);
        for estados in self.homologacion.values() {
            for (estado, n) in estados {
                if *estado == "vacio" {
                    continue;
                }
                total += n;
                if matches!(*estado, "exacto" | "memoria") {
                    ok += n;
                }
            }
        }
        if total == 0 { 100.0 } else { 100.0 * ok as f64 / total as f64 }
    }
}

/// Llave no reversible para parear sin exponer el identificador.
pub fn seudonimo(sal: &str, valor: &str) -> String {
    let mut h = Sha256::new();
    h.update(sal.as_bytes());
    h.update([0u8]);
    h.update(valor.trim().as_bytes());
    h.finalize().iter().take(12).map(|b| format!("{b:02x}")).collect()
}

/// `sal`: secreto del INE para las llaves; obligatorio si la solicitud seudonimiza columnas.
pub fn recibir(sol: &Solicitud, tabla: &Tabla, h: &Homologador, sal: Option<&str>) -> Result<Recepcion, String> {
    let (mapa, _) = sol.mapear(&tabla.encabezados);
    let mut rec = Recepcion { reporte: sol.validar(&tabla.encabezados, tabla.filas.iter().cloned()), ..Default::default() };

    let presentes: Vec<(&Columna, usize)> = sol.columnas.iter().filter_map(|c| mapa.get(&c.nombre).map(|&i| (c, i))).collect();
    for (c, _) in &presentes {
        match c.sensible.as_deref() {
            Some("eliminar") => rec.columnas_eliminadas.push(c.nombre.clone()),
            Some("seudonimizar") => {
                if sal.map_or(true, str::is_empty) {
                    return Err(format!("la columna {:?} se seudonimiza: defina el secreto FARO_SAL", c.nombre));
                }
                rec.columnas_seudonimizadas.push(c.nombre.clone());
            }
            Some(otro) => return Err(format!("columna {:?}: sensible = {otro:?} no existe (use eliminar o seudonimizar)", c.nombre)),
            None => {}
        }
    }
    let salen: Vec<(&Columna, usize)> = presentes.iter().copied().filter(|(c, _)| c.sensible.as_deref() != Some("eliminar")).collect();

    // Orden de resolución geográfica: primero las columnas sin contexto.
    let mut geo: Vec<usize> = (0..salen.len()).filter(|&k| salen[k].0.homologar.is_some()).collect();
    geo.sort_by_key(|&k| salen[k].0.contexto.is_some());
    let niveles: HashMap<usize, Nivel> = geo.iter().map(|&k| Ok((k, salen[k].0.homologar.as_deref().unwrap().parse::<Nivel>()?))).collect::<Result<_, String>>()?;

    for (c, _) in &salen {
        rec.encabezados.push(c.nombre.clone());
        if c.homologar.is_some() {
            rec.encabezados.push(format!("{}_codigo", c.nombre));
            rec.encabezados.push(format!("{}_estado", c.nombre));
        }
    }

    let mut cache: HashMap<(usize, String, String), Resultado> = HashMap::new();
    let mut pendientes: BTreeMap<(String, String, String), Pendiente> = BTreeMap::new();

    for fila in &tabla.filas {
        let mut codigos: HashMap<&str, Option<String>> = HashMap::new();
        let mut resultados: HashMap<usize, Resultado> = HashMap::new();
        for &k in &geo {
            let (c, i) = salen[k];
            let valor = fila.get(i).cloned().unwrap_or_default();
            let ctx = c.contexto.as_deref().and_then(|x| codigos.get(x).cloned().flatten());
            let llave = (k, ctx.clone().unwrap_or_default(), valor.clone());
            let r = cache.entry(llave).or_insert_with(|| h.homologar(&valor, niveles[&k], ctx.as_deref())).clone();
            *rec.homologacion.entry(c.nombre.clone()).or_default().entry(r.estado.texto()).or_default() += 1;
            if !r.estado.resuelto() && r.estado != Estado::Vacio {
                pendientes
                    .entry((c.nombre.clone(), ctx.clone().unwrap_or_default(), valor.clone()))
                    .or_insert_with(|| Pendiente { columna: c.nombre.clone(), nivel: niveles[&k], contexto: ctx.unwrap_or_default(), valor, veces: 0, resultado: r.clone() })
                    .veces += 1;
            }
            codigos.insert(&c.nombre, r.codigo.clone());
            resultados.insert(k, r);
        }

        let mut salida = Vec::with_capacity(rec.encabezados.len());
        for (k, (c, i)) in salen.iter().enumerate() {
            let crudo = fila.get(*i).map(|v| v.trim()).unwrap_or("");
            let vacio = es_ignorado(crudo);
            let valor = if c.sensible.as_deref() == Some("seudonimizar") {
                if vacio { String::new() } else { seudonimo(sal.unwrap_or(""), crudo) }
            } else if !c.valores.is_empty() {
                match c.canonico(crudo) {
                    Some(v) => v.to_string(),
                    None => {
                        if !crudo.is_empty() {
                            *rec.valores_no_reconocidos.entry(c.nombre.clone()).or_default().entry(crudo.to_string()).or_default() += 1;
                        }
                        String::new()
                    }
                }
            } else if vacio {
                String::new()
            } else if c.tipo == Tipo::Fecha {
                fecha_iso(crudo).unwrap_or_default()
            } else {
                crudo.to_string()
            };
            salida.push(valor);
            if let Some(r) = resultados.get(&k) {
                salida.push(r.codigo.clone().unwrap_or_default());
                salida.push(r.estado.texto().to_string());
            }
        }
        rec.filas.push(salida);
    }
    rec.pendientes = pendientes.into_values().collect();
    rec.pendientes.sort_by(|a, b| b.veces.cmp(&a.veces));
    Ok(rec)
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::catalogo::{Catalogo, Lugar};
    use crate::Memoria;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn la_salida_es_estandar_y_no_deja_pasar_datos_personales() {
        let mut cat = Catalogo::default();
        for (codigo, nombre, nivel, padre) in [("08", "Totonicapán", Nivel::Departamento, ""), ("0801", "Totonicapán", Nivel::Municipio, "08")] {
            cat.agregar(Lugar { codigo: codigo.into(), nombre: nombre.into(), nivel, padre: padre.into(), alias: String::new(), clave: String::new(), claves: vec![] });
        }
        let sol: Solicitud = toml::from_str(
            r#"
            id = "p"
            titulo = "p"
            institucion = "p"
            [[columna]]
            nombre = "persona"
            tipo = "texto"
            alias = ["CUI"]
            sensible = "seudonimizar"
            [[columna]]
            nombre = "nombre"
            tipo = "texto"
            alias = ["NOMBRE_FALLECIDO"]
            sensible = "eliminar"
            [[columna]]
            nombre = "fecha_hecho"
            tipo = "fecha"
            alias = ["FECHA_DEFUNCION"]
            [[columna]]
            nombre = "departamento"
            tipo = "texto"
            alias = ["DEPTO_DEFUNCION"]
            homologar = "departamento"
            [[columna]]
            nombre = "municipio"
            tipo = "texto"
            alias = ["MUNIC_DEFUNCION"]
            homologar = "municipio"
            contexto = "departamento"
            [[columna]]
            nombre = "sexo"
            tipo = "texto"
            valores = ["hombre", "mujer", "ignorado"]
            equivalencias = { "MASCULINO" = "hombre", "FEMENINO" = "mujer" }
            "#,
        )
        .unwrap();
        let tabla = Tabla {
            encabezados: s(&["CUI", "NOMBRE_FALLECIDO", "FECHA_DEFUNCION", "DEPTO_DEFUNCION", "MUNIC_DEFUNCION", "SEXO", "DIRECCION_MADRE"]),
            filas: vec![s(&["1234567890801", "Persona Ficticia", "28/06/2026", "8", "1", "MASCULINO", "calle x"]), s(&["1234567890801", "Persona Ficticia", "04/07/2026", "43", "171", "INDETERMINADO", ""])],
            ..Default::default()
        };
        let m = Memoria::default();
        let h = Homologador::new(&cat, &m);
        assert!(recibir(&sol, &tabla, &h, None).is_err(), "sin secreto no se seudonimiza");
        let r = recibir(&sol, &tabla, &h, Some("secreto-de-prueba")).unwrap();
        assert_eq!(r.encabezados, s(&["persona", "fecha_hecho", "departamento", "departamento_codigo", "departamento_estado", "municipio", "municipio_codigo", "municipio_estado", "sexo"]));
        let f = &r.filas[0];
        assert_eq!(&f[1..], &s(&["2026-06-28", "8", "08", "exacto", "1", "0801", "exacto", "hombre"])[..]);
        assert_eq!(f[0].len(), 24);
        assert_eq!(f[0], r.filas[1][0], "la misma persona da la misma llave");
        let todo = format!("{:?}", r.filas);
        assert!(!todo.contains("1234567890801") && !todo.contains("Ficticia") && !todo.contains("calle x"));
        assert_eq!(r.filas[1][4], "codigo_invalido");
        assert_eq!(r.valores_no_reconocidos["sexo"]["INDETERMINADO"], 1);
        assert_eq!(r.columnas_eliminadas, vec!["nombre"]);
        assert!((r.porcentaje_resuelto() - 50.0).abs() < 1e-9);
    }
}
