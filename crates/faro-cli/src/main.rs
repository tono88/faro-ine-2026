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
    /// Verifica el catálogo contra el Listado de lugares poblados del Censo 2018 y carga sus lugares.
    Catalogo {
        #[command(subcommand)]
        accion: AccionCatalogo,
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
enum AccionCatalogo {
    /// Compara departamentos y municipios con el listado, código por código. No cambia nada.
    Verificar {
        /// Excel "Listado de lugares poblados" (faro fuentes descargar censo-2018-lugares-poblados).
        #[arg(long)]
        listado: PathBuf,
    },
    /// Adopta los nombres oficiales de municipio (el anterior queda como alias) y escribe
    /// lugares_poblados.csv con coordenadas. Solo procede si los códigos coinciden.
    Importar {
        #[arg(long)]
        listado: PathBuf,
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
        /// Solo recursos cuyo nombre contenga este texto.
        #[arg(long)]
        recurso: Option<String>,
        #[arg(long, default_value = "datos/crudo")]
        destino: PathBuf,
    },
    /// Descarga un archivo de otra fuente abierta por su URL, con el mismo registro de procedencia.
    Url {
        url: String,
        /// Nombre corto de la fuente (carpeta de destino).
        #[arg(long)]
        fuente: String,
        #[arg(long)]
        editor: String,
        #[arg(long)]
        licencia: String,
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
        Comando::Catalogo { accion } => catalogo_censo(&catalogo, &cli.catalogo, accion),
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
        Fuentes::Url { url, fuente, editor, licencia, destino } => {
            if !url.starts_with("https://") {
                return Err("solo se descargan URL https".into());
            }
            let archivo = destino.join(&fuente).join(faro_fuentes::procedencia::nombre_de_url(&url));
            let d = manifiesto.descargar(&fuente, &url, &archivo, &editor, &licencia)?;
            println!("{:>6}  {:>10} bytes  {}", d.estado, d.bytes, d.archivo);
        }
        Fuentes::Descargar { conjunto, formato, recurso, destino } => {
            let c = faro_fuentes::Ckan::new(portal).conjunto(&conjunto)?;
            let quiere = |f: &str| formato.is_empty() || formato.iter().any(|x| x.eq_ignore_ascii_case(f));
            let mut fallos = 0;
            let nombre_ok = |n: &str| recurso.as_deref().map_or(true, |x| n.to_lowercase().contains(&x.to_lowercase()));
            for r in c.resources.iter().filter(|r| quiere(&r.format) && nombre_ok(&r.name) && r.url.starts_with("http")) {
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

fn informar(nivel: &str, v: &faro_homologa::censo::Verificacion) {
    println!("{nivel}: {} en el listado · {} con el mismo nombre · {} con otro nombre · {} faltan · {} sobran", v.en_listado, v.iguales, v.diferencias.len(), v.faltan.len(), v.sobran.len());
    for (codigo, nombre) in &v.faltan {
        println!("  falta en el catálogo   {codigo}  {nombre}");
    }
    for (codigo, nombre) in &v.sobran {
        println!("  no está en el listado  {codigo}  {nombre}");
    }
    for d in &v.diferencias {
        let nota = if d.solo_grafia { "solo grafía" } else if d.cubierto_por_alias { "ya era alias" } else { "nombre nuevo" };
        println!("  {}  catálogo: {:<28} listado: {:<32} ({nota})", d.codigo, d.en_catalogo, d.en_listado);
    }
}

fn catalogo_censo(catalogo: &Catalogo, carpeta: &Path, accion: AccionCatalogo) -> Result<ExitCode, String> {
    use faro_homologa::censo;
    let (AccionCatalogo::Verificar { listado } | AccionCatalogo::Importar { listado }) = &accion;
    let lugares = censo::interpretar(&faro_homologa::lectura::leer_crudo(listado)?)?;
    let departamentos = censo::verificar(catalogo, &lugares, Nivel::Departamento);
    let municipios = censo::verificar(catalogo, &lugares, Nivel::Municipio);
    informar("departamentos", &departamentos);
    informar("municipios", &municipios);
    let con_coordenadas = lugares.iter().filter(|l| l.longitud.is_some()).count();
    println!("lugares poblados: {} en el listado · {con_coordenadas} con coordenadas · {} sin coordenadas", lugares.len(), lugares.len() - con_coordenadas);
    let coinciden = departamentos.codigos_coinciden() && municipios.codigos_coinciden();
    if matches!(accion, AccionCatalogo::Verificar { .. }) {
        return Ok(if coinciden && departamentos.diferencias.is_empty() && municipios.diferencias.is_empty() { ExitCode::SUCCESS } else { ExitCode::from(2) });
    }
    if !coinciden || !departamentos.diferencias.is_empty() {
        return Err("los códigos o los departamentos no coinciden con el listado: corrija el catálogo a mano antes de importar".into());
    }

    let texto = |e: csv::Error| e.to_string();
    let oficiales = censo::oficiales(&lugares, Nivel::Municipio);
    let ruta_municipios = carpeta.join("municipios.csv");
    // Mismo fin de línea que el resto del catálogo, para que el historial muestre solo lo que cambió.
    let escritor = |ruta: &Path| csv::WriterBuilder::new().terminator(csv::Terminator::CRLF).from_path(ruta);
    let mut w = escritor(&ruta_municipios).map_err(texto)?;
    w.write_record(["codigo", "nombre", "nivel", "padre", "alias"]).map_err(texto)?;
    for (codigo, oficial) in &oficiales {
        let l = catalogo.por_codigo(codigo).ok_or(format!("{codigo}: no está en el catálogo"))?;
        w.write_record([codigo.as_str(), oficial.as_str(), "municipio", l.padre.as_str(), censo::alias_al_adoptar(oficial, &l.nombre, &l.alias).as_str()]).map_err(texto)?;
    }
    w.flush().map_err(|e| e.to_string())?;

    let ruta_lugares = carpeta.join("lugares_poblados.csv");
    let mut w = escritor(&ruta_lugares).map_err(texto)?;
    w.write_record(["codigo", "nombre", "nivel", "padre", "alias", "longitud", "latitud"]).map_err(texto)?;
    let numero = |v: Option<f64>| v.map(|x| x.to_string()).unwrap_or_default();
    for l in &lugares {
        w.write_record([l.codigo.as_str(), l.nombre.as_str(), "lugar_poblado", l.municipio.as_str(), "", numero(l.longitud).as_str(), numero(l.latitud).as_str()]).map_err(texto)?;
    }
    w.flush().map_err(|e| e.to_string())?;
    println!("escrito: {} ({} municipios, {} nombres actualizados) y {} ({} lugares poblados)", ruta_municipios.display(), oficiales.len(), municipios.diferencias.len(), ruta_lugares.display(), lugares.len());
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
