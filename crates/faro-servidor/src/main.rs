//! Servidor de FARO: una sola aplicación que recibe envíos, muestra lo pendiente, guarda las
//! equivalencias confirmadas y se mantiene al día con el portal del INE.
mod campo;
mod tablero;

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
    /// Censo 2018 por lugar poblado: código -> [población, hombres, mujeres, 0–14, 15–64, 65+, urbana, rural, hogares, viviendas, ocupadas].
    censo_lugar: HashMap<String, Vec<i64>>,
    /// Población por municipio, año y grupo quinquenal: código -> [(año, edad desde, hombres, mujeres)].
    edades: HashMap<String, Vec<(i32, u32, i64, i64)>>,
    /// Inseguridad alimentaria aguda por departamento (CIF, SESAN).
    cif: Vec<HashMap<String, String>>,
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
    /// Comparativo «antes · con FARO · cómo escala» (datos/estandar/mejoras.json).
    mejoras: Value,
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
    /// Uso del suelo por bloque de 1 km (GHSL 2020): (longitud, latitud, ha construidas, ha no residenciales).
    uso_bloques: Vec<(f64, f64, f64, f64)>,
    /// Material de techo más frecuente por lugar poblado (Censo 2018): código -> (material, % de viviendas).
    techo_lugar: HashMap<String, (String, f64)>,
    /// Servicios de formularios admitidos para la encuesta de campo (FARO_FORMULARIOS).
    formularios: Vec<String>,
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
    // La cadena de conexión no tiene valor por omisión: ninguna clave vive en el código.
    let url = std::env::var("DATABASE_URL").ok().filter(|v| !v.is_empty()).ok_or("defina DATABASE_URL (por ejemplo: host=localhost user=faro password=… dbname=faro)")?;
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
    // Ninguna consulta puede ocupar la base más de medio minuto.
    bd.batch_execute("SET statement_timeout = '30s'").await.map_err(|e| e.to_string())?;

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
    // Los últimos precios quedan calculados desde el arranque, para que la primera visita no espere.
    if let Err(e) = asegurar_ultimos(&bd).await {
        eprintln!("AVISO: no se pudo preparar la vista de últimos precios ({e}).");
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
    // Primer arranque: un administrador inicial que debe cambiar su contraseña. La contraseña inicial
    // no está en el código: la da la instalación (FARO_CLAVE_INICIAL; el despliegue con Docker la
    // deja en «admin», como dice el README) y, si no la da, se genera al azar y se escribe una vez
    // en el registro del servidor.
    if bd.query_one("SELECT count(*) FROM usuarios", &[]).await.map_err(|e| e.to_string())?.get::<_, i64>(0) == 0 {
        let (inicial, al_azar) = match std::env::var("FARO_CLAVE_INICIAL").ok().filter(|v| !v.is_empty()) {
            Some(c) => (c, false),
            None => {
                use rand_core::{OsRng, RngCore};
                let mut b = [0u8; 9];
                OsRng.fill_bytes(&mut b);
                (b.iter().map(|x| format!("{x:02x}")).collect::<String>(), true)
            }
        };
        bd.execute("INSERT INTO usuarios (usuario, nombre, rol, clave, debe_cambiar) VALUES ('admin', 'Administrador', 'admin', $1, TRUE)", &[&huella_de_clave(&inicial)?]).await.map_err(|e| e.to_string())?;
        if al_azar {
            eprintln!("AVISO: se creó el usuario inicial «admin». Su contraseña inicial, generada al azar, es: {inicial}  (no se volverá a mostrar; cámbiela al entrar)");
        } else {
            eprintln!("AVISO: se creó el usuario inicial «admin» con la contraseña inicial indicada en FARO_CLAVE_INICIAL. Cámbiela al entrar.");
        }
    }
    let estado = Arc::new(Estado {
        bd,
        uso_bloques: csv::Reader::from_path(dir_referencia.join("uso_construido_bloques.csv")).map(|mut l| l.records().flatten().filter_map(|f| Some((f.get(1)?.parse().ok()?, f.get(2)?.parse().ok()?, f.get(3)?.parse().ok()?, f.get(4)?.parse().ok()?))).collect()).unwrap_or_default(),
        techo_lugar: leer_techo_lugar(&dir_referencia),
        formularios: var("FARO_FORMULARIOS", "forms.office.com,forms.microsoft.com,forms.cloud.microsoft,forms.office365.us").split(',').map(|x| x.trim().to_lowercase()).filter(|x| !x.is_empty()).collect(),
        vial,
        referencias: csv::Reader::from_path(var("FARO_REFERENCIAS_INE", "datos/catalogo_precios/precios_referencia_ine.csv")).map(|mut l| l.deserialize().flatten().collect()).unwrap_or_default(),
        cif: cif.clone(),
        talla: talla.clone(),
        censo_lugar: leer_censo_lugar(&dir_referencia),
        edades: {
            let mut m: HashMap<String, Vec<(i32, u32, i64, i64)>> = HashMap::new();
            if let Ok(mut l) = csv::Reader::from_path(dir_referencia.join("poblacion_municipio_edades.csv")) {
                for p in l.deserialize::<faro_territorio::poblacion::PoblacionEdad>().flatten() {
                    m.entry(p.municipio).or_default().push((p.anio, p.edad_desde, p.hombres, p.mujeres));
                }
            }
            m
        },
        tareas: faro_fuentes::muestreo::cargar_tareas(&PathBuf::from(var("FARO_MUESTREOS", "datos/estandar/muestreos.toml"))).unwrap_or_else(|e| {
            eprintln!("AVISO: sin tareas de muestreo ({e}).");
            vec![]
        }),
        canasta: csv::Reader::from_path(var("FARO_CANASTA", "datos/catalogo_precios/cba_precios.csv")).map(|mut l| l.deserialize::<HashMap<String, String>>().flatten().filter_map(|f| Some((f.get("periodo")?.clone(), f.get("producto")?.clone(), f.get("precio_promedio")?.parse().ok()?))).collect()).unwrap_or_default(),
        bloques,
        epocas,
        dir_scripts: PathBuf::from(var("FARO_SCRIPTS", "scripts/reproducir")),
        mejoras: std::fs::read_to_string(var("FARO_MEJORAS", "datos/estandar/mejoras.json")).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_else(|| json!({ "areas": [], "puntos": [] })),
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
        .route("/api/envios/:solicitud", axum::routing::delete(api_envio_borrar))
        .route("/api/envios/:solicitud", post(api_enviar).layer(DefaultBodyLimit::max(300 * 1024 * 1024)))
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
        .route("/api/mejoras", get(|State(e): State<Compartido>| async move { Json(e.mejoras.clone()) }))
        .route("/api/reproducir/poblacion", post(api_reproducir_poblacion))
        .route("/api/sesion", get(api_sesion).post(api_ingresar).delete(api_salir))
        .route("/api/perfil", post(api_perfil))
        .route("/api/usuarios", get(api_usuarios).post(api_usuario_crear))
        .route("/api/usuarios/:id", post(api_usuario_cambiar))
        .route("/api/grupos", get(api_grupos).post(api_grupo_guardar))
        .route("/api/grupos/:id", post(api_grupo_guardar_id).delete(api_grupo_borrar))
        .route("/api/tableros/interpretar", post(api_tablero_interpretar))
        .route("/api/tableros", get(api_tableros).post(api_tablero_guardar))
        .route("/api/tableros/:id", get(api_tablero).post(api_tablero_guardar_id).delete(api_tablero_borrar))
        .route("/api/tableros/:id/publicar", post(api_tablero_publicar))
        .route("/api/bitacora", get(api_bitacora))
        .route("/api/campo", get(api_campo))
        .route("/api/campo/rutas", post(api_campo_ruta))
        .route("/api/campo/rutas/:id", axum::routing::delete(api_campo_ruta_borrar))
        .route("/api/campo/visitas", post(api_campo_visita))
        .route("/api/campo/demo", post(api_campo_demo).delete(api_campo_demo_borrar))
        .route("/api/campo/visitas.csv", get(api_campo_visitas_csv))
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
        .route("/api/publicados/decision", post(api_decision_precio))
        .route("/api/muestreos/ejecutar_todas", post(api_muestreos_todas))
        .route("/api/informe_precio", get(api_informe_precio))
        .route("/informe-precio", get(|| async { Html(include_str!("../web/informe_precio.html")) }))
        .route("/api/balance", post(api_balance).get(api_balance_oficial))
        .route("/api/buscar_lugar", get(api_buscar_lugar))
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
        .route("/idiomas.js", get(|| async { ([("content-type", "text/javascript; charset=utf-8"), ("cache-control", "no-cache")], include_str!("../web/idiomas.js")) }))
        .route("/api/idiomas", get(api_idiomas))
        .route("/api/idiomas/:codigo", get(api_idioma))
        .route("/vendor/ubuntu-sans/ubuntu-sans-400.woff2", get(|| async { ([("content-type", "font/woff2"), ("cache-control", "public, max-age=604800")], include_bytes!("../web/vendor/ubuntu-sans/ubuntu-sans-400.woff2").as_slice()) }))
        .route("/vendor/ubuntu-sans/ubuntu-sans-500.woff2", get(|| async { ([("content-type", "font/woff2"), ("cache-control", "public, max-age=604800")], include_bytes!("../web/vendor/ubuntu-sans/ubuntu-sans-500.woff2").as_slice()) }))
        .route("/vendor/ubuntu-sans/ubuntu-sans-600.woff2", get(|| async { ([("content-type", "font/woff2"), ("cache-control", "public, max-age=604800")], include_bytes!("../web/vendor/ubuntu-sans/ubuntu-sans-600.woff2").as_slice()) }))
        .route("/vendor/ubuntu-sans/ubuntu-sans-700.woff2", get(|| async { ([("content-type", "font/woff2"), ("cache-control", "public, max-age=604800")], include_bytes!("../web/vendor/ubuntu-sans/ubuntu-sans-700.woff2").as_slice()) }))
        .route("/vendor/ine-horizontal.svg", get(|| async { ([("content-type", "image/svg+xml"), ("cache-control", "public, max-age=604800")], include_str!("../web/vendor/ine-horizontal.svg")) }))
        .route("/api/certificaciones", post(api_certificar))
        .route("/api/certificaciones/:folio", get(api_certificacion))
        .route("/vendor/ine-horizontal-blanco.svg", get(|| async { ([("content-type", "image/svg+xml"), ("cache-control", "public, max-age=604800")], include_str!("../web/vendor/ine-horizontal-blanco.svg")) }))
        .route("/vendor/leaflet.js", get(|| async { ([("content-type", "text/javascript; charset=utf-8"), ("cache-control", "public, max-age=86400")], include_str!("../web/vendor/leaflet.js")) }))
        .route("/vendor/leaflet.css", get(|| async { ([("content-type", "text/css; charset=utf-8"), ("cache-control", "public, max-age=86400")], include_str!("../web/vendor/leaflet.css")) }))
        // Tope general de 20 MB; solo la recepción de envíos admite archivos grandes.
        .layer(DefaultBodyLimit::max(20 * 1024 * 1024))
        .layer(axum::middleware::from_fn(seguridad))
        .with_state(estado);

    let puerto = var("FARO_PUERTO", "8080");
    // Por omisión solo atiende en esta máquina; para abrirlo a la red: FARO_ESCUCHA=0.0.0.0 (detrás de HTTPS).
    let escucha = var("FARO_ESCUCHA", "127.0.0.1");
    let oyente = tokio::net::TcpListener::bind(format!("{escucha}:{puerto}")).await.map_err(|e| format!("no se pudo escuchar en {escucha}:{puerto}: {e}"))?;
    eprintln!("FARO escuchando en http://{escucha}:{puerto}");
    axum::serve(oyente, app)
        .with_graceful_shutdown(async { let _ = tokio::signal::ctrl_c().await; })
        .await
        .map_err(|e| e.to_string())
}

/// Dos defensas para todas las rutas: (1) una petición que cambia datos y trae `Origin` de otro
/// sitio se rechaza (falsificación de peticiones); (2) toda respuesta lleva cabeceras que impiden
/// incrustar el portal en otra página, adivinar tipos de contenido o cargar código de terceros.
async fn seguridad(pedido: axum::extract::Request, siguiente: axum::middleware::Next) -> Response {
    use axum::http::{header, HeaderValue, Method};
    if !matches!(*pedido.method(), Method::GET | Method::HEAD | Method::OPTIONS) {
        let cab = pedido.headers();
        let (origen, anfitrion) = (cab.get(header::ORIGIN).and_then(|v| v.to_str().ok()), cab.get(header::HOST).and_then(|v| v.to_str().ok()));
        if let (Some(o), Some(h)) = (origen, anfitrion) {
            if o.split("://").nth(1) != Some(h) {
                return (StatusCode::FORBIDDEN, Json(json!({ "error": "petición rechazada: viene de otro sitio" }))).into_response();
            }
        }
    }
    let mut respuesta = siguiente.run(pedido).await;
    let c = respuesta.headers_mut();
    c.insert("x-content-type-options", HeaderValue::from_static("nosniff"));
    c.insert("x-frame-options", HeaderValue::from_static("DENY"));
    c.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    // La posición y la cámara solo las puede pedir el propio portal (registro de visitas de campo); nadie más.
    c.insert("permissions-policy", HeaderValue::from_static("geolocation=(self), camera=(self), microphone=()"));
    // Los guiones y estilos van dentro de la propia página (por eso 'unsafe-inline'); la tipografía la
    // sirve el propio portal; de fuera solo se admiten las teselas del mapa base y las fotos, por https.
    c.insert("content-security-policy", HeaderValue::from_static("default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; font-src 'self'; img-src 'self' data: https:; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'; object-src 'none'"));
    respuesta
}

/// Devuelve quién hace la operación. Las operaciones que cambian datos exigen una sesión de
/// analista o administrador que ya cambió su contraseña inicial. La clave de servicio
/// (FARO_TOKEN, cabecera `x-faro-token`) queda para procesos automáticos y actúa como analista.
async fn autorizar(e: &Estado, cabeceras: &HeaderMap) -> Res<Sesion> {
    if let Some(s) = sesion_de(e, cabeceras).await? {
        if s.debe_cambiar {
            return Err(Fallo(StatusCode::FORBIDDEN, "antes de hacer cambios debe cambiar su contraseña inicial en «Mi perfil»".into()));
        }
        // Los operadores de campo solo registran visitas (eso se autoriza aparte, en `operador_de_campo`).
        // Las cuentas de institución solo envían sus archivos (eso se autoriza aparte, en `quien_envia`).
        return if s.rol != "admin" && s.rol != "analista" { Err(Fallo(StatusCode::FORBIDDEN, format!("su usuario es de {}: no puede hacer cambios en esta parte", match s.rol.as_str() { "consulta" => "consulta", "operador" => "campo", _ => "institución" }))) } else { Ok(s) };
    }
    // La clave de servicio se compara por su huella y recorriendo todos los bytes (tiempo constante).
    let igual = |a: &str, b: &str| { use sha2::{Digest, Sha256}; let (x, y) = (Sha256::digest(a.as_bytes()), Sha256::digest(b.as_bytes())); x.iter().zip(y.iter()).fold(0u8, |d, (p, q)| d | (p ^ q)) == 0 };
    match (&e.token, cabeceras.get("x-faro-token").and_then(|v| v.to_str().ok())) {
        (Some(t), Some(dada)) if igual(t, dada) => Ok(Sesion { id: 0, usuario: "servicio".into(), nombre: "Clave de servicio".into(), rol: "analista".into(), debe_cambiar: false, secciones: None, institucion: String::new() }),
        _ => Err(Fallo(StatusCode::UNAUTHORIZED, "inicie sesión para hacer cambios".into())),
    }
}

/// Lecturas que muestran lo que llegó de las instituciones: piden una sesión de cualquier rol.
async fn con_sesion(e: &Estado, cabeceras: &HeaderMap) -> Res<Sesion> {
    let s = sesion_de(e, cabeceras).await?.ok_or(Fallo(StatusCode::UNAUTHORIZED, "inicie sesión para ver esta información".into()))?;
    // Con la contraseña inicial (conocida por quien instaló o creó la cuenta) no se lee nada interno.
    if s.debe_cambiar {
        return Err(Fallo(StatusCode::FORBIDDEN, "cambie su contraseña inicial en «Mi perfil» para ver esta información".into()));
    }
    Ok(s)
}

/// Lo que llegó de todas las instituciones (pendientes, equivalencias) lo ve solo el personal del
/// INE: administradores, analistas y cuentas de consulta. No las cuentas de institución ni de campo.
async fn personal_del_ine(e: &Estado, cabeceras: &HeaderMap) -> Res<Sesion> {
    let s = con_sesion(e, cabeceras).await?;
    if matches!(s.rol.as_str(), "admin" | "analista" | "consulta") { Ok(s) } else { Err(Fallo(StatusCode::FORBIDDEN, "esta información es de uso interno del INE".into())) }
}

/// Quién puede subir un envío de esta solicitud: un analista o administrador con acceso a la
/// recepción, o una cuenta de institución cuando la solicitud es de su misma institución.
async fn quien_envia(e: &Estado, cabeceras: &HeaderMap, sol: &Solicitud) -> Res<Sesion> {
    match sesion_de(e, cabeceras).await? {
        Some(s) if s.rol == "institucion" => {
            if s.debe_cambiar {
                return Err(Fallo(StatusCode::FORBIDDEN, "antes de enviar debe cambiar su contraseña inicial en «Mi perfil»".into()));
            }
            if s.institucion.is_empty() || s.institucion != sol.institucion {
                return Err(Fallo(StatusCode::FORBIDDEN, format!("su usuario envía por {}: no puede subir archivos de {}", if s.institucion.is_empty() { "ninguna institución" } else { &s.institucion }, sol.institucion)));
            }
            Ok(s)
        }
        _ => autorizar_en(e, cabeceras, "recibir").await,
    }
}

/// Como `autorizar`, y además el grupo del usuario debe incluir la sección (los administradores entran a todas).
async fn autorizar_en(e: &Estado, cabeceras: &HeaderMap, seccion: &str) -> Res<Sesion> {
    let s = autorizar(e, cabeceras).await?;
    if s.rol != "admin" && s.secciones.as_ref().is_some_and(|v| !v.iter().any(|x| x == seccion)) {
        return Err(Fallo(StatusCode::FORBIDDEN, format!("su grupo no tiene acceso a esta parte del portal ({seccion})")));
    }
    Ok(s)
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

/// Huella de una solicitud: cambia cuando cambia cualquiera de sus reglas (columnas, alias, país,
/// corte, valores sin dato...). Sirve para saber si un envío se analizó con las reglas de hoy.
fn huella_de_reglas(sol: &Solicitud) -> String {
    use sha2::{Digest, Sha256};
    // La solicitud trae tablas sin orden fijo (las equivalencias de cada columna): escrita tal cual,
    // la misma solicitud daría huellas distintas. Por eso se escribe con las claves ordenadas.
    fn ordenado(v: &Value) -> String {
        match v {
            Value::Object(m) => {
                let mut claves: Vec<&String> = m.keys().collect();
                claves.sort();
                format!("{{{}}}", claves.iter().map(|k| format!("{k:?}:{}", ordenado(&m[*k]))).collect::<Vec<_>>().join(","))
            }
            Value::Array(a) => format!("[{}]", a.iter().map(ordenado).collect::<Vec<_>>().join(",")),
            otro => otro.to_string(),
        }
    }
    let texto = serde_json::to_value(sol).map(|v| ordenado(&v)).unwrap_or_default();
    Sha256::digest(texto.as_bytes()).iter().take(6).map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod pruebas_de_reglas {
    use super::*;

    /// Marcas (`<1>`, `</1>`, `<1/>`), datos (`{0}`) y números (`#`) de una clave o de su traducción.
    fn piezas_de(texto: &str) -> (Vec<String>, Vec<String>, usize) {
        let (mut marcas, mut datos) = (Vec::new(), Vec::new());
        let c: Vec<char> = texto.chars().collect();
        let mut i = 0;
        while i < c.len() {
            if c[i] == '<' {
                if let Some(f) = c[i..].iter().position(|x| *x == '>').map(|p| i + p) {
                    let dentro: String = c[i + 1..f].iter().collect();
                    if !dentro.is_empty() && dentro.trim_matches('/').chars().all(|x| x.is_ascii_digit()) {
                        marcas.push(dentro);
                        i = f + 1;
                        continue;
                    }
                }
            } else if c[i] == '{' && i + 2 < c.len() && c[i + 1].is_ascii_digit() && c[i + 2] == '}' {
                datos.push(c[i + 1].to_string());
            }
            i += 1;
        }
        marcas.sort();
        datos.sort();
        (marcas, datos, texto.matches('#').count())
    }

    #[test]
    fn el_codigo_de_idioma_no_deja_salir_de_la_carpeta() {
        assert!(codigo_de_idioma("en") && codigo_de_idioma("cak"));
        for malo in ["", "e", "EN", "espa", "../x", "en.json", "e/", "é"] {
            assert!(!codigo_de_idioma(malo), "{malo:?} no es un código de idioma");
        }
    }

    /// Palabras de un texto en kaqchikel, sin marcas, entidades, números ni datos; el saltillo cuenta
    /// como letra dentro de la palabra.
    fn palabras_de(texto: &str) -> Vec<String> {
        let c: Vec<char> = texto.replace(['’', '‘', '`'], "'").chars().collect();
        let (mut salida, mut actual, mut i) = (Vec::new(), String::new(), 0);
        let mut cerrar = |w: &mut String, salida: &mut Vec<String>| {
            let limpia = w.trim_matches('\'').to_lowercase();
            if !limpia.is_empty() { salida.push(limpia); }
            w.clear();
        };
        while i < c.len() {
            let hasta = |fin: char, vale: &dyn Fn(char) -> bool| c[i + 1..].iter().position(|x| *x == fin).filter(|p| c[i + 1..i + 1 + p].iter().all(|x| vale(*x))).map(|p| i + 1 + p);
            let salto = match c[i] {
                '<' => hasta('>', &|x| x.is_ascii_digit() || x == '/'),
                '&' => hasta(';', &|x| x.is_ascii_lowercase()),
                '{' => hasta('}', &|x| x.is_ascii_digit()),
                _ => None,
            };
            if let Some(f) = salto { cerrar(&mut actual, &mut salida); i = f + 1; continue; }
            if c[i].is_alphabetic() || c[i] == '\'' { actual.push(c[i]); } else { cerrar(&mut actual, &mut salida); }
            i += 1;
        }
        cerrar(&mut actual, &mut salida);
        salida
    }

    /// Cada traducción conserva las piezas de su clave: si no, el portal la descartaría en silencio.
    /// En kaqchikel, además, cada rótulo debe constar en la hoja de revisión y cada una de sus palabras
    /// en la lista de palabras con fuente: no entra nada sin procedencia.
    #[test]
    fn los_diccionarios_conservan_marcas_numeros_y_datos() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../datos/estandar/idiomas");
        let leer = |n: &str| -> Value { serde_json::from_str(&std::fs::read_to_string(dir.join(n)).unwrap()).unwrap_or_else(|e| panic!("{n}: {e}")) };
        let plantilla: std::collections::HashSet<String> = leer("plantilla.json")["textos"].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect();
        assert!(plantilla.len() > 1500, "la plantilla debe traer los textos de la pantalla");
        let columna = |archivo: &str, n: usize| -> std::collections::HashSet<String> { csv::Reader::from_path(dir.join(archivo)).unwrap().records().flatten().map(|r| r[n].to_string()).collect() };
        let (con_fuente, en_revision) = (columna("cak_palabras.csv", 0), columna("cak_revision.csv", 1));
        for (codigo, minimo) in [("en", 0.99), ("cak", 0.0)] {
            let dic = leer(&format!("{codigo}.json"));
            let hechos = textos_traducidos(&dic);
            for (clave, valor) in &hechos {
                let valor = valor.as_str().unwrap();
                assert_eq!(piezas_de(clave), piezas_de(valor), "{codigo}: «{clave}» → «{valor}»");
                assert!(plantilla.contains(clave), "{codigo}: «{clave}» ya no es un texto de la pantalla (correr scripts/idiomas/comprobar.js --podar)");
                if codigo == "cak" {
                    assert!(en_revision.contains(valor), "kaqchikel fuera de la hoja de revisión (cak_revision.csv): «{clave}» → «{valor}»");
                    for palabra in palabras_de(valor) {
                        assert!(con_fuente.contains(&palabra), "kaqchikel: la palabra «{palabra}» de «{valor}» no está en cak_palabras.csv (sin fuente)");
                    }
                }
            }
            let cubre = hechos.len() as f64 / plantilla.len() as f64;
            assert!(cubre >= minimo, "{codigo} cubre {:.1} % de la pantalla y se anuncia como completo", cubre * 100.0);
        }
        let cak = leer("cak.json");
        assert!(cak["estado"] == "borrador" || cak["revisado_por"].as_str().is_some_and(|n| !n.trim().is_empty()), "el kaqchikel sigue siendo borrador hasta que lo revise una persona hablante y conste su nombre en «revisado_por»");
    }

    #[test]
    fn una_celda_protegida_no_se_puede_despejar_por_resta() {
        let celda = |sol: &str, m: &str, n: i64| (sol.to_string(), m.to_string(), n);
        let ocultas = |celdas: Vec<(String, String, i64)>, exacto| celdas_ocultas(celdas.into_iter(), exacto, 3);
        // Un solo municipio con conteo pequeño: se oculta también el siguiente más chico del departamento,
        // porque el total del departamento sí se publica.
        let o = ocultas(vec![celda("s", "0101", 1), celda("s", "0102", 4), celda("s", "0103", 50), celda("s", "0201", 9)], false);
        assert!(o.contains(&("s".into(), "0101".into())) && o.contains(&("s".into(), "0102".into())), "debe ocultar la pequeña y su complemento");
        assert!(!o.contains(&("s".into(), "0103".into())) && !o.contains(&("s".into(), "0201".into())), "no debe ocultar de más ni en otro departamento");
        // Dos pequeñas en el mismo departamento ya se cubren entre sí.
        let o = ocultas(vec![celda("s", "0101", 1), celda("s", "0102", 2), celda("s", "0103", 50)], false);
        assert_eq!(o.len(), 2);
        // Cada solicitud se protege por separado; los ceros no cuentan como celda.
        let o = ocultas(vec![celda("a", "0101", 2), celda("a", "0102", 0), celda("a", "0103", 7), celda("b", "0101", 30)], false);
        assert!(o.contains(&("a".into(), "0101".into())) && o.contains(&("a".into(), "0103".into())) && !o.contains(&("a".into(), "0102".into())) && !o.contains(&("b".into(), "0101".into())));
        // Quien es analista ve los conteos exactos; con umbral 0 o 1 no hay protección.
        assert!(ocultas(vec![celda("s", "0101", 1), celda("s", "0102", 4)], true).is_empty());
        assert!(celdas_ocultas(vec![celda("s", "0101", 1)].into_iter(), false, 1).is_empty());
    }

    #[test]
    fn la_misma_solicitud_da_siempre_la_misma_huella() {
        // PNC declara equivalencias (una tabla sin orden fijo): cada lectura la arma en otro orden.
        let ruta = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../datos/solicitudes/pnc-detenidos-v1.toml");
        let huellas: std::collections::HashSet<String> = (0..20).map(|_| huella_de_reglas(&Solicitud::cargar(&ruta).unwrap())).collect();
        assert_eq!(huellas.len(), 1, "la huella cambió entre lecturas de la misma solicitud");
        // Y cambia cuando cambia una regla.
        let mut otra = Solicitud::cargar(&ruta).unwrap();
        otra.columnas[0].alias.push("OTRO NOMBRE".into());
        assert!(!huellas.contains(&huella_de_reglas(&otra)));
    }
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
                    (SELECT count(*) FROM (SELECT 1 FROM pendientes WHERE NOT resuelto AND envio_id IN (SELECT id FROM envios_vigentes) GROUP BY columna, nivel, contexto, valor) p),
                    (SELECT count(*) FROM equivalencias) + (SELECT count(*) FROM equivalencias_valor),
                    (SELECT count(*) FROM conjuntos_ine), (SELECT count(DISTINCT municipio) FROM edificaciones)",
            &[],
        )
        .await
        .map_err(interno)?;
    Ok(Json(json!({
        "envios": f.get::<_, i64>(0), "filas": f.get::<_, i64>(1), "pendientes": f.get::<_, i64>(2),
        "equivalencias": f.get::<_, i64>(3), "conjuntos_ine": f.get::<_, i64>(4), "municipios_con_edificaciones": f.get::<_, i64>(5),
        "lugares_en_catalogo": e.catalogo.len(), "requiere_token": e.token.is_some(), "umbral_de_celda": umbral_de_celda(),
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

async fn api_envios(State(e): State<Compartido>, cabeceras: HeaderMap) -> Res<Json<Value>> {
    let quien = con_sesion(&e, &cabeceras).await?;
    if quien.rol == "operador" {
        return Err(Fallo(StatusCode::FORBIDDEN, "esta información es de uso interno del INE".into()));
    }
    // Una cuenta de institución ve solo los envíos de su institución.
    let solo_de: Option<String> = (quien.rol == "institucion").then(|| quien.institucion.clone());
    // Cada envío con el periodo de sus datos y si sigue vigente o fue reemplazado por uno posterior.
    let filas = e.bd.query(
        "SELECT e.id, e.solicitud, e.institucion, e.archivo, to_char(e.recibido, 'YYYY-MM-DD HH24:MI'), e.filas, e.pct_resuelto, e.problemas,
                (SELECT min(corte) FROM hechos h WHERE h.envio_id = e.id AND corte <> ''), (SELECT max(corte) FROM hechos h WHERE h.envio_id = e.id AND corte <> ''),
                (SELECT count(DISTINCT corte) FROM hechos h WHERE h.envio_id = e.id AND corte <> ''),
                e.id IN (SELECT id FROM envios_vigentes),
                (SELECT count(DISTINCT h.corte) FROM hechos h WHERE h.envio_id = e.id AND h.corte <> '' AND h.envio_id = (SELECT max(envio_id) FROM hechos x WHERE x.solicitud = h.solicitud AND x.corte = h.corte)),
                (SELECT max(x.envio_id) FROM hechos h JOIN hechos x ON x.solicitud = h.solicitud AND x.corte = h.corte AND x.envio_id > h.envio_id WHERE h.envio_id = e.id),
                (SELECT min(periodo) FROM hechos h WHERE h.envio_id = e.id AND periodo <> ''), (SELECT max(periodo) FROM hechos h WHERE h.envio_id = e.id AND periodo <> ''),
                (SELECT count(*) FROM (SELECT 1 FROM pendientes p WHERE p.envio_id = e.id AND NOT p.resuelto GROUP BY columna, nivel, contexto, valor) q)
         FROM envios e WHERE $1::text IS NULL OR e.institucion = $1 ORDER BY e.id DESC LIMIT 200", &[&solo_de]).await.map_err(interno)?;
    let viejos = envios_con_reglas_anteriores(&e).await?;
    Ok(Json(json!(filas.iter().map(|f| json!({
        "reglas_anteriores": viejos.contains(&f.get::<_, i64>(0)),
        "id": f.get::<_, i64>(0), "solicitud": f.get::<_, String>(1), "institucion": f.get::<_, String>(2), "archivo": f.get::<_, String>(3),
        "recibido": f.get::<_, String>(4), "filas": f.get::<_, i64>(5), "pct_resuelto": f.get::<_, f64>(6), "problemas": f.get::<_, i64>(7),
        "periodo_desde": f.get::<_, Option<String>>(8), "periodo_hasta": f.get::<_, Option<String>>(9), "periodos": f.get::<_, i64>(10),
        "vigente": f.get::<_, bool>(11), "periodos_vigentes": f.get::<_, i64>(12), "reemplazado_por": f.get::<_, Option<i64>>(13),
        "hechos_desde": f.get::<_, Option<String>>(14), "hechos_hasta": f.get::<_, Option<String>>(15), "pendientes": f.get::<_, i64>(16),
    })).collect::<Vec<_>>())))
}

/// Envíos cuya solicitud cambió después de analizarlos (o que se analizaron antes de que se
/// guardara la huella de reglas).
async fn envios_con_reglas_anteriores(e: &Compartido) -> Res<std::collections::HashSet<i64>> {
    let actuales: HashMap<String, String> = solicitudes(e).map_err(interno)?.iter().map(|s| (s.id.clone(), huella_de_reglas(s))).collect();
    Ok(e.bd.query("SELECT id, solicitud, reglas FROM envios", &[]).await.map_err(interno)?.iter()
        .filter(|f| actuales.get(&f.get::<_, String>(1)).is_some_and(|h| f.get::<_, Option<String>>(2).as_deref() != Some(h.as_str())))
        .map(|f| f.get(0)).collect())
}

/// Elimina un envío con sus pendientes y sus conteos (quedan las equivalencias aprendidas).
async fn api_envio_borrar(State(e): State<Compartido>, Ruta(id): Ruta<i64>, cabeceras: HeaderMap) -> Res<Json<Value>> {
    let quien = autorizar_en(&e, &cabeceras, "recibir").await?;
    let n = e.bd.execute("DELETE FROM envios WHERE id = $1", &[&id]).await.map_err(interno)?;
    if n == 0 {
        return Err(pedido("no existe ese envío"));
    }
    // También se borra su salida fila por fila del volumen privado.
    let _ = std::fs::remove_dir_all(e.dir_privado.join("envios").join(id.to_string()));
    bitacora(&e, &quien.nombre, "envio_borrado", &format!("envío n.º {id}")).await;
    Ok(Json(json!({ "borrado": true })))
}

/// Conteos por corte, periodo del hecho (AAAA-MM), municipio y sexo a partir de la salida estándar.
/// El corte sale de las columnas que declara la solicitud (fecha de registro, o año y mes); si no
/// declara ninguna, es el mismo mes del hecho.
fn agregar(rec: &Recepcion, corte: &[String]) -> BTreeMap<(String, String, String, String), i64> {
    let pos = |n: &str| rec.encabezados.iter().position(|e| e == n);
    let (fecha, muni, sexo) = (pos("fecha_hecho"), pos("municipio_codigo"), pos("sexo"));
    let cols: Vec<usize> = corte.iter().filter_map(|c| pos(c)).collect();
    let mut conteo = BTreeMap::new();
    for f in &rec.filas {
        let periodo = fecha.and_then(|i| f[i].get(..7)).unwrap_or("").to_string();
        let de_corte = match cols.as_slice() {
            [una] => f[*una].get(..7).filter(|t| t.len() == 7 && t.as_bytes()[4] == b'-').map(String::from),
            [anio, mes] => f[*anio].trim().parse::<u32>().ok().zip(f[*mes].trim().parse::<u32>().ok()).filter(|(a, m)| (1900..=2100).contains(a) && (1..=12).contains(m)).map(|(a, m)| format!("{a}-{m:02}")),
            _ => None,
        };
        let m = muni.map(|i| f[i].clone()).unwrap_or_default();
        let s = sexo.map(|i| f[i].clone()).unwrap_or_default();
        *conteo.entry((de_corte.unwrap_or_else(|| periodo.clone()), periodo, m, s)).or_insert(0) += 1;
    }
    conteo
}

async fn api_enviar(State(e): State<Compartido>, Ruta(id): Ruta<String>, cabeceras: HeaderMap, mut partes: Multipart) -> Res<Json<Value>> {
    let sol = solicitudes(&e).map_err(interno)?.into_iter().find(|s| s.id == id).ok_or_else(|| pedido(format!("no existe la solicitud {id}")))?;
    let quien = quien_envia(&e, &cabeceras, &sol).await?;
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
    // La guarda borra el temporal pase lo que pase: error de lectura, pánico o conexión cortada.
    struct Temporal(PathBuf);
    impl Drop for Temporal {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }
    let _guarda = Temporal(temporal.clone());
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
            "INSERT INTO envios (solicitud, institucion, archivo, filas, pct_resuelto, problemas, reporte, reglas) VALUES ($1,$2,$3,$4,$5,$6,$7,$8) RETURNING id",
            &[&sol.id, &sol.institucion, &nombre, &(rec.reporte.filas as i64), &rec.porcentaje_resuelto(), &problemas, &reporte, &huella_de_reglas(&sol)],
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
    for ((corte, periodo, municipio, sexo), n) in agregar(&rec, &sol.corte) {
        e.bd.execute("INSERT INTO hechos (envio_id, solicitud, periodo, municipio, sexo, n, corte) VALUES ($1,$2,$3,$4,$5,$6,$7)", &[&envio, &sol.id, &periodo, &municipio, &sexo, &n, &corte]).await.map_err(interno)?;
    }
    // El mismo archivo de la misma solicitud siempre reemplaza al anterior, aunque aquel se haya
    // guardado con otra regla de corte: sus conteos pasan al corte del nuevo y quedan reemplazados.
    e.bd.execute(
        "UPDATE hechos SET corte = (SELECT min(corte) FROM hechos WHERE envio_id = $1)
         WHERE envio_id IN (SELECT id FROM envios WHERE solicitud = $2 AND archivo = $3 AND id < $1) AND EXISTS (SELECT 1 FROM hechos WHERE envio_id = $1)",
        &[&envio, &sol.id, &nombre],
    )
    .await
    .map_err(interno)?;
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
    // En la bitácora va el nombre del archivo y el número de filas; nunca su contenido.
    bitacora(&e, &quien.nombre, "envio_recibido", &format!("envío n.º {envio} de {}: {nombre}", sol.id)).await;
    Ok(Json(json!({ "envio": envio, "solicitud": sol.id, "archivo": nombre, "porcentaje_resuelto": rec.porcentaje_resuelto(), "recepcion": rec })))
}

async fn api_pendientes(State(e): State<Compartido>, cabeceras: HeaderMap) -> Res<Json<Value>> {
    personal_del_ine(&e, &cabeceras).await?;
    // Un mismo valor puede venir en varios envíos: se muestra una vez, con el total.
    let filas = e
        .bd
        .query(
            "SELECT columna, nivel, contexto, valor, sum(veces)::bigint, max(estado), (array_agg(candidatos ORDER BY id DESC))[1], string_agg(DISTINCT institucion, ', '), array_agg(DISTINCT envio_id)
             FROM pendientes WHERE NOT resuelto AND envio_id IN (SELECT id FROM envios_vigentes) GROUP BY columna, nivel, contexto, valor ORDER BY 5 DESC LIMIT 500",
            &[],
        )
        .await
        .map_err(interno)?;
    // Envíos analizados con una versión anterior de su solicitud: sus pendientes pueden ser viejos.
    let viejos = envios_con_reglas_anteriores(&e).await?;
    Ok(Json(json!(filas.iter().map(|f| {
        let ctx: String = f.get(2);
        let pista = pista_de_codigo(&f.get::<_, String>(1), &f.get::<_, String>(3));
        let envios: Vec<i64> = f.get(8);
        json!({
            "pista": pista, "envios": envios, "reglas_anteriores": !envios.is_empty() && envios.iter().all(|i| viejos.contains(i)),
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
    // La equivalencia se guarda a nombre de quien inició sesión (columna `confirmado_por`), no del
    // nombre que venga en el pedido. Solo la clave de servicio puede indicar otro nombre.
    let quien = autorizar_en(&e, &cabeceras, "pendientes").await?;
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
        bitacora(&e, &c.usuario, "equivalencia_confirmada", &format!("producto del IPMC -> {}", c.codigo)).await;
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
        bitacora(&e, &c.usuario, "equivalencia_confirmada", &format!("valor de {} -> {valor} ({n} pendientes resueltos)", c.contexto)).await;
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
    bitacora(&e, &c.usuario, "equivalencia_confirmada", &format!("{nivel} -> {} ({n} pendientes resueltos)", lugar.codigo)).await;
    Ok(Json(json!({ "grabado": true, "codigo": lugar.codigo, "nombre": lugar.nombre, "pendientes_resueltos": n })))
}

async fn api_equivalencias(State(e): State<Compartido>, cabeceras: HeaderMap) -> Res<Json<Value>> {
    personal_del_ine(&e, &cabeceras).await?;
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

async fn api_hechos(State(e): State<Compartido>, cabeceras: HeaderMap) -> Res<Json<Value>> {
    let (exacto, umbral) = (ve_conteos_exactos(&e, &cabeceras).await, umbral_de_celda());
    // Una celda de 1 a (umbral − 1) no se publica; tampoco su complemento, que la delataría por resta.
    let chica = |n: Option<i64>| !exacto && n.is_some_and(|n| n > 0 && n < umbral);
    // Solo el envío más reciente de cada solicitud y corte (mes de registro), para no contar dos veces un reenvío.
    let filas = e
        .bd
        .query(
            "WITH ultimo AS (SELECT solicitud, corte, max(envio_id) AS envio_id FROM hechos GROUP BY 1, 2)
             SELECT h.solicitud, left(h.municipio, 2), sum(h.n)::bigint, sum(h.n) FILTER (WHERE h.sexo = 'hombre')::bigint, sum(h.n) FILTER (WHERE h.sexo = 'mujer')::bigint,
                    max(left(h.periodo, 4)), count(DISTINCT h.periodo) FILTER (WHERE h.periodo <> '')
             FROM hechos h JOIN ultimo u USING (solicitud, corte, envio_id) GROUP BY 1, 2 ORDER BY 1, 2",
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
        let (h, m): (Option<i64>, Option<i64>) = (f.get(3), f.get(4));
        let oculta_sexo = chica(h) || chica(m);
        if chica(Some(total)) {
            return json!({ "solicitud": f.get::<_, String>(0), "departamento": d, "nombre": e.catalogo.por_codigo(&d).map(|l| l.nombre.clone()).unwrap_or_else(|| "(sin lugar)".into()), "total": null, "protegido": true, "anio": anio, "meses": f.get::<_, i64>(6) });
        }
        json!({ "solicitud": f.get::<_, String>(0), "departamento": d, "nombre": e.catalogo.por_codigo(&d).map(|l| l.nombre.clone()).unwrap_or_else(|| "(sin lugar)".into()),
                "total": total, "hombres": if oculta_sexo { None } else { h }, "mujeres": if oculta_sexo { None } else { m }, "protegido": oculta_sexo,
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
                    (SELECT count(*) FROM (SELECT 1 FROM pendientes p JOIN envios x ON x.id = p.envio_id WHERE NOT p.resuelto AND x.id IN (SELECT id FROM envios_vigentes) AND x.solicitud = e.solicitud GROUP BY p.columna, p.nivel, p.contexto, p.valor) q)
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
    // Opcional: llevar la serie más allá de su último año (no oficial), con las dos curvas.
    let extender = q.get("extender").and_then(|a| a.parse::<i32>().ok()).filter(|a| *a > max).map(|a| a.min(max + 40));
    let serie_de = |codigo: &str| -> Vec<(i32, f64)> { (min..=max).filter_map(|a| Some((a, e.poblacion.total(codigo, a)? as f64))).collect() };
    let mut municipios: Vec<Value> = e
        .poblacion
        .municipios()
        .iter()
        .filter_map(|m| e.poblacion.cambio(m, desde, hasta))
        .map(|c| {
            let l = e.catalogo.por_codigo(&c.codigo);
            let mut v = serde_json::to_value(&c).unwrap_or(Value::Null);
            if let Some(x) = extender.and_then(|a| faro_territorio::crecimiento::extrapolar(&serie_de(&c.codigo), a)) {
                v["extrapolacion"] = json!(x);
            }
            v["municipio"] = json!(l.map(|l| l.nombre.clone()));
            v["departamento"] = json!(l.and_then(|l| e.catalogo.por_codigo(&l.padre)).map(|d| d.nombre.clone()));
            v
        })
        .collect();
    municipios.sort_by(|a, b| b["tasa_anual_pct"].as_f64().unwrap_or(0.0).total_cmp(&a["tasa_anual_pct"].as_f64().unwrap_or(0.0)));
    // El total del país al año extendido es la suma de los municipios, curva por curva.
    let suma = |k: &str| municipios.iter().filter_map(|m| m["extrapolacion"][k].as_f64().or_else(|| m["extrapolacion"]["exponencial"].as_f64())).sum::<f64>();
    let extendido = extender.map(|a| json!({ "anio": a, "horizonte": a - max, "horizonte_corto": faro_territorio::crecimiento::HORIZONTE_CORTO, "exponencial": suma("exponencial"), "logistica": suma("logistica"), "sugerida": suma("valor"),
        "con_techo": municipios.iter().filter(|m| m["extrapolacion"]["logistica"].is_number()).count() }));
    Ok(Json(json!({ "disponible": true, "desde": desde, "hasta": hasta, "anio_minimo": min, "anio_maximo": max, "pais": e.poblacion.cambio("", desde, hasta), "extendido": extendido, "municipios": municipios })))
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
            eprintln!("sincronización con el INE: {detalle}");
            e.bd.execute("INSERT INTO sincronizaciones (ok, detalle) VALUES (FALSE, $1)", &[&"no se pudo consultar el portal del INE (sin conexión o sin respuesta)"]).await.map_err(interno)?;
            return Err(Fallo(StatusCode::BAD_GATEWAY, "no se pudo consultar el portal del INE (sin conexión o sin respuesta)".into()));
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
    autorizar_en(&e, &cabeceras, "ine").await?;
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
    autorizar_en(&e, &cabeceras, "crecimiento").await?;
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
    autorizar_en(&e, &cabeceras, "operadores").await?;
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
    autorizar_en(&e, &cabeceras, "precios").await?;
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

/// Censo 2018 por lugar poblado (`faro censo-lugares`): código -> cifras, en el orden de las columnas del CSV.
fn leer_censo_lugar(dir: &std::path::Path) -> HashMap<String, Vec<i64>> {
    let Ok(mut lector) = csv::Reader::from_path(dir.join("censo2018_lugar_poblado.csv")) else { return HashMap::new() };
    lector.records().flatten().filter_map(|f| Some((f.get(0)?.to_string(), f.iter().skip(1).map(|c| c.parse().unwrap_or(0)).collect()))).collect()
}

/// Techo más frecuente en cada lugar poblado (Censo 2018, cuadro C2): código -> (material, % de las viviendas).
fn leer_techo_lugar(dir: &std::path::Path) -> HashMap<String, (String, f64)> {
    const MATERIALES: [(&str, &str); 6] = [("techo_concreto", "concreto"), ("techo_lamina", "lámina metálica"), ("techo_asbesto_cemento", "asbesto cemento"), ("techo_teja", "teja"), ("techo_paja_palma", "paja o palma"), ("techo_desecho", "material de desecho")];
    let Ok(mut lector) = csv::Reader::from_path(dir.join("censo2018_techos_lugar_poblado.csv")) else { return HashMap::new() };
    lector.deserialize::<HashMap<String, String>>().flatten().filter_map(|f| {
        let n = |k: &str| f.get(k).and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
        let total = n("viviendas_particulares");
        let (columna, nombre) = MATERIALES.iter().copied().max_by(|a, b| n(a.0).total_cmp(&n(b.0)))?;
        (total > 0.0 && n(columna) > 0.0).then(|| (f.get("codigo").cloned().unwrap_or_default(), (nombre.to_string(), 100.0 * n(columna) / total)))
    }).collect()
}

/// Indicadores del Censo 2018 por municipio (`faro censo-indicadores`): código -> cifra -> valor.
fn leer_censo_indicadores(dir: &std::path::Path) -> HashMap<String, HashMap<String, f64>> {
    let Ok(mut lector) = csv::Reader::from_path(dir.join("censo2018_indicadores_municipio.csv")) else { return HashMap::new() };
    lector.deserialize::<HashMap<String, String>>().flatten().filter_map(|f| Some((f.get("municipio")?.clone(), f.iter().filter_map(|(k, v)| Some((k.clone(), v.parse().ok()?))).collect()))).collect()
}

/// Prepara lo que se publica en `/capas`: municipios con sus indicadores, enlaces viales probables,
/// lugares sin conexión, lugares poblados y las tablas CSV de referencia. Si falta un archivo de
/// entrada, esa capa no se publica. Devuelve las capas y su catálogo (título, fuente, licencia).
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
        let censo_ind = leer_censo_indicadores(dir);
        // Kilómetros de vía por tipo y superficie (`faro vial`): municipio -> columna -> km.
        let vias: HashMap<String, HashMap<String, f64>> = csv::Reader::from_path(dir.join("red_vial_tipos_municipio.csv")).map(|mut l| l.deserialize::<HashMap<String, String>>().flatten()
            .filter_map(|f| Some((f.get("municipio")?.clone(), f.iter().filter_map(|(k, v)| Some((k.clone(), v.parse().ok()?))).collect()))).collect()).unwrap_or_default();
        // Uso del suelo construido en 2020 (GHSL): municipio -> (hectáreas construidas, hectáreas no residenciales).
        let uso: HashMap<String, (f64, f64)> = csv::Reader::from_path(dir.join("uso_construido_municipio.csv")).map(|mut l| l.deserialize::<HashMap<String, String>>().flatten().filter(|f| f.get("anio").is_some_and(|a| a == "2020"))
            .filter_map(|f| Some((f.get("municipio")?.clone(), (f.get("construido_ha")?.parse().ok()?, f.get("no_residencial_ha")?.parse().ok()?)))).collect()).unwrap_or_default();
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
            if let Some(v) = vias.get(&codigo) {
                let km = |k: &str| v.get(k).copied().unwrap_or(0.0);
                let uno = |x: f64| (x * 10.0).round() / 10.0;
                let con_dato = km("km_pavimentada") + km("km_sin_pavimentar");
                p["vias_km"] = json!(uno(km("km_vehiculos")));
                p["vias_principales_km"] = json!(uno(km("km_principal") + km("km_secundaria")));
                p["vias_terraceria_km"] = json!(uno(km("km_terraceria")));
                p["vias_pavimentadas_km"] = json!(uno(km("km_pavimentada")));
                p["vias_con_superficie_km"] = json!(uno(con_dato));
                // Dos porcentajes distintos: cuánta red tiene la superficie anotada, y de esa, cuánta está pavimentada.
                p["vias_superficie_anotada_pct"] = json!((km("km_vehiculos") > 0.0).then(|| uno(100.0 * con_dato / km("km_vehiculos"))));
                p["vias_pavimentadas_pct"] = json!((con_dato > 0.0).then(|| uno(100.0 * km("km_pavimentada") / con_dato)));
                p["vias_km_por_mil_hab"] = json!(p["poblacion_actual"].as_f64().filter(|h| *h > 0.0).map(|h| (km("km_vehiculos") / h * 100_000.0).round() / 100.0));
            }
            if let Some((total, nres)) = uso.get(&codigo) {
                p["no_residencial_ha_2020"] = json!(nres);
                p["residencial_ha_2020"] = json!(((total - nres).max(0.0) * 100.0).round() / 100.0);
                p["no_residencial_pct_2020"] = json!((*total > 0.0).then(|| (nres / total * 1000.0).round() / 10.0));
            }
            // Censo 2018: porcentajes y, para poder agregarlos por departamento sin promediar mal, sus denominadores.
            if let Some(c) = censo_ind.get(&codigo) {
                let v = |k: &str| c.get(k).copied().unwrap_or(0.0);
                let pct = |num: f64, den: f64| (den > 0.0).then(|| (num / den * 1000.0).round() / 10.0);
                p["censo_alfabetismo_pct"] = json!(pct(v("alfabetas_hombres") + v("alfabetas_mujeres"), v("poblacion_7_o_mas")));
                p["censo_internet_pct"] = json!(pct(v("usa_internet"), v("poblacion_7_o_mas")));
                p["censo_agua_en_vivienda_pct"] = json!(pct(v("agua_tuberia_en_vivienda"), v("hogares")));
                p["censo_drenaje_pct"] = json!(pct(v("sanitario_con_drenaje"), v("hogares")));
                p["censo_electricidad_pct"] = json!(pct(v("alumbrado_red_electrica"), v("hogares")));
                p["censo_lena_pct"] = json!(pct(v("cocina_con_lena"), v("hogares")));
                p["censo_techo_lamina_pct"] = json!(pct(v("techo_lamina"), v("viviendas_particulares")));
                p["censo_techo_concreto_pct"] = json!(pct(v("techo_concreto"), v("viviendas_particulares")));
                p["censo_piso_tierra_pct"] = json!(pct(v("piso_tierra"), v("viviendas_con_piso_declarado")));
                p["censo_maya_pct"] = json!(pct(v("pueblo_maya"), v("poblacion_censada")));
                p["censo_pea_pct"] = json!(pct(v("pea"), v("poblacion_15_o_mas")));
                p["censo_educ_media_pct"] = json!(pct(v("educ_basico") + v("educ_diversificado") + v("educ_licenciatura") + v("educ_posgrado"), v("poblacion_4_o_mas")));
                p["censo_basura_recolectada_pct"] = json!(pct(v("basura_servicio_municipal") + v("basura_servicio_privado"), v("hogares")));
                p["censo_vivienda_propia_pct"] = json!(pct(v("vivienda_propia"), v("hogares")));
                p["censo_propietaria_mujer_pct"] = json!(pct(v("propietaria_mujer"), v("vivienda_propia")));
                (p["censo_pob15"], p["censo_pob4"], p["censo_viviendas_propias"]) = (json!(v("poblacion_15_o_mas")), json!(v("poblacion_4_o_mas")), json!(v("vivienda_propia")));
                (p["censo_pob7"], p["censo_hogares"], p["censo_viviendas"], p["censo_viviendas_piso"], p["censo_poblacion"]) = (json!(v("poblacion_7_o_mas")), json!(v("hogares")), json!(v("viviendas_particulares")), json!(v("viviendas_con_piso_declarado")), json!(v("poblacion_censada")));
            }
            p["proveedores_construccion"] = json!(construccion.proveedores.iter().filter(|x| x.get("municipio_codigo").map(String::as_str) == Some(codigo.as_str())).count());
            p["lugares_poblados"] = json!(lugares_de.get(&codigo).copied().unwrap_or(0));
            let (suelto, sin) = sin_conexion.get(&codigo).copied().unwrap_or((0, 0));
            p["lugares_en_tramo_suelto"] = json!(suelto);
            p["lugares_sin_camino"] = json!(sin);
            p["lugares_dentro_de_su_limite_pct"] = json!(acuerdo.get(&codigo));
        }
        publicar("municipios.geojson", GEOJSON, g.to_string(), "Municipios con indicadores",
            "340 municipios con código INE: población 2018, actual y 2030 (proyecciones INE), crecimiento anual, lugares poblados y cuántos no están conectados a la red vial.",
            "Límites: FARO, a partir de CONRED / OCHA (COD-AB Guatemala): contornos simplificados, Sanarate y Sansare corregidos y dos lagos quitados. Población: INE, proyecciones 2015–2030. Lugares: INE, Censo 2018. Caminos: © colaboradores de OpenStreetMap. Superficie construida: Comisión Europea, JRC, GHSL GHS-BUILT-S R2023A.", "CC BY-IGO (límites) · ODC-BY (población) · CC BY (censo) · ODbL (caminos) · CC BY 4.0 (GHSL)");
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
        ("departamentos.geojson", GEOJSON, "Departamentos", "22 departamentos con código INE, para mapas a menor detalle.", "FARO, a partir de CONRED / OCHA (COD-AB Guatemala): contornos simplificados y unidos por departamento.", "CC BY-IGO"),
        ("red_vial_principal.geojson", GEOJSON, "Carreteras principales y secundarias, con tipo y superficie", "Trazo simplificado de las vías de mayor jerarquía (troncales, primarias, secundarias y terciarias) con su clase, su tipo original, su nombre y si la superficie anotada es pavimentada, sin pavimentar o no tiene dato.", "© colaboradores de OpenStreetMap (HOT/HDX), corte del 3-nov-2026.", "ODbL 1.0"),
        ("red_vial_tipos_municipio.csv", "text/csv; charset=utf-8", "Kilómetros de vía por tipo y superficie, por municipio", "Kilómetros de vías principales, secundarias, locales, de terracería y senderos, y de las vías para vehículos, cuántos son pavimentados, sin pavimentar o no tienen la superficie anotada. Con código INE.", "FARO, a partir de © colaboradores de OpenStreetMap (HOT/HDX), corte del 3-nov-2026.", "ODbL 1.0"),
        ("red_vial_contexto.geojson", GEOJSON, "Caminos mapeados alrededor de los enlaces probables", "Tramos de camino a 1.5 km de cada enlace propuesto; indica si pertenecen a la red principal o a un tramo suelto.", "© colaboradores de OpenStreetMap (HOT/HDX), corte del 3-nov-2026.", "ODbL 1.0"),
        ("censo2018_indicadores_municipio.csv", "text/csv; charset=utf-8", "Condiciones de vida por municipio, Censo 2018", "Alfabetismo, nivel educativo, actividad económica, uso de celular, computadora e internet, pueblo de pertenencia, agua, servicio sanitario, alumbrado, combustible para cocinar, eliminación de la basura, tenencia de la vivienda y materiales de pared, techo y piso: conteos con su denominador, por municipio.", "INE, XII Censo Nacional de Población y VII de Vivienda 2018 (cuadros A5, A9, A11, A12, A13, B1, B2, B3, B4, B6, C2 y C3).", "CC BY"),
        ("censo2018_techos_lugar_poblado.csv", "text/csv; charset=utf-8", "Material del techo por lugar poblado, Censo 2018", "Viviendas particulares por material predominante del techo (concreto, lámina, asbesto cemento, teja, paja o palma, desecho) en cada lugar poblado.", "INE, XII Censo Nacional de Población y VII de Vivienda 2018 (cuadro C2).", "CC BY"),
        ("uso_construido_municipio.csv", "text/csv; charset=utf-8", "Superficie construida residencial y no residencial por municipio, 2000 y 2020", "Hectáreas construidas, no residenciales y residenciales, y porcentaje no residencial, con código INE del municipio.", "FARO, a partir de Comisión Europea, JRC: GHSL GHS-BUILT-S R2023A (capa total y componente NRES).", "CC BY 4.0"),
        ("uso_construido_bloques.csv", "text/csv; charset=utf-8", "Uso del suelo construido en bloques de 1 km, 2020", "Hectáreas construidas y hectáreas no residenciales por bloque de 1 km, con el municipio al que pertenece.", "FARO, a partir de Comisión Europea, JRC: GHSL GHS-BUILT-S R2023A (capa total y componente NRES).", "CC BY 4.0"),
        ("superficie_construida_bloques.csv", "text/csv; charset=utf-8", "Superficie construida en bloques de 1 km, 1975–2020", "Hectáreas construidas por bloque de 1 km y época, con el municipio al que pertenece: la huella edificada del país.", "FARO, a partir de Comisión Europea, JRC: GHSL GHS-BUILT-S R2023A.", "CC BY 4.0"),
    ] {
        if let Ok(texto) = std::fs::read_to_string(dir.join(archivo)) {
            publicar(archivo, tipo, texto, titulo, detalle, fuente, licencia);
        }
    }
    let censo = leer_censo_lugar(dir);
    let dato = |codigo: &str, i: usize| censo.get(codigo).and_then(|c| c.get(i)).copied();
    let puntos: Vec<Value> = catalogo.de_nivel(Nivel::LugarPoblado, None).filter(|l| l.longitud.is_some()).map(|l| json!({ "type": "Feature", "properties": { "codigo": l.codigo, "nombre": l.nombre, "municipio": l.padre,
        "poblacion_2018": dato(&l.codigo, 0), "hogares_2018": dato(&l.codigo, 8), "viviendas_2018": dato(&l.codigo, 9) }, "geometry": { "type": "Point", "coordinates": [l.longitud, l.latitud] } })).collect();
    if let Ok(texto) = std::fs::read_to_string(dir.join("censo2018_lugar_poblado.csv")) {
        publicar("censo2018_lugar_poblado.csv", "text/csv; charset=utf-8", texto, "Censo 2018 por lugar poblado", "Población por sexo y grandes grupos de edad, área urbana y rural, hogares y viviendas de 20,036 lugares poblados, con código INE.", "INE, XII Censo Nacional de Población y VII de Vivienda 2018, cuadros A1, B8 y C1 por lugar poblado.", "CC BY");
    }
    if let Ok(texto) = std::fs::read_to_string(dir.join("poblacion_municipio_edades.csv")) {
        publicar("poblacion_municipio_edades.csv", "text/csv; charset=utf-8", texto, "Población por municipio, sexo y grupo de edad (2018, 2026, 2030)", "Hombres y mujeres por grupo quinquenal de edad en cada municipio, con el código INE ya corregido.", "INE, Estimaciones y proyecciones de población 2015–2030.", "ODC-BY");
    }
    if !puntos.is_empty() {
        publicar("lugares_poblados.geojson", GEOJSON, json!({ "type": "FeatureCollection", "features": puntos }).to_string(), "Lugares poblados del Censo 2018", "19,723 lugares poblados con código INE, centroide y población, hogares y viviendas del censo.", "INE, XII Censo Nacional de Población y VII de Vivienda 2018.", "CC BY");
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

/// Protección estadística para quien no es analista. De una lista de celdas (solicitud, municipio,
/// conteo) dice cuáles no se publican: las pequeñas (de 1 a umbral − 1) y, si en un departamento
/// queda una sola oculta, también la siguiente más pequeña, para que no se obtenga por resta contra
/// el total del departamento, que sí se publica. Toda respuesta que entregue conteos por municipio
/// debe usar esta misma lista: si una ruta ocultara menos que otra, la resta volvería a ser posible.
fn celdas_ocultas(celdas: impl Iterator<Item = (String, String, i64)>, exacto: bool, umbral: i64) -> std::collections::HashSet<(String, String)> {
    let mut ocultas = std::collections::HashSet::new();
    if exacto || umbral <= 1 {
        return ocultas;
    }
    let mut por_depto: BTreeMap<(String, String), Vec<(i64, String)>> = BTreeMap::new();
    for (sol, municipio, total) in celdas {
        por_depto.entry((sol, municipio.chars().take(2).collect())).or_default().push((total, municipio));
    }
    for ((sol, _), mut lista) in por_depto {
        lista.sort();
        let chicas = lista.iter().filter(|(n, _)| *n > 0 && *n < umbral).count();
        let ocultar = if chicas == 1 { 2 } else { chicas };
        ocultas.extend(lista.into_iter().filter(|(n, _)| *n > 0).take(ocultar).map(|(_, m)| (sol.clone(), m)));
    }
    ocultas
}

/// Conteos vigentes por solicitud y municipio (el envío más reciente de cada corte).
const CONTEOS_POR_MUNICIPIO: &str = "WITH ultimo AS (SELECT solicitud, corte, max(envio_id) AS envio_id FROM hechos GROUP BY 1, 2)
             SELECT h.solicitud, h.municipio, sum(h.n)::bigint, max(left(h.periodo, 4)) FROM hechos h JOIN ultimo u USING (solicitud, corte, envio_id) WHERE h.municipio <> '' GROUP BY 1, 2";

/// Registros recibidos por municipio y solicitud (solo conteos), con tasa por 100 mil habitantes.
async fn api_registros_municipio(State(e): State<Compartido>, cabeceras: HeaderMap) -> Res<Json<Value>> {
    let (exacto, umbral) = (ve_conteos_exactos(&e, &cabeceras).await, umbral_de_celda());
    let filas = e
        .bd
        .query(
            "WITH ultimo AS (SELECT solicitud, corte, max(envio_id) AS envio_id FROM hechos GROUP BY 1, 2)
             SELECT h.solicitud, h.municipio, sum(h.n)::bigint, max(left(h.periodo, 4)) FROM hechos h JOIN ultimo u USING (solicitud, corte, envio_id) WHERE h.municipio <> '' GROUP BY 1, 2",
            &[],
        )
        .await
        .map_err(interno)?;
    let ocultas = celdas_ocultas(filas.iter().map(|f| (f.get(0), f.get(1), f.get(2))), exacto, umbral);
    let mut salida: BTreeMap<String, BTreeMap<String, Value>> = BTreeMap::new();
    for f in &filas {
        let (sol, municipio, total): (String, String, i64) = (f.get(0), f.get(1), f.get(2));
        let poblacion = f.get::<_, Option<String>>(3).and_then(|a| a.parse::<i32>().ok()).and_then(|a| e.poblacion.total(&municipio, a));
        let celda = if ocultas.contains(&(sol.clone(), municipio.clone())) { json!({ "total": null, "por_100_mil": null, "protegido": true }) } else { json!({ "total": total, "por_100_mil": poblacion.filter(|p| *p > 0).map(|p| (total as f64 * 1_000_000.0 / p as f64).round() / 10.0) }) };
        salida.entry(sol).or_default().insert(municipio, celda);
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
                    (SELECT count(*) FROM (SELECT 1 FROM pendientes WHERE NOT resuelto AND envio_id IN (SELECT id FROM envios_vigentes) GROUP BY columna, nivel, contexto, valor) p),
                    (SELECT count(*) FROM equivalencias) + (SELECT count(*) FROM equivalencias_valor), (SELECT coalesce(sum(n),0)::bigint FROM hechos),
                    (SELECT count(*) FROM conjuntos_ine), (SELECT count(DISTINCT institucion) FROM envios),
                    (SELECT count(*) FROM rutas_campo), (SELECT count(*) FROM paradas_campo), (SELECT count(*) FROM visitas_campo), (SELECT count(*) FROM visitas_campo WHERE coherencia = 'revisar')",
            &[],
        )
        .await
        .map_err(interno)?;
    Ok(Json(json!({
        "solicitudes": solicitudes(&e).map_err(interno)?.len(), "envios": f.get::<_, i64>(0), "filas": f.get::<_, i64>(1), "problemas": f.get::<_, i64>(2), "pendientes": f.get::<_, i64>(3),
        "equivalencias": f.get::<_, i64>(4), "registros_agregados": f.get::<_, i64>(5), "conjuntos_ine": f.get::<_, i64>(6), "instituciones": f.get::<_, i64>(7),
        "lugares_en_catalogo": e.catalogo.len(), "capas": e.catalogo_capas.len(), "fuentes": e.fuentes["fuentes"].as_array().map_or(0, Vec::len),
        "municipios_con_poblacion": e.poblacion.municipios().len(), "articulos_ipmc": e.construccion.articulos.len(), "precios_referencia_ine": e.referencias.len(),
        "rutas_de_campo": f.get::<_, i64>(8), "paradas_de_campo": f.get::<_, i64>(9), "visitas_de_campo": f.get::<_, i64>(10), "visitas_por_revisar": f.get::<_, i64>(11),
    })))
}

/// Repite, desde el portal del INE, el proceso que produjo la tabla de población: consulta el
/// conjunto, descarga el libro y su diccionario, los procesa y compara el resultado con la tabla
/// que usa este servidor. Devuelve cada paso con sus cifras.
async fn api_reproducir_poblacion(State(e): State<Compartido>, cabeceras: HeaderMap) -> Res<Json<Value>> {
    use faro_territorio::poblacion::{procesar, Paso};
    autorizar_en(&e, &cabeceras, "crecimiento").await?;
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

/// Carga el levantamiento de precios publicados que viene con el sistema
/// (`datos/referencia/precios_publicados.csv`), con la fecha en que se consultó cada precio.
/// Si la base ya tiene tantas filas como el CSV, no hace nada; si no, borra lo cargado y lo carga de nuevo.
async fn sembrar_precios(bd: &Client, ruta: &std::path::Path) -> Result<usize, String> {
    if !ruta.exists() {
        return Ok(0);
    }
    let filas: Vec<HashMap<String, String>> = csv::Reader::from_path(ruta).map_err(|e| e.to_string())?.deserialize().collect::<Result<_, _>>().map_err(|e| e.to_string())?;
    let filas: Vec<&HashMap<String, String>> = filas.iter().filter(|f| f.get("precio_q").is_some_and(|p| p.parse::<f64>().is_ok_and(|x| x > 0.0)) && f.get("fecha_consulta").is_some_and(|d| d.len() == 10)).collect();
    // El cambio se detecta solo por el número de filas: un CSV corregido con las mismas filas no se recarga.
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
            "INSERT INTO precios_observados (tarea, rubro, proveedor, producto, marca, precio, url, termino, ean, precio_lista, observado, imagen)
             SELECT $1, r, t, p, m, x, u, c, e, l, d::date, i FROM unnest($2::text[], $3::text[], $4::text[], $5::text[], $6::float8[], $7::text[], $8::text[], $9::text[], $10::float8[], $11::text[], $12::text[]) AS f(r, t, p, m, x, u, c, e, l, d, i)",
            &[&LEVANTAMIENTO, &col("rubro"), &col("tienda"), &col("producto"), &col("marca"), &precios, &col("url"), &col("categoria_tienda"), &col("codigo_barras"), &listas, &col("fecha_consulta"), &col("imagen")],
        )
        .await
        .map_err(|e| e.to_string())?;
    }
    Ok(filas.len())
}

/// Tareas que se están ejecutando en este momento (para no lanzar dos veces la misma).
/// Marca de las observaciones con que se calculó `precios_ultimos` (mayor id y cantidad) y cuándo.
static MARCA_ULTIMOS: std::sync::Mutex<Option<((i64, i64), std::time::Instant)>> = std::sync::Mutex::new(None);

/// Deja al día la vista `precios_ultimos` si las observaciones cambiaron desde el último refresco.
/// Mientras un muestreo está agregando filas se refresca a lo sumo cada medio minuto; si se borraron
/// filas (recarga del levantamiento) o es la primera vez, de inmediato.
async fn asegurar_ultimos(bd: &tokio_postgres::Client) -> Result<(), tokio_postgres::Error> {
    let f = bd.query_one("SELECT coalesce(max(id), 0), count(*) FROM precios_observados", &[]).await?;
    let ahora: (i64, i64) = (f.get(0), f.get(1));
    let anterior = *MARCA_ULTIMOS.lock().unwrap_or_else(|e| e.into_inner());
    let hace_falta = match anterior {
        None => true,
        Some((marca, _)) if marca == ahora => false,
        Some((marca, cuando)) => ahora.1 < marca.1 || cuando.elapsed().as_secs() >= 30,
    };
    if hace_falta {
        bd.batch_execute("REFRESH MATERIALIZED VIEW precios_ultimos").await?;
        *MARCA_ULTIMOS.lock().unwrap_or_else(|e| e.into_inner()) = Some((ahora, std::time::Instant::now()));
    }
    Ok(())
}

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
            "INSERT INTO precios_observados (tarea, rubro, proveedor, producto, marca, precio, unidad, url, termino, ean, precio_lista, imagen)
             SELECT $1, $2, $3, * FROM unnest($4::text[], $5::text[], $6::float8[], $7::text[], $8::text[], $9::text[], $10::text[], $11::float8[], $12::text[])",
            &[&tarea.id, &tarea.rubro, &tarea.proveedor, &col(|o| &o.producto), &col(|o| &o.marca), &precios, &col(|o| &o.unidad), &col(|o| &o.url), &col(|o| &o.termino), &col(|o| &o.ean), &listas, &col(|o| &o.imagen)],
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

/// Lo que el diccionario de estadísticas vitales del INE define para estas mismas columnas
/// («Extranjero», «Ignorado»). No se sugieren países ni edades: por decisión del equipo, una
/// columna de municipio no recibe pistas de otros catálogos.
fn pista_de_codigo(nivel: &str, valor: &str) -> Option<String> {
    let n: u32 = valor.trim().parse().ok()?;
    let mut pistas: Vec<String> = Vec::new();
    match (nivel, n) {
        ("departamento", 23) | ("municipio", 2300) => pistas.push("en el diccionario del INE este código es «Extranjero»".into()),
        ("departamento", 99) | ("municipio", 9999) => pistas.push("en el diccionario del INE este código es «Ignorado»".into()),
        (_, 0) => pistas.push("el cero no figura como lugar en el diccionario del INE".into()),
        _ => {}
    }
    (!pistas.is_empty()).then(|| pistas.join("; "))
}

#[derive(Deserialize)]
struct ConfigMuestreo {
    activa: Option<bool>,
    cada_horas: Option<i32>,
}

async fn api_muestreo_configurar(State(e): State<Compartido>, Ruta(id): Ruta<String>, cabeceras: HeaderMap, Json(c): Json<ConfigMuestreo>) -> Res<Json<Value>> {
    autorizar_en(&e, &cabeceras, "muestreos").await?;
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
    autorizar_en(&e, &cabeceras, "muestreos").await?;
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
    let _turno = turno_pesado()?;
    asegurar_ultimos(&e.bd).await.map_err(interno)?;
    // La pantalla pide por partes: primero los precios (rápido) y después cada vista pesada. Sin `parte` va todo.
    let parte = q.get("parte").cloned().unwrap_or_default();
    let quiere = |p: &str| parte.is_empty() || parte == p;
    const ULTIMOS: &str = "SELECT rubro, proveedor, producto, marca, precio, precio_lista, ean, termino, url, observado FROM precios_ultimos p WHERE NOT EXISTS (SELECT 1 FROM decisiones_precio d WHERE d.decision = 'error' AND p.ean <> '' AND d.rubro = p.rubro AND d.ean = p.ean AND d.proveedor = p.proveedor)";
    const PLANO: &str = "translate(lower(producto), 'áéíóúüñ', 'aeiouun')";
    let rubros = e.bd.query(&format!("SELECT rubro, count(*), count(DISTINCT proveedor), count(*) FILTER (WHERE ean <> ''), to_char(max(observado), 'YYYY-MM-DD'), string_agg(DISTINCT proveedor, ', ') FROM ({ULTIMOS}) u GROUP BY 1 ORDER BY 2 DESC"), &[]).await.map_err(interno)?;
    let rubros: Vec<Value> = rubros.iter().map(|f| json!({ "rubro": f.get::<_, String>(0), "productos": f.get::<_, i64>(1), "proveedores": f.get::<_, i64>(2), "con_codigo": f.get::<_, i64>(3), "ultima": f.get::<_, Option<String>>(4), "tiendas": f.get::<_, Option<String>>(5) })).collect();
    if parte == "rubros" {
        return Ok(Json(json!({ "rubros": rubros })));
    }
    let rubro = q.get("rubro").cloned().filter(|r| !r.is_empty()).or_else(|| rubros.first().and_then(|r| r["rubro"].as_str().map(String::from))).unwrap_or_default();
    let proveedor = q.get("proveedor").cloned().unwrap_or_default();
    // Cada palabra buscada debe aparecer en el nombre del producto (sin tildes ni mayúsculas).
    let palabras: Vec<String> = faro_homologa::normalizar::basico(q.get("q").map(String::as_str).unwrap_or("")).split(' ').filter(|p| !p.is_empty()).map(|p| format!("%{}%", p.to_lowercase())).collect();
    let umbral: f64 = q.get("umbral").and_then(|u| u.parse().ok()).filter(|u: &f64| *u > 0.0).unwrap_or(15.0);
    let filtro = format!("rubro = $1 AND ($2 = '' OR proveedor = $2) AND {PLANO} LIKE ALL($3::text[])");

    let resumen = e.bd.query_one(&format!("SELECT count(*), min(precio), percentile_cont(0.5) WITHIN GROUP (ORDER BY precio), max(precio), avg(precio), mode() WITHIN GROUP (ORDER BY precio) FROM ({ULTIMOS}) u WHERE {filtro}"), &[&rubro, &proveedor, &palabras]).await.map_err(interno)?;
    // Tiendas informantes del rubro: cuánto publica cada una y desde cuándo se le sigue.
    let informantes = if !quiere("informantes") { Vec::new() } else { e.bd.query(
        "SELECT proveedor, count(DISTINCT url), count(DISTINCT url) FILTER (WHERE ean <> ''), count(DISTINCT observado::date), to_char(min(observado), 'YYYY-MM-DD'), to_char(max(observado), 'YYYY-MM-DD'), count(DISTINCT termino), min(split_part(url, '/', 3))
         FROM precios_observados WHERE rubro = $1 GROUP BY 1 ORDER BY 2 DESC", &[&rubro]).await.map_err(interno)? };
    let informantes: Vec<Value> = informantes.iter().map(|f| json!({ "proveedor": f.get::<_, String>(0), "productos": f.get::<_, i64>(1), "con_codigo": f.get::<_, i64>(2), "dias": f.get::<_, i64>(3), "desde": f.get::<_, Option<String>>(4), "hasta": f.get::<_, Option<String>>(5), "categorias": f.get::<_, i64>(6), "sitio": f.get::<_, Option<String>>(7) })).collect();
    let filas = e.bd.query(&format!("SELECT proveedor, producto, marca, precio, precio_lista, ean, termino, url, to_char(observado, 'YYYY-MM-DD') FROM ({ULTIMOS}) u WHERE {filtro} ORDER BY precio LIMIT 200"), &[&rubro, &proveedor, &palabras]).await.map_err(interno)?;
    let filas: Vec<Value> = filas.iter().map(|f| json!({ "proveedor": f.get::<_, String>(0), "producto": f.get::<_, String>(1), "marca": f.get::<_, String>(2), "precio": f.get::<_, f64>(3), "precio_lista": f.get::<_, Option<f64>>(4),
        "ean": f.get::<_, String>(5), "categoria": f.get::<_, String>(6), "url": f.get::<_, String>(7), "fecha": f.get::<_, String>(8) })).collect();

    // Mismo código de barras en dos o más tiendas: es el mismo producto sin que nadie lo interprete.
    let iguales = if !quiere("codigo") { Vec::new() } else { e.bd.query(&format!(
        "SELECT ean, min(producto), count(DISTINCT proveedor), min(precio), max(precio),
                json_agg(json_build_object('proveedor', proveedor, 'precio', precio, 'url', url, 'producto', producto) ORDER BY precio)::text, count(*) OVER ()
         FROM ({ULTIMOS}) u WHERE {filtro} AND ean <> '' GROUP BY ean HAVING count(DISTINCT proveedor) >= 2 ORDER BY max(precio) / min(precio) DESC, ean LIMIT 100"), &[&rubro, &proveedor, &palabras]).await.map_err(interno)? };
    let total_iguales = iguales.first().map_or(0, |f| f.get::<_, i64>(6));
    let iguales: Vec<Value> = iguales.iter().map(|f| { let (a, b) = (f.get::<_, f64>(3), f.get::<_, f64>(4));
        json!({ "ean": f.get::<_, String>(0), "producto": f.get::<_, String>(1), "tiendas": f.get::<_, i64>(2), "minimo": a, "maximo": b, "diferencia_pct": (b / a - 1.0) * 100.0, "ofertas": serde_json::from_str::<Value>(&f.get::<_, String>(5)).unwrap_or(Value::Null) }) }).collect();

    // Diferencias de 50 % o más entre tiendas para un mismo código: se explican con reglas o se piden confirmar.
    // Aquí sí entran los precios marcados como error, para que la decisión quede a la vista.
    let grupos = if !quiere("diferencias") { Vec::new() } else { e.bd.query(
        "SELECT ean, min(producto), json_agg(json_build_object('proveedor', proveedor, 'precio', precio, 'precio_lista', precio_lista, 'url', url) ORDER BY precio)::text
         FROM (SELECT DISTINCT ON (proveedor, ean) proveedor, ean, producto, precio, precio_lista, url FROM precios_observados WHERE rubro = $1 AND ean <> '' ORDER BY proveedor, ean, observado DESC) u
         GROUP BY ean HAVING count(*) >= 2 AND max(precio) >= min(precio) * 1.5", &[&rubro]).await.map_err(interno)? };
    let decisiones: HashMap<(String, String), (String, String)> = e.bd.query("SELECT ean, proveedor, decision, usuario FROM decisiones_precio WHERE rubro = $1", &[&rubro]).await.map_err(interno)?
        .iter().map(|f| ((f.get(0), f.get(1)), (f.get(2), f.get(3)))).collect();
    let mut conteo: BTreeMap<&str, usize> = BTreeMap::new();
    let mut diferencias: Vec<Value> = grupos.iter().filter_map(|f| {
        let (ean, producto): (String, String) = (f.get(0), f.get(1));
        let ofertas: Vec<Value> = serde_json::from_str(&f.get::<_, String>(2)).ok()?;
        let precios: Vec<(String, f64, Option<f64>)> = ofertas.iter().filter_map(|o| Some((o["proveedor"].as_str()?.to_string(), o["precio"].as_f64()?, o["precio_lista"].as_f64()))).collect();
        let d = faro_territorio::precios::explicar_diferencia(&precios)?;
        let ofertas: Vec<Value> = ofertas.into_iter().map(|mut o| { let k = (ean.clone(), o["proveedor"].as_str().unwrap_or("").to_string()); if let Some((dec, quien)) = decisiones.get(&k) { o["decision"] = json!(dec); o["decidio"] = json!(quien); } o }).collect();
        let decididas = ofertas.iter().filter(|o| o.get("decision").is_some()).count();
        // Resuelto: una oferta no necesita decisión; lo demás, cuando alguien marcó un error o confirmó todas las tiendas.
        let resuelto = d.clase == "oferta" || ofertas.iter().any(|o| o["decision"] == "error") || decididas == ofertas.len();
        *conteo.entry(if d.clase != "oferta" && resuelto { "decididos" } else { d.clase }).or_default() += 1;
        Some(json!({ "ean": ean, "producto": producto, "clase": d.clase, "tienda": d.tienda, "explicacion": d.explicacion, "razon": d.razon, "razon_regular": d.razon_regular, "resuelto": resuelto, "ofertas": ofertas }))
    }).filter(|d| palabras.is_empty() || { let n = faro_homologa::normalizar::basico(d["producto"].as_str().unwrap_or("")); palabras.iter().all(|p| n.contains(p.trim_matches('%'))) }).collect();
    // Primero lo que espera una decisión (atípicos y luego sin regla), de mayor a menor diferencia.
    let orden = |d: &Value| (d["resuelto"].as_bool().unwrap_or(false), match d["clase"].as_str() { Some("atipico") => 0, Some("por_confirmar") => 1, _ => 2 }, -(d["razon"].as_f64().unwrap_or(0.0) * 1000.0) as i64);
    diferencias.sort_by_key(orden);
    let total_diferencias = diferencias.len();
    diferencias.truncate(300);

    // Cambio de precio del mismo producto en la misma tienda entre sus dos últimos días de muestreo.
    let cambios = if !quiere("cambios") { Vec::new() } else { e.bd.query(&format!(
        "WITH dias AS (SELECT DISTINCT ON (proveedor, url, observado::date) rubro, proveedor, url, producto, precio, observado::date AS dia FROM precios_observados WHERE rubro = $1 ORDER BY proveedor, url, observado::date, observado DESC),
              orden AS (SELECT *, row_number() OVER w AS n, lead(precio) OVER w AS antes, lead(dia) OVER w AS dia_antes FROM dias WINDOW w AS (PARTITION BY proveedor, url ORDER BY dia DESC))
         SELECT proveedor, producto, antes, precio, to_char(dia_antes, 'YYYY-MM-DD'), to_char(dia, 'YYYY-MM-DD'), url, count(*) OVER (), count(*) FILTER (WHERE abs(precio / antes - 1) * 100 >= $4) OVER ()
         FROM orden WHERE n = 1 AND antes > 0 AND {filtro} ORDER BY abs(precio / antes - 1) DESC LIMIT 100"), &[&rubro, &proveedor, &palabras, &umbral]).await.map_err(interno)? };
    let (comparados, atipicos) = cambios.first().map_or((0, 0), |f| (f.get::<_, i64>(7), f.get::<_, i64>(8)));
    let cambios: Vec<Value> = cambios.iter().map(|f| { let (antes, ahora) = (f.get::<_, f64>(2), f.get::<_, f64>(3)); let v = (ahora / antes - 1.0) * 100.0;
        json!({ "proveedor": f.get::<_, String>(0), "producto": f.get::<_, String>(1), "antes": antes, "ahora": ahora, "fecha_antes": f.get::<_, String>(4), "fecha_ahora": f.get::<_, String>(5), "url": f.get::<_, String>(6), "variacion_pct": v, "atipico": v.abs() >= umbral }) })
        .filter(|c| c["variacion_pct"].as_f64().is_some_and(|v| v != 0.0)).collect();

    Ok(Json(json!({ "rubros": rubros, "rubro": rubro, "total": resumen.get::<_, i64>(0), "minimo": resumen.get::<_, Option<f64>>(1), "mediana": resumen.get::<_, Option<f64>>(2), "maximo": resumen.get::<_, Option<f64>>(3), "media": resumen.get::<_, Option<f64>>(4), "moda": resumen.get::<_, Option<f64>>(5), "informantes": informantes, "diferencias": diferencias, "total_diferencias": total_diferencias, "conteo_diferencias": conteo,
        "filas": filas, "mismo_codigo": iguales, "total_mismo_codigo": total_iguales, "cambios": cambios, "comparados": comparados, "atipicos": atipicos, "umbral_pct": umbral })))
}

/// Informe de precio de referencia a partir de los precios publicados que coinciden con una
/// búsqueda: mismas reglas que la sección de cotizaciones (media, mediana, moda; mediana si hay
/// extremos). Todas las fuentes son secundarias (catálogos en línea) y el informe lo dice.
async fn api_informe_precio(State(e): State<Compartido>, cabeceras: HeaderMap, Query(q): Query<HashMap<String, String>>) -> Res<Json<Value>> {
    let _turno = turno_pesado()?;
    asegurar_ultimos(&e.bd).await.map_err(interno)?;
    // Bajar fotos nuevas es trabajo del servidor hacia afuera: solo con sesión. Sin ella se usan las ya calculadas.
    let puede_bajar = sesion_de(&e, &cabeceras).await?.is_some();
    let rubro = q.get("rubro").cloned().unwrap_or_default();
    let proveedor = q.get("proveedor").cloned().unwrap_or_default();
    let texto = q.get("q").map(|t| t.trim().to_string()).unwrap_or_default();
    let palabras: Vec<String> = faro_homologa::normalizar::basico(&texto).split(' ').filter(|p| !p.is_empty()).map(|p| format!("%{}%", p.to_lowercase())).collect();
    if palabras.is_empty() {
        return Err(pedido("escriba el producto: el informe se hace sobre una búsqueda concreta"));
    }
    // Productos que la persona apartó por no ser comparables (parámetro `sin`: direcciones separadas por «|»).
    let excluidos: Vec<String> = q.get("sin").map(|t| t.split('|').filter(|u| !u.is_empty()).map(String::from).collect()).unwrap_or_default();
    let filas = e.bd.query(
        "SELECT proveedor, producto, marca, precio, ean, url, to_char(observado, 'YYYY-MM-DD'), imagen FROM
           (SELECT * FROM precios_ultimos p WHERE NOT EXISTS (SELECT 1 FROM decisiones_precio d WHERE d.decision = 'error' AND p.ean <> '' AND d.rubro = p.rubro AND d.ean = p.ean AND d.proveedor = p.proveedor)) u
         WHERE ($1 = '' OR rubro = $1) AND ($2 = '' OR proveedor = $2) AND translate(lower(producto), 'áéíóúüñ', 'aeiouun') LIKE ALL($3::text[]) AND NOT (url = ANY($4::text[])) ORDER BY precio LIMIT 300",
        &[&rubro, &proveedor, &palabras, &excluidos]).await.map_err(interno)?;
    // Huella visual de cada foto (se calcula una vez y queda guardada).
    let imagenes: Vec<String> = filas.iter().map(|f| f.get::<_, String>(7)).filter(|u| !u.is_empty()).collect();
    let huellas = huellas_de(&e, &imagenes, puede_bajar).await?;
    let umbrales = faro_fuentes::foto::Umbrales::default();
    let se_parecen = |a: &str, b: &str| huellas.get(a).zip(huellas.get(b)).is_some_and(|(x, y)| faro_fuentes::foto::se_ven_iguales(x, y, &umbrales));
    // Si la persona eligió un producto de referencia, se dejan solo los que se ven como él o
    // comparten su código de barras.
    let referencia = q.get("como").filter(|u| !u.is_empty()).and_then(|u| filas.iter().find(|f| &f.get::<_, String>(5) == u));
    let total_antes = filas.len();
    let conservadas: Vec<&tokio_postgres::Row> = match referencia {
        None => filas.iter().collect(),
        Some(r) => {
            let (ean_r, img_r): (String, String) = (r.get(4), r.get(7));
            filas.iter().filter(|f| f.get::<_, String>(5) == r.get::<_, String>(5) || (!ean_r.is_empty() && f.get::<_, String>(4) == ean_r) || se_parecen(&img_r, &f.get::<_, String>(7))).collect()
        }
    };
    let cotizaciones: Vec<faro_territorio::precios::Cotizacion> = conservadas.iter().map(|f| faro_territorio::precios::Cotizacion { producto: texto.clone(), precio: f.get(3), fuente: f.get(0), tipo_fuente: "secundaria".into(), unidad: String::new() }).collect();
    let p = faro_territorio::precios::Parametros::default();
    let analisis = (!cotizaciones.is_empty()).then(|| faro_territorio::precios::analizar(&texto, &cotizaciones.iter().collect::<Vec<_>>(), &p));
    let tiendas: std::collections::BTreeSet<String> = conservadas.iter().map(|f| f.get::<_, String>(0)).collect();
    let filtro_foto = referencia.map(|r| json!({ "referencia": r.get::<_, String>(1), "tienda": r.get::<_, String>(0), "url": r.get::<_, String>(5), "conservados": conservadas.len(), "descartados": total_antes - conservadas.len(), "con_huella": huellas.contains_key(&r.get::<_, String>(7)) }));
    Ok(Json(json!({ "producto": texto, "rubro": rubro, "tienda": proveedor, "parametros": p, "analisis": analisis, "tiendas": tiendas, "excluidos": excluidos.len(), "filtro_foto": filtro_foto, "umbrales_foto": umbrales,
        "fuentes": conservadas.iter().map(|f| { let img: String = f.get(7);
            // Cuántas otras fuentes de este informe se ven como ésta: ayuda a elegir la referencia.
            let parecidas = conservadas.iter().filter(|g| g.get::<_, String>(5) != f.get::<_, String>(5) && se_parecen(&img, &g.get::<_, String>(7))).count();
            json!({ "tienda": f.get::<_, String>(0), "producto": f.get::<_, String>(1), "marca": f.get::<_, String>(2), "precio": f.get::<_, f64>(3), "codigo_barras": f.get::<_, String>(4), "url": f.get::<_, String>(5), "fecha": f.get::<_, String>(6),
                    "imagen": img, "con_huella": huellas.contains_key(&img), "parecidas": parecidas }) }).collect::<Vec<_>>() })))
}

/// Huellas visuales de las fotos pedidas: las guardadas y, de las que faltan, hasta 150 por
/// consulta (descarga con pausa; el resto se calcula en la siguiente).
async fn huellas_de(e: &Compartido, imagenes: &[String], bajar: bool) -> Res<HashMap<String, faro_fuentes::foto::Huella>> {
    let mut huellas: HashMap<String, faro_fuentes::foto::Huella> = HashMap::new();
    let mut intentadas: std::collections::HashSet<String> = std::collections::HashSet::new();
    for f in e.bd.query("SELECT imagen, forma, color FROM huellas_foto WHERE imagen = ANY($1::text[])", &[&imagenes]).await.map_err(interno)? {
        intentadas.insert(f.get(0));
        if let (Some(forma), Some(color)) = (f.get::<_, Option<i64>>(1), f.get::<_, Option<Vec<f32>>>(2)) {
            huellas.insert(f.get(0), faro_fuentes::foto::Huella { forma: forma as u64, color });
        }
    }
    let faltan: Vec<String> = imagenes.iter().filter(|u| !intentadas.contains(*u)).cloned().collect::<std::collections::BTreeSet<_>>().into_iter().take(150).collect();
    if faltan.is_empty() || !bajar {
        return Ok(huellas);
    }
    let nuevas = tokio::task::spawn_blocking(move || {
        let agente = faro_fuentes::foto::agente();
        faltan.into_iter().map(|u| { let h = faro_fuentes::foto::huella_de_url(&agente, &u); std::thread::sleep(std::time::Duration::from_millis(60)); (u, h) }).collect::<Vec<_>>()
    }).await.map_err(interno)?;
    for (u, h) in nuevas {
        // También se anota la foto que no se pudo leer, para no pedirla de nuevo en cada consulta.
        e.bd.execute("INSERT INTO huellas_foto (imagen, forma, color) VALUES ($1, $2, $3) ON CONFLICT (imagen) DO NOTHING", &[&u, &h.as_ref().map(|x| x.forma as i64), &h.as_ref().map(|x| x.color.clone())]).await.map_err(interno)?;
        if let Some(h) = h {
            huellas.insert(u, h);
        }
    }
    Ok(huellas)
}

/// Calculadora de derivados de la Hoja de Balance de Alimentos: el cuerpo es la tabla de tasas (CSV).
async fn api_balance(State(e): State<Compartido>, cabeceras: HeaderMap, Query(q): Query<HashMap<String, String>>, cuerpo: String) -> Res<Json<Value>> {
    // Cargar una tabla propia es una acción de quien trabaja el dato: pide sesión, como las demás cargas.
    autorizar_en(&e, &cabeceras, "balance").await?;
    let tasas = faro_territorio::balance::leer_tasas(cuerpo.as_bytes()).map_err(pedido)?;
    let primarios = faro_territorio::balance::primarios(&tasas);
    let balance = match q.get("producto").filter(|p| !p.is_empty()) {
        Some(producto) => {
            let cantidad: f64 = q.get("cantidad").and_then(|c| c.replace(',', "").parse().ok()).ok_or_else(|| pedido("escriba la cantidad del producto primario"))?;
            Some(faro_territorio::balance::calcular(&tasas, producto, cantidad).map_err(pedido)?)
        }
        None => None,
    };
    Ok(Json(json!({ "primarios": primarios, "filas": tasas.len(), "balance": balance })))
}

/// Carpeta de los diccionarios de la interfaz: un archivo por idioma (`en.json`, `cak.json`) y la
/// plantilla con todos los textos de la pantalla en español.
fn carpeta_idiomas() -> PathBuf {
    PathBuf::from(var("FARO_ESTANDAR", "datos/estandar")).join("idiomas")
}

/// El código de un idioma: dos o tres letras minúsculas (ISO 639). Nada más llega al disco.
fn codigo_de_idioma(codigo: &str) -> bool {
    (2..=3).contains(&codigo.len()) && codigo.bytes().all(|b| b.is_ascii_lowercase())
}

/// Las traducciones que sí están hechas (las vacías se dejan en el archivo para quien revisa).
fn textos_traducidos(dic: &Value) -> serde_json::Map<String, Value> {
    dic["textos"].as_object().map(|t| t.iter().filter(|(_, v)| v.as_str().is_some_and(|x| !x.trim().is_empty())).map(|(k, v)| (k.clone(), v.clone())).collect()).unwrap_or_default()
}

/// Idiomas disponibles para la interfaz, con su estado y cuánto de la pantalla cubren.
/// El español es el idioma de origen; los datos no se traducen en ningún idioma.
async fn api_idiomas() -> Json<Value> {
    let dir = carpeta_idiomas();
    let leer = |nombre: &str| std::fs::read_to_string(dir.join(nombre)).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok());
    let plantilla: std::collections::HashSet<String> = leer("plantilla.json").and_then(|p| p["textos"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())).unwrap_or_default();
    let mut idiomas = vec![json!({ "codigo": "es", "nombre": "Español", "estado": "origen", "cobertura_pct": 100 })];
    let mut archivos: Vec<String> = std::fs::read_dir(&dir).map(|d| d.flatten().filter_map(|a| a.file_name().into_string().ok()).collect()).unwrap_or_default();
    archivos.sort();
    for archivo in archivos {
        let Some(codigo) = archivo.strip_suffix(".json").filter(|c| codigo_de_idioma(c)) else { continue };
        let Some(dic) = leer(&archivo) else { continue };
        let hechos = textos_traducidos(&dic);
        let de_la_plantilla = hechos.keys().filter(|k| plantilla.contains(*k)).count();
        idiomas.push(json!({
            "codigo": codigo, "nombre": dic["nombre"].as_str().unwrap_or(codigo), "nombre_es": dic["nombre_es"], "estado": dic["estado"].as_str().unwrap_or("borrador"),
            "revisado_por": dic["revisado_por"], "nota": dic["nota"], "textos": hechos.len(), "de_la_plantilla": de_la_plantilla,
            "cobertura_pct": if plantilla.is_empty() { Value::Null } else { json!((de_la_plantilla as f64 * 1000.0 / plantilla.len() as f64).round() / 10.0) },
        }));
    }
    // Primero el idioma de origen, luego los completos y al final los borradores.
    idiomas.sort_by_key(|i| match i["estado"].as_str() { Some("origen") => 0, Some("borrador") => 2, _ => 1 });
    Json(json!({ "idiomas": idiomas, "textos_de_la_pantalla": plantilla.len(), "regla": "Se traduce la interfaz. Los nombres oficiales, los valores de los catálogos, las cifras y las citas se muestran como los publica su fuente." }))
}

/// El diccionario de un idioma, sin las entradas vacías. Se lee del disco en cada pedido: quien
/// corrige una traducción la ve al recargar la página, sin reiniciar el servidor.
async fn api_idioma(Ruta(codigo): Ruta<String>) -> Res<Response> {
    if !codigo_de_idioma(&codigo) { return Err(pedido("el código de idioma son dos o tres letras minúsculas")); }
    let texto = std::fs::read_to_string(carpeta_idiomas().join(format!("{codigo}.json"))).map_err(|_| pedido("este servidor no tiene ese idioma"))?;
    let mut dic: Value = serde_json::from_str(&texto).map_err(|e| interno(format!("el diccionario «{codigo}» no es JSON válido: {e}")))?;
    let hechos = textos_traducidos(&dic);
    dic["textos"] = Value::Object(hechos);
    Ok(([("cache-control", "no-cache")], Json(dic)).into_response())
}

/// Calculadora con las tasas de extracción de FAO para Guatemala, contrastadas con las cuentas de
/// la Hoja de Balance de Alimentos 2024 del INE (producción del derivado ÷ primario procesado).
async fn api_balance_oficial(Query(q): Query<HashMap<String, String>>) -> Res<Json<Value>> {
    use faro_homologa::normalizar::basico;
    let dir = std::path::PathBuf::from(var("FARO_ESTANDAR", "datos/estandar"));
    let texto = std::fs::read_to_string(dir.join("tasas_extraccion_fao_guatemala.csv")).map_err(|_| pedido("este servidor no tiene la tabla de tasas de FAO"))?;
    let tasas = faro_territorio::balance::leer_tasas(texto.as_bytes()).map_err(interno)?;
    let detalle: Vec<HashMap<String, String>> = csv::Reader::from_reader(texto.as_bytes()).deserialize().flatten().collect();
    // Cuentas del INE: alimento -> (producción, procesado).
    let cuentas: HashMap<String, (String, Option<f64>, Option<f64>)> = csv::Reader::from_path(var("FARO_HBA", "datos/referencia/hba_2024_cuentas.csv")).map(|mut l| l.deserialize::<HashMap<String, String>>().flatten()
        .filter_map(|f| Some((basico(f.get("alimento")?), (f.get("alimento")?.clone(), f.get("produccion_t")?.parse().ok(), f.get("procesado_t")?.parse().ok())))).collect()).unwrap_or_default();
    let primarios: Vec<Value> = faro_territorio::balance::primarios(&tasas).into_iter().map(|p| { let c = cuentas.get(&basico(&p)); json!({ "primario": p, "procesado_ine_t": c.and_then(|c| c.2).filter(|x| *x > 0.0) }) }).collect();
    let Some(producto) = q.get("producto").filter(|p| !p.is_empty()) else {
        return Ok(Json(json!({ "primarios": primarios, "filas": tasas.len() })));
    };
    let procesado_ine = cuentas.get(&basico(producto)).and_then(|c| c.2).filter(|x| *x > 0.0);
    let cantidad: f64 = q.get("cantidad").and_then(|c| c.replace(',', "").parse().ok()).or(procesado_ine).ok_or_else(|| pedido("escriba la cantidad del producto primario que se procesa"))?;
    let balance = faro_territorio::balance::calcular(&tasas, producto, cantidad).map_err(pedido)?;
    // Al lado de cada derivado, lo que el INE publicó para 2024 y la tasa que resulta de sus propias cifras.
    let contraste: Vec<Value> = balance.derivados.iter().map(|d| {
        let fila = detalle.iter().find(|f| f.get("derivado") == Some(&d.derivado) && f.get("primario").is_some_and(|p| p.eq_ignore_ascii_case(&balance.primario)));
        let nombre_ine = fila.and_then(|f| f.get("derivado_en_hba_ine")).filter(|n| !n.is_empty()).cloned().unwrap_or_else(|| d.derivado.clone());
        let ine = cuentas.get(&basico(&nombre_ine));
        let produccion = ine.and_then(|c| c.1).filter(|x| *x > 0.0);
        json!({ "derivado": d.derivado, "relacion": fila.and_then(|f| f.get("relacion")), "nombre_ine": ine.map(|c| c.0.clone()), "produccion_ine_t": produccion,
                "tasa_implicita_ine_pct": produccion.zip(procesado_ine).map(|(p, x)| (p / x * 1000.0).round() / 10.0) })
    }).collect();
    Ok(Json(json!({ "primarios": primarios, "filas": tasas.len(), "balance": balance, "procesado_ine_t": procesado_ine, "contraste": contraste })))
}

/// Busca departamentos, municipios y lugares poblados por nombre o código (para ir a ellos en el mapa).
async fn api_buscar_lugar(State(e): State<Compartido>, Query(q): Query<HashMap<String, String>>) -> Res<Json<Value>> {
    let texto = faro_homologa::normalizar::basico(q.get("q").map(String::as_str).unwrap_or(""));
    if texto.chars().count() < 2 {
        return Ok(Json(json!([])));
    }
    let palabras: Vec<&str> = texto.split(' ').collect();
    let mut hallados: Vec<(u8, usize, Value)> = Vec::new();
    for (orden, nivel) in [(0u8, Nivel::Departamento), (1, Nivel::Municipio), (2, Nivel::LugarPoblado)] {
        for l in e.catalogo.de_nivel(nivel, None) {
            let nombre = faro_homologa::normalizar::basico(&l.nombre);
            // También se busca en los otros nombres de uso común del catálogo (alias).
            let en = |t: &str| palabras.iter().all(|p| t.split(' ').any(|n| n.starts_with(p)));
            let coincide = l.codigo == texto || en(&nombre) || l.alias.split('|').filter(|a| !a.is_empty()).any(|a| en(&faro_homologa::normalizar::basico(a)));
            if !coincide {
                continue;
            }
            let padre = e.catalogo.por_codigo(&l.padre);
            let abuelo = padre.and_then(|p| e.catalogo.por_codigo(&p.padre));
            let donde = match orden { 0 => String::new(), 1 => padre.map(|p| p.nombre.clone()).unwrap_or_default(), _ => format!("{}, {}", padre.map(|p| p.nombre.as_str()).unwrap_or(""), abuelo.map(|p| p.nombre.as_str()).unwrap_or("")) };
            // Primero lo que coincide exacto o empieza igual; luego por nivel.
            let nombre_nivel = ["departamento", "municipio", "lugar"][orden as usize];
            let cercania = if nombre == texto || l.codigo == texto { 0 } else if nombre.starts_with(&texto) { 1 } else { 2 };
            hallados.push((cercania, hallados.len(), json!({ "nivel": nombre_nivel, "codigo": l.codigo, "nombre": l.nombre, "donde": donde, "municipio": if orden == 2 { l.padre.clone() } else { String::new() }, "poblacion_2018": e.censo_lugar.get(&l.codigo).map(|c| c[0]), "lon": l.longitud, "lat": l.latitud })));
        }
    }
    let total = hallados.len();
    hallados.sort_by_key(|(c, i, _)| (*c, *i));
    Ok(Json(json!({ "total": total, "lugares": hallados.into_iter().take(15).map(|(_, _, v)| v).collect::<Vec<_>>() })))
}

#[derive(Deserialize)]
struct DecisionPrecio {
    rubro: String,
    ean: String,
    proveedor: String,
    /// "correcto", "error" o vacío para quitar la decisión.
    decision: String,
}

/// Guarda lo que una persona decidió sobre el precio de una tienda para un código de barras.
async fn api_decision_precio(State(e): State<Compartido>, cabeceras: HeaderMap, Json(d): Json<DecisionPrecio>) -> Res<Json<Value>> {
    let quien = autorizar_en(&e, &cabeceras, "rubros").await?;
    match d.decision.as_str() {
        "" => { e.bd.execute("DELETE FROM decisiones_precio WHERE rubro = $1 AND ean = $2 AND proveedor = $3", &[&d.rubro, &d.ean, &d.proveedor]).await.map_err(interno)?; }
        "correcto" | "error" => {
            e.bd.execute("INSERT INTO decisiones_precio (rubro, ean, proveedor, decision, usuario) VALUES ($1,$2,$3,$4,$5) ON CONFLICT (rubro, ean, proveedor) DO UPDATE SET decision = $4, usuario = $5, fecha = now()", &[&d.rubro, &d.ean, &d.proveedor, &d.decision, &quien.nombre]).await.map_err(interno)?;
        }
        otra => return Err(pedido(format!("decisión {otra:?} no admitida (correcto o error)"))),
    }
    Ok(Json(json!({ "guardado": true })))
}

/// Ejecuta todas las tareas de muestreo: en segundo plano, una tienda a la vez por sitio.
async fn api_muestreos_todas(State(e): State<Compartido>, cabeceras: HeaderMap) -> Res<Json<Value>> {
    autorizar_en(&e, &cabeceras, "muestreos").await?;
    let mut por_sitio: BTreeMap<String, Vec<faro_fuentes::muestreo::Tarea>> = BTreeMap::new();
    for t in &e.tareas {
        por_sitio.entry(t.base.clone()).or_default().push(t.clone());
    }
    let (sitios, tareas) = (por_sitio.len(), e.tareas.len());
    for (_, lista) in por_sitio {
        let e = e.clone();
        tokio::spawn(async move {
            for t in lista {
                let r = ejecutar_muestreo(&e, &t).await;
                eprintln!("muestreo {}: {}", t.id, r.map(|v| v.to_string()).unwrap_or_else(|f| f.1));
            }
        });
    }
    Ok(Json(json!({ "en_curso": true, "tareas": tareas, "sitios": sitios })))
}

/// Los precios publicados de un rubro (el más reciente de cada producto de cada tienda), en CSV.
async fn api_publicados_csv(State(e): State<Compartido>, Query(q): Query<HashMap<String, String>>) -> Res<Response> {
    asegurar_ultimos(&e.bd).await.map_err(interno)?;
    let _turno = turno_pesado()?;
    let rubro = q.get("rubro").cloned().unwrap_or_default();
    let proveedor = q.get("proveedor").cloned().unwrap_or_default();
    let palabras: Vec<String> = faro_homologa::normalizar::basico(q.get("q").map(String::as_str).unwrap_or("")).split(' ').filter(|p| !p.is_empty()).map(|p| format!("%{}%", p.to_lowercase())).collect();
    let filas = e.bd.query(
        "SELECT rubro, proveedor, producto, marca, precio, precio_lista, ean, termino, url, to_char(observado, 'YYYY-MM-DD') FROM
           (SELECT * FROM precios_ultimos p WHERE NOT EXISTS (SELECT 1 FROM decisiones_precio d WHERE d.decision = 'error' AND p.ean <> '' AND d.rubro = p.rubro AND d.ean = p.ean AND d.proveedor = p.proveedor)) u
         WHERE ($1 = '' OR rubro = $1) AND ($2 = '' OR proveedor = $2) AND translate(lower(producto), 'áéíóúüñ', 'aeiouun') LIKE ALL($3::text[]) ORDER BY rubro, producto, precio LIMIT 300000",
        &[&rubro, &proveedor, &palabras]).await.map_err(interno)?;
    // Una celda que empieza con = + - @ se abriría como fórmula en una hoja de cálculo: se neutraliza.
    let campo = |t: String| { let t = if t.starts_with(['=', '+', '-', '@', '\t', '\r']) { format!("'{t}") } else { t }; if t.contains([',', '"', '\n']) { format!("\"{}\"", t.replace('"', "\"\"")) } else { t } };
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

/// Censo 2018 de un municipio: suma de sus lugares poblados y sus lugares más grandes.
fn censo_del_municipio(e: &Estado, codigo: &str) -> Option<Value> {
    let lugares: Vec<(&faro_homologa::catalogo::Lugar, &Vec<i64>)> = e.catalogo.de_nivel(Nivel::LugarPoblado, Some(codigo)).filter_map(|l| Some((l, e.censo_lugar.get(&l.codigo)?))).collect();
    if lugares.is_empty() {
        return None;
    }
    let suma = |i: usize| lugares.iter().map(|(_, c)| c.get(i).copied().unwrap_or(0)).sum::<i64>();
    let mut mayores = lugares.clone();
    mayores.sort_by_key(|(_, c)| std::cmp::Reverse(c[0]));
    Some(json!({ "lugares": lugares.len(), "poblacion": suma(0), "hombres": suma(1), "mujeres": suma(2), "edad_0_14": suma(3), "edad_15_64": suma(4), "edad_65_mas": suma(5), "urbana": suma(6), "rural": suma(7), "hogares": suma(8), "viviendas": suma(9), "viviendas_ocupadas": suma(10),
        "mayores": mayores.iter().take(8).map(|(l, c)| json!({ "codigo": l.codigo, "nombre": l.nombre, "poblacion": c[0], "hogares": c[8], "viviendas": c[9] })).collect::<Vec<_>>() }))
}

/// Todo lo que FARO sabe de un municipio, para la ficha del mapa y el reporte.
async fn api_municipio(State(e): State<Compartido>, Ruta(codigo): Ruta<String>, cabeceras: HeaderMap) -> Res<Json<Value>> {
    let (exacto, umbral) = (ve_conteos_exactos(&e, &cabeceras).await, umbral_de_celda());
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
    let registros = e.bd.query("WITH ultimo AS (SELECT solicitud, corte, max(envio_id) AS envio_id FROM hechos GROUP BY 1, 2) SELECT h.solicitud, sum(h.n)::bigint FROM hechos h JOIN ultimo u USING (solicitud, corte, envio_id) WHERE h.municipio = $1 GROUP BY 1 ORDER BY 1", &[&codigo]).await.map_err(interno)?;
    // La ficha oculta exactamente las mismas celdas que el mapa (incluida la supresión complementaria).
    let ocultas = celdas_ocultas(e.bd.query(CONTEOS_POR_MUNICIPIO, &[]).await.map_err(interno)?.iter().map(|f| (f.get(0), f.get(1), f.get(2))), exacto, umbral);
    Ok(Json(json!({
        "codigo": codigo, "nombre": l.nombre, "departamento": e.catalogo.por_codigo(&l.padre).map(|d| d.nombre.clone()), "anio_actual": hoy,
        "poblacion": poblacion, "poblacion_cambio": e.poblacion.cambio(&codigo, 2018, hasta),
        "puesto_poblacion": puesto(&|m| e.poblacion.total(m, hoy).map(|v| v as f64)), "puesto_crecimiento": puesto(&|m| e.poblacion.cambio(m, 2018, hasta).map(|c| c.tasa_anual_pct)),
        "construido": serie.iter().map(|p| json!([p.anio, (p.edificaciones * 10.0).round() / 10.0])).collect::<Vec<_>>(), "construido_analisis": analisis,
        "lugares_poblados": e.catalogo.de_nivel(Nivel::LugarPoblado, Some(&codigo)).count(), "sin_conexion": sin_conexion,
        "censo_2018": censo_del_municipio(&e, &codigo),
        "edades": e.edades.get(&codigo).map(|v| v.iter().map(|(a, d, h, m)| json!([a, d, h, m])).collect::<Vec<_>>()),
        "proveedores": e.construccion.proveedores.iter().filter(|p| p.get("municipio_codigo").map(String::as_str) == Some(codigo.as_str())).filter_map(|p| p.get("nombre")).collect::<Vec<_>>(),
        "registros": registros.iter().map(|f| { let n: i64 = f.get(1); if ocultas.contains(&(f.get::<_, String>(0), codigo.clone())) { json!({ "solicitud": f.get::<_, String>(0), "total": null, "protegido": true }) } else { json!({ "solicitud": f.get::<_, String>(0), "total": n }) } }).collect::<Vec<_>>(),
        "censo_indicadores": leer_censo_indicadores(&PathBuf::from(var("FARO_REFERENCIA", "datos/referencia"))).remove(&codigo),
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
    let quien = autorizar_en(&e, &cabeceras, "memoria").await?;
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
    /// Secciones del portal que su grupo puede usar. `None` (usuario sin grupo) = todas las que su rol permite.
    secciones: Option<Vec<String>>,
    /// Institución por la que envía (solo en las cuentas de rol «institucion»); vacío en las demás.
    #[serde(default)]
    institucion: String,
}

const ROLES: [&str; 5] = ["admin", "analista", "consulta", "operador", "institucion"];
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
    let f = e.bd.query_opt("SELECT u.id, u.usuario, u.nombre, u.rol, u.debe_cambiar, g.secciones, g.nombre, u.institucion FROM sesiones s JOIN usuarios u ON u.id = s.usuario_id LEFT JOIN grupos g ON g.id = u.grupo_id WHERE s.huella = $1 AND s.expira > now() AND u.activo", &[&huella_de_llave(&llave)]).await.map_err(interno)?;
    Ok(f.map(|f| Sesion { id: f.get(0), usuario: f.get(1), nombre: f.get(2), rol: f.get(3), debe_cambiar: f.get(4), secciones: f.get(5), institucion: f.get(7) }))
}

fn galleta(llave: &str, segundos: i64) -> (&'static str, String) {
    // Con FARO_HTTPS=1 (portal detrás de TLS) la galleta solo viaja cifrada.
    let segura = if std::env::var("FARO_HTTPS").is_ok_and(|v| v == "1") { "; Secure" } else { "" };
    ("set-cookie", format!("faro_sesion={llave}; HttpOnly; SameSite=Strict; Path=/; Max-Age={segundos}{segura}"))
}

async fn api_sesion(State(e): State<Compartido>, cabeceras: HeaderMap) -> Res<Json<Value>> {
    Ok(Json(match sesion_de(&e, &cabeceras).await? {
        Some(s) => json!({ "sesion": true, "usuario": s, "guia_evaluacion": guia_de_evaluacion() }),
        None => json!({ "sesion": false, "guia_evaluacion": guia_de_evaluacion() }),
    }))
}

/// La «Guía para evaluadores» existe para el concurso: `FARO_GUIA_EVALUACION=0` la quita del menú.
fn guia_de_evaluacion() -> bool {
    std::env::var("FARO_GUIA_EVALUACION").map(|v| v.trim() != "0").unwrap_or(true)
}

#[derive(Deserialize)]
struct Certificacion {
    municipio: String,
    #[serde(default)]
    expediente: String,
}

/// Certifica un reporte municipal: anota quién lo certificó, cuándo y la huella de los datos entregados,
/// y devuelve un folio. No guarda nada de quien pidió la información: su nombre y su correo solo se
/// escriben en el documento, en el navegador de quien certifica.
async fn api_certificar(State(e): State<Compartido>, cabeceras: HeaderMap, Json(c): Json<Certificacion>) -> Res<Json<Value>> {
    let quien = autorizar_en(&e, &cabeceras, "certificar").await?;
    let l = e.catalogo.por_codigo(&c.municipio).filter(|l| l.nivel == Nivel::Municipio).ok_or_else(|| pedido(format!("{} no es un municipio", c.municipio)))?;
    let (nombre, codigo) = (l.nombre.clone(), c.municipio.clone());
    let expediente: String = c.expediente.trim().chars().filter(|x| !x.is_control()).take(60).collect();
    // La huella se calcula sobre lo que ve cualquier visitante (sin sesión): así se puede volver a comprobar.
    let Json(datos) = api_municipio(State(e.clone()), Ruta(codigo.clone()), HeaderMap::new()).await?;
    let huella = huella_de(&datos);
    let f = e.bd.query_one("INSERT INTO certificaciones (municipio, expediente, quien, huella) VALUES ($1,$2,$3,$4) RETURNING id, to_char(cuando AT TIME ZONE 'America/Guatemala', 'YYYY-MM-DD HH24:MI'), extract(year from cuando AT TIME ZONE 'America/Guatemala')::int", &[&codigo, &expediente, &quien.nombre, &huella]).await.map_err(interno)?;
    let folio = format!("FARO-{}-{:05}", f.get::<_, i32>(2), f.get::<_, i64>(0));
    bitacora(&e, &quien.nombre, "certificar reporte", &format!("{folio}: {nombre} ({codigo}), expediente {expediente}")).await;
    Ok(Json(json!({ "folio": folio, "numero": f.get::<_, i64>(0), "cuando": f.get::<_, String>(1), "huella": huella, "certifica": quien.nombre, "municipio": codigo, "nombre": nombre })))
}

fn huella_de(datos: &Value) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(datos.to_string().as_bytes()).iter().map(|b| format!("{b:02x}")).collect()
}

/// Comprobación pública de una certificación: dice si existe y si los datos de hoy son los mismos que se certificaron.
async fn api_certificacion(State(e): State<Compartido>, Ruta(folio): Ruta<String>) -> Res<Json<Value>> {
    let numero: i64 = folio.rsplit('-').next().and_then(|n| n.parse().ok()).ok_or_else(|| pedido("el folio tiene la forma FARO-AAAA-00000"))?;
    let Some(f) = e.bd.query_opt("SELECT municipio, expediente, quien, huella, to_char(cuando AT TIME ZONE 'America/Guatemala', 'YYYY-MM-DD HH24:MI'), extract(year from cuando AT TIME ZONE 'America/Guatemala')::int FROM certificaciones WHERE id = $1", &[&numero]).await.map_err(interno)? else {
        return Err(Fallo(StatusCode::NOT_FOUND, "no hay una certificación con ese folio".into()));
    };
    let (codigo, huella): (String, String) = (f.get(0), f.get(3));
    let Json(datos) = api_municipio(State(e.clone()), Ruta(codigo.clone()), HeaderMap::new()).await?;
    Ok(Json(json!({ "folio": format!("FARO-{}-{:05}", f.get::<_, i32>(5), numero), "municipio": codigo, "expediente": f.get::<_, String>(1), "certifica": f.get::<_, String>(2), "cuando": f.get::<_, String>(4), "huella": huella, "datos_sin_cambios": huella_de(&datos) == huella })))
}

#[derive(Deserialize)]
struct Ingreso {
    usuario: String,
    clave: String,
}

/// Protección estadística (secreto estadístico): un conteo de registros administrativos menor que
/// este umbral podría señalar a personas concretas, así que no se publica su valor exacto.
/// `FARO_UMBRAL_CELDA` lo cambia (3 por omisión; 0 lo desactiva).
fn umbral_de_celda() -> i64 {
    std::env::var("FARO_UMBRAL_CELDA").ok().and_then(|v| v.parse().ok()).unwrap_or(3)
}

/// ¿Quien consulta puede ver los conteos exactos? Solo analistas y administradores con sesión.
async fn ve_conteos_exactos(e: &Estado, cabeceras: &HeaderMap) -> bool {
    matches!(sesion_de(e, cabeceras).await, Ok(Some(s)) if (s.rol == "admin" || s.rol == "analista") && !s.debe_cambiar)
}

/// Fallos seguidos de ingreso por usuario y hasta cuándo queda bloqueado.
static INTENTOS: std::sync::LazyLock<std::sync::Mutex<HashMap<String, (u32, std::time::Instant)>>> = std::sync::LazyLock::new(Default::default);
/// Verificaciones de contraseña simultáneas.
static ARGON: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(3);
/// Consultas pesadas simultáneas (precios publicados, descargas, informes).
static PESADAS: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(3);
fn turno_pesado() -> Res<tokio::sync::SemaphorePermit<'static>> {
    PESADAS.try_acquire().map_err(|_| Fallo(StatusCode::SERVICE_UNAVAILABLE, "el portal está atendiendo otras consultas grandes: intente de nuevo en unos segundos".into()))
}

async fn api_ingresar(State(e): State<Compartido>, Json(i): Json<Ingreso>) -> Result<Response, Fallo> {
    let f = e.bd.query_opt("SELECT id, usuario, nombre, rol, debe_cambiar, clave FROM usuarios WHERE usuario = $1 AND activo", &[&i.usuario.trim().to_lowercase()]).await.map_err(interno)?;
    // Freno a los intentos: tras cinco fallos seguidos de un usuario se le bloquea medio minuto, y
    // el bloqueo se duplica con cada fallo más (tope de 15 minutos). Un ingreso correcto lo borra.
    let quien = i.usuario.trim().to_lowercase();
    if let Some(espera) = INTENTOS.lock().map_err(interno)?.get(&quien).and_then(|(_, hasta)| hasta.checked_duration_since(std::time::Instant::now())) {
        return Err(Fallo(StatusCode::TOO_MANY_REQUESTS, format!("demasiados intentos fallidos: espere {} segundos", espera.as_secs() + 1)));
    }
    let (clave, guardada) = (i.clave.clone(), f.as_ref().map(|f| f.get::<_, String>(5)));
    // Si el usuario no existe, la contraseña se verifica igual contra una huella de relleno: así la
    // respuesta tarda lo mismo y el tiempo no delata qué usuarios existen. Como máximo tres
    // verificaciones a la vez, para que una ráfaga de intentos no agote la memoria.
    static RELLENO: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    let _turno = ARGON.acquire().await.map_err(interno)?;
    let bien = tokio::task::spawn_blocking(move || {
        let existe = guardada.is_some();
        let huella = guardada.unwrap_or_else(|| RELLENO.get_or_init(|| huella_de_clave("relleno-sin-uso").unwrap_or_default()).clone());
        clave_correcta(&clave, &huella) && existe
    })
    .await
    .map_err(interno)?;
    drop(_turno);
    let Some(f) = f.filter(|_| bien) else {
        {
            let mut intentos = INTENTOS.lock().map_err(interno)?;
            if intentos.len() > 10_000 { intentos.clear(); }
            let (fallos, hasta) = intentos.entry(quien).or_insert((0, std::time::Instant::now()));
            *fallos += 1;
            if *fallos >= 5 {
                *hasta = std::time::Instant::now() + std::time::Duration::from_secs((30u64 << (*fallos - 5).min(5)).min(900));
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(700)).await;
        return Err(Fallo(StatusCode::UNAUTHORIZED, "usuario o contraseña incorrectos".into()));
    };
    INTENTOS.lock().map_err(interno)?.remove(&quien);
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
    let extra = e.bd.query_one("SELECT g.secciones, u.institucion FROM usuarios u LEFT JOIN grupos g ON g.id = u.grupo_id WHERE u.id = $1", &[&id]).await.map_err(interno)?;
    let s = Sesion { id, usuario: f.get(1), nombre: f.get(2), rol: f.get(3), debe_cambiar: f.get(4), secciones: extra.get(0), institucion: extra.get(1) };
    Ok(([galleta(&llave, HORAS_DE_SESION * 3600)], Json(json!({ "sesion": true, "usuario": s }))).into_response())
}

async fn api_salir(State(e): State<Compartido>, cabeceras: HeaderMap) -> Result<Response, Fallo> {
    if let Some(llave) = llave_de(&cabeceras) {
        e.bd.execute("DELETE FROM sesiones WHERE huella = $1", &[&huella_de_llave(&llave)]).await.map_err(interno)?;
    }
    Ok(([galleta("", 0)], Json(json!({ "sesion": false }))).into_response())
}

fn clave_aceptable(clave: &str, usuario: &str) -> Res<()> {
    const COMUNES: [&str; 14] = ["password", "contraseña", "contrasena", "12345678", "123456789", "1234567890", "qwertyuiop", "guatemala", "administrador", "admin12345", "bienvenido", "abcdefghij", "0123456789", "ine2026ine"];
    let baja = clave.to_lowercase();
    let clases = [clave.chars().any(|c| c.is_lowercase()), clave.chars().any(|c| c.is_uppercase()), clave.chars().any(|c| c.is_ascii_digit()), clave.chars().any(|c| !c.is_alphanumeric())].iter().filter(|x| **x).count();
    if clave.chars().count() < 10 {
        return Err(pedido("la contraseña debe tener al menos 10 caracteres"));
    }
    if clases < 2 || COMUNES.iter().any(|c| baja.contains(c)) || (!usuario.is_empty() && baja.contains(&usuario.to_lowercase())) {
        return Err(pedido("la contraseña es demasiado fácil de adivinar: combine letras con números o signos y no use su usuario ni palabras comunes"));
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
        // El nombre firma la bitácora y los tableros: nadie puede tomar el nombre (ni el usuario) de otra cuenta.
        let ocupado: i64 = e.bd.query_one("SELECT count(*) FROM usuarios WHERE id <> $2 AND (lower(nombre) = lower($1) OR lower(usuario) = lower($1))", &[&nombre, &s.id]).await.map_err(interno)?.get(0);
        if ocupado > 0 || nombre.chars().count() > 80 {
            return Err(pedido("ese nombre ya lo usa otra cuenta (o es demasiado largo): elija otro"));
        }
        e.bd.execute("UPDATE usuarios SET nombre = $1 WHERE id = $2", &[&nombre, &s.id]).await.map_err(interno)?;
    }
    if let Some(nueva) = c.clave_nueva.filter(|n| !n.is_empty()) {
        clave_aceptable(&nueva, &s.usuario)?;
        let guardada: String = e.bd.query_one("SELECT clave FROM usuarios WHERE id = $1", &[&s.id]).await.map_err(interno)?.get(0);
        let actual = c.clave_actual.unwrap_or_default();
        let huella = tokio::task::spawn_blocking(move || if clave_correcta(&actual, &guardada) { huella_de_clave(&nueva).map(Some) } else { Ok(None) }).await.map_err(interno)?.map_err(interno)?;
        let huella = huella.ok_or_else(|| pedido("la contraseña actual no es correcta"))?;
        e.bd.execute("UPDATE usuarios SET clave = $1, debe_cambiar = FALSE WHERE id = $2", &[&huella, &s.id]).await.map_err(interno)?;
        // Las otras sesiones abiertas de este usuario dejan de valer.
        let esta = llave_de(&cabeceras).map(|l| huella_de_llave(&l)).unwrap_or_default();
        e.bd.execute("DELETE FROM sesiones WHERE usuario_id = $1 AND huella <> $2", &[&s.id, &esta]).await.map_err(interno)?;
    }
    Ok(Json(json!({ "guardado": true })))
}

async fn solo_admin(e: &Estado, cabeceras: &HeaderMap) -> Res<Sesion> {
    let s = sesion_de(e, cabeceras).await?.filter(|s| s.rol == "admin").ok_or(Fallo(StatusCode::FORBIDDEN, "solo un administrador puede manejar usuarios".into()))?;
    if s.debe_cambiar {
        return Err(Fallo(StatusCode::FORBIDDEN, "antes de manejar usuarios debe cambiar su contraseña inicial en «Mi perfil»".into()));
    }
    Ok(s)
}

async fn api_usuarios(State(e): State<Compartido>, cabeceras: HeaderMap) -> Res<Json<Value>> {
    solo_admin(&e, &cabeceras).await?;
    let filas = e.bd.query("SELECT id, usuario, nombre, rol, activo, debe_cambiar, to_char(creado, 'YYYY-MM-DD'), to_char(ultimo_ingreso, 'YYYY-MM-DD HH24:MI'), grupo_id, institucion FROM usuarios ORDER BY id", &[]).await.map_err(interno)?;
    Ok(Json(json!(filas.iter().map(|f| json!({ "id": f.get::<_, i64>(0), "usuario": f.get::<_, String>(1), "nombre": f.get::<_, String>(2), "rol": f.get::<_, String>(3), "activo": f.get::<_, bool>(4),
        "debe_cambiar": f.get::<_, bool>(5), "creado": f.get::<_, String>(6), "ultimo_ingreso": f.get::<_, Option<String>>(7), "grupo_id": f.get::<_, Option<i64>>(8), "institucion": f.get::<_, String>(9) })).collect::<Vec<_>>())))
}

#[derive(Deserialize)]
struct UsuarioNuevo {
    usuario: String,
    nombre: String,
    rol: String,
    clave: String,
    /// Para el rol «institucion»: por cuál institución envía (tal como figura en sus solicitudes).
    #[serde(default)]
    institucion: String,
}

/// Instituciones que tienen al menos una solicitud: las únicas a las que se puede atar una cuenta.
fn instituciones(e: &Estado) -> Vec<String> {
    let mut v: Vec<String> = solicitudes(e).unwrap_or_default().into_iter().map(|s| s.institucion).collect();
    v.sort();
    v.dedup();
    v
}

async fn api_usuario_crear(State(e): State<Compartido>, cabeceras: HeaderMap, Json(u): Json<UsuarioNuevo>) -> Res<Json<Value>> {
    let admin = solo_admin(&e, &cabeceras).await?;
    let usuario = u.usuario.trim().to_lowercase();
    if usuario.len() < 3 || !usuario.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-')) {
        return Err(pedido("el usuario debe tener al menos 3 caracteres: letras sin tilde, números, punto, guion o guion bajo"));
    }
    if u.nombre.trim().is_empty() || !ROLES.contains(&u.rol.as_str()) {
        return Err(pedido("indique el nombre y un rol: admin, analista, consulta, operador o institucion"));
    }
    let institucion = if u.rol == "institucion" { u.institucion.trim().to_string() } else { String::new() };
    if u.rol == "institucion" && !instituciones(&e).contains(&institucion) {
        return Err(pedido(format!("una cuenta de institución debe indicar por cuál envía: {}", instituciones(&e).join(", "))));
    }
    clave_aceptable(&u.clave, &usuario)?;
    let clave = u.clave.clone();
    let huella = tokio::task::spawn_blocking(move || huella_de_clave(&clave)).await.map_err(interno)?.map_err(interno)?;
    let n = e.bd.execute("INSERT INTO usuarios (usuario, nombre, rol, clave, debe_cambiar, institucion) VALUES ($1,$2,$3,$4, TRUE, $5) ON CONFLICT (usuario) DO NOTHING", &[&usuario, &u.nombre.trim(), &u.rol, &huella, &institucion]).await.map_err(interno)?;
    if n == 0 {
        return Err(pedido(format!("ya existe el usuario {usuario}")));
    }
    bitacora(&e, &admin.nombre, "cuenta_creada", &format!("{usuario} ({}{})", u.rol, if institucion.is_empty() { String::new() } else { format!(": {institucion}") })).await;
    Ok(Json(json!({ "creado": usuario })))
}

#[derive(Deserialize)]
struct CambioUsuario {
    nombre: Option<String>,
    rol: Option<String>,
    activo: Option<bool>,
    clave: Option<String>,
    /// Grupo al que pasa; 0 lo deja sin grupo (todas las secciones que su rol permite).
    grupo_id: Option<i64>,
    /// Institución por la que envía (cuentas de rol «institucion»).
    institucion: Option<String>,
}

async fn api_usuario_cambiar(State(e): State<Compartido>, Ruta(id): Ruta<i64>, cabeceras: HeaderMap, Json(c): Json<CambioUsuario>) -> Res<Json<Value>> {
    let admin = solo_admin(&e, &cabeceras).await?;
    let actual = e.bd.query_opt("SELECT rol, activo FROM usuarios WHERE id = $1", &[&id]).await.map_err(interno)?.ok_or_else(|| pedido("no existe ese usuario"))?;
    if c.rol.as_deref().is_some_and(|r| !ROLES.contains(&r)) {
        return Err(pedido("rol no válido: admin, analista, consulta, operador o institucion"));
    }
    if let Some(i) = c.institucion.as_deref().map(str::trim) {
        if !i.is_empty() && !instituciones(&e).contains(&i.to_string()) {
            return Err(pedido(format!("institución no reconocida; las que tienen solicitud son: {}", instituciones(&e).join(", "))));
        }
        e.bd.execute("UPDATE usuarios SET institucion = $1 WHERE id = $2", &[&i, &id]).await.map_err(interno)?;
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
    if let Some(g) = c.grupo_id {
        let g = (g > 0).then_some(g);
        e.bd.execute("UPDATE usuarios SET grupo_id = $1 WHERE id = $2", &[&g, &id]).await.map_err(|_| pedido("no existe ese grupo"))?;
    }
    if let Some(clave) = c.clave.filter(|k| !k.is_empty()) {
        clave_aceptable(&clave, "")?;
        let huella = tokio::task::spawn_blocking(move || huella_de_clave(&clave)).await.map_err(interno)?.map_err(interno)?;
        // Contraseña puesta por el administrador: el usuario debe cambiarla y sus sesiones se cierran.
        e.bd.execute("UPDATE usuarios SET clave = $1, debe_cambiar = TRUE WHERE id = $2", &[&huella, &id]).await.map_err(interno)?;
        e.bd.execute("DELETE FROM sesiones WHERE usuario_id = $1", &[&id]).await.map_err(interno)?;
    }
    bitacora(&e, &admin.nombre, "cuenta_cambiada", &format!("usuario n.º {id}")).await;
    Ok(Json(json!({ "guardado": true })))
}

// ---------------------------------------------------------------------------------------------
// Grupos de usuarios
// ---------------------------------------------------------------------------------------------

/// Secciones del portal que un grupo puede tener.
const SECCIONES: [&str; 26] = ["mapa", "comparar", "certificar", "mejoras", "proceso", "tableros", "recibir", "pendientes", "memoria", "resultados", "crecimiento", "vial", "operadores", "campo", "alimentaria", "construccion", "precios", "canasta", "rubros", "balance", "muestreos", "ine", "capas", "acerca", "perfil", "usuarios"];

async fn api_grupos(State(e): State<Compartido>, cabeceras: HeaderMap) -> Res<Json<Value>> {
    solo_admin(&e, &cabeceras).await?;
    let filas = e.bd.query("SELECT g.id, g.nombre, g.descripcion, g.secciones, (SELECT count(*) FROM usuarios u WHERE u.grupo_id = g.id) FROM grupos g ORDER BY g.nombre", &[]).await.map_err(interno)?;
    Ok(Json(json!({ "secciones": SECCIONES, "grupos": filas.iter().map(|f| json!({ "id": f.get::<_, i64>(0), "nombre": f.get::<_, String>(1), "descripcion": f.get::<_, String>(2), "secciones": f.get::<_, Vec<String>>(3), "usuarios": f.get::<_, i64>(4) })).collect::<Vec<_>>() })))
}

#[derive(Deserialize)]
struct GrupoDatos {
    nombre: String,
    #[serde(default)]
    descripcion: String,
    secciones: Vec<String>,
}

fn grupo_valido(g: &GrupoDatos) -> Res<Vec<String>> {
    if g.nombre.trim().len() < 3 || g.nombre.chars().count() > 60 {
        return Err(pedido("el grupo necesita un nombre de 3 a 60 caracteres"));
    }
    if let Some(mala) = g.secciones.iter().find(|s| !SECCIONES.contains(&s.as_str())) {
        return Err(pedido(format!("sección desconocida: {mala}")));
    }
    // "usuarios" es solo de administradores; el perfil propio siempre se puede ver.
    let mut secciones: Vec<String> = g.secciones.iter().filter(|s| *s != "usuarios").cloned().collect();
    if !secciones.iter().any(|s| s == "perfil") {
        secciones.push("perfil".into());
    }
    Ok(secciones)
}

async fn api_grupo_guardar(State(e): State<Compartido>, cabeceras: HeaderMap, Json(g): Json<GrupoDatos>) -> Res<Json<Value>> {
    solo_admin(&e, &cabeceras).await?;
    let secciones = grupo_valido(&g)?;
    let id: i64 = e.bd.query_one("INSERT INTO grupos (nombre, descripcion, secciones) VALUES ($1,$2,$3) RETURNING id", &[&g.nombre.trim(), &g.descripcion.trim(), &secciones]).await.map_err(|_| pedido("ya existe un grupo con ese nombre"))?.get(0);
    Ok(Json(json!({ "id": id })))
}

async fn api_grupo_guardar_id(State(e): State<Compartido>, Ruta(id): Ruta<i64>, cabeceras: HeaderMap, Json(g): Json<GrupoDatos>) -> Res<Json<Value>> {
    solo_admin(&e, &cabeceras).await?;
    let secciones = grupo_valido(&g)?;
    let n = e.bd.execute("UPDATE grupos SET nombre = $1, descripcion = $2, secciones = $3 WHERE id = $4", &[&g.nombre.trim(), &g.descripcion.trim(), &secciones, &id]).await.map_err(|_| pedido("ya existe un grupo con ese nombre"))?;
    if n == 0 { return Err(pedido("no existe ese grupo")); }
    Ok(Json(json!({ "guardado": true })))
}

async fn api_grupo_borrar(State(e): State<Compartido>, Ruta(id): Ruta<i64>, cabeceras: HeaderMap) -> Res<Json<Value>> {
    solo_admin(&e, &cabeceras).await?;
    // Sus usuarios quedan sin grupo (la llave foránea los pone en nulo).
    e.bd.execute("DELETE FROM grupos WHERE id = $1", &[&id]).await.map_err(interno)?;
    Ok(Json(json!({ "borrado": true })))
}

// ---------------------------------------------------------------------------------------------
// Tableros
// ---------------------------------------------------------------------------------------------

#[derive(Deserialize)]
struct Solicitado {
    texto: String,
}

/// De una frase a la definición de un tablero (sin guardar nada).
async fn api_tablero_interpretar(State(e): State<Compartido>, Json(s): Json<Solicitado>) -> Res<Json<Value>> {
    if s.texto.trim().is_empty() || s.texto.chars().count() > 500 {
        return Err(pedido("escriba la solicitud en una frase (hasta 500 caracteres)"));
    }
    Ok(Json(json!({ "interpretacion": tablero::interpretar(&s.texto, &e.catalogo), "indicadores": indicadores_de_tablero(), "tipos": tablero::TIPOS })))
}

// ---- Trabajo de campo ----

/// Quién puede registrar visitas: cualquier usuario con sesión que no sea de consulta y que, si
/// pertenece a un grupo, tenga en él la sección «campo» u «operadores». Los administradores siempre
/// pueden. Quien no ha cambiado su contraseña inicial no puede.
async fn operador_de_campo(e: &Estado, cabeceras: &HeaderMap) -> Res<Sesion> {
    let s = con_sesion(e, cabeceras).await?;
    if s.debe_cambiar {
        return Err(Fallo(StatusCode::FORBIDDEN, "antes de registrar visitas debe cambiar su contraseña inicial en «Mi perfil»".into()));
    }
    if s.rol == "consulta" || s.rol == "institucion" || (s.rol != "admin" && s.secciones.as_ref().is_some_and(|v| !v.iter().any(|x| x == "campo" || x == "operadores"))) {
        return Err(Fallo(StatusCode::FORBIDDEN, "su usuario no tiene acceso al trabajo de campo".into()));
    }
    Ok(s)
}

/// ¿Coordina el trabajo de campo? (crea rutas y ve las visitas de todos). Sí: los administradores,
/// y los analistas sin grupo o cuyo grupo incluye la sección «operadores».
fn coordina_campo(s: &Sesion) -> bool {
    s.rol == "admin" || (s.rol == "analista" && s.secciones.as_ref().is_none_or(|v| v.iter().any(|x| x == "operadores")))
}

/// Rutas con sus paradas y la última visita de cada una. Un operador ve las suyas; quien coordina, todas.
async fn api_campo(State(e): State<Compartido>, cabeceras: HeaderMap) -> Res<Json<Value>> {
    let s = operador_de_campo(&e, &cabeceras).await?;
    let coordina = coordina_campo(&s);
    let rutas = e.bd.query("SELECT r.id, r.nombre, r.operador_id, u.nombre, r.formulario <> '', r.largo_km, to_char(r.creada, 'YYYY-MM-DD'), r.creada_por, r.demo FROM rutas_campo r JOIN usuarios u ON u.id = r.operador_id WHERE $1 OR r.operador_id = $2 ORDER BY r.id DESC LIMIT 200", &[&coordina, &s.id]).await.map_err(interno)?;
    let ids: Vec<i64> = rutas.iter().map(|f| f.get(0)).collect();
    let paradas = e
        .bd
        .query(
            "SELECT p.id, p.ruta_id, p.orden, p.lugar, p.nombre, p.longitud, p.latitud, v.id, to_char(v.cuando, 'YYYY-MM-DD HH24:MI'), v.distancia_m, v.foto_clase, v.foto_confianza, v.coherencia, v.cruce, (SELECT count(*) FROM visitas_campo x WHERE x.parada_id = p.id)
             FROM paradas_campo p LEFT JOIN LATERAL (SELECT * FROM visitas_campo v WHERE v.parada_id = p.id ORDER BY v.id DESC LIMIT 1) v ON TRUE WHERE p.ruta_id = ANY($1) ORDER BY p.ruta_id, p.orden",
            &[&ids],
        )
        .await
        .map_err(interno)?;
    let de_ruta = |id: i64| -> Vec<Value> {
        paradas.iter().filter(|p| p.get::<_, i64>(1) == id).map(|p| {
            let lugar: String = p.get(3);
            json!({ "id": p.get::<_, i64>(0), "orden": p.get::<_, i32>(2), "nombre": p.get::<_, String>(4), "longitud": p.get::<_, f64>(5), "latitud": p.get::<_, f64>(6),
                "municipio": e.catalogo.por_codigo(lugar.get(..4).unwrap_or("")).map(|l| l.nombre.clone()), "techo": e.techo_lugar.get(&lugar).map(|(m, pct)| json!({ "material": m, "pct": (pct * 10.0).round() / 10.0 })), "lugar": lugar,
                "visitas": p.get::<_, i64>(14), "visita": p.get::<_, Option<i64>>(7).map(|v| json!({ "id": v, "cuando": p.get::<_, Option<String>>(8), "distancia_m": p.get::<_, Option<f32>>(9), "foto_clase": p.get::<_, Option<String>>(10), "foto_confianza": p.get::<_, Option<f32>>(11), "coherencia": p.get::<_, Option<String>>(12), "cruce": p.get::<_, Option<Value>>(13) })) })
        }).collect()
    };
    let operadores = if coordina {
        e.bd.query("SELECT u.id, u.nombre, u.usuario, u.rol, g.nombre FROM usuarios u LEFT JOIN grupos g ON g.id = u.grupo_id WHERE u.activo AND (u.rol = 'operador' OR 'campo' = ANY(g.secciones)) ORDER BY u.nombre", &[]).await.map_err(interno)?
            .iter().map(|f| json!({ "id": f.get::<_, i64>(0), "nombre": f.get::<_, String>(1), "usuario": f.get::<_, String>(2), "grupo": f.get::<_, Option<String>>(4) })).collect()
    } else {
        Vec::new()
    };
    Ok(Json(json!({ "coordina": coordina, "operadores": operadores, "formularios_admitidos": e.formularios, "clasificador": clasificador_de_fotos().map(|c| c.descripcion()),
        "reglas": { "cerca_m": campo::CERCA_M, "lejos_m": campo::LEJOS_M, "confianza_minima": campo::CONFIANZA_MINIMA },
        "rutas": rutas.iter().map(|f| { let id: i64 = f.get(0); json!({ "id": id, "nombre": f.get::<_, String>(1), "operador_id": f.get::<_, i64>(2), "operador": f.get::<_, String>(3), "con_formulario": f.get::<_, bool>(4), "largo_km": f.get::<_, f32>(5), "creada": f.get::<_, String>(6), "creada_por": f.get::<_, String>(7), "demo": f.get::<_, bool>(8), "paradas": de_ruta(id) }) }).collect::<Vec<_>>() })))
}

#[derive(Deserialize)]
struct RutaNueva {
    nombre: String,
    operador_id: i64,
    #[serde(default)]
    formulario: String,
    /// Códigos de lugar poblado del INE (siete dígitos).
    lugares: Vec<String>,
}

/// Crea una ruta: valida el formulario, toma las coordenadas del catálogo y ordena las paradas.
async fn api_campo_ruta(State(e): State<Compartido>, cabeceras: HeaderMap, Json(r): Json<RutaNueva>) -> Res<Json<Value>> {
    let quien = autorizar_en(&e, &cabeceras, "operadores").await?;
    let nombre = r.nombre.trim();
    if nombre.is_empty() || nombre.chars().count() > 120 {
        return Err(pedido("la ruta necesita un nombre de hasta 120 caracteres"));
    }
    campo::formulario_admitido(r.formulario.trim(), &e.formularios).map_err(pedido)?;
    if r.lugares.is_empty() || r.lugares.len() > 80 {
        return Err(pedido("una ruta lleva entre 1 y 80 paradas"));
    }
    let existe = e.bd.query_opt("SELECT 1 FROM usuarios u LEFT JOIN grupos g ON g.id = u.grupo_id WHERE u.id = $1 AND u.activo AND (u.rol = 'operador' OR 'campo' = ANY(g.secciones))", &[&r.operador_id]).await.map_err(interno)?;
    if existe.is_none() {
        return Err(pedido("ese usuario no es un operador de campo activo"));
    }
    // Solo lugares poblados del catálogo y con coordenadas: el sistema no inventa dónde queda un lugar.
    let mut paradas: Vec<(&faro_homologa::Lugar, (f64, f64))> = Vec::new();
    for codigo in &r.lugares {
        let l = e.catalogo.por_codigo(codigo.trim()).filter(|l| l.nivel == Nivel::LugarPoblado).ok_or_else(|| pedido(format!("{codigo}: no es un lugar poblado del catálogo")))?;
        let (Some(lon), Some(lat)) = (l.longitud, l.latitud) else { return Err(pedido(format!("{} ({codigo}) no tiene coordenadas en el catálogo", l.nombre))) };
        if !paradas.iter().any(|(x, _)| x.codigo == l.codigo) {
            paradas.push((l, (lon, lat)));
        }
    }
    let puntos: Vec<(f64, f64)> = paradas.iter().map(|p| p.1).collect();
    let orden = campo::ordenar_ruta(&puntos);
    let km = campo::largo_m(&puntos, &orden) / 1000.0;
    let id: i64 = e.bd.query_one("INSERT INTO rutas_campo (nombre, operador_id, formulario, largo_km, creada_por) VALUES ($1,$2,$3,$4,$5) RETURNING id", &[&nombre, &r.operador_id, &r.formulario.trim(), &(km as f32), &quien.nombre]).await.map_err(interno)?.get(0);
    for (n, i) in orden.iter().enumerate() {
        let (l, (lon, lat)) = &paradas[*i];
        e.bd.execute("INSERT INTO paradas_campo (ruta_id, orden, lugar, nombre, longitud, latitud) VALUES ($1,$2,$3,$4,$5,$6)", &[&id, &(n as i32 + 1), &l.codigo, &l.nombre, lon, lat]).await.map_err(interno)?;
    }
    bitacora(&e, &quien.nombre, "ruta_de_campo_creada", &format!("n.º {id}: {nombre} ({} paradas)", paradas.len())).await;
    Ok(Json(json!({ "id": id, "paradas": paradas.len(), "largo_km": (km * 10.0).round() / 10.0 })))
}

async fn api_campo_ruta_borrar(State(e): State<Compartido>, Ruta(id): Ruta<i64>, cabeceras: HeaderMap) -> Res<Json<Value>> {
    let quien = autorizar_en(&e, &cabeceras, "operadores").await?;
    let n = e.bd.execute("DELETE FROM rutas_campo WHERE id = $1", &[&id]).await.map_err(interno)?;
    if n == 0 {
        return Err(pedido("no existe esa ruta"));
    }
    bitacora(&e, &quien.nombre, "ruta_de_campo_borrada", &format!("n.º {id}")).await;
    Ok(Json(json!({ "borrada": id })))
}

#[derive(Deserialize)]
struct VisitaNueva {
    parada_id: i64,
    longitud: Option<f64>,
    latitud: Option<f64>,
    precision_m: Option<f64>,
    /// Foto en JPEG o PNG, codificada en base64 (sin el prefijo «data:»). No se guarda.
    #[serde(default)]
    foto: String,
    #[serde(default)]
    notas: String,
    /// Solo en el recorrido de demostración: usar una posición simulada junto a la parada.
    #[serde(default)]
    simulada: bool,
}

/// Evidencia de una visita a partir de su posición y su foto: distancia al punto de la parada,
/// bloque de 1 km del satélite y techo del lugar según el censo; y el cruce que resulta.
fn cruce_de_visita(e: &Estado, lugar: &str, punto: (f64, f64), posicion: Option<(f64, f64)>, precision: Option<f64>, foto: Option<(String, f32)>) -> (Option<f64>, campo::Cruce) {
    let distancia = posicion.map(|pos| campo::distancia_m(pos, punto));
    // Bloque de 1 km más cercano a la posición: vale si su centro está a menos de 750 m (dentro del bloque o pegado a él).
    let bloque = posicion.and_then(|pos| e.uso_bloques.iter().map(|b| (campo::distancia_m(pos, (b.0, b.1)), b)).filter(|(d, _)| *d <= 750.0).min_by(|a, b| a.0.total_cmp(&b.0)).map(|(_, b)| *b));
    let evidencia = campo::Evidencia { distancia_m: distancia, precision_m: precision, foto, construido_ha: bloque.map(|b| b.2), no_residencial_ha: bloque.map(|b| b.3), hay_satelite: !e.uso_bloques.is_empty(), techo: e.techo_lugar.get(lugar).cloned() };
    (distancia, campo::cruzar(&evidencia))
}

/// Dirección de formulario del recorrido de demostración. Ese formulario no existe: sirve para ver
/// cómo queda el enlace con el lugar, la posición y el número de visita.
const FORMULARIO_DE_DEMOSTRACION: &str = "https://forms.office.com/r/DEMOSTRACION?lugar={lugar}&latitud={latitud}&longitud={longitud}&visita={visita}&operador={operador}";

/// Carga (o vuelve a cargar) el recorrido de demostración de quien lo pide: seis lugares poblados
/// reales de Sololá y dos visitas ficticias ya registradas, una que coincide y otra lejos del punto.
async fn api_campo_demo(State(e): State<Compartido>, cabeceras: HeaderMap) -> Res<Json<Value>> {
    let quien = operador_de_campo(&e, &cabeceras).await?;
    e.bd.execute("DELETE FROM rutas_campo WHERE demo AND operador_id = $1", &[&quien.id]).await.map_err(interno)?;
    let paradas: Vec<(&faro_homologa::Lugar, (f64, f64))> = e.catalogo.de_nivel(Nivel::LugarPoblado, Some("0701")).filter_map(|l| Some((l, (l.longitud?, l.latitud?)))).take(6).collect();
    if paradas.len() < 3 {
        return Err(pedido("este servidor no tiene el catálogo de lugares poblados de Sololá para armar la demostración"));
    }
    let puntos: Vec<(f64, f64)> = paradas.iter().map(|p| p.1).collect();
    let orden = campo::ordenar_ruta(&puntos);
    let km = campo::largo_m(&puntos, &orden) / 1000.0;
    let ruta: i64 = e.bd.query_one("INSERT INTO rutas_campo (nombre, operador_id, formulario, largo_km, creada_por, demo) VALUES ($1,$2,$3,$4,$5, TRUE) RETURNING id", &[&"Recorrido de demostración (visitas ficticias)", &quien.id, &FORMULARIO_DE_DEMOSTRACION, &(km as f32), &quien.nombre]).await.map_err(interno)?.get(0);
    for (n, i) in orden.iter().enumerate() {
        let (l, (lon, lat)) = &paradas[*i];
        let parada: i64 = e.bd.query_one("INSERT INTO paradas_campo (ruta_id, orden, lugar, nombre, longitud, latitud) VALUES ($1,$2,$3,$4,$5,$6) RETURNING id", &[&ruta, &(n as i32 + 1), &l.codigo, &l.nombre, lon, lat]).await.map_err(interno)?.get(0);
        // Dos visitas de muestra: la primera junto al punto; la segunda, a unos 17 km, para ver una visita marcada.
        let posicion = match n { 0 => Some((lon + 0.0010, lat + 0.0012)), 1 => Some((*lon, lat + 0.15)), _ => None };
        if let Some(pos) = posicion {
            let (distancia, cruce) = cruce_de_visita(&e, &l.codigo, (*lon, *lat), Some(pos), Some(15.0), None);
            e.bd.execute("INSERT INTO visitas_campo (parada_id, operador, longitud, latitud, precision_m, distancia_m, coherencia, cruce, notas) VALUES ($1,$2,$3,$4,15,$5,$6,$7,'Visita ficticia del recorrido de demostración.')",
                &[&parada, &format!("{} (demostración)", quien.nombre), &pos.0, &pos.1, &distancia.map(|d| d as f32), &cruce.coherencia, &json!({ "cruce": cruce, "foto": Value::Null })]).await.map_err(interno)?;
        }
    }
    Ok(Json(json!({ "ruta": ruta, "paradas": paradas.len() })))
}

/// Quita el recorrido de demostración de quien lo pide.
async fn api_campo_demo_borrar(State(e): State<Compartido>, cabeceras: HeaderMap) -> Res<Json<Value>> {
    let quien = operador_de_campo(&e, &cabeceras).await?;
    let n = e.bd.execute("DELETE FROM rutas_campo WHERE demo AND operador_id = $1", &[&quien.id]).await.map_err(interno)?;
    Ok(Json(json!({ "borradas": n })))
}

/// Registra una visita: posición del teléfono, foto (se clasifica y se descarta) y cruce con el
/// satélite y el censo. Devuelve el resultado y la dirección del formulario ya llena.
async fn api_campo_visita(State(e): State<Compartido>, cabeceras: HeaderMap, Json(v): Json<VisitaNueva>) -> Res<Json<Value>> {
    let quien = operador_de_campo(&e, &cabeceras).await?;
    let p = e.bd.query_opt("SELECT p.lugar, p.nombre, p.longitud, p.latitud, r.operador_id, r.formulario, r.id, r.demo FROM paradas_campo p JOIN rutas_campo r ON r.id = p.ruta_id WHERE p.id = $1", &[&v.parada_id]).await.map_err(interno)?.ok_or_else(|| pedido("no existe esa parada"))?;
    if p.get::<_, i64>(4) != quien.id && !coordina_campo(&quien) {
        return Err(Fallo(StatusCode::FORBIDDEN, "esa parada es de la ruta de otro operador".into()));
    }
    let (lugar, nombre): (String, String) = (p.get(0), p.get(1));
    let (demo, punto): (bool, (f64, f64)) = (p.get(7), (p.get(2), p.get(3)));
    let simulada = v.simulada && demo;
    // La posición vale solo si es un punto real del planeta; lo demás se toma como «sin posición».
    // En el recorrido de demostración se puede pedir una posición simulada, a unos 170 m de la parada.
    let posicion = if simulada { Some((punto.0 + 0.0010, punto.1 + 0.0012)) } else { v.longitud.zip(v.latitud).filter(|(lon, lat)| lon.is_finite() && lat.is_finite() && lon.abs() <= 180.0 && lat.abs() <= 90.0) };
    let precision = if simulada { Some(15.0) } else { v.precision_m.filter(|x| x.is_finite() && *x >= 0.0 && *x < 100_000.0) };
    let mut foto = None;
    let mut foto_detalle = Value::Null;
    if !v.foto.is_empty() {
        if v.foto.len() > 4_000_000 {
            return Err(pedido("la foto es demasiado grande: envíela reducida (el portal la reduce solo antes de enviarla)"));
        }
        let bytes = de_base64(&v.foto).ok_or_else(|| pedido("la foto no llegó bien codificada"))?;
        match clasificador_de_fotos() {
            Some(c) => {
                // La clasificación ocupa procesador: se hace fuera del hilo que atiende las demás consultas.
                // `PESADAS` limita cuántas tareas de este tipo corren a la vez.
                let _turno = PESADAS.acquire().await.map_err(interno)?;
                let r = tokio::task::spawn_blocking(move || c.clasificar(&bytes)).await.map_err(interno)?.map_err(pedido)?;
                foto = Some((r.clase.clone(), r.confianza));
                foto_detalle = json!(r);
            }
            None => foto_detalle = json!({ "aviso": "el clasificador de fotos no está instalado en este servidor: la foto no se evaluó" }),
        }
    }
    let (distancia, cruce) = cruce_de_visita(&e, &lugar, punto, posicion, precision, foto.clone());
    let notas: String = format!("{}{}", if simulada { "Posición simulada (demostración). " } else { "" }, v.notas.trim()).chars().take(500).collect();
    let fila = e
        .bd
        .query_one(
            "INSERT INTO visitas_campo (parada_id, operador, longitud, latitud, precision_m, distancia_m, foto_clase, foto_confianza, coherencia, cruce, notas) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11) RETURNING id, to_char(cuando, 'YYYY-MM-DD\"T\"HH24:MI')",
            &[&v.parada_id, &quien.nombre, &posicion.map(|x| x.0), &posicion.map(|x| x.1), &precision.map(|x| x as f32), &distancia.map(|x| x as f32), &foto.as_ref().map(|f| f.0.clone()), &foto.as_ref().map(|f| f.1), &cruce.coherencia, &json!({ "cruce": cruce, "foto": foto_detalle }), &notas],
        )
        .await
        .map_err(interno)?;
    let (id, cuando): (i64, String) = (fila.get(0), fila.get(1));
    let plantilla: String = p.get(5);
    let formulario = (!plantilla.is_empty()).then(|| campo::enlace_formulario(&plantilla, &[("visita", id.to_string()), ("lugar", lugar.clone()), ("nombre", nombre.clone()), ("municipio", lugar.get(..4).unwrap_or("").to_string()),
        ("latitud", posicion.map(|x| format!("{:.6}", x.1)).unwrap_or_default()), ("longitud", posicion.map(|x| format!("{:.6}", x.0)).unwrap_or_default()), ("precision", precision.map(|x| format!("{x:.0}")).unwrap_or_default()), ("operador", quien.usuario.clone()), ("fecha", cuando.clone())]));
    bitacora(&e, &quien.nombre, "visita_de_campo", &format!("visita n.º {id} en {nombre} ({lugar}): {}", cruce.coherencia)).await;
    Ok(Json(json!({ "id": id, "cuando": cuando, "distancia_m": distancia.map(|d| d.round()), "cruce": cruce, "foto": foto_detalle, "formulario": formulario, "simulada": simulada, "demo": demo })))
}

/// Visitas en CSV, para unirlas con las respuestas del formulario por el número de visita.
async fn api_campo_visitas_csv(State(e): State<Compartido>, cabeceras: HeaderMap) -> Res<Response> {
    autorizar_en(&e, &cabeceras, "operadores").await?;
    let filas = e.bd.query("SELECT v.id, to_char(v.cuando, 'YYYY-MM-DD HH24:MI'), r.nombre, v.operador, p.orden, p.lugar, p.nombre, v.latitud, v.longitud, v.precision_m, v.distancia_m, v.foto_clase, v.foto_confianza, v.coherencia, v.cruce->'cruce'->'dudas', v.notas, r.demo FROM visitas_campo v JOIN paradas_campo p ON p.id = v.parada_id JOIN rutas_campo r ON r.id = p.ruta_id ORDER BY v.id", &[]).await.map_err(interno)?;
    let mut w = csv::WriterBuilder::new().terminator(csv::Terminator::CRLF).from_writer(Vec::new());
    w.write_record(["visita", "cuando", "ruta", "operador", "parada", "lugar_poblado", "nombre", "latitud", "longitud", "precision_m", "distancia_al_punto_m", "foto_clase", "foto_confianza", "coherencia", "dudas", "notas", "demostracion"]).map_err(interno)?;
    let t = |x: Option<f64>, d: usize| x.map(|v| format!("{v:.d$}")).unwrap_or_default();
    // Un texto que empieza con = + - @ se abriría como fórmula en una hoja de cálculo: se neutraliza.
    let s = |t: String| if t.starts_with(['=', '+', '-', '@', '\t', '\r']) { format!("'{t}") } else { t };
    for f in &filas {
        let dudas = f.get::<_, Option<Value>>(14).and_then(|d| d.as_array().map(|a| a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join("; "))).unwrap_or_default();
        w.write_record([f.get::<_, i64>(0).to_string(), f.get(1), s(f.get(2)), s(f.get(3)), f.get::<_, i32>(4).to_string(), f.get(5), s(f.get(6)), t(f.get(7), 6), t(f.get(8), 6), t(f.get::<_, Option<f32>>(9).map(f64::from), 0), t(f.get::<_, Option<f32>>(10).map(f64::from), 0),
            f.get::<_, Option<String>>(11).unwrap_or_default(), t(f.get::<_, Option<f32>>(12).map(f64::from), 2), f.get(13), s(dudas), s(f.get(15)), if f.get::<_, bool>(16) { "sí".to_string() } else { "no".to_string() }]).map_err(interno)?;
    }
    let cuerpo = w.into_inner().map_err(interno)?;
    use axum::http::header;
    Ok(([(header::CONTENT_TYPE, "text/csv; charset=utf-8"), (header::CONTENT_DISPOSITION, "attachment; filename=\"visitas_de_campo.csv\"")], cuerpo).into_response())
}

/// Decodifica base64 estándar (con o sin relleno). Se escribe aquí para no sumar una dependencia por una sola función.
fn de_base64(texto: &str) -> Option<Vec<u8>> {
    let mut salida = Vec::with_capacity(texto.len() * 3 / 4);
    let (mut acumulado, mut bits) = (0u32, 0u32);
    for c in texto.bytes().filter(|c| !c.is_ascii_whitespace() && *c != b'=') {
        let v = match c { b'A'..=b'Z' => c - b'A', b'a'..=b'z' => c - b'a' + 26, b'0'..=b'9' => c - b'0' + 52, b'+' | b'-' => 62, b'/' | b'_' => 63, _ => return None };
        acumulado = (acumulado << 6) | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            salida.push((acumulado >> bits) as u8);
            acumulado &= (1 << bits) - 1;
        }
    }
    Some(salida)
}

/// Clasificador de fotos de fachada, si el modelo está instalado (carpeta `FARO_MODELOS`).
/// Se carga una sola vez, la primera vez que hace falta.
fn clasificador_de_fotos() -> Option<&'static faro_fuentes::fachada::Clasificador> {
    static UNICO: std::sync::OnceLock<Option<faro_fuentes::fachada::Clasificador>> = std::sync::OnceLock::new();
    UNICO
        .get_or_init(|| match faro_fuentes::fachada::Clasificador::cargar(&PathBuf::from(var("FARO_MODELOS", "datos/modelos"))) {
            Ok(c) => Some(c),
            Err(x) => {
                eprintln!("clasificador de fotos no disponible: {x}");
                None
            }
        })
        .as_ref()
}

/// Deja constancia de una acción en la bitácora. Si la escritura falla no se interrumpe la acción:
/// se avisa en el registro del servidor.
async fn bitacora(e: &Estado, quien: &str, accion: &str, detalle: &str) {
    let detalle: String = detalle.chars().take(300).collect();
    if let Err(x) = e.bd.execute("INSERT INTO bitacora (quien, accion, detalle) VALUES ($1,$2,$3)", &[&quien, &accion, &detalle]).await {
        eprintln!("bitácora: no se pudo anotar {accion}: {x}");
    }
}

/// Últimas acciones anotadas (solo administradores).
async fn api_bitacora(State(e): State<Compartido>, cabeceras: HeaderMap) -> Res<Json<Value>> {
    solo_admin(&e, &cabeceras).await?;
    let filas = e.bd.query("SELECT to_char(cuando, 'YYYY-MM-DD HH24:MI'), quien, accion, detalle FROM bitacora ORDER BY id DESC LIMIT 300", &[]).await.map_err(interno)?;
    Ok(Json(json!(filas.iter().map(|f| json!({ "cuando": f.get::<_, String>(0), "quien": f.get::<_, String>(1), "accion": f.get::<_, String>(2), "detalle": f.get::<_, String>(3) })).collect::<Vec<_>>())))
}

/// Indicadores que un tablero puede usar, con su unidad (los lee el diseñador).
fn indicadores_de_tablero() -> Vec<Value> {
    tablero::INDICADORES.iter().map(|i| json!({ "id": i.0, "nombre": i.1, "unidad": i.2 })).collect()
}

/// Lista de tableros: los publicados para cualquiera y, con sesión, además los borradores propios
/// (un administrador ve todos).
async fn api_tableros(State(e): State<Compartido>, cabeceras: HeaderMap) -> Res<Json<Value>> {
    let quien = sesion_de(&e, &cabeceras).await.ok().flatten();
    let (nombre, admin) = quien.as_ref().map(|s| (s.nombre.clone(), s.rol == "admin")).unwrap_or_default();
    let filas = e
        .bd
        .query(
            "SELECT id, titulo, solicitud, autor, to_char(cambiado, 'YYYY-MM-DD HH24:MI'), jsonb_array_length(definicion->'piezas'), definicion->'territorio'->>'nombre', publico, coalesce(definicion->>'descripcion', '')
             FROM tableros WHERE publico OR autor = $1 OR $2 ORDER BY cambiado DESC LIMIT 300",
            &[&nombre, &admin],
        )
        .await
        .map_err(interno)?;
    Ok(Json(json!({ "indicadores": indicadores_de_tablero(), "tipos": tablero::TIPOS,
        "tableros": filas.iter().map(|f| { let autor: String = f.get(3); json!({ "id": f.get::<_, i64>(0), "titulo": f.get::<_, String>(1), "solicitud": f.get::<_, String>(2), "cambiado": f.get::<_, String>(4), "piezas": f.get::<_, Option<i32>>(5), "territorio": f.get::<_, Option<String>>(6),
            "publico": f.get::<_, bool>(7), "descripcion": f.get::<_, String>(8), "editable": quien.is_some() && (admin || autor == nombre), "autor": autor }) }).collect::<Vec<_>>() })))
}

/// Un tablero: si está publicado lo lee cualquiera; un borrador, solo su autor o un administrador.
async fn api_tablero(State(e): State<Compartido>, Ruta(id): Ruta<i64>, cabeceras: HeaderMap) -> Res<Json<Value>> {
    let quien = sesion_de(&e, &cabeceras).await.ok().flatten();
    let (nombre, admin) = quien.as_ref().map(|s| (s.nombre.clone(), s.rol == "admin")).unwrap_or_default();
    let f = e.bd.query_opt("SELECT id, titulo, solicitud, definicion, autor, publico FROM tableros WHERE id = $1 AND (publico OR autor = $2 OR $3)", &[&id, &nombre, &admin]).await.map_err(interno)?.ok_or_else(|| pedido("no existe ese tablero o todavía no está publicado"))?;
    let autor: String = f.get(4);
    Ok(Json(json!({ "id": f.get::<_, i64>(0), "titulo": f.get::<_, String>(1), "solicitud": f.get::<_, String>(2), "definicion": f.get::<_, Value>(3), "publico": f.get::<_, bool>(5),
        "editable": quien.is_some() && (admin || autor == nombre), "autor": autor, "indicadores": indicadores_de_tablero() })))
}

#[derive(Deserialize)]
struct Publicacion {
    publico: bool,
}

/// Publica un tablero (pasa a verse en el portal sin cuenta) o lo devuelve a borrador.
async fn api_tablero_publicar(State(e): State<Compartido>, Ruta(id): Ruta<i64>, cabeceras: HeaderMap, Json(p): Json<Publicacion>) -> Res<Json<Value>> {
    let quien = autorizar_en(&e, &cabeceras, "tableros").await?;
    let n = e.bd.execute("UPDATE tableros SET publico = $1, cambiado = now() WHERE id = $2 AND (autor = $3 OR $4)", &[&p.publico, &id, &quien.nombre, &(quien.rol == "admin")]).await.map_err(interno)?;
    if n == 0 { return Err(Fallo(StatusCode::FORBIDDEN, "solo su autor o un administrador puede publicar o retirar este tablero".into())); }
    bitacora(&e, &quien.nombre, if p.publico { "tablero_publicado" } else { "tablero_retirado" }, &id.to_string()).await;
    Ok(Json(json!({ "id": id, "publico": p.publico })))
}

#[derive(Deserialize)]
struct TableroDatos {
    #[serde(default)]
    solicitud: String,
    definicion: tablero::Tablero,
}

async fn api_tablero_guardar(State(e): State<Compartido>, cabeceras: HeaderMap, Json(t): Json<TableroDatos>) -> Res<Json<Value>> {
    let quien = autorizar_en(&e, &cabeceras, "tableros").await?;
    tablero::validar(&t.definicion, &e.catalogo).map_err(pedido)?;
    let id: i64 = e.bd.query_one("INSERT INTO tableros (titulo, solicitud, definicion, autor) VALUES ($1,$2,$3,$4) RETURNING id", &[&t.definicion.titulo.trim(), &t.solicitud.chars().take(500).collect::<String>(), &json!(t.definicion), &quien.nombre]).await.map_err(interno)?.get(0);
    Ok(Json(json!({ "id": id })))
}

async fn api_tablero_guardar_id(State(e): State<Compartido>, Ruta(id): Ruta<i64>, cabeceras: HeaderMap, Json(t): Json<TableroDatos>) -> Res<Json<Value>> {
    let quien = autorizar_en(&e, &cabeceras, "tableros").await?;
    tablero::validar(&t.definicion, &e.catalogo).map_err(pedido)?;
    // Lo cambia su autor o un administrador.
    let n = e.bd.execute("UPDATE tableros SET titulo = $1, definicion = $2, cambiado = now() WHERE id = $3 AND (autor = $4 OR $5)", &[&t.definicion.titulo.trim(), &json!(t.definicion), &id, &quien.nombre, &(quien.rol == "admin")]).await.map_err(interno)?;
    if n == 0 { return Err(Fallo(StatusCode::FORBIDDEN, "solo su autor o un administrador puede cambiar este tablero".into())); }
    Ok(Json(json!({ "id": id })))
}

async fn api_tablero_borrar(State(e): State<Compartido>, Ruta(id): Ruta<i64>, cabeceras: HeaderMap) -> Res<Json<Value>> {
    let quien = autorizar_en(&e, &cabeceras, "tableros").await?;
    let n = e.bd.execute("DELETE FROM tableros WHERE id = $1 AND (autor = $2 OR $3)", &[&id, &quien.nombre, &(quien.rol == "admin")]).await.map_err(interno)?;
    if n == 0 { return Err(Fallo(StatusCode::FORBIDDEN, "solo su autor o un administrador puede borrar este tablero".into())); }
    Ok(Json(json!({ "borrado": true })))
}
