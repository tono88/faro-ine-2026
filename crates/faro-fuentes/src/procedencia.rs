//! Descarga con procedencia: cada archivo queda con URL, fecha, tamaño y huella SHA-256
//! en un manifiesto de solo agregar. De ahí sale la tabla de fuentes del proyecto.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Descarga {
    pub fuente: String,
    pub url: String,
    pub archivo: String,
    pub fecha: String,
    pub bytes: u64,
    pub sha256: String,
    pub editor: String,
    pub licencia: String,
    /// "nuevo", "cambio" o "igual" respecto de la descarga anterior de la misma URL.
    pub estado: String,
}

pub struct Manifiesto {
    ruta: PathBuf,
}

/// Límite por archivo: evita llenar el disco con una respuesta inesperada.
pub const MAX_BYTES: u64 = 2 * 1024 * 1024 * 1024;

impl Manifiesto {
    pub fn new(ruta: &Path) -> Self {
        Self { ruta: ruta.to_path_buf() }
    }

    pub fn leer(&self) -> Result<Vec<Descarga>, String> {
        if !self.ruta.exists() {
            return Ok(vec![]);
        }
        std::fs::read_to_string(&self.ruta)
            .map_err(|e| e.to_string())?
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| serde_json::from_str(l).map_err(|e| format!("{}: {e}", self.ruta.display())))
            .collect()
    }

    fn agregar(&self, d: &Descarga) -> Result<(), String> {
        if let Some(dir) = self.ruta.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let mut f = std::fs::OpenOptions::new().create(true).append(true).open(&self.ruta).map_err(|e| e.to_string())?;
        writeln!(f, "{}", serde_json::to_string(d).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
    }

    /// Registra un flujo de bytes ya abierto (de red o de disco) y lo guarda en `destino`.
    pub fn registrar(
        &self,
        fuente: &str,
        url: &str,
        mut contenido: impl Read,
        destino: &Path,
        editor: &str,
        licencia: &str,
    ) -> Result<Descarga, String> {
        if let Some(dir) = destino.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        // Se escribe a un temporal y se renombra: nunca queda un archivo a medias.
        let temporal = destino.with_extension("parcial");
        let mut salida = std::fs::File::create(&temporal).map_err(|e| format!("{}: {e}", temporal.display()))?;
        let mut huella = Sha256::new();
        let mut bytes = 0u64;
        let mut bloque = [0u8; 64 * 1024];
        loop {
            let n = contenido.read(&mut bloque).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            bytes += n as u64;
            if bytes > MAX_BYTES {
                drop(salida);
                let _ = std::fs::remove_file(&temporal);
                return Err(format!("{url}: supera el límite de {MAX_BYTES} bytes"));
            }
            huella.update(&bloque[..n]);
            salida.write_all(&bloque[..n]).map_err(|e| e.to_string())?;
        }
        drop(salida);
        std::fs::rename(&temporal, destino).map_err(|e| e.to_string())?;
        let sha256: String = huella.finalize().iter().map(|b| format!("{b:02x}")).collect();
        let anterior = self.leer()?.into_iter().rev().find(|d| d.url == url);
        let estado = match anterior {
            None => "nuevo",
            Some(a) if a.sha256 == sha256 => "igual",
            Some(_) => "cambio",
        };
        let d = Descarga {
            fuente: fuente.to_string(),
            url: url.to_string(),
            archivo: destino.display().to_string(),
            fecha: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            bytes,
            sha256,
            editor: editor.to_string(),
            licencia: licencia.to_string(),
            estado: estado.to_string(),
        };
        self.agregar(&d)?;
        Ok(d)
    }

    /// Descarga `url` por HTTP y la registra.
    pub fn descargar(&self, fuente: &str, url: &str, destino: &Path, editor: &str, licencia: &str) -> Result<Descarga, String> {
        let respuesta = crate::agente().get(url).call().map_err(|e| format!("{url}: {e}"))?;
        let tipo = respuesta.content_type().to_string();
        let d = self.registrar(fuente, url, respuesta.into_reader(), destino, editor, licencia)?;
        if tipo.starts_with("text/html") {
            return Err(format!("{url}: el servidor devolvió una página web, no el archivo (revisar el enlace)"));
        }
        Ok(d)
    }

    /// Tabla Markdown de fuentes: la última descarga de cada URL.
    pub fn citar(&self) -> Result<String, String> {
        let mut ultimas: Vec<Descarga> = vec![];
        for d in self.leer()? {
            match ultimas.iter_mut().find(|u| u.url == d.url) {
                Some(u) => *u = d,
                None => ultimas.push(d),
            }
        }
        let mut t = String::from("| Fuente | Editor | Fecha de descarga (UTC) | Licencia | SHA-256 | URL |\n|---|---|---|---|---|---|\n");
        for d in ultimas {
            t.push_str(&format!("| {} | {} | {} | {} | `{}` | {} |\n", d.fuente, d.editor, d.fecha, d.licencia, &d.sha256[..12], d.url));
        }
        Ok(t)
    }
}

/// Nombre de archivo seguro a partir de una URL.
pub fn nombre_de_url(url: &str) -> String {
    let base = url.split(['?', '#']).next().unwrap_or(url).trim_end_matches('/');
    let nombre: String = base
        .rsplit('/')
        .next()
        .unwrap_or("recurso")
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' })
        .take(120)
        .collect();
    if nombre.is_empty() { "recurso".into() } else { nombre }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn registra_huella_y_detecta_igual_y_cambio() {
        let dir = std::env::temp_dir().join(format!("faro-proc-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let m = Manifiesto::new(&dir.join("manifiesto.jsonl"));
        let url = "https://ejemplo.gt/a.csv";
        let d1 = m.registrar("x", url, &b"a,b\n1,2\n"[..], &dir.join("a.csv"), "INE", "cc-by").unwrap();
        assert_eq!(d1.estado, "nuevo");
        assert_eq!(d1.bytes, 8);
        assert_eq!(d1.sha256.len(), 64);
        assert_eq!(m.registrar("x", url, &b"a,b\n1,2\n"[..], &dir.join("a.csv"), "INE", "cc-by").unwrap().estado, "igual");
        assert_eq!(m.registrar("x", url, &b"a,b\n1,3\n"[..], &dir.join("a.csv"), "INE", "cc-by").unwrap().estado, "cambio");
        let tabla = m.citar().unwrap();
        assert_eq!(tabla.lines().count(), 3, "una fila por URL: {tabla}");
        assert!(!dir.join("a.parcial").exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn nombres_de_archivo_seguros() {
        assert_eq!(nombre_de_url("https://x.gt/d/centroides de lugares.xlsx?x=1"), "centroides_de_lugares.xlsx");
        assert_eq!(nombre_de_url("https://x.gt/../../etc/passwd"), "passwd");
        assert_eq!(nombre_de_url("https://x.gt/"), "x.gt");
    }
}
