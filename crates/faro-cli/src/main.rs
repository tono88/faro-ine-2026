//! `faro`: recepción y homologación de envíos de registros administrativos.
use clap::{Parser, Subcommand};
use faro_homologa::solicitud::Solicitud;
use faro_homologa::{Catalogo, Homologador, Memoria, Nivel};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "faro", version, about = "Recepción y homologación de registros administrativos para el INE")]
struct Cli {
    /// Carpeta con los catálogos oficiales (.csv con codigo,nombre,nivel,padre).
    #[arg(long, global = true, default_value = "datos/catalogo")]
    catalogo: PathBuf,
    /// Archivo de equivalencias confirmadas (JSONL, se crea si no existe).
    #[arg(long, global = true, default_value = "datos/memoria/equivalencias.jsonl")]
    memoria: PathBuf,
    #[command(subcommand)]
    comando: Comando,
}

#[derive(Subcommand)]
enum Comando {
    /// Valida un envío contra su solicitud y homologa sus columnas geográficas.
    Recibir {
        #[arg(long)]
        solicitud: PathBuf,
        #[arg(long)]
        entrada: PathBuf,
        /// Carpeta de salida (homologado.csv, pendientes.csv, reporte.json).
        #[arg(long, default_value = "salida")]
        salida: PathBuf,
    },
    /// Consulta a qué lugar oficial corresponde un texto.
    Homologar {
        texto: String,
        #[arg(long, default_value = "municipio")]
        nivel: Nivel,
        /// Código del nivel superior (ej. 05 para municipios de Escuintla).
        #[arg(long)]
        contexto: Option<String>,
    },
    /// Graba una equivalencia confirmada por una persona.
    Confirmar {
        texto: String,
        #[arg(long)]
        codigo: String,
        #[arg(long, default_value = "municipio")]
        nivel: Nivel,
        #[arg(long)]
        contexto: Option<String>,
        #[arg(long, default_value = "")]
        institucion: String,
        #[arg(long, default_value = "")]
        usuario: String,
    },
    /// Descubre y descarga datos abiertos del INE dejando registro de procedencia.
    Fuentes {
        /// Portal CKAN.
        #[arg(long, default_value = faro_fuentes::INE_CKAN)]
        portal: String,
        #[arg(long, default_value = "datos/manifiesto.jsonl")]
        manifiesto: PathBuf,
        #[command(subcommand)]
        accion: Fuentes,
    },
    /// Crecimiento de edificaciones por territorio y proyección (CSV: codigo,anio,edificaciones).
    Crecimiento {
        #[arg(long)]
        entrada: PathBuf,
        /// Año al que se proyecta.
        #[arg(long, default_value_t = 2030)]
        proyectar: i32,
        #[arg(long, default_value = "salida/crecimiento.csv")]
        salida: PathBuf,
    },
    /// Promedios mensuales de calidad del aire con la regla de representatividad (formato INSIVUMEH).
    Aire {
        #[arg(long)]
        entrada: PathBuf,
        /// Un día con más horas sin dato que este número no cuenta.
        #[arg(long, default_value_t = 4)]
        max_horas_sin_dato: usize,
        /// Un mes con más días no válidos que este número se descarta.
        #[arg(long, default_value_t = 2)]
        max_dias_no_validos: usize,
        #[arg(long, default_value = "salida/aire_mensual.csv")]
        salida: PathBuf,
    },
}

#[derive(Subcommand)]
enum Fuentes {
    /// Lista todos los conjuntos y recursos del portal (CSV).
    Inventario {
        #[arg(long, default_value = "datos/inventario_ine.csv")]
        salida: PathBuf,
    },
    /// Descarga los recursos de un conjunto a datos/crudo/<conjunto>/.
    Descargar {
        conjunto: String,
        /// Solo estos formatos (ej. XLSX,CSV). Vacío = todos.
        #[arg(long, value_delimiter = ',')]
        formato: Vec<String>,
        #[arg(long, default_value = "datos/crudo")]
        destino: PathBuf,
    },
    /// Tabla de fuentes con fecha de descarga y huella, para la documentación.
    Citar,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match ejecutar(cli) {
        Ok(codigo) => codigo,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(1)
        }
    }
}

fn ejecutar(cli: Cli) -> Result<ExitCode, String> {
    // Estos comandos no necesitan catálogo ni memoria.
    let comando = match cli.comando {
        Comando::Fuentes { portal, manifiesto, accion } => return fuentes(&portal, &manifiesto, accion),
        Comando::Crecimiento { entrada, proyectar, salida } => return crecimiento(&entrada, proyectar, &salida),
        Comando::Aire { entrada, max_horas_sin_dato, max_dias_no_validos, salida } => {
            let filas = faro_homologa::lectura::leer_crudo(&entrada)?;
            let meses = faro_territorio::aire::promedios_mensuales(&filas, max_horas_sin_dato, max_dias_no_validos)?;
            if let Some(dir) = salida.parent() {
                std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            }
            let mut w = csv::Writer::from_path(&salida).map_err(|e| e.to_string())?;
            for m in &meses {
                w.serialize(m).map_err(|e| e.to_string())?;
            }
            w.flush().map_err(|e| e.to_string())?;
            let cuenta = |e: &str| meses.iter().filter(|m| m.estado == e).count();
            println!("{} meses-estación-contaminante: {} válidos, {} descartados, {} sin datos -> {}", meses.len(), cuenta("valido"), cuenta("descartado"), cuenta("sin_datos"), salida.display());
            return Ok(ExitCode::SUCCESS);
        }
        otro => otro,
    };
    let catalogo = Catalogo::cargar_carpeta(&cli.catalogo)?;
    let mut memoria = Memoria::abrir(&cli.memoria)?;
    match comando {
        Comando::Homologar { texto, nivel, contexto } => {
            let r = Homologador::new(&catalogo, &memoria).homologar(&texto, nivel, contexto.as_deref());
            println!("{}", serde_json::to_string_pretty(&r).map_err(|e| e.to_string())?);
            Ok(if r.estado.resuelto() { ExitCode::SUCCESS } else { ExitCode::from(2) })
        }
        Comando::Confirmar { texto, codigo, nivel, contexto, institucion, usuario } => {
            let lugar = catalogo.por_codigo(&codigo).ok_or(format!("el código {codigo} no existe en el catálogo"))?;
            if lugar.nivel != nivel {
                return Err(format!("{codigo} ({}) no es del nivel pedido", lugar.nombre));
            }
            if let Some(c) = &contexto {
                if lugar.padre != *c {
                    return Err(format!("{codigo} ({}) no pertenece a {c}", lugar.nombre));
                }
            }
            memoria.confirmar(&texto, nivel, contexto.as_deref(), &codigo, &institucion, &usuario)?;
            println!("grabado: {texto:?} = {codigo} {}", lugar.nombre);
            Ok(ExitCode::SUCCESS)
        }
        Comando::Recibir { solicitud, entrada, salida } => recibir(&catalogo, &memoria, &solicitud, &entrada, &salida),
        Comando::Fuentes { .. } | Comando::Crecimiento { .. } | Comando::Aire { .. } => unreachable!("atendidos arriba"),
    }
}

fn fuentes(portal: &str, ruta_manifiesto: &Path, accion: Fuentes) -> Result<ExitCode, String> {
    let manifiesto = faro_fuentes::Manifiesto::new(ruta_manifiesto);
    match accion {
        Fuentes::Citar => print!("{}", manifiesto.citar()?),
        Fuentes::Inventario { salida } => {
            let conjuntos = faro_fuentes::Ckan::new(portal).inventario()?;
            if let Some(dir) = salida.parent() {
                std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            }
            let mut w = csv::Writer::from_path(&salida).map_err(|e| e.to_string())?;
            w.write_record(["conjunto", "titulo", "licencia", "modificado", "recurso_id", "recurso", "formato", "datastore", "url"]).map_err(|e| e.to_string())?;
            let mut recursos = 0;
            for c in &conjuntos {
                for r in &c.resources {
                    recursos += 1;
                    w.write_record([&c.name, &c.title, c.licencia(), &c.metadata_modified, &r.id, &r.name, &r.format.to_uppercase(), &r.datastore_active.to_string(), &r.url]).map_err(|e| e.to_string())?;
                }
            }
            w.flush().map_err(|e| e.to_string())?;
            println!("{} conjuntos, {recursos} recursos -> {}", conjuntos.len(), salida.display());
        }
        Fuentes::Descargar { conjunto, formato, destino } => {
            let c = faro_fuentes::Ckan::new(portal).conjunto(&conjunto)?;
            let quiere = |f: &str| formato.is_empty() || formato.iter().any(|x| x.eq_ignore_ascii_case(f));
            let mut fallos = 0;
            for r in c.resources.iter().filter(|r| quiere(&r.format) && r.url.starts_with("http")) {
                let id_corto: String = r.id.chars().take(8).collect();
                let archivo = destino.join(&c.name).join(format!("{id_corto}_{}", faro_fuentes::procedencia::nombre_de_url(&r.url)));
                match manifiesto.descargar(&c.name, &r.url, &archivo, "Instituto Nacional de Estadística (INE)", c.licencia()) {
                    Ok(d) => println!("{:>6}  {:>10} bytes  {}", d.estado, d.bytes, d.archivo),
                    Err(e) => {
                        fallos += 1;
                        eprintln!("fallo: {e}");
                    }
                }
            }
            if fallos > 0 {
                return Ok(ExitCode::from(2));
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn crecimiento(entrada: &Path, proyectar: i32, salida: &Path) -> Result<ExitCode, String> {
    let series = faro_territorio::leer_series(entrada)?;
    if let Some(dir) = salida.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let mut w = csv::Writer::from_path(salida).map_err(|e| e.to_string())?;
    let (mut hechos, mut omitidos) = (vec![], 0);
    for (codigo, serie) in &series {
        match faro_territorio::analizar(codigo, serie, proyectar) {
            Some(c) => hechos.push(c),
            None => omitidos += 1,
        }
    }
    hechos.sort_by(|a, b| b.tasa_anual_pct.total_cmp(&a.tasa_anual_pct));
    for c in &hechos {
        w.serialize(c).map_err(|e| e.to_string())?;
    }
    w.flush().map_err(|e| e.to_string())?;
    println!("{} territorios analizados, {omitidos} omitidos por serie insuficiente -> {}", hechos.len(), salida.display());
    for c in hechos.iter().take(5) {
        println!("  {}  {:+.1} % anual  {:.0} -> {:.0}  proyección {}: {:.0} ± {:.0} (confianza {})",
            c.codigo, c.tasa_anual_pct, c.edificaciones_inicial, c.edificaciones_final, c.anio_proyectado, c.proyeccion, c.proyeccion_margen, c.confianza);
    }
    Ok(ExitCode::SUCCESS)
}

fn recibir(catalogo: &Catalogo, memoria: &Memoria, ruta_sol: &Path, entrada: &Path, salida: &Path) -> Result<ExitCode, String> {
    let sol = Solicitud::cargar(ruta_sol)?;
    let tabla = faro_homologa::lectura::leer(entrada, sol.hoja.as_deref(), &sol.nombres_esperados())?;
    let sal = std::env::var("FARO_SAL").ok();
    let rec = faro_homologa::recepcion::recibir(&sol, &tabla, &Homologador::new(catalogo, memoria), sal.as_deref())?;

    std::fs::create_dir_all(salida).map_err(|e| e.to_string())?;
    let mut w = csv::Writer::from_path(salida.join("homologado.csv")).map_err(|e| e.to_string())?;
    w.write_record(&rec.encabezados).map_err(|e| e.to_string())?;
    for f in &rec.filas {
        w.write_record(f).map_err(|e| e.to_string())?;
    }
    w.flush().map_err(|e| e.to_string())?;

    let mut wp = csv::Writer::from_path(salida.join("pendientes.csv")).map_err(|e| e.to_string())?;
    wp.write_record(["columna", "contexto", "valor", "veces", "estado", "candidatos"]).map_err(|e| e.to_string())?;
    for p in &rec.pendientes {
        let cands = p.resultado.candidatos.iter().map(|c| format!("{} {} ({:.2})", c.codigo, c.nombre, c.puntaje)).collect::<Vec<_>>().join(" | ");
        wp.write_record([&p.columna, &p.contexto, &p.valor, &p.veces.to_string(), p.resultado.estado.texto(), &cands]).map_err(|e| e.to_string())?;
    }
    wp.flush().map_err(|e| e.to_string())?;

    let resumen = serde_json::json!({
        "solicitud": sol.id, "institucion": sol.institucion, "archivo": entrada.file_name().map(|n| n.to_string_lossy().to_string()),
        "hojas": tabla.hojas, "fila_encabezado": tabla.fila_encabezado,
        "porcentaje_resuelto": rec.porcentaje_resuelto(), "equivalencias_en_memoria": memoria.len(), "recepcion": rec,
    });
    std::fs::write(salida.join("reporte.json"), serde_json::to_string_pretty(&resumen).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;

    let r = &rec.reporte;
    println!("{} · {} filas · encabezado en fila {} · estructura {}", sol.id, r.filas, tabla.fila_encabezado, if r.aceptado() { "aceptada" } else { "con problemas" });
    println!("columnas: {} reconocidas, {} no pedidas (no pasan), {} eliminadas, {} seudonimizadas",
        r.columnas_reconocidas.len(), r.columnas_sobrantes.len(), rec.columnas_eliminadas.len(), rec.columnas_seudonimizadas.len());
    for p in r.problemas.iter().take(8) {
        println!("  fila {} · {} · {}", p.fila, p.columna, p.detalle);
    }
    if r.problemas.len() + r.problemas_omitidos > 8 {
        println!("  ... y {} problema(s) más en reporte.json", r.problemas.len() + r.problemas_omitidos - 8);
    }
    for (col, estados) in &rec.homologacion {
        println!("{col}: {}", estados.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(" "));
    }
    for (col, valores) in &rec.valores_no_reconocidos {
        let mut v: Vec<_> = valores.iter().collect();
        v.sort_by(|a, b| b.1.cmp(a.1));
        println!("{col}: {} valor(es) fuera de catálogo: {}", v.len(), v.iter().take(5).map(|(k, n)| format!("{k:?}×{n}")).collect::<Vec<_>>().join(", "));
    }
    println!("lugares resueltos: {:.1} % · {} valor(es) distintos por revisar -> {}", rec.porcentaje_resuelto(), rec.pendientes.len(), salida.join("pendientes.csv").display());
    Ok(if r.aceptado() && rec.pendientes.is_empty() { ExitCode::SUCCESS } else { ExitCode::from(2) })
}
