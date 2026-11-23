//! Cliente mínimo de la API CKAN v3 (la que usa datos.ine.gob.gt).
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recurso {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub datastore_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conjunto {
    pub name: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub license_id: Option<String>,
    #[serde(default)]
    pub metadata_modified: String,
    #[serde(default)]
    pub resources: Vec<Recurso>,
}

impl Conjunto {
    /// Licencia declarada en el portal, o aviso de que no la declara.
    pub fn licencia(&self) -> &str {
        match self.license_id.as_deref() {
            Some(l) if !l.trim().is_empty() => l,
            _ => "no declarada",
        }
    }
}

/// Extrae `result` de una respuesta CKAN, o el mensaje de error del portal.
pub fn resultado(respuesta: Value) -> Result<Value, String> {
    if respuesta.get("success").and_then(Value::as_bool) != Some(true) {
        let detalle = respuesta.get("error").map(Value::to_string).unwrap_or_else(|| "respuesta inesperada".into());
        return Err(format!("el portal respondió con error: {detalle}"));
    }
    respuesta.get("result").cloned().ok_or_else(|| "respuesta sin 'result'".to_string())
}

pub struct Ckan {
    base: String,
    agente: ureq::Agent,
}

impl Ckan {
    pub fn new(base: &str) -> Self {
        Self { base: base.trim_end_matches('/').to_string(), agente: crate::agente() }
    }

    fn accion(&self, nombre: &str, parametros: &[(&str, &str)]) -> Result<Value, String> {
        let url = format!("{}/api/3/action/{nombre}", self.base);
        let mut pedido = self.agente.get(&url);
        for (k, v) in parametros {
            pedido = pedido.query(k, v);
        }
        let cuerpo: Value = pedido
            .call()
            .map_err(|e| format!("{url}: {e}"))?
            .into_json()
            .map_err(|e| format!("{url}: la respuesta no es JSON ({e})"))?;
        resultado(cuerpo)
    }

    /// Nombres de todos los conjuntos del portal.
    pub fn listar(&self) -> Result<Vec<String>, String> {
        serde_json::from_value(self.accion("package_list", &[])?).map_err(|e| e.to_string())
    }

    pub fn conjunto(&self, nombre: &str) -> Result<Conjunto, String> {
        serde_json::from_value(self.accion("package_show", &[("id", nombre)])?).map_err(|e| e.to_string())
    }

    /// Todos los conjuntos con sus recursos, paginando la búsqueda del portal.
    pub fn inventario(&self) -> Result<Vec<Conjunto>, String> {
        let mut todos: Vec<Conjunto> = vec![];
        loop {
            let inicio = todos.len().to_string();
            let r = self.accion("package_search", &[("rows", "100"), ("start", &inicio), ("sort", "name asc")])?;
            let total = r.get("count").and_then(Value::as_u64).unwrap_or(0) as usize;
            let lote: Vec<Conjunto> =
                serde_json::from_value(r.get("results").cloned().unwrap_or(Value::Array(vec![]))).map_err(|e| e.to_string())?;
            if lote.is_empty() {
                break;
            }
            todos.extend(lote);
            if todos.len() >= total {
                break;
            }
        }
        Ok(todos)
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    // Forma real de package_show en datos.ine.gob.gt (consultado el 23-nov-2026), recortada.
    const MUESTRA: &str = include_str!("../pruebas/package_show_lugares_poblados.json");

    #[test]
    fn interpreta_un_conjunto_del_ine() {
        let c: Conjunto = serde_json::from_value(resultado(serde_json::from_str(MUESTRA).unwrap()).unwrap()).unwrap();
        assert_eq!(c.name, "censo-2018-lugares-poblados");
        assert_eq!(c.licencia(), "cc-by");
        assert_eq!(c.resources[0].format, "XLSX");
        assert!(c.resources[0].url.ends_with(".xlsx"));
        assert!(c.resources[1].datastore_active);
    }

    #[test]
    fn licencia_vacia_se_reporta_como_no_declarada() {
        let c: Conjunto = serde_json::from_str(r#"{"name":"x","license_id":""}"#).unwrap();
        assert_eq!(c.licencia(), "no declarada");
        let c: Conjunto = serde_json::from_str(r#"{"name":"x","license_id":null}"#).unwrap();
        assert_eq!(c.licencia(), "no declarada");
    }

    #[test]
    fn error_del_portal_se_propaga() {
        let e = resultado(serde_json::json!({"success": false, "error": {"message": "No encontrado"}})).unwrap_err();
        assert!(e.contains("No encontrado"));
    }
}
