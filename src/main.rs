//! Reflective Research Agent – Webserver (axum) mit SSE-Live-Stream des Agenten-Workflows.

#[macro_use]
mod i18n;
mod agent;
mod config;
mod llm;
mod store;
mod text;
mod tools;

use std::{
    convert::Infallible,
    path::PathBuf,
    sync::{Arc, RwLock},
    time::Duration,
};

use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    routing::{get, post},
    Json, Router,
};
use futures::{stream, Stream, StreamExt};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio_stream::wrappers::BroadcastStream;

use crate::{
    config::Config,
    i18n::Lang,
    store::{Emitter, Run, Runs},
};

const INDEX_HTML: &str = include_str!("../static/index.html");
const APP_JS: &str = include_str!("../static/app.js");
const STYLE_CSS: &str = include_str!("../static/style.css");

#[derive(Clone)]
struct AppState {
    cfg: Arc<RwLock<Config>>,
    cfg_path: PathBuf,
    runs: Runs,
    http: reqwest::Client,
    data_dir: PathBuf,
}

type ApiError = (StatusCode, String);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let (cfg, cfg_path) = config::load()?;
    let data_dir = PathBuf::from(std::env::var("RA_DATA_DIR").unwrap_or_else(|_| "data".into()));
    std::fs::create_dir_all(data_dir.join("runs"))?;
    let runs = store::load_runs(&data_dir.join("runs"));

    let http = reqwest::Client::builder()
        .user_agent(concat!("ReflectiveResearchAgent/", env!("CARGO_PKG_VERSION"), " (local research assistant)"))
        .connect_timeout(Duration::from_secs(10))
        .build()?;

    let addr = format!("{}:{}", cfg.server.host, cfg.server.port);
    let port = cfg.server.port;
    tracing::info!(
        "Konfiguration: {} | LLM {} ({}) | SearXNG: {}",
        cfg_path.display(),
        cfg.llm.base_url,
        cfg.llm.model,
        if cfg.search.searxng_url.is_empty() { "nicht konfiguriert" } else { &cfg.search.searxng_url }
    );

    let state = AppState { cfg: Arc::new(RwLock::new(cfg)), cfg_path, runs, http, data_dir };

    let app = Router::new()
        .route("/", get(|| async { asset("text/html; charset=utf-8", INDEX_HTML) }))
        .route("/app.js", get(|| async { asset("application/javascript; charset=utf-8", APP_JS) }))
        .route("/style.css", get(|| async { asset("text/css; charset=utf-8", STYLE_CSS) }))
        .route("/api/config", get(get_config).put(put_config))
        .route("/api/health", get(health))
        .route("/api/runs", get(list_runs).post(create_run))
        .route("/api/runs/{id}", get(get_run).delete(delete_run))
        .route("/api/runs/{id}/events", get(run_events))
        .route("/api/runs/{id}/cancel", post(cancel_run))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Research Agent läuft auf http://localhost:{port}");
    axum::serve(listener, app).await?;
    Ok(())
}

fn asset(content_type: &'static str, body: &'static str) -> Response {
    ([(header::CONTENT_TYPE, content_type), (header::CACHE_CONTROL, "no-cache")], body).into_response()
}

fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 64 && id.chars().all(|c| c.is_ascii_alphanumeric())
}

fn not_found(lang: Lang) -> ApiError {
    (StatusCode::NOT_FOUND, tr!(lang, "Recherche nicht gefunden", "Research not found"))
}

/// Sprache der Oberfläche aus dem Header `X-UI-Lang` (Standard: Deutsch).
fn ui_lang(headers: &HeaderMap) -> Lang {
    headers.get("x-ui-lang").and_then(|v| v.to_str().ok()).map(Lang::parse).unwrap_or_default()
}

// ---------------------------------------------------------------------- Konfiguration

async fn get_config(State(st): State<AppState>) -> Json<Value> {
    let cfg = st.cfg.read().unwrap().clone();
    Json(cfg.public_json())
}

async fn put_config(State(st): State<AppState>, Json(mut incoming): Json<Config>) -> Result<Json<Value>, ApiError> {
    let saved = {
        let mut cfg = st.cfg.write().unwrap();
        incoming.server = cfg.server.clone();
        if incoming.llm.api_key == config::MASK {
            incoming.llm.api_key = cfg.llm.api_key.clone();
        }
        incoming.normalize();
        *cfg = incoming;
        cfg.clone()
    };
    config::save(&st.cfg_path, &saved).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    tracing::info!("Konfiguration gespeichert ({})", st.cfg_path.display());
    Ok(Json(saved.public_json()))
}

async fn health(State(st): State<AppState>, headers: HeaderMap) -> Json<Value> {
    let lang = ui_lang(&headers);
    let cfg = st.cfg.read().unwrap().clone();
    let (llm, searx) = tokio::join!(check_llm(&st.http, &cfg, lang), check_searx(&st.http, &cfg, lang));
    Json(json!({ "llm": llm, "searxng": searx }))
}

async fn check_llm(http: &reqwest::Client, cfg: &Config, lang: Lang) -> Value {
    let mut req = http.get(format!("{}/models", cfg.llm.base_url)).timeout(Duration::from_secs(6));
    if !cfg.llm.api_key.is_empty() {
        req = req.bearer_auth(&cfg.llm.api_key);
    }
    match req.send().await {
        Ok(r) if r.status().is_success() => {
            let v: Value = r.json().await.unwrap_or(Value::Null);
            let models: Vec<Value> = v["data"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|m| {
                    json!({
                        "id": m["id"],
                        "name": m.get("name").filter(|n| n.is_string()).unwrap_or(&m["id"]),
                        "loaded": m["status"]["value"].as_str() == Some("loaded"),
                    })
                })
                .collect();
            let found = models.iter().any(|m| m["id"].as_str() == Some(cfg.llm.model.as_str()));
            let detail = if found {
                tr!(lang, "verbunden", "connected")
            } else {
                tr!(lang, "Modell wird vom Server nicht gelistet", "Model is not listed by the server")
            };
            json!({ "ok": true, "model": cfg.llm.model, "model_found": found, "models": models, "detail": detail })
        }
        Ok(r) => json!({ "ok": false, "model": cfg.llm.model, "detail": format!("HTTP {}", r.status()) }),
        Err(e) => json!({ "ok": false, "model": cfg.llm.model, "detail": e.to_string() }),
    }
}

async fn check_searx(http: &reqwest::Client, cfg: &Config, lang: Lang) -> Value {
    let base = cfg.search.searxng_url.trim();
    if base.is_empty() {
        return json!({ "ok": false, "configured": false, "detail": tr!(lang, "Keine SearXNG-URL hinterlegt", "No SearXNG URL configured") });
    }
    let resp = http
        .get(format!("{base}/search"))
        .query(&[("q", "test"), ("format", "json")])
        .header(header::ACCEPT, "application/json")
        .timeout(Duration::from_secs(10))
        .send()
        .await;
    match resp {
        Ok(r) if r.status().is_success() => match r.json::<Value>().await {
            Ok(v) => {
                let n = v["results"].as_array().map_or(0, |a| a.len());
                let detail = tr!(lang, "erreichbar ({} Treffer für Testanfrage)", "reachable ({} results for test query)", n);
                json!({ "ok": true, "configured": true, "detail": detail })
            }
            Err(_) => json!({
                "ok": false, "configured": true,
                "detail": tr!(lang, "Antwort ist kein JSON – 'json' in search.formats aktivieren", "Response is not JSON – enable 'json' in search.formats")
            }),
        },
        Ok(r) if r.status() == StatusCode::FORBIDDEN => json!({
            "ok": false, "configured": true,
            "detail": tr!(
                lang,
                "HTTP 403 – JSON-Format in der settings.yml freischalten (search.formats: [html, json])",
                "HTTP 403 – enable the JSON format in settings.yml (search.formats: [html, json])"
            )
        }),
        Ok(r) => json!({ "ok": false, "configured": true, "detail": format!("HTTP {}", r.status()) }),
        Err(e) => json!({ "ok": false, "configured": true, "detail": e.to_string() }),
    }
}

// ---------------------------------------------------------------------- Recherchen

#[derive(Deserialize)]
struct CreateRun {
    prompt: String,
    #[serde(default)]
    depth: Option<String>,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    ui_lang: Option<String>,
}

async fn create_run(State(st): State<AppState>, headers: HeaderMap, Json(req): Json<CreateRun>) -> Result<Json<Value>, ApiError> {
    let ui = req.ui_lang.as_deref().map(Lang::parse).unwrap_or_else(|| ui_lang(&headers));
    let prompt = req.prompt.trim().to_string();
    if prompt.is_empty() {
        return Err((StatusCode::BAD_REQUEST, tr!(ui, "Bitte eine Recherchefrage eingeben", "Please enter a research question")));
    }
    let opts = agent::RunOptions {
        depth: req.depth.unwrap_or_else(|| "standard".into()),
        language: req.language.unwrap_or_else(|| if ui == Lang::En { "en".into() } else { "de".into() }),
        ui_lang: ui,
    };
    let id = uuid::Uuid::new_v4().simple().to_string()[..12].to_string();
    let (tx, _) = tokio::sync::broadcast::channel(16384);
    {
        let mut runs = st.runs.lock().unwrap();
        runs.insert(id.clone(), Run::new(id.clone(), prompt.clone(), json!(opts), tx));
    }

    let cfg = st.cfg.read().unwrap().clone();
    let em = Emitter { runs: st.runs.clone(), id: id.clone() };
    let (runs, data_dir, http, run_id) = (st.runs.clone(), st.data_dir.clone(), st.http.clone(), id.clone());
    tracing::info!("Neue Recherche {run_id}: {}", text::truncate(&prompt, 120));
    let handle = tokio::spawn(async move {
        match agent::run(cfg, http, em.clone(), prompt, opts).await {
            Ok(report) => {
                tracing::info!("Recherche {run_id} abgeschlossen");
                store::finish(&runs, &data_dir, &run_id, "done", Some(report));
            }
            Err(e) => {
                tracing::error!("Recherche {run_id} fehlgeschlagen: {e:#}");
                em.emit(json!({ "type": "error", "message": format!("{e:#}") }));
                store::finish(&runs, &data_dir, &run_id, "error", None);
            }
        }
    });
    {
        let mut runs = st.runs.lock().unwrap();
        if let Some(run) = runs.get_mut(&id) {
            if run.status == "running" {
                run.abort = Some(handle.abort_handle());
            }
        }
    }
    Ok(Json(json!({ "id": id })))
}

async fn list_runs(State(st): State<AppState>) -> Json<Value> {
    let runs = st.runs.lock().unwrap();
    let mut items: Vec<&Run> = runs.values().collect();
    items.sort_by(|a, b| b.created.cmp(&a.created));
    Json(Value::Array(items.iter().map(|r| r.summary()).collect()))
}

async fn get_run(State(st): State<AppState>, Path(id): Path<String>, headers: HeaderMap) -> Result<Json<Value>, ApiError> {
    let runs = st.runs.lock().unwrap();
    let run = runs.get(&id).ok_or_else(|| not_found(ui_lang(&headers)))?;
    Ok(Json(serde_json::to_value(run.persisted()).unwrap_or_default()))
}

async fn delete_run(State(st): State<AppState>, Path(id): Path<String>, headers: HeaderMap) -> Result<StatusCode, ApiError> {
    if !valid_id(&id) {
        return Err(not_found(ui_lang(&headers)));
    }
    let removed = st.runs.lock().unwrap().remove(&id);
    if let Some(handle) = removed.and_then(|r| r.abort) {
        handle.abort();
    }
    let _ = std::fs::remove_file(st.data_dir.join("runs").join(format!("{id}.json")));
    Ok(StatusCode::NO_CONTENT)
}

async fn cancel_run(State(st): State<AppState>, Path(id): Path<String>, headers: HeaderMap) -> Result<Json<Value>, ApiError> {
    let handle = {
        let mut runs = st.runs.lock().unwrap();
        let run = runs.get_mut(&id).ok_or_else(|| not_found(ui_lang(&headers)))?;
        if run.status != "running" {
            return Ok(Json(json!({ "ok": false, "status": run.status })));
        }
        run.abort.take()
    };
    if let Some(h) = handle {
        h.abort();
    }
    Emitter { runs: st.runs.clone(), id: id.clone() }.emit(json!({ "type": "cancelled" }));
    store::finish(&st.runs, &st.data_dir, &id, "cancelled", None);
    Ok(Json(json!({ "ok": true })))
}

/// Server-Sent Events: erst die komplette Historie, dann Live-Events bis zum Abschluss.
async fn run_events(
    State(st): State<AppState>,
    Path(id): Path<String>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, StatusCode> {
    let (history, rx) = {
        let runs = st.runs.lock().unwrap();
        let run = runs.get(&id).ok_or(StatusCode::NOT_FOUND)?;
        (run.events.clone(), run.tx.as_ref().map(|tx| tx.subscribe()))
    };
    let live = match rx {
        Some(rx) => BroadcastStream::new(rx).filter_map(|r| async move { r.ok() }).boxed(),
        None => stream::empty::<Value>().boxed(),
    };
    let events = stream::iter(history).chain(live).map(|ev| Ok(Event::default().data(ev.to_string())));
    Ok(Sse::new(events).keep_alive(KeepAlive::default()))
}
