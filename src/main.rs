#![windows_subsystem = "windows"]
mod desktop;
mod platform;
mod settings;
mod webview;
use axum::{
    extract::{Path, Query, Request, State},
    http::{header, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    os::windows::process::CommandExt,
    sync::{Arc, RwLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use sysinfo::{ProcessesToUpdate, System};
#[derive(Clone)]
struct App {
    snapshot: Arc<RwLock<Value>>,
    sensors: Arc<RwLock<Value>>,
    extended: Arc<RwLock<Value>>,
    token: Arc<String>,
    authority: Arc<String>,
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
async fn guard(State(a): State<App>, req: Request, next: Next) -> Response {
    let host = req
        .headers()
        .get(header::HOST)
        .and_then(|v| v.to_str().ok());
    let expected = format!("http://{}", a.authority);
    let origin = req
        .headers()
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok());
    if host != Some(a.authority.as_str()) || origin.is_some_and(|o| o != expected) {
        return StatusCode::FORBIDDEN.into_response();
    }
    if req.method() != axum::http::Method::GET
        && req
            .headers()
            .get("x-corescope-token")
            .and_then(|v| v.to_str().ok())
            != Some(a.token.as_str())
    {
        return StatusCode::FORBIDDEN.into_response();
    }
    let mut response = next.run(req).await;
    let h = response.headers_mut();
    h.insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    h.insert(header::X_CONTENT_TYPE_OPTIONS, "nosniff".parse().unwrap());
    h.insert(header::CONTENT_SECURITY_POLICY,"default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; frame-ancestors 'none'; base-uri 'none'; form-action 'none'".parse().unwrap());
    response
}
async fn index(State(a): State<App>) -> Html<String> {
    Html(
        include_str!("../assets/index.html")
            .replace("__TOKEN__", &a.token)
            .replace("/*__I18N__*/", include_str!("../assets/i18n.js")),
    )
}
async fn snapshot(State(a): State<App>) -> Json<Value> {
    let mut s = a.snapshot.read().unwrap().clone();
    s["telemetry"] = a.sensors.read().unwrap().clone();
    let ext = a.extended.read().unwrap().clone();
    let mut list = s["telemetry"]["sensors"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    list.extend(ext["sensors"].as_array().cloned().unwrap_or_default());
    s["telemetry"]["sensors"] = json!(list);
    s["telemetry"]["extended"] = ext;
    s["settings"] = json!(settings::get());
    Json(s)
}
type ApiResult = Result<Json<Value>, (StatusCode, Json<Value>)>;
fn result(r: Result<Value, String>) -> ApiResult {
    r.map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"error":e}))))
}
#[derive(Deserialize)]
struct Affinity {
    identity: String,
    cores: Vec<usize>,
}
async fn affinity(Path(pid): Path<u32>, Json(body): Json<Affinity>) -> ApiResult {
    result(platform::affinity(pid, &body.identity, &body.cores))
}
#[derive(Deserialize)]
struct CoreAccess {
    identity: String,
    core: usize,
    enabled: bool,
}
async fn core_access(Path(pid): Path<u32>, Json(body): Json<CoreAccess>) -> ApiResult {
    result(platform::core_access(
        pid,
        &body.identity,
        body.core,
        body.enabled,
    ))
}
#[derive(Deserialize)]
struct MemoryQuery {
    identity: String,
}
async fn memory(Path(pid): Path<u32>, Query(q): Query<MemoryQuery>) -> ApiResult {
    result(
        tokio::task::spawn_blocking(move || platform::memory(pid, &q.identity))
            .await
            .unwrap_or_else(|e| Err(e.to_string())),
    )
}
#[derive(Deserialize)]
struct Launch {
    path: String,
    cores: Vec<usize>,
}
async fn launch(Json(b): Json<Launch>) -> ApiResult {
    result(platform::launch(&b.path, &b.cores))
}
async fn save_settings(Json(b): Json<settings::Settings>) -> ApiResult {
    result(settings::save(b).map(|s| json!(s)))
}
async fn start_sensors() -> ApiResult {
    result(settings::start_sensors().map(|_| json!({"ok":true})))
}
fn collect(a: App) {
    let mut system = System::new_all();
    loop {
        std::thread::sleep(Duration::from_secs(1));
        system.refresh_cpu_all();
        system.refresh_memory();
        system.refresh_processes(ProcessesToUpdate::All, true);
        let cpus: Vec<Value> = system
            .cpus()
            .iter()
            .enumerate()
            .map(|(id, c)| json!({"id":id,"usage":c.cpu_usage(),"mhz":c.frequency()}))
            .collect();
        let mut processes: Vec<Value> = system
            .processes()
            .iter()
            .map(|(pid, p)| {
                let mut v = platform::summary(pid.as_u32());
                v["pid"] = json!(pid.as_u32());
                v["name"] = json!(p.name().to_string_lossy());
                v["cpu"] = json!(p.cpu_usage() / cpus.len().max(1) as f32);
                v["memory"] = json!(p.memory());
                v["virtual_memory"] = json!(p.virtual_memory());
                v
            })
            .collect();
        processes.sort_by(|a, b| {
            b["cpu"]
                .as_f64()
                .unwrap_or_default()
                .total_cmp(&a["cpu"].as_f64().unwrap_or_default())
        });
        *a.snapshot.write().unwrap() = json!({"ready":true,"time":now(),"host":System::host_name(),"cpu_name":system.cpus().first().map(|c|c.brand()),"cpu_usage":system.global_cpu_usage(),"cpus":cpus,"processes":processes,"memory_total":system.total_memory(),"memory_used":system.used_memory(),"affinity_supported":platform::groups_supported()});
    }
}
fn sensors(a: App, extended: bool) {
    let mut history: std::collections::HashMap<String, Vec<(u64, f64)>> =
        std::collections::HashMap::new();
    loop {
        if extended && !settings::get().extended_sensors {
            *a.extended.write().unwrap() = json!({"sensors":[],"provider":"disabled"});
            std::thread::sleep(Duration::from_secs(1));
            continue;
        }
        let mut command = std::process::Command::new("powershell.exe");
        command
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                include_str!("../assets/sensors.ps1"),
            ])
            .env(
                "CORESCOPE_SENSOR_MODE",
                if extended { "extended" } else { "windows" },
            )
            .env(
                "CORESCOPE_HARDWARE_DIR",
                std::env::current_exe()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .join("hardware"),
            )
            .creation_flags(0x08000000)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null());
        let value = match command.spawn() {
            Ok(mut child) => {
                let stdout = child.stdout.take().unwrap();
                let reader = std::thread::spawn(move || {
                    use std::io::Read;
                    let mut bytes = Vec::new();
                    let _ = stdout.take(4 * 1024 * 1024).read_to_end(&mut bytes);
                    bytes
                });
                let start = std::time::Instant::now();
                loop {
                    match child.try_wait() {
                        Ok(Some(_)) => break,
                        Err(_) => {
                            let _ = child.kill();
                            break;
                        }
                        _ if start.elapsed() > Duration::from_secs(12) => {
                            let _ = child.kill();
                            break;
                        }
                        _ => std::thread::sleep(Duration::from_millis(100)),
                    }
                }
                let _ = child.wait();
                reader
                    .join()
                    .ok()
                    .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
            }
            Err(_) => None,
        };
        let mut completed = match value {
            Some(mut v) => {
                v["time"] = json!(now());
                if let Some(list) = v["sensors"].as_array_mut() {
                    for s in list {
                        if s["kind"] == "Temperature" {
                            if let (Some(id), Some(value)) =
                                (s["identifier"].as_str(), s["value"].as_f64())
                            {
                                let h = history.entry(id.to_owned()).or_default();
                                h.push((now(), value));
                                if h.len() > 120 {
                                    h.remove(0);
                                }
                                s["history"] = json!(h);
                            }
                        }
                    }
                }
                v
            }
            None => {
                json!({"provider":null,"sensors":[],"time":now(),"error":"Не удалось прочитать провайдер датчиков"})
            }
        };
        if let Some(list) = completed["sensors"].as_array_mut() {
            for s in list {
                s["sample_time"] = json!(now());
                s["source"] = json!(if extended {
                    "LibreHardwareMonitorLib"
                } else {
                    "Windows"
                });
            }
        }
        if extended {
            *a.extended.write().unwrap() = completed;
        } else {
            *a.sensors.write().unwrap() = completed;
        }
        std::thread::sleep(Duration::from_secs(5));
    }
}
async fn run_server(
    ready: Option<std::sync::mpsc::Sender<String>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let port = args
        .windows(2)
        .find(|p| p[0] == "--port")
        .map(|p| p[1].parse::<u16>())
        .transpose()?
        .unwrap_or(if ready.is_some() { 0 } else { 8899 });
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await?;
    let authority = listener.local_addr()?.to_string();
    let mut random = [0u8; 32];
    getrandom::fill(&mut random).map_err(|e| e.to_string())?;
    let a = App {
        snapshot: Arc::new(RwLock::new(json!({"ready":false}))),
        sensors: Arc::new(RwLock::new(json!({"sensors":[],"provider":null}))),
        extended: Arc::new(RwLock::new(json!({"sensors":[],"provider":null}))),
        token: Arc::new(random.iter().map(|b| format!("{b:02x}")).collect()),
        authority: Arc::new(authority.clone()),
    };
    let c = a.clone();
    std::thread::spawn(move || collect(c));
    let c = a.clone();
    std::thread::spawn(move || sensors(c, false));
    let c = a.clone();
    std::thread::spawn(move || sensors(c, true));
    let app = Router::new()
        .route("/", get(index))
        .route("/api/snapshot", get(snapshot))
        .route("/api/process/{pid}/memory", get(memory))
        .route("/api/process/{pid}/affinity", post(affinity))
        .route("/api/process/{pid}/core", post(core_access))
        .route("/api/launch", post(launch))
        .route("/api/settings", post(save_settings))
        .route("/api/sensors/start", post(start_sensors))
        .layer(middleware::from_fn_with_state(a.clone(), guard))
        .with_state(a);
    println!("CoreScope: http://{authority} — Ctrl+C to stop");
    if let Some(ready) = ready {
        let _ = ready.send(format!("http://{authority}"));
    }
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}

fn main() {
    let headless = std::env::args().any(|a| a == "--no-open");
    if !headless && desktop::activate_existing() {
        return;
    }
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    if headless {
        if let Err(e) = runtime.block_on(run_server(None)) {
            eprintln!("{e}");
            std::process::exit(1)
        }
        return;
    }
    let (send, receive) = std::sync::mpsc::channel();
    runtime.spawn(async move {
        if let Err(e) = run_server(Some(send)).await {
            desktop::error(&e.to_string());
        }
    });
    match receive.recv_timeout(Duration::from_secs(15)) {
        Ok(url) => {
            if let Err(e) = desktop::run(url) {
                desktop::error(&e)
            }
        }
        Err(e) => desktop::error(&format!("CoreScope could not start: {e}")),
    }
    runtime.shutdown_timeout(Duration::from_secs(2));
}
