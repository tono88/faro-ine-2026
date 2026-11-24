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

/// Un valor de catálogo cerrado (sexo, pueblo...) que no se reconoció y espera decisión.
#[derive(Debug, Clone, Serialize)]
pub struct PendienteValor {
    pub columna: String,
    pub dimension: String,
    pub valor: String,
    pub veces: usize,
    /// Valores permitidos entre los que elige la persona.
    pub opciones: Vec<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct Recepcion {
    pub reporte: Reporte,
    #[serde(skip)]
    pub encabezados: Vec<String>,
    #[serde(skip)]
    pub filas: Vec<Vec<String>>,
    pub pendientes: Vec<Pendiente>,
    /// Valores de catálogos cerrados por decidir, del más al menos frecuente.
    pub pendientes_valor: Vec<PendienteValor>,
    /// Valores resueltos con una equivalencia confirmada antes: columna -> filas.
    pub valores_por_memoria: BTreeMap<String, usize>,
    /// Lugares resueltos con la columna de respaldo (ej. el nombre cuando el código no existe): columna -> filas.
    pub resueltos_por_respaldo: BTreeMap<String, usize>,
    /// Lugares de otro país según la columna de país de la solicitud: columna -> filas. No se homologan.
    pub en_el_extranjero: BTreeMap<String, usize>,
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

/// Recibe un envío: valida la tabla contra la solicitud, homologa las columnas geográficas, lleva
/// los valores de catálogo cerrado a su forma canónica y arma la salida estándar. Elimina o
/// seudonimiza las columnas sensibles y junta lo que queda pendiente de decisión.
/// `sal`: secreto del INE para las llaves; obligatorio si la solicitud seudonimiza columnas.
pub fn recibir(sol: &Solicitud, tabla: &Tabla, h: &Homologador, sal: Option<&str>) -> Result<Recepcion, String> {
    let (mapa, _) = sol.mapear(&tabla.encabezados);
    let aprendido = |c: &Columna, v: &str| h.memoria.buscar_valor(c.dimension(), v).is_some_and(|d| c.canonico(d).is_some());
    let mut rec = Recepcion { reporte: sol.validar_con(&tabla.encabezados, tabla.filas.iter().cloned(), &aprendido), ..Default::default() };
    // Columnas de país y de respaldo: de cada lista, el índice en el archivo de la primera que exista.
    let normal: Vec<String> = tabla.encabezados.iter().map(|e| crate::normalizar::basico(e)).collect();
    let pais_de = |c: &Columna| c.pais.iter().find_map(|r| normal.iter().position(|e| *e == crate::normalizar::basico(r)));
    let respaldo_de = |c: &Columna| c.respaldo.iter().find_map(|r| normal.iter().position(|e| *e == crate::normalizar::basico(r)));

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
    let mut pendientes_valor: BTreeMap<(String, String), PendienteValor> = BTreeMap::new();

    for fila in &tabla.filas {
        let mut codigos: HashMap<&str, Option<String>> = HashMap::new();
        let mut resultados: HashMap<usize, Resultado> = HashMap::new();
        for &k in &geo {
            let (c, i) = salen[k];
            let mut valor = fila.get(i).cloned().unwrap_or_default();
            if c.es_sin_dato(&valor) {
                valor.clear();
            }
            // Si la fila dice que el lugar está en otro país, su "departamento" y "municipio" son
            // divisiones de ese país: no se buscan en el catálogo nacional ni quedan pendientes.
            if let Some(pais) = pais_de(c).and_then(|j| fila.get(j)).map(|p| p.trim().trim_end_matches(".0")).filter(|p| !p.is_empty() && !es_ignorado(p) && !c.es_sin_dato(p)) {
                if pais != c.pais_local && !valor.trim().is_empty() {
                    *rec.en_el_extranjero.entry(c.nombre.clone()).or_default() += 1;
                    valor.clear();
                }
            }
            let ctx = c.contexto.as_deref().and_then(|x| codigos.get(x).cloned().flatten());
            let llave = (k, ctx.clone().unwrap_or_default(), valor.clone());
            let mut r = cache.entry(llave).or_insert_with(|| h.homologar(&valor, niveles[&k], ctx.as_deref())).clone();
            // Si la columna principal no se resolvió, se prueba la de respaldo (mismo criterio: exacto o memoria).
            if !r.estado.resuelto() && r.estado != Estado::Vacio {
                if let Some(otro) = respaldo_de(c).and_then(|j| fila.get(j)).filter(|v| !v.trim().is_empty()) {
                    let llave = (k, ctx.clone().unwrap_or_default(), format!("\u{1}{otro}"));
                    let r2 = cache.entry(llave).or_insert_with(|| h.homologar(otro, niveles[&k], ctx.as_deref())).clone();
                    if r2.estado.resuelto() {
                        *rec.resueltos_por_respaldo.entry(c.nombre.clone()).or_default() += 1;
                        r = r2;
                    }
                }
            }
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
            let vacio = es_ignorado(crudo) || c.es_sin_dato(crudo);
            let valor = if c.sensible.as_deref() == Some("seudonimizar") {
                if vacio { String::new() } else { seudonimo(sal.unwrap_or(""), crudo) }
            } else if !c.valores.is_empty() {
                // Catálogo de la solicitud; si no, lo que una persona confirmó antes para esta dimensión.
                let por_memoria = || h.memoria.buscar_valor(c.dimension(), crudo).and_then(|d| c.canonico(d));
                match c.canonico(crudo) {
                    Some(v) => v.to_string(),
                    None if vacio => String::new(),
                    None => match por_memoria() {
                        Some(v) => {
                            *rec.valores_por_memoria.entry(c.nombre.clone()).or_default() += 1;
                            v.to_string()
                        }
                        None => {
                            *rec.valores_no_reconocidos.entry(c.nombre.clone()).or_default().entry(crudo.to_string()).or_default() += 1;
                            pendientes_valor
                                .entry((c.nombre.clone(), crate::normalizar::basico(crudo)))
                                .or_insert_with(|| PendienteValor { columna: c.nombre.clone(), dimension: c.dimension().to_string(), valor: crudo.to_string(), veces: 0, opciones: c.valores.clone() })
                                .veces += 1;
                            String::new()
                        }
                    },
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
    rec.pendientes_valor = pendientes_valor.into_values().collect();
    rec.pendientes_valor.sort_by(|a, b| b.veces.cmp(&a.veces));
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
            cat.agregar(Lugar { codigo: codigo.into(), nombre: nombre.into(), nivel, padre: padre.into(), alias: String::new(), ..Default::default() });
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
        assert_eq!((r.pendientes_valor[0].dimension.as_str(), r.pendientes_valor[0].valor.as_str(), r.pendientes_valor[0].veces), ("sexo", "INDETERMINADO", 1));

        // Una persona confirma el valor y el código propio de la institución: el reenvío sale limpio.
        let mut m = Memoria::default();
        m.confirmar_valor("sexo", "Indeterminado", "ignorado", "p", "analista").unwrap();
        m.confirmar("171", Nivel::Municipio, None, "0801", "p", "analista").unwrap();
        let r2 = recibir(&sol, &tabla, &Homologador::new(&cat, &m), Some("secreto-de-prueba")).unwrap();
        assert_eq!(r2.filas[1][8], "ignorado");
        assert_eq!((r2.valores_por_memoria["sexo"], r2.pendientes_valor.len()), (1, 0));
        assert_eq!(&r2.filas[1][6..8], &s(&["0801", "memoria"])[..]);
        assert!(r2.reporte.problemas.iter().all(|p| p.columna != "sexo"), "lo aprendido ya no es un problema");
        assert_eq!(r.columnas_eliminadas, vec!["nombre"]);
        assert!((r.porcentaje_resuelto() - 50.0).abs() < 1e-9);
    }

    #[test]
    fn un_lugar_de_otro_pais_no_se_busca_en_el_catalogo() {
        let sol: Solicitud = toml::from_str(r#"
            id = "p"
            institucion = "X"
            titulo = "t"
            [[columna]]
            nombre = "departamento"
            tipo = "texto"
            homologar = "departamento"
            pais = ["COD_PAIS"]
            sin_dato = ["-1"]
            [[columna]]
            nombre = "municipio"
            tipo = "texto"
            homologar = "municipio"
            contexto = "departamento"
            pais = ["COD_PAIS"]
            sin_dato = ["-1"]
        "#).unwrap();
        let mut catalogo = Catalogo::default();
        for (codigo, nombre, nivel, padre) in [("05", "Escuintla", Nivel::Departamento, ""), ("0509", "San José", Nivel::Municipio, "05")] {
            catalogo.agregar(Lugar { codigo: codigo.into(), nombre: nombre.into(), nivel, padre: padre.into(), ..Default::default() });
        }
        let memoria = Memoria::default();
        let h = Homologador::new(&catalogo, &memoria);
        // 320 = Guatemala: se homologa. 840 = otro país: 37 y 90 son divisiones de ese país. -1 = sin dato.
        let tabla = Tabla { encabezados: s(&["COD_PAIS", "departamento", "municipio"]), filas: vec![s(&["320", "5", "9"]), s(&["840", "37", "90"]), s(&["320", "-1", "-1"]), s(&["320", "37", "90"])], ..Default::default() };
        let r = recibir(&sol, &tabla, &h, None).unwrap();
        // El municipio 9 del departamento 5 es el 0509: el código de RENAP es el del INE en dos partes.
        let col = |n: &str| r.encabezados.iter().position(|e| e == n).unwrap();
        assert_eq!(r.filas[0][col("municipio_codigo")], "0509");
        assert_eq!(r.en_el_extranjero["departamento"], 1);
        assert_eq!(r.en_el_extranjero["municipio"], 1);
        // "-1" es sin dato, pero "1" es un código válido: el signo cuenta.
        assert!(sol.columnas[0].es_sin_dato("-1") && sol.columnas[0].es_sin_dato(" -1.0") && !sol.columnas[0].es_sin_dato("1") && !sol.columnas[0].es_sin_dato("11"));
        // Solo queda pendiente la fila que dice ser de Guatemala con un departamento que no existe.
        assert_eq!(r.pendientes.iter().map(|p| (p.columna.as_str(), p.valor.as_str(), p.veces)).collect::<Vec<_>>(), vec![("departamento", "37", 1), ("municipio", "90", 1)]);
    }

    #[test]
    fn el_nombre_respalda_al_codigo_y_98_99_son_sin_dato() {
        let mut cat = Catalogo::default();
        for (codigo, nombre, nivel, padre) in [("05", "Escuintla", Nivel::Departamento, ""), ("0510", "Iztapa", Nivel::Municipio, "05")] {
            cat.agregar(Lugar { codigo: codigo.into(), nombre: nombre.into(), nivel, padre: padre.into(), ..Default::default() });
        }
        let sol: Solicitud = toml::from_str(
            r#"
            id = "p"
            titulo = "p"
            institucion = "p"
            [[columna]]
            nombre = "departamento"
            tipo = "texto"
            homologar = "departamento"
            sin_dato = ["99"]
            [[columna]]
            nombre = "municipio"
            tipo = "texto"
            homologar = "municipio"
            contexto = "departamento"
            respaldo = ["NOMBRE_MUNICIPIO"]
            [[columna]]
            nombre = "edad"
            tipo = "entero"
            sin_dato = ["98", "99", "999"]
            "#,
        )
        .unwrap();
        let tabla = Tabla {
            encabezados: s(&["departamento", "municipio", "Nombre_Municipio", "edad"]),
            filas: vec![s(&["5", "37", "IZTAPA", "99"]), s(&["5", "37", "Otro lugar", "34"]), s(&["99", "", "", "999"])],
            ..Default::default()
        };
        let m = Memoria::default();
        let r = recibir(&sol, &tabla, &Homologador::new(&cat, &m), None).unwrap();
        let col = |n: &str| r.encabezados.iter().position(|e| e == n).unwrap();
        assert_eq!((r.filas[0][col("municipio_codigo")].as_str(), r.filas[0][col("edad")].as_str()), ("0510", ""), "el nombre resolvió el código 37 y 99 años es sin dato");
        assert_eq!(r.resueltos_por_respaldo["municipio"], 1);
        assert_eq!(r.filas[1][col("municipio_estado")], "codigo_invalido", "si el respaldo tampoco resuelve, queda pendiente");
        assert_eq!((r.filas[2][col("departamento_estado")].as_str(), r.filas[2][col("edad")].as_str()), ("vacio", ""));
        assert_eq!(r.pendientes.len(), 1);
    }
}
