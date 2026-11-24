//! Servidor de FARO: una sola aplicación que recibe envíos, muestra lo pendiente, guarda las
//! equivalencias confirmadas y se mantiene al día con el portal del INE.
use axum::extract::{DefaultBodyLimit, Multipart, Path as Ruta, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use faro_homologa::lectura;
use faro_homologa::recepcion::{recibir, Recepcion};
use faro_homologa::solicitud::Solicitud;
use faro_homologa::{Catalogo, Equivalencia, EquivalenciaValor, Homologador, Memoria, Nivel};
use faro_territorio::poblacion::Poblacion;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::Arc;
use tokio_postgres::{Client, NoTls};

struct Estado {
    bd: Client,
    catalogo: Catalogo,
    /// Proyecciones de población del INE por municipio y año (denominador de tasas).
    poblacion: Poblacion,
    dir_ejemplos: PathBuf,
    /// Resultado del análisis de la red vial (`faro vial`), ya con nombres del catálogo.
    vial: Value,
    /// Retardo en talla en escolares por municipio (Censo de Talla, SESAN): código -> fila.
    talla: HashMap<String, HashMap<String, String>>,
    /// Inseguridad alimentaria aguda por departamento (CIF, SESAN).
    cif: Vec<HashMap<String, String>>,
    /// Catálogo de países del diccionario de estadísticas vitales del INE: código -> nombre.
    paises: HashMap<String, String>,
    /// Tareas de muestreo de precios (datos/estandar/muestreos.toml).
    tareas: Vec<faro_fuentes::muestreo::Tarea>,
    /// Canasta básica: (periodo, producto, precio promedio del INE).
    canasta: Vec<(String, String, f64)>,
    /// Bloques de 1 km con superficie construida por época, por municipio (para la animación).
    bloques: HashMap<String, Vec<Vec<f64>>>,
    epocas: Vec<i32>,
    dir_scripts: PathBuf,
    /// Precios de referencia publicados por el INE (tabla limpia de `faro referencias`).
    referencias: Vec<HashMap<String, String>>,
    /// Fuentes y citas (datos/estandar/fuentes.json).
    fuentes: Value,
    /// Materiales de construcción: reglas del IPMC, artículos, precios publicados y proveedores.
    construccion: Construccion,
    /// Superficie construida (hectáreas) por municipio y época, de GHSL: código -> [(año, ha)].
    construido: BTreeMap<String, Vec<faro_territorio::Punto>>,
    /// Capas geográficas listas para publicar: nombre de archivo -> (tipo de contenido, texto).
    capas: HashMap<String, (&'static str, String)>,
    /// Descripción de cada capa (título, fuente, licencia) para el catálogo.
    catalogo_capas: Vec<Value>,
    dir_solicitudes: PathBuf,
    dir_privado: PathBuf,
    token: Option<String>,
    sal: Option<String>,
    portal: String,
}
type Compartido = Arc<Estado>;

/// Error de la API: mensaje claro para quien usa la pantalla.
struct Fallo(StatusCode, String);
impl IntoResponse for Fallo {
    fn into_response(self) -> Response {
        (self.0, Json(json!({ "error": self.1 }))).into_response()
    }
}
fn interno(e: impl std::fmt::Display) -> Fallo {
    eprintln!("error interno: {e}");
    Fallo(StatusCode::INTERNAL_SERVER_ERROR, "error interno; revise la bitácora del servidor".into())
}
fn pedido(e: impl std::fmt::Display) -> Fallo {
    Fallo(StatusCode::BAD_REQUEST, e.to_string())
}
type Res<T> = Result<T, Fallo>;

fn var(nombre: &str, defecto: &str) -> String {
    std::env::var(nombre).ok().filter(|v| !v.is_empty()).unwrap_or_else(|| defecto.to_string())
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let url = var("DATABASE_URL", "host=localhost user=faro password=faro dbname=faro");
    // La base puede tardar unos segundos en aceptar conexiones al levantar con Docker.
    let mut intento = 0;
    let (bd, conexion) = loop {
        match tokio_postgres::connect(&url, NoTls).await {
            Ok(par) => break par,
            Err(e) if intento < 30 => {
                intento += 1;
                eprintln!("esperando la base de datos ({intento}/30): {e}");
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
            Err(e) => return Err(format!("no se pudo conectar a la base de datos: {e}")),
        }
    };
    tokio::spawn(async move {
        if let Err(e) = conexion.await {
            eprintln!("se perdió la conexión a la base de datos: {e}");
            std::process::exit(1); // Docker reinicia el servicio
        }
    });
    bd.batch_execute(include_str!("esquema.sql")).await.map_err(|e| format!("esquema: {e}"))?;

    // Sin la tabla de población el servidor funciona igual; solo no calcula tasas.
    let ruta_poblacion = PathBuf::from(var("FARO_POBLACION", "datos/referencia/poblacion_municipio.csv"));
    let poblacion = match faro_territorio::poblacion::leer(&ruta_poblacion) {
        Ok(filas) => Poblacion::new(filas),
        Err(e) => {
            eprintln!("AVISO: sin proyecciones de población ({e}); no se calcularán tasas.");
            Poblacion::default()
        }
    };
    let catalogo = Catalogo::cargar_carpeta(&PathBuf::from(var("FARO_CATALOGO", "datos/catalogo")))?;
    let vial = cargar_vial(&PathBuf::from(var("FARO_REFERENCIA", "datos/referencia")), &catalogo).unwrap_or_else(|e| {
        eprintln!("AVISO: sin análisis de red vial ({e}).");
        json!({ "disponible": false })
    });
    let dir_referencia = PathBuf::from(var("FARO_REFERENCIA", "datos/referencia"));
    match sembrar_precios(&bd, &dir_referencia.join("precios_publicados.csv")).await {
        Ok(0) => {}
        Ok(n) => eprintln!("Precios publicados del levantamiento inicial: {n} cargados."),
        Err(e) => eprintln!("AVISO: no se cargó el levantamiento inicial de precios ({e})."),
    }
    let mut construido: BTreeMap<String, Vec<faro_territorio::Punto>> = BTreeMap::new();
    if let Ok(mut lector) = csv::Reader::from_path(dir_referencia.join("superficie_construida_municipio.csv")) {
        for f in lector.deserialize::<HashMap<String, String>>().flatten() {
            if let (Some(m), Some(Ok(anio)), Some(Ok(ha))) = (f.get("municipio"), f.get("anio").map(|a| a.parse::<i32>()), f.get("construido_ha").map(|h| h.parse::<f64>())) {
                construido.entry(m.clone()).or_default().push(faro_territorio::Punto { anio, edificaciones: ha });
            }
        }
    }
    // Bloques construidos: municipio -> filas [longitud, latitud, ha por época...].
    let (mut bloques, mut epocas): (HashMap<String, Vec<Vec<f64>>>, Vec<i32>) = (HashMap::new(), Vec::new());
    if let Ok(mut lector) = csv::Reader::from_path(dir_referencia.join("superficie_construida_bloques.csv")) {
        epocas = lector.headers().map(|h| h.iter().filter_map(|c| c.strip_prefix("ha_")?.parse().ok()).collect()).unwrap_or_default();
        for f in lector.records().flatten() {
            let numeros: Vec<f64> = f.iter().skip(1).filter_map(|c| c.parse().ok()).collect();
            if numeros.len() == epocas.len() + 2 {
                bloques.entry(f[0].to_string()).or_default().push(numeros);
            }
        }
    }
    let cif: Vec<HashMap<String, String>> = csv::Reader::from_path(dir_referencia.join("cif_departamentos.csv")).map(|mut l| l.deserialize().flatten().collect()).unwrap_or_default();
    let talla: HashMap<String, HashMap<String, String>> = csv::Reader::from_path(dir_referencia.join("censo_talla_municipio.csv")).map(|mut l| l.deserialize::<HashMap<String, String>>().flatten().filter_map(|f| Some((f.get("municipio")?.clone(), f))).collect()).unwrap_or_default();
    let construccion = Construccion::cargar(&dir_referencia, &catalogo);
    let (capas, catalogo_capas) = armar_capas(&dir_referencia, &catalogo, &poblacion, &vial, &construido, &construccion, &cif, &talla);
    // Primer arranque: un administrador inicial (admin / admin) que debe cambiar su contraseña.
    if bd.query_one("SELECT count(*) FROM usuarios", &[]).await.map_err(|e| e.to_string())?.get::<_, i64>(0) == 0 {
        bd.execute("INSERT INTO usuarios (usuario, nombre, rol, clave, debe_cambiar) VALUES ('admin', 'Administrador', 'admin', $1, TRUE)", &[&huella_de_clave("admin")?]).await.map_err(|e| e.to_string())?;
        eprintln!("AVISO: se creó el usuario inicial admin con contraseña admin. Cámbiela al entrar.");
    }
    let estado = Arc::new(Estado {
        bd,
        vial,
        referencias: csv::Reader::from_path(var("FARO_REFERENCIAS_INE", "datos/catalogo_precios/precios_referencia_ine.csv")).map(|mut l| l.deserialize().flatten().collect()).unwrap_or_default(),
        cif: cif.clone(),
        talla: talla.clone(),
        paises: csv::Reader::from_path(var("FARO_PAISES", "datos/estandar/paises_ine.csv")).map(|mut l| l.deserialize::<(String, String)>().flatten().collect()).unwrap_or_default(),
        tareas: faro_fuentes::muestreo::cargar_tareas(&PathBuf::from(var("FARO_MUESTREOS", "datos/estandar/muestreos.toml"))).unwrap_or_else(|e| {
            eprintln!("AVISO: sin tareas de muestreo ({e}).");
            vec![]
        }),
        canasta: csv::Reader::from_path(var("FARO_CANASTA", "datos/catalogo_precios/cba_precios.csv")).map(|mut l| l.deserialize::<HashMap<String, String>>().flatten().filter_map(|f| Some((f.get("periodo")?.clone(), f.get("producto")?.clone(), f.get("precio_promedio")?.parse().ok()?))).collect()).unwrap_or_default(),
        bloques,
        epocas,
        dir_scripts: PathBuf::from(var("FARO_SCRIPTS", "scripts/reproducir")),
        fuentes: std::fs::read_to_string(var("FARO_FUENTES", "datos/estandar/fuentes.json")).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_else(|| json!({ "fuentes": [] })),
        construccion,
        construido,
        capas,
        catalogo_capas,
        poblacion,
        dir_ejemplos: PathBuf::from(var("FARO_EJEMPLOS", "datos/ejemplos")),
        catalogo,
        dir_solicitudes: PathBuf::from(var("FARO_SOLICITUDES", "datos/solicitudes")),
        dir_privado: PathBuf::from(var("FARO_PRIVADO", "datos/privado")),
        token: std::env::var("FARO_TOKEN").ok().filter(|t| !t.is_empty()),
        sal: std::env::var("FARO_SAL").ok().filter(|t| !t.is_empty()),
        portal: var("FARO_PORTAL_INE", faro_fuentes::INE_CKAN),
    });
    if estado.token.is_none() {
        eprintln!("FARO_TOKEN no está definido: no hay clave de servicio; los cambios requieren iniciar sesión.");
    }

    // Muestreos de precios: cada diez minutos se revisa qué tareas activas ya tocan.
    {
        let e = estado.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(600)).await;
                for t in e.tareas.clone() {
                    let toca = e.bd.query_opt("SELECT 1 FROM muestreos WHERE id = $1 AND activa AND (ultima IS NULL OR ultima < now() - make_interval(hours => cada_horas))", &[&t.id]).await;
                    if matches!(toca, Ok(Some(_))) {
                        let r = ejecutar_muestreo(&e, &t).await;
                        eprintln!("muestreo {}: {}", t.id, r.map(|v| v.to_string()).unwrap_or_else(|f| f.1));
                    }
                }
            }
        });
    }

    // Conexión automática con el INE: al arrancar y luego cada FARO_SYNC_HORAS.
    let horas: u64 = var("FARO_SYNC_HORAS", "24").parse().unwrap_or(24);
    if horas > 0 {
        let e = estado.clone();
        tokio::spawn(async move {
            loop {
                let r = sincronizar_ine(&e).await;
                eprintln!("sincronización con el INE: {}", r.map(|v| v.to_string()).unwrap_or_else(|f| f.1));
                tokio::time::sleep(std::time::Duration::from_secs(horas * 3600)).await;
            }
        });
    }

    let app = Router::new()
        .route("/", get(|| async { Html(include_str!("../web/index.html")) }))
        .route("/api/estado", get(api_estado))
        .route("/api/solicitudes", get(api_solicitudes))
        .route("/api/catalogo/:nivel", get(api_catalogo))
        .route("/api/envios", get(api_envios))
        .route("/api/envios/:solicitud", post(api_enviar))
        .route("/api/pendientes", get(api_pendientes))
        .route("/api/confirmar", post(api_confirmar))
        .route("/api/equivalencias", get(api_equivalencias))
        .route("/api/hechos", get(api_hechos))
        .route("/api/ine", get(api_ine))
        .route("/api/ine/sincronizar", post(api_sincronizar))
        .route("/api/edificaciones", post(api_edificaciones))
        .route("/api/crecimiento", get(api_crecimiento))
        .route("/api/asignar", post(api_asignar))
        .route("/api/poblacion", get(api_poblacion))
        .route("/api/construido", get(api_construido))
        .route("/api/construccion", get(api_construccion))
        .route("/api/referencias", get(api_referencias))
        .route("/api/proceso", get(api_proceso))
        .route("/api/fuentes", get(|State(e): State<Compartido>| async move { Json(e.fuentes.clone()) }))
        .route("/api/reproducir/poblacion", post(api_reproducir_poblacion))
        .route("/api/sesion", get(api_sesion).post(api_ingresar).delete(api_salir))
        .route("/api/perfil", post(api_perfil))
        .route("/api/usuarios", get(api_usuarios).post(api_usuario_crear))
        .route("/api/usuarios/:id", post(api_usuario_cambiar))
        .route("/api/cif", get(|State(e): State<Compartido>| async move {
            let mut talla: Vec<Value> = e.talla.values().map(|f| { let mut v = json!(f); v["departamento"] = json!(f.get("municipio").and_then(|m| e.catalogo.por_codigo(&m[..2])).map(|d| d.nombre.clone())); v }).collect();
            talla.sort_by(|a, b| b["retardo_en_talla_pct"].as_str().and_then(|x| x.parse::<f64>().ok()).unwrap_or(0.0).total_cmp(&a["retardo_en_talla_pct"].as_str().and_then(|x| x.parse::<f64>().ok()).unwrap_or(0.0)));
            Json(json!({ "disponible": !e.cif.is_empty(), "departamentos": e.cif, "talla": talla }))
        }))
        .route("/api/muestreos", get(api_muestreos))
        .route("/api/muestreos/:id", post(api_muestreo_configurar))
        .route("/api/muestreos/:id/ejecutar", post(api_muestreo_ejecutar))
        .route("/api/canasta", get(api_canasta))
        .route("/api/publicados", get(api_publicados))
        .route("/api/publicados.csv", get(api_publicados_csv))
        .route("/api/municipio/:codigo", get(api_municipio))
        .route("/api/solicitudes/:id/plantilla.csv", get(plantilla))
        .route("/api/equivalencias/tabla", post(api_tabla_equivalencias))
        .route("/scripts/:archivo", get(script))
        .route("/reporte", get(|| async { Html(include_str!("../web/reporte.html")) }))
        .route("/api/vial", get(|State(e): State<Compartido>| async move { Json(e.vial.clone()) }))
        .route("/api/precios", post(api_precios))
        .route("/api/avance", get(api_avance))
        .route("/ejemplos/:archivo", get(ejemplo))
        .route("/capas", get(api_capas))
        .route("/capas/:archivo", get(capa))
        .route("/api/registros_municipio", get(api_registros_municipio))
        .route("/vendor/leaflet.js", get(|| async { ([("content-type", "text/javascript; charset=utf-8"), ("cache-control", "public, max-age=86400")], include_str!("../web/vendor/leaflet.js")) }))
        .route("/vendor/leaflet.css", get(|| async { ([("content-type", "text/css; charset=utf-8"), ("cache-control", "public, max-age=86400")], include_str!("../web/vendor/leaflet.css")) }))
        .layer(DefaultBodyLimit::max(300 * 1024 * 1024))
        .with_state(estado);

    let puerto = var("FARO_PUERTO", "8080");
    let oyente = tokio::net::TcpListener::bind(format!("0.0.0.0:{puerto}")).await.map_err(|e| e.to_string())?;
    eprintln!("FARO escuchando en http://0.0.0.0:{puerto}");
    axum::serve(oyente, app)
        .with_graceful_shutdown(async { let _ = tokio::signal::ctrl_c().await; })
        .await
        .map_err(|e| e.to_string())
}

/// Las operaciones que cambian datos exigen el token cuando está configurado.
/// Quién hace la operación. Las operaciones que cambian datos exigen una sesión de analista o
/// administrador; la clave de servicio (FARO_TOKEN) queda para procesos automáticos.
async fn autorizar(e: &Estado, cabeceras: &HeaderMap) -> Res<Sesion> {
    if let Some(s) = sesion_de(e, cabeceras).await? {
        return if s.rol == "consulta" { Err(Fallo(StatusCode::FORBIDDEN, "su usuario es de consulta: no puede hacer cambios".into())) } else { Ok(s) };
    }
    match &e.token {
        Some(t) if cabeceras.get("x-faro-token").and_then(|v| v.to_str().ok()) == Some(t.as_str()) => Ok(Sesion { id: 0, usuario: "servicio".into(), nombre: "Clave de servicio".into(), rol: "analista".into(), debe_cambiar: false }),
        _ => Err(Fallo(StatusCode::UNAUTHORIZED, "inicie sesión para hacer cambios".into())),
    }
}

async fn memoria(e: &Estado) -> Res<Memoria> {
    let filas = e.bd.query("SELECT original, clave, nivel, contexto, codigo FROM equivalencias ORDER BY id", &[]).await.map_err(interno)?;
    let valores = e.bd.query("SELECT dimension, original, clave, valor FROM equivalencias_valor ORDER BY id", &[]).await.map_err(interno)?;
    let valores: Vec<EquivalenciaValor> = valores
        .iter()
        .map(|f| EquivalenciaValor { dimension: f.get(0), original: f.get(1), clave: f.get(2), valor: f.get(3), institucion: String::new(), confirmado_por: String::new(), fecha: String::new() })
        .collect();
    Ok(Memoria::desde(filas.iter().filter_map(|f| {
        Some(Equivalencia {
            original: f.get(0),
            clave: f.get(1),
            nivel: f.get::<_, String>(2).parse().ok()?,
            contexto: f.get(3),
            codigo: f.get(4),
            institucion: String::new(),
            confirmado_por: String::new(),
            fecha: String::new(),
        })
    }))
    .con_valores(valores))
}

fn solicitudes(e: &Estado) -> Result<Vec<Solicitud>, String> {
    let mut rutas: Vec<PathBuf> = std::fs::read_dir(&e.dir_solicitudes)
        .map_err(|x| format!("{}: {x}", e.dir_solicitudes.display()))?
        .filter_map(|x| x.ok().map(|x| x.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "toml"))
        .collect();
    rutas.sort();
    rutas.iter().map(|r| Solicitud::cargar(r)).collect()
}

async fn api_estado(State(e): State<Compartido>) -> Res<Json<Value>> {
    let f = e
        .bd
        .query_one(
            "SELECT (SELECT count(*) FROM envios), (SELECT coalesce(sum(filas),0)::bigint FROM envios),
                    (SELECT count(*) FROM (SELECT 1 FROM pendientes WHERE NOT resuelto GROUP BY columna, nivel, contexto, valor) p),
                    (SELECT count(*) FROM equivalencias) + (SELECT count(*) FROM equivalencias_valor),
                    (SELECT count(*) FROM conjuntos_ine), (SELECT count(DISTINCT municipio) FROM edificaciones)",
            &[],
        )
        .await
        .map_err(interno)?;
    Ok(Json(json!({
        "envios": f.get::<_, i64>(0), "filas": f.get::<_, i64>(1), "pendientes": f.get::<_, i64>(2),
        "equivalencias": f.get::<_, i64>(3), "conjuntos_ine": f.get::<_, i64>(4), "municipios_con_edificaciones": f.get::<_, i64>(5),
        "lugares_en_catalogo": e.catalogo.len(), "requiere_token": e.token.is_some(),
    })))
}

async fn api_solicitudes(State(e): State<Compartido>) -> Res<Json<Value>> {
    Ok(Json(json!(solicitudes(&e).map_err(interno)?)))
}

async fn api_catalogo(State(e): State<Compartido>, Ruta(nivel): Ruta<String>, Query(q): Query<HashMap<String, String>>) -> Res<Json<Value>> {
    let nivel: Nivel = nivel.parse().map_err(pedido)?;
    let padre = q.get("padre").map(String::as_str).filter(|p| !p.is_empty());
    Ok(Json(json!(e.catalogo.de_nivel(nivel, padre).map(|l| json!({"codigo": l.codigo, "nombre": l.nombre, "padre": l.padre, "padre_nombre": e.catalogo.por_codigo(&l.padre).map(|p| p.nombre.clone())})).collect::<Vec<_>>())))
}

async fn api_envios(State(e): State<Compartido>) -> Res<Json<Value>> {
    let filas = e
        .bd
        .query("SELECT id, solicitud, institucion, archivo, to_char(recibido, 'YYYY-MM-DD HH24:MI'), filas, pct_resuelto, problemas FROM envios ORDER BY id DESC LIMIT 100", &[])
        .await
        .map_err(interno)?;
    Ok(Json(json!(filas.iter().map(|f| json!({
        "id": f.get::<_, i64>(0), "solicitud": f.get::<_, String>(1), "institucion": f.get::<_, String>(2), "archivo": f.get::<_, String>(3),
        "recibido": f.get::<_, String>(4), "filas": f.get::<_, i64>(5), "pct_resuelto": f.get::<_, f64>(6), "problemas": f.get::<_, i64>(7),
    })).collect::<Vec<_>>())))
}

/// Conteos por periodo (AAAA-MM), municipio y sexo a partir de la salida estándar.
fn agregar(rec: &Recepcion) -> BTreeMap<(String, String, String), i64> {
    let pos = |n: &str| rec.encabezados.iter().position(|e| e == n);
    let (fecha, muni, sexo) = (pos("fecha_hecho"), pos("municipio_codigo"), pos("sexo"));
    let mut conteo = BTreeMap::new();
    for f in &rec.filas {
        let periodo = fecha.and_then(|i| f[i].get(..7)).unwrap_or("").to_string();
        let m = muni.map(|i| f[i].clone()).unwrap_or_default();
        let s = sexo.map(|i| f[i].clone()).unwrap_or_default();
        *conteo.entry((periodo, m, s)).or_insert(0) += 1;
    }
    conteo
}

async fn api_enviar(State(e): State<Compartido>, Ruta(id): Ruta<String>, cabeceras: HeaderMap, mut partes: Multipart) -> Res<Json<Value>> {
    autorizar(&e, &cabeceras).await?;
    let sol = solicitudes(&e).map_err(interno)?.into_iter().find(|s| s.id == id).ok_or_else(|| pedido(format!("no existe la solicitud {id}")))?;
    let mut archivo = None;
    while let Some(parte) = partes.next_field().await.map_err(pedido)? {
        if parte.name() == Some("archivo") {
            let nombre = parte.file_name().unwrap_or("envio.csv").to_string();
            archivo = Some((nombre, parte.bytes().await.map_err(pedido)?));
        }
    }
    let (nombre, bytes) = archivo.ok_or_else(|| pedido("falta el campo 'archivo'"))?;
    let ext = nombre.rsplit('.').next().unwrap_or("csv").to_lowercase();
    if !matches!(ext.as_str(), "csv" | "txt" | "xlsx" | "xlsm" | "xls" | "ods") {
        return Err(pedido("formato no admitido: envíe CSV o Excel"));
    }
    let mem = memoria(&e).await?;

    // El archivo original solo vive en un temporal mientras se procesa.
    let temporal = std::env::temp_dir().join(format!("faro-{}-{}.{ext}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0)));
    tokio::fs::write(&temporal, &bytes).await.map_err(interno)?;
    let (e2, sol2, temporal2) = (e.clone(), sol.clone(), temporal.clone());
    let resultado = tokio::task::spawn_blocking(move || {
        let tabla = lectura::leer(&temporal2, sol2.hoja.as_deref(), &sol2.nombres_esperados())?;
        recibir(&sol2, &tabla, &Homologador::new(&e2.catalogo, &mem), e2.sal.as_deref())
    })
    .await
    .map_err(interno)?;
    let _ = tokio::fs::remove_file(&temporal).await;
    let rec = resultado.map_err(pedido)?;

    let problemas = (rec.reporte.problemas.len() + rec.reporte.problemas_omitidos) as i64;
    let reporte = serde_json::to_value(&rec).map_err(interno)?;
    let fila = e
        .bd
        .query_one(
            "INSERT INTO envios (solicitud, institucion, archivo, filas, pct_resuelto, problemas, reporte) VALUES ($1,$2,$3,$4,$5,$6,$7) RETURNING id",
            &[&sol.id, &sol.institucion, &nombre, &(rec.reporte.filas as i64), &rec.porcentaje_resuelto(), &problemas, &reporte],
        )
        .await
        .map_err(interno)?;
    let envio: i64 = fila.get(0);
    for p in &rec.pendientes {
        e.bd.execute(
            "INSERT INTO pendientes (envio_id, institucion, columna, nivel, contexto, valor, veces, estado, candidatos) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)",
            &[&envio, &sol.institucion, &p.columna, &serde_json::to_value(p.nivel).ok().and_then(|v| v.as_str().map(str::to_string)).unwrap_or_default(), &p.contexto, &p.valor, &(p.veces as i64), &p.resultado.estado.texto(), &json!(p.resultado.candidatos)],
        )
        .await
        .map_err(interno)?;
    }
    // Valores de catálogos cerrados por decidir: van a la misma lista, con nivel "valor".
    for p in &rec.pendientes_valor {
        let opciones: Vec<Value> = p.opciones.iter().map(|o| json!({ "codigo": o, "nombre": o, "puntaje": 0.0 })).collect();
        e.bd.execute(
            "INSERT INTO pendientes (envio_id, institucion, columna, nivel, contexto, valor, veces, estado, candidatos) VALUES ($1,$2,$3,'valor',$4,$5,$6,'fuera_de_catalogo',$7)",
            &[&envio, &sol.institucion, &p.columna, &p.dimension, &p.valor, &(p.veces as i64), &json!(opciones)],
        )
        .await
        .map_err(interno)?;
    }
    for ((periodo, municipio, sexo), n) in agregar(&rec) {
        e.bd.execute("INSERT INTO hechos (envio_id, solicitud, periodo, municipio, sexo, n) VALUES ($1,$2,$3,$4,$5,$6)", &[&envio, &sol.id, &periodo, &municipio, &sexo, &n]).await.map_err(interno)?;
    }
    // Salida estándar fila por fila: al volumen privado, nunca al repositorio.
    let dir = e.dir_privado.join("envios").join(envio.to_string());
    let rec = tokio::task::spawn_blocking(move || -> Result<Recepcion, String> {
        std::fs::create_dir_all(&dir).map_err(|x| x.to_string())?;
        let mut w = csv::Writer::from_path(dir.join("homologado.csv")).map_err(|x| x.to_string())?;
        w.write_record(&rec.encabezados).map_err(|x| x.to_string())?;
        for f in &rec.filas {
            w.write_record(f).map_err(|x| x.to_string())?;
        }
        w.flush().map_err(|x| x.to_string())?;
        Ok(rec)
    })
    .await
    .map_err(interno)?
    .map_err(interno)?;
    Ok(Json(json!({ "envio": envio, "solicitud": sol.id, "archivo": nombre, "porcentaje_resuelto": rec.porcentaje_resuelto(), "recepcion": rec })))
}

async fn api_pendientes(State(e): State<Compartido>) -> Res<Json<Value>> {
    // Un mismo valor puede venir en varios envíos: se muestra una vez, con el total.
    let filas = e
        .bd
        .query(
            "SELECT columna, nivel, contexto, valor, sum(veces)::bigint, max(estado), (array_agg(candidatos ORDER BY id DESC))[1], string_agg(DISTINCT institucion, ', ')
             FROM pendientes WHERE NOT resuelto GROUP BY columna, nivel, contexto, valor ORDER BY 5 DESC LIMIT 500",
            &[],
        )
        .await
        .map_err(interno)?;
    Ok(Json(json!(filas.iter().map(|f| {
        let ctx: String = f.get(2);
        let pista = pista_de_codigo(&e, &f.get::<_, String>(1), &f.get::<_, String>(3));
        json!({
            "pista": pista,
            "columna": f.get::<_, String>(0), "nivel": f.get::<_, String>(1), "contexto": ctx,
            "contexto_nombre": e.catalogo.por_codigo(&ctx).map(|l| l.nombre.clone()),
            "valor": f.get::<_, String>(3), "veces": f.get::<_, i64>(4), "estado": f.get::<_, String>(5),
            "candidatos": f.get::<_, Value>(6), "instituciones": f.get::<_, String>(7),
        })
    }).collect::<Vec<_>>())))
}

#[derive(Deserialize)]
struct Confirmacion {
    valor: String,
    /// departamento | municipio | lugar_poblado | valor (catálogo cerrado; `contexto` es la dimensión).
    nivel: String,
    #[serde(default)]
    contexto: String,
    codigo: String,
    #[serde(default)]
    institucion: String,
    #[serde(default)]
    usuario: String,
}

async fn api_confirmar(State(e): State<Compartido>, cabeceras: HeaderMap, Json(c): Json<Confirmacion>) -> Res<Json<Value>> {
    // En la bitácora queda quien inició sesión, no un nombre escrito a mano.
    let quien = autorizar(&e, &cabeceras).await?;
    let mut c = c;
    if quien.id != 0 || c.usuario.trim().is_empty() {
        c.usuario = quien.nombre;
    }
    if c.nivel == "valor" && c.contexto == faro_homologa::productos::DIMENSION {
        // Decisión de producto → artículo del IPMC ("ninguno" = no corresponde a ningún artículo).
        if c.codigo != "ninguno" && !c.codigo.parse::<u32>().is_ok_and(|n| e.construccion.reglas.articulos.iter().any(|r| r.codigo == n)) {
            return Err(pedido(format!("{:?} no es un artículo del IPMC con regla", c.codigo)));
        }
        e.bd.execute(
            "INSERT INTO equivalencias_valor (dimension, original, clave, valor, institucion, confirmado_por) VALUES ($1,$2,$3,$4,$5,$6)",
            &[&c.contexto, &c.valor, &faro_homologa::normalizar::basico(&c.valor), &c.codigo, &c.institucion, &c.usuario],
        )
        .await
        .map_err(interno)?;
        return Ok(Json(json!({ "grabado": true, "codigo": c.codigo, "nombre": "(IPMC)", "pendientes_resueltos": 0 })));
    }
    if c.nivel == "valor" {
        // El valor elegido debe pertenecer al catálogo de esa dimensión en alguna solicitud.
        let permitido = solicitudes(&e).map_err(interno)?.iter().flat_map(|s| s.columnas.clone()).filter(|col| col.dimension() == c.contexto).find_map(|col| col.canonico(&c.codigo).map(str::to_string));
        let valor = permitido.ok_or_else(|| pedido(format!("{:?} no es un valor permitido de {}", c.codigo, c.contexto)))?;
        e.bd.execute(
            "INSERT INTO equivalencias_valor (dimension, original, clave, valor, institucion, confirmado_por) SELECT $1,$2,$3,$4,$5,$6
             WHERE NOT EXISTS (SELECT 1 FROM equivalencias_valor x WHERE x.id = (SELECT max(id) FROM equivalencias_valor y WHERE y.dimension = $1 AND y.clave = $3) AND x.valor = $4)",
            &[&c.contexto, &c.valor, &faro_homologa::normalizar::basico(&c.valor), &valor, &c.institucion, &c.usuario],
        )
        .await
        .map_err(interno)?;
        let n = e.bd.execute("UPDATE pendientes SET resuelto = TRUE WHERE NOT resuelto AND nivel = 'valor' AND contexto = $1 AND valor = $2", &[&c.contexto, &c.valor]).await.map_err(interno)?;
        return Ok(Json(json!({ "grabado": true, "codigo": valor, "nombre": format!("({})", c.contexto), "pendientes_resueltos": n })));
    }
    let nivel_pedido: Nivel = c.nivel.parse().map_err(pedido)?;
    let lugar = e.catalogo.por_codigo(&c.codigo).ok_or_else(|| pedido(format!("el código {} no existe en el catálogo", c.codigo)))?;
    if lugar.nivel != nivel_pedido {
        return Err(pedido(format!("{} ({}) no es del nivel indicado", c.codigo, lugar.nombre)));
    }
    let nivel = c.nivel.clone();
    let clave = faro_homologa::normalizar::clave(&c.valor);
    // Repetir la misma confirmación (doble clic) no duplica la bitácora.
    e.bd.execute(
        "INSERT INTO equivalencias (original, clave, nivel, contexto, codigo, institucion, confirmado_por) SELECT $1,$2,$3,$4,$5,$6,$7
         WHERE NOT EXISTS (SELECT 1 FROM equivalencias x WHERE x.id = (SELECT max(id) FROM equivalencias y WHERE y.nivel = $3 AND y.contexto = $4 AND y.clave = $2) AND x.codigo = $5)",
        &[&c.valor, &clave, &nivel, &c.contexto, &c.codigo, &c.institucion, &c.usuario],
    )
    .await
    .map_err(interno)?;
    let n = e.bd.execute("UPDATE pendientes SET resuelto = TRUE WHERE NOT resuelto AND nivel = $1 AND contexto = $2 AND valor = $3", &[&nivel, &c.contexto, &c.valor]).await.map_err(interno)?;
    Ok(Json(json!({ "grabado": true, "codigo": lugar.codigo, "nombre": lugar.nombre, "pendientes_resueltos": n })))
}

async fn api_equivalencias(State(e): State<Compartido>) -> Res<Json<Value>> {
    let filas = e
        .bd
        .query(
            "SELECT original, nivel, contexto, codigo, institucion, confirmado_por, to_char(fecha, 'YYYY-MM-DD HH24:MI') FROM (
                SELECT original, nivel, contexto, codigo, institucion, confirmado_por, fecha FROM equivalencias
                UNION ALL SELECT original, 'valor', dimension, valor, institucion, confirmado_por, fecha FROM equivalencias_valor) t
             ORDER BY fecha DESC LIMIT 500",
            &[],
        )
        .await
        .map_err(interno)?;
    Ok(Json(json!(filas.iter().map(|f| {
        let codigo: String = f.get(3);
        json!({ "original": f.get::<_, String>(0), "nivel": f.get::<_, String>(1), "contexto": f.get::<_, String>(2), "codigo": codigo,
                "nombre": e.catalogo.por_codigo(&codigo).map(|l| l.nombre.clone()), "institucion": f.get::<_, String>(4),
                "confirmado_por": f.get::<_, String>(5), "fecha": f.get::<_, String>(6) })
    }).collect::<Vec<_>>())))
}

async fn api_hechos(State(e): State<Compartido>) -> Res<Json<Value>> {
    // Solo el envío más reciente de cada solicitud y periodo, para no contar dos veces un reenvío.
    let filas = e
        .bd
        .query(
            "WITH ultimo AS (SELECT solicitud, periodo, max(envio_id) AS envio_id FROM hechos GROUP BY 1, 2)
             SELECT h.solicitud, left(h.municipio, 2), sum(h.n)::bigint, sum(h.n) FILTER (WHERE h.sexo = 'hombre')::bigint, sum(h.n) FILTER (WHERE h.sexo = 'mujer')::bigint,
                    max(left(h.periodo, 4)), count(DISTINCT h.periodo) FILTER (WHERE h.periodo <> '')
             FROM hechos h JOIN ultimo u USING (solicitud, periodo, envio_id) GROUP BY 1, 2 ORDER BY 1, 2",
            &[],
        )
        .await
        .map_err(interno)?;
    Ok(Json(json!(filas.iter().map(|f| {
        let d: String = f.get(1);
        let total: i64 = f.get(2);
        // Denominador: proyección del INE para el departamento en el año más reciente recibido.
        let anio: Option<i32> = f.get::<_, Option<String>>(5).and_then(|a| a.parse().ok());
        let poblacion = anio.filter(|_| d.len() == 2).and_then(|a| e.poblacion.total(&d, a));
        json!({ "solicitud": f.get::<_, String>(0), "departamento": d, "nombre": e.catalogo.por_codigo(&d).map(|l| l.nombre.clone()).unwrap_or_else(|| "(sin lugar)".into()),
                "total": total, "hombres": f.get::<_, Option<i64>>(3), "mujeres": f.get::<_, Option<i64>>(4),
                "anio": anio, "meses": f.get::<_, i64>(6), "poblacion": poblacion,
                "por_100_mil": poblacion.filter(|p| *p > 0).map(|p| total as f64 * 100_000.0 / p as f64) })
    }).collect::<Vec<_>>())))
}

/// Avance de la recepción: qué ha enviado cada institución, cuándo y con qué calidad.
async fn api_avance(State(e): State<Compartido>) -> Res<Json<Value>> {
    let filas = e
        .bd
        .query(
            "SELECT e.institucion, e.solicitud, count(*), sum(e.filas)::bigint, to_char(max(e.recibido), 'YYYY-MM-DD HH24:MI'),
                    (array_agg(e.pct_resuelto ORDER BY e.id DESC))[1], (array_agg(e.problemas ORDER BY e.id DESC))[1],
                    (SELECT count(*) FROM (SELECT 1 FROM pendientes p JOIN envios x ON x.id = p.envio_id WHERE NOT p.resuelto AND x.solicitud = e.solicitud GROUP BY p.columna, p.nivel, p.contexto, p.valor) q)
             FROM envios e GROUP BY e.institucion, e.solicitud ORDER BY 1, 2",
            &[],
        )
        .await
        .map_err(interno)?;
    let recibidas: Vec<String> = filas.iter().map(|f| f.get(1)).collect();
    let mut salida: Vec<Value> = filas.iter().map(|f| json!({
        "institucion": f.get::<_, String>(0), "solicitud": f.get::<_, String>(1), "envios": f.get::<_, i64>(2), "filas": f.get::<_, i64>(3),
        "ultimo": f.get::<_, String>(4), "pct_resuelto": f.get::<_, f64>(5), "problemas": f.get::<_, i64>(6), "pendientes": f.get::<_, i64>(7),
    })).collect();
    // Lo que se pidió y todavía no llega también es avance.
    for s in solicitudes(&e).map_err(interno)?.iter().filter(|s| !recibidas.contains(&s.id)) {
        salida.push(json!({ "institucion": s.institucion, "solicitud": s.id, "periodicidad": s.periodicidad, "envios": 0, "filas": 0, "ultimo": null, "pct_resuelto": null, "problemas": 0, "pendientes": 0 }));
    }
    Ok(Json(json!(salida)))
}

/// Crecimiento de la población por municipio entre dos años, según las proyecciones del INE.
async fn api_poblacion(State(e): State<Compartido>, Query(q): Query<HashMap<String, String>>) -> Res<Json<Value>> {
    let Some((min, max)) = e.poblacion.anios() else {
        return Ok(Json(json!({ "disponible": false, "municipios": [] })));
    };
    let anio = |k: &str, defecto: i32| q.get(k).and_then(|a| a.parse::<i32>().ok()).unwrap_or(defecto).clamp(min, max);
    let (desde, hasta) = (anio("desde", 2018), anio("hasta", max));
    if hasta <= desde {
        return Err(pedido(format!("el año final debe ser mayor que el inicial (hay datos de {min} a {max})")));
    }
    let mut municipios: Vec<Value> = e
        .poblacion
        .municipios()
        .iter()
        .filter_map(|m| e.poblacion.cambio(m, desde, hasta))
        .map(|c| {
            let l = e.catalogo.por_codigo(&c.codigo);
            let mut v = serde_json::to_value(&c).unwrap_or(Value::Null);
            v["municipio"] = json!(l.map(|l| l.nombre.clone()));
            v["departamento"] = json!(l.and_then(|l| e.catalogo.por_codigo(&l.padre)).map(|d| d.nombre.clone()));
            v
        })
        .collect();
    municipios.sort_by(|a, b| b["tasa_anual_pct"].as_f64().unwrap_or(0.0).total_cmp(&a["tasa_anual_pct"].as_f64().unwrap_or(0.0)));
    Ok(Json(json!({ "disponible": true, "desde": desde, "hasta": hasta, "anio_minimo": min, "anio_maximo": max, "pais": e.poblacion.cambio("", desde, hasta), "municipios": municipios })))
}

/// Archivos de ejemplo (ficticios) para probar cada pantalla sin tener datos propios.
async fn ejemplo(State(e): State<Compartido>, Ruta(archivo): Ruta<String>) -> Response {
    let limpio = archivo.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.')) && !archivo.contains("..") && archivo.ends_with(".csv");
    match (limpio, tokio::fs::read(e.dir_ejemplos.join(&archivo)).await) {
        (true, Ok(bytes)) => ([("content-type", "text/csv; charset=utf-8".to_string()), ("content-disposition", format!("attachment; filename=\"{archivo}\""))], bytes).into_response(),
        _ => Fallo(StatusCode::NOT_FOUND, "no existe ese ejemplo".into()).into_response(),
    }
}

async fn sincronizar_ine(e: &Compartido) -> Res<Value> {
    let portal = e.portal.clone();
    let inventario = tokio::task::spawn_blocking(move || faro_fuentes::Ckan::new(&portal).inventario()).await.map_err(interno)?;
    let conjuntos = match inventario {
        Ok(c) => c,
        Err(detalle) => {
            e.bd.execute("INSERT INTO sincronizaciones (ok, detalle) VALUES (FALSE, $1)", &[&detalle]).await.map_err(interno)?;
            return Err(Fallo(StatusCode::BAD_GATEWAY, format!("no se pudo consultar el portal del INE: {detalle}")));
        }
    };
    let (mut nuevos, mut cambiados) = (0i32, 0i32);
    for c in &conjuntos {
        let mut formatos: Vec<String> = c.resources.iter().map(|r| r.format.to_uppercase()).filter(|f| !f.is_empty()).collect();
        formatos.sort();
        formatos.dedup();
        let previo = e.bd.query_opt("SELECT modificado FROM conjuntos_ine WHERE nombre = $1", &[&c.name]).await.map_err(interno)?;
        match previo {
            None => nuevos += 1,
            Some(f) if f.get::<_, String>(0) != c.metadata_modified => cambiados += 1,
            Some(_) => continue,
        }
        e.bd.execute(
            "INSERT INTO conjuntos_ine (nombre, titulo, licencia, modificado, recursos, formatos) VALUES ($1,$2,$3,$4,$5,$6)
             ON CONFLICT (nombre) DO UPDATE SET titulo = $2, licencia = $3, modificado = $4, recursos = $5, formatos = $6, cambio = now()",
            &[&c.name, &c.title, &c.licencia(), &c.metadata_modified, &(c.resources.len() as i32), &formatos.join(", ")],
        )
        .await
        .map_err(interno)?;
    }
    e.bd.execute("INSERT INTO sincronizaciones (ok, conjuntos, nuevos, cambiados) VALUES (TRUE, $1, $2, $3)", &[&(conjuntos.len() as i32), &nuevos, &cambiados]).await.map_err(interno)?;
    Ok(json!({ "conjuntos": conjuntos.len(), "nuevos": nuevos, "cambiados": cambiados }))
}

async fn api_sincronizar(State(e): State<Compartido>, cabeceras: HeaderMap) -> Res<Json<Value>> {
    autorizar(&e, &cabeceras).await?;
    Ok(Json(sincronizar_ine(&e).await?))
}

async fn api_ine(State(e): State<Compartido>) -> Res<Json<Value>> {
    let conjuntos = e.bd.query("SELECT nombre, titulo, licencia, left(modificado, 10), recursos, formatos, to_char(cambio, 'YYYY-MM-DD') FROM conjuntos_ine ORDER BY modificado DESC", &[]).await.map_err(interno)?;
    let ultima = e.bd.query_opt("SELECT to_char(inicio, 'YYYY-MM-DD HH24:MI'), ok, conjuntos, nuevos, cambiados, detalle FROM sincronizaciones ORDER BY id DESC LIMIT 1", &[]).await.map_err(interno)?;
    Ok(Json(json!({
        "ultima": ultima.map(|f| json!({ "cuando": f.get::<_, String>(0), "ok": f.get::<_, bool>(1), "conjuntos": f.get::<_, i32>(2), "nuevos": f.get::<_, i32>(3), "cambiados": f.get::<_, i32>(4), "detalle": f.get::<_, String>(5) })),
        "conjuntos": conjuntos.iter().map(|f| json!({ "nombre": f.get::<_, String>(0), "titulo": f.get::<_, String>(1), "licencia": f.get::<_, String>(2),
            "modificado": f.get::<_, String>(3), "recursos": f.get::<_, i32>(4), "formatos": f.get::<_, String>(5), "detectado": f.get::<_, String>(6) })).collect::<Vec<_>>(),
    })))
}

/// Carga la serie de edificaciones. Acepta dos formas de CSV:
/// - `codigo, anio, edificaciones`: el código INE del municipio, que debe existir en el catálogo;
/// - `departamento, municipio, anio, edificaciones`: nombres, que se homologan al código INE con
///   el mismo motor que los envíos.
async fn api_edificaciones(State(e): State<Compartido>, cabeceras: HeaderMap, cuerpo: String) -> Res<Json<Value>> {
    autorizar(&e, &cabeceras).await?;
    let mem = memoria(&e).await?;
    let h = Homologador::new(&e.catalogo, &mem);
    let mut lector = csv::Reader::from_reader(cuerpo.as_bytes());
    let cab: Vec<String> = lector.headers().map_err(pedido)?.iter().map(|c| c.trim().to_lowercase()).collect();
    let pos = |n: &str| cab.iter().position(|c| c == n).ok_or_else(|| pedido(format!("falta la columna {n}")));
    let (ia, ie) = (pos("anio")?, pos("edificaciones")?);
    let ic = cab.iter().position(|c| c == "codigo");
    let nombres = match ic {
        Some(_) => None,
        None => Some((pos("departamento")?, pos("municipio")?)),
    };
    let (mut cargadas, mut sin_lugar) = (0, BTreeMap::<String, usize>::new());
    for fila in lector.records() {
        let f = fila.map_err(pedido)?;
        let celda = |i: usize| f.get(i).unwrap_or("").trim();
        let (codigo, etiqueta) = match (ic, nombres) {
            (Some(i), _) => {
                let c = celda(i);
                (e.catalogo.por_codigo(c).filter(|l| l.nivel == Nivel::Municipio).map(|l| l.codigo.clone()), format!("código {c}"))
            }
            (None, Some((id, im))) => {
                let depto = h.homologar(celda(id), Nivel::Departamento, None);
                (h.homologar(celda(im), Nivel::Municipio, depto.codigo.as_deref()).codigo, format!("{} / {}", celda(id), celda(im)))
            }
            (None, None) => (None, String::new()),
        };
        let (Some(codigo), Ok(anio), Ok(valor)) = (codigo, celda(ia).parse::<i32>(), celda(ie).parse::<f64>()) else {
            *sin_lugar.entry(etiqueta).or_default() += 1;
            continue;
        };
        e.bd.execute("INSERT INTO edificaciones (municipio, anio, edificaciones) VALUES ($1,$2,$3) ON CONFLICT (municipio, anio) DO UPDATE SET edificaciones = $3", &[&codigo, &anio, &valor]).await.map_err(interno)?;
        cargadas += 1;
    }
    Ok(Json(json!({ "filas_cargadas": cargadas, "sin_lugar_reconocido": sin_lugar })))
}

async fn api_crecimiento(State(e): State<Compartido>, Query(q): Query<HashMap<String, String>>) -> Res<Json<Value>> {
    let anio: i32 = q.get("anio").and_then(|a| a.parse().ok()).unwrap_or(2030);
    if !(2024..=2050).contains(&anio) {
        return Err(pedido("el año de proyección debe estar entre 2024 y 2050"));
    }
    let filas = e.bd.query("SELECT municipio, anio, edificaciones FROM edificaciones ORDER BY 1, 2", &[]).await.map_err(interno)?;
    let mut series: BTreeMap<String, Vec<faro_territorio::Punto>> = BTreeMap::new();
    for f in &filas {
        series.entry(f.get(0)).or_default().push(faro_territorio::Punto { anio: f.get(1), edificaciones: f.get(2) });
    }
    let mut salida: Vec<Value> = series
        .iter()
        .filter_map(|(codigo, s)| faro_territorio::analizar(codigo, s, anio))
        .map(|c| {
            let l = e.catalogo.por_codigo(&c.codigo);
            let mut v = serde_json::to_value(&c).unwrap_or(Value::Null);
            v["municipio"] = json!(l.map(|l| l.nombre.clone()));
            v["departamento"] = json!(l.and_then(|l| e.catalogo.por_codigo(&l.padre)).map(|d| d.nombre.clone()));
            v
        })
        .collect();
    salida.sort_by(|a, b| b["tasa_anual_pct"].as_f64().unwrap_or(0.0).total_cmp(&a["tasa_anual_pct"].as_f64().unwrap_or(0.0)));
    Ok(Json(json!(salida)))
}

#[derive(Deserialize)]
struct PedidoAsignacion {
    /// Texto CSV: operador, lugar (o longitud y latitud).
    operadores: String,
    /// Texto CSV: puesto, lugar (o longitud y latitud), operadores.
    puestos: String,
}

/// Reparte operadores entre puestos con la menor distancia total. Dónde vive cada operador es un
/// dato personal: se usa para el cálculo y se devuelve el resultado, pero no se guarda nada.
async fn api_asignar(State(e): State<Compartido>, cabeceras: HeaderMap, Json(p): Json<PedidoAsignacion>) -> Res<Json<Value>> {
    use faro_territorio::asignacion::{self, Coordenada};
    autorizar(&e, &cabeceras).await?;
    let resultado = tokio::task::spawn_blocking(move || {
        let donde = |codigo: &str| faro_homologa::censo::centroide(&e.catalogo, codigo).map(|(longitud, latitud)| Coordenada { longitud, latitud });
        let operadores = asignacion::leer_operadores(p.operadores.as_bytes(), &donde).map_err(|x| format!("operadores: {x}"))?;
        let puestos = asignacion::leer_puestos(p.puestos.as_bytes(), &donde).map_err(|x| format!("puestos: {x}"))?;
        let plazas: usize = puestos.iter().map(|x| x.operadores).sum();
        let a = asignacion::asignar(&operadores, &puestos)?;
        Ok::<_, String>(json!({ "operadores": operadores.len(), "puestos": puestos.len(), "plazas": plazas, "asignacion": a }))
    })
    .await
    .map_err(interno)?;
    Ok(Json(resultado.map_err(pedido)?))
}

/// Precio de referencia por producto. Calcula y devuelve; no guarda las cotizaciones.
async fn api_precios(State(e): State<Compartido>, cabeceras: HeaderMap, cuerpo: String) -> Res<Json<Value>> {
    autorizar(&e, &cabeceras).await?;
    let p = faro_territorio::precios::Parametros::default();
    let productos = faro_territorio::precios::analizar_csv(cuerpo.as_bytes(), &p).map_err(pedido)?;
    Ok(Json(json!({ "parametros": p, "productos": productos })))
}

/// Lee lo que dejó `faro vial` y le pone nombres: resumen, conteo por departamento, enlaces
/// probables y lugares sin camino mapeado (los más lejanos primero).
fn cargar_vial(dir: &std::path::Path, catalogo: &Catalogo) -> Result<Value, String> {
    let leer = |nombre: &str| -> Result<Vec<HashMap<String, String>>, String> {
        let ruta = dir.join(nombre);
        let mut lector = csv::Reader::from_path(&ruta).map_err(|e| format!("{}: {e}", ruta.display()))?;
        lector.deserialize().collect::<Result<_, _>>().map_err(|e| format!("{}: {e}", ruta.display()))
    };
    let resumen: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("red_vial_resumen.json")).map_err(|e| format!("red_vial_resumen.json: {e}"))?).map_err(|e| e.to_string())?;
    let nombre = |codigo: &str| catalogo.por_codigo(codigo).map(|l| l.nombre.clone()).unwrap_or_default();
    let lugar = |codigo: &str| json!({ "codigo": codigo, "nombre": nombre(codigo), "municipio": nombre(&codigo[..4.min(codigo.len())]), "departamento": nombre(&codigo[..2.min(codigo.len())]) });
    let num = |f: &HashMap<String, String>, k: &str| f.get(k).and_then(|v| v.parse::<f64>().ok());

    let lugares = leer("red_vial_lugares.csv")?;
    // Por departamento: cuántos lugares con coordenadas tiene y cuántos quedaron sin conexión.
    let mut por_depto: BTreeMap<String, [usize; 3]> = BTreeMap::new();
    for l in catalogo.de_nivel(Nivel::LugarPoblado, None).filter(|l| l.longitud.is_some()) {
        por_depto.entry(l.codigo[..2].to_string()).or_default()[0] += 1;
    }
    for l in &lugares {
        let codigo = l.get("codigo").map(String::as_str).unwrap_or("");
        if codigo.len() >= 2 {
            por_depto.entry(codigo[..2].to_string()).or_default()[if l.get("conexion").map(String::as_str) == Some("tramo_suelto") { 1 } else { 2 }] += 1;
        }
    }
    let mut sin_camino: Vec<Value> = lugares
        .iter()
        .filter(|l| l.get("conexion").map(String::as_str) == Some("sin_camino"))
        .map(|l| {
            let mut v = lugar(l.get("codigo").map(String::as_str).unwrap_or(""));
            let c = catalogo.por_codigo(v["codigo"].as_str().unwrap_or(""));
            v["longitud"] = json!(c.and_then(|c| c.longitud));
            v["latitud"] = json!(c.and_then(|c| c.latitud));
            v["distancia_m"] = json!(num(l, "distancia_m"));
            v
        })
        .collect();
    sin_camino.sort_by(|a, b| b["distancia_m"].as_f64().unwrap_or(f64::MAX).total_cmp(&a["distancia_m"].as_f64().unwrap_or(f64::MAX)));
    let enlaces: Vec<Value> = leer("red_vial_enlaces.csv")?
        .iter()
        .map(|e| {
            let codigos: Vec<&str> = e.get("lugares").map(|l| l.split('|').filter(|c| !c.is_empty()).collect()).unwrap_or_default();
            json!({ "desde": [num(e, "desde_longitud"), num(e, "desde_latitud")], "hasta": [num(e, "hasta_longitud"), num(e, "hasta_latitud")], "km": num(e, "km"), "tramo_km": num(e, "tramo_km"),
                    "lugares": codigos.iter().map(|c| lugar(c)).collect::<Vec<_>>() })
        })
        .collect();
    Ok(json!({
        "disponible": true, "resumen": resumen,
        "departamentos": por_depto.iter().map(|(d, [total, suelto, sin])| json!({ "codigo": d, "nombre": nombre(d), "lugares": total, "tramo_suelto": suelto, "sin_camino": sin })).collect::<Vec<_>>(),
        "enlaces": enlaces, "sin_camino": sin_camino,
    }))
}

/// Prepara las capas geográficas: municipios con sus indicadores, enlaces viales probables y
/// lugares sin conexión. Si falta un insumo, esa capa simplemente no se publica.
fn armar_capas(dir: &std::path::Path, catalogo: &Catalogo, poblacion: &Poblacion, vial: &Value, construido: &BTreeMap<String, Vec<faro_territorio::Punto>>, construccion: &Construccion, cif: &[HashMap<String, String>], talla: &HashMap<String, HashMap<String, String>>) -> (HashMap<String, (&'static str, String)>, Vec<Value>) {
    const GEOJSON: &str = "application/geo+json; charset=utf-8";
    let mut capas = HashMap::new();
    let mut descripcion = Vec::new();
    let mut publicar = |archivo: &str, tipo: &'static str, texto: String, titulo: &str, detalle: &str, fuente: &str, licencia: &str| {
        descripcion.push(json!({ "archivo": archivo, "url": format!("/capas/{archivo}"), "formato": if tipo == GEOJSON { "GeoJSON" } else { "CSV" }, "titulo": titulo, "descripcion": detalle, "fuente": fuente, "licencia": licencia, "bytes": texto.len() }));
        capas.insert(archivo.to_string(), (tipo, texto));
    };

    // Lugares sin conexión por municipio, a partir del análisis vial.
    let mut sin_conexion: HashMap<String, (i64, i64)> = HashMap::new();
    if let Ok(mut lector) = csv::Reader::from_path(dir.join("red_vial_lugares.csv")) {
        for f in lector.deserialize::<HashMap<String, String>>().flatten() {
            let (Some(codigo), Some(conexion)) = (f.get("codigo"), f.get("conexion")) else { continue };
            let e = sin_conexion.entry(codigo.chars().take(4).collect()).or_default();
            if conexion == "tramo_suelto" { e.0 += 1 } else { e.1 += 1 }
        }
    }
    let mut lugares_de: HashMap<String, i64> = HashMap::new();
    for l in catalogo.de_nivel(Nivel::LugarPoblado, None) {
        *lugares_de.entry(l.padre.clone()).or_default() += 1;
    }
    let mut acuerdo: HashMap<String, f64> = HashMap::new();
    if let Ok(mut lector) = csv::Reader::from_path(dir.join("limites_verificacion.csv")) {
        for f in lector.deserialize::<HashMap<String, String>>().flatten() {
            if let (Some(m), Some(p)) = (f.get("municipio"), f.get("pct").and_then(|p| p.parse().ok())) {
                acuerdo.insert(m.clone(), p);
            }
        }
    }

    if let Some(mut g) = std::fs::read_to_string(dir.join("municipios.geojson")).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok()) {
        let (desde, hasta) = poblacion.anios().unwrap_or((2018, 2030));
        let hoy = 2026.clamp(desde, hasta);
        for f in g["features"].as_array_mut().into_iter().flatten() {
            let codigo = f["properties"]["codigo"].as_str().unwrap_or("").to_string();
            let p = &mut f["properties"];
            p["poblacion_2018"] = json!(poblacion.total(&codigo, 2018));
            p["poblacion_actual"] = json!(poblacion.total(&codigo, hoy));
            p["anio_actual"] = json!(hoy);
            p["poblacion_2030"] = json!(poblacion.total(&codigo, hasta));
            p["crecimiento_anual_pct"] = json!(poblacion.cambio(&codigo, 2018, hasta).map(|c| (c.tasa_anual_pct * 100.0).round() / 100.0));
            // Superficie construida vista por satélite (GHSL): 2000 y 2020, ritmo anual y metros por habitante.
            let ha = |anio: i32| construido.get(&codigo).and_then(|s| s.iter().find(|x| x.anio == anio)).map(|x| x.edificaciones);
            p["construido_ha_2000"] = json!(ha(2000).map(|v| v.round()));
            p["construido_ha_2020"] = json!(ha(2020).map(|v| v.round()));
            p["construido_crecimiento_anual_pct"] = json!(ha(2000).zip(ha(2020)).filter(|(a, _)| *a > 0.0).map(|(a, b)| (((b / a).powf(1.0 / 20.0) - 1.0) * 10000.0).round() / 100.0));
            p["construido_m2_por_habitante_2020"] = json!(ha(2020).zip(poblacion.total(&codigo, 2020)).filter(|(_, n)| *n > 0).map(|(h, n)| (h * 10_000.0 / n as f64).round()));
            // Inseguridad alimentaria aguda: el dato es del departamento (CIF), no del municipio.
            let del_depto = cif.iter().find(|f| f.get("departamento").map(String::as_str) == Some(&codigo[..2]));
            p["cif_fase_area_departamento"] = json!(del_depto.and_then(|f| f.get("fase_area")?.parse::<i64>().ok()));
            p["cif_fase3_o_superior_pct_departamento"] = json!(del_depto.and_then(|f| f.get("fase3_o_superior_pct")?.parse::<f64>().ok()));
            p["retardo_en_talla_pct"] = json!(talla.get(&codigo).and_then(|f| f.get("retardo_en_talla_pct")?.parse::<f64>().ok()));
            p["vulnerabilidad_nutricional"] = json!(talla.get(&codigo).and_then(|f| f.get("vulnerabilidad")));
            p["escolares_evaluados"] = json!(talla.get(&codigo).and_then(|f| f.get("escolares_evaluados")?.parse::<f64>().ok()));
            p["proveedores_construccion"] = json!(construccion.proveedores.iter().filter(|x| x.get("municipio_codigo").map(String::as_str) == Some(codigo.as_str())).count());
            p["lugares_poblados"] = json!(lugares_de.get(&codigo).copied().unwrap_or(0));
            let (suelto, sin) = sin_conexion.get(&codigo).copied().unwrap_or((0, 0));
            p["lugares_en_tramo_suelto"] = json!(suelto);
            p["lugares_sin_camino"] = json!(sin);
            p["lugares_dentro_de_su_limite_pct"] = json!(acuerdo.get(&codigo));
        }
        publicar("municipios.geojson", GEOJSON, g.to_string(), "Municipios con indicadores",
            "340 municipios con código INE: población 2018, actual y 2030 (proyecciones INE), crecimiento anual, lugares poblados y cuántos no están conectados a la red vial.",
            "Límites: CONRED / OCHA (COD-AB Guatemala). Población: INE, proyecciones 2015–2030. Lugares: INE, Censo 2018. Caminos: © colaboradores de OpenStreetMap. Superficie construida: Comisión Europea, JRC, GHSL GHS-BUILT-S R2023A.", "CC BY-IGO (límites) · ODC-BY (población) · CC BY (censo) · ODbL (caminos) · CC BY 4.0 (GHSL)");
    }
    if vial["disponible"] == json!(true) {
        let enlaces: Vec<Value> = vial["enlaces"].as_array().into_iter().flatten().map(|e| json!({ "type": "Feature",
            "properties": { "km": e["km"], "tramo_km": e["tramo_km"], "lugares": e["lugares"].as_array().map_or(0, Vec::len), "municipio": e["lugares"][0]["municipio"], "departamento": e["lugares"][0]["departamento"],
                            "nombres": e["lugares"].as_array().into_iter().flatten().take(8).filter_map(|l| l["nombre"].as_str()).collect::<Vec<_>>().join(", ") },
            "geometry": { "type": "LineString", "coordinates": [e["desde"], e["hasta"]] } })).collect();
        publicar("red_vial_enlaces.geojson", GEOJSON, json!({ "type": "FeatureCollection", "features": enlaces }).to_string(), "Enlaces viales que probablemente faltan en el mapa",
            "Para cada tramo de camino desconectado con lugares poblados, el hueco más corto hasta la red principal.", "FARO, a partir de © colaboradores de OpenStreetMap (HOT/HDX) e INE, Censo 2018.", "ODbL 1.0");
        let mut puntos: Vec<Value> = vial["sin_camino"].as_array().into_iter().flatten().filter(|l| l["longitud"].is_number()).map(|l| json!({ "type": "Feature",
            "properties": { "codigo": l["codigo"], "nombre": l["nombre"], "municipio": l["municipio"], "departamento": l["departamento"], "conexion": "sin_camino", "distancia_m": l["distancia_m"] },
            "geometry": { "type": "Point", "coordinates": [l["longitud"], l["latitud"]] } })).collect();
        // Los lugares en tramo suelto salen de la tabla de enlaces y del catálogo.
        for e in vial["enlaces"].as_array().into_iter().flatten() {
            for l in e["lugares"].as_array().into_iter().flatten() {
                if let Some(c) = l["codigo"].as_str().and_then(|c| catalogo.por_codigo(c)).filter(|c| c.longitud.is_some()) {
                    puntos.push(json!({ "type": "Feature", "properties": { "codigo": c.codigo, "nombre": c.nombre, "municipio": l["municipio"], "departamento": l["departamento"], "conexion": "tramo_suelto", "distancia_m": Value::Null },
                                        "geometry": { "type": "Point", "coordinates": [c.longitud, c.latitud] } }));
                }
            }
        }
        publicar("lugares_sin_conexion.geojson", GEOJSON, json!({ "type": "FeatureCollection", "features": puntos }).to_string(), "Lugares poblados sin conexión a la red vial",
            "Lugares del Censo 2018 sin camino para vehículos a 500 m, o en un tramo que no se une a la red principal.", "FARO, a partir de INE, Censo 2018 y © colaboradores de OpenStreetMap (HOT/HDX).", "ODbL 1.0");
    }
    for (archivo, titulo, detalle) in [("proveedores_construccion.csv", "Proveedores de materiales de construcción", "Perfil público de 116 negocios: tipo, ubicación, canales y si publican precios. Sin datos de personas."),
                                       ("precios_construccion.csv", "Precios publicados de materiales de construcción", "Precios tal como los publican las tiendas en línea, con la dirección de la página y la fecha de consulta.")] {
        if let Ok(texto) = std::fs::read_to_string(dir.join(archivo)) {
            publicar(archivo, "text/csv; charset=utf-8", texto, titulo, detalle, "Levantamiento de FARO en páginas públicas de los negocios, 24-nov-2026.", "Datos públicos de cada negocio; cite la página de origen");
        }
    }
    for (archivo, tipo, titulo, detalle, fuente, licencia) in [
        ("censo_talla_municipio.csv", "text/csv; charset=utf-8", "Retardo en talla en escolares por municipio, Censo de Talla 2024", "Porcentaje de escolares con retardo en talla (desnutrición crónica), escolares evaluados y categoría de vulnerabilidad nutricional, con código INE.", "SESAN, SIINSAN: V Censo Nacional de Talla en Escolares, 2024.", "publicación oficial; cite la fuente"),
        ("cif_departamentos.csv", "text/csv; charset=utf-8", "Inseguridad alimentaria aguda por departamento (CIF), octubre 2026 – febrero 2027", "Personas en cada fase de la CIF, fase del área y Fase 3 o superior, con código INE del departamento.", "SESAN y Grupo Técnico de la CIF Guatemala, actualización publicada el 19-nov-2026 y corregida el 20-nov-2026.", "publicación oficial; cite la fuente"),
        ("departamentos.geojson", GEOJSON, "Departamentos", "22 departamentos con código INE, para mapas a menor detalle.", "CONRED / OCHA (COD-AB Guatemala).", "CC BY-IGO"),
        ("red_vial_contexto.geojson", GEOJSON, "Caminos mapeados alrededor de los enlaces probables", "Tramos de camino a 1.5 km de cada enlace propuesto; indica si pertenecen a la red principal o a un tramo suelto.", "© colaboradores de OpenStreetMap (HOT/HDX), corte del 3-nov-2026.", "ODbL 1.0"),
        ("superficie_construida_bloques.csv", "text/csv; charset=utf-8", "Superficie construida en bloques de 1 km, 1975–2020", "Hectáreas construidas por bloque de 1 km y época, con el municipio al que pertenece: la huella edificada del país.", "FARO, a partir de Comisión Europea, JRC: GHSL GHS-BUILT-S R2023A.", "CC BY 4.0"),
    ] {
        if let Ok(texto) = std::fs::read_to_string(dir.join(archivo)) {
            publicar(archivo, tipo, texto, titulo, detalle, fuente, licencia);
        }
    }
    let puntos: Vec<Value> = catalogo.de_nivel(Nivel::LugarPoblado, None).filter(|l| l.longitud.is_some()).map(|l| json!({ "type": "Feature", "properties": { "codigo": l.codigo, "nombre": l.nombre, "municipio": l.padre }, "geometry": { "type": "Point", "coordinates": [l.longitud, l.latitud] } })).collect();
    if !puntos.is_empty() {
        publicar("lugares_poblados.geojson", GEOJSON, json!({ "type": "FeatureCollection", "features": puntos }).to_string(), "Lugares poblados del Censo 2018", "19,723 lugares poblados con código INE y centroide.", "INE, XII Censo Nacional de Población y VII de Vivienda 2018.", "CC BY");
    }
    if let Ok(texto) = std::fs::read_to_string(dir.join("superficie_construida_municipio.csv")) {
        publicar("superficie_construida_municipio.csv", "text/csv; charset=utf-8", texto, "Superficie construida por municipio, 1975–2020", "Hectáreas construidas vistas por satélite, cada cinco años, por municipio con código INE.", "FARO, a partir de Comisión Europea, JRC: GHSL GHS-BUILT-S R2023A (100 m).", "CC BY 4.0");
    }
    if let Ok(texto) = std::fs::read_to_string(dir.join("poblacion_municipio.csv")) {
        publicar("poblacion_municipio.csv", "text/csv; charset=utf-8", texto, "Población por municipio y año, 2015–2030", "Total, hombres y mujeres por municipio y año, con el código INE ya corregido.", "INE, Estimaciones y proyecciones de población 2015–2030.", "ODC-BY");
    }
    (capas, descripcion)
}

/// Catálogo de capas publicadas: lo que un portal o un SIG necesita para consumirlas.
async fn api_capas(State(e): State<Compartido>) -> Response {
    ([("access-control-allow-origin", "*")], Json(json!({ "titulo": "FARO · capas publicadas", "nota": "Datos abiertos derivados de fuentes oficiales y abiertas. Cite la fuente de cada capa.", "capas": e.catalogo_capas }))).into_response()
}

async fn capa(State(e): State<Compartido>, Ruta(archivo): Ruta<String>) -> Response {
    match e.capas.get(&archivo) {
        // Abiertas a otros sitios (CORS) para que un portal de mapas las lea directamente.
        Some((tipo, texto)) => ([("content-type", *tipo), ("access-control-allow-origin", "*"), ("cache-control", "public, max-age=300")], texto.clone()).into_response(),
        None => Fallo(StatusCode::NOT_FOUND, "no existe esa capa".into()).into_response(),
    }
}

/// Registros recibidos por municipio y solicitud (solo conteos), con tasa por 100 mil habitantes.
async fn api_registros_municipio(State(e): State<Compartido>) -> Res<Json<Value>> {
    let filas = e
        .bd
        .query(
            "WITH ultimo AS (SELECT solicitud, periodo, max(envio_id) AS envio_id FROM hechos GROUP BY 1, 2)
             SELECT h.solicitud, h.municipio, sum(h.n)::bigint, max(left(h.periodo, 4)) FROM hechos h JOIN ultimo u USING (solicitud, periodo, envio_id) WHERE h.municipio <> '' GROUP BY 1, 2",
            &[],
        )
        .await
        .map_err(interno)?;
    let mut salida: BTreeMap<String, BTreeMap<String, Value>> = BTreeMap::new();
    for f in &filas {
        let (municipio, total): (String, i64) = (f.get(1), f.get(2));
        let poblacion = f.get::<_, Option<String>>(3).and_then(|a| a.parse::<i32>().ok()).and_then(|a| e.poblacion.total(&municipio, a));
        salida.entry(f.get(0)).or_default().insert(municipio, json!({ "total": total, "por_100_mil": poblacion.filter(|p| *p > 0).map(|p| (total as f64 * 1_000_000.0 / p as f64).round() / 10.0) }));
    }
    Ok(Json(json!(salida)))
}

/// Crecimiento de la superficie construida por municipio (GHSL) y su proyección. La tendencia se
/// calcula con las épocas desde `desde` (por defecto 2000), que provienen de imágenes más precisas.
async fn api_construido(State(e): State<Compartido>, Query(q): Query<HashMap<String, String>>) -> Res<Json<Value>> {
    let anio: i32 = q.get("anio").and_then(|a| a.parse().ok()).unwrap_or(2030);
    let desde: i32 = q.get("desde").and_then(|a| a.parse().ok()).unwrap_or(2000);
    if !(2021..=2050).contains(&anio) {
        return Err(pedido("el año de proyección debe estar entre 2021 y 2050"));
    }
    let mut salida: Vec<Value> = e
        .construido
        .iter()
        .filter_map(|(codigo, serie)| {
            let reciente: Vec<faro_territorio::Punto> = serie.iter().copied().filter(|p| p.anio >= desde).collect();
            let c = faro_territorio::analizar(codigo, &reciente, anio)?;
            let l = e.catalogo.por_codigo(codigo);
            let mut v = serde_json::to_value(&c).ok()?;
            v["municipio"] = json!(l.map(|l| l.nombre.clone()));
            v["departamento"] = json!(l.and_then(|l| e.catalogo.por_codigo(&l.padre)).map(|d| d.nombre.clone()));
            v["serie"] = json!(serie.iter().map(|p| json!([p.anio, p.edificaciones.round()])).collect::<Vec<_>>());
            let poblacion = e.poblacion.cambio(codigo, 2018, 2030);
            v["poblacion_tasa_anual_pct"] = json!(poblacion.map(|p| p.tasa_anual_pct));
            Some(v)
        })
        .collect();
    salida.sort_by(|a, b| b["cambio_absoluto"].as_f64().unwrap_or(0.0).total_cmp(&a["cambio_absoluto"].as_f64().unwrap_or(0.0)));
    let pais = |a: i32| e.construido.values().filter_map(|s| s.iter().find(|p| p.anio == a)).map(|p| p.edificaciones).sum::<f64>();
    let epocas: std::collections::BTreeSet<i32> = e.construido.values().flatten().map(|p| p.anio).collect();
    Ok(Json(json!({ "disponible": !e.construido.is_empty(), "desde": desde, "pais": epocas.iter().map(|a| json!([a, pais(*a).round()])).collect::<Vec<_>>(), "municipios": salida })))
}

/// Lo necesario para comparar precios de materiales de construcción contra el IPMC.
struct Construccion {
    reglas: faro_homologa::productos::Reglas,
    /// Artículos oficiales: código -> fila del catálogo (artículo, unidad, índice, variación).
    articulos: BTreeMap<u32, HashMap<String, String>>,
    precios: Vec<HashMap<String, String>>,
    /// Perfiles; se les agrega `municipio_codigo` cuando el municipio se reconoce sin duda.
    proveedores: Vec<HashMap<String, String>>,
}

impl Construccion {
    fn cargar(dir: &std::path::Path, catalogo: &Catalogo) -> Self {
        let leer = |ruta: PathBuf| -> Vec<HashMap<String, String>> { csv::Reader::from_path(&ruta).map(|mut l| l.deserialize().flatten().collect()).unwrap_or_default() };
        let reglas = faro_homologa::productos::Reglas::cargar(&PathBuf::from(var("FARO_REGLAS_IPMC", "datos/estandar/ipmc_reglas.toml"))).unwrap_or_else(|e| {
            eprintln!("AVISO: sin reglas del IPMC ({e}).");
            faro_homologa::productos::Reglas { articulos: vec![] }
        });
        let articulos = leer(PathBuf::from(var("FARO_IPMC", "datos/catalogo_precios/ipmc_articulos.csv"))).into_iter().filter_map(|f| Some((f.get("codigo")?.parse().ok()?, f))).collect();
        // El municipio de cada proveedor se homologa con el mismo motor que los envíos.
        let memoria = Memoria::default();
        let h = Homologador::new(catalogo, &memoria);
        let mut proveedores = leer(dir.join("proveedores_construccion.csv"));
        for p in &mut proveedores {
            let depto = h.homologar(p.get("departamento").map(String::as_str).unwrap_or(""), Nivel::Departamento, None);
            let muni = h.homologar(p.get("municipio").map(String::as_str).unwrap_or(""), Nivel::Municipio, depto.codigo.as_deref());
            p.insert("departamento_codigo".into(), depto.codigo.unwrap_or_default());
            p.insert("municipio_codigo".into(), muni.codigo.unwrap_or_default());
        }
        Construccion { reglas, articulos, precios: leer(dir.join("precios_construccion.csv")), proveedores }
    }
}

/// Precios de materiales de construcción: cada producto publicado se homologa contra el IPMC y,
/// por artículo, se compara un precio por proveedor.
async fn api_construccion(State(e): State<Compartido>) -> Res<Json<Value>> {
    use faro_homologa::productos;
    use faro_territorio::construccion::{resumir, Observacion};
    let mem = memoria(&e).await?;
    let c = &e.construccion;
    // Precios del levantamiento inicial más los del muestreo automático (el más reciente de cada
    // producto y proveedor sustituye al anterior).
    let observados = e.bd.query("SELECT DISTINCT ON (proveedor, producto) proveedor, producto, precio, unidad, url, to_char(observado, 'YYYY-MM-DD') FROM precios_observados WHERE rubro = 'construccion' ORDER BY proveedor, producto, observado DESC", &[]).await.map_err(interno)?;
    let nuevos: Vec<HashMap<String, String>> = observados.iter().map(|f| HashMap::from([("proveedor".to_string(), f.get::<_, String>(0)), ("producto".into(), f.get(1)), ("precio".into(), f.get::<_, f64>(2).to_string()), ("unidad_publicada".into(), f.get(3)), ("url".into(), f.get(4)), ("fecha".into(), f.get(5))])).collect();
    let ya: std::collections::HashSet<(String, String)> = nuevos.iter().map(|f| (f["proveedor"].clone(), f["producto"].clone())).collect();
    let observaciones: Vec<Observacion> = c
        .precios
        .iter()
        .filter(|f| !ya.contains(&(f.get("proveedor").cloned().unwrap_or_default(), f.get("producto").cloned().unwrap_or_default())))
        .chain(nuevos.iter())
        .filter_map(|f| {
            let campo = |k: &str| f.get(k).cloned().unwrap_or_default();
            let precio: f64 = campo("precio").parse().ok()?;
            let r = productos::homologar(&c.reglas, &mem, &campo("producto"), &campo("unidad_publicada"), precio);
            Some(Observacion { proveedor: campo("proveedor"), producto: campo("producto"), url: campo("url"), fecha: campo("fecha"), precio, unidad: campo("unidad_publicada"), codigo: r.codigo, estado: r.estado.texto().to_string(), precio_oficial: r.precio_oficial, motivo: r.motivo })
        })
        .collect();
    let resumenes: HashMap<u32, Value> = resumir(&observaciones, &faro_territorio::precios::Parametros::default()).into_iter().map(|r| (r.codigo, json!(r))).collect();
    let con_regla: std::collections::BTreeSet<u32> = c.reglas.articulos.iter().map(|r| r.codigo).collect();
    let articulos: Vec<Value> = c.articulos.iter().map(|(codigo, a)| {
        let campo = |k: &str| a.get(k).cloned().unwrap_or_default();
        json!({ "codigo": codigo, "grupo": campo("grupo"), "articulo": campo("articulo"), "unidad": campo("unidad"), "periodo": campo("periodo"), "indice": campo("indice").parse::<f64>().ok(),
                "variacion_interanual_pct": campo("variacion_interanual_pct").parse::<f64>().ok(), "tiene_regla": con_regla.contains(codigo), "comparacion": resumenes.get(codigo) })
    }).collect();
    let cuenta = |estado: &str| observaciones.iter().filter(|o| o.estado == estado).count();
    let mut por_proveedor: HashMap<&str, usize> = HashMap::new();
    for o in &observaciones {
        *por_proveedor.entry(&o.proveedor).or_default() += 1;
    }
    let proveedores: Vec<Value> = c.proveedores.iter().map(|p| {
        let mut v = json!(p);
        v["precios"] = json!(por_proveedor.get(p.get("nombre").map(String::as_str).unwrap_or("")).copied().unwrap_or(0));
        v
    }).collect();
    let nombre_articulo = |codigo: Option<u32>| codigo.and_then(|k| c.articulos.get(&k)).and_then(|a| a.get("articulo")).cloned();
    let por_revisar: Vec<Value> = observaciones.iter().filter(|o| o.estado == "revisar").take(400).map(|o| { let mut v = json!(o); v["articulo"] = json!(nombre_articulo(o.codigo)); v }).collect();
    Ok(Json(json!({
        "resumen": { "proveedores": c.proveedores.len(), "proveedores_con_precios": por_proveedor.len(), "precios": observaciones.len(), "del_muestreo": nuevos.len(), "comparables": cuenta("comparable") + cuenta("memoria"), "aprendidos": cuenta("memoria"),
                     "por_revisar": cuenta("revisar"), "distintos": cuenta("distinto"), "descartados": cuenta("descartado"), "articulos_con_regla": con_regla.len(), "articulos": c.articulos.len(),
                     "articulos_con_precio": resumenes.values().filter(|r| r["por_proveedor"].as_array().is_some_and(|a| !a.is_empty())).count() },
        "articulos": articulos, "por_revisar": por_revisar, "proveedores": proveedores,
    })))
}

/// Busca en los precios de referencia publicados por el INE. Todas las palabras de la consulta
/// deben aparecer en el producto, sus características o su categoría.
async fn api_referencias(State(e): State<Compartido>, Query(q): Query<HashMap<String, String>>) -> Res<Json<Value>> {
    let consulta = faro_homologa::normalizar::basico(q.get("q").map(String::as_str).unwrap_or(""));
    let palabras: Vec<&str> = consulta.split(' ').filter(|p| p.len() >= 2).collect();
    let categorias: std::collections::BTreeSet<&str> = e.referencias.iter().filter_map(|f| f.get("categoria").map(String::as_str)).collect();
    if palabras.is_empty() {
        return Ok(Json(json!({ "total": e.referencias.len(), "categorias": categorias.len(), "coincidencias": 0, "filas": [] })));
    }
    let hallados: Vec<&HashMap<String, String>> = e
        .referencias
        .iter()
        .filter(|f| {
            let texto = faro_homologa::normalizar::basico(&format!("{} {} {}", f.get("producto").map_or("", String::as_str), f.get("caracteristicas").map_or("", String::as_str), f.get("categoria").map_or("", String::as_str)));
            palabras.iter().all(|p| texto.split(' ').any(|t| t.starts_with(p)))
        })
        .collect();
    let mut precios: Vec<f64> = hallados.iter().filter_map(|f| f.get("precio")?.parse().ok()).collect();
    precios.sort_by(f64::total_cmp);
    let mut filas: Vec<&&HashMap<String, String>> = hallados.iter().collect();
    filas.sort_by(|a, b| b.get("fecha").cmp(&a.get("fecha")));
    Ok(Json(json!({ "total": e.referencias.len(), "categorias": categorias.len(), "coincidencias": hallados.len(),
        "minimo": precios.first(), "mediana": precios.get(precios.len() / 2), "maximo": precios.last(), "filas": filas.into_iter().take(100).collect::<Vec<_>>() })))
}

/// Cifras de cada etapa del proceso, para el diagrama de flujo.
async fn api_proceso(State(e): State<Compartido>) -> Res<Json<Value>> {
    let f = e
        .bd
        .query_one(
            "SELECT (SELECT count(*) FROM envios), (SELECT coalesce(sum(filas),0)::bigint FROM envios), (SELECT coalesce(sum(problemas),0)::bigint FROM envios),
                    (SELECT count(*) FROM (SELECT 1 FROM pendientes WHERE NOT resuelto GROUP BY columna, nivel, contexto, valor) p),
                    (SELECT count(*) FROM equivalencias) + (SELECT count(*) FROM equivalencias_valor), (SELECT coalesce(sum(n),0)::bigint FROM hechos),
                    (SELECT count(*) FROM conjuntos_ine), (SELECT count(DISTINCT institucion) FROM envios)",
            &[],
        )
        .await
        .map_err(interno)?;
    Ok(Json(json!({
        "solicitudes": solicitudes(&e).map_err(interno)?.len(), "envios": f.get::<_, i64>(0), "filas": f.get::<_, i64>(1), "problemas": f.get::<_, i64>(2), "pendientes": f.get::<_, i64>(3),
        "equivalencias": f.get::<_, i64>(4), "registros_agregados": f.get::<_, i64>(5), "conjuntos_ine": f.get::<_, i64>(6), "instituciones": f.get::<_, i64>(7),
        "lugares_en_catalogo": e.catalogo.len(), "capas": e.catalogo_capas.len(), "fuentes": e.fuentes["fuentes"].as_array().map_or(0, Vec::len),
        "municipios_con_poblacion": e.poblacion.municipios().len(), "articulos_ipmc": e.construccion.articulos.len(), "precios_referencia_ine": e.referencias.len(),
    })))
}

/// Repite, desde el portal del INE, el proceso que produjo la tabla de población: consulta el
/// conjunto, descarga el libro y su diccionario, los procesa y compara el resultado con la tabla
/// que usa este servidor. Devuelve cada paso con sus cifras.
async fn api_reproducir_poblacion(State(e): State<Compartido>, cabeceras: HeaderMap) -> Res<Json<Value>> {
    use faro_territorio::poblacion::{procesar, Paso};
    autorizar(&e, &cabeceras).await?;
    let resultado = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        const CONJUNTO: &str = "proyecciones-de-poblacion-2015-2030";
        let mut pasos: Vec<Paso> = Vec::new();
        let conjunto = faro_fuentes::Ckan::new(&e.portal).conjunto(CONJUNTO)?;
        let libros: Vec<&faro_fuentes::ckan::Recurso> = conjunto.resources.iter().filter(|r| r.format.eq_ignore_ascii_case("xlsx")).collect();
        let es_diccionario = |r: &&&faro_fuentes::ckan::Recurso| r.name.to_lowercase().contains("diccionario");
        let (libro, diccionario) = (libros.iter().find(|r| !es_diccionario(r)).ok_or("el conjunto no trae el libro de proyecciones")?, libros.iter().find(es_diccionario).ok_or("el conjunto no trae el diccionario")?);
        pasos.push(Paso { titulo: "Consultar el portal del INE".into(), detalle: format!("Conjunto «{}» en {}; licencia {}; última modificación {}. Recursos: «{}» y «{}».", conjunto.title, e.portal, conjunto.licencia(), conjunto.metadata_modified.get(..10).unwrap_or(""), libro.name, diccionario.name) });
        // Las descargas quedan en una carpeta temporal con su propio registro de procedencia.
        let dir = std::env::temp_dir().join("faro-reproducir");
        let manifiesto = faro_fuentes::Manifiesto::new(&dir.join("manifiesto.jsonl"));
        let mut rutas = Vec::new();
        for r in [libro, diccionario] {
            let destino = dir.join(faro_fuentes::procedencia::nombre_de_url(&r.url));
            let d = manifiesto.descargar(CONJUNTO, &r.url, &destino, "Instituto Nacional de Estadística (INE)", conjunto.licencia())?;
            pasos.push(Paso { titulo: "Descargar".into(), detalle: format!("{} · {} bytes · {} · SHA-256 {}", r.url, d.bytes, d.fecha, d.sha256) });
            rutas.push(destino);
        }
        let memoria = Memoria::default();
        let h = Homologador::new(&e.catalogo, &memoria);
        let es_municipio = |m: &str| e.catalogo.por_codigo(m).is_some_and(|l| l.nivel == Nivel::Municipio);
        let ubicar = |codigo: &str, nombre: &str| {
            let r = h.homologar(nombre, Nivel::Municipio, Some(&codigo[..2.min(codigo.len())]));
            r.codigo.filter(|_| r.estado.resuelto()).ok_or(format!("{codigo} ({nombre}): no coincide con un único municipio de su departamento"))
        };
        let (filas, proceso) = procesar(&lectura::leer_crudo(&rutas[0])?, Some(&lectura::leer_crudo(&rutas[1])?), &es_municipio, &ubicar)?;
        pasos.extend(proceso);
        // ¿Sale lo mismo que la tabla con la que trabaja este servidor?
        let distintas = filas.iter().filter(|f| e.poblacion.fila(&f.municipio, f.anio) != Some(f)).count();
        let identica = distintas == 0 && filas.len() == e.poblacion.len();
        pasos.push(Paso { titulo: "Comparar con la tabla en uso".into(), detalle: if identica { format!("Las {} filas recién calculadas son idénticas a las que usa este servidor.", filas.len()) } else { format!("{distintas} filas difieren de la tabla en uso (que tiene {} filas): el INE pudo haber actualizado el libro.", e.poblacion.len()) } });
        Ok(json!({ "pasos": pasos, "identica": identica, "filas": filas.len() }))
    })
    .await
    .map_err(interno)?;
    Ok(Json(resultado.map_err(|x| Fallo(StatusCode::BAD_GATEWAY, format!("no se pudo repetir el proceso: {x}")).1).map_err(pedido)?))
}

// ---------------------------------------------------------------------------------------------
// Muestreo automático de precios
// ---------------------------------------------------------------------------------------------

/// Nombre con que se guardan los precios del levantamiento que viene con el sistema.
const LEVANTAMIENTO: &str = "levantamiento-inicial";

/// Carga una sola vez el levantamiento de precios publicados que viene con el sistema
/// (`datos/referencia/precios_publicados.csv`), con la fecha en que se consultó cada precio.
async fn sembrar_precios(bd: &Client, ruta: &std::path::Path) -> Result<usize, String> {
    if !ruta.exists() {
        return Ok(0);
    }
    let filas: Vec<HashMap<String, String>> = csv::Reader::from_path(ruta).map_err(|e| e.to_string())?.deserialize().collect::<Result<_, _>>().map_err(|e| e.to_string())?;
    let filas: Vec<&HashMap<String, String>> = filas.iter().filter(|f| f.get("precio_q").is_some_and(|p| p.parse::<f64>().is_ok_and(|x| x > 0.0)) && f.get("fecha_consulta").is_some_and(|d| d.len() == 10)).collect();
    // Si la tabla del sistema cambió (una versión nueva trae un levantamiento más amplio), se reemplaza.
    let cargadas: i64 = bd.query_one("SELECT count(*) FROM precios_observados WHERE tarea = $1", &[&LEVANTAMIENTO]).await.map_err(|e| e.to_string())?.get(0);
    if cargadas == filas.len() as i64 {
        return Ok(0);
    }
    bd.execute("DELETE FROM precios_observados WHERE tarea = $1", &[&LEVANTAMIENTO]).await.map_err(|e| e.to_string())?;
    for lote in filas.chunks(500) {
        let col = |k: &str| lote.iter().map(|f| f.get(k).map(String::as_str).unwrap_or("")).collect::<Vec<&str>>();
        let precios: Vec<f64> = lote.iter().map(|f| f["precio_q"].parse().unwrap_or(0.0)).collect();
        let listas: Vec<Option<f64>> = lote.iter().map(|f| f.get("precio_lista_q").and_then(|p| p.parse().ok())).collect();
        bd.execute(
            "INSERT INTO precios_observados (tarea, rubro, proveedor, producto, marca, precio, url, termino, ean, precio_lista, observado)
             SELECT $1, r, t, p, m, x, u, c, e, l, d::date FROM unnest($2::text[], $3::text[], $4::text[], $5::text[], $6::float8[], $7::text[], $8::text[], $9::text[], $10::float8[], $11::text[]) AS f(r, t, p, m, x, u, c, e, l, d)",
            &[&LEVANTAMIENTO, &col("rubro"), &col("tienda"), &col("producto"), &col("marca"), &precios, &col("url"), &col("categoria_tienda"), &col("codigo_barras"), &listas, &col("fecha_consulta")],
        )
        .await
        .map_err(|e| e.to_string())?;
    }
    Ok(filas.len())
}

/// Tareas que se están ejecutando en este momento (para no lanzar dos veces la misma).
static EN_CURSO: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

/// Ejecuta una tarea de muestreo, guarda lo observado y deja constancia del resultado.
async fn ejecutar_muestreo(e: &Compartido, tarea: &faro_fuentes::muestreo::Tarea) -> Res<Value> {
    {
        let mut en_curso = EN_CURSO.lock().map_err(interno)?;
        if en_curso.contains(&tarea.id) {
            return Err(pedido("esta tarea ya se está ejecutando"));
        }
        en_curso.push(tarea.id.clone());
    }
    let resultado = muestrear_y_guardar(e, tarea).await;
    if let Ok(mut en_curso) = EN_CURSO.lock() {
        en_curso.retain(|id| id != &tarea.id);
    }
    resultado
}

async fn muestrear_y_guardar(e: &Compartido, tarea: &faro_fuentes::muestreo::Tarea) -> Res<Value> {
    let t = tarea.clone();
    let (ofertas, bitacora) = tokio::task::spawn_blocking(move || faro_fuentes::muestreo::ejecutar(&t)).await.map_err(interno)?;
    // Se guarda por lotes: un catálogo completo trae miles de precios.
    for lote in ofertas.chunks(500) {
        let col = |f: fn(&faro_fuentes::muestreo::Oferta) -> &str| lote.iter().map(f).collect::<Vec<&str>>();
        let precios: Vec<f64> = lote.iter().map(|o| o.precio).collect();
        let listas: Vec<Option<f64>> = lote.iter().map(|o| o.precio_lista).collect();
        e.bd.execute(
            "INSERT INTO precios_observados (tarea, rubro, proveedor, producto, marca, precio, unidad, url, termino, ean, precio_lista)
             SELECT $1, $2, $3, * FROM unnest($4::text[], $5::text[], $6::float8[], $7::text[], $8::text[], $9::text[], $10::text[], $11::float8[])",
            &[&tarea.id, &tarea.rubro, &tarea.proveedor, &col(|o| &o.producto), &col(|o| &o.marca), &precios, &col(|o| &o.unidad), &col(|o| &o.url), &col(|o| &o.termino), &col(|o| &o.ean), &listas],
        )
        .await
        .map_err(interno)?;
    }
    let detalle = bitacora.join("; ");
    e.bd.execute(
        "INSERT INTO muestreos (id, activa, cada_horas, ultima, ultimo_ok, detalle) VALUES ($1,$2,$3, now(), $4, $5)
         ON CONFLICT (id) DO UPDATE SET ultima = now(), ultimo_ok = $4, detalle = $5",
        &[&tarea.id, &tarea.activa, &tarea.cada_horas, &!ofertas.is_empty(), &detalle],
    )
    .await
    .map_err(interno)?;
    Ok(json!({ "precios": ofertas.len(), "detalle": detalle }))
}

async fn api_muestreos(State(e): State<Compartido>) -> Res<Json<Value>> {
    let estados = e.bd.query("SELECT id, activa, cada_horas, to_char(ultima, 'YYYY-MM-DD HH24:MI'), ultimo_ok, detalle FROM muestreos", &[]).await.map_err(interno)?;
    let conteos = e.bd.query("SELECT tarea, count(*), count(DISTINCT producto), count(DISTINCT observado::date) FROM precios_observados GROUP BY 1", &[]).await.map_err(interno)?;
    Ok(Json(json!(e.tareas.iter().map(|t| {
        let est = estados.iter().find(|f| f.get::<_, String>(0) == t.id);
        let n = conteos.iter().find(|f| f.get::<_, String>(0) == t.id);
        let en_curso = EN_CURSO.lock().map(|c| c.contains(&t.id)).unwrap_or(false);
        json!({ "id": t.id, "titulo": t.titulo, "rubro": t.rubro, "proveedor": t.proveedor, "base": t.base, "terminos": t.terminos, "tipo": t.tipo, "categorias": t.categorias, "max_resultados": t.max_resultados, "en_curso": en_curso,
                "activa": est.map_or(t.activa, |f| f.get(1)), "cada_horas": est.map_or(t.cada_horas, |f| f.get(2)),
                "ultima": est.and_then(|f| f.get::<_, Option<String>>(3)), "ultimo_ok": est.and_then(|f| f.get::<_, Option<bool>>(4)), "detalle": est.map(|f| f.get::<_, String>(5)),
                "precios": n.map_or(0, |f| f.get::<_, i64>(1)), "productos": n.map_or(0, |f| f.get::<_, i64>(2)), "corridas": n.map_or(0, |f| f.get::<_, i64>(3)) })
    }).collect::<Vec<_>>())))
}

/// Qué más podría ser un código de lugar que no existe, según el diccionario de estadísticas
/// vitales del INE. Es una pista para quien revisa: el sistema no decide con ella.
fn pista_de_codigo(e: &Estado, nivel: &str, valor: &str) -> Option<String> {
    let n: u32 = valor.trim().parse().ok()?;
    let mut pistas: Vec<String> = Vec::new();
    match (nivel, n) {
        ("departamento", 23) | ("municipio", 2300) => pistas.push("en el diccionario del INE este código es «Extranjero»".into()),
        ("departamento", 99) | ("municipio", 9999) => pistas.push("en el diccionario del INE este código es «Ignorado»".into()),
        (_, 0) => pistas.push("el cero no está en el diccionario del INE como lugar: suele ser un campo sin llenar".into()),
        _ => {}
    }
    let fuera = match nivel { "departamento" => n > 22 && n != 23 && n != 99, "municipio" => n != 2300 && n != 9999, _ => false };
    if fuera {
        if let Some(pais) = e.paises.get(&n.to_string()).filter(|p| !p.eq_ignore_ascii_case("ignorado")) {
            pistas.push(format!("{n} es «{pais}» en el catálogo de países del INE"));
        }
        if (10..=91).contains(&n) {
            pistas.push(format!("{n} también es una edad posible"));
        }
        if !pistas.is_empty() {
            pistas.push("revise si las columnas del archivo están corridas o mal rotuladas".into());
        }
    }
    (!pistas.is_empty()).then(|| pistas.join("; "))
}

#[derive(Deserialize)]
struct ConfigMuestreo {
    activa: Option<bool>,
    cada_horas: Option<i32>,
}

async fn api_muestreo_configurar(State(e): State<Compartido>, Ruta(id): Ruta<String>, cabeceras: HeaderMap, Json(c): Json<ConfigMuestreo>) -> Res<Json<Value>> {
    autorizar(&e, &cabeceras).await?;
    let t = e.tareas.iter().find(|t| t.id == id).ok_or_else(|| pedido(format!("no existe la tarea {id}")))?;
    if c.cada_horas.is_some_and(|h| !(1..=24 * 90).contains(&h)) {
        return Err(pedido("la frecuencia debe estar entre 1 hora y 90 días"));
    }
    e.bd.execute(
        "INSERT INTO muestreos (id, activa, cada_horas) VALUES ($1, coalesce($2::bool, $4::bool), coalesce($3::int4, $5::int4))
         ON CONFLICT (id) DO UPDATE SET activa = coalesce($2::bool, muestreos.activa), cada_horas = coalesce($3::int4, muestreos.cada_horas)",
        &[&id, &c.activa, &c.cada_horas, &t.activa, &t.cada_horas],
    )
    .await
    .map_err(interno)?;
    Ok(Json(json!({ "guardado": true })))
}

async fn api_muestreo_ejecutar(State(e): State<Compartido>, Ruta(id): Ruta<String>, cabeceras: HeaderMap) -> Res<Json<Value>> {
    autorizar(&e, &cabeceras).await?;
    let t = e.tareas.iter().find(|t| t.id == id).ok_or_else(|| pedido(format!("no existe la tarea {id}")))?.clone();
    if t.tipo == "vtex" {
        return Ok(Json(ejecutar_muestreo(&e, &t).await?));
    }
    // Un catálogo completo tarda minutos: corre aparte y la pantalla consulta el avance.
    if EN_CURSO.lock().map_err(interno)?.contains(&t.id) {
        return Err(pedido("esta tarea ya se está ejecutando"));
    }
    tokio::spawn(async move {
        let r = ejecutar_muestreo(&e, &t).await;
        eprintln!("muestreo {}: {}", t.id, r.map(|v| v.to_string()).unwrap_or_else(|f| f.1));
    });
    Ok(Json(json!({ "en_curso": true })))
}

/// Precios publicados por rubro: el más reciente de cada producto de cada tienda, con búsqueda;
/// el mismo código de barras en tiendas distintas; y los cambios de precio entre dos muestreos.
async fn api_publicados(State(e): State<Compartido>, Query(q): Query<HashMap<String, String>>) -> Res<Json<Value>> {
    const ULTIMOS: &str = "SELECT DISTINCT ON (rubro, proveedor, url) rubro, proveedor, producto, marca, precio, precio_lista, ean, termino, url, observado FROM precios_observados ORDER BY rubro, proveedor, url, observado DESC";
    const PLANO: &str = "translate(lower(producto), 'áéíóúüñ', 'aeiouun')";
    let rubros = e.bd.query(&format!("SELECT rubro, count(*), count(DISTINCT proveedor), count(*) FILTER (WHERE ean <> ''), to_char(max(observado), 'YYYY-MM-DD'), string_agg(DISTINCT proveedor, ', ') FROM ({ULTIMOS}) u GROUP BY 1 ORDER BY 2 DESC"), &[]).await.map_err(interno)?;
    let rubros: Vec<Value> = rubros.iter().map(|f| json!({ "rubro": f.get::<_, String>(0), "productos": f.get::<_, i64>(1), "proveedores": f.get::<_, i64>(2), "con_codigo": f.get::<_, i64>(3), "ultima": f.get::<_, Option<String>>(4), "tiendas": f.get::<_, Option<String>>(5) })).collect();
    let rubro = q.get("rubro").cloned().filter(|r| !r.is_empty()).or_else(|| rubros.first().and_then(|r| r["rubro"].as_str().map(String::from))).unwrap_or_default();
    let proveedor = q.get("proveedor").cloned().unwrap_or_default();
    // Cada palabra buscada debe aparecer en el nombre del producto (sin tildes ni mayúsculas).
    let palabras: Vec<String> = faro_homologa::normalizar::basico(q.get("q").map(String::as_str).unwrap_or("")).split(' ').filter(|p| !p.is_empty()).map(|p| format!("%{}%", p.to_lowercase())).collect();
    let umbral: f64 = q.get("umbral").and_then(|u| u.parse().ok()).filter(|u: &f64| *u > 0.0).unwrap_or(15.0);
    let filtro = format!("rubro = $1 AND ($2 = '' OR proveedor = $2) AND {PLANO} LIKE ALL($3::text[])");

    let resumen = e.bd.query_one(&format!("SELECT count(*), min(precio), percentile_cont(0.5) WITHIN GROUP (ORDER BY precio), max(precio), avg(precio), mode() WITHIN GROUP (ORDER BY precio) FROM ({ULTIMOS}) u WHERE {filtro}"), &[&rubro, &proveedor, &palabras]).await.map_err(interno)?;
    // Tiendas informantes del rubro: cuánto publica cada una y desde cuándo se le sigue.
    let informantes = e.bd.query(
        "SELECT proveedor, count(DISTINCT url), count(DISTINCT url) FILTER (WHERE ean <> ''), count(DISTINCT observado::date), to_char(min(observado), 'YYYY-MM-DD'), to_char(max(observado), 'YYYY-MM-DD'), count(DISTINCT termino), min(split_part(url, '/', 3))
         FROM precios_observados WHERE rubro = $1 GROUP BY 1 ORDER BY 2 DESC", &[&rubro]).await.map_err(interno)?;
    let informantes: Vec<Value> = informantes.iter().map(|f| json!({ "proveedor": f.get::<_, String>(0), "productos": f.get::<_, i64>(1), "con_codigo": f.get::<_, i64>(2), "dias": f.get::<_, i64>(3), "desde": f.get::<_, Option<String>>(4), "hasta": f.get::<_, Option<String>>(5), "categorias": f.get::<_, i64>(6), "sitio": f.get::<_, Option<String>>(7) })).collect();
    let filas = e.bd.query(&format!("SELECT proveedor, producto, marca, precio, precio_lista, ean, termino, url, to_char(observado, 'YYYY-MM-DD') FROM ({ULTIMOS}) u WHERE {filtro} ORDER BY precio LIMIT 200"), &[&rubro, &proveedor, &palabras]).await.map_err(interno)?;
    let filas: Vec<Value> = filas.iter().map(|f| json!({ "proveedor": f.get::<_, String>(0), "producto": f.get::<_, String>(1), "marca": f.get::<_, String>(2), "precio": f.get::<_, f64>(3), "precio_lista": f.get::<_, Option<f64>>(4),
        "ean": f.get::<_, String>(5), "categoria": f.get::<_, String>(6), "url": f.get::<_, String>(7), "fecha": f.get::<_, String>(8) })).collect();

    // Mismo código de barras en dos o más tiendas: es el mismo producto sin que nadie lo interprete.
    let iguales = e.bd.query(&format!(
        "SELECT ean, min(producto), count(DISTINCT proveedor), min(precio), max(precio),
                json_agg(json_build_object('proveedor', proveedor, 'precio', precio, 'url', url, 'producto', producto) ORDER BY precio)::text, count(*) OVER ()
         FROM ({ULTIMOS}) u WHERE {filtro} AND ean <> '' GROUP BY ean HAVING count(DISTINCT proveedor) >= 2 ORDER BY max(precio) / min(precio) DESC, ean LIMIT 100"), &[&rubro, &proveedor, &palabras]).await.map_err(interno)?;
    let total_iguales = iguales.first().map_or(0, |f| f.get::<_, i64>(6));
    let iguales: Vec<Value> = iguales.iter().map(|f| { let (a, b) = (f.get::<_, f64>(3), f.get::<_, f64>(4));
        json!({ "ean": f.get::<_, String>(0), "producto": f.get::<_, String>(1), "tiendas": f.get::<_, i64>(2), "minimo": a, "maximo": b, "diferencia_pct": (b / a - 1.0) * 100.0, "ofertas": serde_json::from_str::<Value>(&f.get::<_, String>(5)).unwrap_or(Value::Null) }) }).collect();

    // Cambio de precio del mismo producto en la misma tienda entre sus dos últimos días de muestreo.
    let cambios = e.bd.query(&format!(
        "WITH dias AS (SELECT DISTINCT ON (proveedor, url, observado::date) rubro, proveedor, url, producto, precio, observado::date AS dia FROM precios_observados WHERE rubro = $1 ORDER BY proveedor, url, observado::date, observado DESC),
              orden AS (SELECT *, row_number() OVER w AS n, lead(precio) OVER w AS antes, lead(dia) OVER w AS dia_antes FROM dias WINDOW w AS (PARTITION BY proveedor, url ORDER BY dia DESC))
         SELECT proveedor, producto, antes, precio, to_char(dia_antes, 'YYYY-MM-DD'), to_char(dia, 'YYYY-MM-DD'), url, count(*) OVER (), count(*) FILTER (WHERE abs(precio / antes - 1) * 100 >= $4) OVER ()
         FROM orden WHERE n = 1 AND antes > 0 AND {filtro} ORDER BY abs(precio / antes - 1) DESC LIMIT 100"), &[&rubro, &proveedor, &palabras, &umbral]).await.map_err(interno)?;
    let (comparados, atipicos) = cambios.first().map_or((0, 0), |f| (f.get::<_, i64>(7), f.get::<_, i64>(8)));
    let cambios: Vec<Value> = cambios.iter().map(|f| { let (antes, ahora) = (f.get::<_, f64>(2), f.get::<_, f64>(3)); let v = (ahora / antes - 1.0) * 100.0;
        json!({ "proveedor": f.get::<_, String>(0), "producto": f.get::<_, String>(1), "antes": antes, "ahora": ahora, "fecha_antes": f.get::<_, String>(4), "fecha_ahora": f.get::<_, String>(5), "url": f.get::<_, String>(6), "variacion_pct": v, "atipico": v.abs() >= umbral }) })
        .filter(|c| c["variacion_pct"].as_f64().is_some_and(|v| v != 0.0)).collect();

    Ok(Json(json!({ "rubros": rubros, "rubro": rubro, "total": resumen.get::<_, i64>(0), "minimo": resumen.get::<_, Option<f64>>(1), "mediana": resumen.get::<_, Option<f64>>(2), "maximo": resumen.get::<_, Option<f64>>(3), "media": resumen.get::<_, Option<f64>>(4), "moda": resumen.get::<_, Option<f64>>(5), "informantes": informantes,
        "filas": filas, "mismo_codigo": iguales, "total_mismo_codigo": total_iguales, "cambios": cambios, "comparados": comparados, "atipicos": atipicos, "umbral_pct": umbral })))
}

/// Los precios publicados de un rubro (el más reciente de cada producto de cada tienda), en CSV.
async fn api_publicados_csv(State(e): State<Compartido>, Query(q): Query<HashMap<String, String>>) -> Res<Response> {
    let rubro = q.get("rubro").cloned().unwrap_or_default();
    let proveedor = q.get("proveedor").cloned().unwrap_or_default();
    let palabras: Vec<String> = faro_homologa::normalizar::basico(q.get("q").map(String::as_str).unwrap_or("")).split(' ').filter(|p| !p.is_empty()).map(|p| format!("%{}%", p.to_lowercase())).collect();
    let filas = e.bd.query(
        "SELECT rubro, proveedor, producto, marca, precio, precio_lista, ean, termino, url, to_char(observado, 'YYYY-MM-DD') FROM
           (SELECT DISTINCT ON (rubro, proveedor, url) * FROM precios_observados ORDER BY rubro, proveedor, url, observado DESC) u
         WHERE ($1 = '' OR rubro = $1) AND ($2 = '' OR proveedor = $2) AND translate(lower(producto), 'áéíóúüñ', 'aeiouun') LIKE ALL($3::text[]) ORDER BY rubro, producto, precio LIMIT 300000",
        &[&rubro, &proveedor, &palabras]).await.map_err(interno)?;
    let campo = |t: String| if t.contains([',', '"', '\n']) { format!("\"{}\"", t.replace('"', "\"\"")) } else { t };
    let mut csv = String::from("rubro,tienda,producto,marca,precio_q,precio_lista_q,codigo_barras,categoria_tienda,url,fecha_consulta\n");
    for f in &filas {
        let texto = |i: usize| campo(f.get::<_, String>(i));
        csv.push_str(&format!("{},{},{},{},{},{},{},{},{},{}\n", texto(0), texto(1), texto(2), texto(3), f.get::<_, f64>(4), f.get::<_, Option<f64>>(5).map(|x| x.to_string()).unwrap_or_default(), texto(6), texto(7), texto(8), texto(9)));
    }
    Ok(([(axum::http::header::CONTENT_TYPE, "text/csv; charset=utf-8"), (axum::http::header::CONTENT_DISPOSITION, "attachment; filename=\"precios_publicados.csv\"")], csv).into_response())
}

/// Canasta básica: precio promedio del INE por producto (último mes y un año antes) y, al lado,
/// lo que publican las tiendas muestreadas. No se comparan solos: el INE no publica la unidad.
async fn api_canasta(State(e): State<Compartido>) -> Res<Json<Value>> {
    let periodos: std::collections::BTreeSet<&str> = e.canasta.iter().map(|(p, _, _)| p.as_str()).collect();
    let (Some(ultimo), Some(primero)) = (periodos.last().copied(), periodos.first().copied()) else {
        return Ok(Json(json!({ "disponible": false, "productos": [] })));
    };
    let anio_antes = format!("{}{}", ultimo[..4].parse::<i32>().unwrap_or(0) - 1, &ultimo[4..]);
    let observados = e.bd.query("SELECT DISTINCT ON (proveedor, producto) termino, proveedor, producto, precio, unidad, url, to_char(observado, 'YYYY-MM-DD') FROM precios_observados WHERE rubro = 'alimentos' ORDER BY proveedor, producto, observado DESC", &[]).await.map_err(interno)?;
    let mut nombres: Vec<&str> = e.canasta.iter().filter(|(p, _, _)| p == ultimo).map(|(_, n, _)| n.as_str()).collect();
    nombres.dedup();
    let productos: Vec<Value> = nombres.iter().map(|nombre| {
        let precio = |periodo: &str| e.canasta.iter().find(|(p, n, _)| p == periodo && n == nombre).map(|(_, _, x)| *x);
        let clave = faro_homologa::normalizar::basico(nombre);
        // Ofertas cuyo término de búsqueda aparece, palabra por palabra, en el nombre del producto del INE.
        let mut ofertas: Vec<Value> = observados.iter().filter(|f| { let t = faro_homologa::normalizar::basico(&f.get::<_, String>(0)); !t.is_empty() && t.split(' ').all(|w| clave.split(' ').any(|c| c.starts_with(w.trim_end_matches('s')))) })
            .map(|f| json!({ "proveedor": f.get::<_, String>(1), "producto": f.get::<_, String>(2), "precio": f.get::<_, f64>(3), "unidad": f.get::<_, String>(4), "url": f.get::<_, String>(5), "fecha": f.get::<_, String>(6) })).collect();
        ofertas.sort_by(|a, b| a["precio"].as_f64().unwrap_or(0.0).total_cmp(&b["precio"].as_f64().unwrap_or(0.0)));
        let (ahora, antes) = (precio(ultimo), precio(&anio_antes));
        json!({ "producto": nombre, "precio_ine": ahora, "precio_un_anio_antes": antes, "variacion_pct": ahora.zip(antes).filter(|(_, b)| *b > 0.0).map(|(a, b)| (a / b - 1.0) * 100.0),
                "serie": e.canasta.iter().filter(|(_, n, _)| n == nombre).map(|(p, _, x)| json!([p, x])).collect::<Vec<_>>(), "publicados": ofertas.len(), "ofertas": ofertas.into_iter().take(12).collect::<Vec<_>>() })
    }).collect();
    Ok(Json(json!({ "disponible": true, "periodo": ultimo, "desde": primero, "productos": productos, "observados": observados.len() })))
}

// ---------------------------------------------------------------------------------------------
// Ficha de municipio, plantillas, tabla de códigos y procedimientos descargables
// ---------------------------------------------------------------------------------------------

/// Todo lo que FARO sabe de un municipio, para la ficha del mapa y el reporte.
async fn api_municipio(State(e): State<Compartido>, Ruta(codigo): Ruta<String>) -> Res<Json<Value>> {
    let l = e.catalogo.por_codigo(&codigo).filter(|l| l.nivel == Nivel::Municipio).ok_or_else(|| pedido(format!("{codigo} no es un municipio")))?;
    let (desde, hasta) = e.poblacion.anios().unwrap_or((2018, 2030));
    let poblacion: Vec<Value> = (desde..=hasta).filter_map(|a| e.poblacion.fila(&codigo, a)).map(|p| json!([p.anio, p.total, p.hombres, p.mujeres])).collect();
    let serie = e.construido.get(&codigo).cloned().unwrap_or_default();
    let reciente: Vec<faro_territorio::Punto> = serie.iter().copied().filter(|p| p.anio >= 2000).collect();
    let analisis = faro_territorio::analizar(&codigo, &reciente, 2030);
    // Puesto entre los 340 municipios (1 = el mayor).
    let puesto = |valor: &dyn Fn(&str) -> Option<f64>| {
        let propio = valor(&codigo)?;
        Some(1 + e.poblacion.municipios().iter().filter(|m| valor(m).is_some_and(|v| v > propio)).count())
    };
    let hoy = 2026.clamp(desde, hasta);
    let sin_conexion: Vec<Value> = e.vial["sin_camino"].as_array().into_iter().flatten().filter(|x| x["codigo"].as_str().is_some_and(|c| c.starts_with(&codigo))).map(|x| json!({ "nombre": x["nombre"], "codigo": x["codigo"], "distancia_m": x["distancia_m"], "conexion": "sin_camino" }))
        .chain(e.vial["enlaces"].as_array().into_iter().flatten().flat_map(|en| en["lugares"].as_array().into_iter().flatten().filter(|x| x["codigo"].as_str().is_some_and(|c| c.starts_with(&codigo))).map(|x| json!({ "nombre": x["nombre"], "codigo": x["codigo"], "conexion": "tramo_suelto", "hueco_km": en["km"] })).collect::<Vec<_>>()))
        .collect();
    let registros = e.bd.query("SELECT solicitud, sum(n)::bigint FROM hechos WHERE municipio = $1 GROUP BY 1 ORDER BY 1", &[&codigo]).await.map_err(interno)?;
    Ok(Json(json!({
        "codigo": codigo, "nombre": l.nombre, "departamento": e.catalogo.por_codigo(&l.padre).map(|d| d.nombre.clone()), "anio_actual": hoy,
        "poblacion": poblacion, "poblacion_cambio": e.poblacion.cambio(&codigo, 2018, hasta),
        "puesto_poblacion": puesto(&|m| e.poblacion.total(m, hoy).map(|v| v as f64)), "puesto_crecimiento": puesto(&|m| e.poblacion.cambio(m, 2018, hasta).map(|c| c.tasa_anual_pct)),
        "construido": serie.iter().map(|p| json!([p.anio, (p.edificaciones * 10.0).round() / 10.0])).collect::<Vec<_>>(), "construido_analisis": analisis,
        "lugares_poblados": e.catalogo.de_nivel(Nivel::LugarPoblado, Some(&codigo)).count(), "sin_conexion": sin_conexion,
        "proveedores": e.construccion.proveedores.iter().filter(|p| p.get("municipio_codigo").map(String::as_str) == Some(codigo.as_str())).filter_map(|p| p.get("nombre")).collect::<Vec<_>>(),
        "registros": registros.iter().map(|f| json!({ "solicitud": f.get::<_, String>(0), "total": f.get::<_, i64>(1) })).collect::<Vec<_>>(),
        "talla": e.talla.get(&codigo),
        "cif": e.cif.iter().find(|f| f.get("departamento").map(String::as_str) == Some(&codigo[..2])),
        "epocas": e.epocas, "bloques": e.bloques.get(&codigo),
    })))
}

/// Archivo de ejemplo para una solicitud: el encabezado que se espera y dos filas ficticias.
async fn plantilla(State(e): State<Compartido>, Ruta(id): Ruta<String>) -> Response {
    let Some(sol) = solicitudes(&e).ok().and_then(|s| s.into_iter().find(|s| s.id == id)) else {
        return Fallo(StatusCode::NOT_FOUND, "no existe esa solicitud".into()).into_response();
    };
    let ejemplo = |c: &faro_homologa::solicitud::Columna, n: usize| -> String {
        use faro_homologa::solicitud::Tipo;
        match (c.homologar.as_deref(), c.sensible.as_deref()) {
            (Some("departamento"), _) => ["Escuintla", "Sololá"][n].into(),
            (Some("municipio"), _) => ["Iztapa", "Panajachel"][n].into(),
            (Some(_), _) => String::new(),
            (_, Some("seudonimizar")) => ["0000000000101", "0000000000202"][n].into(),
            (_, Some(_)) => "(no se usa)".into(),
            _ if !c.valores.is_empty() => c.valores[n % c.valores.len()].clone(),
            _ => match c.tipo {
                Tipo::Fecha => ["2026-10-03", "04/10/2026"][n].into(),
                Tipo::Entero => ["34", "7"][n].into(),
                Tipo::Decimal => ["1.5", "20.25"][n].into(),
                Tipo::Texto => String::new(),
            },
        }
    };
    let mut w = csv::Writer::from_writer(vec![]);
    let _ = w.write_record(sol.columnas.iter().map(|c| c.nombre.as_str()));
    for n in 0..2 {
        let _ = w.write_record(sol.columnas.iter().map(|c| ejemplo(c, n)));
    }
    let cuerpo = w.into_inner().unwrap_or_default();
    ([("content-type", "text/csv; charset=utf-8".to_string()), ("content-disposition", format!("attachment; filename=\"ejemplo_ficticio_{id}.csv\""))], cuerpo).into_response()
}

/// Carga una tabla de códigos de una institución (CSV: nivel, contexto, valor, codigo,
/// institucion) como equivalencias confirmadas por quien la sube. Las filas que no cuadran con
/// el catálogo se rechazan una por una y se devuelven.
async fn api_tabla_equivalencias(State(e): State<Compartido>, cabeceras: HeaderMap, Query(q): Query<HashMap<String, String>>, cuerpo: String) -> Res<Json<Value>> {
    let quien = autorizar(&e, &cabeceras).await?;
    let usuario = quien.nombre.as_str();
    let _ = &q;
    let mut lector = csv::ReaderBuilder::new().trim(csv::Trim::All).flexible(true).from_reader(cuerpo.trim_start_matches('\u{feff}').as_bytes());
    let cab: Vec<String> = lector.headers().map_err(pedido)?.iter().map(|c| c.to_lowercase()).collect();
    let pos = |n: &str| cab.iter().position(|c| c == n).ok_or_else(|| pedido(format!("falta la columna {n}")));
    let (inivel, ictx, ival, icod) = (pos("nivel")?, pos("contexto")?, pos("valor")?, pos("codigo")?);
    let iinst = cab.iter().position(|c| c == "institucion");
    let columnas: Vec<faro_homologa::solicitud::Columna> = solicitudes(&e).map_err(interno)?.into_iter().flat_map(|s| s.columnas).collect();
    let (mut grabadas, mut rechazadas) = (0, Vec::new());
    for (n, fila) in lector.records().enumerate() {
        let f = fila.map_err(pedido)?;
        let c = |i: usize| f.get(i).unwrap_or("");
        let (nivel, contexto, valor, codigo, institucion) = (c(inivel), c(ictx), c(ival), c(icod), iinst.map_or("", c));
        if valor.is_empty() || codigo.is_empty() {
            rechazadas.push(format!("fila {}: falta el valor o el código", n + 2));
            continue;
        }
        if nivel == "valor" {
            match columnas.iter().filter(|col| col.dimension() == contexto).find_map(|col| col.canonico(codigo).map(str::to_string)) {
                Some(canonico) => {
                    e.bd.execute("INSERT INTO equivalencias_valor (dimension, original, clave, valor, institucion, confirmado_por) VALUES ($1,$2,$3,$4,$5,$6)", &[&contexto, &valor, &faro_homologa::normalizar::basico(valor), &canonico, &institucion, &usuario]).await.map_err(interno)?;
                    grabadas += 1;
                }
                None => rechazadas.push(format!("fila {}: {codigo:?} no es un valor permitido de {contexto}", n + 2)),
            }
            continue;
        }
        match (nivel.parse::<Nivel>(), e.catalogo.por_codigo(codigo)) {
            (Ok(niv), Some(l)) if l.nivel == niv => {
                e.bd.execute("INSERT INTO equivalencias (original, clave, nivel, contexto, codigo, institucion, confirmado_por) VALUES ($1,$2,$3,$4,$5,$6,$7)", &[&valor, &faro_homologa::normalizar::clave(valor), &nivel, &contexto, &codigo, &institucion, &usuario]).await.map_err(interno)?;
                e.bd.execute("UPDATE pendientes SET resuelto = TRUE WHERE NOT resuelto AND nivel = $1 AND contexto = $2 AND valor = $3", &[&nivel, &contexto, &valor]).await.map_err(interno)?;
                grabadas += 1;
            }
            (Err(_), _) => rechazadas.push(format!("fila {}: nivel {nivel:?} no existe (departamento, municipio, lugar_poblado o valor)", n + 2)),
            _ => rechazadas.push(format!("fila {}: el código {codigo} no existe en el catálogo como {nivel}", n + 2)),
        }
    }
    Ok(Json(json!({ "grabadas": grabadas, "rechazadas": rechazadas })))
}

/// Procedimientos para repetir cada etapa fuera del portal (carpeta scripts/reproducir).
async fn script(State(e): State<Compartido>, Ruta(archivo): Ruta<String>) -> Response {
    let limpio = archivo.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.')) && !archivo.contains("..") && (archivo.ends_with(".sh") || archivo.ends_with(".ps1"));
    match (limpio, tokio::fs::read(e.dir_scripts.join(&archivo)).await) {
        (true, Ok(bytes)) => ([("content-type", "text/plain; charset=utf-8".to_string()), ("content-disposition", format!("attachment; filename=\"{archivo}\""))], bytes).into_response(),
        _ => Fallo(StatusCode::NOT_FOUND, "no existe ese procedimiento".into()).into_response(),
    }
}

// ---------------------------------------------------------------------------------------------
// Cuentas: inicio de sesión, perfil y manejo de usuarios
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize)]
struct Sesion {
    id: i64,
    usuario: String,
    nombre: String,
    rol: String,
    debe_cambiar: bool,
}

const ROLES: [&str; 3] = ["admin", "analista", "consulta"];
const HORAS_DE_SESION: i64 = 12;

fn huella_de_clave(clave: &str) -> Result<String, String> {
    use argon2::password_hash::{rand_core::OsRng, PasswordHasher, SaltString};
    argon2::Argon2::default().hash_password(clave.as_bytes(), &SaltString::generate(&mut OsRng)).map(|h| h.to_string()).map_err(|e| e.to_string())
}

fn clave_correcta(clave: &str, huella: &str) -> bool {
    use argon2::password_hash::{PasswordHash, PasswordVerifier};
    PasswordHash::new(huella).is_ok_and(|h| argon2::Argon2::default().verify_password(clave.as_bytes(), &h).is_ok())
}

fn huella_de_llave(llave: &str) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(llave.as_bytes()).iter().map(|b| format!("{b:02x}")).collect()
}

fn llave_de(cabeceras: &HeaderMap) -> Option<String> {
    cabeceras.get("cookie")?.to_str().ok()?.split(';').find_map(|c| c.trim().strip_prefix("faro_sesion=").map(str::to_string)).filter(|l| !l.is_empty())
}

async fn sesion_de(e: &Estado, cabeceras: &HeaderMap) -> Res<Option<Sesion>> {
    let Some(llave) = llave_de(cabeceras) else { return Ok(None) };
    let f = e.bd.query_opt("SELECT u.id, u.usuario, u.nombre, u.rol, u.debe_cambiar FROM sesiones s JOIN usuarios u ON u.id = s.usuario_id WHERE s.huella = $1 AND s.expira > now() AND u.activo", &[&huella_de_llave(&llave)]).await.map_err(interno)?;
    Ok(f.map(|f| Sesion { id: f.get(0), usuario: f.get(1), nombre: f.get(2), rol: f.get(3), debe_cambiar: f.get(4) }))
}

fn galleta(llave: &str, segundos: i64) -> (&'static str, String) {
    ("set-cookie", format!("faro_sesion={llave}; HttpOnly; SameSite=Strict; Path=/; Max-Age={segundos}"))
}

async fn api_sesion(State(e): State<Compartido>, cabeceras: HeaderMap) -> Res<Json<Value>> {
    Ok(Json(match sesion_de(&e, &cabeceras).await? {
        Some(s) => json!({ "sesion": true, "usuario": s }),
        None => json!({ "sesion": false }),
    }))
}

#[derive(Deserialize)]
struct Ingreso {
    usuario: String,
    clave: String,
}

async fn api_ingresar(State(e): State<Compartido>, Json(i): Json<Ingreso>) -> Result<Response, Fallo> {
    let f = e.bd.query_opt("SELECT id, usuario, nombre, rol, debe_cambiar, clave FROM usuarios WHERE usuario = $1 AND activo", &[&i.usuario.trim().to_lowercase()]).await.map_err(interno)?;
    let (clave, guardada) = (i.clave.clone(), f.as_ref().map(|f| f.get::<_, String>(5)));
    // Verificar cuesta lo mismo exista o no el usuario, y un fallo se demora: frena los intentos en serie.
    let bien = tokio::task::spawn_blocking(move || guardada.is_some_and(|g| clave_correcta(&clave, &g))).await.map_err(interno)?;
    let Some(f) = f.filter(|_| bien) else {
        tokio::time::sleep(std::time::Duration::from_millis(700)).await;
        return Err(Fallo(StatusCode::UNAUTHORIZED, "usuario o contraseña incorrectos".into()));
    };
    let llave = {
        use rand_core::{OsRng, RngCore};
        let mut b = [0u8; 32];
        OsRng.fill_bytes(&mut b);
        b.iter().map(|x| format!("{x:02x}")).collect::<String>()
    };
    let id: i64 = f.get(0);
    e.bd.execute("DELETE FROM sesiones WHERE expira < now()", &[]).await.map_err(interno)?;
    e.bd.execute("INSERT INTO sesiones (huella, usuario_id, expira) VALUES ($1, $2, now() + make_interval(hours => $3))", &[&huella_de_llave(&llave), &id, &(HORAS_DE_SESION as i32)]).await.map_err(interno)?;
    e.bd.execute("UPDATE usuarios SET ultimo_ingreso = now() WHERE id = $1", &[&id]).await.map_err(interno)?;
    let s = Sesion { id, usuario: f.get(1), nombre: f.get(2), rol: f.get(3), debe_cambiar: f.get(4) };
    Ok(([galleta(&llave, HORAS_DE_SESION * 3600)], Json(json!({ "sesion": true, "usuario": s }))).into_response())
}

async fn api_salir(State(e): State<Compartido>, cabeceras: HeaderMap) -> Result<Response, Fallo> {
    if let Some(llave) = llave_de(&cabeceras) {
        e.bd.execute("DELETE FROM sesiones WHERE huella = $1", &[&huella_de_llave(&llave)]).await.map_err(interno)?;
    }
    Ok(([galleta("", 0)], Json(json!({ "sesion": false }))).into_response())
}

fn clave_aceptable(clave: &str) -> Res<()> {
    if clave.chars().count() < 8 {
        return Err(pedido("la contraseña debe tener al menos 8 caracteres"));
    }
    Ok(())
}

#[derive(Deserialize)]
struct CambioPerfil {
    nombre: Option<String>,
    clave_actual: Option<String>,
    clave_nueva: Option<String>,
}

async fn api_perfil(State(e): State<Compartido>, cabeceras: HeaderMap, Json(c): Json<CambioPerfil>) -> Res<Json<Value>> {
    let s = sesion_de(&e, &cabeceras).await?.ok_or(Fallo(StatusCode::UNAUTHORIZED, "inicie sesión".into()))?;
    if let Some(nombre) = c.nombre.as_deref().map(str::trim).filter(|n| !n.is_empty()) {
        e.bd.execute("UPDATE usuarios SET nombre = $1 WHERE id = $2", &[&nombre, &s.id]).await.map_err(interno)?;
    }
    if let Some(nueva) = c.clave_nueva.filter(|n| !n.is_empty()) {
        clave_aceptable(&nueva)?;
        let guardada: String = e.bd.query_one("SELECT clave FROM usuarios WHERE id = $1", &[&s.id]).await.map_err(interno)?.get(0);
        let actual = c.clave_actual.unwrap_or_default();
        let huella = tokio::task::spawn_blocking(move || if clave_correcta(&actual, &guardada) { huella_de_clave(&nueva).map(Some) } else { Ok(None) }).await.map_err(interno)?.map_err(interno)?;
        let huella = huella.ok_or_else(|| pedido("la contraseña actual no es correcta"))?;
        e.bd.execute("UPDATE usuarios SET clave = $1, debe_cambiar = FALSE WHERE id = $2", &[&huella, &s.id]).await.map_err(interno)?;
    }
    Ok(Json(json!({ "guardado": true })))
}

async fn solo_admin(e: &Estado, cabeceras: &HeaderMap) -> Res<Sesion> {
    sesion_de(e, cabeceras).await?.filter(|s| s.rol == "admin").ok_or(Fallo(StatusCode::FORBIDDEN, "solo un administrador puede manejar usuarios".into()))
}

async fn api_usuarios(State(e): State<Compartido>, cabeceras: HeaderMap) -> Res<Json<Value>> {
    solo_admin(&e, &cabeceras).await?;
    let filas = e.bd.query("SELECT id, usuario, nombre, rol, activo, debe_cambiar, to_char(creado, 'YYYY-MM-DD'), to_char(ultimo_ingreso, 'YYYY-MM-DD HH24:MI') FROM usuarios ORDER BY id", &[]).await.map_err(interno)?;
    Ok(Json(json!(filas.iter().map(|f| json!({ "id": f.get::<_, i64>(0), "usuario": f.get::<_, String>(1), "nombre": f.get::<_, String>(2), "rol": f.get::<_, String>(3), "activo": f.get::<_, bool>(4),
        "debe_cambiar": f.get::<_, bool>(5), "creado": f.get::<_, String>(6), "ultimo_ingreso": f.get::<_, Option<String>>(7) })).collect::<Vec<_>>())))
}

#[derive(Deserialize)]
struct UsuarioNuevo {
    usuario: String,
    nombre: String,
    rol: String,
    clave: String,
}

async fn api_usuario_crear(State(e): State<Compartido>, cabeceras: HeaderMap, Json(u): Json<UsuarioNuevo>) -> Res<Json<Value>> {
    solo_admin(&e, &cabeceras).await?;
    let usuario = u.usuario.trim().to_lowercase();
    if usuario.len() < 3 || !usuario.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-')) {
        return Err(pedido("el usuario debe tener al menos 3 caracteres: letras sin tilde, números, punto, guion o guion bajo"));
    }
    if u.nombre.trim().is_empty() || !ROLES.contains(&u.rol.as_str()) {
        return Err(pedido("indique el nombre y un rol: admin, analista o consulta"));
    }
    clave_aceptable(&u.clave)?;
    let clave = u.clave.clone();
    let huella = tokio::task::spawn_blocking(move || huella_de_clave(&clave)).await.map_err(interno)?.map_err(interno)?;
    let n = e.bd.execute("INSERT INTO usuarios (usuario, nombre, rol, clave, debe_cambiar) VALUES ($1,$2,$3,$4, TRUE) ON CONFLICT (usuario) DO NOTHING", &[&usuario, &u.nombre.trim(), &u.rol, &huella]).await.map_err(interno)?;
    if n == 0 {
        return Err(pedido(format!("ya existe el usuario {usuario}")));
    }
    Ok(Json(json!({ "creado": usuario })))
}

#[derive(Deserialize)]
struct CambioUsuario {
    nombre: Option<String>,
    rol: Option<String>,
    activo: Option<bool>,
    clave: Option<String>,
}

async fn api_usuario_cambiar(State(e): State<Compartido>, Ruta(id): Ruta<i64>, cabeceras: HeaderMap, Json(c): Json<CambioUsuario>) -> Res<Json<Value>> {
    let admin = solo_admin(&e, &cabeceras).await?;
    let actual = e.bd.query_opt("SELECT rol, activo FROM usuarios WHERE id = $1", &[&id]).await.map_err(interno)?.ok_or_else(|| pedido("no existe ese usuario"))?;
    if c.rol.as_deref().is_some_and(|r| !ROLES.contains(&r)) {
        return Err(pedido("rol no válido: admin, analista o consulta"));
    }
    // Nunca se queda el sistema sin un administrador activo.
    let deja_de_ser_admin = actual.get::<_, String>(0) == "admin" && actual.get::<_, bool>(1) && (c.rol.as_deref().is_some_and(|r| r != "admin") || c.activo == Some(false));
    if deja_de_ser_admin {
        if id == admin.id {
            return Err(pedido("no puede quitarse a usted mismo el rol de administrador ni desactivarse"));
        }
        let otros: i64 = e.bd.query_one("SELECT count(*) FROM usuarios WHERE rol = 'admin' AND activo AND id <> $1", &[&id]).await.map_err(interno)?.get(0);
        if otros == 0 {
            return Err(pedido("debe quedar al menos un administrador activo"));
        }
    }
    if let Some(nombre) = c.nombre.as_deref().map(str::trim).filter(|n| !n.is_empty()) {
        e.bd.execute("UPDATE usuarios SET nombre = $1 WHERE id = $2", &[&nombre, &id]).await.map_err(interno)?;
    }
    if let Some(rol) = &c.rol {
        e.bd.execute("UPDATE usuarios SET rol = $1 WHERE id = $2", &[rol, &id]).await.map_err(interno)?;
    }
    if let Some(activo) = c.activo {
        e.bd.execute("UPDATE usuarios SET activo = $1 WHERE id = $2", &[&activo, &id]).await.map_err(interno)?;
        if !activo {
            e.bd.execute("DELETE FROM sesiones WHERE usuario_id = $1", &[&id]).await.map_err(interno)?;
        }
    }
    if let Some(clave) = c.clave.filter(|k| !k.is_empty()) {
        clave_aceptable(&clave)?;
        let huella = tokio::task::spawn_blocking(move || huella_de_clave(&clave)).await.map_err(interno)?.map_err(interno)?;
        // Contraseña puesta por el administrador: el usuario debe cambiarla y sus sesiones se cierran.
        e.bd.execute("UPDATE usuarios SET clave = $1, debe_cambiar = TRUE WHERE id = $2", &[&huella, &id]).await.map_err(interno)?;
        e.bd.execute("DELETE FROM sesiones WHERE usuario_id = $1", &[&id]).await.map_err(interno)?;
    }
    Ok(Json(json!({ "guardado": true })))
}
