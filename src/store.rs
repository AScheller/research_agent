//! Laufende und gespeicherte Recherchen inkl. Event-Historie und Live-Broadcast.

use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::{sync::broadcast, task::AbortHandle};

pub type Runs = Arc<Mutex<HashMap<String, Run>>>;

/// Event-Typen, die nur live relevant sind und nicht gespeichert werden.
const TRANSIENT: [&str; 2] = ["token", "reasoning"];

pub struct Run {
    pub id: String,
    pub prompt: String,
    pub created: String,
    pub status: String,
    pub options: Value,
    pub report: Option<String>,
    pub events: Vec<Value>,
    pub seq: u64,
    pub tx: Option<broadcast::Sender<Value>>,
    pub abort: Option<AbortHandle>,
}

#[derive(Serialize, Deserialize)]
pub struct PersistedRun {
    pub id: String,
    pub prompt: String,
    pub created: String,
    pub status: String,
    #[serde(default)]
    pub options: Value,
    #[serde(default)]
    pub report: Option<String>,
    #[serde(default)]
    pub events: Vec<Value>,
}

impl Run {
    pub fn new(id: String, prompt: String, options: Value, tx: broadcast::Sender<Value>) -> Self {
        Self {
            id,
            prompt,
            created: chrono::Local::now().to_rfc3339(),
            status: "running".into(),
            options,
            report: None,
            events: Vec::new(),
            seq: 0,
            tx: Some(tx),
            abort: None,
        }
    }

    fn from_persisted(p: PersistedRun) -> Self {
        let seq = p.events.iter().filter_map(|e| e["seq"].as_u64()).max().unwrap_or(0);
        Self {
            id: p.id,
            prompt: p.prompt,
            created: p.created,
            status: p.status,
            options: p.options,
            report: p.report,
            events: p.events,
            seq,
            tx: None,
            abort: None,
        }
    }

    pub fn persisted(&self) -> PersistedRun {
        PersistedRun {
            id: self.id.clone(),
            prompt: self.prompt.clone(),
            created: self.created.clone(),
            status: self.status.clone(),
            options: self.options.clone(),
            report: self.report.clone(),
            events: self
                .events
                .iter()
                .filter(|e| !TRANSIENT.contains(&e["type"].as_str().unwrap_or("")))
                .cloned()
                .collect(),
        }
    }

    pub fn summary(&self) -> Value {
        let title = self
            .report
            .as_deref()
            .and_then(|r| r.lines().find_map(|l| l.strip_prefix("# ")))
            .map(|t| t.trim().to_string());
        json!({
            "id": self.id,
            "prompt": self.prompt,
            "title": title,
            "created": self.created,
            "status": self.status,
        })
    }
}

pub fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

/// Schreibt Events in die Historie eines Laufs und verteilt sie live an alle SSE-Clients.
#[derive(Clone)]
pub struct Emitter {
    pub runs: Runs,
    pub id: String,
}

impl Emitter {
    pub fn emit(&self, mut ev: Value) {
        let mut map = self.runs.lock().unwrap();
        let Some(run) = map.get_mut(&self.id) else { return };
        if run.status != "running" {
            return;
        }
        run.seq += 1;
        ev["seq"] = json!(run.seq);
        ev["ts"] = json!(now_ms());
        if let Some(tx) = &run.tx {
            let _ = tx.send(ev.clone());
        }
        run.events.push(ev);
    }
}

/// Schließt einen Lauf ab, sendet das `done`-Event, beendet die Streams und speichert ihn.
pub fn finish(runs: &Runs, data_dir: &Path, id: &str, status: &str, report: Option<String>) {
    let snapshot = {
        let mut map = runs.lock().unwrap();
        let Some(run) = map.get_mut(id) else { return };
        if run.status != "running" {
            return;
        }
        run.status = status.to_string();
        if report.is_some() {
            run.report = report;
        }
        run.seq += 1;
        let ev = json!({"type": "done", "status": status, "seq": run.seq, "ts": now_ms()});
        run.events.push(ev.clone());
        if let Some(tx) = run.tx.take() {
            let _ = tx.send(ev);
        }
        run.abort = None;
        run.persisted()
    };
    let path = data_dir.join("runs").join(format!("{id}.json"));
    match serde_json::to_vec_pretty(&snapshot) {
        Ok(bytes) => {
            if let Err(e) = std::fs::write(&path, bytes) {
                tracing::warn!("Konnte {} nicht speichern: {e}", path.display());
            }
        }
        Err(e) => tracing::warn!("Serialisierung von Lauf {id} fehlgeschlagen: {e}"),
    }
}

fn read_run(path: &Path) -> anyhow::Result<PersistedRun> {
    let raw = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw)?)
}

pub fn load_runs(dir: &Path) -> Runs {
    let mut map = HashMap::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            match read_run(&path) {
                Ok(p) => {
                    map.insert(p.id.clone(), Run::from_persisted(p));
                }
                Err(e) => tracing::warn!("{}: {e}", path.display()),
            }
        }
    }
    tracing::info!("{} gespeicherte Recherchen geladen", map.len());
    Arc::new(Mutex::new(map))
}
