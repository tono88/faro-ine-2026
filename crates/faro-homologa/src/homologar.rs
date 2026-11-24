//! Decide a qué lugar oficial corresponde un texto libre.
//!
//! Regla de oro: el sistema solo asigna solo cuando no hay duda (coincidencia exacta y única,
//! o equivalencia ya confirmada). Todo lo demás se propone y lo decide una persona; su
//! decisión queda en la memoria y no se vuelve a preguntar.
use crate::catalogo::{Catalogo, Nivel};
use crate::memoria::Memoria;
use crate::normalizar::{clave, similitud};
use serde::Serialize;

/// Puntaje mínimo para proponer un candidato.
pub const UMBRAL_CANDIDATO: f64 = 0.80;
/// Puntaje desde el cual un candidato único se marca como sugerencia fuerte.
pub const UMBRAL_SUGERIDO: f64 = 0.93;
/// Ventaja mínima sobre el segundo candidato para ser sugerencia fuerte.
pub const MARGEN: f64 = 0.03;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Estado {
    /// Coincide con un único nombre oficial.
    Exacto,
    /// Coincide con una equivalencia confirmada antes por una persona.
    Memoria,
    /// Un candidato muy parecido y sin competencia: falta confirmarlo.
    Sugerido,
    /// Varios candidatos posibles (ej. "San José" sin departamento): decide una persona.
    Ambiguo,
    /// Hay candidatos parecidos pero ninguno claro.
    Revisar,
    SinCoincidencia,
    /// El nombre no existe donde se buscó, pero sí (exacto) en otro departamento o municipio:
    /// probablemente el nivel superior viene mal. Decide una persona.
    FueraDeContexto,
    /// Llegó un número que no corresponde a ningún código del catálogo.
    CodigoInvalido,
    /// Vacío o marcado como ignorado en origen ("IGNORADO", "-1", "N.D.").
    Vacio,
}

/// Formas en que las instituciones escriben "no se sabe".
pub fn es_ignorado(texto: &str) -> bool {
    let t = texto.trim();
    if t.is_empty() || t.starts_with('-') && t[1..].chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    matches!(
        crate::normalizar::basico(t).as_str(),
        "ignorado" | "ignorada" | "desconocido" | "no indica" | "no determinado" | "sin dato" | "sin datos" | "n d" | "nd" | "ns nr" | "todos" | "none" | "null" | "na" | "n a"
    )
}

/// Código INE a partir de un número: departamento a 2 dígitos; municipio como
/// correlativo dentro de su departamento (con contexto) o código completo de 3–4 dígitos.
/// Lugar poblado: código completo de 6–7 dígitos (municipio + correlativo de 3).
fn codigo_numerico(texto: &str, nivel: Nivel, contexto: Option<&str>) -> Option<String> {
    let t = texto.trim().trim_end_matches(".0");
    if t.is_empty() || !t.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(match nivel {
        Nivel::Departamento => format!("{t:0>2}"),
        Nivel::Municipio => match contexto {
            Some(c) if t.len() <= 2 => format!("{c}{t:0>2}"),
            _ => format!("{t:0>4}"),
        },
        Nivel::LugarPoblado => format!("{t:0>7}"),
    })
}

impl Estado {
    pub fn texto(self) -> &'static str {
        match self {
            Estado::Exacto => "exacto",
            Estado::Memoria => "memoria",
            Estado::Sugerido => "sugerido",
            Estado::Ambiguo => "ambiguo",
            Estado::Revisar => "revisar",
            Estado::SinCoincidencia => "sin_coincidencia",
            Estado::FueraDeContexto => "fuera_de_contexto",
            Estado::CodigoInvalido => "codigo_invalido",
            Estado::Vacio => "vacio",
        }
    }

    /// ¿Se puede usar el código sin intervención humana?
    pub fn resuelto(self) -> bool {
        matches!(self, Estado::Exacto | Estado::Memoria)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Candidato {
    pub codigo: String,
    pub nombre: String,
    pub puntaje: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Resultado {
    pub original: String,
    pub estado: Estado,
    /// Código asignado (solo si el estado es exacto o memoria).
    pub codigo: Option<String>,
    pub nombre_oficial: Option<String>,
    pub puntaje: f64,
    /// Hasta 3 candidatos, del más al menos parecido.
    pub candidatos: Vec<Candidato>,
}

pub struct Homologador<'a> {
    pub catalogo: &'a Catalogo,
    pub memoria: &'a Memoria,
}

impl<'a> Homologador<'a> {
    pub fn new(catalogo: &'a Catalogo, memoria: &'a Memoria) -> Self {
        Self { catalogo, memoria }
    }

    /// Homologa `texto` contra los lugares de `nivel`. `contexto` es el código del nivel
    /// superior (ej. "05" al buscar un municipio de Escuintla) y reduce la ambigüedad.
    pub fn homologar(&self, texto: &str, nivel: Nivel, contexto: Option<&str>) -> Resultado {
        let k = clave(texto);
        let mut r = Resultado {
            original: texto.to_string(),
            estado: Estado::Vacio,
            codigo: None,
            nombre_oficial: None,
            puntaje: 0.0,
            candidatos: vec![],
        };
        if k.is_empty() || es_ignorado(texto) {
            return r;
        }

        // Si viene un código, se interpreta: "8" + contexto "08" no aplica a departamento;
        // municipio "1" con contexto "08" es 0801; municipio "101" es 0101.
        if let Some(codigo) = codigo_numerico(texto, nivel, contexto) {
            return match self.catalogo.por_codigo(&codigo) {
                Some(l) if l.nivel == nivel && contexto.map_or(true, |c| l.padre == c) => {
                    let codigo = l.codigo.clone();
                    self.asignar(r, Estado::Exacto, &codigo, 1.0)
                }
                _ => {
                    // Un código propio de la institución que una persona ya tradujo.
                    if let Some(c) = self.memoria.buscar(nivel, contexto, &k).filter(|c| self.catalogo.por_codigo(c).is_some()) {
                        let c = c.to_string();
                        return self.asignar(r, Estado::Memoria, &c, 1.0);
                    }
                    r.estado = Estado::CodigoInvalido;
                    r
                }
            };
        }

        let mut puntuados: Vec<Candidato> = self
            .catalogo
            .de_nivel(nivel, contexto)
            .map(|l| Candidato {
                codigo: l.codigo.clone(),
                nombre: l.nombre.clone(),
                puntaje: l.claves.iter().map(|c| similitud(&k, c)).fold(0.0, f64::max),
            })
            .filter(|c| c.puntaje >= UMBRAL_CANDIDATO)
            .collect();
        puntuados.sort_by(|a, b| b.puntaje.total_cmp(&a.puntaje).then(a.codigo.cmp(&b.codigo)));

        let exactos: Vec<&Candidato> = puntuados.iter().filter(|c| c.puntaje >= 0.99).collect();
        if exactos.len() == 1 {
            let codigo = exactos[0].codigo.clone();
            return self.asignar(r, Estado::Exacto, &codigo, 1.0);
        }

        // La memoria resuelve lo que el catálogo no puede (alias y ambigüedades ya decididas).
        if let Some(codigo) = self.memoria.buscar(nivel, contexto, &k) {
            if self.catalogo.por_codigo(codigo).is_some() {
                let codigo = codigo.to_string();
                return self.asignar(r, Estado::Memoria, &codigo, 1.0);
            }
        }

        r.puntaje = puntuados.first().map_or(0.0, |c| c.puntaje);
        r.estado = if exactos.len() > 1 {
            Estado::Ambiguo
        } else if puntuados.is_empty() {
            Estado::SinCoincidencia
        } else {
            let segundo = puntuados.get(1).map_or(0.0, |c| c.puntaje);
            if r.puntaje >= UMBRAL_SUGERIDO && r.puntaje - segundo >= MARGEN {
                Estado::Sugerido
            } else {
                Estado::Revisar
            }
        };
        // En un ambiguo solo se muestran los empatados: son las únicas opciones reales.
        let limite = if r.estado == Estado::Ambiguo { exactos.len() } else { 3 };
        puntuados.truncate(limite);
        r.candidatos = puntuados;

        // "Camotán" buscado en El Progreso no existe, pero sí en Chiquimula: se muestra para que
        // una persona decida si el departamento vino mal. Nunca se asigna solo.
        if contexto.is_some() && r.candidatos.iter().all(|c| c.puntaje < UMBRAL_SUGERIDO) {
            let mut fuera: Vec<Candidato> = self
                .catalogo
                .de_nivel(nivel, None)
                .filter(|l| l.claves.iter().any(|c| *c == k))
                .map(|l| {
                    let padre = self.catalogo.por_codigo(&l.padre).map_or(String::new(), |p| format!(" ({})", p.nombre));
                    Candidato { codigo: l.codigo.clone(), nombre: format!("{}{padre}", l.nombre), puntaje: 1.0 }
                })
                .collect();
            if !fuera.is_empty() {
                fuera.truncate(5);
                fuera.extend(r.candidatos.drain(..));
                fuera.truncate(5);
                r.candidatos = fuera;
                r.estado = Estado::FueraDeContexto;
            }
        }
        r
    }

    fn asignar(&self, mut r: Resultado, estado: Estado, codigo: &str, puntaje: f64) -> Resultado {
        let l = self.catalogo.por_codigo(codigo).expect("código verificado antes de asignar");
        r.estado = estado;
        r.codigo = Some(l.codigo.clone());
        r.nombre_oficial = Some(l.nombre.clone());
        r.puntaje = puntaje;
        r
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::catalogo::Lugar;

    fn catalogo() -> Catalogo {
        let mut c = Catalogo::default();
        for (codigo, nombre, nivel, padre) in [
            ("05", "Escuintla", Nivel::Departamento, ""),
            ("17", "Petén", Nivel::Departamento, ""),
            ("0501", "Escuintla", Nivel::Municipio, "05"),
            ("0509", "San José", Nivel::Municipio, "05"),
            ("0510", "Iztapa", Nivel::Municipio, "05"),
            ("0502", "Santa Lucía Cotzumalguapa", Nivel::Municipio, "05"),
            ("1702", "San José", Nivel::Municipio, "17"),
            ("1703", "San Benito", Nivel::Municipio, "17"),
        ] {
            c.agregar(Lugar { codigo: codigo.into(), nombre: nombre.into(), nivel, padre: padre.into(), alias: if codigo == "0509" { "Puerto San José".into() } else { String::new() }, ..Default::default() });
        }
        c
    }

    #[test]
    fn exacto_ignora_tildes_y_mayusculas() {
        let (c, m) = (catalogo(), Memoria::default());
        let r = Homologador::new(&c, &m).homologar("SANTA LUCIA COTZUMALGUAPA", Nivel::Municipio, None);
        assert_eq!((r.estado, r.codigo.as_deref()), (Estado::Exacto, Some("0502")));
    }

    #[test]
    fn san_jose_sin_departamento_es_ambiguo_y_con_departamento_es_exacto() {
        let (c, m) = (catalogo(), Memoria::default());
        let h = Homologador::new(&c, &m);
        let r = h.homologar("San José", Nivel::Municipio, None);
        assert_eq!(r.estado, Estado::Ambiguo);
        assert_eq!(r.candidatos.len(), 2);
        assert!(r.codigo.is_none(), "nunca se asigna un código ambiguo");
        let r = h.homologar("San José", Nivel::Municipio, Some("17"));
        assert_eq!((r.estado, r.codigo.as_deref()), (Estado::Exacto, Some("1702")));
    }

    #[test]
    fn nombre_que_existe_en_otro_departamento_se_propone_sin_asignar() {
        let (c, m) = (catalogo(), Memoria::default());
        let r = Homologador::new(&c, &m).homologar("San Benito", Nivel::Municipio, Some("05"));
        assert_eq!(r.estado, Estado::FueraDeContexto);
        assert!(r.codigo.is_none());
        assert_eq!((r.candidatos[0].codigo.as_str(), r.candidatos[0].nombre.as_str()), ("1703", "San Benito (Petén)"));
    }

    #[test]
    fn letras_danadas_no_impiden_reconocer_el_lugar() {
        let (c, m) = (catalogo(), Memoria::default());
        let r = Homologador::new(&c, &m).homologar("Santa LucÝa Cotzumalguapa", Nivel::Municipio, Some("05"));
        assert_eq!((r.estado, r.codigo.as_deref()), (Estado::Exacto, Some("0502")));
    }

    #[test]
    fn un_codigo_propio_de_la_institucion_se_aprende_una_vez() {
        let c = catalogo();
        let mut m = Memoria::default();
        assert_eq!(Homologador::new(&c, &m).homologar("37", Nivel::Municipio, Some("05")).estado, Estado::CodigoInvalido);
        m.confirmar("37", Nivel::Municipio, Some("05"), "0510", "RENAP", "analista").unwrap();
        let r = Homologador::new(&c, &m).homologar("37", Nivel::Municipio, Some("05"));
        assert_eq!((r.estado, r.codigo.as_deref()), (Estado::Memoria, Some("0510")));
        assert_eq!(Homologador::new(&c, &m).homologar("37", Nivel::Municipio, Some("17")).estado, Estado::CodigoInvalido, "vale solo en su departamento");
    }

    #[test]
    fn error_de_dedo_se_sugiere_pero_no_se_asigna() {
        let (c, m) = (catalogo(), Memoria::default());
        let r = Homologador::new(&c, &m).homologar("Sta. Lucia Cotzumalguapa", Nivel::Municipio, Some("05"));
        assert_eq!(r.estado, Estado::Exacto, "la abreviatura Sta. se expande");
        let r = Homologador::new(&c, &m).homologar("Santa Lucia Cotzumalgupa", Nivel::Municipio, Some("05"));
        assert_eq!(r.estado, Estado::Sugerido);
        assert!(r.codigo.is_none());
        assert_eq!(r.candidatos[0].codigo, "0502");
    }

    #[test]
    fn el_puerto_se_aprende_una_vez() {
        let c = catalogo();
        let mut m = Memoria::default();
        {
            let r = Homologador::new(&c, &m).homologar("El Puerto", Nivel::Municipio, Some("05"));
            assert!(!r.estado.resuelto());
        }
        m.confirmar("El Puerto", Nivel::Municipio, Some("05"), "0509", "MP", "analista").unwrap();
        let h = Homologador::new(&c, &m);
        for v in ["el puerto", "EL PUERTO."] {
            let r = h.homologar(v, Nivel::Municipio, Some("05"));
            assert_eq!((r.estado, r.codigo.as_deref()), (Estado::Memoria, Some("0509")), "falló {v:?}");
        }
        // Los alias del catálogo resuelven sin memoria.
        for v in ["Pto. San José", "puerto de san jose"] {
            assert_eq!(h.homologar(v, Nivel::Municipio, Some("05")).estado, Estado::Exacto, "falló {v:?}");
        }
        // La equivalencia vale solo en su contexto: en Petén "El Puerto" no se asigna.
        assert!(!h.homologar("El Puerto", Nivel::Municipio, Some("17")).estado.resuelto());
    }

    #[test]
    fn codigos_como_los_envian_renap_y_pnc() {
        let (c, m) = (catalogo(), Memoria::default());
        let h = Homologador::new(&c, &m);
        // RENAP: departamento "5" y municipio "9" como correlativo.
        assert_eq!(h.homologar("5", Nivel::Departamento, None).codigo.as_deref(), Some("05"));
        assert_eq!(h.homologar("9", Nivel::Municipio, Some("05")).codigo.as_deref(), Some("0509"));
        // PNC: código completo sin cero inicial.
        assert_eq!(h.homologar("510", Nivel::Municipio, None).codigo.as_deref(), Some("0510"));
        // Un código que no existe no se "arregla": se reporta.
        assert_eq!(h.homologar("97", Nivel::Municipio, Some("05")).estado, Estado::CodigoInvalido);
        assert_eq!(h.homologar("43", Nivel::Departamento, None).estado, Estado::CodigoInvalido);
        for v in ["-1", "IGNORADO", "N.D.", "Todos", ""] {
            assert_eq!(h.homologar(v, Nivel::Municipio, None).estado, Estado::Vacio, "{v:?}");
        }
    }

    #[test]
    fn codigo_oficial_se_respeta_y_basura_no_coincide() {
        let (c, m) = (catalogo(), Memoria::default());
        let h = Homologador::new(&c, &m);
        assert_eq!(h.homologar("0510", Nivel::Municipio, None).codigo.as_deref(), Some("0510"));
        assert_eq!(h.homologar("xxxxxxx", Nivel::Municipio, None).estado, Estado::SinCoincidencia);
        assert_eq!(h.homologar("  ", Nivel::Municipio, None).estado, Estado::Vacio);
    }
}
