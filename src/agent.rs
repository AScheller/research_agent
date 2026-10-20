//! Der reflektierende Research-Workflow:
//! Planer → Research-Agenten (Tool-Nutzung) → Writer (Entwurf) → Editor (Reflexion) → finaler Bericht.

use std::{collections::BTreeSet, sync::LazyLock, time::Instant};

use anyhow::Result;
use futures::future::join_all;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    config::Config,
    i18n::Lang,
    llm::{Llm, Reply},
    store::Emitter,
    text,
    tools::{Source, Tools},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunOptions {
    pub depth: String,
    /// Sprache des Berichts: `de`, `en` oder `auto`
    pub language: String,
    /// Sprache der Oberfläche (Meldungen, Fehler)
    #[serde(default)]
    pub ui_lang: Lang,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentKind {
    Research,
    Writer,
    Editor,
}

#[derive(Clone, Debug, Serialize)]
pub struct Step {
    pub index: usize,
    pub agent: AgentKind,
    pub task: String,
    #[serde(rename = "final")]
    pub is_final: bool,
}

struct Note {
    task: String,
    output: String,
}

struct StepOut {
    text: String,
    review: Option<String>,
    reasoning: Option<String>,
}

struct Ctx {
    cfg: Config,
    llm: Llm,
    tools: Tools,
    em: Emitter,
    prompt: String,
    lang: &'static str,
    english: bool,
    max_steps: usize,
    rounds: usize,
    date: String,
}

pub async fn run(cfg: Config, http: reqwest::Client, em: Emitter, prompt: String, opts: RunOptions) -> Result<String> {
    let (max_steps, rounds, max_results) = match opts.depth.as_str() {
        "quick" => (4, 3, 5),
        "deep" => (cfg.agent.max_steps.max(9), cfg.agent.max_tool_rounds.max(8), cfg.search.max_results.max(10)),
        _ => (cfg.agent.max_steps, cfg.agent.max_tool_rounds, cfg.search.max_results),
    };
    let (lang, english) = match opts.language.as_str() {
        "en" => ("English", true),
        "de" => ("German (Deutsch)", false),
        _ => ("the same language as the research question", opts.ui_lang == Lang::En),
    };
    // Wikipedia-Standardsprache folgt der Berichtssprache (bei „auto“ gilt die Konfiguration)
    let mut tool_cfg = cfg.clone();
    if matches!(opts.language.as_str(), "de" | "en") {
        tool_cfg.search.wikipedia_lang = opts.language.clone();
    }
    let ctx = Ctx {
        llm: Llm::new(http.clone(), cfg.llm.clone(), opts.ui_lang),
        tools: Tools::new(http, tool_cfg, opts.ui_lang, max_results, em.clone()),
        em: em.clone(),
        prompt,
        lang,
        english,
        max_steps,
        rounds,
        date: chrono::Local::now().format("%Y-%m-%d").to_string(),
        cfg,
    };
    em.emit(json!({
        "type": "run_start",
        "prompt": ctx.prompt,
        "model": ctx.cfg.llm.model,
        "options": opts,
        "tools": ctx.tools.names(),
        "web_search": ctx.tools.web_enabled(),
    }));
    ctx.execute().await
}

fn msg(role: &str, content: &str) -> Value {
    json!({"role": role, "content": content})
}

fn non_empty(s: String) -> Option<String> {
    let t = s.trim();
    (!t.is_empty()).then(|| t.to_string())
}

impl Ctx {
    fn t(&self, de: &str, en: &str) -> String {
        if self.english { en } else { de }.to_string()
    }

    async fn execute(&self) -> Result<String> {
        self.em.emit(json!({"type": "phase", "phase": "planning"}));
        let t0 = Instant::now();
        let steps = self.plan().await?;
        self.em.emit(json!({"type": "plan", "steps": steps, "duration_ms": t0.elapsed().as_millis() as u64}));

        let mut notes: Vec<Note> = Vec::new();
        let mut draft: Option<String> = None;
        let mut report = String::new();

        for step in &steps {
            self.em.emit(json!({"type": "step_start", "index": step.index}));
            let started = Instant::now();
            let result = if step.is_final {
                self.final_report(step, &notes, draft.as_deref()).await
            } else {
                match step.agent {
                    AgentKind::Research => self.research(step, &notes).await,
                    AgentKind::Writer => self.writer(step, &notes, draft.as_deref()).await,
                    AgentKind::Editor => self.editor(step, &notes, draft.as_deref()).await,
                }
            };
            let ms = started.elapsed().as_millis() as u64;
            match result {
                Ok(out) => {
                    self.em.emit(json!({
                        "type": "step_done",
                        "index": step.index,
                        "output": out.text,
                        "review": out.review,
                        "reasoning": out.reasoning,
                        "duration_ms": ms,
                    }));
                    if step.is_final {
                        report = out.text;
                    } else if step.agent == AgentKind::Research {
                        notes.push(Note { task: step.task.clone(), output: out.text });
                    } else {
                        draft = Some(out.text);
                    }
                }
                Err(e) => {
                    tracing::warn!("Schritt {} fehlgeschlagen: {e:#}", step.index + 1);
                    self.em.emit(json!({
                        "type": "step_error",
                        "index": step.index,
                        "message": format!("{e:#}"),
                        "duration_ms": ms,
                    }));
                    if step.is_final {
                        return Err(e);
                    }
                }
            }
        }

        let full = self.with_references(&report);
        self.em.emit(json!({"type": "report", "markdown": full}));
        Ok(full)
    }

    // ------------------------------------------------------------------ Planung

    async fn plan(&self) -> Result<Vec<Step>> {
        let sys = format!(
            r#"You are the planning agent of a multi-agent research system. Today is {date}.
Available agents:
- "research": searches and reads sources with these tools: {tools}.
- "writer": writes text (drafts) based on the collected findings.
- "editor": critically reviews a draft (clarity, coherence, coverage, citation accuracy) and improves it.

Create a concise plan of 3 to {max} steps for the user's research question:
1. Start with research steps: first a broad search, then targeted searches (e.g. scientific papers on arXiv, definitions on Wikipedia, recent developments, practical examples), then one step that ranks the sources by relevance and authority and extracts the key facts, definitions and numbers.
2. Then exactly one "writer" step (initial draft) followed by exactly one "editor" step (review and refine).
3. Do NOT add a final report step - it is appended automatically.
Each task is ONE specific, actionable sentence written in {lang}, tailored to the question (name concrete aspects, not generic phrases).
Answer ONLY with JSON in this format: {{"steps":[{{"agent":"research","task":"..."}}]}}"#,
            date = self.date,
            tools = self.tools.describe(),
            max = self.max_steps,
            lang = self.lang,
        );
        let messages = [msg("system", &sys), msg("user", &format!("Research question:\n{}", self.prompt))];
        let reply = self.llm.chat(&messages, None, None, 0.2, 2048).await?;
        let mut raw = parse_plan(&reply.content).unwrap_or_default();
        if raw.iter().all(|(_, t)| t.trim().is_empty()) {
            tracing::warn!("Plan nicht lesbar: {}", text::truncate(&reply.content, 400));
            self.em.emit(json!({
                "type": "notice",
                "code": "plan_fallback",
                "message": "Plan not readable – using the default plan.",
            }));
            raw = self.default_plan();
        }
        Ok(self.normalize_plan(raw))
    }

    fn default_plan(&self) -> Vec<(AgentKind, String)> {
        use AgentKind::*;
        let mut v = Vec::new();
        if self.tools.web_enabled() {
            v.push((Research, self.t(
                "Führe eine breite Websuche zur Fragestellung durch und sammle die relevantesten Quellen (Titel, Autoren, Jahr, URL).",
                "Run a broad web search on the question and collect the most relevant sources (title, authors, year, URL).",
            )));
        } else {
            v.push((Research, self.t(
                "Recherchiere Grundlagen und zentrale Definitionen zur Fragestellung auf Wikipedia.",
                "Research the fundamentals and key definitions of the question on Wikipedia.",
            )));
        }
        if self.cfg.search.enable_arxiv {
            v.push((Research, self.t(
                "Suche auf arXiv nach wissenschaftlichen Arbeiten zum Thema und halte deren Kernergebnisse fest.",
                "Search arXiv for scientific papers on the topic and record their key results.",
            )));
        }
        v.push((Research, self.t(
            "Bewerte die gesammelten Quellen nach Relevanz und Autorität, lies die wichtigsten im Detail und extrahiere zentrale Fakten, Definitionen und Zahlen.",
            "Rank the collected sources by relevance and authority, read the most important ones in depth and extract key facts, definitions and numbers.",
        )));
        v.push((Writer, self.t(
            "Erstelle einen ersten Entwurf des Berichts mit allen zentralen Erkenntnissen und Inline-Zitaten [n].",
            "Draft an initial report with all key findings and inline citations [n].",
        )));
        v.push((Editor, self.t(
            "Prüfe den Entwurf auf Klarheit, Kohärenz, Vollständigkeit und Zitiergenauigkeit und überarbeite ihn.",
            "Review the draft for clarity, coherence, completeness and citation accuracy, and refine it.",
        )));
        v
    }

    /// Erzwingt die Struktur Research… → Writer → Editor → finaler Writer.
    fn normalize_plan(&self, raw: Vec<(AgentKind, String)>) -> Vec<Step> {
        let raw: Vec<(AgentKind, String)> = raw
            .into_iter()
            .map(|(a, t)| (a, t.trim().to_string()))
            .filter(|(_, t)| !t.is_empty())
            .collect();
        let defaults = self.default_plan();
        let pick = |kind: AgentKind| {
            raw.iter()
                .find(|(a, _)| *a == kind)
                .or_else(|| defaults.iter().find(|(a, _)| *a == kind))
                .map(|(_, t)| t.clone())
                .unwrap_or_default()
        };
        let mut research: Vec<String> = raw.iter().filter(|(a, _)| *a == AgentKind::Research).map(|(_, t)| t.clone()).collect();
        if research.is_empty() {
            research = defaults.iter().filter(|(a, _)| *a == AgentKind::Research).map(|(_, t)| t.clone()).collect();
        }
        research.truncate(self.max_steps.saturating_sub(2).max(1));

        let mut tasks: Vec<(AgentKind, String, bool)> = research.into_iter().map(|t| (AgentKind::Research, t, false)).collect();
        tasks.push((AgentKind::Writer, pick(AgentKind::Writer), false));
        tasks.push((AgentKind::Editor, pick(AgentKind::Editor), false));
        tasks.push((
            AgentKind::Writer,
            self.t(
                "Erstelle einen umfassenden Markdown-Bericht, der alle Erkenntnisse und Ergebnisse der vorherigen Schritte nutzt, Inline-Zitate [n] enthält, alle früheren Quellen bewahrt und detailliert sowie in sich geschlossen ist.",
                "Generate a comprehensive Markdown report that uses all findings and outputs from previous steps, includes inline citations [n], preserves earlier sources, and is detailed and self-contained.",
            ),
            true,
        ));
        tasks
            .into_iter()
            .enumerate()
            .map(|(index, (agent, task, is_final))| Step { index, agent, task, is_final })
            .collect()
    }

    // ------------------------------------------------------------------ Kontext

    fn source_list(&self, with_snippets: bool) -> String {
        let reg = self.tools.registry.lock().unwrap();
        reg.items
            .iter()
            .map(|s| {
                let mut line = format!("[{}] {} — {}", s.n, s.title, s.url);
                if !s.meta.is_empty() {
                    line.push_str(&format!(" ({})", s.meta));
                }
                if with_snippets && !s.snippet.is_empty() {
                    line.push_str(&format!("\n    {}", text::truncate(&s.snippet, 280)));
                }
                line
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn context_block(&self, notes: &[Note], with_snippets: bool) -> String {
        let mut s = format!("## Research question\n{}\n", self.prompt);
        if !notes.is_empty() {
            s.push_str("\n## Findings from previous research steps\n");
            for (k, n) in notes.iter().enumerate() {
                s.push_str(&format!("\n### Research step {}: {}\n{}\n", k + 1, n.task, text::truncate(&n.output, 8000)));
            }
        }
        let list = self.source_list(with_snippets);
        if !list.is_empty() {
            s.push_str(&format!("\n## Numbered source list (cite only these numbers)\n{list}\n"));
        }
        s
    }

    // ------------------------------------------------------------------ Agenten

    async fn research(&self, step: &Step, notes: &[Note]) -> Result<StepOut> {
        let schemas = self.tools.schemas();
        let sys = format!(
            r#"You are a meticulous research agent in a multi-agent research system. Today is {date}.
Complete the CURRENT TASK using your tools: {tools}.

Guidelines:
- Plan your searches: start broad, then refine with specific keywords. Vary your queries and also search in English for technical or scientific topics.
- Prefer authoritative sources: peer-reviewed papers, arXiv preprints, official documentation, recognized institutions, reputable media.
- Use fetch_url to read the 1-3 most promising pages in depth when snippets are not enough.
- You may call several tools in parallel. You have at most {rounds} tool rounds.
- Every source in the tool results has a number like [7]. Cite ONLY these numbers. Never invent sources, titles or URLs.
- When you have enough information, stop calling tools and answer with a structured Markdown summary in {lang}: the key findings as bullet points (definitions, facts, numbers, dates, names, methods), each with its citation(s) [n], followed by a short list of the most relevant sources for this task (number + why it matters). No introduction, no closing remarks."#,
            date = self.date,
            tools = self.tools.names().join(", "),
            rounds = self.rounds,
            lang = self.lang,
        );
        let user = format!(
            "{}\n## Current task (step {})\n{}",
            self.context_block(notes, false),
            step.index + 1,
            step.task
        );
        let mut messages = vec![msg("system", &sys), msg("user", &user)];
        let mut reasoning = String::new();
        let mut used_tools = false;
        let mut nudged = false;

        for round in 0..self.rounds {
            let reply = self.llm.chat(&messages, Some(&schemas), None, 0.3, 4096).await?;
            push_reasoning(&mut reasoning, &reply);
            if reply.tool_calls.is_empty() {
                // Ein Research-Schritt soll recherchieren und nicht nur Vorhandenes umformulieren.
                if !used_tools && !nudged && round + 1 < self.rounds {
                    nudged = true;
                    messages.push(json!({"role": "assistant", "content": reply.content}));
                    messages.push(msg(
                        "user",
                        "Before you answer: use your tools at least once for this task to verify and deepen the findings - e.g. read the 1-3 most relevant sources in full with fetch_url, or run targeted searches for aspects that are still missing. Then write your final summary.",
                    ));
                    continue;
                }
                if !reply.content.trim().is_empty() {
                    return Ok(StepOut { text: reply.content, review: None, reasoning: non_empty(reasoning) });
                }
                break;
            }
            used_tools = true;
            if !reply.content.trim().is_empty() {
                self.em.emit(json!({"type": "thought", "index": step.index, "text": reply.content}));
            }
            let content = if reply.content.trim().is_empty() { Value::Null } else { Value::String(reply.content.clone()) };
            let calls: Vec<Value> = reply.tool_calls.iter().map(|c| c.raw.clone()).collect();
            messages.push(json!({"role": "assistant", "content": content, "tool_calls": calls}));

            let results = join_all(reply.tool_calls.iter().map(|call| async move {
                self.em.emit(json!({
                    "type": "tool_call",
                    "index": step.index,
                    "id": call.id,
                    "name": call.name,
                    "args": call.args,
                    "round": round + 1,
                }));
                let t = Instant::now();
                let out = self.tools.call(&call.name, &call.args).await;
                self.em.emit(json!({
                    "type": "tool_result",
                    "index": step.index,
                    "id": call.id,
                    "name": call.name,
                    "ok": out.ok,
                    "summary": out.summary,
                    "count": out.count,
                    "unit": out.unit,
                    "items": out.items,
                    "preview": out.preview,
                    "duration_ms": t.elapsed().as_millis() as u64,
                }));
                (call.id.clone(), out.text)
            }))
            .await;
            for (id, content) in results {
                messages.push(json!({"role": "tool", "tool_call_id": id, "content": content}));
            }
        }

        messages.push(msg(
            "user",
            "The tool budget for this step is used up. Do not call any more tools. Now write your final structured summary for the current task, citing sources with [n].",
        ));
        let reply = self.llm.chat(&messages, Some(&schemas), Some("none"), 0.3, 4096).await?;
        push_reasoning(&mut reasoning, &reply);
        Ok(StepOut { text: reply.content, review: None, reasoning: non_empty(reasoning) })
    }

    /// Streamt eine Writer-/Editor-Antwort Token für Token an die Oberfläche.
    async fn stream(&self, index: usize, sys: &str, user: &str) -> Result<Reply> {
        let messages = [msg("system", sys), msg("user", user)];
        let em = &self.em;
        self.llm
            .chat_stream(&messages, self.cfg.llm.temperature, self.cfg.llm.max_tokens, |delta, is_reasoning| {
                let kind = if is_reasoning { "reasoning" } else { "token" };
                em.emit(json!({"type": kind, "index": index, "delta": delta}));
            })
            .await
    }

    async fn writer(&self, step: &Step, notes: &[Note], draft: Option<&str>) -> Result<StepOut> {
        let sys = format!(
            "You are an expert research writer. Today is {}. Write in {}.\n\
Base every statement strictly on the provided findings and sources and cite them inline with their numbers in square brackets, e.g. [3] or [2][5]. \
Never invent sources, numbers or URLs. Do not add a reference list. \
Use a clear Markdown structure (headings, bullet points, tables where useful). Output only the text itself.",
            self.date, self.lang
        );
        let mut user = self.context_block(notes, true);
        if let Some(d) = draft {
            user.push_str(&format!("\n## Previous draft\n{d}\n"));
        }
        user.push_str(&format!("\n## Your task\n{}\n", step.task));
        let reply = self.stream(step.index, &sys, &user).await?;
        Ok(StepOut { text: text::unwrap_fence(&reply.content), review: None, reasoning: non_empty(reply.reasoning) })
    }

    async fn editor(&self, step: &Step, notes: &[Note], draft: Option<&str>) -> Result<StepOut> {
        let Some(draft) = draft else {
            // Ohne Entwurf gibt es nichts zu prüfen – dann zuerst schreiben.
            return self.writer(step, notes, None).await;
        };
        let sys = format!(
            r#"You are a critical senior editor for research reports. Today is {date}. Write in {lang}.
Review the draft for: clarity and readability, logical structure and coherence, coverage of all aspects of the research question, consistency with the findings, and citation accuracy (every [n] must exist in the source list and support the claim; important claims without citation need one).
Answer in exactly this format:
<review>
- 4 to 8 bullet points: concrete problems you found and how you fixed them
</review>
<revised>
the complete improved draft in Markdown (keep all valid citations [n], no reference list)
</revised>"#,
            date = self.date,
            lang = self.lang,
        );
        let user = format!(
            "{}\n## Draft to review\n{draft}\n\n## Your task\n{}\n",
            self.context_block(notes, true),
            step.task
        );
        let reply = self.stream(step.index, &sys, &user).await?;
        let (review, revised) = split_review(&reply.content);
        let revised = if revised.chars().count() < 200 { draft.to_string() } else { revised };
        Ok(StepOut { text: revised, review, reasoning: non_empty(reply.reasoning) })
    }

    async fn final_report(&self, step: &Step, notes: &[Note], draft: Option<&str>) -> Result<StepOut> {
        let sys = format!(
            r#"You are an expert research writer producing the final deliverable. Today is {date}. Write in {lang}.
Write a comprehensive, detailed and self-contained research report in Markdown that uses ALL findings and outputs from the previous steps.
Structure:
- a level-1 heading (# Title) with a precise title
- a short summary section (3-5 sentences)
- several level-2 sections (## Heading), with level-3 subsections where useful, that cover every aspect of the research question in depth; use tables to compare approaches, levels, metrics or options where helpful
- a section with conclusions / practical recommendations
- a short section with open questions and outlook
Rules:
- Cite every non-trivial claim inline with the source numbers in square brackets, e.g. [4] or [2][7]. Use only numbers from the source list. Never invent sources or URLs.
- Preserve the sources used in earlier steps; prefer authoritative sources.
- Do NOT write a references or sources list at the end - it is appended automatically.
- Output only the report itself, without preamble."#,
            date = self.date,
            lang = self.lang,
        );
        let mut user = self.context_block(notes, true);
        if let Some(d) = draft {
            user.push_str(&format!("\n## Edited draft (use it as the basis, then extend and improve it)\n{d}\n"));
        }
        user.push_str(&format!("\n## Your task\n{}\n", step.task));
        let reply = self.stream(step.index, &sys, &user).await?;
        let report = strip_reference_section(&text::unwrap_fence(&reply.content));
        Ok(StepOut { text: report, review: None, reasoning: non_empty(reply.reasoning) })
    }

    /// Hängt ein Quellenverzeichnis aus dem Register an (nur tatsächlich zitierte Quellen).
    fn with_references(&self, report: &str) -> String {
        let reg = self.tools.registry.lock().unwrap();
        if reg.items.is_empty() {
            return report.to_string();
        }
        let cited = cited_numbers(report);
        let list: Vec<&Source> = if cited.is_empty() {
            reg.items.iter().collect()
        } else {
            cited.iter().filter_map(|n| reg.get(*n)).collect()
        };
        let heading = if self.english { "References" } else { "Quellen" };
        let mut out = report.trim_end().to_string();
        out.push_str(&format!("\n\n## {heading}\n\n"));
        for s in list {
            out.push_str(&format!("- [{}] [{}]({}){}\n", s.n, md_title(&s.title), md_url(&s.url), ref_suffix(s)));
        }
        out
    }
}

fn push_reasoning(acc: &mut String, reply: &Reply) {
    if !reply.reasoning.trim().is_empty() {
        acc.push_str(reply.reasoning.trim());
        acc.push_str("\n\n");
    }
}

// ---------------------------------------------------------------------- Parsing

fn agent_from(s: &str) -> AgentKind {
    let l = s.to_lowercase();
    if l.contains("edit") {
        AgentKind::Editor
    } else if l.contains("writ") || l.contains("schreib") {
        AgentKind::Writer
    } else {
        AgentKind::Research
    }
}

fn extract_json(s: &str) -> Option<Value> {
    if let Ok(v) = serde_json::from_str::<Value>(s.trim()) {
        return Some(v);
    }
    for (open, close) in [('{', '}'), ('[', ']')] {
        if let (Some(a), Some(b)) = (s.find(open), s.rfind(close)) {
            if a < b {
                if let Ok(v) = serde_json::from_str::<Value>(&s[a..=b]) {
                    return Some(v);
                }
            }
        }
    }
    None
}

fn parse_plan(text: &str) -> Option<Vec<(AgentKind, String)>> {
    let v = extract_json(text)?;
    let arr = match &v {
        Value::Array(a) => a.clone(),
        _ => v.get("steps")?.as_array()?.clone(),
    };
    let mut out = Vec::new();
    for item in &arr {
        if let Some(s) = item.as_str() {
            out.push((agent_from(s), s.to_string()));
            continue;
        }
        let agent = ["agent", "role"].iter().find_map(|k| item.get(*k).and_then(|x| x.as_str())).unwrap_or("research");
        let task = ["task", "description", "step"].iter().find_map(|k| item.get(*k).and_then(|x| x.as_str())).unwrap_or_default();
        out.push((agent_from(agent), task.to_string()));
    }
    Some(out)
}

fn between(s: &str, open: &str, close: &str) -> Option<String> {
    let start = s.find(open)? + open.len();
    let end = s[start..].find(close).map(|j| start + j).unwrap_or(s.len());
    Some(s[start..end].to_string())
}

/// Trennt die Editor-Antwort in Review-Notizen und überarbeiteten Entwurf.
fn split_review(s: &str) -> (Option<String>, String) {
    let review = between(s, "<review>", "</review>").map(|r| r.trim().to_string()).filter(|r| !r.is_empty());
    if let Some(revised) = between(s, "<revised>", "</revised>").filter(|r| !r.trim().is_empty()) {
        return (review, text::unwrap_fence(&revised));
    }
    let rest = match s.find("</review>") {
        Some(i) => &s[i + "</review>".len()..],
        None => s,
    };
    (review, text::unwrap_fence(rest))
}

static RE_CITE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[(\d{1,3}(?:\s*[,;–-]\s*\d{1,3})*)\]").unwrap());
static RE_REFS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?im)^#{1,3}\s*(quellen|quellenverzeichnis|referenzen|literatur|literaturverzeichnis|references|sources|bibliography)\b.*$").unwrap()
});

fn cited_numbers(s: &str) -> BTreeSet<usize> {
    let mut set = BTreeSet::new();
    for c in RE_CITE.captures_iter(s) {
        for part in c[1].split(|ch| ch == ',' || ch == ';') {
            let nums: Vec<usize> = part.split(|ch| ch == '–' || ch == '-').filter_map(|x| x.trim().parse().ok()).collect();
            match nums.as_slice() {
                [a, b] if a <= b && b - a <= 20 => set.extend(*a..=*b),
                [a, ..] => {
                    set.insert(*a);
                }
                _ => {}
            }
        }
    }
    set
}

/// Entfernt ein vom Modell trotz Anweisung geschriebenes Quellenverzeichnis am Ende.
fn strip_reference_section(s: &str) -> String {
    match RE_REFS.find_iter(s).last() {
        Some(m) if m.start() > s.len() / 2 => s[..m.start()].trim_end().to_string(),
        _ => s.to_string(),
    }
}

fn md_title(s: &str) -> String {
    s.replace('[', "(").replace(']', ")")
}

fn md_url(s: &str) -> String {
    s.replace(' ', "%20").replace('(', "%28").replace(')', "%29")
}

fn ref_suffix(s: &Source) -> String {
    let mut parts: Vec<String> = Vec::new();
    match s.kind.as_str() {
        "arxiv" => parts.push("arXiv".into()),
        "wikipedia" => parts.push("Wikipedia".into()),
        _ => {
            let host = text::host_of(&s.url);
            if !host.is_empty() {
                parts.push(host);
            }
        }
    }
    if !s.meta.is_empty() {
        parts.push(s.meta.clone());
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!(" — {}", parts.join(" · "))
    }
}
