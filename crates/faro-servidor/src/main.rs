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
use faro_homologa::{Catalogo, Equivalencia, Homologador, Memoria, Nivel};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::Arc;
use tokio_postgres::{Client, NoTls};

struct Estado {
    bd: Client,
    catalogo: Catalogo,
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

    let estado = Arc::new(Estado {
        bd,
        catalogo: Catalogo::cargar_carpeta(&PathBuf::from(var("FARO_CATALOGO", "datos/catalogo")))?,
        dir_solicitudes: PathBuf::from(var("FARO_SOLICITUDES", "datos/solicitudes")),
        dir_privado: PathBuf::from(var("FARO_PRIVADO", "datos/privado")),
        token: std::env::var("FARO_TOKEN").ok().filter(|t| !t.is_empty()),
        sal: std::env::var("FARO_SAL").ok().filter(|t| !t.is_empty()),
        portal: var("FARO_PORTAL_INE", faro_fuentes::INE_CKAN),
    });
    if estado.token.is_none() {
        eprintln!("AVISO: FARO_TOKEN no está definido; cualquiera con acceso a la red puede enviar y confirmar.");
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
fn autorizar(e: &Estado, cabeceras: &HeaderMap) -> Res<()> {
    match &e.token {
        None => Ok(()),
        Some(t) if cabeceras.get("x-faro-token").and_then(|v| v.to_str().ok()) == Some(t.as_str()) => Ok(()),
        Some(_) => Err(Fallo(StatusCode::UNAUTHORIZED, "token inválido o ausente".into())),
    }
}

async fn memoria(e: &Estado) -> Res<Memoria> {
    let filas = e.bd.query("SELECT original, clave, nivel, contexto, codigo FROM equivalencias ORDER BY id", &[]).await.map_err(interno)?;
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
    })))
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
                    (SELECT count(*) FROM pendientes WHERE NOT resuelto), (SELECT count(*) FROM equivalencias),
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
    Ok(Json(json!(e.catalogo.de_nivel(nivel, padre).map(|l| json!({"codigo": l.codigo, "nombre": l.nombre, "padre": l.padre})).collect::<Vec<_>>())))
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
    autorizar(&e, &cabeceras)?;
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
        json!({
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
    nivel: Nivel,
    #[serde(default)]
    contexto: String,
    codigo: String,
    #[serde(default)]
    institucion: String,
    #[serde(default)]
    usuario: String,
}

async fn api_confirmar(State(e): State<Compartido>, cabeceras: HeaderMap, Json(c): Json<Confirmacion>) -> Res<Json<Value>> {
    autorizar(&e, &cabeceras)?;
    let lugar = e.catalogo.por_codigo(&c.codigo).ok_or_else(|| pedido(format!("el código {} no existe en el catálogo", c.codigo)))?;
    if lugar.nivel != c.nivel {
        return Err(pedido(format!("{} ({}) no es del nivel indicado", c.codigo, lugar.nombre)));
    }
    if c.usuario.trim().is_empty() {
        return Err(pedido("indique quién confirma: queda en la bitácora"));
    }
    let nivel = serde_json::to_value(c.nivel).ok().and_then(|v| v.as_str().map(str::to_string)).unwrap_or_default();
    let clave = faro_homologa::normalizar::clave(&c.valor);
    e.bd.execute(
        "INSERT INTO equivalencias (original, clave, nivel, contexto, codigo, institucion, confirmado_por) VALUES ($1,$2,$3,$4,$5,$6,$7)",
        &[&c.valor, &clave, &nivel, &c.contexto, &c.codigo, &c.institucion, &c.usuario],
    )
    .await
    .map_err(interno)?;
    let n = e.bd.execute("UPDATE pendientes SET resuelto = TRUE WHERE NOT resuelto AND nivel = $1 AND contexto = $2 AND valor = $3", &[&nivel, &c.contexto, &c.valor]).await.map_err(interno)?;
    Ok(Json(json!({ "grabado": true, "codigo": lugar.codigo, "nombre": lugar.nombre, "pendientes_resueltos": n })))
}

async fn api_equivalencias(State(e): State<Compartido>) -> Res<Json<Value>> {
    let filas = e.bd.query("SELECT original, nivel, contexto, codigo, institucion, confirmado_por, to_char(fecha, 'YYYY-MM-DD HH24:MI') FROM equivalencias ORDER BY id DESC LIMIT 500", &[]).await.map_err(interno)?;
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
             SELECT h.solicitud, left(h.municipio, 2), sum(h.n)::bigint, sum(h.n) FILTER (WHERE h.sexo = 'hombre')::bigint, sum(h.n) FILTER (WHERE h.sexo = 'mujer')::bigint
             FROM hechos h JOIN ultimo u USING (solicitud, periodo, envio_id) GROUP BY 1, 2 ORDER BY 1, 2",
            &[],
        )
        .await
        .map_err(interno)?;
    Ok(Json(json!(filas.iter().map(|f| {
        let d: String = f.get(1);
        json!({ "solicitud": f.get::<_, String>(0), "departamento": d, "nombre": e.catalogo.por_codigo(&d).map(|l| l.nombre.clone()).unwrap_or_else(|| "(sin lugar)".into()),
                "total": f.get::<_, i64>(2), "hombres": f.get::<_, Option<i64>>(3), "mujeres": f.get::<_, Option<i64>>(4) })
    }).collect::<Vec<_>>())))
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
    autorizar(&e, &cabeceras)?;
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
    autorizar(&e, &cabeceras)?;
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
    autorizar(&e, &cabeceras)?;
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
