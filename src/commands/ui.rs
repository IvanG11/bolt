use crate::commands::{list, status, switch};
use crate::config::Config;
use anyhow::Result;
use axum::{
    Router,
    body::Body,
    extract::{Path, Query, State},
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    http::{Uri, header},
    response::{
        IntoResponse, Json, Response,
        sse::{Event, KeepAlive, Sse},
    },
    routing::{delete, get, post},
};
use futures_util::{SinkExt, StreamExt};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

#[derive(RustEmbed)]
#[folder = "ui/dist/"]
struct Assets;

type SharedState = Arc<AppState>;

struct AppState {
    config: tokio::sync::RwLock<Config>,
}

#[derive(Serialize)]
struct ApiResult {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

pub fn run(config: Config, port: u16) -> Result<()> {
    tokio::runtime::Runtime::new()?.block_on(serve(config, port))
}

async fn serve(config: Config, port: u16) -> Result<()> {
    let state = Arc::new(AppState {
        config: tokio::sync::RwLock::new(config),
    });

    let app = Router::new()
        .route("/api/projects", get(api_list_projects))
        .route("/api/projects/stop-all", post(api_stop_all_projects))
        .route("/api/containers", get(api_list_containers))
        .route("/api/projects/{name}/start", post(api_start_project))
        .route("/api/projects/{name}/stop", post(api_stop_project))
        .route("/api/projects/{name}/restart", post(api_restart_project))
        .route("/api/projects/{name}/logs", get(api_project_logs))
        .route("/api/projects/{name}/info", get(api_project_info))
        .route("/api/projects/{name}/subdirs/{subdir}/start", post(api_start_subdir))
        .route("/api/projects/{name}/subdirs/{subdir}/stop", post(api_stop_subdir))
        .route("/api/projects/{name}/subdirs/{subdir}/restart", post(api_restart_subdir))
        .route("/api/config", get(api_get_config))
        .route("/api/config/dir", post(api_set_dir))
        .route("/api/config/ignore", post(api_add_ignore))
        .route("/api/config/ignore/{project}", delete(api_remove_ignore))
        .route(
            "/api/config/projects/{name}/subdirs",
            post(api_set_subdirs).delete(api_clear_subdirs),
        )
        .route("/api/projects/{name}/build", get(api_build_project))
        .route("/api/projects/{name}/services", get(api_project_services))
        .route("/api/projects/{name}/shell", get(api_shell))
        .fallback(static_handler)
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    let url = format!("http://{}", addr);

    println!("Bolt UI → {}", url);
    println!("Press Ctrl+C to stop.");

    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        tokio::task::spawn_blocking(move || { let _ = open::that(url); });
    });

    axum::serve(listener, app).await?;
    Ok(())
}

async fn static_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match Assets::get(path) {
        Some(content) => (
            [(header::CONTENT_TYPE, mime_for_path(path))],
            Body::from(content.data),
        )
            .into_response(),
        None => match Assets::get("index.html") {
            Some(index) => (
                [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                Body::from(index.data),
            )
                .into_response(),
            None => (axum::http::StatusCode::NOT_FOUND, "Not found").into_response(),
        },
    }
}

fn mime_for_path(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "application/javascript",
        "css" => "text/css",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "json" => "application/json",
        _ => "application/octet-stream",
    }
}

// ── Project handlers ──────────────────────────────────────────────────

async fn api_list_projects(State(state): State<SharedState>) -> Json<serde_json::Value> {
    let config = state.config.read().await.clone();
    match list::list_with_status(&config) {
        Ok(projects) => Json(serde_json::json!({ "projects": projects })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_stop_all_projects(State(state): State<SharedState>) -> Json<ApiResult> {
    let config = state.config.read().await.clone();
    match tokio::task::spawn_blocking(move || switch::stop_all(&config)).await {
        Ok(Ok(_)) => Json(ApiResult { ok: true, error: None }),
        Ok(Err(e)) => Json(ApiResult { ok: false, error: Some(e.to_string()) }),
        Err(e) => Json(ApiResult { ok: false, error: Some(e.to_string()) }),
    }
}

async fn api_list_containers() -> Json<serde_json::Value> {
    match status::get_containers() {
        Ok(containers) => Json(serde_json::json!({ "containers": containers })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_start_project(
    Path(name): Path<String>,
    State(state): State<SharedState>,
) -> Json<ApiResult> {
    let config = state.config.read().await.clone();
    match tokio::task::spawn_blocking(move || switch::start(&name, &config)).await {
        Ok(Ok(_)) => Json(ApiResult { ok: true, error: None }),
        Ok(Err(e)) => Json(ApiResult { ok: false, error: Some(e.to_string()) }),
        Err(e) => Json(ApiResult { ok: false, error: Some(e.to_string()) }),
    }
}

async fn api_stop_project(
    Path(name): Path<String>,
    State(state): State<SharedState>,
) -> Json<ApiResult> {
    let config = state.config.read().await.clone();
    match tokio::task::spawn_blocking(move || switch::stop(&name, &config)).await {
        Ok(Ok(_)) => Json(ApiResult { ok: true, error: None }),
        Ok(Err(e)) => Json(ApiResult { ok: false, error: Some(e.to_string()) }),
        Err(e) => Json(ApiResult { ok: false, error: Some(e.to_string()) }),
    }
}

async fn api_restart_project(
    Path(name): Path<String>,
    State(state): State<SharedState>,
) -> Json<ApiResult> {
    let config = state.config.read().await.clone();
    match tokio::task::spawn_blocking(move || switch::restart(&name, &config)).await {
        Ok(Ok(_)) => Json(ApiResult { ok: true, error: None }),
        Ok(Err(e)) => Json(ApiResult { ok: false, error: Some(e.to_string()) }),
        Err(e) => Json(ApiResult { ok: false, error: Some(e.to_string()) }),
    }
}

async fn api_start_subdir(
    Path((name, subdir)): Path<(String, String)>,
    State(state): State<SharedState>,
) -> Json<ApiResult> {
    let config = state.config.read().await.clone();
    match tokio::task::spawn_blocking(move || switch::start_subdir(&name, &subdir, &config)).await {
        Ok(Ok(_)) => Json(ApiResult { ok: true, error: None }),
        Ok(Err(e)) => Json(ApiResult { ok: false, error: Some(e.to_string()) }),
        Err(e) => Json(ApiResult { ok: false, error: Some(e.to_string()) }),
    }
}

async fn api_stop_subdir(
    Path((name, subdir)): Path<(String, String)>,
    State(state): State<SharedState>,
) -> Json<ApiResult> {
    let config = state.config.read().await.clone();
    match tokio::task::spawn_blocking(move || switch::stop_subdir(&name, &subdir, &config)).await {
        Ok(Ok(_)) => Json(ApiResult { ok: true, error: None }),
        Ok(Err(e)) => Json(ApiResult { ok: false, error: Some(e.to_string()) }),
        Err(e) => Json(ApiResult { ok: false, error: Some(e.to_string()) }),
    }
}

async fn api_restart_subdir(
    Path((name, subdir)): Path<(String, String)>,
    State(state): State<SharedState>,
) -> Json<ApiResult> {
    let config = state.config.read().await.clone();
    match tokio::task::spawn_blocking(move || switch::restart_subdir(&name, &subdir, &config)).await {
        Ok(Ok(_)) => Json(ApiResult { ok: true, error: None }),
        Ok(Err(e)) => Json(ApiResult { ok: false, error: Some(e.to_string()) }),
        Err(e) => Json(ApiResult { ok: false, error: Some(e.to_string()) }),
    }
}

// ── Build (SSE stream) ────────────────────────────────────────────────

async fn api_build_project(
    Path(name): Path<String>,
    State(state): State<SharedState>,
) -> Sse<impl futures_core::Stream<Item = Result<Event, std::convert::Infallible>>> {
    use tokio::io::{AsyncBufReadExt, BufReader};

    let (tx, mut rx) = tokio::sync::mpsc::channel::<Event>(256);
    let config = state.config.read().await.clone();
    let project_dir = config.projects_dir.join(&name);
    let compose_files = find_compose_files(&project_dir, &name, &config);
    drop(config);

    tokio::spawn(async move {
        if compose_files.is_empty() {
            let _ = tx.send(Event::default().event("done").data("error: no compose files found")).await;
            return;
        }

        for compose_file in compose_files {
            let tx = tx.clone();
            let Ok(mut child) = tokio::process::Command::new("docker")
                .args(["compose", "-f"])
                .arg(&compose_file)
                .args(["build", "--progress=plain"])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
            else {
                let _ = tx.send(Event::default().event("done").data("error: failed to spawn docker")).await;
                return;
            };

            let stdout = child.stdout.take().unwrap();
            let stderr = child.stderr.take().unwrap();
            let tx_out = tx.clone();
            let tx_err = tx.clone();

            let out_task = tokio::spawn(async move {
                let mut lines = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    if tx_out.send(Event::default().data(line)).await.is_err() { break; }
                }
            });
            let err_task = tokio::spawn(async move {
                let mut lines = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    if tx_err.send(Event::default().data(line)).await.is_err() { break; }
                }
            });

            let _ = tokio::join!(out_task, err_task);
            let status = child.wait().await.map(|s| s.success()).unwrap_or(false);
            if !status {
                let _ = tx.send(Event::default().event("done").data("error")).await;
                return;
            }
        }

        let _ = tx.send(Event::default().event("done").data("ok")).await;
    });

    let stream = async_stream::stream! {
        while let Some(event) = rx.recv().await {
            yield Ok::<Event, std::convert::Infallible>(event);
        }
    };

    Sse::new(stream)
}

// ── Project info (path + ports) ───────────────────────────────────────

#[derive(Serialize)]
struct PortEntry {
    service: String,
    host_port: u16,
    container_port: u16,
    protocol: String,
}

async fn api_project_info(
    Path(name): Path<String>,
    State(state): State<SharedState>,
) -> Json<serde_json::Value> {
    let config = state.config.read().await.clone();
    let project_dir = config.projects_dir.join(&name);
    let path_str = project_dir.to_string_lossy().to_string();
    let compose_files = find_compose_files(&project_dir, &name, &config);
    drop(config);

    let mut seen = std::collections::HashSet::new();
    let mut ports: Vec<PortEntry> = Vec::new();

    for compose_file in &compose_files {
        let Ok(output) = tokio::process::Command::new("docker")
            .args(["compose", "-f"])
            .arg(compose_file)
            .args(["ps", "--format", "json"])
            .output()
            .await
        else {
            continue;
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        for container in parse_compose_ps(&stdout) {
            let service = container["Service"].as_str().unwrap_or("").to_string();
            if let Some(publishers) = container["Publishers"].as_array() {
                for pub_entry in publishers {
                    let host_port = pub_entry["PublishedPort"].as_u64().unwrap_or(0) as u16;
                    let container_port = pub_entry["TargetPort"].as_u64().unwrap_or(0) as u16;
                    let protocol = pub_entry["Protocol"].as_str().unwrap_or("tcp").to_string();
                    if host_port > 0 && seen.insert((host_port, container_port, protocol.clone(), service.clone())) {
                        ports.push(PortEntry { service: service.clone(), host_port, container_port, protocol });
                    }
                }
            }
        }
    }

    Json(serde_json::json!({ "path": path_str, "ports": ports }))
}

fn parse_compose_ps(stdout: &str) -> Vec<serde_json::Value> {
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return vec![];
    }
    if trimmed.starts_with('[') {
        serde_json::from_str(trimmed).unwrap_or_default()
    } else {
        // NDJSON: one object per line (older Docker Compose)
        trimmed.lines().filter_map(|l| serde_json::from_str(l).ok()).collect()
    }
}

// ── Logs ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct LogsQuery {
    subdir: Option<String>,
}

async fn api_project_logs(
    Path(name): Path<String>,
    Query(query): Query<LogsQuery>,
    State(state): State<SharedState>,
) -> Sse<impl futures_core::Stream<Item = Result<Event, Infallible>>> {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(256);

    let config = state.config.read().await.clone();
    let project_dir = config.projects_dir.join(&name);
    let all_files = find_compose_files(&project_dir, &name, &config);

    let compose_files = match query.subdir {
        Some(ref subdir) => all_files
            .into_iter()
            .filter(|p| {
                p.parent()
                    .and_then(|d| d.file_name())
                    .map(|n| n == subdir.as_str())
                    .unwrap_or(false)
            })
            .collect(),
        None => all_files,
    };

    for compose_file in compose_files {
        let tx = tx.clone();
        tokio::spawn(stream_compose_logs(tx, compose_file));
    }
    drop(tx);

    let stream = async_stream::stream! {
        while let Some(line) = rx.recv().await {
            yield Ok::<Event, Infallible>(Event::default().data(line));
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn stream_compose_logs(tx: tokio::sync::mpsc::Sender<String>, compose_file: PathBuf) {
    use tokio::io::{AsyncBufReadExt, BufReader};

    let Ok(mut child) = tokio::process::Command::new("docker")
        .args(["compose", "-f"])
        .arg(&compose_file)
        .args(["logs", "--follow", "--tail=100", "--no-color"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    else {
        return;
    };

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let tx_out = tx.clone();
    tokio::spawn(async move {
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if tx_out.send(line).await.is_err() { break; }
        }
    });

    let mut lines = BufReader::new(stderr).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        if tx.send(line).await.is_err() { break; }
    }
}

// ── Config handlers ───────────────────────────────────────────────────

async fn api_get_config(State(state): State<SharedState>) -> Json<serde_json::Value> {
    let config = state.config.read().await;
    Json(serde_json::json!({
        "projects_dir": config.projects_dir.to_string_lossy(),
        "ignore": config.ignore,
        "projects": config.projects,
    }))
}

#[derive(Deserialize)]
struct SetDirBody { path: String }

async fn api_set_dir(
    State(state): State<SharedState>,
    Json(body): Json<SetDirBody>,
) -> Json<ApiResult> {
    let mut config = state.config.write().await;
    config.projects_dir = PathBuf::from(&body.path);
    match config.save() {
        Ok(_) => Json(ApiResult { ok: true, error: None }),
        Err(e) => Json(ApiResult { ok: false, error: Some(e.to_string()) }),
    }
}

#[derive(Deserialize)]
struct IgnoreBody { project: String }

async fn api_add_ignore(
    State(state): State<SharedState>,
    Json(body): Json<IgnoreBody>,
) -> Json<ApiResult> {
    let mut config = state.config.write().await;
    if !config.ignore.contains(&body.project) {
        config.ignore.push(body.project);
    }
    match config.save() {
        Ok(_) => Json(ApiResult { ok: true, error: None }),
        Err(e) => Json(ApiResult { ok: false, error: Some(e.to_string()) }),
    }
}

async fn api_remove_ignore(
    Path(project): Path<String>,
    State(state): State<SharedState>,
) -> Json<ApiResult> {
    let mut config = state.config.write().await;
    config.ignore.retain(|p| p != &project);
    match config.save() {
        Ok(_) => Json(ApiResult { ok: true, error: None }),
        Err(e) => Json(ApiResult { ok: false, error: Some(e.to_string()) }),
    }
}

#[derive(Deserialize)]
struct SetSubdirsBody { subdirs: Vec<String> }

async fn api_set_subdirs(
    Path(name): Path<String>,
    State(state): State<SharedState>,
    Json(body): Json<SetSubdirsBody>,
) -> Json<ApiResult> {
    let mut config = state.config.write().await;
    config.projects.entry(name).or_default().subdirs = body.subdirs;
    match config.save() {
        Ok(_) => Json(ApiResult { ok: true, error: None }),
        Err(e) => Json(ApiResult { ok: false, error: Some(e.to_string()) }),
    }
}

async fn api_clear_subdirs(
    Path(name): Path<String>,
    State(state): State<SharedState>,
) -> Json<ApiResult> {
    let mut config = state.config.write().await;
    config.projects.remove(&name);
    match config.save() {
        Ok(_) => Json(ApiResult { ok: true, error: None }),
        Err(e) => Json(ApiResult { ok: false, error: Some(e.to_string()) }),
    }
}

// ── Services (compose services for non-subdir projects) ──

async fn api_project_services(
    Path(name): Path<String>,
    State(state): State<SharedState>,
) -> Json<serde_json::Value> {
    let config = state.config.read().await.clone();
    let project_dir = config.projects_dir.join(&name);
    let compose_files = find_compose_files(&project_dir, &name, &config);
    drop(config);

    // Only meaningful when there's a single root compose file (not subdir-based)
    let Some(compose_file) = compose_files.first() else {
        return Json(serde_json::json!({ "services": [] }));
    };

    let Ok(output) = tokio::process::Command::new("docker")
        .args(["compose", "-f"]).arg(compose_file)
        .args(["ps", "--format", "json", "--all"])
        .output().await
    else {
        return Json(serde_json::json!({ "services": [] }));
    };

    let mut seen = std::collections::HashSet::new();
    let services: Vec<_> = parse_compose_ps(&String::from_utf8_lossy(&output.stdout))
        .into_iter()
        .filter_map(|c| {
            let svc = c["Service"].as_str()?.to_string();
            let status = if c["State"].as_str() == Some("running") { "running" } else { "stopped" };
            seen.insert(svc.clone()).then_some(serde_json::json!({ "name": svc, "status": status }))
        })
        .collect();

    Json(serde_json::json!({ "services": services }))
}

// ── Shell (WebSocket + PTY) ───────────────────────────

#[derive(Deserialize)]
struct ShellQuery {
    service: Option<String>,
}

async fn api_shell(
    Path(name): Path<String>,
    Query(query): Query<ShellQuery>,
    State(state): State<SharedState>,
    ws: WebSocketUpgrade,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| async move {
        if let Err(e) = run_shell(socket, name, query, state).await {
            eprintln!("shell: {e}");
        }
    })
}

async fn run_shell(socket: WebSocket, name: String, query: ShellQuery, state: SharedState) -> anyhow::Result<()> {
    use std::ffi::CString;
    use std::os::unix::io::{AsRawFd, FromRawFd};
    use std::os::unix::process::CommandExt;

    let config = state.config.read().await.clone();
    let project_dir = config.projects_dir.join(&name);
    let all_files = find_compose_files(&project_dir, &name, &config);
    drop(config);

    // Try to match compose file by subdir name first (subdir-based projects).
    // If not found, fall back to the first compose file and match container by service name.
    let (compose_file, match_by_service) = match query.service.as_deref() {
        Some(svc) => {
            if let Some(f) = all_files.iter().find(|f| {
                f.parent().and_then(|d| d.file_name()).map(|n| n.to_str() == Some(svc)).unwrap_or(false)
            }) {
                (f.clone(), false)
            } else {
                (all_files.into_iter().next().ok_or_else(|| anyhow::anyhow!("no compose file"))?, true)
            }
        }
        None => (all_files.into_iter().next().ok_or_else(|| anyhow::anyhow!("no compose file"))?, false),
    };

    let output = tokio::process::Command::new("docker")
        .args(["compose", "-f"]).arg(&compose_file)
        .args(["ps", "--format", "json"])
        .output().await?;

    let containers = parse_compose_ps(&String::from_utf8_lossy(&output.stdout));
    let container = if match_by_service {
        let svc = query.service.as_deref().unwrap();
        containers.iter().find(|c| c["Service"].as_str() == Some(svc)).cloned()
    } else {
        containers.iter()
            .find(|c| c["State"].as_str() == Some("running") || c["Status"].as_str().map(|s| s.starts_with("Up")).unwrap_or(false))
            .or_else(|| containers.first())
            .cloned()
    }.ok_or_else(|| anyhow::anyhow!("container not found"))?;

    let container_name = container["Name"].as_str()
        .ok_or_else(|| anyhow::anyhow!("no container name"))?.to_string();

    // Allocate PTY using POSIX functions
    let master_fd = unsafe { libc::posix_openpt(libc::O_RDWR | libc::O_NOCTTY) };
    anyhow::ensure!(master_fd >= 0, "posix_openpt failed");
    unsafe { libc::grantpt(master_fd); libc::unlockpt(master_fd); }
    let slave_path = unsafe { std::ffi::CStr::from_ptr(libc::ptsname(master_fd)).to_string_lossy().into_owned() };
    let slave_name = CString::new(slave_path)?;
    let slave_fd = unsafe { libc::open(slave_name.as_ptr(), libc::O_RDWR) };
    anyhow::ensure!(slave_fd >= 0, "open slave PTY failed");

    // Spawn docker exec with PTY slave as stdio
    // Try bash first (has readline/history), fall back to sh
    let mut std_cmd = std::process::Command::new("docker");
    std_cmd.args(["exec", "-it", &container_name,
        "sh", "-c", "command -v bash > /dev/null 2>&1 && exec bash || exec sh"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    unsafe {
        std_cmd.pre_exec(move || {
            libc::setsid();
            libc::ioctl(slave_fd, libc::TIOCSCTTY as _, 0i32);
            libc::dup2(slave_fd, 0);
            libc::dup2(slave_fd, 1);
            libc::dup2(slave_fd, 2);
            if slave_fd > 2 { libc::close(slave_fd); }
            Ok(())
        });
    }

    let mut child = tokio::process::Command::from(std_cmd).spawn()?;
    unsafe { libc::close(slave_fd); }

    // Wrap master in AsyncFd for proper epoll-based async I/O on PTY
    unsafe { libc::fcntl(master_fd, libc::F_SETFL, libc::O_NONBLOCK); }
    let master_file = unsafe { std::fs::File::from_raw_fd(master_fd) };
    let afd = std::sync::Arc::new(tokio::io::unix::AsyncFd::new(master_file)?);

    let (mut ws_tx, mut ws_rx) = socket.split();

    // PTY → WebSocket
    let afd_r = afd.clone();
    let to_ws = tokio::spawn(async move {
        let mut buf = vec![0u8; 4096];
        loop {
            let mut guard = match afd_r.readable().await {
                Ok(g) => g,
                Err(_) => break,
            };
            match guard.try_io(|inner| {
                let n = unsafe { libc::read(inner.get_ref().as_raw_fd(), buf.as_mut_ptr() as *mut _, buf.len()) };
                if n < 0 { Err(std::io::Error::last_os_error()) } else { Ok(n as usize) }
            }) {
                Ok(Ok(0)) => break,
                Ok(Ok(n)) => {
                    if ws_tx.send(Message::Binary(buf[..n].to_vec().into())).await.is_err() { break; }
                }
                Ok(Err(_)) => break,
                Err(_) => {} // WouldBlock, retry
            }
        }
    });

    // WebSocket → PTY
    while let Some(Ok(msg)) = ws_rx.next().await {
        let bytes: Vec<u8> = match &msg {
            Message::Binary(d) => d.to_vec(),
            Message::Text(t) => {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(t) {
                    if v["type"] == "resize" {
                        let ws = libc::winsize {
                            ws_col: v["cols"].as_u64().unwrap_or(80) as u16,
                            ws_row: v["rows"].as_u64().unwrap_or(24) as u16,
                            ws_xpixel: 0, ws_ypixel: 0,
                        };
                        unsafe { libc::ioctl(master_fd, libc::TIOCSWINSZ as _, &ws as *const _); }
                    }
                    continue;
                }
                t.as_bytes().to_vec()
            }
            Message::Close(_) => break,
            _ => continue,
        };

        let mut written = 0;
        while written < bytes.len() {
            let mut guard = match afd.writable().await {
                Ok(g) => g,
                Err(_) => { break; }
            };
            match guard.try_io(|inner| {
                let n = unsafe { libc::write(inner.get_ref().as_raw_fd(), bytes[written..].as_ptr() as *const _, bytes.len() - written) };
                if n < 0 { Err(std::io::Error::last_os_error()) } else { Ok(n as usize) }
            }) {
                Ok(Ok(n)) => written += n,
                Ok(Err(_)) => break,
                Err(_) => {} // WouldBlock, retry
            }
        }
    }

    to_ws.abort();
    child.kill().await.ok();
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────

fn find_compose_files(
    project_dir: &std::path::Path,
    project_name: &str,
    config: &Config,
) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let compose = project_dir.join("docker-compose.yml");

    if compose.exists() {
        files.push(compose);
        return files;
    }

    let allowed = config.subdirs_for(project_name);

    if let Ok(entries) = std::fs::read_dir(project_dir) {
        let mut entries: Vec<_> = entries.flatten().collect();
        entries.sort_by_key(|e| e.file_name());

        for sub in entries {
            let sub_name = sub.file_name().to_string_lossy().to_string();
            let sub_compose = sub.path().join("docker-compose.yml");

            if !sub_compose.exists() {
                continue;
            }

            if let Some(allowed_list) = allowed {
                if !allowed_list.contains(&sub_name) {
                    continue;
                }
            }

            files.push(sub_compose);
        }
    }

    files
}
