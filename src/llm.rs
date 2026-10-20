//! OpenAI-kompatibler Chat-Client (vLLM / llama-swap) mit Tool-Calls und Streaming.

use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use futures::StreamExt;
use serde_json::{json, Value};

use crate::{config::LlmCfg, i18n::Lang, text};

#[derive(Clone)]
pub struct Llm {
    http: reqwest::Client,
    cfg: LlmCfg,
    ui: Lang,
}

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub args: Value,
    /// Tool-Call im Originalformat für die Nachrichtenhistorie
    pub raw: Value,
}

#[derive(Debug, Default)]
pub struct Reply {
    pub content: String,
    pub reasoning: String,
    pub tool_calls: Vec<ToolCall>,
}

impl Llm {
    pub fn new(http: reqwest::Client, cfg: LlmCfg, ui: Lang) -> Self {
        Self { http, cfg, ui }
    }

    async fn send(&self, body: &Value, timeout: Duration) -> Result<reqwest::Response> {
        let url = format!("{}/chat/completions", self.cfg.base_url);
        let mut last_err = None;
        for attempt in 0..3u64 {
            if attempt > 0 {
                tokio::time::sleep(Duration::from_secs(2 + attempt * 2)).await;
            }
            let mut req = self.http.post(&url).json(body).timeout(timeout);
            if !self.cfg.api_key.is_empty() {
                req = req.bearer_auth(&self.cfg.api_key);
            }
            match req.send().await {
                Ok(r) if r.status().is_success() => return Ok(r),
                Ok(r) => {
                    let status = r.status();
                    let body = r.text().await.unwrap_or_default();
                    let err = anyhow!(tr!(
                        self.ui,
                        "LLM-Server antwortete mit HTTP {}: {}",
                        "LLM server responded with HTTP {}: {}",
                        status,
                        text::truncate(&body, 600)
                    ));
                    if status.is_server_error() || status.as_u16() == 429 {
                        last_err = Some(err);
                        continue;
                    }
                    return Err(err);
                }
                Err(e) => {
                    let timeout = e.is_timeout();
                    last_err = Some(anyhow!(tr!(
                        self.ui,
                        "LLM nicht erreichbar ({}): {}",
                        "LLM not reachable ({}): {}",
                        self.cfg.base_url,
                        e
                    )));
                    if timeout {
                        break;
                    }
                }
            }
        }
        Err(last_err.unwrap_or_else(|| anyhow!(tr!(self.ui, "LLM-Anfrage fehlgeschlagen", "LLM request failed"))))
    }

    /// Nicht-streamender Aufruf, optional mit Tools (`tool_choice`: None = auto).
    pub async fn chat(
        &self,
        messages: &[Value],
        tools: Option<&[Value]>,
        tool_choice: Option<&str>,
        temperature: f64,
        max_tokens: u32,
    ) -> Result<Reply> {
        let mut body = json!({
            "model": self.cfg.model,
            "messages": messages,
            "temperature": temperature,
            "max_tokens": max_tokens,
            "stream": false,
        });
        if let Some(tools) = tools.filter(|t| !t.is_empty()) {
            body["tools"] = json!(tools);
            if let Some(choice) = tool_choice {
                body["tool_choice"] = json!(choice);
            }
        }
        let resp = self.send(&body, Duration::from_secs(900)).await?;
        let v: Value = resp
            .json()
            .await
            .context(tr!(self.ui, "Ungültige JSON-Antwort vom LLM", "Invalid JSON response from the LLM"))?;
        if let Some(err) = v.get("error") {
            bail!(tr!(self.ui, "LLM-Fehler: {}", "LLM error: {}", err));
        }
        let msg = &v["choices"][0]["message"];
        let mut reply = Reply {
            content: msg["content"].as_str().unwrap_or_default().to_string(),
            reasoning: msg["reasoning_content"]
                .as_str()
                .or_else(|| msg["reasoning"].as_str())
                .unwrap_or_default()
                .to_string(),
            tool_calls: Vec::new(),
        };
        if let Some(calls) = msg["tool_calls"].as_array() {
            for (i, tc) in calls.iter().enumerate() {
                let name = tc["function"]["name"].as_str().unwrap_or_default().to_string();
                if name.is_empty() {
                    continue;
                }
                let id = tc["id"].as_str().map(String::from).unwrap_or_else(|| format!("call_{i}"));
                let raw_args = &tc["function"]["arguments"];
                let args = match raw_args {
                    Value::String(s) => serde_json::from_str(s).unwrap_or_else(|_| json!({ "_raw": s })),
                    other => other.clone(),
                };
                let args_str = raw_args.as_str().map(String::from).unwrap_or_else(|| args.to_string());
                let raw = json!({
                    "id": id,
                    "type": "function",
                    "function": { "name": name, "arguments": args_str },
                });
                reply.tool_calls.push(ToolCall { id, name, args, raw });
            }
        }
        Ok(reply)
    }

    /// Streamender Aufruf ohne Tools; `on_delta(text, is_reasoning)` wird pro Token-Stück aufgerufen.
    pub async fn chat_stream<F>(&self, messages: &[Value], temperature: f64, max_tokens: u32, mut on_delta: F) -> Result<Reply>
    where
        F: FnMut(&str, bool) + Send,
    {
        let body = json!({
            "model": self.cfg.model,
            "messages": messages,
            "temperature": temperature,
            "max_tokens": max_tokens,
            "stream": true,
        });
        let resp = self.send(&body, Duration::from_secs(1800)).await?;
        let mut stream = resp.bytes_stream();
        let mut buf: Vec<u8> = Vec::new();
        let mut reply = Reply::default();

        'outer: while let Some(chunk) = stream.next().await {
            let chunk = chunk.context(tr!(self.ui, "Stream vom LLM abgebrochen", "LLM stream aborted"))?;
            buf.extend_from_slice(&chunk);
            while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                let raw: Vec<u8> = buf.drain(..=pos).collect();
                let line = String::from_utf8_lossy(&raw);
                let Some(data) = line.trim().strip_prefix("data:") else { continue };
                let data = data.trim();
                if data == "[DONE]" {
                    break 'outer;
                }
                let Ok(v) = serde_json::from_str::<Value>(data) else { continue };
                if let Some(err) = v.get("error") {
                    bail!(tr!(self.ui, "LLM-Fehler: {}", "LLM error: {}", err));
                }
                let delta = &v["choices"][0]["delta"];
                if let Some(r) = delta["reasoning_content"].as_str().or_else(|| delta["reasoning"].as_str()) {
                    if !r.is_empty() {
                        reply.reasoning.push_str(r);
                        on_delta(r, true);
                    }
                }
                if let Some(c) = delta["content"].as_str() {
                    if !c.is_empty() {
                        reply.content.push_str(c);
                        on_delta(c, false);
                    }
                }
            }
        }
        Ok(reply)
    }
}
