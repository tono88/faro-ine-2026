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
    Vacio,
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
        if k.is_empty() {
            return r;
        }

        // Si ya viene el código oficial, se respeta.
        if let Some(l) = self.catalogo.por_codigo(texto.trim()) {
            if l.nivel == nivel && contexto.map_or(true, |c| l.padre == c) {
                return self.asignar(r, Estado::Exacto, &l.codigo, 1.0);
            }
        }

        let mut puntuados: Vec<Candidato> = self
            .catalogo
            .de_nivel(nivel, contexto)
            .map(|l| Candidato { codigo: l.codigo.clone(), nombre: l.nombre.clone(), puntaje: similitud(&k, &l.clave) })
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
            c.agregar(Lugar { codigo: codigo.into(), nombre: nombre.into(), nivel, padre: padre.into(), clave: String::new() });
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
        m.confirmar("Puerto de San José", Nivel::Municipio, Some("05"), "0509", "MP", "analista").unwrap();
        let h = Homologador::new(&c, &m);
        for v in ["el puerto", "EL PUERTO.", "Pto. San José", "puerto san jose"] {
            let r = h.homologar(v, Nivel::Municipio, Some("05"));
            assert_eq!((r.estado, r.codigo.as_deref()), (Estado::Memoria, Some("0509")), "falló {v:?}");
        }
        // La equivalencia vale solo en su contexto: en Petén "El Puerto" no se asigna.
        assert!(!h.homologar("El Puerto", Nivel::Municipio, Some("17")).estado.resuelto());
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
