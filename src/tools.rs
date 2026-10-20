//! Werkzeuge des Research-Agents: SearXNG-Websuche, arXiv, Wikipedia, Seitenabruf
//! sowie das nummerierte Quellenregister, auf das sich alle Zitate [n] beziehen.

use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
    time::{Duration, Instant},
};

use anyhow::{anyhow, bail, Result};
use regex::Regex;
use serde::Serialize;
use serde_json::{json, Value};

use crate::{config::Config, i18n::Lang, store::Emitter, text};

#[derive(Clone, Debug, Serialize)]
pub struct Source {
    pub n: usize,
    pub title: String,
    pub url: String,
    pub kind: String,
    pub meta: String,
    pub snippet: String,
}

#[derive(Default)]
pub struct Registry {
    pub items: Vec<Source>,
    by_key: HashMap<String, usize>,
}

impl Registry {
    fn add(&mut self, title: &str, url: &str, kind: &str, meta: &str, snippet: &str) -> (usize, Option<Source>) {
        let key = url_key(url);
        let snippet = text::truncate(&text::collapse_ws(snippet), 600);
        if let Some(&n) = self.by_key.get(&key) {
            let s = &mut self.items[n - 1];
            if s.snippet.chars().count() < snippet.chars().count() {
                s.snippet = snippet;
            }
            if s.meta.is_empty() && !meta.is_empty() {
                s.meta = meta.to_string();
            }
            return (n, None);
        }
        let n = self.items.len() + 1;
        let title = text::collapse_ws(title);
        let src = Source {
            n,
            title: if title.is_empty() { url.to_string() } else { title },
            url: url.to_string(),
            kind: kind.to_string(),
            meta: meta.to_string(),
            snippet,
        };
        self.items.push(src.clone());
        self.by_key.insert(key, n);
        (n, Some(src))
    }

    pub fn get(&self, n: usize) -> Option<&Source> {
        n.checked_sub(1).and_then(|i| self.items.get(i))
    }
}

/// Normalisierter Schlüssel zur Duplikaterkennung (Schema, www, Fragment, arXiv-Versionen).
fn url_key(url: &str) -> String {
    static RE_ARXIV_VERSION: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"v\d+$").unwrap());
    let mut u = url.trim().to_lowercase();
    if let Some(i) = u.find('#') {
        u.truncate(i);
    }
    for prefix in ["https://", "http://"] {
        if let Some(rest) = u.strip_prefix(prefix) {
            u = rest.to_string();
            break;
        }
    }
    if let Some(rest) = u.strip_prefix("www.") {
        u = rest.to_string();
    }
    u = u.trim_end_matches('/').to_string();
    if u.contains("arxiv.org/") {
        u = u.replace("export.arxiv.org", "arxiv.org").replace("arxiv.org/pdf/", "arxiv.org/abs/");
        if let Some(rest) = u.strip_suffix(".pdf") {
            u = rest.to_string();
        }
        u = RE_ARXIV_VERSION.replace(&u, "").into_owned();
    }
    u
}

pub struct ToolOutput {
    pub ok: bool,
    /// Text, der dem Modell als Tool-Ergebnis übergeben wird
    pub text: String,
    /// Kurzfassung bzw. Fehlermeldung für die Oberfläche (in der UI-Sprache des Laufs)
    pub summary: String,
    /// Anzahl + Einheit, damit die Oberfläche die Kurzfassung in jeder Sprache darstellen kann
    pub count: Option<usize>,
    pub unit: &'static str,
    pub items: Vec<Value>,
    pub preview: Option<String>,
}

impl ToolOutput {
    fn error(msg: String) -> Self {
        Self { ok: false, text: format!("ERROR: {msg}"), summary: msg, count: None, unit: "", items: Vec::new(), preview: None }
    }

    fn ok(text: String, summary: String, count: usize, unit: &'static str, items: Vec<Value>, preview: Option<String>) -> Self {
        Self { ok: true, text, summary, count: Some(count), unit, items, preview }
    }
}

static ARXIV_GATE: LazyLock<tokio::sync::Mutex<Option<Instant>>> = LazyLock::new(|| tokio::sync::Mutex::new(None));

struct ArxivEntry {
    url: String,
    title: String,
    summary: String,
    published: String,
    authors: Vec<String>,
    category: String,
}

pub struct Tools {
    http: reqwest::Client,
    cfg: Config,
    ui: Lang,
    max_results: usize,
    pub registry: Arc<Mutex<Registry>>,
    em: Emitter,
}

impl Tools {
    pub fn new(http: reqwest::Client, cfg: Config, ui: Lang, max_results: usize, em: Emitter) -> Self {
        Self { http, cfg, ui, max_results, registry: Arc::new(Mutex::new(Registry::default())), em }
    }

    pub fn web_enabled(&self) -> bool {
        !self.cfg.search.searxng_url.trim().is_empty()
    }

    pub fn names(&self) -> Vec<&'static str> {
        let mut v = Vec::new();
        if self.web_enabled() {
            v.push("web_search");
        }
        if self.cfg.search.enable_arxiv {
            v.push("arxiv_search");
        }
        if self.cfg.search.enable_wikipedia {
            v.push("wikipedia_search");
        }
        v.push("fetch_url");
        v
    }

    /// Beschreibung der verfügbaren Werkzeuge für den Planer.
    pub fn describe(&self) -> String {
        let mut v = Vec::new();
        if self.web_enabled() {
            v.push("web_search (web metasearch via SearXNG; categories general/news/science/it)");
        }
        if self.cfg.search.enable_arxiv {
            v.push("arxiv_search (scientific preprints on arXiv)");
        }
        if self.cfg.search.enable_wikipedia {
            v.push("wikipedia_search (encyclopedic definitions and background)");
        }
        v.push("fetch_url (read a web page in full)");
        v.join(", ")
    }

    pub fn schemas(&self) -> Vec<Value> {
        let mut v = Vec::new();
        if self.web_enabled() {
            v.push(json!({"type": "function", "function": {
                "name": "web_search",
                "description": "Search the web via the SearXNG metasearch engine. Returns numbered results with title, URL, date and snippet.",
                "parameters": {"type": "object", "properties": {
                    "query": {"type": "string", "description": "Search query with precise keywords. English queries often find more technical and scientific sources."},
                    "category": {"type": "string", "enum": ["general", "news", "science", "it"], "description": "Search category (default: general)."},
                    "time_range": {"type": "string", "enum": ["day", "week", "month", "year"], "description": "Optional: only recent results."},
                    "max_results": {"type": "integer", "description": "Number of results (1-20)."}
                }, "required": ["query"]}
            }}));
        }
        if self.cfg.search.enable_arxiv {
            v.push(json!({"type": "function", "function": {
                "name": "arxiv_search",
                "description": "Search arXiv for scientific papers and preprints. Returns title, authors, date, category, abstract and arXiv URL.",
                "parameters": {"type": "object", "properties": {
                    "query": {"type": "string", "description": "Keywords in English, e.g. 'LLM agent autonomy levels'."},
                    "max_results": {"type": "integer", "description": "Number of papers (1-15)."}
                }, "required": ["query"]}
            }}));
        }
        if self.cfg.search.enable_wikipedia {
            v.push(json!({"type": "function", "function": {
                "name": "wikipedia_search",
                "description": "Search Wikipedia for definitions and background knowledge. Returns the introductions of the best matching articles.",
                "parameters": {"type": "object", "properties": {
                    "query": {"type": "string", "description": "Search term or concept."},
                    "lang": {"type": "string", "description": "Wikipedia language code, e.g. 'de' or 'en'."}
                }, "required": ["query"]}
            }}));
        }
        v.push(json!({"type": "function", "function": {
            "name": "fetch_url",
            "description": "Fetch a web page and return its main text content. Use it to read the most promising sources in depth.",
            "parameters": {"type": "object", "properties": {
                "url": {"type": "string", "description": "Absolute http(s) URL."}
            }, "required": ["url"]}
        }}));
        v
    }

    pub async fn call(&self, name: &str, args: &Value) -> ToolOutput {
        let result = match name {
            "web_search" if self.web_enabled() => self.web_search(args).await,
            "web_search" => Err(anyhow!(tr!(
                self.ui,
                "Websuche ist nicht konfiguriert (SearXNG-URL fehlt). Nutze arxiv_search, wikipedia_search oder fetch_url.",
                "Web search is not configured (SearXNG URL missing). Use arxiv_search, wikipedia_search or fetch_url."
            ))),
            "arxiv_search" if self.cfg.search.enable_arxiv => self.arxiv_search(args).await,
            "wikipedia_search" if self.cfg.search.enable_wikipedia => self.wikipedia_search(args).await,
            "fetch_url" => self.fetch_url(args).await,
            other => Err(anyhow!(tr!(self.ui, "Unbekanntes oder deaktiviertes Tool: {}", "Unknown or disabled tool: {}", other))),
        };
        result.unwrap_or_else(|e| ToolOutput::error(format!("{e:#}")))
    }

    /// Registriert eine Quelle und meldet neue Quellen live an die Oberfläche.
    fn register(&self, title: &str, url: &str, kind: &str, meta: &str, snippet: &str) -> usize {
        let (n, new) = self.registry.lock().unwrap().add(title, url, kind, meta, snippet);
        if let Some(src) = new {
            self.em.emit(json!({"type": "source", "source": src}));
        }
        n
    }

    fn arg_str(&self, args: &Value, key: &str) -> Result<String> {
        args.get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| anyhow!(tr!(self.ui, "Parameter '{}' fehlt", "Parameter '{}' is missing", key)))
    }

    async fn web_search(&self, args: &Value) -> Result<ToolOutput> {
        let ui = self.ui;
        let query = self.arg_str(args, "query")?;
        let max = arg_usize(args, "max_results").unwrap_or(self.max_results).clamp(1, 20);
        let category = match args["category"].as_str().unwrap_or("general") {
            c @ ("news" | "science" | "it") => c,
            _ => "general",
        };
        let mut params: Vec<(&str, String)> = vec![
            ("q", query.clone()),
            ("format", "json".into()),
            ("categories", category.into()),
            ("safesearch", "0".into()),
        ];
        let lang = self.cfg.search.language.trim();
        if !lang.is_empty() && lang != "auto" {
            params.push(("language", lang.into()));
        }
        if let Some(range) = args["time_range"].as_str().filter(|t| matches!(*t, "day" | "week" | "month" | "year")) {
            params.push(("time_range", range.into()));
        }

        let url = format!("{}/search", self.cfg.search.searxng_url);
        let resp = self
            .http
            .get(&url)
            .query(&params)
            .header(reqwest::header::ACCEPT, "application/json")
            .timeout(Duration::from_secs(25))
            .send()
            .await
            .map_err(|e| anyhow!(tr!(ui, "SearXNG nicht erreichbar: {}", "SearXNG not reachable: {}", e)))?;
        if !resp.status().is_success() {
            bail!(tr!(
                ui,
                "SearXNG antwortete mit HTTP {} (ist 'json' in search.formats aktiviert?)",
                "SearXNG responded with HTTP {} (is 'json' enabled in search.formats?)",
                resp.status()
            ));
        }
        let v: Value = resp.json().await.map_err(|_| {
            anyhow!(tr!(
                ui,
                "SearXNG lieferte kein JSON – bitte 'json' in search.formats aktivieren",
                "SearXNG did not return JSON – please enable 'json' in search.formats"
            ))
        })?;

        let mut out = format!("Web search results for \"{query}\" (category: {category}):\n\n");
        let mut items = Vec::new();
        for r in v["results"].as_array().into_iter().flatten() {
            if items.len() >= max {
                break;
            }
            let Some(link) = r["url"].as_str().filter(|u| u.starts_with("http")) else { continue };
            let title = r["title"].as_str().unwrap_or(link);
            let snippet = r["content"].as_str().unwrap_or_default();
            let date: String = r["publishedDate"].as_str().unwrap_or_default().chars().take(10).collect();
            let n = self.register(title, link, "web", &date, snippet);
            let meta = [text::host_of(link), date.clone()].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" · ");
            out.push_str(&fmt_item(n, title, link, &meta, &text::truncate(snippet, 400)));
            items.push(json!({"n": n, "title": title, "url": link, "kind": "web", "meta": date}));
        }
        for a in v["answers"].as_array().into_iter().flatten().take(2) {
            if let Some(s) = a.as_str().or_else(|| a["answer"].as_str()) {
                out.push_str(&format!("\nDirect answer: {s}\n"));
            }
        }
        if items.is_empty() {
            out.push_str("No results. Try a different, broader or English query.\n");
        }
        let suggestions: Vec<&str> = v["suggestions"].as_array().into_iter().flatten().filter_map(|s| s.as_str()).take(5).collect();
        if !suggestions.is_empty() {
            out.push_str(&format!("\nRelated queries: {}\n", suggestions.join(", ")));
        }
        let count = items.len();
        Ok(ToolOutput::ok(out, tr!(ui, "{} Treffer", "{} results", count), count, "results", items, None))
    }

    async fn arxiv_search(&self, args: &Value) -> Result<ToolOutput> {
        let ui = self.ui;
        let query = self.arg_str(args, "query")?;
        let max = arg_usize(args, "max_results").unwrap_or(self.max_results.min(8)).clamp(1, 15);
        let terms: Vec<String> = query
            .split(|c: char| !c.is_alphanumeric() && c != '-')
            .filter(|w| w.chars().count() > 1)
            .take(8)
            .map(|w| format!("all:{w}"))
            .collect();
        if terms.is_empty() {
            bail!(tr!(ui, "Leere Suchanfrage", "Empty search query"));
        }
        let mut entries = self.arxiv_query(&terms.join(" AND "), max).await?;
        if entries.is_empty() && terms.len() > 3 {
            entries = self.arxiv_query(&terms[..3].join(" AND "), max).await?;
        }

        let mut out = format!("arXiv results for \"{query}\":\n\n");
        let mut items = Vec::new();
        for e in &entries {
            let authors = if e.authors.len() > 4 {
                format!("{} et al.", e.authors[..3].join(", "))
            } else {
                e.authors.join(", ")
            };
            let meta = [e.published.clone(), authors, e.category.clone()]
                .into_iter()
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(" · ");
            let n = self.register(&e.title, &e.url, "arxiv", &meta, &e.summary);
            out.push_str(&fmt_item(n, &e.title, &e.url, &format!("arXiv · {meta}"), &format!("Abstract: {}", text::truncate(&e.summary, 700))));
            items.push(json!({"n": n, "title": e.title, "url": e.url, "kind": "arxiv", "meta": meta}));
        }
        if items.is_empty() {
            out.push_str("No papers found. Try fewer or different English keywords.\n");
        }
        let count = items.len();
        Ok(ToolOutput::ok(out, tr!(ui, "{} Paper", "{} papers", count), count, "papers", items, None))
    }

    async fn arxiv_query(&self, search_query: &str, max: usize) -> Result<Vec<ArxivEntry>> {
        {
            // arXiv bittet um max. eine Anfrage alle 3 Sekunden
            let mut last = ARXIV_GATE.lock().await;
            if let Some(t) = *last {
                let wait = Duration::from_secs(3).saturating_sub(t.elapsed());
                if !wait.is_zero() {
                    tokio::time::sleep(wait).await;
                }
            }
            *last = Some(Instant::now());
        }
        let ui = self.ui;
        let max_s = max.to_string();
        let resp = self
            .http
            .get("https://export.arxiv.org/api/query")
            .query(&[("search_query", search_query), ("start", "0"), ("max_results", max_s.as_str()), ("sortBy", "relevance")])
            .timeout(Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| anyhow!(tr!(ui, "arXiv nicht erreichbar: {}", "arXiv not reachable: {}", e)))?;
        if !resp.status().is_success() {
            bail!(tr!(ui, "arXiv antwortete mit HTTP {}", "arXiv responded with HTTP {}", resp.status()));
        }
        let body = resp.text().await?;
        parse_arxiv(&body).map_err(|e| anyhow!(tr!(ui, "arXiv-Antwort nicht lesbar: {}", "Could not parse arXiv response: {}", e)))
    }

    async fn wikipedia_search(&self, args: &Value) -> Result<ToolOutput> {
        let ui = self.ui;
        let query = self.arg_str(args, "query")?;
        let lang = args["lang"]
            .as_str()
            .map(|l| l.trim().to_lowercase())
            .filter(|l| (2..=3).contains(&l.len()) && l.chars().all(|c| c.is_ascii_lowercase()))
            .unwrap_or_else(|| self.cfg.search.wikipedia_lang.clone());
        let api = format!("https://{lang}.wikipedia.org/w/api.php");
        let v: Value = self
            .http
            .get(&api)
            .query(&[("action", "query"), ("list", "search"), ("srsearch", query.as_str()), ("srlimit", "5"), ("format", "json"), ("utf8", "1")])
            .timeout(Duration::from_secs(20))
            .send()
            .await
            .map_err(|e| anyhow!(tr!(ui, "Wikipedia nicht erreichbar: {}", "Wikipedia not reachable: {}", e)))?
            .error_for_status()?
            .json()
            .await?;
        let hits: Vec<Value> = v["query"]["search"].as_array().cloned().unwrap_or_default();
        if hits.is_empty() {
            return Ok(ToolOutput::ok(
                format!("No Wikipedia ({lang}) articles found for \"{query}\". Try another term or language."),
                tr!(ui, "{} Artikel", "{} articles", 0),
                0,
                "articles",
                Vec::new(),
                None,
            ));
        }

        let titles: Vec<&str> = hits.iter().filter_map(|h| h["title"].as_str()).take(3).collect();
        let joined = titles.join("|");
        let mut extracts: HashMap<String, String> = HashMap::new();
        let ex = self
            .http
            .get(&api)
            .query(&[
                ("action", "query"),
                ("prop", "extracts"),
                ("exintro", "1"),
                ("explaintext", "1"),
                ("exlimit", "3"),
                ("titles", joined.as_str()),
                ("format", "json"),
                ("utf8", "1"),
            ])
            .timeout(Duration::from_secs(20))
            .send()
            .await;
        if let Ok(resp) = ex {
            if let Ok(ex) = resp.json::<Value>().await {
                for page in ex["query"]["pages"].as_object().into_iter().flat_map(|o| o.values()) {
                    if let (Some(t), Some(e)) = (page["title"].as_str(), page["extract"].as_str()) {
                        extracts.insert(t.to_string(), e.to_string());
                    }
                }
            }
        }

        let mut out = format!("Wikipedia ({lang}) results for \"{query}\":\n\n");
        let mut items = Vec::new();
        for h in &hits {
            let Some(title) = h["title"].as_str() else { continue };
            let link = format!("https://{lang}.wikipedia.org/wiki/{}", title.replace(' ', "_").replace('?', "%3F"));
            let snippet = extracts
                .get(title)
                .filter(|e| !e.trim().is_empty())
                .cloned()
                .unwrap_or_else(|| text::clean_inline(h["snippet"].as_str().unwrap_or_default()));
            let n = self.register(title, &link, "wikipedia", "", &snippet);
            out.push_str(&fmt_item(n, title, &link, &format!("Wikipedia ({lang})"), &text::truncate(&snippet, 1500)));
            items.push(json!({"n": n, "title": title, "url": link, "kind": "wikipedia", "meta": ""}));
        }
        let count = items.len();
        Ok(ToolOutput::ok(out, tr!(ui, "{} Artikel", "{} articles", count), count, "articles", items, None))
    }

    async fn fetch_url(&self, args: &Value) -> Result<ToolOutput> {
        let ui = self.ui;
        let mut url = self.arg_str(args, "url")?;
        if !url.starts_with("http://") && !url.starts_with("https://") {
            bail!(tr!(ui, "Nur absolute http(s)-URLs werden unterstützt", "Only absolute http(s) URLs are supported"));
        }
        if url.contains("arxiv.org/pdf/") {
            url = url.replace("/pdf/", "/abs/").trim_end_matches(".pdf").to_string();
        }
        let resp = self
            .http
            .get(&url)
            .header(reqwest::header::ACCEPT, "text/html,application/xhtml+xml,text/plain;q=0.9,*/*;q=0.5")
            .header(reqwest::header::ACCEPT_LANGUAGE, "de,en;q=0.8")
            .timeout(Duration::from_secs(25))
            .send()
            .await
            .map_err(|e| anyhow!(tr!(ui, "Seite nicht erreichbar: {}", "Page not reachable: {}", e)))?;
        let status = resp.status();
        if !status.is_success() {
            bail!(tr!(ui, "HTTP {} beim Abruf von {}", "HTTP {} while fetching {}", status, url));
        }
        let ct = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_lowercase();
        if ct.contains("pdf") {
            bail!(tr!(
                ui,
                "PDF-Dokumente können nicht gelesen werden – nutze Abstract oder Suchergebnis",
                "PDF documents cannot be read – use the abstract or search result instead"
            ));
        }
        let body = resp.text().await?;
        let body = text::clip_bytes(&body, 3_000_000);
        let (title, content) = if ct.contains("html") || body.trim_start().starts_with('<') {
            text::html_to_text(body)
        } else {
            (String::new(), body.to_string())
        };
        if content.chars().count() < 80 {
            bail!(tr!(
                ui,
                "Kein lesbarer Text gefunden (evtl. eine JavaScript-Seite)",
                "No readable text found (possibly a JavaScript-only page)"
            ));
        }
        let max = self.cfg.agent.fetch_max_chars;
        let n = self.register(if title.is_empty() { &url } else { &title }, &url, "page", "", &content);
        let shown_title = if title.is_empty() { url.clone() } else { title.clone() };
        let out = format!("Content of [{n}] {shown_title}\nURL: {url}\n\n{}", text::truncate(&content, max));
        let chars = content.chars().count().min(max);
        Ok(ToolOutput::ok(
            out,
            tr!(ui, "{} Zeichen gelesen", "{} characters read", chars),
            chars,
            "chars",
            vec![json!({"n": n, "title": shown_title, "url": url, "kind": "page", "meta": ""})],
            Some(text::truncate(&content, 700)),
        ))
    }
}

fn parse_arxiv(xml: &str) -> Result<Vec<ArxivEntry>> {
    let doc = roxmltree::Document::parse(xml)?;
    let mut out = Vec::new();
    for e in doc.descendants().filter(|n| n.has_tag_name("entry")) {
        let child_text = |name: &str| {
            e.children()
                .find(|c| c.has_tag_name(name))
                .and_then(|c| c.text())
                .map(text::collapse_ws)
                .unwrap_or_default()
        };
        let id = child_text("id");
        if id.is_empty() || id.contains("/api/errors") {
            continue;
        }
        let authors = e
            .children()
            .filter(|c| c.has_tag_name("author"))
            .filter_map(|a| a.children().find(|c| c.has_tag_name("name")).and_then(|c| c.text()))
            .map(|s| s.trim().to_string())
            .collect();
        let category = e
            .children()
            .find(|c| c.has_tag_name("primary_category"))
            .and_then(|c| c.attribute("term"))
            .unwrap_or_default()
            .to_string();
        out.push(ArxivEntry {
            url: id.replace("http://", "https://"),
            title: child_text("title"),
            summary: child_text("summary"),
            published: child_text("published").chars().take(10).collect(),
            authors,
            category,
        });
    }
    Ok(out)
}

fn fmt_item(n: usize, title: &str, url: &str, meta: &str, snippet: &str) -> String {
    let mut s = format!("[{n}] {title}\n    URL: {url}\n");
    if !meta.is_empty() {
        s.push_str(&format!("    {meta}\n"));
    }
    if !snippet.is_empty() {
        s.push_str(&format!("    {snippet}\n"));
    }
    s.push('\n');
    s
}

fn arg_usize(args: &Value, key: &str) -> Option<usize> {
    let v = &args[key];
    v.as_u64().or_else(|| v.as_str().and_then(|s| s.trim().parse().ok())).map(|n| n as usize)
}
