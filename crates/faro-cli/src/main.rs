//! `faro`: recepción y homologación de envíos de registros administrativos.
use clap::{Parser, Subcommand};
use faro_homologa::solicitud::Solicitud;
use faro_homologa::{Catalogo, Estado, Homologador, Memoria, Nivel, Resultado};
use std::collections::{BTreeMap, HashMap};
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
    let catalogo = Catalogo::cargar_carpeta(&cli.catalogo)?;
    let mut memoria = Memoria::abrir(&cli.memoria)?;
    match cli.comando {
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
    }
}

/// Una columna a homologar: índice propio, nivel y columna de contexto.
struct Geo {
    nombre: String,
    indice: usize,
    nivel: Nivel,
    contexto: Option<String>,
}

fn recibir(catalogo: &Catalogo, memoria: &Memoria, ruta_sol: &Path, entrada: &Path, salida: &Path) -> Result<ExitCode, String> {
    let sol = Solicitud::cargar(ruta_sol)?;
    let mut lector = csv::ReaderBuilder::new().flexible(true).from_path(entrada).map_err(|e| format!("{}: {e}", entrada.display()))?;
    let encabezados: Vec<String> = lector.headers().map_err(|e| e.to_string())?.iter().map(str::to_string).collect();
    let filas: Vec<Vec<String>> = lector
        .records()
        .map(|r| r.map(|r| r.iter().map(str::to_string).collect()).map_err(|e| e.to_string()))
        .collect::<Result<_, _>>()?;

    let reporte = sol.validar(&encabezados, filas.iter().cloned());
    let (mapa, _) = sol.mapear(&encabezados);

    // Columnas geográficas; primero las que no dependen de otra (departamento antes que municipio).
    let mut geos: Vec<Geo> = vec![];
    for c in &sol.columnas {
        if let (Some(nivel), Some(&indice)) = (&c.homologar, mapa.get(&c.nombre)) {
            geos.push(Geo { nombre: c.nombre.clone(), indice, nivel: nivel.parse()?, contexto: c.contexto.clone() });
        }
    }
    let mut orden: Vec<usize> = (0..geos.len()).collect();
    orden.sort_by_key(|&i| geos[i].contexto.is_some());

    let h = Homologador::new(catalogo, memoria);
    let mut cache: HashMap<(String, String, String), Resultado> = HashMap::new();
    // (columna, contexto, valor) -> (veces, resultado)
    let mut pendientes: BTreeMap<(String, String, String), (usize, Resultado)> = BTreeMap::new();
    let mut conteo: BTreeMap<&'static str, usize> = BTreeMap::new();

    std::fs::create_dir_all(salida).map_err(|e| e.to_string())?;
    let mut w = csv::Writer::from_path(salida.join("homologado.csv")).map_err(|e| e.to_string())?;
    let mut cab = encabezados.clone();
    for g in &geos {
        cab.push(format!("{}_codigo", g.nombre));
        cab.push(format!("{}_estado", g.nombre));
    }
    w.write_record(&cab).map_err(|e| e.to_string())?;

    for fila in &filas {
        let mut codigos: HashMap<&str, Option<String>> = HashMap::new();
        let mut extra = vec![(String::new(), String::new()); geos.len()];
        for &i in &orden {
            let g = &geos[i];
            let valor = fila.get(g.indice).cloned().unwrap_or_default();
            // Sin código del nivel superior no se restringe la búsqueda: saldrá ambiguo si lo es.
            let ctx = g.contexto.as_deref().and_then(|c| codigos.get(c).cloned().flatten());
            let llave = (g.nombre.clone(), ctx.clone().unwrap_or_default(), valor.clone());
            let r = cache.entry(llave.clone()).or_insert_with(|| h.homologar(&valor, g.nivel, ctx.as_deref())).clone();
            *conteo.entry(r.estado.texto()).or_default() += 1;
            if !r.estado.resuelto() && r.estado != Estado::Vacio {
                pendientes.entry(llave).or_insert((0, r.clone())).0 += 1;
            }
            codigos.insert(&g.nombre, r.codigo.clone());
            extra[i] = (r.codigo.unwrap_or_default(), r.estado.texto().to_string());
        }
        let mut registro = fila.clone();
        registro.resize(encabezados.len(), String::new());
        for (codigo, estado) in extra {
            registro.push(codigo);
            registro.push(estado);
        }
        w.write_record(&registro).map_err(|e| e.to_string())?;
    }
    w.flush().map_err(|e| e.to_string())?;

    let mut wp = csv::Writer::from_path(salida.join("pendientes.csv")).map_err(|e| e.to_string())?;
    wp.write_record(["columna", "contexto", "valor", "veces", "estado", "candidatos"]).map_err(|e| e.to_string())?;
    for ((col, ctx, valor), (veces, r)) in &pendientes {
        let cands = r.candidatos.iter().map(|c| format!("{} {} ({:.2})", c.codigo, c.nombre, c.puntaje)).collect::<Vec<_>>().join(" | ");
        wp.write_record([col, ctx, valor, &veces.to_string(), r.estado.texto(), &cands]).map_err(|e| e.to_string())?;
    }
    wp.flush().map_err(|e| e.to_string())?;

    let resumen = serde_json::json!({
        "solicitud": sol.id,
        "institucion": sol.institucion,
        "archivo": entrada.display().to_string(),
        "filas": reporte.filas,
        "estructura_aceptada": reporte.aceptado(),
        "validacion": reporte,
        "homologacion": conteo,
        "valores_pendientes": pendientes.len(),
        "equivalencias_en_memoria": memoria.len(),
    });
    std::fs::write(salida.join("reporte.json"), serde_json::to_string_pretty(&resumen).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;

    println!("{} · {} filas · estructura {}", sol.id, reporte.filas, if reporte.aceptado() { "aceptada" } else { "con problemas" });
    for p in reporte.problemas.iter().take(10) {
        println!("  fila {} · {} · {}", p.fila, p.columna, p.detalle);
    }
    println!("homologación: {}", conteo.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(" "));
    println!("{} valor(es) distintos por revisar -> {}", pendientes.len(), salida.join("pendientes.csv").display());
    Ok(if reporte.aceptado() && pendientes.is_empty() { ExitCode::SUCCESS } else { ExitCode::from(2) })
}
