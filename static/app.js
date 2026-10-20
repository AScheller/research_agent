'use strict';

/* ================================================================
   Reflective Research Agent – Frontend (DE / EN)
   ================================================================ */

// ---------- helpers ----------
const $ = (s, r = document) => r.querySelector(s);
const $$ = (s, r = document) => [...r.querySelectorAll(s)];
const ESC = { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' };
const esc = (s) => String(s ?? '').replace(/[&<>"']/g, (c) => ESC[c]);
const store = {
  get(k, d) { try { return localStorage.getItem(k) ?? d; } catch { return d; } },
  set(k, v) { try { localStorage.setItem(k, v); } catch { /* ignore */ } },
};

// ================================================================
//  i18n
// ================================================================
const I18N = {
  de: {
    brand_sub: 'reflektierend · tool-nutzend',
    new_research: 'Neue Recherche',
    history: 'Verlauf',
    no_history: 'Noch keine Recherchen.',
    ui_language: 'Sprache der Oberfläche',
    toggle_theme: 'Hell/Dunkel umschalten',
    settings: 'Einstellungen',
    menu: 'Menü',
    eyebrow: 'Reflective Research Agent — Planung & Tool-Nutzung',
    hero_title: 'Was möchtest du <span class="grad">erforschen</span>?',
    hero_lede: 'Ein Planer zerlegt deine Frage in Schritte. Research-Agenten durchsuchen Web, arXiv und Wikipedia, ein Writer verfasst den Bericht, und ein Editor reflektiert und verbessert ihn – angetrieben von',
    prompt_placeholder: 'z. B. Was ist Agentic AI und wie kann man über den Grad der Autonomie das Ergebnis verbessern? Fokus auf Autonomie-Stufen, Benchmarks und Praxisbeispiele aus 2024–2026.',
    depth: 'Tiefe',
    depth_quick: 'Schnell',
    depth_standard: 'Standard',
    depth_deep: 'Gründlich',
    report_language: 'Bericht',
    report_language_hint: 'Sprache des Berichts',
    kbd_ctrl: 'Strg',
    start: 'Recherche starten',
    tip: '<b>Tipp:</b> Füge 3–4 Details hinzu – Zeitraum, Fokus, Zielgruppe oder gewünschte Quellenarten –, damit die Agenten gezielter suchen.',
    planner: 'Planer',
    pipe_planner: 'zerlegt die Frage in konkrete Schritte',
    pipe_research: 'SearXNG, arXiv, Wikipedia & Seiten lesen',
    pipe_writer: 'verfasst Entwürfe mit Zitaten [n]',
    pipe_editor: 'reflektiert, prüft & verbessert',
    made_by: 'Open-Source-Projekt von',
    research_question: 'Recherchefrage',
    edit: 'Bearbeiten',
    edit_hint: 'Frage bearbeiten und erneut starten',
    cancel: 'Abbrechen',
    steps: 'Schritte',
    sources_lc: 'Quellen',
    expand_all: 'Alle aufklappen',
    collapse_all: 'Alle zuklappen',
    tab_report: 'Bericht',
    tab_sources: 'Quellen',
    copy_md: 'Markdown kopieren',
    download_md: 'Als Markdown herunterladen',
    print: 'Drucken / als PDF speichern',
    close: 'Schließen',
    legend_llm: 'Sprachmodell · OpenAI-kompatibel',
    model: 'Modell',
    optional: 'optional',
    temperature: 'Temperatur',
    legend_search: 'Websuche · SearXNG',
    searx_hint: 'In der <code>settings.yml</code> von SearXNG muss unter <code>search.formats</code> auch <code>json</code> stehen. Ohne URL nutzt der Agent arXiv, Wikipedia und direkte Seitenabrufe.',
    search_language: 'Suchsprache',
    max_results: 'Max. Treffer',
    wikipedia_language: 'Wikipedia-Sprache (bei „Auto“)',
    arxiv_search: 'arXiv-Suche',
    wikipedia_search: 'Wikipedia-Suche',
    max_steps: 'Max. Planschritte',
    tool_rounds: 'Tool-Runden / Schritt',
    chars_per_page: 'Zeichen pro Seite',
    check_connection: 'Verbindung prüfen',
    save: 'Speichern',

    status_running: 'läuft',
    status_done: 'fertig',
    status_error: 'Fehler',
    status_cancelled: 'abgebrochen',
    tool_web_search: 'Websuche',
    tool_arxiv_search: 'arXiv',
    tool_wikipedia_search: 'Wikipedia',
    tool_fetch_url: 'Seite lesen',
    kind_web: 'Web',
    kind_arxiv: 'arXiv',
    kind_wikipedia: 'Wikipedia',
    kind_page: 'Seite',
    just_now: 'gerade eben',
    min_ago: 'vor {n} Min.',
    h_ago: 'vor {n} Std.',
    start_failed: 'Start fehlgeschlagen: {msg}',
    connecting: 'Verbinde mit dem Agenten…',
    planning: 'Generiere Schritte für die Recherche:',
    planned: '<b>{n} Schritte</b> geplant',
    notice_no_searx: 'Websuche (SearXNG) ist noch nicht konfiguriert – der Agent nutzt arXiv, Wikipedia und direkte Seitenabrufe. Die URL kannst du in den Einstellungen hinterlegen.',
    notice_plan_fallback: 'Der Plan des Modells war nicht lesbar – es wird ein Standardplan verwendet.',
    notice_cancelled: 'Die Recherche wurde abgebrochen.',
    err_not_found: 'Diese Recherche wurde nicht gefunden (evtl. wurde der Server während des Laufs neu gestartet).',
    err_connection: 'Verbindung zum Server verloren.',
    final_report: 'Writer agent · Finaler Bericht',
    live_output: 'Live-Ausgabe',
    review_notes: 'Review-Notizen',
    result: 'Ergebnis',
    reasoning: 'Gedankengang des Modells',
    tool_calls_one: '{n} Tool-Aufruf',
    tool_calls_other: '{n} Tool-Aufrufe',
    new_sources_one: '{n} neue Quelle',
    new_sources_other: '{n} neue Quellen',
    words_one: '{n} Wort',
    words_other: '{n} Wörter',
    final_running: 'Der finale Bericht wird live im Bericht-Panel geschrieben…',
    final_done: 'Finaler Bericht erstellt – {n} Wörter. Siehe Bericht-Panel.',
    error: 'Fehler',
    unit_results_one: '{n} Treffer',
    unit_results_other: '{n} Treffer',
    unit_papers_one: '{n} Paper',
    unit_papers_other: '{n} Paper',
    unit_articles_one: '{n} Artikel',
    unit_articles_other: '{n} Artikel',
    unit_chars_one: '{n} Zeichen gelesen',
    unit_chars_other: '{n} Zeichen gelesen',
    review_heading: 'Review',
    revised_heading: 'Überarbeiteter Entwurf',
    no_report: 'Es wurde kein Bericht erstellt.',
    report_placeholder: 'Der Bericht erscheint hier, sobald der Writer agent den finalen Bericht schreibt.',
    report_starting: 'Writer agent beginnt mit dem finalen Bericht…',
    report_info: '≈ {n} Wörter · {m} Min. Lesezeit',
    report_writing: '≈ {n} Wörter · wird geschrieben…',
    all: 'Alle',
    no_sources: 'Noch keine Quellen gesammelt.',
    no_report_yet: 'Noch kein Bericht vorhanden',
    copied: 'Markdown in die Zwischenablage kopiert',
    copy_failed: 'Kopieren nicht möglich',
    file_fallback: 'bericht',
    delete: 'Löschen',
    delete_confirm: 'Diese Recherche wirklich löschen?',
    not_configured: 'nicht konfiguriert',
    online: 'online',
    pill_error: 'Fehler',
    backend_down: 'Backend nicht erreichbar',
    checking: 'Prüfe Verbindungen…',
    loaded: 'geladen',
    config_load_failed: 'Konfiguration nicht ladbar: {msg}',
    settings_saved: 'Einstellungen gespeichert',
    save_failed: 'Speichern fehlgeschlagen: {msg}',
  },
  en: {
    brand_sub: 'reflective · tool-using',
    new_research: 'New research',
    history: 'History',
    no_history: 'No research yet.',
    ui_language: 'Interface language',
    toggle_theme: 'Toggle light/dark',
    settings: 'Settings',
    menu: 'Menu',
    eyebrow: 'Reflective Research Agent — Planning & Tool Use',
    hero_title: 'What would you like to <span class="grad">research</span>?',
    hero_lede: 'A planner breaks your question into steps. Research agents search the web, arXiv and Wikipedia, a writer drafts the report, and an editor reflects on it and improves it – powered by',
    prompt_placeholder: 'e.g. What is agentic AI and how can the degree of autonomy be used to improve results? Focus on autonomy levels, benchmarks and real-world examples from 2024–2026.',
    depth: 'Depth',
    depth_quick: 'Quick',
    depth_standard: 'Standard',
    depth_deep: 'Thorough',
    report_language: 'Report',
    report_language_hint: 'Language of the report',
    kbd_ctrl: 'Ctrl',
    start: 'Start research',
    tip: '<b>Tip:</b> Add 3–4 details – time frame, focus, audience or preferred types of sources – so the agents can search more precisely.',
    planner: 'Planner',
    pipe_planner: 'breaks the question into concrete steps',
    pipe_research: 'SearXNG, arXiv, Wikipedia & page reading',
    pipe_writer: 'drafts text with citations [n]',
    pipe_editor: 'reflects, reviews & improves',
    made_by: 'Open-source project by',
    research_question: 'Research question',
    edit: 'Edit',
    edit_hint: 'Edit the question and run again',
    cancel: 'Cancel',
    steps: 'steps',
    sources_lc: 'sources',
    expand_all: 'Expand all',
    collapse_all: 'Collapse all',
    tab_report: 'Report',
    tab_sources: 'Sources',
    copy_md: 'Copy Markdown',
    download_md: 'Download as Markdown',
    print: 'Print / save as PDF',
    close: 'Close',
    legend_llm: 'Language model · OpenAI-compatible',
    model: 'Model',
    optional: 'optional',
    temperature: 'Temperature',
    legend_search: 'Web search · SearXNG',
    searx_hint: 'SearXNG must list <code>json</code> under <code>search.formats</code> in its <code>settings.yml</code>. Without a URL the agent uses arXiv, Wikipedia and direct page fetches.',
    search_language: 'Search language',
    max_results: 'Max. results',
    wikipedia_language: 'Wikipedia language (for “Auto”)',
    arxiv_search: 'arXiv search',
    wikipedia_search: 'Wikipedia search',
    max_steps: 'Max. plan steps',
    tool_rounds: 'Tool rounds / step',
    chars_per_page: 'Characters per page',
    check_connection: 'Check connection',
    save: 'Save',

    status_running: 'running',
    status_done: 'done',
    status_error: 'error',
    status_cancelled: 'cancelled',
    tool_web_search: 'Web search',
    tool_arxiv_search: 'arXiv',
    tool_wikipedia_search: 'Wikipedia',
    tool_fetch_url: 'Read page',
    kind_web: 'Web',
    kind_arxiv: 'arXiv',
    kind_wikipedia: 'Wikipedia',
    kind_page: 'Page',
    just_now: 'just now',
    min_ago: '{n} min ago',
    h_ago: '{n} h ago',
    start_failed: 'Start failed: {msg}',
    connecting: 'Connecting to the agent…',
    planning: 'Generating steps for the research:',
    planned: '<b>{n} steps</b> planned',
    notice_no_searx: 'Web search (SearXNG) is not configured yet – the agent uses arXiv, Wikipedia and direct page fetches. You can add the URL in the settings.',
    notice_plan_fallback: 'The model’s plan could not be read – using a default plan.',
    notice_cancelled: 'The research was cancelled.',
    err_not_found: 'This research was not found (the server may have been restarted during the run).',
    err_connection: 'Lost connection to the server.',
    final_report: 'Writer agent · Final report',
    live_output: 'Live output',
    review_notes: 'Review notes',
    result: 'Result',
    reasoning: 'Model reasoning',
    tool_calls_one: '{n} tool call',
    tool_calls_other: '{n} tool calls',
    new_sources_one: '{n} new source',
    new_sources_other: '{n} new sources',
    words_one: '{n} word',
    words_other: '{n} words',
    final_running: 'The final report is being written live in the report panel…',
    final_done: 'Final report created – {n} words. See the report panel.',
    error: 'Error',
    unit_results_one: '{n} result',
    unit_results_other: '{n} results',
    unit_papers_one: '{n} paper',
    unit_papers_other: '{n} papers',
    unit_articles_one: '{n} article',
    unit_articles_other: '{n} articles',
    unit_chars_one: '{n} character read',
    unit_chars_other: '{n} characters read',
    review_heading: 'Review',
    revised_heading: 'Revised draft',
    no_report: 'No report was created.',
    report_placeholder: 'The report will appear here as soon as the writer agent writes the final report.',
    report_starting: 'Writer agent is starting the final report…',
    report_info: '≈ {n} words · {m} min read',
    report_writing: '≈ {n} words · writing…',
    all: 'All',
    no_sources: 'No sources collected yet.',
    no_report_yet: 'No report yet',
    copied: 'Markdown copied to clipboard',
    copy_failed: 'Copy failed',
    file_fallback: 'report',
    delete: 'Delete',
    delete_confirm: 'Really delete this research?',
    not_configured: 'not configured',
    online: 'online',
    pill_error: 'error',
    backend_down: 'Backend not reachable',
    checking: 'Checking connections…',
    loaded: 'loaded',
    config_load_failed: 'Could not load configuration: {msg}',
    settings_saved: 'Settings saved',
    save_failed: 'Saving failed: {msg}',
  },
};

const EXAMPLES = {
  de: [
    'Was ist Agentic AI und wie kann man über den Grad der Autonomie das Ergebnis verbessern? Fokus: Definitionen, Autonomie-Stufen, Benchmarks und Praxisbeispiele aus 2024–2026.',
    'Vergleiche Retrieval-Augmented Generation mit Long-Context-Modellen: Kosten, Antwortqualität und Latenz – belegt mit aktuellen Studien und Benchmarks.',
    'Stand der Forschung zu Festkörperbatterien: führende Hersteller, erreichte Energiedichten, Zeitplan zur Serienreife und offene Probleme.',
    'Wie wirken sich Wärmepumpen in Bestandsgebäuden in Deutschland auf Energiekosten und CO₂ aus? Mit Felddaten, Förderprogrammen und Grenzen.',
  ],
  en: [
    'What is agentic AI and how can the degree of autonomy be used to improve results? Focus: definitions, autonomy levels, benchmarks and real-world examples from 2024–2026.',
    'Compare retrieval-augmented generation with long-context models: cost, answer quality and latency – backed by recent studies and benchmarks.',
    'State of research on solid-state batteries: leading manufacturers, achieved energy densities, timeline to mass production and open problems.',
    'How do heat pumps in existing buildings affect energy costs and CO₂ emissions? Include field data, subsidy programs and limitations.',
  ],
};

const S = {
  ui: (() => {
    const saved = store.get('ra-ui-lang', null);
    if (saved === 'de' || saved === 'en') return saved;
    return (navigator.language || 'de').toLowerCase().startsWith('de') ? 'de' : 'en';
  })(),
  runId: null, es: null, lastSeq: 0, terminal: false, errors: 0,
  prompt: '', model: '', steps: [], sources: new Map(), srcFilter: 'all',
  report: '', reportLive: '', startTs: 0, endTs: 0, status: 'idle', timer: null,
  depth: store.get('ra-depth', 'standard'), lang: null,
  health: null, config: null, allOpen: false, setters: {},
};
S.lang = store.get('ra-report-lang', null) || S.ui;

function t(key, vars) {
  let s = I18N[S.ui]?.[key] ?? I18N.de[key] ?? key;
  if (vars) s = s.replace(/\{(\w+)\}/g, (m, k) => (vars[k] ?? m));
  return s;
}
const locale = () => (S.ui === 'de' ? 'de-DE' : 'en-US');
const fmtNum = (n) => Number(n).toLocaleString(locale());
/** Pluralisierte Übersetzung: `key_one` / `key_other`, `{n}` wird lokalisiert formatiert. */
function tn(key, n, vars = {}) { return t(`${key}_${n === 1 ? 'one' : 'other'}`, { ...vars, n: fmtNum(n) }); }

function hostOf(u) { try { return new URL(u).hostname.replace(/^www\./, ''); } catch { return ''; } }
function fmtDur(ms) {
  if (ms == null) return '';
  const s = ms / 1000;
  if (s < 60) return `${s.toLocaleString(locale(), { maximumFractionDigits: s < 10 ? 1 : 0 })} s`;
  return `${Math.floor(s / 60)} min ${Math.round(s % 60)} s`;
}
function fmtClock(ms) {
  const s = Math.max(0, Math.floor(ms / 1000));
  return `${String(Math.floor(s / 60)).padStart(2, '0')}:${String(s % 60).padStart(2, '0')}`;
}
function relTime(iso) {
  const d = new Date(iso);
  const diff = (Date.now() - d.getTime()) / 1000;
  if (diff < 60) return t('just_now');
  if (diff < 3600) return t('min_ago', { n: Math.floor(diff / 60) });
  if (diff < 86400) return t('h_ago', { n: Math.floor(diff / 3600) });
  return d.toLocaleDateString(locale(), { day: '2-digit', month: 'short', year: 'numeric' });
}
function slugify(s) {
  return String(s).toLowerCase().replace(/ä/g, 'ae').replace(/ö/g, 'oe').replace(/ü/g, 'ue').replace(/ß/g, 'ss')
    .normalize('NFKD').replace(/[̀-ͯ]/g, '').replace(/[^a-z0-9]+/g, '-').replace(/^-+|-+$/g, '').slice(0, 60) || t('file_fallback');
}
function wordCount(s) { return (s.match(/[\p{L}\p{N}]+/gu) || []).length; }
const statusText = (s) => t(`status_${s}`);
const toolLabel = (name) => I18N[S.ui][`tool_${name}`] ?? name;
const kindLabel = (k) => I18N[S.ui][`kind_${k}`] ?? k;

// ---------- icons ----------
const ICONS = {
  search: '<circle cx="11" cy="11" r="7"/><path d="m20 20-3.5-3.5"/>',
  globe: '<circle cx="12" cy="12" r="9"/><path d="M3 12h18"/><path d="M12 3a14 14 0 0 1 0 18a14 14 0 0 1 0-18z"/>',
  paper: '<path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z"/><path d="M14 3v5h5"/><path d="M9 13h6M9 17h4"/>',
  book: '<path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20V3H6.5A2.5 2.5 0 0 0 4 5.5z"/><path d="M4 19.5A2.5 2.5 0 0 0 6.5 22H20v-5"/>',
  link: '<path d="M10 13a5 5 0 0 0 7.5.5l3-3a5 5 0 0 0-7-7l-1.7 1.7"/><path d="M14 11a5 5 0 0 0-7.5-.5l-3 3a5 5 0 0 0 7 7l1.7-1.7"/>',
  pen: '<path d="M12 20h9"/><path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4z"/>',
  review: '<path d="m9 11 3 3 8-8"/><path d="M20 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/>',
  plan: '<path d="M10 6h10M10 12h10M10 18h10"/><path d="m3.5 6 1.2 1.2L7 5M3.5 12l1.2 1.2L7 11M3.5 18l1.2 1.2L7 17"/>',
  check: '<path d="m5 12.5 4.5 4.5L19 7.5"/>',
  x: '<path d="M6 6l12 12M18 6 6 18"/>',
  chev: '<path d="m6 9 6 6 6-6"/>',
  copy: '<rect x="9" y="9" width="12" height="12" rx="2"/><path d="M5 15V5a2 2 0 0 1 2-2h10"/>',
  download: '<path d="M12 3v12"/><path d="m7 10 5 5 5-5"/><path d="M5 21h14"/>',
  print: '<path d="M6 9V3h12v6"/><rect x="3" y="9" width="18" height="8" rx="2"/><path d="M7 14h10v7H7z"/>',
  sliders: '<path d="M4 6h9M17 6h3M4 12h3M11 12h9M4 18h11M19 18h1"/><circle cx="15" cy="6" r="2"/><circle cx="9" cy="12" r="2"/><circle cx="17" cy="18" r="2"/>',
  moon: '<path d="M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8z"/>',
  sun: '<circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"/>',
  plus: '<path d="M12 5v14M5 12h14"/>',
  trash: '<path d="M4 7h16"/><path d="M10 11v6M14 11v6"/><path d="M6 7l1 13h10l1-13"/><path d="M9 7V4h6v3"/>',
  stop: '<rect x="6" y="6" width="12" height="12" rx="2"/>',
  spark: '<path d="M12 3l1.9 5.1L19 10l-5.1 1.9L12 17l-1.9-5.1L5 10l5.1-1.9z"/><path d="M19 16l.7 1.8 1.8.7-1.8.7L19 21l-.7-1.8-1.8-.7 1.8-.7z"/>',
  clock: '<circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/>',
  menu: '<path d="M4 6h16M4 12h16M4 18h16"/>',
  arrow: '<path d="M5 12h14M13 6l6 6-6 6"/>',
  redo: '<path d="M3 12a9 9 0 0 1 15.5-6.2L21 8"/><path d="M21 3v5h-5"/><path d="M21 12a9 9 0 0 1-15.5 6.2L3 16"/><path d="M3 21v-5h5"/>',
  info: '<circle cx="12" cy="12" r="9"/><path d="M12 11v5M12 8h.01"/>',
  plug: '<path d="M9 2v6M15 2v6"/><path d="M6 8h12v4a6 6 0 0 1-12 0z"/><path d="M12 18v4"/>',
};
function icon(name, cls = '') {
  return `<svg class="ic ${cls}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${ICONS[name] || ''}</svg>`;
}

const AGENTS = {
  research: { label: 'Research agent', fn: 'research_agent', icon: 'search' },
  writer: { label: 'Writer agent', fn: 'writer_agent', icon: 'pen' },
  editor: { label: 'Editor agent', fn: 'editor_agent', icon: 'review' },
};
const TOOL_ICONS = { web_search: 'globe', arxiv_search: 'paper', wikipedia_search: 'book', fetch_url: 'link' };
const KINDS = ['web', 'arxiv', 'wikipedia', 'page'];

// ================================================================
//  Markdown
// ================================================================
function citeHtml(inner) {
  const nums = [];
  for (const part of inner.split(/[,;]/)) {
    const [a, b] = part.split(/[–-]/).map((x) => parseInt(x, 10));
    if (Number.isFinite(a) && Number.isFinite(b) && b >= a && b - a <= 20) { for (let i = a; i <= b; i++) nums.push(i); }
    else if (Number.isFinite(a)) nums.push(a);
  }
  return `<span class="cites">${nums.map((n) => {
    const src = S.sources.get(n);
    const href = src ? esc(src.url) : `#src-${n}`;
    return `<a class="cite" data-n="${n}" href="${href}" target="_blank" rel="noopener noreferrer">${n}</a>`;
  }).join('')}</span>`;
}

function inline(src) {
  const ph = [];
  const keep = (html) => { ph.push(html); return `\u0001${ph.length - 1}\u0002`; };
  const link = (url, text) => keep(`<a href="${esc(url)}" target="_blank" rel="noopener noreferrer">${text}</a>`);
  let s = String(src);
  s = s.replace(/`([^`\n]+)`/g, (m, c) => keep(`<code>${esc(c)}</code>`));
  s = s.replace(/<br\s*\/?>/gi, () => keep('<br>'));
  s = s.replace(/!?\[([^\]\n]+)\]\(\s*<?((?:https?:\/\/|mailto:|#)[^\s)>]*)>?(?:\s+"[^"]*")?\s*\)/g, (m, tx, u) => link(u, esc(tx)));
  s = s.replace(/<(https?:\/\/[^>\s]+)>/g, (m, u) => link(u, esc(u)));
  s = s.replace(/https?:\/\/[^\s<>()[\]"']*[^\s<>()[\]"'.,;:!?]/g, (u) => link(u, esc(u.length > 70 ? u.slice(0, 67) + '…' : u)));
  s = s.replace(/\[(\d{1,3}(?:\s*[,;–-]\s*\d{1,3})*)\]/g, (m, inner) => keep(citeHtml(inner)));
  s = esc(s);
  s = s.replace(/\*\*(?=\S)(.+?)(?<=\S)\*\*/g, '<strong>$1</strong>');
  s = s.replace(/__(?=\S)(.+?)(?<=\S)__/g, '<strong>$1</strong>');
  s = s.replace(/(^|[^\w*])\*(?=\S)(.+?)(?<=\S)\*(?!\*)/g, '$1<em>$2</em>');
  s = s.replace(/(^|[^\w_])_(?=\S)(.+?)(?<=\S)_(?!\w)/g, '$1<em>$2</em>');
  s = s.replace(/~~(.+?)~~/g, '<del>$1</del>');
  s = s.replace(/\u0001(\d+)\u0002/g, (m, i) => ph[+i]);
  return s;
}

const RE_LIST = /^\s*([-*+]|\d+[.)])\s+/;
const RE_BLOCK_START = /^(#{1,6}\s|\s*```|\s*~~~|\s*>)/;
const indentOf = (l) => l.match(/^\s*/)[0].replace(/\t/g, '    ').length;
const isBlank = (l) => /^\s*$/.test(l);
const isTableSep = (l) => /^\s*\|?\s*:?-{2,}:?\s*(\|\s*:?-{2,}:?\s*)*\|?\s*$/.test(l) && l.includes('-');
const splitRow = (r) => r.trim().replace(/^\|/, '').replace(/\|$/, '').split(/(?<!\\)\|/).map((c) => c.trim().replace(/\\\|/g, '|'));

function parseList(lines, start) {
  const base = indentOf(lines[start]);
  const ordered = /^\s*\d+[.)]/.test(lines[start]);
  const first = ordered ? parseInt(lines[start].trim(), 10) : 1;
  let html = ordered ? `<ol${first !== 1 ? ` start="${first}"` : ''}>` : '<ul>';
  let i = start;
  while (i < lines.length) {
    const l = lines[i];
    if (isBlank(l)) {
      let j = i + 1;
      while (j < lines.length && isBlank(lines[j])) j++;
      if (j < lines.length && indentOf(lines[j]) === base && RE_LIST.test(lines[j])) { i = j; continue; }
      break;
    }
    const m = l.match(/^\s*(?:[-*+]|\d+[.)])\s+(.*)$/);
    if (!m || indentOf(l) !== base) break;
    let content = m[1];
    const sub = [];
    i++;
    while (i < lines.length) {
      const n = lines[i];
      if (isBlank(n)) {
        let j = i + 1;
        while (j < lines.length && isBlank(lines[j])) j++;
        if (j < lines.length && indentOf(lines[j]) > base) { sub.push(''); i = j; continue; }
        break;
      }
      if (indentOf(n) > base) { sub.push(n); i++; continue; }
      if (!RE_LIST.test(n) && !RE_BLOCK_START.test(n) && !n.includes('|') && !sub.length) { content += '\n' + n.trim(); i++; continue; }
      break;
    }
    content = content.replace(/^\[([ xX])\]\s+/, (mm, c) => (c.trim() ? '☑ ' : '☐ '));
    let inner = inline(content);
    if (sub.length) {
      const ind = Math.min(...sub.filter((x) => x.trim()).map(indentOf));
      inner += md(sub.map((x) => x.replace(/\t/g, '    ').slice(Math.min(ind, indentOf(x)))).join('\n'));
    }
    html += `<li>${inner}</li>`;
  }
  return [html + (ordered ? '</ol>' : '</ul>'), i];
}

function md(src) {
  const lines = String(src || '').replace(/\r\n?/g, '\n').split('\n');
  const out = [];
  let i = 0;
  while (i < lines.length) {
    const l = lines[i];
    let m;
    if (isBlank(l)) { i++; continue; }
    if ((m = l.match(/^\s*(```|~~~)\s*([\w+#-]*)/))) {
      const fence = m[1]; const buf = []; i++;
      while (i < lines.length && !lines[i].trim().startsWith(fence)) { buf.push(lines[i]); i++; }
      i++;
      out.push(`<pre><code>${esc(buf.join('\n'))}</code></pre>`);
      continue;
    }
    if ((m = l.match(/^(#{1,6})\s+(.*?)\s*#*\s*$/))) {
      const lvl = m[1].length;
      out.push(`<h${lvl}>${inline(m[2])}</h${lvl}>`);
      i++; continue;
    }
    if (/^\s*([-*_])(\s*\1){2,}\s*$/.test(l)) { out.push('<hr>'); i++; continue; }
    if (/^\s*>/.test(l)) {
      const buf = [];
      while (i < lines.length && /^\s*>/.test(lines[i])) { buf.push(lines[i].replace(/^\s*>\s?/, '')); i++; }
      out.push(`<blockquote>${md(buf.join('\n'))}</blockquote>`);
      continue;
    }
    if (l.includes('|') && i + 1 < lines.length && isTableSep(lines[i + 1])) {
      const head = splitRow(l);
      const aligns = splitRow(lines[i + 1]).map((c) => (c.startsWith(':') && c.endsWith(':') ? 'center' : c.endsWith(':') ? 'right' : ''));
      const al = (k) => (aligns[k] ? ` style="text-align:${aligns[k]}"` : '');
      i += 2;
      const rows = [];
      while (i < lines.length && lines[i].includes('|') && !isBlank(lines[i])) { rows.push(splitRow(lines[i])); i++; }
      out.push(`<div class="table-wrap"><table><thead><tr>${head.map((c, k) => `<th${al(k)}>${inline(c)}</th>`).join('')}</tr></thead><tbody>${rows.map((r) => `<tr>${head.map((_, k) => `<td${al(k)}>${inline(r[k] ?? '')}</td>`).join('')}</tr>`).join('')}</tbody></table></div>`);
      continue;
    }
    if (RE_LIST.test(l)) { const [html, next] = parseList(lines, i); out.push(html); i = next; continue; }
    const buf = [l]; i++;
    while (i < lines.length && !isBlank(lines[i]) && !RE_BLOCK_START.test(lines[i]) && !RE_LIST.test(lines[i])
      && !(lines[i].includes('|') && i + 1 < lines.length && isTableSep(lines[i + 1]))) { buf.push(lines[i]); i++; }
    out.push(`<p>${inline(buf.join('\n'))}</p>`);
  }
  return out.join('\n');
}

// ================================================================
//  API
// ================================================================
async function api(method, url, body) {
  const headers = { 'X-UI-Lang': S.ui };
  if (body) headers['Content-Type'] = 'application/json';
  const res = await fetch(url, { method, headers, body: body ? JSON.stringify(body) : undefined });
  if (!res.ok) throw new Error((await res.text()) || `HTTP ${res.status}`);
  if (res.status === 204) return null;
  const ct = res.headers.get('content-type') || '';
  return ct.includes('json') ? res.json() : res.text();
}

let toastTimer;
function toast(msg, kind = '') {
  const el = $('#toast');
  el.textContent = msg;
  el.className = `toast show ${kind}`;
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => { el.className = 'toast'; }, 2800);
}

// ================================================================
//  Static texts / language switch
// ================================================================
function applyI18n() {
  document.documentElement.lang = S.ui;
  $$('[data-i18n]').forEach((el) => { el.textContent = t(el.dataset.i18n); });
  $$('[data-i18n-html]').forEach((el) => { el.innerHTML = t(el.dataset.i18nHtml); });
  $$('[data-i18n-ph]').forEach((el) => { el.placeholder = t(el.dataset.i18nPh); });
  $$('[data-i18n-title]').forEach((el) => {
    const v = t(el.dataset.i18nTitle);
    el.title = v;
    el.setAttribute('aria-label', v);
  });
  $$('#seg-ui button').forEach((b) => b.classList.toggle('on', b.dataset.v === S.ui));
  $('#btn-expand-all').textContent = t(S.allOpen ? 'collapse_all' : 'expand_all');
  renderExamples();
}

function renderExamples() {
  $('#examples').innerHTML = EXAMPLES[S.ui].map((e, i) => `<button type="button" class="chip" data-ex="${i}">${esc(e.length > 92 ? e.slice(0, 90) + '…' : e)}</button>`).join('');
}

function setUiLang(lang) {
  if (lang === S.ui) return;
  S.ui = lang;
  store.set('ra-ui-lang', lang);
  // Berichtssprache folgt der Oberfläche, solange sie nicht explizit gewählt wurde
  if (!store.get('ra-report-lang', null)) S.setters.lang?.(lang, false);
  applyI18n();
  renderHistory();
  refreshHealth();
  if (S.runId && !$('#run').classList.contains('hidden')) openRun(S.runId, S.expandOnLoad); // Lauf in neuer Sprache neu aufbauen
}

// ================================================================
//  Views
// ================================================================
function showComposer(prefill) {
  closeES();
  S.runId = null;
  history.replaceState(null, '', location.pathname);
  $('#run').classList.add('hidden');
  $('#composer').classList.remove('hidden');
  if (prefill != null) $('#prompt').value = prefill;
  $('#prompt').focus();
  markActiveHistory();
  closeNav();
}

function showRunView() {
  $('#composer').classList.add('hidden');
  $('#run').classList.remove('hidden');
  closeNav();
}

function closeNav() { $('#app').classList.remove('nav-open'); }

// ================================================================
//  Run lifecycle
// ================================================================
async function startRun() {
  const prompt = $('#prompt').value.trim();
  if (!prompt) {
    const card = $('.prompt-card');
    card.classList.remove('shake'); void card.offsetWidth; card.classList.add('shake');
    $('#prompt').focus();
    return;
  }
  const btn = $('#btn-start');
  btn.disabled = true;
  try {
    const r = await api('POST', '/api/runs', { prompt, depth: S.depth, language: S.lang, ui_lang: S.ui });
    openRun(r.id);
    loadHistory();
  } catch (e) {
    toast(t('start_failed', { msg: e.message }), 'err');
  } finally {
    btn.disabled = false;
  }
}

function resetRun() {
  stopTimer();
  Object.assign(S, {
    lastSeq: 0, terminal: false, errors: 0, prompt: '', model: '', steps: [], sources: new Map(), srcFilter: 'all',
    report: '', reportLive: '', startTs: 0, endTs: 0, allOpen: false,
  });
  $('#steps').innerHTML = '';
  $('#plan-status').innerHTML = `<span class="spin"></span><span>${t('connecting')}</span>`;
  $('#notices').innerHTML = '';
  $('#run-error').classList.add('hidden');
  $('#run-q').textContent = '';
  $('#run-time').textContent = '00:00';
  $('#run-model').textContent = '–';
  $('#btn-expand-all').textContent = t('expand_all');
  setStatus('running');
  updateProgress();
  renderSources();
  renderReport();
  switchTab('report');
}

function openRun(id, expand = false) {
  closeES();
  resetRun();
  S.runId = id;
  S.expandOnLoad = expand;
  history.replaceState(null, '', `#run=${id}${expand ? '&expand' : ''}`);
  showRunView();
  markActiveHistory();
  connect();
}

function connect() {
  const es = new EventSource(`/api/runs/${S.runId}/events`);
  S.es = es;
  es.onmessage = (e) => {
    S.errors = 0;
    let ev;
    try { ev = JSON.parse(e.data); } catch { return; }
    try { onEvent(ev); } catch (err) { console.error('event error', ev, err); }
  };
  es.onerror = () => {
    if (S.terminal) { closeES(); return; }
    if (es.readyState === EventSource.CLOSED || ++S.errors > 8) {
      closeES();
      showError(!S.steps.length && !S.prompt ? t('err_not_found') : t('err_connection'));
      setStatus('error');
      stopTimer();
    }
  };
}

function closeES() { if (S.es) { S.es.close(); S.es = null; } }

async function cancelRun() {
  if (!S.runId) return;
  try { await api('POST', `/api/runs/${S.runId}/cancel`); } catch (e) { toast(e.message, 'err'); }
}

// ================================================================
//  Event handling
// ================================================================
function onEvent(ev) {
  if (ev.seq != null) {
    if (ev.seq <= S.lastSeq) return;
    S.lastSeq = ev.seq;
  }
  const h = HANDLERS[ev.type];
  if (h) h(ev);
}

const stepAt = (i) => S.steps[i];

const HANDLERS = {
  run_start(ev) {
    S.startTs = ev.ts;
    S.prompt = ev.prompt;
    S.model = ev.model;
    $('#run-q').textContent = ev.prompt;
    $('#run-model').textContent = ev.model;
    if (ev.web_search === false) notice(t('notice_no_searx'));
    setStatus('running');
    startTimer();
  },
  phase(ev) {
    if (ev.phase === 'planning') {
      $('#plan-status').innerHTML = `<span class="spin"></span><span>${t('planning')} <span class="muted">${esc(S.prompt.slice(0, 90))}${S.prompt.length > 90 ? '…' : ''}</span></span>`;
    }
  },
  notice(ev) {
    const key = ev.code ? `notice_${ev.code}` : null;
    notice(key && I18N[S.ui][key] ? t(key) : ev.message);
  },
  plan(ev) {
    // `#run=<id>&expand` öffnet alle Schritte (z. B. für geteilte Links)
    const expand = S.expandOnLoad;
    if (expand) { S.allOpen = true; $('#btn-expand-all').textContent = t('collapse_all'); }
    S.steps = ev.steps.map((s) => ({
      ...s, status: 'pending', tools: new Map(), toolCount: 0, srcCount: 0,
      stream: '', output: '', review: '', reasoning: '', open: expand, userToggled: expand, el: null,
    }));
    renderSteps();
    $('#plan-status').innerHTML = `${icon('check')}<span>${t('planned', { n: ev.steps.length })} <span class="muted">· ${fmtDur(ev.duration_ms)}</span></span>`;
    updateProgress();
  },
  step_start(ev) {
    const st = stepAt(ev.index); if (!st) return;
    st.status = 'running';
    st.startTs = ev.ts;
    if (!st.userToggled) st.open = true;
    updateStep(st);
    updateProgress();
    if (st.final) { S.reportLive = ''; renderReportLive(); switchTab('report'); }
    // Nur bei Live-Läufen mitscrollen (nicht beim Wiedergeben gespeicherter Läufe)
    if (Date.now() - ev.ts < 5000) requestAnimationFrame(() => scrollStepIntoView(st));
  },
  thought(ev) {
    const st = stepAt(ev.index); if (!st) return;
    $('.activity', st.el).insertAdjacentHTML('beforeend', `<div class="thought">${icon('spark')}<div>${inline(ev.text)}</div></div>`);
  },
  tool_call(ev) { addToolCall(stepAt(ev.index), ev); },
  tool_result(ev) { updateToolResult(stepAt(ev.index), ev); },
  source(ev) {
    S.sources.set(ev.source.n, ev.source);
    const running = S.steps.find((s) => s.status === 'running');
    if (running) { running.srcCount++; updateSub(running); }
    $('#run-sources').textContent = S.sources.size;
    $('#src-count').textContent = S.sources.size;
    scheduleSources();
  },
  token(ev) {
    const st = stepAt(ev.index); if (!st) return;
    st.stream += ev.delta;
    scheduleStream(st);
  },
  reasoning(ev) {
    const st = stepAt(ev.index); if (!st) return;
    st.reasoning += ev.delta;
  },
  step_done(ev) {
    const st = stepAt(ev.index); if (!st) return;
    st.status = 'done';
    st.durationMs = ev.duration_ms;
    st.output = ev.output || '';
    st.review = ev.review || '';
    if (ev.reasoning) st.reasoning = ev.reasoning;
    if (!st.userToggled) st.open = false;
    updateStep(st, true);
    updateProgress();
  },
  step_error(ev) {
    const st = stepAt(ev.index); if (!st) return;
    st.status = 'error';
    st.error = ev.message;
    st.durationMs = ev.duration_ms;
    st.open = true;
    updateStep(st, true);
    updateProgress();
  },
  report(ev) {
    S.report = ev.markdown || '';
    renderReport();
  },
  error(ev) { showError(ev.message); },
  cancelled() { notice(t('notice_cancelled')); },
  done(ev) {
    S.terminal = true;
    S.endTs = ev.ts;
    setStatus(ev.status);
    stopTimer();
    closeES();
    $$('.caret').forEach((c) => c.remove());
    if (!S.report && S.reportLive) { S.report = S.reportLive; renderReport(); }
    else if (!S.report) renderReport();
    updateProgress();
    loadHistory();
  },
};

function notice(msg) {
  $('#notices').insertAdjacentHTML('beforeend', `<div class="notice">${icon('info')}<span>${esc(msg)}</span></div>`);
}

function showError(msg) {
  const box = $('#run-error');
  box.textContent = msg;
  box.classList.remove('hidden');
}

function setStatus(s) {
  S.status = s;
  const b = $('#run-status');
  b.dataset.s = s;
  b.innerHTML = `${s === 'running' ? '<span class="spin sm"></span>' : ''}${esc(statusText(s))}`;
  $('#run').classList.toggle('is-running', s === 'running');
  $('#btn-cancel').classList.toggle('hidden', s !== 'running');
}

function updateProgress() {
  const total = S.steps.length;
  const done = S.steps.filter((s) => s.status === 'done' || s.status === 'error').length;
  const half = S.steps.some((s) => s.status === 'running') ? 0.5 : 0;
  let pct = total ? ((done + half) / total) * 100 : (S.status === 'running' ? 3 : 0);
  if (S.status === 'done') pct = 100;
  $('#run-bar').style.width = `${pct}%`;
  $('#run-steps').textContent = total ? `${done}/${total}` : '–';
}

function startTimer() { stopTimer(); S.timer = setInterval(tick, 500); tick(); }
function stopTimer() { if (S.timer) clearInterval(S.timer); S.timer = null; tick(); }
function tick() {
  if (!S.startTs) return;
  const end = S.endTs || (S.status === 'running' ? Date.now() : S.startTs);
  $('#run-time').textContent = fmtClock(end - S.startTs);
  if (S.status === 'running') {
    const st = S.steps.find((s) => s.status === 'running');
    if (st?.el && st.startTs) $('.step-meta', st.el).textContent = fmtClock(Date.now() - st.startTs);
  }
}

// ================================================================
//  Steps
// ================================================================
function renderSteps() {
  const ol = $('#steps');
  ol.innerHTML = '';
  for (const st of S.steps) { st.el = buildStep(st); ol.appendChild(st.el); }
}

function buildStep(st) {
  const a = AGENTS[st.agent] || AGENTS.research;
  const li = document.createElement('li');
  li.className = `step${st.final ? ' final' : ''}`;
  li.dataset.agent = st.agent;
  li.dataset.status = st.status;
  li.style.animationDelay = `${st.index * 40}ms`;
  li.innerHTML = `
    <span class="step-node"><span class="num">${st.index + 1}</span></span>
    <div class="step-head" role="button" tabindex="0" aria-expanded="false">
      <div class="step-main">
        <div class="step-top">
          <span class="agent-chip ${st.agent}">${icon(a.icon)}${esc(st.final ? t('final_report') : a.label)}</span>
          <span class="step-meta"></span>
        </div>
        <div class="step-task">${esc(st.task)}</div>
        <div class="step-sub"></div>
      </div>
      <span class="step-toggle">${icon('chev')}</span>
    </div>
    <div class="step-body"><div class="step-body-inner"><div class="step-pad">
      <div class="call-line">${icon('plug')}<span>Called <span class="fn">${a.fn}</span>(task)</span></div>
      <div class="activity"></div>
      <div class="stream-wrap hidden"><div class="box-label">${icon('pen')}${t('live_output')}</div><div class="stream md"></div></div>
      <div class="review-wrap hidden"><div class="box-label">${icon('review')}${t('review_notes')}</div><div class="review md"></div></div>
      <div class="output-wrap hidden"><div class="box-label">${icon('check')}${t('result')}</div><div class="output md"></div></div>
      <details class="reasoning hidden"><summary>${t('reasoning')}</summary><pre></pre></details>
      <div class="step-err hidden"></div>
    </div></div></div>`;
  const head = $('.step-head', li);
  const toggle = () => { st.open = !st.open; st.userToggled = true; applyOpen(st); };
  head.addEventListener('click', toggle);
  head.addEventListener('keydown', (e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); toggle(); } });
  return li;
}

/** Scrollt nur die Workflow-Spalte, niemals die ganze Seite. */
function scrollStepIntoView(st) {
  const box = $('#wf-scroll');
  if (!st.el || box.scrollHeight <= box.clientHeight) return;
  const b = box.getBoundingClientRect();
  const r = st.el.getBoundingClientRect();
  if (r.top < b.top || r.bottom > b.bottom) box.scrollTo({ top: box.scrollTop + r.top - b.top - 12, behavior: 'smooth' });
}

function applyOpen(st) {
  st.el.classList.toggle('open', st.open);
  $('.step-head', st.el).setAttribute('aria-expanded', String(st.open));
}

function updateSub(st) {
  const parts = [];
  if (st.toolCount) parts.push(`<span>${icon('plug')}${tn('tool_calls', st.toolCount)}</span>`);
  if (st.srcCount) parts.push(`<span>${icon('link')}${tn('new_sources', st.srcCount)}</span>`);
  const text = st.output || st.stream;
  if (st.agent !== 'research' && text) parts.push(`<span>${icon('pen')}${tn('words', wordCount(text))}</span>`);
  $('.step-sub', st.el).innerHTML = parts.join('');
}

function updateStep(st, finished = false) {
  const li = st.el; if (!li) return;
  li.dataset.status = st.status;
  applyOpen(st);
  const node = $('.step-node', li);
  node.innerHTML = st.status === 'running' ? '<span class="ring"></span>'
    : st.status === 'done' ? icon('check')
      : st.status === 'error' ? icon('x')
        : `<span class="num">${st.index + 1}</span>`;
  const meta = $('.step-meta', li);
  if (st.status === 'done' || st.status === 'error') meta.textContent = fmtDur(st.durationMs);
  if (st.status === 'running' && st.final) {
    $('.activity', li).innerHTML = `<div class="final-hint">${icon('paper')}<span>${t('final_running')}</span></div>`;
  }
  if (!finished) return;

  $('.stream-wrap', li).classList.add('hidden');
  st.stream = '';
  if (st.review) {
    $('.review-wrap', li).classList.remove('hidden');
    $('.review', li).innerHTML = md(st.review);
  }
  if (st.output) {
    if (st.final) {
      $('.activity', li).innerHTML = `<div class="final-hint">${icon('check')}<span>${t('final_done', { n: fmtNum(wordCount(st.output)) })}</span></div>`;
    } else {
      $('.output-wrap', li).classList.remove('hidden');
      $('.output', li).innerHTML = md(st.output);
    }
  }
  if (st.reasoning && st.reasoning.trim()) {
    const d = $('details.reasoning', li);
    d.classList.remove('hidden');
    $('pre', d).textContent = st.reasoning.trim();
  }
  if (st.error) {
    const e = $('.step-err', li);
    e.textContent = st.error;
    e.classList.remove('hidden');
  }
  $$('.tool.pending .tool-status', li).forEach((x) => { x.innerHTML = '<span class="muted">–</span>'; });
  updateSub(st);
}

function addToolCall(st, ev) {
  if (!st?.el) return;
  const args = ev.args || {};
  const main = args.query ?? args.url ?? JSON.stringify(args);
  const extra = [args.category && args.category !== 'general' ? args.category : null, args.time_range, args.lang].filter(Boolean).join(' · ');
  const div = document.createElement('div');
  div.className = 'tool pending';
  div.innerHTML = `
    <div class="tool-head">
      <span class="tool-ic ${esc(ev.name)}">${icon(TOOL_ICONS[ev.name] || 'plug')}</span>
      <span class="tool-name">${esc(toolLabel(ev.name))}</span>
      <span class="tool-arg" title="${esc(main)}">${esc(main)}</span>
      ${extra ? `<span class="tool-extra">${esc(extra)}</span>` : ''}
      <span class="tool-status"><span class="spin sm"></span></span>
    </div>
    <div class="tool-results"></div>`;
  st.tools.set(ev.id, div);
  st.toolCount++;
  $('.activity', st.el).appendChild(div);
  updateSub(st);
}

// Ältere Läufe enthalten nur einen fertigen Text wie „5 Treffer“ – Einheit daraus ableiten.
const LEGACY_UNITS = [
  [/^(treffer|results?)$/i, 'results'],
  [/^(paper|papers)$/i, 'papers'],
  [/^(artikel|articles?)$/i, 'articles'],
  [/^(zeichen gelesen|characters? read)$/i, 'chars'],
];
function toolSummary(ev) {
  let { unit, count } = ev;
  if (!unit && ev.summary) {
    const m = ev.summary.match(/^(\d+)\s+(.+)$/);
    const hit = m && LEGACY_UNITS.find(([re]) => re.test(m[2].trim()));
    if (hit) { unit = hit[1]; count = Number(m[1]); }
  }
  if (unit && count != null && I18N[S.ui][`unit_${unit}_other`]) return tn(`unit_${unit}`, count);
  return ev.summary || 'ok';
}

function updateToolResult(st, ev) {
  const div = st?.tools.get(ev.id); if (!div) return;
  div.classList.remove('pending');
  div.classList.add(ev.ok ? 'ok' : 'fail');
  $('.tool-status', div).innerHTML = ev.ok
    ? `<span class="okc">${esc(toolSummary(ev))}</span><span class="muted">· ${fmtDur(ev.duration_ms)}</span>`
    : `<span class="errc">${icon('x')} ${t('error')}</span>`;
  const res = $('.tool-results', div);
  if (!ev.ok) { res.innerHTML = `<div class="tool-err">${esc(ev.summary)}</div>`; return; }
  let html = '';
  if (ev.items?.length) {
    html += `<ul>${ev.items.map((it) => `<li><span class="src-n">${it.n}</span><a href="${esc(it.url)}" target="_blank" rel="noopener noreferrer" title="${esc(it.title)}">${esc(it.title)}</a><span class="host">${esc(hostOf(it.url))}</span></li>`).join('')}</ul>`;
  }
  if (ev.preview) html += `<div class="preview">${esc(ev.preview)}</div>`;
  res.innerHTML = html;
}

// ---------- streaming (throttled per animation frame) ----------
const dirty = new Set();
let rafPending = false;
function scheduleStream(st) {
  dirty.add(st);
  if (!rafPending) { rafPending = true; requestAnimationFrame(flushStreams); }
}
function prettyStream(s) {
  return s.replace(/<review>/g, `#### ${t('review_heading')}\n`).replace(/<\/review>/g, '\n')
    .replace(/<revised>/g, `\n---\n#### ${t('revised_heading')}\n`).replace(/<\/revised>/g, '');
}
function flushStreams() {
  rafPending = false;
  for (const st of dirty) {
    if (st.status !== 'running' || !st.el) continue;
    if (st.final) {
      S.reportLive = st.stream;
      renderReportLive();
    } else {
      const wrap = $('.stream-wrap', st.el);
      const box = $('.stream', st.el);
      wrap.classList.remove('hidden');
      const stick = box.scrollHeight - box.scrollTop - box.clientHeight < 60;
      box.innerHTML = md(prettyStream(st.stream)) + '<span class="caret"></span>';
      if (stick) box.scrollTop = box.scrollHeight;
    }
    updateSub(st);
  }
  dirty.clear();
}

// ================================================================
//  Report & sources
// ================================================================
function renderReport() {
  const body = $('#report-body');
  body.classList.remove('streaming');
  if (S.report) {
    body.innerHTML = md(S.report);
    const w = wordCount(S.report);
    $('#report-info').textContent = t('report_info', { n: fmtNum(w), m: Math.max(1, Math.round(w / 200)) });
    return;
  }
  $('#report-info').textContent = '';
  if (S.status === 'error' || S.status === 'cancelled') {
    body.innerHTML = `<div class="placeholder">${t('no_report')}</div>`;
    return;
  }
  body.innerHTML = `
    <div class="ph-wrap">
      <div class="ph-title"><span class="spin"></span>${t('report_placeholder')}</div>
      <div class="sk h"></div><div class="sk"></div><div class="sk" style="width:92%"></div><div class="sk" style="width:84%"></div>
      <div class="sk" style="width:60%;margin-top:26px"></div><div class="sk"></div><div class="sk" style="width:88%"></div>
    </div>`;
}

function renderReportLive() {
  const body = $('#report-body');
  const scroller = $('#report-scroll');
  const stick = scroller.scrollHeight - scroller.scrollTop - scroller.clientHeight < 120;
  body.classList.add('streaming');
  body.innerHTML = S.reportLive
    ? md(S.reportLive) + '<span class="caret"></span>'
    : `<div class="ph-title"><span class="spin"></span>${t('report_starting')}</div>`;
  const w = wordCount(S.reportLive);
  $('#report-info').textContent = w ? t('report_writing', { n: fmtNum(w) }) : '';
  if (stick) scroller.scrollTop = scroller.scrollHeight;
}

let srcTimer;
function scheduleSources() { clearTimeout(srcTimer); srcTimer = setTimeout(renderSources, 150); }

function renderSources() {
  const all = [...S.sources.values()].sort((a, b) => a.n - b.n);
  $('#src-count').textContent = all.length;
  $('#run-sources').textContent = all.length;
  const counts = all.reduce((acc, s) => { acc[s.kind] = (acc[s.kind] || 0) + 1; return acc; }, {});
  const filters = [['all', t('all'), all.length], ...KINDS.filter((k) => counts[k]).map((k) => [k, kindLabel(k), counts[k]])];
  $('#src-filters').innerHTML = all.length
    ? filters.map(([k, label, n]) => `<button type="button" data-f="${k}" class="${S.srcFilter === k ? 'on' : ''}">${esc(label)} · ${n}</button>`).join('')
    : '';
  const list = S.srcFilter === 'all' ? all : all.filter((s) => s.kind === S.srcFilter);
  $('#sources').innerHTML = list.length ? list.map((s) => `
    <div class="src-card" id="src-${s.n}">
      <div class="src-num">${s.n}</div>
      <div class="src-body">
        <a class="src-title" href="${esc(s.url)}" target="_blank" rel="noopener noreferrer">${esc(s.title)}</a>
        <div class="src-meta"><span class="kind ${esc(s.kind)}">${esc(kindLabel(s.kind))}</span><span>${esc(hostOf(s.url))}</span>${s.meta ? `<span class="muted">${esc(s.meta)}</span>` : ''}</div>
        ${s.snippet ? `<p>${esc(s.snippet)}</p>` : ''}
      </div>
    </div>`).join('') : `<div class="placeholder">${t('no_sources')}</div>`;
}

function switchTab(tab) {
  $$('.tab').forEach((x) => x.classList.toggle('on', x.dataset.tab === tab));
  $('#panel-report').classList.toggle('hidden', tab !== 'report');
  $('#panel-sources').classList.toggle('hidden', tab !== 'sources');
}

// ---------- citation tooltip ----------
function showTip(el, s) {
  const tip = $('#tooltip');
  tip.innerHTML = `
    <div class="tt-head"><span class="src-n">${s.n}</span><span class="kind ${esc(s.kind)}">${esc(kindLabel(s.kind))}</span><span class="muted">${esc(hostOf(s.url))}</span></div>
    <div class="tt-title">${esc(s.title)}</div>
    ${s.meta ? `<div class="muted" style="margin-bottom:4px">${esc(s.meta)}</div>` : ''}
    ${s.snippet ? `<div class="tt-snip">${esc(s.snippet)}</div>` : ''}`;
  tip.classList.remove('hidden');
  const r = el.getBoundingClientRect();
  const w = tip.offsetWidth; const h = tip.offsetHeight;
  const left = Math.min(Math.max(8, r.left + r.width / 2 - w / 2), window.innerWidth - w - 8);
  const top = r.bottom + 8 + h > window.innerHeight ? r.top - h - 8 : r.bottom + 8;
  tip.style.left = `${left}px`;
  tip.style.top = `${Math.max(8, top)}px`;
}
function hideTip() { $('#tooltip').classList.add('hidden'); }

// ---------- export ----------
async function copyReport() {
  if (!S.report) { toast(t('no_report_yet')); return; }
  try {
    if (navigator.clipboard && window.isSecureContext) await navigator.clipboard.writeText(S.report);
    else {
      const ta = document.createElement('textarea');
      ta.value = S.report; ta.style.position = 'fixed'; ta.style.opacity = '0';
      document.body.appendChild(ta); ta.select(); document.execCommand('copy'); ta.remove();
    }
    toast(t('copied'), 'ok');
  } catch { toast(t('copy_failed'), 'err'); }
}
function downloadReport() {
  if (!S.report) { toast(t('no_report_yet')); return; }
  const title = (S.report.match(/^#\s+(.+)$/m) || [])[1] || S.prompt;
  const blob = new Blob([S.report], { type: 'text/markdown;charset=utf-8' });
  const a = document.createElement('a');
  a.href = URL.createObjectURL(blob);
  a.download = `${slugify(title)}.md`;
  document.body.appendChild(a); a.click(); a.remove();
  setTimeout(() => URL.revokeObjectURL(a.href), 1000);
}

// ================================================================
//  History
// ================================================================
let historyCache = [];
async function loadHistory() {
  try { historyCache = await api('GET', '/api/runs'); } catch { return; }
  renderHistory();
}
function renderHistory() {
  const box = $('#history');
  if (!historyCache.length) { box.innerHTML = `<div class="empty-hint">${t('no_history')}</div>`; return; }
  box.innerHTML = historyCache.map((r) => `
    <div class="h-item${r.id === S.runId ? ' active' : ''}" data-id="${esc(r.id)}" title="${esc(r.prompt)}">
      <span class="h-dot ${esc(r.status)}"></span>
      <div class="h-text"><div class="h-title">${esc(r.title || r.prompt)}</div><div class="h-date">${relTime(r.created)} · ${esc(statusText(r.status))}</div></div>
      <button class="h-del" data-del="${esc(r.id)}" title="${t('delete')}" aria-label="${t('delete')}">${icon('trash')}</button>
    </div>`).join('');
}
function markActiveHistory() {
  $$('.h-item').forEach((el) => el.classList.toggle('active', el.dataset.id === S.runId));
}

// ================================================================
//  Health & settings
// ================================================================
function setPill(sel, state, val, title) {
  const p = $(sel);
  p.dataset.state = state;
  $('.pill-val', p).textContent = val;
  p.title = title || '';
}

async function refreshHealth() {
  try {
    const h = await api('GET', '/api/health');
    S.health = h;
    const llmState = h.llm.ok ? (h.llm.model_found === false ? 'warn' : 'ok') : 'err';
    setPill('#pill-llm', llmState, h.llm.model, `LLM: ${h.llm.detail}`);
    const sx = h.searxng;
    setPill('#pill-searx', sx.ok ? 'ok' : (sx.configured ? 'err' : 'warn'),
      sx.ok ? t('online') : (sx.configured ? t('pill_error') : t('not_configured')), `SearXNG: ${sx.detail}`);
    $('#hero-model').textContent = h.llm.model;
  } catch {
    setPill('#pill-llm', 'err', 'Server?', t('backend_down'));
    setPill('#pill-searx', 'err', 'Server?', t('backend_down'));
  }
  if (!$('#settings').classList.contains('hidden')) renderHealthBox();
}

function renderHealthBox() {
  const h = S.health;
  if (!h) { $('#health-box').innerHTML = ''; return; }
  const llmCls = h.llm.ok ? (h.llm.model_found === false ? 'warn' : 'ok') : 'err';
  const sxCls = h.searxng.ok ? 'ok' : (h.searxng.configured ? 'err' : 'warn');
  $('#health-box').innerHTML = `
    <div class="hb-row"><span class="dot ${llmCls}"></span><b>LLM</b><span>${esc(h.llm.model)} – ${esc(h.llm.detail)}</span></div>
    <div class="hb-row"><span class="dot ${sxCls}"></span><b>SearXNG</b><span>${esc(h.searxng.detail)}</span></div>`;
  $('#model-list').innerHTML = (h.llm.models || []).map((m) => `<option value="${esc(m.id)}">${esc(m.name)}${m.loaded ? ` (${t('loaded')})` : ''}</option>`).join('');
}

const getPath = (o, p) => p.split('.').reduce((a, k) => a?.[k], o);
function setPath(o, p, v) {
  const ks = p.split('.');
  let c = o;
  ks.slice(0, -1).forEach((k) => { c[k] = c[k] || {}; c = c[k]; });
  c[ks[ks.length - 1]] = v;
}

async function openSettings() {
  try { S.config = await api('GET', '/api/config'); } catch (e) { toast(t('config_load_failed', { msg: e.message }), 'err'); return; }
  $$('[name]', $('#settings-form')).forEach((inp) => {
    const v = getPath(S.config, inp.name);
    if (inp.type === 'checkbox') inp.checked = !!v; else inp.value = v ?? '';
  });
  renderHealthBox();
  $('#settings').classList.remove('hidden');
  closeNav();
}
function closeSettings() { $('#settings').classList.add('hidden'); }

async function saveSettings(e) {
  e.preventDefault();
  const cfg = JSON.parse(JSON.stringify(S.config || {}));
  $$('[name]', $('#settings-form')).forEach((inp) => {
    let v;
    if (inp.type === 'checkbox') v = inp.checked;
    else if (inp.type === 'number') v = inp.value === '' ? undefined : Number(inp.value);
    else v = inp.value.trim();
    if (v !== undefined) setPath(cfg, inp.name, v);
  });
  try {
    S.config = await api('PUT', '/api/config', cfg);
    toast(t('settings_saved'), 'ok');
    closeSettings();
    refreshHealth();
  } catch (err) {
    toast(t('save_failed', { msg: err.message }), 'err');
  }
}

// ---------- theme ----------
function applyTheme(th) {
  document.documentElement.dataset.theme = th;
  $('#btn-theme').innerHTML = icon(th === 'dark' ? 'sun' : 'moon');
}

// ================================================================
//  Init
// ================================================================
function initSeg(id, key, storeKey) {
  const seg = $(id);
  const set = (v, persist) => {
    S[key] = v;
    if (persist) store.set(storeKey, v);
    $$('button', seg).forEach((b) => b.classList.toggle('on', b.dataset.v === v));
  };
  S.setters[key] = set;
  set(S[key], false);
  seg.addEventListener('click', (e) => { const b = e.target.closest('button'); if (b) set(b.dataset.v, true); });
}

function init() {
  $$('[data-icon]').forEach((el) => { el.outerHTML = icon(el.dataset.icon); });

  let theme = store.get('ra-theme', null);
  if (!theme) theme = window.matchMedia?.('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
  applyTheme(theme);
  $('#btn-theme').addEventListener('click', () => {
    const next = document.documentElement.dataset.theme === 'dark' ? 'light' : 'dark';
    store.set('ra-theme', next);
    applyTheme(next);
  });

  applyI18n();
  $('#seg-ui').addEventListener('click', (e) => { const b = e.target.closest('button'); if (b) setUiLang(b.dataset.v); });

  $('#examples').addEventListener('click', (e) => {
    const b = e.target.closest('[data-ex]'); if (!b) return;
    $('#prompt').value = EXAMPLES[S.ui][+b.dataset.ex];
    $('#prompt').focus();
  });

  initSeg('#seg-depth', 'depth', 'ra-depth');
  initSeg('#seg-lang', 'lang', 'ra-report-lang');

  $('#btn-start').addEventListener('click', startRun);
  $('#prompt').addEventListener('keydown', (e) => { if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) { e.preventDefault(); startRun(); } });
  $('#btn-new').addEventListener('click', () => showComposer(''));
  $('#btn-rerun').addEventListener('click', () => showComposer(S.prompt));
  $('#btn-cancel').addEventListener('click', cancelRun);
  $('#btn-copy').addEventListener('click', copyReport);
  $('#btn-download').addEventListener('click', downloadReport);
  $('#btn-print').addEventListener('click', () => { switchTab('report'); window.print(); });
  $('#btn-menu').addEventListener('click', () => $('#app').classList.toggle('nav-open'));
  $('#scrim').addEventListener('click', closeNav);
  $('#btn-expand-all').addEventListener('click', () => {
    S.allOpen = !S.allOpen;
    S.steps.forEach((st) => { st.open = S.allOpen; st.userToggled = true; if (st.el) applyOpen(st); });
    $('#btn-expand-all').textContent = t(S.allOpen ? 'collapse_all' : 'expand_all');
  });

  $$('.tab').forEach((x) => x.addEventListener('click', () => switchTab(x.dataset.tab)));
  $('#src-filters').addEventListener('click', (e) => {
    const b = e.target.closest('[data-f]'); if (!b) return;
    S.srcFilter = b.dataset.f;
    renderSources();
  });

  $('#history').addEventListener('click', async (e) => {
    const del = e.target.closest('[data-del]');
    if (del) {
      e.stopPropagation();
      if (!confirm(t('delete_confirm'))) return;
      try { await api('DELETE', `/api/runs/${del.dataset.del}`); } catch (err) { toast(err.message, 'err'); }
      if (del.dataset.del === S.runId) showComposer('');
      loadHistory();
      return;
    }
    const item = e.target.closest('.h-item');
    if (item) openRun(item.dataset.id);
  });

  // Zitate: Tooltip + Klick auf unbekannte Quelle → Quellen-Tab
  document.addEventListener('mouseover', (e) => {
    const c = e.target.closest?.('.cite'); if (!c) return;
    const s = S.sources.get(+c.dataset.n);
    if (s) showTip(c, s);
  });
  document.addEventListener('mouseout', (e) => { if (e.target.closest?.('.cite')) hideTip(); });
  document.addEventListener('click', (e) => {
    const c = e.target.closest?.('.cite'); if (!c) return;
    if (!S.sources.get(+c.dataset.n)) {
      e.preventDefault();
      switchTab('sources');
    }
  });

  $('#btn-settings').addEventListener('click', openSettings);
  $('#pill-llm').addEventListener('click', openSettings);
  $('#pill-searx').addEventListener('click', openSettings);
  $$('[data-close]', $('#settings')).forEach((el) => el.addEventListener('click', closeSettings));
  $('#settings-form').addEventListener('submit', saveSettings);
  $('#btn-test').addEventListener('click', async () => {
    $('#health-box').innerHTML = `<div class="hb-row"><span class="spin sm"></span><span>${t('checking')}</span></div>`;
    await refreshHealth();
    renderHealthBox();
  });
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') { closeSettings(); closeNav(); hideTip(); }
  });

  const wantsExpand = () => /[#&]expand\b/.test(location.hash);
  window.addEventListener('hashchange', () => {
    const h = location.hash.match(/run=([a-z0-9]+)/i);
    if (h && h[1] !== S.runId) openRun(h[1], wantsExpand());
  });

  loadHistory();
  refreshHealth();
  setInterval(refreshHealth, 30000);
  setInterval(() => { if (historyCache.length) renderHistory(); }, 60000);

  const m = location.hash.match(/run=([a-z0-9]+)/i);
  if (m) openRun(m[1], wantsExpand()); else showComposer();
}

init();
