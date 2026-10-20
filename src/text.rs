//! Text-Hilfsfunktionen: Kürzen, HTML → Text, Entities, Markdown-Zäune.

use std::sync::LazyLock;

use regex::Regex;

static RE_DROP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?is)<script\b.*?</script\s*>|<style\b.*?</style\s*>|<noscript\b.*?</noscript\s*>|<svg\b.*?</svg\s*>|<template\b.*?</template\s*>|<iframe\b.*?</iframe\s*>|<!--.*?-->",
    )
    .unwrap()
});
static RE_CHROME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)<nav\b.*?</nav\s*>|<header\b.*?</header\s*>|<footer\b.*?</footer\s*>|<aside\b.*?</aside\s*>|<form\b.*?</form\s*>").unwrap()
});
static RE_TITLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?is)<title[^>]*>(.*?)</title\s*>").unwrap());
static RE_MAIN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)<(?:article|main)\b[^>]*>(.*)</(?:article|main)\s*>").unwrap());
static RE_BLOCK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)<(?:/?(?:p|div|section|article|li|ul|ol|h[1-6]|tr|table|blockquote|pre|dd|dt|figcaption)\b[^>]*|br\s*/?)>").unwrap()
});
static RE_TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)<[^>]*>").unwrap());
static RE_SPACES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[ \t\u{a0}]+").unwrap());
static RE_NL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\n{3,}").unwrap());
static RE_ENT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"&(#[0-9]{1,7}|#[xX][0-9a-fA-F]{1,6}|[a-zA-Z][a-zA-Z0-9]{1,10});").unwrap());

/// Kürzt auf `max_chars` Zeichen (UTF-8-sicher) und hängt „…“ an.
pub fn truncate(s: &str, max_chars: usize) -> String {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => format!("{}…", &s[..idx]),
        None => s.to_string(),
    }
}

/// Schneidet auf höchstens `max` Bytes an einer Zeichengrenze ab.
pub fn clip_bytes(s: &str, max: usize) -> &str {
    if s.len() <= max {
        return s;
    }
    let mut i = max;
    while !s.is_char_boundary(i) {
        i -= 1;
    }
    &s[..i]
}

pub fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn host_of(url: &str) -> String {
    reqwest::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.trim_start_matches("www.").to_string()))
        .unwrap_or_default()
}

/// Entfernt Tags und Entities aus kurzen HTML-Schnipseln (z. B. Wikipedia-Snippets).
pub fn clean_inline(html: &str) -> String {
    collapse_ws(&decode_entities(&RE_TAG.replace_all(html, "")))
}

/// Extrahiert Titel und lesbaren Haupttext aus einer HTML-Seite.
pub fn html_to_text(html: &str) -> (String, String) {
    let title = RE_TITLE.captures(html).map(|c| clean_inline(&c[1])).unwrap_or_default();
    let cleaned = RE_DROP.replace_all(html, " ");
    let cleaned = RE_CHROME.replace_all(&cleaned, " ");
    let body = RE_MAIN
        .captures(&cleaned)
        .map(|c| c[1].to_string())
        .filter(|b| b.len() > 800)
        .unwrap_or_else(|| cleaned.to_string());
    let s = RE_BLOCK.replace_all(&body, "\n");
    let s = RE_TAG.replace_all(&s, "");
    let s = decode_entities(&s);
    let s = RE_SPACES.replace_all(&s, " ");
    let joined = s.lines().map(str::trim).collect::<Vec<_>>().join("\n");
    let text = RE_NL.replace_all(&joined, "\n\n");
    (title, text.trim().to_string())
}

pub fn decode_entities(s: &str) -> String {
    RE_ENT
        .replace_all(s, |c: &regex::Captures| {
            let e = &c[1];
            if let Some(num) = e.strip_prefix('#') {
                let code = match num.strip_prefix('x').or_else(|| num.strip_prefix('X')) {
                    Some(hex) => u32::from_str_radix(hex, 16).ok(),
                    None => num.parse().ok(),
                };
                return code.and_then(char::from_u32).map(|ch| ch.to_string()).unwrap_or_default();
            }
            let rep = match e {
                "amp" => "&",
                "lt" => "<",
                "gt" => ">",
                "quot" => "\"",
                "apos" => "'",
                "nbsp" => " ",
                "auml" => "ä",
                "ouml" => "ö",
                "uuml" => "ü",
                "Auml" => "Ä",
                "Ouml" => "Ö",
                "Uuml" => "Ü",
                "szlig" => "ß",
                "ndash" => "–",
                "mdash" => "—",
                "hellip" => "…",
                "lsquo" => "‘",
                "rsquo" => "’",
                "ldquo" => "“",
                "rdquo" => "”",
                "bdquo" => "„",
                "laquo" => "«",
                "raquo" => "»",
                "copy" => "©",
                "reg" => "®",
                "euro" => "€",
                "middot" => "·",
                "bull" => "•",
                "eacute" => "é",
                "egrave" => "è",
                "aacute" => "á",
                "agrave" => "à",
                "ccedil" => "ç",
                "times" => "×",
                "deg" => "°",
                _ => return c[0].to_string(),
            };
            rep.to_string()
        })
        .into_owned()
}

/// Entfernt einen umschließenden ```markdown-Zaun, falls das Modell die Antwort darin verpackt.
pub fn unwrap_fence(s: &str) -> String {
    let t = s.trim();
    if let Some(rest) = t.strip_prefix("```") {
        if let Some(nl) = rest.find('\n') {
            let lang = rest[..nl].trim().to_lowercase();
            if lang.is_empty() || lang == "markdown" || lang == "md" {
                if let Some(inner) = rest[nl + 1..].trim_end().strip_suffix("```") {
                    return inner.trim().to_string();
                }
            }
        }
    }
    t.to_string()
}
