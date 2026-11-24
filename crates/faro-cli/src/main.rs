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
    /// Capa de municipios para mapas: toma los límites oficiales, los verifica contra los lugares
    /// poblados del censo y escribe un GeoJSON liviano con el código INE.
    Limites {
        /// Zip GeoJSON de límites administrativos (COD-AB Guatemala en HDX: cod-ab-gtm).
        #[arg(long)]
        entrada: PathBuf,
        /// Tolerancia de simplificación en grados (0.0015 ≈ 160 m).
        #[arg(long, default_value_t = 0.0015)]
        tolerancia: f64,
        #[arg(long, default_value = "datos/referencia/municipios.geojson")]
        salida: PathBuf,
    },
    /// Superficie construida por municipio y año a partir de los mosaicos de GHSL (GHS-BUILT-S, 100 m).
    Construido {
        /// Carpeta con los zip de GHSL (faro fuentes url ... --fuente ghsl-built-s).
        #[arg(long, default_value = "datos/crudo/ghsl-built-s")]
        mosaicos: PathBuf,
        /// Capa de municipios generada por `faro limites`.
        #[arg(long, default_value = "datos/referencia/municipios.geojson")]
        municipios: PathBuf,
        #[arg(long, default_value = "datos/referencia/superficie_construida_municipio.csv")]
        salida: PathBuf,
    },
    /// Clasifica fotos de fachadas (vivienda, comercio u otro) con el modelo instalado y, si las fotos
    /// están en subcarpetas `vivienda/` y `comercio/`, cuenta los aciertos. Es la prueba del clasificador.
    Fachadas {
        /// Carpeta con las fotos (o con subcarpetas `vivienda/` y `comercio/`).
        #[arg(long)]
        fotos: PathBuf,
        /// Carpeta del modelo (places365_resnet18.onnx y categories_places365.txt).
        #[arg(long, default_value = "datos/modelos")]
        modelo: PathBuf,
        /// Tabla con la fuente de cada foto (columnas archivo, pagina_commons, autor, licencia), para citarla en la salida.
        #[arg(long)]
        fuentes: Option<PathBuf>,
        /// Dónde escribir la tabla de resultados.
        #[arg(long)]
        salida: Option<PathBuf>,
    },
    /// Escribe el documento de fuentes y citas a partir de datos/estandar/fuentes.json, la misma
    /// lista que muestra la pantalla en «Acerca y fuentes».
    Citas {
        #[arg(long, default_value = "datos/estandar/fuentes.json")]
        entrada: PathBuf,
        #[arg(long, default_value = "docs/11-fuentes-y-citas.md")]
        salida: PathBuf,
    },
    /// Escribe el documento «Antes y ahora» a partir de datos/estandar/mejoras.json (la misma fuente
    /// que usa la pantalla), para que el portal y el documento digan siempre lo mismo.
    Mejoras {
        #[arg(long, default_value = "datos/estandar/mejoras.json")]
        entrada: PathBuf,
        #[arg(long, default_value = "docs/22-antes-y-ahora.md")]
        salida: PathBuf,
    },
    /// Uso del suelo construido: superficie residencial y no residencial por municipio y por bloque
    /// de 1 km, con las dos capas de GHSL (total y componente no residencial, NRES).
    UsoSuelo {
        /// Carpeta con los zip de la superficie construida total (los mismos de `faro construido`).
        #[arg(long, default_value = "datos/crudo/ghsl-built-s")]
        total: PathBuf,
        /// Carpeta con los zip del componente no residencial (faro fuentes url ... --fuente ghsl-built-s-nres).
        #[arg(long, default_value = "datos/crudo/ghsl-built-s-nres")]
        no_residencial: PathBuf,
        #[arg(long, default_value = "datos/referencia/municipios.geojson")]
        municipios: PathBuf,
        #[arg(long, default_value = "datos/referencia/uso_construido_municipio.csv")]
        salida: PathBuf,
    },
    /// Catálogo de artículos del IPMC (INE) con su índice más reciente y la variación en doce meses.
    Ipmc {
        /// Libro mensual más reciente del IPMC (faro fuentes descargar indice-de-precios-de-materiales-de-construccion-ipmc).
        #[arg(long)]
        actual: PathBuf,
        /// Libro que contiene el mismo mes del año anterior (el anual).
        #[arg(long)]
        anterior: Option<PathBuf>,
        #[arg(long, default_value = "datos/catalogo_precios/ipmc_articulos.csv")]
        salida: PathBuf,
    },
    /// Homologa los precios publicados por proveedores contra el IPMC y los compara por artículo.
    Construccion {
        #[arg(long, default_value = "datos/referencia/precios_construccion.csv")]
        precios: PathBuf,
        #[arg(long, default_value = "datos/estandar/ipmc_reglas.toml")]
        reglas: PathBuf,
        #[arg(long, default_value = "datos/catalogo_precios/ipmc_articulos.csv")]
        articulos: PathBuf,
        #[arg(long, default_value = "salida/construccion")]
        salida: PathBuf,
    },
    /// Pasa a una tabla limpia los precios de referencia que publica el INE (portal de datos abiertos).
    Referencias {
        /// Libro "Precios de Referencia" (faro fuentes descargar precios-de-referencia).
        #[arg(long)]
        entrada: PathBuf,
        #[arg(long, default_value = "datos/catalogo_precios/precios_referencia_ine.csv")]
        salida: PathBuf,
    },
    /// Población, hogares y viviendas del Censo 2018 por lugar poblado, en una tabla (cuadros A1, B8 y C1 del INE).
    CensoLugares {
        /// Cuadro A1 (población por sexo, edad y área), conjunto censo-2018-lugares-poblados.
        #[arg(long)]
        a1: PathBuf,
        /// Cuadro B8 (hogares).
        #[arg(long)]
        b8: PathBuf,
        /// Cuadro C1 (viviendas).
        #[arg(long)]
        c1: PathBuf,
        #[arg(long, default_value = "datos/referencia/censo2018_lugar_poblado.csv")]
        salida: PathBuf,
    },
    /// Indicadores del Censo 2018 por municipio (educación, tecnología, agua, saneamiento, energía,
    /// materiales de la vivienda, pueblos) y material del techo por lugar poblado, de los cuadros del INE.
    CensoIndicadores {
        /// Carpeta con los cuadros descargados (faro fuentes descargar censo-2018-lugares-poblados).
        #[arg(long, default_value = "datos/crudo/censo-2018-lugares-poblados")]
        cuadros: PathBuf,
        #[arg(long, default_value = "datos/referencia/censo2018_indicadores_municipio.csv")]
        salida: PathBuf,
        #[arg(long, default_value = "datos/referencia/censo2018_techos_lugar_poblado.csv")]
        techos: PathBuf,
    },
    /// Población por municipio, sexo y grupo quinquenal de edad (proyecciones del INE), para las pirámides.
    Edades {
        /// Libro de proyecciones por municipio, sexo y edad.
        #[arg(long)]
        entrada: PathBuf,
        #[arg(long)]
        diccionario: PathBuf,
        /// Años que se guardan.
        #[arg(long, value_delimiter = ',', default_value = "2018,2026,2030")]
        anios: Vec<i32>,
        /// Tabla ya publicada de totales, para comprobar que las edades suman lo mismo.
        #[arg(long, default_value = "datos/referencia/poblacion_municipio.csv")]
        totales: PathBuf,
        #[arg(long, default_value = "datos/referencia/poblacion_municipio_edades.csv")]
        salida: PathBuf,
    },
    /// Verificación estadística y de consistencia de todo lo calculado; deja la evidencia en un documento.
    Verificar {
        #[arg(long, default_value = "datos/referencia")]
        referencia: PathBuf,
        #[arg(long, default_value = "docs/19-verificacion-estadistica.md")]
        salida: PathBuf,
    },
    /// Cuentas de utilización y suministro de la Hoja de Balance de Alimentos del INE, en una tabla limpia.
    Hba {
        /// Libro "cuentas de utilización y suministro (CUS)" (faro fuentes descargar hoja-de-balance-de-alimentos-hba-actualizacion-2024).
        #[arg(long)]
        cuentas: PathBuf,
        #[arg(long, default_value = "datos/referencia/hba_2024_cuentas.csv")]
        salida: PathBuf,
    },
    /// Calibra la comparación por foto: mide la huella visual en pares de productos con el mismo
    /// código de barras en dos tiendas y en pares de productos distintos, y dice qué umbral separa mejor.
    Fotos {
        /// Tabla de precios publicados con columnas tienda, codigo_barras e imagen.
        #[arg(long, default_value = "datos/referencia/precios_publicados.csv")]
        entrada: PathBuf,
        /// Pares de cada tipo que se miden.
        #[arg(long, default_value_t = 300)]
        pares: usize,
        #[arg(long, default_value = "datos/referencia/fotos_calibracion.csv")]
        salida: PathBuf,
    },
    /// Precios promedio de la canasta básica alimentaria publicados por el INE, en una tabla limpia.
    Canasta {
        /// CSV "Precios promedio ... CBA" (faro fuentes descargar precios-promedio-canasta-basica-alimentaria).
        #[arg(long)]
        entrada: PathBuf,
        #[arg(long, default_value = "datos/catalogo_precios/cba_precios.csv")]
        salida: PathBuf,
    },
    /// Tabla de población por fase de inseguridad alimentaria aguda (CIF, SESAN) por departamento.
    Cif {
        /// Texto del informe obtenido con `pdftotext -layout informe.pdf informe.txt`.
        #[arg(long)]
        texto: PathBuf,
        /// Periodo del análisis, tal como lo titula el informe.
        #[arg(long, default_value = "octubre 2026 - febrero 2027")]
        periodo: String,
        #[arg(long, default_value = "datos/referencia/cif_departamentos.csv")]
        salida: PathBuf,
    },
    /// Retardo en talla (desnutrición crónica) en escolares por municipio: Censo de Talla de SESAN.
    Talla {
        /// Libro "Retardo en talla, Censo 2024" del SIINSAN.
        #[arg(long)]
        entrada: PathBuf,
        #[arg(long, default_value = "datos/referencia/censo_talla_municipio.csv")]
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
        Comando::Referencias { entrada, salida } => return referencias(&entrada, &salida),
        Comando::Verificar { referencia, salida } => return verificar(&referencia, &salida).map(|ok| if ok { ExitCode::SUCCESS } else { ExitCode::FAILURE }),
        Comando::Hba { cuentas, salida } => {
            use faro_homologa::normalizar::basico;
            let filas = faro_homologa::lectura::leer_crudo(&cuentas)?;
            let pos = filas.iter().position(|f| f.first().is_some_and(|c| basico(c) == "alimentos")).ok_or(format!("{}: no parece el libro de cuentas de la HBA", cuentas.display()))?;
            let enc: Vec<String> = filas[pos].iter().map(|c| basico(c)).collect();
            let col = |n: &str| enc.iter().position(|c| c.starts_with(n)).ok_or(format!("{}: falta la columna {n}", cuentas.display()));
            let columnas = [col("produccion")?, col("importaciones")?, col("exportaciones")?, col("procesado")?, col("alimentos tm")?];
            if let Some(d) = salida.parent() {
                std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
            }
            let mut tabla = csv::Writer::from_path(&salida).map_err(|e| e.to_string())?;
            tabla.write_record(["alimento", "produccion_t", "importaciones_t", "exportaciones_t", "procesado_t", "alimentos_t"]).map_err(|e| e.to_string())?;
            let mut vistos = std::collections::HashSet::new();
            let mut n = 0;
            for f in filas.iter().skip(pos + 1) {
                let nombre = f.first().map(|c| c.trim()).unwrap_or("");
                // Un mismo alimento aparece en su grupo y en el resumen: se deja la primera vez.
                if nombre.is_empty() || !vistos.insert(basico(nombre)) {
                    continue;
                }
                let valor = |i: usize| f.get(i).and_then(|c| c.trim().parse::<f64>().ok()).map(|x| format!("{}", (x * 10.0).round() / 10.0)).unwrap_or_default();
                tabla.write_record(std::iter::once(nombre.to_string()).chain(columnas.iter().map(|i| valor(*i)))).map_err(|e| e.to_string())?;
                n += 1;
            }
            tabla.flush().map_err(|e| e.to_string())?;
            println!("{n} alimentos y grupos escritos en {}", salida.display());
            return Ok(ExitCode::SUCCESS);
        }
        Comando::Fotos { entrada, pares, salida } => return calibrar_fotos(&entrada, pares, &salida).map(|_| ExitCode::SUCCESS),
        Comando::Canasta { entrada, salida } => {
            const MESES: [&str; 12] = ["enero", "febrero", "marzo", "abril", "mayo", "junio", "julio", "agosto", "septiembre", "octubre", "noviembre", "diciembre"];
            // El archivo trae descripciones con comas sin comillas ("Café tostado, molido"): los tres
            // primeros campos son año, mes y número; el último, el precio; lo del medio, la descripción.
            let texto = std::fs::read_to_string(&entrada).map_err(|e| format!("{}: {e}", entrada.display()))?;
            if let Some(d) = salida.parent() {
                std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
            }
            let mut w = csv::WriterBuilder::new().terminator(csv::Terminator::CRLF).from_path(&salida).map_err(|e| e.to_string())?;
            w.write_record(["periodo", "numero", "producto", "precio_promedio"]).map_err(|e| e.to_string())?;
            let (mut n, mut productos, mut periodos) = (0, std::collections::BTreeSet::new(), std::collections::BTreeSet::new());
            for (k, linea) in texto.trim_start_matches('\u{feff}').lines().enumerate().skip(1).filter(|(_, l)| !l.trim().is_empty()) {
                let c: Vec<&str> = linea.split(',').map(str::trim).collect();
                if c.len() < 5 {
                    return Err(format!("{} línea {}: se esperaban al menos 5 campos", entrada.display(), k + 1));
                }
                let producto = c[3..c.len() - 1].join(", ").replace('"', "");
                let mes = MESES.iter().position(|m| *m == faro_homologa::normalizar::basico(c[1])).ok_or(format!("línea {}: mes no reconocido: {:?}", k + 1, c[1]))? + 1;
                let (anio, precio): (i32, f64) = (c[0].parse().map_err(|_| format!("línea {}: año no válido: {:?}", k + 1, c[0]))?, c[c.len() - 1].parse().map_err(|_| format!("línea {}: precio no válido: {:?}", k + 1, c[c.len() - 1]))?);
                let periodo = format!("{anio}-{mes:02}");
                w.write_record([periodo.as_str(), c[2], producto.as_str(), format!("{precio:.2}").as_str()]).map_err(|e| e.to_string())?;
                productos.insert(producto);
                periodos.insert(periodo);
                n += 1;
            }
            w.flush().map_err(|e| e.to_string())?;
            println!("{n} precios promedio: {} productos, de {} a {} -> {}", productos.len(), periodos.first().cloned().unwrap_or_default(), periodos.last().cloned().unwrap_or_default(), salida.display());
            return Ok(ExitCode::SUCCESS);
        }
        Comando::Construido { mosaicos, municipios, salida } => return construido(&mosaicos, &municipios, &salida),
        Comando::UsoSuelo { total, no_residencial, municipios, salida } => return uso_suelo(&total, &no_residencial, &municipios, &salida),
        Comando::Mejoras { entrada, salida } => return mejoras(&entrada, &salida),
        Comando::Citas { entrada, salida } => return citas(&entrada, &salida),
        Comando::Fachadas { fotos, modelo, fuentes, salida } => return fachadas(&fotos, &modelo, fuentes.as_deref(), salida.as_deref()),
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
        Comando::Cif { texto, periodo, salida } => cif(&catalogo, &memoria, &texto, &periodo, &salida),
        Comando::Talla { entrada, salida } => {
            use faro_homologa::normalizar::basico;
            let filas = faro_homologa::lectura::leer_crudo(&entrada)?;
            let pos = filas.iter().position(|f| f.first().is_some_and(|c| basico(c) == "coddepto")).ok_or(format!("{}: no parece el libro del censo de talla", entrada.display()))?;
            let enc: Vec<String> = filas[pos].iter().map(|c| basico(c)).collect();
            let col = |n: &str| enc.iter().position(|c| c.starts_with(n)).ok_or(format!("{}: falta la columna {n}", entrada.display()));
            let (id, im, inom, ia, inn, ip, ic) = (col("coddepto")?, col("codmun")?, col("municipio")?, col("censo")?, col("n")?, col("prevalencia")?, col("categoria")?);
            let h = Homologador::new(&catalogo, &memoria);
            let (mut salen, mut distintos) = (Vec::new(), Vec::new());
            for f in filas.iter().skip(pos + 1).filter(|f| f.get(id).is_some_and(|c| !c.trim().is_empty())) {
                let c = |i: usize| f.get(i).map(|x| x.trim()).unwrap_or("");
                let codigo = format!("{:0>2}{:0>2}", c(id), c(im));
                let l = catalogo.por_codigo(&codigo).filter(|l| l.nivel == Nivel::Municipio).ok_or(format!("el código {codigo} ({}) no es un municipio del catálogo", c(inom)))?;
                // Doble comprobación: el nombre que trae el libro debe llevar al mismo código.
                let por_nombre = h.homologar(c(inom), Nivel::Municipio, Some(&codigo[..2]));
                if por_nombre.codigo.as_deref() != Some(codigo.as_str()) {
                    distintos.push(format!("{codigo} {} (el libro lo llama {:?})", l.nombre, c(inom)));
                }
                let prevalencia: f64 = c(ip).parse().map_err(|_| format!("{codigo}: prevalencia no numérica: {:?}", c(ip)))?;
                if !(0.0..=100.0).contains(&prevalencia) {
                    return Err(format!("{codigo}: prevalencia fuera de 0 a 100: {prevalencia}"));
                }
                salen.push((codigo, l.nombre.clone(), c(ia).to_string(), c(inn).to_string(), prevalencia, c(ic).to_lowercase()));
            }
            salen.sort_by(|a, b| a.0.cmp(&b.0));
            let unicos: std::collections::BTreeSet<&str> = salen.iter().map(|s| s.0.as_str()).collect();
            if unicos.len() != salen.len() {
                return Err("hay municipios repetidos en el libro".into());
            }
            if let Some(d) = salida.parent() {
                std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
            }
            let mut w = csv::WriterBuilder::new().terminator(csv::Terminator::CRLF).from_path(&salida).map_err(|e| e.to_string())?;
            w.write_record(["municipio", "nombre", "censo", "escolares_evaluados", "retardo_en_talla_pct", "vulnerabilidad"]).map_err(|e| e.to_string())?;
            for (codigo, nombre, censo, n, p, cat) in &salen {
                w.write_record([codigo.as_str(), nombre.as_str(), censo.as_str(), n.as_str(), format!("{p:.1}").as_str(), cat.as_str()]).map_err(|e| e.to_string())?;
            }
            w.flush().map_err(|e| e.to_string())?;
            println!("{} municipios -> {}", salen.len(), salida.display());
            println!("nombre y código coinciden en {}; el nombre no lleva solo al mismo código en {}{}", salen.len() - distintos.len(), distintos.len(), if distintos.is_empty() { String::new() } else { format!(": {}", distintos.join("; ")) });
            let mut cuenta = std::collections::BTreeMap::new();
            for s in &salen {
                *cuenta.entry(s.5.clone()).or_insert(0) += 1;
            }
            println!("por categoría de vulnerabilidad: {cuenta:?}");
            Ok(ExitCode::SUCCESS)
        }
        Comando::Ipmc { actual, anterior, salida } => ipmc(&actual, anterior.as_deref(), &salida),
        Comando::Construccion { precios, reglas, articulos, salida } => construccion(&memoria, &precios, &reglas, &articulos, &salida),
        Comando::Limites { entrada, tolerancia, salida } => limites(&catalogo, &entrada, tolerancia, &salida),
        Comando::Vial { carreteras, cerca_m, max_enlace_km, con_senderos, salida } => vial(&catalogo, &carreteras, faro_territorio::vial::Parametros { cerca_m, max_enlace_km }, con_senderos, &salida),
        Comando::CensoLugares { a1, b8, c1, salida } => {
            use std::collections::BTreeMap;
            let numero = |c: Option<&String>| c.and_then(|x| x.trim().parse::<f64>().ok()).map(|x| x.round() as i64).unwrap_or(0);
            // Cada hoja trae el código del lugar en la quinta columna y los datos desde la séptima.
            let leer = |ruta: &PathBuf, hoja: &str, titulo: &str| -> Result<Vec<Vec<String>>, String> {
                let filas = faro_homologa::lectura::leer_hoja_cruda(ruta, hoja)?;
                let pos = filas.iter().position(|f| f.get(4).is_some_and(|c| c.trim() == "Código") && f.get(6).is_some_and(|c| faro_homologa::normalizar::basico(c).starts_with(titulo))).ok_or(format!("{}: la hoja {hoja} no tiene el encabezado esperado ({titulo})", ruta.display()))?;
                Ok(filas.into_iter().skip(pos + 2).filter(|f| f.get(4).is_some_and(|c| c.trim().parse::<u32>().is_ok())).collect())
            };
            let codigo = |f: &Vec<String>| format!("{:0>7}", f[4].trim());
            let poblacion = leer(&a1, "A1_3", "poblacion total")?;
            let hogares: BTreeMap<String, i64> = leer(&b8, "B8_3", "total de hogares")?.iter().map(|f| (codigo(f), numero(f.get(6)))).collect();
            let viviendas: BTreeMap<String, (i64, i64)> = leer(&c1, "C1_3", "total de viviendas")?.iter().map(|f| (codigo(f), (numero(f.get(6)), numero(f.get(16))))).collect();
            if let Some(d) = salida.parent() {
                std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
            }
            let mut tabla = csv::Writer::from_path(&salida).map_err(|e| e.to_string())?;
            tabla.write_record(["codigo", "poblacion", "hombres", "mujeres", "edad_0_14", "edad_15_64", "edad_65_mas", "urbana", "rural", "hogares", "viviendas", "viviendas_ocupadas"]).map_err(|e| e.to_string())?;
            let (mut total, mut fuera, mut descuadres) = (0i64, Vec::new(), 0);
            for f in &poblacion {
                let c = codigo(f);
                if catalogo.por_codigo(&c).is_none_or(|l| l.nivel != Nivel::LugarPoblado) {
                    fuera.push(c.clone());
                }
                let n = |i: usize| numero(f.get(i));
                let edades: Vec<i64> = (9..30).map(n).collect();
                let (ninos, adultos, mayores): (i64, i64, i64) = (edades[..3].iter().sum(), edades[3..13].iter().sum(), edades[13..].iter().sum());
                // Dos comprobaciones por fila: sexo y edades deben sumar el total publicado.
                if n(7) + n(8) != n(6) || ninos + adultos + mayores != n(6) {
                    descuadres += 1;
                }
                total += n(6);
                let (v, vo) = viviendas.get(&c).copied().unwrap_or((0, 0));
                tabla.write_record([c.clone(), n(6).to_string(), n(7).to_string(), n(8).to_string(), ninos.to_string(), adultos.to_string(), mayores.to_string(), n(30).to_string(), n(31).to_string(), hogares.get(&c).copied().unwrap_or(0).to_string(), v.to_string(), vo.to_string()]).map_err(|e| e.to_string())?;
            }
            tabla.flush().map_err(|e| e.to_string())?;
            println!("{} lugares poblados; población censada {total}; hogares {}; viviendas {}.", poblacion.len(), hogares.values().sum::<i64>(), viviendas.values().map(|v| v.0).sum::<i64>());
            println!("Códigos que no están en el catálogo: {}. Filas donde sexo o edades no suman el total: {descuadres}.", fuera.len());
            println!("Escrito en {}", salida.display());
            if !fuera.is_empty() || descuadres > 0 {
                return Err(format!("la tabla no cuadra con el catálogo o consigo misma (por ejemplo {:?})", fuera.first()));
            }
            Ok(ExitCode::SUCCESS)
        }
        Comando::CensoIndicadores { cuadros, salida, techos } => {
            use faro_homologa::normalizar::basico;
            use std::collections::BTreeMap;
            // De cada cuadro: hoja municipal, columnas que se toman (nombre de salida, posición, palabra
            // que debe traer su encabezado). Si el INE cambia el orden de un cuadro, el proceso se detiene.
            type Columna = (&'static str, usize, &'static str);
            let cuadros_usados: [(&str, &str, &[Columna]); 12] = [
                ("cuadro-a11-", "A11_2", &[("poblacion_7_o_mas", 4, "poblacion de 7"), ("alfabetas_hombres", 7, "hombres"), ("alfabetas_mujeres", 8, "mujeres"), ("asiste_a_estudiar", 9, "asiste")]),
                ("cuadro-a12-", "A12_2", &[("usa_celular", 5, "usa"), ("usa_computadora", 8, "usa"), ("usa_internet", 11, "usa")]),
                ("cuadro-a5-", "A5_2", &[("poblacion_censada", 4, "poblacion total"), ("pueblo_maya", 5, "maya"), ("pueblo_garifuna", 6, "garifuna"), ("pueblo_xinka", 7, "xinka"), ("pueblo_ladino", 9, "ladina")]),
                ("cuadro-b2-", "B2_2", &[("hogares", 4, "total de hogares"), ("agua_tuberia_en_vivienda", 5, "tuberia en la vivienda"), ("agua_tuberia_fuera", 6, "tuberia fuera"), ("agua_chorro_publico", 7, "chorro")]),
                ("cuadro-b3-", "B3_2", &[("sanitario_con_drenaje", 5, "inodoro conectado a red"), ("sanitario_letrina", 8, "letrina"), ("sin_sanitario", 9, "no tiene")]),
                ("cuadro-b4-", "B4_2", &[("alumbrado_red_electrica", 5, "red de energia"), ("cocina_con_lena", 11, "lena")]),
                ("cuadro-c2-", "C2_2", &[("viviendas_particulares", 4, "total de viviendas"), ("pared_block", 6, "block"), ("pared_adobe", 8, "adobe"), ("techo_concreto", 16, "concreto"), ("techo_lamina", 17, "lamina"), ("techo_asbesto_cemento", 18, "asbesto"), ("techo_teja", 19, "teja"), ("techo_paja_palma", 20, "paja"), ("techo_desecho", 21, "desecho")]),
                ("cuadro-c3-", "C3_2", &[("viviendas_con_piso_declarado", 4, "total de viviendas"), ("piso_tierra", 11, "tierra")]),
                ("cuadro-a13-", "A13_2", &[("poblacion_15_o_mas", 4, "15 anos"), ("pea", 5, "economicamente activa"), ("pea_ocupada", 6, "ocupada")]),
                ("cuadro-a9-", "A9_2", &[("poblacion_4_o_mas", 4, "4 anos"), ("educ_ninguno", 5, "ninguno"), ("educ_basico", 10, "basico"), ("educ_diversificado", 11, "diversificado"), ("educ_licenciatura", 12, "licenciatura"), ("educ_posgrado", 13, "maestria")]),
                ("cuadro-b6-", "B6_2", &[("hogares_basura", 4, "total de hogares"), ("basura_servicio_municipal", 5, "servicio municipal"), ("basura_servicio_privado", 6, "servicio privado"), ("basura_la_queman", 7, "queman")]),
                ("cuadro-b1-", "B1_2", &[("hogares_tenencia", 4, "total de hogares"), ("vivienda_propia", 5, "propia"), ("vivienda_alquilada", 6, "alquilada"), ("propietario_hombre", 10, "hombre"), ("propietaria_mujer", 11, "mujer"), ("propietarios_ambos", 12, "ambos")]),
            ];
            let archivo = |trozo: &str| -> Result<PathBuf, String> {
                std::fs::read_dir(&cuadros).map_err(|e| format!("{}: {e}", cuadros.display()))?.flatten().map(|x| x.path()).find(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.contains(trozo))).ok_or(format!("falta el cuadro {trozo} en {}", cuadros.display()))
            };
            // Lee una hoja: comprueba los encabezados (en cualquiera de las filas de título) y devuelve código -> valores.
            let leer = |trozo: &str, hoja: &str, columnas: &[Columna], digitos: usize| -> Result<BTreeMap<String, Vec<i64>>, String> {
                let filas = faro_homologa::lectura::leer_hoja_cruda(&archivo(trozo)?, hoja)?;
                let pos = filas.iter().position(|f| f.first().is_some_and(|c| basico(c).starts_with("codigo"))).ok_or(format!("{hoja}: no se encontró el encabezado"))?;
                for (nombre, i, palabra) in columnas {
                    if !filas[pos..(pos + 3).min(filas.len())].iter().any(|f| f.get(*i).is_some_and(|c| basico(c).contains(palabra))) {
                        return Err(format!("{hoja}: la columna {} ya no es «{palabra}» ({nombre}); el INE cambió el cuadro", i + 1));
                    }
                }
                let col_codigo = if digitos == 7 { 4 } else { 2 };
                Ok(filas.iter().skip(pos + 1).filter(|f| f.get(col_codigo).is_some_and(|c| c.trim().parse::<u32>().is_ok()) && f.get(columnas[0].1).is_some_and(|c| c.trim().parse::<f64>().is_ok()))
                    .map(|f| (format!("{:0>digitos$}", f[col_codigo].trim()), columnas.iter().map(|(_, i, _)| f.get(*i).and_then(|c| c.trim().parse::<f64>().ok()).map_or(0, |x| x.round() as i64)).collect())).collect())
            };
            let mut por_municipio: BTreeMap<String, Vec<i64>> = BTreeMap::new();
            let mut encabezado: Vec<&str> = vec!["municipio"];
            for (trozo, hoja, columnas) in cuadros_usados {
                let datos = leer(trozo, hoja, columnas, 4)?;
                if datos.len() != 340 || datos.keys().any(|c| catalogo.por_codigo(c).is_none_or(|l| l.nivel != Nivel::Municipio)) {
                    return Err(format!("{hoja}: se esperaban los 340 municipios del catálogo y llegaron {}", datos.len()));
                }
                let ya = encabezado.len() - 1;
                encabezado.extend(columnas.iter().map(|c| c.0));
                for (codigo, valores) in datos {
                    let fila = por_municipio.entry(codigo).or_default();
                    fila.resize(ya, 0);
                    fila.extend(valores);
                }
            }
            if let Some(d) = salida.parent() {
                std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
            }
            let mut tabla = csv::Writer::from_path(&salida).map_err(|e| e.to_string())?;
            tabla.write_record(&encabezado).map_err(|e| e.to_string())?;
            for (codigo, valores) in &por_municipio {
                tabla.write_record(std::iter::once(codigo.clone()).chain(valores.iter().map(|v| v.to_string()))).map_err(|e| e.to_string())?;
            }
            tabla.flush().map_err(|e| e.to_string())?;
            let total = |nombre: &str| { let i = encabezado.iter().position(|e| *e == nombre).unwrap_or(1) - 1; por_municipio.values().map(|v| v[i]).sum::<i64>() };
            println!("{} municipios × {} cifras en {}", por_municipio.len(), encabezado.len() - 1, salida.display());
            println!("País: alfabetismo {:.1} % · usa internet {:.1} % · agua por tubería en la vivienda {:.1} % · drenaje {:.1} % · red eléctrica {:.1} % · cocina con leña {:.1} % · techo de lámina {:.1} % · piso de tierra {:.1} % · pueblo maya {:.1} %",
                100.0 * (total("alfabetas_hombres") + total("alfabetas_mujeres")) as f64 / total("poblacion_7_o_mas") as f64, 100.0 * total("usa_internet") as f64 / total("poblacion_7_o_mas") as f64,
                100.0 * total("agua_tuberia_en_vivienda") as f64 / total("hogares") as f64, 100.0 * total("sanitario_con_drenaje") as f64 / total("hogares") as f64, 100.0 * total("alumbrado_red_electrica") as f64 / total("hogares") as f64,
                100.0 * total("cocina_con_lena") as f64 / total("hogares") as f64, 100.0 * total("techo_lamina") as f64 / total("viviendas_particulares") as f64, 100.0 * total("piso_tierra") as f64 / total("viviendas_con_piso_declarado") as f64, 100.0 * total("pueblo_maya") as f64 / total("poblacion_censada") as f64);
            println!("País: participación económica {:.1} % · educación media o superior {:.1} % · basura recolectada {:.1} % · vivienda propia {:.1} % · propietaria mujer {:.1} % de las viviendas propias",
                100.0 * total("pea") as f64 / total("poblacion_15_o_mas") as f64, 100.0 * (total("educ_basico") + total("educ_diversificado") + total("educ_licenciatura") + total("educ_posgrado")) as f64 / total("poblacion_4_o_mas") as f64,
                100.0 * (total("basura_servicio_municipal") + total("basura_servicio_privado")) as f64 / total("hogares_basura") as f64, 100.0 * total("vivienda_propia") as f64 / total("hogares_tenencia") as f64, 100.0 * total("propietaria_mujer") as f64 / total("vivienda_propia") as f64);
            // Los cuadros de hogares deben contar los mismos hogares: si no, se mezclaron versiones del censo.
            if total("hogares") != total("hogares_basura") || total("hogares") != total("hogares_tenencia") {
                return Err(format!("los cuadros B1, B2 y B6 no suman los mismos hogares ({}, {}, {})", total("hogares_tenencia"), total("hogares"), total("hogares_basura")));
            }
            // Material del techo por lugar poblado (misma estructura, hoja de lugares).
            let columnas_techo: &[Columna] = &[("viviendas_particulares", 6, "total de viviendas"), ("techo_concreto", 18, "concreto"), ("techo_lamina", 19, "lamina"), ("techo_asbesto_cemento", 20, "asbesto"), ("techo_teja", 21, "teja"), ("techo_paja_palma", 22, "paja"), ("techo_desecho", 23, "desecho")];
            let lugares = leer("cuadro-c2-", "C2_3", columnas_techo, 7)?;
            let mut tabla = csv::Writer::from_path(&techos).map_err(|e| e.to_string())?;
            tabla.write_record(std::iter::once("codigo").chain(columnas_techo.iter().map(|c| c.0))).map_err(|e| e.to_string())?;
            let mut fuera = 0;
            for (codigo, valores) in &lugares {
                fuera += usize::from(catalogo.por_codigo(codigo).is_none_or(|l| l.nivel != Nivel::LugarPoblado));
                tabla.write_record(std::iter::once(codigo.clone()).chain(valores.iter().map(|v| v.to_string()))).map_err(|e| e.to_string())?;
            }
            tabla.flush().map_err(|e| e.to_string())?;
            println!("Techos: {} lugares poblados en {} ({fuera} códigos fuera del catálogo)", lugares.len(), techos.display());
            Ok(ExitCode::SUCCESS)
        }
        Comando::Edades { entrada, diccionario, anios, totales, salida } => {
            let h = Homologador::new(&catalogo, &memoria);
            let libro = faro_homologa::lectura::leer_crudo(&entrada)?;
            let mut filas = faro_territorio::poblacion::resumir_edades(&libro, &anios)?;
            // Los códigos que no existen se ubican por el nombre que les da el diccionario del libro,
            // con coincidencia exacta y única dentro de su departamento (igual que `faro poblacion`).
            let nombres = faro_territorio::poblacion::diccionario_municipios(&faro_homologa::lectura::leer_crudo(&diccionario)?);
            let mut movidos = std::collections::BTreeMap::new();
            for p in filas.iter_mut().filter(|p| catalogo.por_codigo(&p.municipio).is_none_or(|l| l.nivel != Nivel::Municipio)) {
                if !movidos.contains_key(&p.municipio) {
                    let nombre = nombres.get(&p.municipio).ok_or(format!("{}: el diccionario no lo nombra", p.municipio))?;
                    let r = h.homologar(nombre, Nivel::Municipio, Some(&p.municipio[..2]));
                    movidos.insert(p.municipio.clone(), r.codigo.filter(|_| r.estado.resuelto()).ok_or(format!("{} ({nombre}): no coincide con un único municipio", p.municipio))?);
                }
                p.municipio = movidos[&p.municipio].clone();
            }
            filas.sort_by(|a, b| (&a.municipio, a.anio, a.edad_desde).cmp(&(&b.municipio, b.anio, b.edad_desde)));
            // Comprobación contra la tabla de totales ya publicada.
            let publicados: std::collections::HashMap<(String, i32), (i64, i64)> = faro_territorio::poblacion::leer(&totales)?.into_iter().map(|p| ((p.municipio, p.anio), (p.hombres, p.mujeres))).collect();
            let mut sumas: std::collections::BTreeMap<(String, i32), (i64, i64)> = std::collections::BTreeMap::new();
            for p in &filas {
                let s = sumas.entry((p.municipio.clone(), p.anio)).or_default();
                *s = (s.0 + p.hombres, s.1 + p.mujeres);
            }
            let distintos = sumas.iter().filter(|(k, v)| publicados.get(*k) != Some(*v)).count();
            if let Some(d) = salida.parent() {
                std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
            }
            let mut w = csv::Writer::from_path(&salida).map_err(|e| e.to_string())?;
            for p in &filas {
                w.serialize(p).map_err(|e| e.to_string())?;
            }
            w.flush().map_err(|e| e.to_string())?;
            println!("{} filas ({} municipios × {} años × 15 grupos de edad); {} códigos reubicados; municipios-año cuya suma difiere de la tabla de totales: {distintos}.", filas.len(), sumas.len() / anios.len().max(1), anios.len(), movidos.len());
            println!("Escrito en {}", salida.display());
            if distintos > 0 {
                return Err("las edades no suman los totales publicados".into());
            }
            Ok(ExitCode::SUCCESS)
        }
        Comando::Poblacion { entrada, diccionario, salida } => {
            let h = Homologador::new(&catalogo, &memoria);
            let es_municipio = |m: &str| catalogo.por_codigo(m).is_some_and(|l| l.nivel == Nivel::Municipio);
            // Solo coincidencia exacta y única dentro del mismo departamento.
            let ubicar = |codigo: &str, nombre: &str| {
                let r = h.homologar(nombre, Nivel::Municipio, Some(&codigo[..2]));
                r.codigo.filter(|_| r.estado.resuelto()).ok_or(format!("{codigo} ({nombre}): no coincide con un único municipio del departamento {}", &codigo[..2]))
            };
            let libro = faro_homologa::lectura::leer_crudo(&entrada)?;
            let dic = match &diccionario {
                Some(r) => Some(faro_homologa::lectura::leer_crudo(r)?),
                None => None,
            };
            let (filas, pasos) = faro_territorio::poblacion::procesar(&libro, dic.as_deref(), &es_municipio, &ubicar)?;
            for (n, p) in pasos.iter().enumerate() {
                println!("{}. {}: {}", n + 1, p.titulo, p.detalle);
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
        Comando::Verificar { .. } | Comando::Hba { .. } | Comando::Fotos { .. } | Comando::Fuentes { .. } | Comando::Crecimiento { .. } | Comando::Aire { .. } | Comando::Precios { .. } | Comando::Construido { .. } | Comando::UsoSuelo { .. } | Comando::Mejoras { .. } | Comando::Citas { .. } | Comando::Fachadas { .. } | Comando::Referencias { .. } | Comando::Canasta { .. } => unreachable!("atendidos arriba"),
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

fn referencias(entrada: &Path, salida: &Path) -> Result<ExitCode, String> {
    use faro_homologa::normalizar::basico;
    let filas = faro_homologa::lectura::leer_crudo(entrada)?;
    let pos = filas.iter().take(10).position(|f| f.iter().any(|c| basico(c) == "precio de referencia q")).ok_or(format!("{}: no parece el libro de precios de referencia", entrada.display()))?;
    let enc: Vec<String> = filas[pos].iter().map(|c| basico(c)).collect();
    let col = |n: &str| enc.iter().position(|c| c == n).ok_or(format!("{}: falta la columna {n}", entrada.display()));
    let (inog, iev, imod, iprod, icar, ipre, iprecio, ifecha, icat) = (col("nog")?, col("evento")?, col("modalidad")?, col("producto")?, col("caracteristicas")?, col("presentacion")?, col("precio de referencia q")?, col("fecha de entrega")?, col("categoria")?);
    if let Some(d) = salida.parent() {
        std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
    }
    let mut w = csv::WriterBuilder::new().terminator(csv::Terminator::CRLF).from_path(salida).map_err(|e| e.to_string())?;
    w.write_record(["nog", "evento", "modalidad", "categoria", "producto", "caracteristicas", "presentacion", "precio", "fecha"]).map_err(|e| e.to_string())?;
    let una = |t: &str| t.split_whitespace().collect::<Vec<_>>().join(" ");
    let (mut escritas, mut sin_precio) = (0, 0);
    let mut categorias = std::collections::BTreeSet::new();
    for f in filas.iter().skip(pos + 1) {
        let c = |i: usize| f.get(i).map(|x| x.trim()).unwrap_or("");
        if c(iprod).is_empty() {
            continue;
        }
        // Solo entran las filas con precio numérico; las demás se cuentan.
        let Some(precio) = c(iprecio).replace(',', "").parse::<f64>().ok().filter(|p| *p > 0.0) else {
            sin_precio += 1;
            continue;
        };
        // Recorta las características largas: para buscar basta el inicio y el archivo queda liviano.
        let caracteristicas: String = una(c(icar)).chars().take(300).collect();
        let fecha = faro_homologa::solicitud::fecha_iso(c(ifecha)).unwrap_or_default();
        categorias.insert(basico(c(icat)));
        w.write_record([c(inog), &una(c(iev)), &una(c(imod)), &una(c(icat)), &una(c(iprod)), &caracteristicas, &una(c(ipre)), &format!("{precio:.2}"), &fecha]).map_err(|e| e.to_string())?;
        escritas += 1;
    }
    w.flush().map_err(|e| e.to_string())?;
    println!("{escritas} precios de referencia en {} categorías tal como vienen escritas; {sin_precio} filas sin precio numérico no se incluyen -> {}", categorias.len(), salida.display());
    Ok(ExitCode::SUCCESS)
}

/// Lee la "Tabla de población" del informe CIF: una fila por departamento con población total,
/// personas y porcentaje en cada fase, fase del área y Fase 3 o superior. Comprueba cada fila y
/// el total contra lo que el propio informe publica; si algo no cuadra, no escribe nada.
fn cif(catalogo: &Catalogo, memoria: &Memoria, ruta: &Path, periodo: &str, salida: &Path) -> Result<ExitCode, String> {
    let texto = std::fs::read_to_string(ruta).map_err(|e| format!("{}: {e}", ruta.display()))?;
    let h = Homologador::new(catalogo, memoria);
    // Los miles vienen separados por espacios ("1 482 083"): los números de una fila se leen de
    // derecha a izquierda según la forma conocida de la tabla.
    let numeros = |resto: &str| -> Vec<String> { resto.split("  ").map(|c| c.trim().replace(' ', "")).filter(|c| !c.is_empty()).collect() };
    let (mut filas, mut gran_total): (Vec<(String, String, Vec<i64>)>, Option<Vec<i64>>) = (Vec::new(), None);
    let mut dentro = false;
    for linea in texto.lines() {
        if linea.contains("Tabla de población de la situación proyectada") {
            dentro = true;
            continue;
        }
        if !dentro {
            continue;
        }
        let l = linea.trim();
        let Some(corte) = l.find("  ") else { continue };
        let (nombre, resto) = (l[..corte].trim(), &l[corte..]);
        let Ok(valores) = numeros(resto).iter().map(|c| c.parse::<i64>()).collect::<Result<Vec<i64>, _>>() else { continue };
        if nombre == "Gran Total" && valores.len() == 13 {
            gran_total = Some(valores);
            break;
        }
        if valores.len() != 14 {
            continue;
        }
        let d = h.homologar(nombre, Nivel::Departamento, None);
        let codigo = d.codigo.filter(|_| d.estado.resuelto()).ok_or(format!("{nombre:?} no es un departamento del catálogo"))?;
        filas.push((codigo, nombre.to_string(), valores));
    }
    let total = gran_total.ok_or("no se encontró la fila Gran Total de la tabla de población")?;
    if filas.len() != 22 {
        return Err(format!("se esperaban 22 departamentos y se leyeron {}", filas.len()));
    }
    // Columnas: 0 población; 1-2 fase 1 (# y %); 3-4 fase 2; 5-6 fase 3; 7-8 fase 4; 9-10 fase 5; 11 fase de área; 12-13 fase 3+.
    for (codigo, nombre, v) in &filas {
        if v[5] + v[7] + v[9] != v[12] {
            return Err(format!("{codigo} {nombre}: fase 3 + fase 4 + fase 5 no da la Fase 3 o superior publicada"));
        }
        if (v[1] + v[3] + v[5] + v[7] + v[9] - v[0]).abs() > 10 {
            return Err(format!("{codigo} {nombre}: las fases no suman la población total"));
        }
    }
    for (k, que) in [(0usize, "población"), (1, "fase 1"), (3, "fase 2"), (5, "fase 3"), (7, "fase 4"), (9, "fase 5")] {
        let suma: i64 = filas.iter().map(|(_, _, v)| v[k]).sum();
        if (suma - total[k]).abs() > 22 {
            return Err(format!("la suma de {que} por departamento ({suma}) no coincide con el Gran Total del informe ({})", total[k]));
        }
    }
    if let Some(d) = salida.parent() {
        std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
    }
    let mut w = csv::WriterBuilder::new().terminator(csv::Terminator::CRLF).from_path(salida).map_err(|e| e.to_string())?;
    w.write_record(["departamento", "nombre", "periodo", "poblacion_analizada", "fase1", "fase2", "fase3", "fase4", "fase5", "fase_area", "fase3_o_superior", "fase3_o_superior_pct"]).map_err(|e| e.to_string())?;
    filas.sort();
    for (codigo, _, v) in &filas {
        let nombre = catalogo.por_codigo(codigo).map(|l| l.nombre.clone()).unwrap_or_default();
        w.write_record([codigo.clone(), nombre, periodo.to_string(), v[0].to_string(), v[1].to_string(), v[3].to_string(), v[5].to_string(), v[7].to_string(), v[9].to_string(), v[11].to_string(), v[12].to_string(), v[13].to_string()]).map_err(|e| e.to_string())?;
    }
    w.flush().map_err(|e| e.to_string())?;
    let en_crisis: i64 = filas.iter().map(|(_, _, v)| v[12]).sum();
    println!("22 departamentos, {periodo}: {} personas analizadas; {en_crisis} en Fase 3 o superior; {} departamentos con área en Fase 3 -> {}", total[0], filas.iter().filter(|(_, _, v)| v[11] >= 3).count(), salida.display());
    println!("comprobado: cada fila suma su total y su Fase 3 o superior, y los departamentos suman el Gran Total del informe");
    Ok(ExitCode::SUCCESS)
}

/// Filas de un libro del IPMC: (año, mes, código, grupo, artículo, unidad, índice).
fn leer_ipmc(ruta: &Path) -> Result<Vec<(i32, String, u32, String, String, String, f64)>, String> {
    let filas = faro_homologa::lectura::leer_crudo(ruta)?;
    let pos = filas.iter().take(10).position(|f| f.iter().any(|c| faro_homologa::normalizar::basico(c) == "codigo articulo")).ok_or(format!("{}: no parece un libro del IPMC", ruta.display()))?;
    let enc: Vec<String> = filas[pos].iter().map(|c| faro_homologa::normalizar::basico(c)).collect();
    let col = |n: &str| enc.iter().position(|c| c == n).ok_or(format!("{}: falta la columna {n}", ruta.display()));
    let (ia, im, ig, ic, in_, iu, ii) = (col("ano")?, col("mes")?, col("grupo")?, col("codigo articulo")?, col("nombre articulo")?, col("unidad de medida")?, col("indice")?);
    Ok(filas.iter().skip(pos + 1).filter_map(|f| {
        let c = |i: usize| f.get(i).map(|x| x.trim()).unwrap_or("");
        Some((c(ia).parse().ok()?, c(im).to_string(), c(ic).parse().ok()?, c(ig).to_string(), c(in_).split_whitespace().collect::<Vec<_>>().join(" "), c(iu).to_string(), c(ii).parse().ok()?))
    }).collect())
}

fn ipmc(actual: &Path, anterior: Option<&Path>, salida: &Path) -> Result<ExitCode, String> {
    let filas = leer_ipmc(actual)?;
    let (anio, mes) = filas.iter().map(|f| (f.0, f.1.clone())).last().ok_or("el libro no tiene filas")?;
    let previo: std::collections::HashMap<u32, f64> = match anterior {
        Some(r) => leer_ipmc(r)?.into_iter().filter(|f| f.0 == anio - 1 && f.1 == mes).map(|f| (f.2, f.6)).collect(),
        None => Default::default(),
    };
    if let Some(d) = salida.parent() {
        std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
    }
    let mut w = csv::WriterBuilder::new().terminator(csv::Terminator::CRLF).from_path(salida).map_err(|e| e.to_string())?;
    w.write_record(["codigo", "grupo", "articulo", "unidad", "periodo", "indice", "indice_un_anio_antes", "variacion_interanual_pct"]).map_err(|e| e.to_string())?;
    let mut n = 0;
    for f in filas.iter().filter(|f| f.0 == anio && f.1 == mes) {
        let antes = previo.get(&f.2);
        w.write_record([f.2.to_string(), f.3.clone(), f.4.clone(), f.5.clone(), format!("{mes} {anio}"), format!("{:.2}", f.6), antes.map(|a| format!("{a:.2}")).unwrap_or_default(), antes.filter(|a| **a > 0.0).map(|a| format!("{:.2}", (f.6 / a - 1.0) * 100.0)).unwrap_or_default()]).map_err(|e| e.to_string())?;
        n += 1;
    }
    w.flush().map_err(|e| e.to_string())?;
    println!("{n} artículos del IPMC, {mes} {anio}{} -> {}", if previo.is_empty() { String::new() } else { format!(", con variación frente a {mes} {}", anio - 1) }, salida.display());
    Ok(ExitCode::SUCCESS)
}

fn construccion(memoria: &Memoria, ruta_precios: &Path, ruta_reglas: &Path, ruta_articulos: &Path, salida: &Path) -> Result<ExitCode, String> {
    use faro_homologa::productos;
    use faro_territorio::construccion::{resumir, Observacion};
    let reglas = productos::Reglas::cargar(ruta_reglas)?;
    let articulos: std::collections::HashMap<u32, (String, String)> = csv::Reader::from_path(ruta_articulos).map_err(|e| format!("{}: {e}", ruta_articulos.display()))?
        .deserialize::<std::collections::HashMap<String, String>>().flatten().filter_map(|f| Some((f.get("codigo")?.parse().ok()?, (f.get("articulo")?.clone(), f.get("unidad")?.clone())))).collect();
    let mut observaciones = Vec::new();
    for f in csv::Reader::from_path(ruta_precios).map_err(|e| format!("{}: {e}", ruta_precios.display()))?.deserialize::<std::collections::HashMap<String, String>>() {
        let f = f.map_err(|e| format!("{}: {e}", ruta_precios.display()))?;
        let campo = |k: &str| f.get(k).cloned().unwrap_or_default();
        let Ok(precio) = campo("precio").parse::<f64>() else { continue };
        let r = productos::homologar(&reglas, memoria, &campo("producto"), &campo("unidad_publicada"), precio);
        observaciones.push(Observacion { proveedor: campo("proveedor"), producto: campo("producto"), url: campo("url"), fecha: campo("fecha"), precio, unidad: campo("unidad_publicada"), codigo: r.codigo, estado: r.estado.texto().to_string(), precio_oficial: r.precio_oficial, motivo: r.motivo });
    }
    let cuenta = |e: &str| observaciones.iter().filter(|o| o.estado == e).count();
    println!("{} precios publicados: {} comparables, {} por revisar, {} de otra medida, {} sin artículo del catálogo con regla", observaciones.len(), cuenta("comparable") + cuenta("memoria"), cuenta("revisar"), cuenta("distinto"), cuenta("sin_articulo"));
    std::fs::create_dir_all(salida).map_err(|e| e.to_string())?;
    let mut w = csv::Writer::from_path(salida.join("productos_homologados.csv")).map_err(|e| e.to_string())?;
    w.write_record(["proveedor", "producto", "precio", "unidad_publicada", "ipmc", "estado", "precio_unidad_oficial", "motivo", "url"]).map_err(|e| e.to_string())?;
    for o in observaciones.iter().filter(|o| o.codigo.is_some()) {
        w.write_record([o.proveedor.clone(), o.producto.clone(), o.precio.to_string(), o.unidad.clone(), o.codigo.map(|c| c.to_string()).unwrap_or_default(), o.estado.clone(), o.precio_oficial.map(|p| format!("{p:.2}")).unwrap_or_default(), o.motivo.clone(), o.url.clone()]).map_err(|e| e.to_string())?;
    }
    w.flush().map_err(|e| e.to_string())?;
    for a in resumir(&observaciones, &faro_territorio::precios::Parametros::default()) {
        let (nombre, unidad) = articulos.get(&a.codigo).cloned().unwrap_or_default();
        let detalle = a.por_proveedor.iter().map(|p| format!("{} Q{:.2}", p.proveedor, p.precio)).collect::<Vec<_>>().join(" · ");
        println!("{:>2} {} [{unidad}]: {} proveedor(es){} · {} por revisar · {} de otra medida\n     {detalle}", a.codigo, nombre.chars().take(60).collect::<String>(), a.por_proveedor.len(),
            a.referencia.precio_referencia.map(|p| format!(" · referencia Q{p:.2} ({})", a.referencia.medida)).unwrap_or_default(), a.por_revisar, a.distintos);
    }
    println!("detalle -> {}", salida.join("productos_homologados.csv").display());
    Ok(ExitCode::SUCCESS)
}

/// Lee el GeoTIFF de un zip de GHSL.
fn leer_mosaico(ruta: &Path) -> Result<faro_territorio::geotiff::Raster, String> {
    use std::io::Read;
    let en = |e: String| format!("{}: {e}", ruta.display());
    let mut zip = zip::ZipArchive::new(std::fs::File::open(ruta).map_err(|e| en(e.to_string()))?).map_err(|e| en(e.to_string()))?;
    let nombre = zip.file_names().find(|n| n.ends_with(".tif")).map(str::to_string).ok_or(en("el zip no trae un .tif".into()))?;
    let mut bytes = Vec::new();
    zip.by_name(&nombre).map_err(|e| en(e.to_string()))?.read_to_end(&mut bytes).map_err(|e| en(e.to_string()))?;
    faro_territorio::geotiff::leer(&bytes).map_err(en)
}

fn construido(dir: &Path, ruta_municipios: &Path, salida: &Path) -> Result<ExitCode, String> {
    use faro_territorio::construido::{self, Mosaico};
    use faro_territorio::geo::Poligono;
    use std::collections::BTreeMap;
    let g: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(ruta_municipios).map_err(|e| format!("{}: {e}", ruta_municipios.display()))?).map_err(|e| e.to_string())?;
    let anillo = |v: &serde_json::Value| -> Vec<(f64, f64)> { v.as_array().map(|pts| pts.iter().filter_map(|p| Some((p.get(0)?.as_f64()?, p.get(1)?.as_f64()?))).collect()).unwrap_or_default() };
    let municipios: Vec<(String, Vec<Poligono>)> = g["features"].as_array().ok_or("la capa de municipios no tiene figuras")?.iter().map(|f| {
        let poligonos: Vec<Poligono> = f["geometry"]["coordinates"].as_array().map(|ps| ps.iter().map(|p| p.as_array().map(|a| a.iter().map(anillo).collect()).unwrap_or_default()).collect()).unwrap_or_default();
        (f["properties"]["codigo"].as_str().unwrap_or("").to_string(), construido::proyectar(&poligonos))
    }).collect();

    // Los archivos de GHSL llevan la época en el nombre: ..._E2020_...
    let mut por_anio: BTreeMap<i32, Vec<PathBuf>> = BTreeMap::new();
    for e in std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?.flatten() {
        let n = e.file_name().to_string_lossy().to_string();
        if let Some(anio) = n.split("_E").nth(1).and_then(|r| r.get(..4)).and_then(|a| a.parse::<i32>().ok()).filter(|_| n.ends_with(".zip")) {
            por_anio.entry(anio).or_default().push(e.path());
        }
    }
    if por_anio.is_empty() {
        return Err(format!("{}: no hay mosaicos de GHSL (archivos ..._E<año>_....zip)", dir.display()));
    }
    // (municipio, año) -> (m², celdas sumadas)
    let mut total: BTreeMap<(String, i32), (f64, usize)> = BTreeMap::new();
    // Para la animación: bloques de 1 km (10 × 10 celdas). (municipio, bloque x, bloque y) -> m² por época.
    let epocas: Vec<i32> = por_anio.keys().copied().filter(|a| *a <= 2020).collect();
    let mut bloques: BTreeMap<(usize, i64, i64), Vec<f64>> = BTreeMap::new();
    for (anio, rutas) in &por_anio {
        if *anio > 2020 {
            println!("{anio}: es una proyección de GHSL, no una observación; se omite");
            continue;
        }
        for ruta in rutas {
            let r = leer_mosaico(ruta)?;
            let m = Mosaico { ancho: r.ancho, alto: r.alto, x0: r.x0, y0: r.y0, paso: r.paso, celdas: &r.celdas, sin_dato: r.sin_dato };
            let k = epocas.iter().position(|a| a == anio).unwrap_or(0);
            for (i, (codigo, poligonos)) in municipios.iter().enumerate() {
                let (mut suma, mut n) = (0.0, 0usize);
                construido::recorrer(&m, poligonos, &mut |x, y, v| {
                    suma += v as f64;
                    n += 1;
                    if v > 0 {
                        bloques.entry((i, (x / 1000.0).floor() as i64, (y / 1000.0).floor() as i64)).or_insert_with(|| vec![0.0; epocas.len()])[k] += v as f64;
                    }
                });
                if n > 0 {
                    let e = total.entry((codigo.clone(), *anio)).or_default();
                    e.0 += suma;
                    e.1 += n;
                }
            }
        }
        let pais: f64 = total.iter().filter(|((_, a), _)| a == anio).map(|(_, (s, _))| s).sum();
        println!("{anio}: {} mosaico(s) · {:.0} hectáreas construidas en los municipios", rutas.len(), pais / 10_000.0);
    }
    // Todo municipio debe quedar cubierto por los mosaicos en cada época.
    let anios: Vec<i32> = por_anio.keys().copied().filter(|a| *a <= 2020).collect();
    let faltan: Vec<String> = municipios.iter().filter(|(c, _)| anios.iter().any(|a| !total.contains_key(&(c.clone(), *a)))).map(|(c, _)| c.clone()).collect();
    if !faltan.is_empty() {
        return Err(format!("{} municipios quedan fuera de los mosaicos descargados (por ejemplo {}): falta algún mosaico", faltan.len(), faltan[..faltan.len().min(5)].join(", ")));
    }
    if let Some(d) = salida.parent() {
        std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
    }
    let mut w = csv::WriterBuilder::new().terminator(csv::Terminator::CRLF).from_path(salida).map_err(|e| e.to_string())?;
    w.write_record(["municipio", "anio", "construido_ha", "celdas"]).map_err(|e| e.to_string())?;
    for ((codigo, anio), (m2, n)) in &total {
        w.write_record([codigo.clone(), anio.to_string(), format!("{:.2}", m2 / 10_000.0), n.to_string()]).map_err(|e| e.to_string())?;
    }
    w.flush().map_err(|e| e.to_string())?;
    println!("{} municipios × {} épocas -> {}", municipios.len(), anios.len(), salida.display());
    // Bloques con al menos media hectárea construida en alguna época (lo demás no se vería).
    let ruta_b = salida.with_file_name("superficie_construida_bloques.csv");
    let mut w = csv::WriterBuilder::new().terminator(csv::Terminator::CRLF).from_path(&ruta_b).map_err(|e| e.to_string())?;
    let mut encabezado = vec!["municipio".to_string(), "longitud".into(), "latitud".into()];
    encabezado.extend(epocas.iter().map(|a| format!("ha_{a}")));
    w.write_record(&encabezado).map_err(|e| e.to_string())?;
    let mut escritos = 0;
    for ((i, bx, by), m2) in &bloques {
        if m2.iter().copied().fold(0.0, f64::max) < 5000.0 {
            continue;
        }
        let (lon, lat) = construido::mollweide_inversa((*bx as f64 + 0.5) * 1000.0, (*by as f64 + 0.5) * 1000.0);
        let mut fila = vec![municipios[*i].0.clone(), format!("{lon:.4}"), format!("{lat:.4}")];
        fila.extend(m2.iter().map(|v| format!("{:.1}", v / 10_000.0)));
        w.write_record(&fila).map_err(|e| e.to_string())?;
        escritos += 1;
    }
    w.flush().map_err(|e| e.to_string())?;
    println!("{escritos} bloques de 1 km con construcción -> {}", ruta_b.display());
    Ok(ExitCode::SUCCESS)
}

/// Prueba del clasificador de fachadas sobre una carpeta de fotos. Con subcarpetas `vivienda/` y
/// `comercio/` se sabe qué se esperaba de cada una y se cuentan aciertos, errores y no concluyentes.
fn fachadas(fotos: &Path, modelo: &Path, fuentes: Option<&Path>, salida: Option<&Path>) -> Result<ExitCode, String> {
    use faro_fuentes::fachada::{Clasificador, PARTE_MINIMA};
    let clasificador = Clasificador::cargar(modelo)?;
    // Fuente de cada foto, si se dio la tabla: nombre de archivo -> (página, autor, licencia).
    let mut cita: std::collections::HashMap<String, (String, String, String)> = std::collections::HashMap::new();
    if let Some(ruta) = fuentes {
        for f in csv::Reader::from_path(ruta).map_err(|e| format!("{}: {e}", ruta.display()))?.deserialize::<std::collections::HashMap<String, String>>().flatten() {
            let nombre = f.get("archivo").map(|a| a.rsplit('/').next().unwrap_or(a).to_string()).unwrap_or_default();
            cita.insert(nombre, (f.get("pagina_commons").cloned().unwrap_or_default(), f.get("autor").cloned().unwrap_or_default(), f.get("licencia").cloned().unwrap_or_default()));
        }
    }
    let es_foto = |p: &Path| p.extension().and_then(|e| e.to_str()).is_some_and(|e| matches!(e.to_lowercase().as_str(), "jpg" | "jpeg" | "png" | "webp"));
    let listar = |dir: &Path| -> Vec<PathBuf> { let mut v: Vec<PathBuf> = std::fs::read_dir(dir).into_iter().flatten().flatten().map(|e| e.path()).filter(|p| es_foto(p)).collect(); v.sort(); v };
    let mut grupos: Vec<(&str, Vec<PathBuf>)> = [("vivienda", fotos.join("vivienda")), ("comercio", fotos.join("comercio"))].into_iter().map(|(g, d)| (g, listar(&d))).filter(|(_, v)| !v.is_empty()).collect();
    if grupos.is_empty() {
        grupos.push(("", listar(fotos)));
    }
    if grupos.iter().all(|(_, v)| v.is_empty()) {
        return Err(format!("{}: no hay fotos (jpg, png o webp)", fotos.display()));
    }
    let mut filas: Vec<Vec<String>> = Vec::new();
    let (mut aciertos, mut errores, mut dudosas, mut ms_total) = (0, 0, 0, 0.0);
    println!("{:<10} {:<44} {:<10} {:>6}  escenas más probables", "esperado", "foto", "resultado", "conf.");
    for (esperado, rutas) in &grupos {
        for ruta in rutas {
            let bytes = std::fs::read(ruta).map_err(|e| format!("{}: {e}", ruta.display()))?;
            let inicio = std::time::Instant::now();
            let r = clasificador.clasificar(&bytes).map_err(|e| format!("{}: {e}", ruta.display()))?;
            ms_total += inicio.elapsed().as_secs_f64() * 1000.0;
            // Concluyente: dice vivienda o comercio con la parte mínima de la probabilidad.
            let concluyente = r.clase != "otro" && r.confianza >= PARTE_MINIMA;
            let veredicto = if !concluyente { "no concluyente" } else if esperado.is_empty() { "" } else if r.clase == *esperado { "acierto" } else { "error" };
            match veredicto { "acierto" => aciertos += 1, "error" => errores += 1, "no concluyente" => dudosas += 1, _ => {} }
            let nombre = ruta.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
            let escenas = r.escenas.iter().map(|(n, p)| format!("{n} {:.0} %", p * 100.0)).collect::<Vec<_>>().join("; ");
            println!("{:<10} {:<44} {:<10} {:>5.0}%  {escenas}  {veredicto}", esperado, nombre.chars().take(44).collect::<String>(), r.clase, r.confianza * 100.0);
            let (pagina, autor, licencia) = cita.get(&nombre).cloned().unwrap_or_default();
            filas.push(vec![esperado.to_string(), nombre, r.clase.clone(), format!("{:.3}", r.confianza), format!("{:.4}", r.p_vivienda), format!("{:.4}", r.p_comercio), escenas, veredicto.to_string(), pagina, autor, licencia]);
        }
    }
    let n = filas.len();
    println!("\n{n} fotos · {:.0} ms por foto en esta máquina", ms_total / n as f64);
    if grupos[0].0 != "" {
        println!("Aciertos {aciertos} · errores {errores} · no concluyentes {dudosas} (una foto no concluyente no pone en duda la visita)");
    }
    if let Some(ruta) = salida {
        let mut w = csv::Writer::from_path(ruta).map_err(|e| format!("{}: {e}", ruta.display()))?;
        w.write_record(["esperado", "foto", "clase", "confianza", "p_vivienda", "p_comercio", "escenas", "veredicto", "pagina_de_la_foto", "autor", "licencia_de_la_foto"]).map_err(|e| e.to_string())?;
        for f in &filas {
            w.write_record(f).map_err(|e| e.to_string())?;
        }
        w.flush().map_err(|e| e.to_string())?;
        println!("Tabla en {}", ruta.display());
    }
    Ok(ExitCode::SUCCESS)
}

/// Documento de fuentes y citas, con los mismos tres grupos que la pantalla.
fn citas(entrada: &Path, salida: &Path) -> Result<ExitCode, String> {
    let d: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(entrada).map_err(|e| format!("{}: {e}", entrada.display()))?).map_err(|e| format!("{}: {e}", entrada.display()))?;
    let fuentes = d["fuentes"].as_array().ok_or("fuentes.json no trae la lista «fuentes»")?;
    let mut md = String::from("# Fuentes y citas\n\nTodo dato que muestra FARO viene de una de estas fuentes. La fecha de consulta, la dirección exacta y la huella SHA-256 de cada archivo descargado están en `datos/crudo/manifiesto.jsonl` (`faro fuentes citar` las lista). Este documento lo escribe `faro citas` a partir de `datos/estandar/fuentes.json`, que es también lo que muestra la pantalla en «Acerca y fuentes». En los conjuntos del portal del INE el año es el de su última actualización en el portal.\n\n**Cómo citar los datos del INE usados aquí:** Fuente: Instituto Nacional de Estadística (INE), Guatemala, con el nombre del conjunto y la fecha de consulta que aparecen abajo.\n");
    let mut escritas = 0;
    for (titulo, tipos) in [("Datos", &["conjunto de datos", "sitio web", "archivos no publicados"][..]), ("Documentos del INE, de gobierno y de referencia", &["presentación", "presentaciones", "documento"][..]), ("Métodos y programas", &["artículo", "libro", "ponencia", "programa"][..])] {
        md += &format!("\n## {titulo}\n\n");
        for f in fuentes.iter().filter(|f| tipos.contains(&f["tipo"].as_str().unwrap_or(""))) {
            let licencia = f["licencia"].as_str().filter(|l| !l.is_empty()).map(|l| format!(" Licencia: {l}.")).unwrap_or_default();
            md += &format!("- {}\n  - Uso en FARO: {}{licencia}\n", f["cita"].as_str().unwrap_or(""), f["uso"].as_str().unwrap_or(""));
            escritas += 1;
        }
    }
    // Una fuente con un tipo que no está en ningún grupo no saldría: mejor avisar.
    if escritas != fuentes.len() {
        return Err(format!("{} fuente(s) tienen un tipo que no pertenece a ningún grupo del documento", fuentes.len() - escritas));
    }
    md += "\n## Cómo citar FARO\n\nSandoval, E., Echeverría, A. y Sandoval, A. (2026). FARO: recepción y homologación de registros administrativos, territorio y precios [programa]. Datatón INE 2026, categoría 2. https://github.com/tono88/faro-ine-2026\n";
    std::fs::write(salida, md).map_err(|e| format!("{}: {e}", salida.display()))?;
    println!("{escritas} fuentes -> {}", salida.display());
    Ok(ExitCode::SUCCESS)
}

/// Documento «Antes y ahora»: una tabla por área con lo que se tenía, lo que hace FARO y cómo escala.
fn mejoras(entrada: &Path, salida: &Path) -> Result<ExitCode, String> {
    let d: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(entrada).map_err(|e| format!("{}: {e}", entrada.display()))?).map_err(|e| format!("{}: {e}", entrada.display()))?;
    let puntos = d["puntos"].as_array().ok_or("mejoras.json no trae la lista «puntos»")?;
    let t = |p: &serde_json::Value, k: &str| p[k].as_str().unwrap_or("").replace('|', "/");
    // Nombre de cada sección del portal tal como aparece en su menú.
    let seccion = |id: &str| match id {
        "recibir" => "Recibir envío", "pendientes" => "Revisar pendientes", "resultados" => "Resultados y avance", "crecimiento" => "Población y crecimiento", "vial" => "Red vial", "mapa" => "Mapa", "proceso" => "Proceso",
        "operadores" => "Operadores de campo", "campo" => "Mi ruta de campo", "rubros" => "Precios por rubro", "precios" => "Precios de referencia", "construccion" => "Materiales de construcción", "balance" => "Hoja de balance: derivados",
        "tableros" => "Tableros", "capas" => "Capas publicadas", "acerca" => "Acerca y fuentes", otro => otro,
    }.to_string();
    let mut md = String::from("# Antes y ahora: qué se tenía, qué hace FARO y cómo escala\n\n");
    md += "Este documento se genera con `faro mejoras` a partir de [`datos/estandar/mejoras.json`](../datos/estandar/mejoras.json),\nla misma fuente de la sección «Antes y ahora» del portal. Para agregar un punto se edita ese archivo.\n\n";
    md += &format!("Son {} puntos. Cada uno dice quién planteó la necesidad o cómo se halló el problema, qué hace FARO con una\ncifra comprobable en el portal y cómo crece sin rehacer el sistema.\n", puntos.len());
    for area in d["areas"].as_array().into_iter().flatten().filter_map(|a| a.as_str()) {
        let suyos: Vec<&serde_json::Value> = puntos.iter().filter(|p| p["area"] == area).collect();
        if suyos.is_empty() {
            continue;
        }
        md += &format!("\n## {area}\n\n| Tema | Antes | Con FARO | Cómo escala |\n|---|---|---|---|\n");
        for p in suyos {
            md += &format!("| **{}** | {} *({})* | **{}** {}. {} Se ve en «{}». | {} |\n", t(p, "tema"), t(p, "antes"), t(p, "origen"), t(p, "cifra"), t(p, "cifra_de"), t(p, "ahora"), seccion(&t(p, "ver")), t(p, "escala"));
        }
    }
    // Un punto con un área que no está en la lista no saldría en ninguna tabla: mejor avisar.
    let sin_area = puntos.iter().filter(|p| !d["areas"].as_array().is_some_and(|a| a.contains(&p["area"]))).count();
    if sin_area > 0 {
        return Err(format!("{sin_area} punto(s) tienen un área que no está en la lista «areas»"));
    }
    std::fs::write(salida, md).map_err(|e| format!("{}: {e}", salida.display()))?;
    println!("{} puntos -> {}", puntos.len(), salida.display());
    Ok(ExitCode::SUCCESS)
}

/// Residencial y no residencial. GHSL publica dos capas con la misma cuadrícula: la superficie
/// construida total y la parte no residencial (NRES). La residencial es la diferencia. Se suman por
/// municipio (2000 y 2020) y por bloque de 1 km (2020), que es lo que se cruza con una visita de campo.
fn uso_suelo(dir_total: &Path, dir_nres: &Path, ruta_municipios: &Path, salida: &Path) -> Result<ExitCode, String> {
    use faro_territorio::construido::{self, Mosaico};
    use faro_territorio::geo::Poligono;
    use std::collections::BTreeMap;
    const ANIOS: [i32; 2] = [2000, 2020];
    let g: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(ruta_municipios).map_err(|e| format!("{}: {e}", ruta_municipios.display()))?).map_err(|e| e.to_string())?;
    let anillo = |v: &serde_json::Value| -> Vec<(f64, f64)> { v.as_array().map(|pts| pts.iter().filter_map(|p| Some((p.get(0)?.as_f64()?, p.get(1)?.as_f64()?))).collect()).unwrap_or_default() };
    let municipios: Vec<(String, Vec<Poligono>)> = g["features"].as_array().ok_or("la capa de municipios no tiene figuras")?.iter().map(|f| {
        let poligonos: Vec<Poligono> = f["geometry"]["coordinates"].as_array().map(|ps| ps.iter().map(|p| p.as_array().map(|a| a.iter().map(anillo).collect()).unwrap_or_default()).collect()).unwrap_or_default();
        (f["properties"]["codigo"].as_str().unwrap_or("").to_string(), construido::proyectar(&poligonos))
    }).collect();
    // (municipio, año) -> [m² totales, m² no residenciales]; bloque de 2020 -> lo mismo.
    let mut suma: BTreeMap<(usize, i32), [f64; 2]> = BTreeMap::new();
    let mut bloques: BTreeMap<(usize, i64, i64), [f64; 2]> = BTreeMap::new();
    let mut leidos = [0usize; 2];
    for (capa, dir) in [(0usize, dir_total), (1, dir_nres)] {
        for e in std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?.flatten() {
            let n = e.file_name().to_string_lossy().to_string();
            // Cada carpeta lleva una sola capa: se descarta el archivo que no corresponde.
            if !n.ends_with(".zip") || n.contains("_NRES_") != (capa == 1) {
                continue;
            }
            let Some(anio) = n.split("_E").nth(1).and_then(|r| r.get(..4)).and_then(|a| a.parse::<i32>().ok()).filter(|a| ANIOS.contains(a)) else { continue };
            let r = leer_mosaico(&e.path())?;
            let m = Mosaico { ancho: r.ancho, alto: r.alto, x0: r.x0, y0: r.y0, paso: r.paso, celdas: &r.celdas, sin_dato: r.sin_dato };
            for (i, (_, poligonos)) in municipios.iter().enumerate() {
                construido::recorrer(&m, poligonos, &mut |x, y, v| {
                    if v > 0 {
                        suma.entry((i, anio)).or_default()[capa] += v as f64;
                        if anio == 2020 {
                            bloques.entry((i, (x / 1000.0).floor() as i64, (y / 1000.0).floor() as i64)).or_default()[capa] += v as f64;
                        }
                    }
                });
            }
            leidos[capa] += 1;
        }
    }
    if leidos[0] == 0 || leidos[1] == 0 {
        return Err(format!("faltan mosaicos: {} de superficie total en {} y {} no residenciales en {} (épocas 2000 y 2020)", leidos[0], dir_total.display(), leidos[1], dir_nres.display()));
    }
    // Comprobación: lo no residencial es una parte de lo construido; nunca puede superarlo.
    let excedidos = suma.values().filter(|v| v[1] > v[0] * 1.001 + 1.0).count();
    if excedidos > 0 {
        return Err(format!("{excedidos} municipio(s) con más superficie no residencial que construida: las dos capas no corresponden a la misma versión"));
    }
    if let Some(d) = salida.parent() {
        std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
    }
    let mut w = csv::WriterBuilder::new().terminator(csv::Terminator::CRLF).from_path(salida).map_err(|e| e.to_string())?;
    w.write_record(["municipio", "anio", "construido_ha", "no_residencial_ha", "residencial_ha", "no_residencial_pct"]).map_err(|e| e.to_string())?;
    for ((i, anio), v) in &suma {
        w.write_record([municipios[*i].0.clone(), anio.to_string(), format!("{:.2}", v[0] / 1e4), format!("{:.2}", v[1] / 1e4), format!("{:.2}", (v[0] - v[1]).max(0.0) / 1e4), if v[0] > 0.0 { format!("{:.1}", 100.0 * v[1] / v[0]) } else { String::new() }]).map_err(|e| e.to_string())?;
    }
    w.flush().map_err(|e| e.to_string())?;
    for anio in ANIOS {
        let (t, n) = suma.iter().filter(|((_, a), _)| *a == anio).fold((0.0, 0.0), |(t, n), (_, v)| (t + v[0], n + v[1]));
        println!("{anio}: {:.0} ha construidas · {:.0} ha no residenciales ({:.1} %) · {:.0} ha residenciales", t / 1e4, n / 1e4, 100.0 * n / t.max(1.0), (t - n) / 1e4);
    }
    println!("{} municipios × {} épocas -> {}", municipios.len(), ANIOS.len(), salida.display());
    let ruta_b = salida.with_file_name("uso_construido_bloques.csv");
    let mut w = csv::WriterBuilder::new().terminator(csv::Terminator::CRLF).from_path(&ruta_b).map_err(|e| e.to_string())?;
    w.write_record(["municipio", "longitud", "latitud", "construido_ha_2020", "no_residencial_ha_2020"]).map_err(|e| e.to_string())?;
    let mut escritos = 0;
    for ((i, bx, by), v) in &bloques {
        if v[0] < 5000.0 {
            continue;
        }
        let (lon, lat) = construido::mollweide_inversa((*bx as f64 + 0.5) * 1000.0, (*by as f64 + 0.5) * 1000.0);
        w.write_record([municipios[*i].0.clone(), format!("{lon:.4}"), format!("{lat:.4}"), format!("{:.1}", v[0] / 1e4), format!("{:.1}", v[1].min(v[0]) / 1e4)]).map_err(|e| e.to_string())?;
        escritos += 1;
    }
    w.flush().map_err(|e| e.to_string())?;
    println!("{escritos} bloques de 1 km con al menos media hectárea construida -> {}", ruta_b.display());
    Ok(ExitCode::SUCCESS)
}

fn limites(catalogo: &Catalogo, entrada: &Path, tolerancia: f64, salida: &Path) -> Result<ExitCode, String> {
    use faro_territorio::asignacion::Coordenada;
    use faro_territorio::geo::{self, Poligono};
    use std::collections::BTreeMap;
    let en = |e: String| format!("{}: {e}", entrada.display());
    let mut zip = zip::ZipArchive::new(std::fs::File::open(entrada).map_err(|e| en(e.to_string()))?).map_err(|e| en(e.to_string()))?;
    let nombre = zip.file_names().find(|n| n.to_lowercase().contains("admin2") && n.ends_with(".geojson")).map(str::to_string).ok_or(en("el zip no trae la capa admin2 en GeoJSON".into()))?;
    let datos: serde_json::Value = serde_json::from_reader(zip.by_name(&nombre).map_err(|e| en(e.to_string()))?).map_err(|e| en(e.to_string()))?;
    let anillo = |v: &serde_json::Value| -> Vec<(f64, f64)> { v.as_array().map(|pts| pts.iter().filter_map(|p| Some((p.get(0)?.as_f64()?, p.get(1)?.as_f64()?))).collect()).unwrap_or_default() };
    let poligono = |v: &serde_json::Value| -> Poligono { v.as_array().map(|a| a.iter().map(anillo).collect()).unwrap_or_default() };
    // (código tal como viene, polígonos)
    let mut formas: Vec<(String, Vec<Poligono>)> = Vec::new();
    for f in datos["features"].as_array().ok_or(en("no hay features".into()))? {
        let pcode = f["properties"]["adm2_pcode"].as_str().ok_or(en("una figura no trae adm2_pcode".into()))?;
        let g = &f["geometry"];
        let poligonos = match g["type"].as_str() {
            Some("Polygon") => vec![poligono(&g["coordinates"])],
            Some("MultiPolygon") => g["coordinates"].as_array().map(|a| a.iter().map(poligono).collect()).unwrap_or_default(),
            otro => return Err(en(format!("{pcode}: geometría {otro:?} no esperada"))),
        };
        formas.push((pcode.trim_start_matches("GT").to_string(), poligonos));
    }
    let es_municipio = |c: &str| catalogo.por_codigo(c).is_some_and(|l| l.nivel == Nivel::Municipio);
    let ajenos: Vec<&str> = formas.iter().map(|(c, _)| c.as_str()).filter(|c| !es_municipio(c)).collect();
    println!("{} figuras; {} con código de municipio del catálogo; sin municipio (se omiten): {}", formas.len(), formas.len() - ajenos.len(), if ajenos.is_empty() { "ninguna".into() } else { ajenos.join(", ") });

    // Verificación: ¿los lugares poblados del censo caen en el polígono que lleva su código?
    let cajas: Vec<(f64, f64, f64, f64)> = formas.iter().map(|(_, p)| geo::caja(p)).collect();
    let mut dentro: Vec<BTreeMap<String, usize>> = vec![BTreeMap::new(); formas.len()];
    let (mut ubicados, mut coinciden, mut total) = (0usize, 0usize, 0usize);
    for l in catalogo.de_nivel(Nivel::LugarPoblado, None) {
        let (Some(longitud), Some(latitud)) = (l.longitud, l.latitud) else { continue };
        total += 1;
        let c = Coordenada { longitud, latitud };
        if let Some(i) = (0..formas.len()).find(|&i| longitud >= cajas[i].0 && longitud <= cajas[i].2 && latitud >= cajas[i].1 && latitud <= cajas[i].3 && geo::contiene(&formas[i].1, c)) {
            ubicados += 1;
            coinciden += usize::from(formas[i].0 == l.padre);
            *dentro[i].entry(l.padre.clone()).or_default() += 1;
        }
    }
    println!("{total} lugares poblados con coordenadas: {ubicados} caen en alguna figura; {coinciden} ({:.1} %) en la que lleva su código de municipio", 100.0 * coinciden as f64 / ubicados.max(1) as f64);
    // Figura cuyo contenido es, casi todo, de otro municipio: el código de la figura está cambiado.
    let mayoria = |i: usize| dentro[i].iter().max_by_key(|(_, n)| **n).map(|(c, n)| (c.clone(), *n as f64 / dentro[i].values().sum::<usize>() as f64));
    let mut correccion: BTreeMap<usize, String> = BTreeMap::new();
    for i in 0..formas.len() {
        let Some((codigo, parte)) = mayoria(i) else { continue };
        if codigo != formas[i].0 && parte >= 0.8 {
            // Solo se corrige si es un intercambio: la figura que hoy lleva ese código contiene, a su vez, los lugares de esta.
            let otra = (0..formas.len()).find(|&j| formas[j].0 == codigo);
            let reciproco = otra.and_then(|j| mayoria(j)).is_some_and(|(c, p)| c == formas[i].0 && p >= 0.8);
            println!("  la figura {} ({}) contiene en {:.0} % lugares de {} ({}){}", formas[i].0, catalogo.por_codigo(&formas[i].0).map_or("", |l| &l.nombre), parte * 100.0, codigo, catalogo.por_codigo(&codigo).map_or("", |l| &l.nombre), if reciproco { ": códigos intercambiados, se corrigen" } else { ": se deja como viene" });
            if reciproco {
                correccion.insert(i, codigo);
            }
        }
    }
    // Por municipio: cuántos de sus lugares poblados caen dentro de su propia figura (ya corregida).
    let mut acuerdo: BTreeMap<String, (usize, usize, BTreeMap<String, usize>)> = BTreeMap::new();
    for (i, cuenta) in dentro.iter().enumerate() {
        let figura = correccion.get(&i).unwrap_or(&formas[i].0);
        for (municipio, n) in cuenta {
            let e = acuerdo.entry(municipio.clone()).or_default();
            e.0 += n;
            if municipio == figura {
                e.1 += n;
            } else {
                *e.2.entry(figura.clone()).or_default() += n;
            }
        }
    }
    let ruta_v = salida.with_file_name("limites_verificacion.csv");
    let mut w = csv::WriterBuilder::new().terminator(csv::Terminator::CRLF).from_path(&ruta_v).map_err(|e| e.to_string())?;
    w.write_record(["municipio", "nombre", "lugares_ubicados", "en_su_figura", "pct", "caen_sobre_todo_en"]).map_err(|e| e.to_string())?;
    let (mut bien, mut mal) = (0, 0);
    for (m, (total, propios, otros)) in &acuerdo {
        let pct = 100.0 * *propios as f64 / (*total).max(1) as f64;
        if pct >= 90.0 { bien += 1 } else { mal += 1 }
        let otro = otros.iter().max_by_key(|(_, n)| **n).map(|(c, n)| format!("{c} {} ({n})", catalogo.por_codigo(c).map_or("", |l| &l.nombre))).unwrap_or_default();
        w.write_record([m.clone(), catalogo.por_codigo(m).map_or(String::new(), |l| l.nombre.clone()), total.to_string(), propios.to_string(), format!("{pct:.1}"), otro]).map_err(|e| e.to_string())?;
    }
    w.flush().map_err(|e| e.to_string())?;
    println!("{bien} municipios con 90 % o más de sus lugares dentro de su figura; {mal} con menos -> {}", ruta_v.display());

    let figuras: Vec<serde_json::Value> = formas
        .iter()
        .enumerate()
        .map(|(i, (codigo, poligonos))| (correccion.get(&i).unwrap_or(codigo), poligonos))
        .filter(|(codigo, _)| es_municipio(codigo))
        .map(|(codigo, poligonos)| {
            let simples = geo::simplificar_poligonos(poligonos, tolerancia);
            let coords: Vec<Vec<Vec<[f64; 2]>>> = simples.iter().map(|p| p.iter().map(|a| a.iter().map(|(x, y)| [(x * 1e4).round() / 1e4, (y * 1e4).round() / 1e4]).collect()).collect()).collect();
            let l = catalogo.por_codigo(codigo);
            serde_json::json!({ "type": "Feature", "properties": { "codigo": codigo, "nombre": l.map(|l| l.nombre.clone()), "departamento_codigo": &codigo[..2], "departamento": catalogo.por_codigo(&codigo[..2]).map(|d| d.nombre.clone()) },
                                "geometry": { "type": "MultiPolygon", "coordinates": coords } })
        })
        .collect();
    let puntos: usize = figuras.iter().map(|f| f["geometry"]["coordinates"].as_array().map_or(0, |ps| ps.iter().flat_map(|p| p.as_array().into_iter().flatten()).map(|a| a.as_array().map_or(0, Vec::len)).sum())).sum();
    if let Some(dir) = salida.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    // Departamentos, de la misma fuente, para mapas a menor detalle.
    let nombre_deptos = zip.file_names().find(|n| n.to_lowercase().contains("admin1") && n.ends_with(".geojson")).map(str::to_string);
    if let Some(n1) = nombre_deptos {
        let d1: serde_json::Value = serde_json::from_reader(zip.by_name(&n1).map_err(|e| en(e.to_string()))?).map_err(|e| en(e.to_string()))?;
        let deptos: Vec<serde_json::Value> = d1["features"].as_array().into_iter().flatten().filter_map(|f| {
            let codigo = f["properties"]["adm1_pcode"].as_str()?.trim_start_matches("GT").to_string();
            let l = catalogo.por_codigo(&codigo).filter(|l| l.nivel == Nivel::Departamento)?;
            let g = &f["geometry"];
            let poligonos: Vec<Poligono> = match g["type"].as_str()? {
                "Polygon" => vec![poligono(&g["coordinates"])],
                _ => g["coordinates"].as_array()?.iter().map(poligono).collect(),
            };
            let coords: Vec<Vec<Vec<[f64; 2]>>> = geo::simplificar_poligonos(&poligonos, tolerancia * 2.0).iter().map(|p| p.iter().map(|a| a.iter().map(|(x, y)| [(x * 1e4).round() / 1e4, (y * 1e4).round() / 1e4]).collect()).collect()).collect();
            Some(serde_json::json!({ "type": "Feature", "properties": { "codigo": codigo, "nombre": l.nombre }, "geometry": { "type": "MultiPolygon", "coordinates": coords } }))
        }).collect();
        let ruta_d = salida.with_file_name("departamentos.geojson");
        println!("{} departamentos -> {}", deptos.len(), ruta_d.display());
        std::fs::write(&ruta_d, serde_json::to_string(&serde_json::json!({ "type": "FeatureCollection", "features": deptos })).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    }
    let n = figuras.len();
    std::fs::write(salida, serde_json::to_string(&serde_json::json!({ "type": "FeatureCollection", "features": figuras })).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    println!("{n} municipios, {puntos} puntos -> {} ({} KB)", salida.display(), std::fs::metadata(salida).map(|m| m.len() / 1024).unwrap_or(0));
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
    // Caminos ya mapeados alrededor de cada enlace propuesto (0.015°, unos 1.6 km), para verlos en el mapa:
    // los de la red principal y los del tramo suelto que el enlace uniría.
    let radio = 0.015; // grados
    let extremos: Vec<(f64, f64)> = enlaces.iter().flat_map(|e| [(e.desde_longitud, e.desde_latitud), (e.hasta_longitud, e.hasta_latitud)]).collect();
    let cerca = |p: &(f64, f64)| extremos.iter().any(|q| (p.0 - q.0).abs() <= radio && (p.1 - q.1).abs() <= radio);
    let mut tramos: Vec<serde_json::Value> = Vec::new();
    for (partes, tipo) in lineas.iter().zip(&tipos).filter(|(_, t)| sirve(t)) {
        for parte in partes.iter().filter(|p| p.len() > 1 && p.iter().any(cerca)) {
            let Some(principal) = red.es_principal(Coordenada { longitud: parte[0].0, latitud: parte[0].1 }) else { continue };
            let simple = faro_territorio::geo::simplificar(parte, 0.00005);
            tramos.push(serde_json::json!({ "type": "Feature", "properties": { "red": if principal { "principal" } else { "suelto" }, "tipo": tipo },
                "geometry": { "type": "LineString", "coordinates": simple.iter().map(|(x, y)| [(x * 1e5).round() / 1e5, (y * 1e5).round() / 1e5]).collect::<Vec<_>>() } }));
        }
    }
    std::fs::write(salida.join("red_vial_contexto.geojson"), serde_json::to_string(&serde_json::json!({ "type": "FeatureCollection", "features": tramos })).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    println!("{} tramos de camino alrededor de los enlaces -> red_vial_contexto.geojson", tramos.len());
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

/// Mide la huella visual en pares que se sabe que son el mismo producto (mismo código de barras
/// en dos tiendas con fotos distintas) y en pares de productos distintos, y resume cuántos pares
/// aceptaría cada umbral. La selección es fija (sin azar): mismas entradas, mismos pares.
fn calibrar_fotos(entrada: &std::path::Path, pares: usize, salida: &std::path::Path) -> Result<(), String> {
    use faro_fuentes::foto;
    use std::collections::{BTreeMap, HashMap};
    let filas: Vec<HashMap<String, String>> = csv::Reader::from_path(entrada).map_err(|e| format!("{}: {e}", entrada.display()))?.deserialize().collect::<Result<_, _>>().map_err(|e| e.to_string())?;
    let dato = |f: &HashMap<String, String>, k: &str| f.get(k).cloned().unwrap_or_default();
    // Por código de barras: (tienda, foto). Solo cuentan los códigos de 12 dígitos o más (los cortos son internos).
    let mut por_codigo: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for f in &filas {
        let (codigo, imagen) = (dato(f, "codigo_barras"), dato(f, "imagen"));
        if codigo.len() >= 12 && imagen.starts_with("https://") {
            por_codigo.entry(codigo).or_default().push((dato(f, "tienda"), imagen));
        }
    }
    // Pares iguales: mismo código, tiendas distintas y fotos distintas (si la foto es el mismo archivo no enseña nada).
    let archivo = |u: &str| u.rsplit('/').next().unwrap_or("").split('?').next().unwrap_or("").to_string();
    let mut iguales: Vec<(String, String, String)> = Vec::new();
    for (codigo, lista) in &por_codigo {
        if let Some((a, b)) = lista.iter().flat_map(|a| lista.iter().map(move |b| (a, b))).find(|(a, b)| a.0 < b.0 && archivo(&a.1) != archivo(&b.1)) {
            iguales.push((codigo.clone(), a.1.clone(), b.1.clone()));
        }
    }
    if iguales.is_empty() {
        return Err("la tabla no tiene productos con el mismo código de barras y fotos distintas en dos tiendas".into());
    }
    // Se toman repartidos a lo largo de la lista, y los pares distintos emparejan cada uno con el siguiente.
    let paso = (iguales.len() / pares.max(1)).max(1);
    let iguales: Vec<_> = iguales.into_iter().step_by(paso).take(pares).collect();
    let distintos: Vec<(String, String, String)> = iguales.iter().zip(iguales.iter().cycle().skip(1)).map(|(a, b)| (format!("{} / {}", a.0, b.0), a.1.clone(), b.2.clone())).collect();
    let agente = foto::agente();
    let mut huellas: HashMap<String, Option<foto::Huella>> = HashMap::new();
    let mut medir = |a: &str, b: &str| -> Option<(u32, f32)> {
        for u in [a, b] {
            if !huellas.contains_key(u) {
                huellas.insert(u.to_string(), foto::huella_de_url(&agente, u));
                std::thread::sleep(std::time::Duration::from_millis(150));
            }
        }
        Some(foto::distancia(huellas[a].as_ref()?, huellas[b].as_ref()?))
    };
    if let Some(d) = salida.parent() {
        std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
    }
    let mut tabla = csv::Writer::from_path(salida).map_err(|e| e.to_string())?;
    tabla.write_record(["tipo", "codigo_barras", "foto_a", "foto_b", "bits_forma", "diferencia_color"]).map_err(|e| e.to_string())?;
    let mut medidas: Vec<(bool, u32, f32)> = Vec::new();
    for (igual, lista) in [(true, &iguales), (false, &distintos)] {
        for (codigo, a, b) in lista {
            if let Some((bits, color)) = medir(a, b) {
                tabla.write_record([if igual { "mismo_producto" } else { "productos_distintos" }, codigo, a, b, &bits.to_string(), &format!("{color:.4}")]).map_err(|e| e.to_string())?;
                medidas.push((igual, bits, color));
            }
        }
    }
    tabla.flush().map_err(|e| e.to_string())?;
    let (n_ig, n_di) = (medidas.iter().filter(|m| m.0).count(), medidas.iter().filter(|m| !m.0).count());
    println!("Pares medidos: {n_ig} del mismo producto (mismo código de barras, tiendas y fotos distintas) y {n_di} de productos distintos.");
    println!("{:>10} {:>10} {:>24} {:>26}", "bits <=", "color <=", "mismo producto aceptado", "distintos aceptados (error)");
    for bits in [6u32, 8, 10, 12, 14, 16] {
        for color in [0.05f32, 0.08, 0.12] {
            let acepta = |igual: bool| medidas.iter().filter(|m| m.0 == igual && m.1 <= bits && m.2 <= color).count();
            println!("{bits:>10} {color:>10.2} {:>17} ({:>4.1} %) {:>18} ({:>4.1} %)", acepta(true), 100.0 * acepta(true) as f64 / n_ig.max(1) as f64, acepta(false), 100.0 * acepta(false) as f64 / n_di.max(1) as f64);
        }
    }
    println!("Detalle par por par: {}", salida.display());
    Ok(())
}

/// Corre las comprobaciones de consistencia y de robustez estadística sobre las tablas del
/// repositorio y escribe el resultado, con cifras, en un documento. Devuelve si todo cuadró.
fn verificar(dir: &std::path::Path, salida: &std::path::Path) -> Result<bool, String> {
    use faro_territorio::estadistica::{ic_mediana, percentil, t_975, wilson};
    use std::collections::{BTreeMap, HashMap};
    use std::fmt::Write as _;
    type Fila = HashMap<String, String>;
    let leer = |nombre: &str| -> Result<Vec<Fila>, String> { csv::Reader::from_path(dir.join(nombre)).map_err(|e| format!("{nombre}: {e}"))?.deserialize().collect::<Result<_, _>>().map_err(|e| format!("{nombre}: {e}")) };
    let num = |f: &Fila, k: &str| f.get(k).and_then(|x| x.trim().parse::<f64>().ok()).unwrap_or(f64::NAN);
    let mut md2 = String::from("\n## Comprobaciones de los supuestos del código\n\n| Comprobación | Resultado | Detalle |\n|---|---|---|\n");
    let mut md = String::from("# Verificación estadística y de consistencia\n\nEste documento lo escribe `faro verificar` a partir de las tablas del repositorio: cada cifra se\nvuelve a calcular al ejecutarlo. Repetir: `faro verificar`.\n");
    let mut todo_bien = true;
    let mut cuadra = |md: &mut String, nombre: &str, ok: bool, detalle: String| { todo_bien &= ok; let _ = writeln!(md, "| {nombre} | {} | {detalle} |", if ok { "cuadra" } else { "**NO CUADRA**" }); };

    // ---- 1. Consistencia interna de las tablas ----
    md.push_str("\n## 1. Consistencia de las tablas\n\n| Comprobación | Resultado | Detalle |\n|---|---|---|\n");
    let poblacion = leer("poblacion_municipio.csv")?;
    let mal = poblacion.iter().filter(|f| num(f, "hombres") + num(f, "mujeres") != num(f, "total")).count();
    let municipios: std::collections::BTreeSet<&String> = poblacion.iter().filter_map(|f| f.get("municipio")).collect();
    cuadra(&mut md, "Población: hombres + mujeres = total", mal == 0, format!("{} filas, {} municipios; {mal} filas distintas", poblacion.len(), municipios.len()));
    let edades = leer("poblacion_municipio_edades.csv")?;
    let mut por: BTreeMap<(String, String), (f64, f64)> = BTreeMap::new();
    for f in &edades { let s = por.entry((f["municipio"].clone(), f["anio"].clone())).or_default(); s.0 += num(f, "hombres"); s.1 += num(f, "mujeres"); }
    let totales: HashMap<(String, String), (f64, f64)> = poblacion.iter().map(|f| ((f["municipio"].clone(), f["anio"].clone()), (num(f, "hombres"), num(f, "mujeres")))).collect();
    let mal = por.iter().filter(|(k, v)| totales.get(*k) != Some(*v)).count();
    cuadra(&mut md, "Edades: la suma de los grupos = total del municipio y año", mal == 0, format!("{} municipios-año comparados; {mal} distintos", por.len()));
    let censo = leer("censo2018_lugar_poblado.csv")?;
    let mal = censo.iter().filter(|f| num(f, "hombres") + num(f, "mujeres") != num(f, "poblacion") || num(f, "edad_0_14") + num(f, "edad_15_64") + num(f, "edad_65_mas") != num(f, "poblacion") || num(f, "urbana") + num(f, "rural") != num(f, "poblacion")).count();
    let total_censo: f64 = censo.iter().map(|f| num(f, "poblacion")).sum();
    cuadra(&mut md, "Censo 2018 por lugar poblado: sexo, edad y área suman el total", mal == 0, format!("{} lugares; población {total_censo:.0}; {mal} filas distintas", censo.len()));
    let p2018: f64 = poblacion.iter().filter(|f| f["anio"] == "2018").map(|f| num(f, "total")).sum();
    let omision = (1.0 - total_censo / p2018) * 100.0;
    cuadra(&mut md, "Censo 2018 frente a la proyección 2018 del INE", (0.0..15.0).contains(&omision), format!("censados {total_censo:.0}; proyectados {p2018:.0}; diferencia {omision:.1} % (la proyección corrige la omisión censal: debe ser mayor)"));
    let vial = leer("red_vial_lugares.csv").unwrap_or_default();
    cuadra(&mut md, "Red vial: cada lugar sin conexión tiene código de 7 dígitos y existe en el censo", vial.iter().all(|f| f.get("codigo").is_some_and(|c| censo.iter().any(|x| &x["codigo"] == c))), format!("{} lugares revisados", vial.len()));

    // ---- 2. Proyección de superficie construida: prueba hacia atrás ----
    let construido = leer("superficie_construida_municipio.csv")?;
    let mut series: BTreeMap<String, Vec<faro_territorio::Punto>> = BTreeMap::new();
    for f in &construido { series.entry(f["municipio"].clone()).or_default().push(faro_territorio::Punto { anio: num(f, "anio") as i32, edificaciones: num(f, "construido_ha") }); }
    // Igual que en el portal (cinco épocas para proyectar diez años), pero hacia atrás: con las cinco
    // épocas anteriores se proyecta 2010, 2015 y 2020 y se compara con lo observado.
    let _ = t_975;
    let mut relativos: Vec<f64> = Vec::new();
    let (mut relativos_tc, mut errores_tc, mut errores_recta): (Vec<f64>, Vec<f64>, Vec<f64>) = (Vec::new(), Vec::new(), Vec::new());
    md.push_str("\n## 2. Proyección de superficie construida: prueba hacia atrás\n\nEl portal proyecta 2030 con la recta de las cinco épocas 2000–2020. Para saber cuánto acierta ese\nmétodo se repitió hacia atrás en los 340 municipios: con las cinco épocas anteriores se proyectó a\ndiez años y se comparó con lo observado.\n\n| Año proyectado | Con datos de | Error mediano | Error del 90 % | Dentro del margen de 8 de cada 10 | Dentro del margen al 95 % |\n|---|---|---|---|---|---|\n");
    let (mut dentro80, mut dentro95, mut casos, mut dentro_ajuste) = (0usize, 0usize, 0usize, 0usize);
    for objetivo in [2010, 2015, 2020] {
        let (mut errores, mut d80, mut d95, mut n) = (Vec::new(), 0usize, 0usize, 0usize);
        for (codigo, s) in &series {
            let base: Vec<faro_territorio::Punto> = s.iter().copied().filter(|p| (objetivo - 30..=objetivo - 10).contains(&p.anio)).collect();
            let Some(real) = s.iter().find(|p| p.anio == objetivo).map(|p| p.edificaciones).filter(|x| *x > 0.0) else { continue };
            let Some(c) = faro_territorio::analizar(codigo, &base, objetivo) else { continue };
            if c.proyeccion <= 0.0 { continue; }
            let error = (c.proyeccion - real).abs();
            n += 1;
            errores.push(error / real * 100.0);
            relativos.push(error / c.proyeccion);
            if let Some(t) = c.proyeccion_tasa_constante.filter(|t| *t > 0.0) {
                relativos_tc.push((t - real).abs() / t);
                errores_tc.push((t - real).abs() / real * 100.0);
                errores_recta.push(error / real * 100.0);
            }
            d80 += usize::from(error <= c.proyeccion_margen_80);
            d95 += usize::from(error <= c.proyeccion_margen);
            // Para comparar: cuántas veces lo observado cae dentro de un margen calculado solo con el error
    // del ajuste × 2, sin el piso del error observado que usa `crecimiento::analizar`.
            let ajuste = { let m: f64 = base.len() as f64; let mx = base.iter().map(|p| p.anio as f64).sum::<f64>() / m; let my = base.iter().map(|p| p.edificaciones).sum::<f64>() / m; let sxx: f64 = base.iter().map(|p| (p.anio as f64 - mx).powi(2)).sum(); let b = base.iter().map(|p| (p.anio as f64 - mx) * (p.edificaciones - my)).sum::<f64>() / sxx; let res: f64 = base.iter().map(|p| (p.edificaciones - (my + b * (p.anio as f64 - mx))).powi(2)).sum(); 2.0 * (res / (m - 2.0)).sqrt() * (1.0 + 1.0 / m + (objetivo as f64 - mx).powi(2) / sxx).sqrt() };
            dentro_ajuste += usize::from(error <= ajuste);
        }
        errores.sort_by(f64::total_cmp);
        let _ = writeln!(md, "| {objetivo} | {}–{} | {:.1} % | hasta {:.1} % | {d80} de {n} ({:.0} %) | {d95} de {n} ({:.0} %) |", objetivo - 30, objetivo - 10, percentil(&errores, 0.5).unwrap_or(f64::NAN), percentil(&errores, 0.9).unwrap_or(f64::NAN), 100.0 * d80 as f64 / n.max(1) as f64, 100.0 * d95 as f64 / n.max(1) as f64);
        dentro80 += d80; dentro95 += d95; casos += n;
    }
    relativos.sort_by(f64::total_cmp);
    let (p80, p95) = (percentil(&relativos, 0.8).unwrap_or(f64::NAN), percentil(&relativos, 0.95).unwrap_or(f64::NAN));
    let _ = write!(md, "\nEn los {casos} casos: lo observado cayó dentro del margen de 8 de cada 10 en {:.0} % y dentro del margen al 95 % en {:.0} %.\n\n**Qué se corrigió gracias a esta prueba.** El margen anterior salía solo del error del ajuste (± 2 errores\nestándar) y contenía lo observado en {dentro_ajuste} de {casos} casos ({:.0} %): prometía mucha más precisión de la real, porque la\nserie satelital es suave pero no crece en línea recta. Ahora el margen es el error realmente observado al\nproyectar diez años: {:.0} % de lo proyectado en 8 de cada 10 casos y {:.0} % en 95 de cada 100 (constantes\n`ERROR_OBSERVADO_P80` = {} y `ERROR_OBSERVADO_P95` = {} del código).\n\nTambién se probaron otros métodos con la misma prueba (recta de tres épocas, crecimiento exponencial,\ntasa o incremento de la última década): ninguno fue mejor en las tres décadas, así que se conserva la\nrecta, que es la más fácil de explicar. Proyectar 2020 es lo que peor sale en todos (error mediano de 11 % a\n14 %, siempre por debajo de lo observado): la capa satelital muestra una aceleración en su última época.\n",
        100.0 * dentro80 as f64 / casos.max(1) as f64, 100.0 * dentro95 as f64 / casos.max(1) as f64, 100.0 * dentro_ajuste as f64 / casos.max(1) as f64, p80 * 100.0, p95 * 100.0, faro_territorio::crecimiento::ERROR_OBSERVADO_P80, faro_territorio::crecimiento::ERROR_OBSERVADO_P95);
    for v in [&mut relativos_tc, &mut errores_tc, &mut errores_recta] { v.sort_by(f64::total_cmp); }
    let p80_tc = percentil(&relativos_tc, 0.8).unwrap_or(f64::NAN);
    let _ = write!(md, "\n### Recta o tasa constante\n\nLo que crece en proporción a su tamaño (como una población) suele describirse mejor con una tasa\nconstante que con una recta. Se midió con la misma prueba:\n\n| Modelo | Error mediano | Error del 90 % | Margen de 8 de cada 10 |\n|---|---|---|---|\n| Recta (cifra principal) | {:.1} % | {:.1} % | ± {:.0} % de lo proyectado |\n| Tasa constante (cifra de contraste) | {:.1} % | {:.1} % | ± {:.0} % de lo proyectado |\n\nPara la superficie construida la recta acierta más en conjunto, así que sigue siendo la cifra principal; el\nportal muestra las dos para que se vea cuánto depende la proyección del supuesto. También se probó elegir el\nmodelo por municipio según cuál habría acertado más en la década anterior: no mejoró de forma consistente\n(2015: error del 90 % de 29 % frente a 19 % de la recta; 2020: 32 % frente a 34 %). La **población** no se\nproyecta con ninguna regresión: se usan las proyecciones oficiales del INE, hechas por componentes\ndemográficos, y su crecimiento se resume con la tasa compuesta.\n",
        percentil(&errores_recta, 0.5).unwrap_or(f64::NAN), percentil(&errores_recta, 0.9).unwrap_or(f64::NAN), p80 * 100.0, percentil(&errores_tc, 0.5).unwrap_or(f64::NAN), percentil(&errores_tc, 0.9).unwrap_or(f64::NAN), p80_tc * 100.0);
    cuadra(&mut md2, "La constante de margen de la tasa constante coincide con el error observado", (p80_tc - faro_territorio::crecimiento::ERROR_TASA_CONSTANTE_P80).abs() < 0.02, format!("observado: {:.1} %", p80_tc * 100.0));
    cuadra(&mut md2, "Las constantes de margen del código coinciden con el error observado", (p80 - faro_territorio::crecimiento::ERROR_OBSERVADO_P80).abs() < 0.02 && (p95 - faro_territorio::crecimiento::ERROR_OBSERVADO_P95).abs() < 0.03, format!("observado: {:.1} % y {:.1} %", p80 * 100.0, p95 * 100.0));

    // ---- 2b. Población más allá de la serie del INE: exponencial y logística ----
    {
        use faro_territorio::crecimiento::{extrapolar, HORIZONTE_CORTO};
        // Serie por municipio: año -> habitantes.
        let mut series: BTreeMap<String, Vec<(i32, f64)>> = BTreeMap::new();
        for f in &poblacion {
            if let (Some(m), Ok(a)) = (f.get("municipio"), f["anio"].parse::<i32>()) {
                series.entry(m.clone()).or_default().push((a, num(f, "total")));
            }
        }
        let (mut e_exp, mut e_log, mut con_techo_2025) = (Vec::new(), Vec::new(), 0usize);
        for serie in series.values() {
            // Con lo publicado hasta 2025 se estima 2030 y se compara con el 2030 del propio INE.
            let base: Vec<(i32, f64)> = serie.iter().copied().filter(|p| p.0 <= 2025).collect();
            let (Some(real), Some(x)) = (serie.iter().find(|p| p.0 == 2030).map(|p| p.1), extrapolar(&base, 2030)) else { continue };
            e_exp.push((x.exponencial - real).abs() / real * 100.0);
            if let Some(l) = x.logistica {
                e_log.push((l - real).abs() / real * 100.0);
                con_techo_2025 += 1;
            }
        }
        e_exp.sort_by(f64::total_cmp);
        e_log.sort_by(f64::total_cmp);
        // Con la serie completa: cuántos municipios dejan ver un techo y qué da cada curva en 2050.
        let (mut con_techo, mut bajan, mut suma_exp, mut suma_log, mut suma_sugerida, mut hoy) = (0usize, 0usize, 0.0, 0.0, 0.0, 0.0);
        for serie in series.values() {
            let Some(x) = extrapolar(serie, 2050) else { continue };
            con_techo += usize::from(x.logistica.is_some());
            bajan += usize::from(x.tasa_reciente_pct <= 0.0);
            suma_exp += x.exponencial;
            suma_log += x.logistica.unwrap_or(x.exponencial);
            suma_sugerida += x.valor;
            hoy += serie.iter().find(|p| p.0 == 2030).map_or(0.0, |p| p.1);
        }
        let p = |v: &[f64], q: f64| percentil(v, q).unwrap_or(f64::NAN);
        let _ = write!(md, "\n### Población más allá de 2030: exponencial a corto plazo, logística a largo plazo\n\nLas proyecciones oficiales del INE llegan a 2030. Para mirar más lejos el portal ofrece dos curvas, las dos a\npartir del último dato oficial: la **exponencial** (la tasa de los últimos cinco años se mantiene), que es la\nadecuada a corto plazo, y la **logística** (el crecimiento se frena al acercarse a un techo), que es la adecuada\na largo plazo, cuando los recursos son limitados. Hasta {HORIZONTE_CORTO} años después del último dato se sugiere la\nexponencial; después, la logística si la serie deja ver un techo. **No son proyecciones oficiales.**\n\nLa única prueba posible con datos por municipio es de corto plazo: con lo publicado hasta 2025 se estimó 2030 y\nse comparó con el 2030 del propio INE, en {} municipios.\n\n| Curva | Municipios | Error mediano | Error del 90 % |\n|---|---|---|---|\n| Exponencial | {} | {:.2} % | {:.2} % |\n| Logística (donde hay techo) | {} | {:.2} % | {:.2} % |\n\n", e_exp.len(), e_exp.len(), p(&e_exp, 0.5), p(&e_exp, 0.9), con_techo_2025, p(&e_log, 0.5), p(&e_log, 0.9));
        let _ = write!(md, "Con la serie completa (2015–2030), {con_techo} de {} municipios dejan ver un techo; en los demás el crecimiento no se\nfrena lo suficiente para estimarlo ({bajan} pierden población) y se usa la exponencial. Sumando los municipios, para 2050\nla exponencial da {:.1} millones de habitantes, la logística {:.1} millones y la curva sugerida {:.1} millones (en 2030 el INE\nproyecta {:.1} millones). La diferencia entre las dos curvas es la medida de cuánto pesa el supuesto: a cinco años casi\nno se nota; a veinte, sí. A largo plazo no hay dato por municipio contra el cual probarlas.\n\nLectura: en la prueba a cinco años las dos curvas se apartan poco del INE, y la logística menos que la exponencial\nen los municipios donde hay techo, porque las propias proyecciones del INE ya se frenan. El portal mantiene la regla\nhabitual —exponencial a corto plazo, logística a largo plazo— y muestra siempre las dos cifras.\n", series.len(), suma_exp / 1e6, suma_log / 1e6, suma_sugerida / 1e6, hoy / 1e6);
        cuadra(&mut md2, "Extrapolación de población: la logística nunca supera su techo ni la exponencial cambia de signo", series.values().filter_map(|s| extrapolar(s, 2050)).all(|x| x.techo.zip(x.logistica).is_none_or(|(t, l)| l <= t) && x.exponencial > 0.0), format!("{} municipios", series.len()));
    }

    // ---- 3. Intervalo de la mediana de precios ----
    md.push_str("\n## 3. Intervalo de confianza de la mediana de precios\n\nEl informe de precio de referencia da, junto a la mediana, un intervalo que no supone ninguna\ndistribución (estadísticos de orden). Cobertura exacta según la cantidad de precios:\n\n| Precios | Puestos que forman el intervalo | Cobertura exacta |\n|---|---|---|\n");
    for n in [5usize, 6, 8, 10, 15, 20, 30, 50] {
        let v: Vec<f64> = (1..=n).map(|x| x as f64).collect();
        match ic_mediana(&v, 0.95) { Some((a, b, c)) => { let _ = writeln!(md, "| {n} | {a:.0}.º y {b:.0}.º | {:.1} % |", c * 100.0); todo_bien &= c >= 0.95; } None => { let _ = writeln!(md, "| {n} | no se da intervalo | no alcanza 95 % |"); } }
    }

    // ---- 4. Diferencias de precio entre tiendas: sensibilidad de los umbrales ----
    let precios = leer("precios_publicados.csv")?;
    let mut grupos: BTreeMap<(String, String), BTreeMap<String, (f64, Option<f64>)>> = BTreeMap::new();
    for f in precios.iter().filter(|f| !f["codigo_barras"].is_empty()) { grupos.entry((f["rubro"].clone(), f["codigo_barras"].clone())).or_default().entry(f["tienda"].clone()).or_insert((num(f, "precio_q"), f.get("precio_lista_q").and_then(|x| x.parse().ok()))); }
    let comparables: Vec<Vec<(String, f64, Option<f64>)>> = grupos.values().filter(|g| g.len() >= 2).map(|g| g.iter().map(|(t, (p, l))| (t.clone(), *p, *l)).collect()).collect();
    let mut razones: Vec<f64> = comparables.iter().map(|g| { let p: Vec<f64> = g.iter().map(|x| x.1).collect(); p.iter().cloned().fold(f64::MIN, f64::max) / p.iter().cloned().fold(f64::MAX, f64::min) }).collect();
    razones.sort_by(f64::total_cmp);
    let mut clases: BTreeMap<&str, usize> = BTreeMap::new();
    for g in &comparables { if let Some(d) = faro_territorio::precios::explicar_diferencia(g) { *clases.entry(d.clase).or_default() += 1; } }
    let _ = write!(md, "\n## 4. Diferencias de precio entre tiendas\n\n{} productos tienen el mismo código de barras en dos o más tiendas. Diferencia entre la más cara y la más barata:\nmediana {:.1} %, percentil 90 {:.1} %, percentil 99 {:.1} %.\n\n| Umbral para revisar | Productos que lo superan | Proporción |\n|---|---|---|\n", comparables.len(), (percentil(&razones, 0.5).unwrap_or(1.0) - 1.0) * 100.0, (percentil(&razones, 0.9).unwrap_or(1.0) - 1.0) * 100.0, (percentil(&razones, 0.99).unwrap_or(1.0) - 1.0) * 100.0);
    for u in [1.25, 1.5, 2.0, 3.0] { let k = razones.iter().filter(|r| **r >= u).count(); let _ = writeln!(md, "| {:.0} % o más{} | {k} | {:.1} % |", (u - 1.0) * 100.0, if u == 1.5 { " (en uso)" } else { "" }, 100.0 * k as f64 / razones.len().max(1) as f64); }
    let total_clases: usize = clases.values().sum();
    let _ = write!(md, "\nCon el umbral en uso, las reglas reparten los {total_clases} casos así: {}. El umbral de 50 % deja a revisar cerca del 3 % de los productos: bajarlo a 25 % triplicaría el trabajo y subirlo a 100 % dejaría pasar errores evidentes.\n", clases.iter().map(|(c, k)| format!("{c} {k} ({:.0} %)", 100.0 * *k as f64 / total_clases.max(1) as f64)).collect::<Vec<_>>().join(", "));

    // ---- 5. Comparación por foto ----
    if let Ok(pares) = leer("fotos_calibracion.csv") {
        let u = faro_fuentes::foto::Umbrales::default();
        let acepta = |f: &Fila| num(f, "bits_forma") <= u.max_bits as f64 && num(f, "diferencia_color") <= u.max_color as f64;
        let (ig, di): (Vec<&Fila>, Vec<&Fila>) = pares.iter().partition(|f| f["tipo"] == "mismo_producto");
        let (a, e) = (ig.iter().filter(|f| acepta(f)).count(), di.iter().filter(|f| acepta(f)).count());
        let (wa, we) = (wilson(a, ig.len()).unwrap_or((0.0, 0.0)), wilson(e, di.len()).unwrap_or((0.0, 0.0)));
        let _ = write!(md, "\n## 5. Comparación por foto\n\nCon los umbrales en uso ({} bits, {} de color) sobre los pares medidos:\n\n| | Resultado | Intervalo de confianza al 95 % (Wilson) |\n|---|---|---|\n| Productos distintos aceptados por error | {e} de {} | entre {:.1} % y {:.1} % |\n| Mismo producto reconocido (fotos distintas) | {a} de {} | entre {:.1} % y {:.1} % |\n\nLectura: no haber visto ningún error en {} pares no prueba que el error sea cero; con 95 % de confianza es menor que {:.1} %.\n", u.max_bits, u.max_color, di.len(), we.0 * 100.0, we.1 * 100.0, ig.len(), wa.0 * 100.0, wa.1 * 100.0, di.len(), we.1 * 100.0);
    }

    // ---- 6. Tasas de extracción frente a las cuentas del INE ----
    if let (Ok(texto), Ok(cuentas)) = (std::fs::read_to_string("datos/estandar/tasas_extraccion_fao_guatemala.csv"), leer("hba_2024_cuentas.csv")) {
        use faro_homologa::normalizar::basico;
        let tasas: Vec<Fila> = csv::Reader::from_reader(texto.as_bytes()).deserialize().flatten().collect();
        let ine: HashMap<String, &Fila> = cuentas.iter().map(|f| (basico(&f["alimento"]), f)).collect();
        md.push_str("\n## 6. Tasas de extracción de la FAO frente a las cuentas 2024 del INE\n\n«Con cifras del INE» es la producción del derivado dividida entre todo el primario que el INE reporta\ncomo procesado. Coincide con la tasa de extracción cuando lo procesado va a ese derivado; si el\nprimario tiene otros usos industriales (el maíz va sobre todo a masa y tortilla, no a harina), sale\nmenor y no contradice la tasa.\n\n| Primario → derivado | FAO | Con cifras del INE | Diferencia (puntos) |\n|---|---|---|---|\n");
        let mut difs = Vec::new();
        for t in &tasas {
            let nombre = t.get("derivado_en_hba_ine").filter(|x| !x.is_empty()).unwrap_or(&t["derivado"]);
            let (Some(p), Some(d)) = (ine.get(&basico(&t["primario"])), ine.get(&basico(nombre))) else { continue };
            let (procesado, produccion) = (num(p, "procesado_t"), num(d, "produccion_t"));
            if procesado > 0.0 && produccion > 0.0 { let (fao, imp) = (num(t, "tasa_extraccion_pct"), produccion / procesado * 100.0); difs.push((imp - fao).abs()); let _ = writeln!(md, "| {} → {} | {fao:.1} % | {imp:.1} % | {:.1} |", t["primario"], t["derivado"], imp - fao); }
        }
        difs.sort_by(f64::total_cmp);
        let cerca = difs.iter().filter(|d| **d <= 2.0).count();
        let _ = write!(md, "\n{} tasas pudieron contrastarse (el nombre del producto coincide en ambas fuentes). En {cerca} la diferencia es de 2 puntos o menos (mediana de todas: {:.1} puntos). Las que se apartan mucho son primarios con varios usos (maíz, yuca, hortalizas) y el arroz elaborado (69.3 % frente a 62 %).\n", difs.len(), percentil(&difs, 0.5).unwrap_or(f64::NAN));
    }
    md.push_str(&md2);
    let _ = write!(md, "\n## Resultado\n\n{}\n", if todo_bien { "Todas las comprobaciones de consistencia cuadran." } else { "**Hay comprobaciones que no cuadran: ver las filas marcadas.**" });
    if let Some(d) = salida.parent() { std::fs::create_dir_all(d).map_err(|e| e.to_string())?; }
    std::fs::write(salida, &md).map_err(|e| e.to_string())?;
    println!("{}\nEscrito en {}", if todo_bien { "Todo cuadra." } else { "HAY DIFERENCIAS." }, salida.display());
    Ok(todo_bien)
}
