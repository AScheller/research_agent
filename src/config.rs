use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Platzhalter, mit dem ein gesetzter API-Key an die Oberfläche ausgeliefert wird.
pub const MASK: &str = "********";

const HEADER: &str = "# Reflective Research Agent – Konfiguration / configuration\n\
# DE: Wird auch über die Einstellungen der Weboberfläche geschrieben.\n\
# EN: Also written by the settings dialog of the web UI.\n\
# Env overrides: LLM_BASE_URL, LLM_MODEL, LLM_API_KEY, SEARXNG_URL, RA_PORT\n\n";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub server: ServerCfg,
    pub llm: LlmCfg,
    pub search: SearchCfg,
    pub agent: AgentCfg,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerCfg {
    pub host: String,
    pub port: u16,
}

impl Default for ServerCfg {
    fn default() -> Self {
        Self { host: "0.0.0.0".into(), port: 8787 }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct LlmCfg {
    /// OpenAI-kompatible Basis-URL (vLLM / llama-swap) inklusive `/v1`
    pub base_url: String,
    pub model: String,
    pub api_key: String,
    pub temperature: f64,
    pub max_tokens: u32,
}

impl Default for LlmCfg {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8000/v1".into(),
            model: "gemma-4-26b".into(),
            api_key: String::new(),
            temperature: 0.4,
            max_tokens: 12000,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct SearchCfg {
    /// Basis-URL der SearXNG-Instanz, z. B. `http://searxng:8080` (leer = Websuche deaktiviert)
    pub searxng_url: String,
    /// SearXNG-Sprachparameter (`all`, `de`, `en`, …)
    pub language: String,
    pub max_results: usize,
    pub wikipedia_lang: String,
    pub enable_arxiv: bool,
    pub enable_wikipedia: bool,
}

impl Default for SearchCfg {
    fn default() -> Self {
        Self {
            searxng_url: String::new(),
            language: "all".into(),
            max_results: 8,
            wikipedia_lang: "en".into(),
            enable_arxiv: true,
            enable_wikipedia: true,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct AgentCfg {
    /// Maximale Anzahl geplanter Schritte (Research + Writer + Editor), ohne finalen Bericht
    pub max_steps: usize,
    /// Maximale Tool-Runden pro Research-Schritt
    pub max_tool_rounds: usize,
    /// Maximale Zeichen, die von einer abgerufenen Webseite an das Modell gehen
    pub fetch_max_chars: usize,
}

impl Default for AgentCfg {
    fn default() -> Self {
        Self { max_steps: 7, max_tool_rounds: 6, fetch_max_chars: 8000 }
    }
}

impl Config {
    pub fn normalize(&mut self) {
        self.llm.base_url = normalize_url(&self.llm.base_url);
        let searx = normalize_url(&self.search.searxng_url);
        self.search.searxng_url = searx.strip_suffix("/search").unwrap_or(&searx).to_string();
        self.search.max_results = self.search.max_results.clamp(1, 20);
        if self.search.wikipedia_lang.trim().is_empty() {
            self.search.wikipedia_lang = "de".into();
        }
        self.agent.max_steps = self.agent.max_steps.clamp(3, 12);
        self.agent.max_tool_rounds = self.agent.max_tool_rounds.clamp(1, 12);
        self.agent.fetch_max_chars = self.agent.fetch_max_chars.clamp(1000, 40000);
        self.llm.max_tokens = self.llm.max_tokens.max(512);
        self.llm.temperature = self.llm.temperature.clamp(0.0, 2.0);
    }

    /// Konfiguration für die Oberfläche (API-Key maskiert).
    pub fn public_json(&self) -> Value {
        let mut v = serde_json::to_value(self).unwrap_or_default();
        if !self.llm.api_key.is_empty() {
            v["llm"]["api_key"] = Value::String(MASK.into());
        }
        v
    }
}

fn normalize_url(s: &str) -> String {
    let s = s.trim().trim_end_matches('/');
    if s.is_empty() {
        String::new()
    } else if s.starts_with("http://") || s.starts_with("https://") {
        s.to_string()
    } else {
        format!("http://{s}")
    }
}

pub fn load() -> anyhow::Result<(Config, PathBuf)> {
    let path = PathBuf::from(std::env::var("RA_CONFIG").unwrap_or_else(|_| "config.toml".into()));
    let mut cfg = if path.exists() {
        let raw = std::fs::read_to_string(&path)?;
        toml::from_str::<Config>(&raw).map_err(|e| anyhow::anyhow!("{}: {e}", path.display()))?
    } else {
        let cfg = Config::default();
        save(&path, &cfg)?;
        cfg
    };

    if let Ok(v) = std::env::var("LLM_BASE_URL") {
        cfg.llm.base_url = v;
    }
    if let Ok(v) = std::env::var("LLM_MODEL") {
        cfg.llm.model = v;
    }
    if let Ok(v) = std::env::var("LLM_API_KEY") {
        cfg.llm.api_key = v;
    }
    if let Ok(v) = std::env::var("SEARXNG_URL") {
        cfg.search.searxng_url = v;
    }
    if let Some(port) = std::env::var("RA_PORT").ok().and_then(|v| v.parse().ok()) {
        cfg.server.port = port;
    }
    cfg.normalize();
    Ok((cfg, path))
}

pub fn save(path: &Path, cfg: &Config) -> anyhow::Result<()> {
    let body = toml::to_string_pretty(cfg)?;
    std::fs::write(path, format!("{HEADER}{body}"))?;
    Ok(())
}
