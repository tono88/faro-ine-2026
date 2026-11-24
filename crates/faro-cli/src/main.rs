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
    /// Resume las proyecciones de población del INE (municipio × sexo × edad) a municipio y año.
    Poblacion {
        /// Excel "Estimaciones y proyecciones ... por municipio" (faro fuentes descargar proyecciones-de-poblacion-2015-2030).
        #[arg(long)]
        entrada: PathBuf,
        /// Diccionario de variables que acompaña al libro. Si el libro trae códigos de municipio
        /// que no existen, se usan sus nombres para ubicar el código oficial.
        #[arg(long)]
        diccionario: Option<PathBuf>,
        #[arg(long, default_value = "datos/referencia/poblacion_municipio.csv")]
        salida: PathBuf,
    },
    /// Precio de referencia por producto a partir de cotizaciones (media, mediana o moda según el caso).
    Precios {
        /// CSV: producto, precio, fuente, tipo_fuente (primaria o secundaria), unidad.
        #[arg(long)]
        cotizaciones: PathBuf,
        /// Variación (%) hasta la cual los precios se consideran parecidos y se usa la media.
        #[arg(long, default_value_t = 15.0)]
        cv_parecidos: f64,
        #[arg(long, default_value = "salida/precios_referencia.csv")]
        salida: PathBuf,
    },
    /// Red vial como grafo: lugares poblados sin camino mapeado, tramos sueltos y enlaces probables.
    Vial {
        /// Zip de carreteras de OpenStreetMap en Shapefile (exportación de HOT en HDX: hotosm_gtm_roads).
        #[arg(long)]
        carreteras: PathBuf,
        /// Un lugar tiene camino si hay una vía a esta distancia (metros) o menos de su centroide.
        #[arg(long, default_value_t = 500.0)]
        cerca_m: f64,
        /// Longitud máxima (km) de un enlace propuesto.
        #[arg(long, default_value_t = 5.0)]
        max_enlace_km: f64,
        /// Incluir senderos y vías peatonales (por defecto solo vías para vehículos).
        #[arg(long)]
        con_senderos: bool,
        #[arg(long, default_value = "datos/referencia")]
        salida: PathBuf,
    },
    /// Reparte operadores de campo entre puestos con la menor distancia total (línea recta).
    Asignar {
        /// CSV: operador, lugar (código de lugar poblado donde vive) o longitud y latitud.
        #[arg(long)]
        operadores: PathBuf,
        /// CSV: puesto, lugar (código de lugar poblado) o longitud y latitud; operadores (cuántos necesita, 1 si falta).
        #[arg(long)]
        puestos: PathBuf,
        #[arg(long, default_value = "salida/asignacion.csv")]
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
        Comando::Precios { cotizaciones, cv_parecidos, salida } => {
            use faro_territorio::precios;
            let p = precios::Parametros { cv_parecidos_pct: cv_parecidos, ..Default::default() };
            let archivo = std::fs::File::open(&cotizaciones).map_err(|e| format!("{}: {e}", cotizaciones.display()))?;
            let resultados = precios::analizar_csv(archivo, &p).map_err(|e| format!("{}: {e}", cotizaciones.display()))?;
            if let Some(dir) = salida.parent() {
                std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            }
            let mut w = csv::Writer::from_path(&salida).map_err(|e| e.to_string())?;
            w.write_record(["producto", "unidad", "cotizaciones", "minimo", "maximo", "media", "mediana", "moda", "variacion_pct", "extremos", "fuentes_primarias_pct", "medida", "precio_referencia", "notas"]).map_err(|e| e.to_string())?;
            let q = |x: f64| format!("{x:.2}");
            for r in &resultados {
                w.write_record([r.producto.clone(), r.unidad.clone(), r.cotizaciones.to_string(), q(r.minimo), q(r.maximo), q(r.media), q(r.mediana), r.moda.map(q).unwrap_or_default(), format!("{:.1}", r.coeficiente_variacion_pct),
                    r.extremos.iter().map(|(f, x)| format!("{f}: {x:.2}")).collect::<Vec<_>>().join(" | "), format!("{:.0}", r.fuentes_primarias_pct), r.medida.clone(), r.precio_referencia.map(q).unwrap_or_default(), r.notas.join(" ")]).map_err(|e| e.to_string())?;
                match r.precio_referencia {
                    Some(x) => println!("{}: Q{x:.2} por {} ({}, {} cotizaciones)", r.producto, if r.unidad.is_empty() { "unidad" } else { &r.unidad }, r.medida, r.cotizaciones),
                    None => println!("{}: sin precio de referencia", r.producto),
                }
                for n in &r.notas {
                    println!("    {n}");
                }
            }
            w.flush().map_err(|e| e.to_string())?;
            println!("{} producto(s) -> {}", resultados.len(), salida.display());
            return Ok(ExitCode::SUCCESS);
        }
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
        Comando::Asignar { operadores, puestos, salida } => asignar(&catalogo, &operadores, &puestos, &salida),
        Comando::Vial { carreteras, cerca_m, max_enlace_km, con_senderos, salida } => vial(&catalogo, &carreteras, faro_territorio::vial::Parametros { cerca_m, max_enlace_km }, con_senderos, &salida),
        Comando::Poblacion { entrada, diccionario, salida } => {
            let mut filas = faro_territorio::poblacion::resumir(&faro_homologa::lectura::leer_crudo(&entrada)?)?;
            // Cada municipio del libro debe existir en el catálogo: si no, el cruce por código no vale.
            let es_municipio = |m: &str| catalogo.por_codigo(m).is_some_and(|l| l.nivel == Nivel::Municipio);
            let ajenos: std::collections::BTreeSet<String> = filas.iter().map(|p| p.municipio.clone()).filter(|m| !es_municipio(m)).collect();
            if !ajenos.is_empty() {
                let ruta = diccionario.ok_or(format!("el libro trae {} códigos de municipio que no existen ({}...): indique --diccionario para ubicarlos por nombre", ajenos.len(), ajenos.iter().next().cloned().unwrap_or_default()))?;
                let nombres = faro_territorio::poblacion::diccionario_municipios(&faro_homologa::lectura::leer_crudo(&ruta)?);
                let h = Homologador::new(&catalogo, &memoria);
                let presentes: std::collections::BTreeSet<String> = filas.iter().map(|p| p.municipio.clone()).collect();
                let mut equivalencias = std::collections::BTreeMap::new();
                println!("{} códigos de municipio del libro no existen en el catálogo; se ubican por el nombre que les da su diccionario:", ajenos.len());
                for codigo in &ajenos {
                    let nombre = nombres.get(codigo).ok_or(format!("{codigo}: el diccionario no lo nombra"))?;
                    // Solo coincidencia exacta y única dentro del mismo departamento, y sin pisar otro código del libro.
                    let r = h.homologar(nombre, Nivel::Municipio, Some(&codigo[..2]));
                    let oficial = r.codigo.filter(|_| r.estado.resuelto()).ok_or(format!("{codigo} ({nombre}): no coincide con un único municipio del departamento {}", &codigo[..2]))?;
                    if presentes.contains(&oficial) || equivalencias.values().any(|v| *v == oficial) {
                        return Err(format!("{codigo} ({nombre}) correspondería a {oficial}, que ya tiene datos propios en el libro"));
                    }
                    println!("  {codigo} {nombre} -> {oficial}");
                    equivalencias.insert(codigo.clone(), oficial);
                }
                faro_territorio::poblacion::recodificar(&mut filas, &equivalencias);
                filas.sort_by(|a, b| (&a.municipio, a.anio).cmp(&(&b.municipio, b.anio)));
            }
            if let Some(dir) = salida.parent() {
                std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            }
            let mut w = csv::WriterBuilder::new().terminator(csv::Terminator::CRLF).from_path(&salida).map_err(|e| e.to_string())?;
            for p in &filas {
                w.serialize(p).map_err(|e| e.to_string())?;
            }
            w.flush().map_err(|e| e.to_string())?;
            let p = faro_territorio::poblacion::Poblacion::new(filas);
            let (desde, hasta) = p.anios().ok_or("el libro no tiene datos")?;
            println!("{} municipios, {desde}–{hasta} -> {}", p.municipios().len(), salida.display());
            for anio in [desde, 2018, hasta] {
                if let Some(n) = p.total("", anio) {
                    println!("  población del país en {anio}: {n}");
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        Comando::Fuentes { .. } | Comando::Crecimiento { .. } | Comando::Aire { .. } | Comando::Precios { .. } => unreachable!("atendidos arriba"),
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

/// Centroide de un lugar poblado del catálogo, para ubicar operadores y puestos.
fn centroide(catalogo: &Catalogo, codigo: &str) -> Result<faro_territorio::asignacion::Coordenada, String> {
    faro_homologa::censo::centroide(catalogo, codigo).map(|(longitud, latitud)| faro_territorio::asignacion::Coordenada { longitud, latitud })
}

fn asignar(catalogo: &Catalogo, ruta_operadores: &Path, ruta_puestos: &Path, salida: &Path) -> Result<ExitCode, String> {
    use faro_territorio::asignacion::{self, Operador, Puesto};
    let abrir = |ruta: &Path| std::fs::File::open(ruta).map_err(|e| format!("{}: {e}", ruta.display()));
    let donde = |codigo: &str| centroide(catalogo, codigo);
    let operadores: Vec<Operador> = asignacion::leer_operadores(abrir(ruta_operadores)?, &donde).map_err(|e| format!("{}: {e}", ruta_operadores.display()))?;
    let puestos: Vec<Puesto> = asignacion::leer_puestos(abrir(ruta_puestos)?, &donde).map_err(|e| format!("{}: {e}", ruta_puestos.display()))?;
    let a = asignacion::asignar(&operadores, &puestos)?;

    if let Some(dir) = salida.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let mut w = csv::Writer::from_path(salida).map_err(|e| e.to_string())?;
    w.write_record(["operador", "puesto", "km"]).map_err(|e| e.to_string())?;
    for x in &a.asignados {
        w.write_record([x.operador.as_str(), x.puesto.as_str(), format!("{:.1}", x.km).as_str()]).map_err(|e| e.to_string())?;
    }
    for o in &a.sin_puesto {
        w.write_record([o.as_str(), "", ""]).map_err(|e| e.to_string())?;
    }
    w.flush().map_err(|e| e.to_string())?;

    let (r, ref_) = (&a.resumen, &a.en_orden_de_lista);
    println!("{} operadores, {} puestos ({} plazas) -> {}", operadores.len(), puestos.len(), puestos.iter().map(|p| p.operadores).sum::<usize>(), salida.display());
    println!("asignados: {} · total {:.1} km · promedio {:.1} km · el más lejano {:.1} km (línea recta)", r.asignados, r.total_km, r.promedio_km, r.maximo_km);
    if ref_.total_km > 0.0 {
        println!("repartiendo en el orden de las listas: total {:.1} km · promedio {:.1} km · el más lejano {:.1} km ({:.0} % más)", ref_.total_km, ref_.promedio_km, ref_.maximo_km, (ref_.total_km / r.total_km.max(f64::MIN_POSITIVE) - 1.0) * 100.0);
    }
    if !a.sin_puesto.is_empty() {
        println!("sin puesto ({}): {}", a.sin_puesto.len(), a.sin_puesto.join(", "));
    }
    for (puesto, n) in &a.sin_cubrir {
        println!("plazas sin cubrir: {puesto} ({n})");
    }
    Ok(ExitCode::SUCCESS)
}

/// Tipos de vía de OpenStreetMap que no son un camino en uso.
const NO_SON_CAMINO: &[&str] = &["proposed", "construction", "raceway", "corridor", "escape", "passing_place", "platform", "rest_area"];
/// Vías que no admiten vehículos.
const SENDEROS: &[&str] = &["path", "footway", "steps", "pedestrian", "cycleway", "bridleway"];

fn vial(catalogo: &Catalogo, carreteras: &Path, p: faro_territorio::vial::Parametros, con_senderos: bool, salida: &Path) -> Result<ExitCode, String> {
    use faro_territorio::asignacion::Coordenada;
    use faro_territorio::vial::{self, Conexion};
    let en = |e: String| format!("{}: {e}", carreteras.display());
    let mut zip = zip::ZipArchive::new(std::fs::File::open(carreteras).map_err(|e| en(e.to_string()))?).map_err(|e| en(e.to_string()))?;
    let nombre = |zip: &zip::ZipArchive<std::fs::File>, fin: &str| zip.file_names().find(|n| n.ends_with(fin)).map(str::to_string).ok_or(en(format!("el zip no trae un archivo *{fin}")));
    let (n_shp, n_dbf) = (nombre(&zip, "lines.shp")?, nombre(&zip, "lines.dbf")?);
    let tipos = faro_territorio::shp::leer_campo(zip.by_name(&n_dbf).map_err(|e| en(e.to_string()))?, "highway").map_err(en)?;
    let lineas = faro_territorio::shp::leer_lineas(zip.by_name(&n_shp).map_err(|e| en(e.to_string()))?).map_err(en)?;
    if tipos.len() != lineas.len() {
        return Err(en(format!("la tabla tiene {} registros y la geometría {}", tipos.len(), lineas.len())));
    }
    let sirve = |t: &str| !t.is_empty() && !NO_SON_CAMINO.contains(&t) && (con_senderos || !SENDEROS.contains(&t));
    let usadas = tipos.iter().filter(|t| sirve(t)).count();
    let red = vial::Red::construir(lineas.iter().zip(&tipos).filter(|(_, t)| sirve(t)).flat_map(|(partes, _)| partes.iter().map(Vec::as_slice)));
    println!("{} vías leídas, {usadas} usadas ({}) · {:.0} km · {} tramos no unidos entre sí · red principal: {:.0} km ({:.1} %)",
        tipos.len(), if con_senderos { "con senderos" } else { "solo para vehículos" }, red.km_total(), red.componentes(), red.km_principal(), 100.0 * red.km_principal() / red.km_total().max(1e-9));

    let lugares: Vec<(String, Coordenada)> = catalogo.de_nivel(Nivel::LugarPoblado, None).filter_map(|l| Some((l.codigo.clone(), Coordenada { longitud: l.longitud?, latitud: l.latitud? }))).collect();
    let clasificados = vial::clasificar(&red, &lugares, &p);
    let enlaces = vial::enlaces_probables(&red, &clasificados, &p);
    let cuenta = |c: Conexion| clasificados.iter().filter(|l| l.conexion == c).count();
    println!("{} lugares poblados con coordenadas: {} conectados a la red principal, {} en un tramo suelto, {} sin camino mapeado a {} m",
        lugares.len(), cuenta(Conexion::Conectado), cuenta(Conexion::TramoSuelto), cuenta(Conexion::SinCamino), p.cerca_m);
    println!("{} enlaces probables de hasta {} km; suman {:.1} km y unirían {} lugares", enlaces.len(), p.max_enlace_km, enlaces.iter().map(|e| e.km).sum::<f64>(), enlaces.iter().map(|e| e.lugares.len()).sum::<usize>());

    std::fs::create_dir_all(salida).map_err(|e| e.to_string())?;
    let texto = |e: csv::Error| e.to_string();
    let n = |x: Option<f64>, d: usize| x.map(|v| format!("{v:.d$}")).unwrap_or_default();
    // Solo los lugares que no están conectados: los demás no necesitan revisión.
    let mut w = csv::WriterBuilder::new().terminator(csv::Terminator::CRLF).from_path(salida.join("red_vial_lugares.csv")).map_err(texto)?;
    w.write_record(["codigo", "conexion", "distancia_m", "via_longitud", "via_latitud", "tramo_km"]).map_err(texto)?;
    for l in clasificados.iter().filter(|l| l.conexion != Conexion::Conectado) {
        w.write_record([l.codigo.clone(), l.conexion.texto().to_string(), n(l.distancia_m, 0), n(l.via_longitud, 5), n(l.via_latitud, 5), n(l.tramo_km, 2)]).map_err(texto)?;
    }
    w.flush().map_err(|e| e.to_string())?;
    let mut w = csv::WriterBuilder::new().terminator(csv::Terminator::CRLF).from_path(salida.join("red_vial_enlaces.csv")).map_err(texto)?;
    w.write_record(["desde_longitud", "desde_latitud", "hasta_longitud", "hasta_latitud", "km", "tramo_km", "lugares"]).map_err(texto)?;
    for e in &enlaces {
        w.write_record([format!("{:.5}", e.desde_longitud), format!("{:.5}", e.desde_latitud), format!("{:.5}", e.hasta_longitud), format!("{:.5}", e.hasta_latitud), format!("{:.3}", e.km), format!("{:.2}", e.tramo_km), e.lugares.join("|")]).map_err(texto)?;
    }
    w.flush().map_err(|e| e.to_string())?;
    let resumen = serde_json::json!({
        "vias_leidas": tipos.len(), "vias_usadas": usadas, "con_senderos": con_senderos, "km": red.km_total(), "tramos": red.componentes(), "km_red_principal": red.km_principal(),
        "cerca_m": p.cerca_m, "max_enlace_km": p.max_enlace_km, "lugares": lugares.len(), "conectados": cuenta(Conexion::Conectado), "tramo_suelto": cuenta(Conexion::TramoSuelto), "sin_camino": cuenta(Conexion::SinCamino),
        "enlaces": enlaces.len(), "enlaces_km": enlaces.iter().map(|e| e.km).sum::<f64>(),
    });
    std::fs::write(salida.join("red_vial_resumen.json"), serde_json::to_string_pretty(&resumen).map_err(|e| e.to_string())? + "\n").map_err(|e| e.to_string())?;
    println!("escrito en {}: red_vial_lugares.csv, red_vial_enlaces.csv, red_vial_resumen.json", salida.display());
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
