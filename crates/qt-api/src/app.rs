//! HTTP surface for the qt-core engine: state, router, handlers, DTOs.

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{DefaultBodyLimit, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;

use qt_core::{
    CharRange, DictionaryDefaults, DictionaryOverrides, DictionarySourceOverrides, Engine, Mode,
    NameCandidate, NameCandidateSource, NameEntityType, NameFilterMemory, NameFilterMode,
    NameFilterOptions, Options, TranslationResult,
};

use crate::name_ai::{entity_type_name, parse_entity_type, GeminiNameReviewer};
use crate::name_ner::{self, NameEntityRecognizer, NerNameSpan};

const MAX_REQUEST_SCAN_RANGE: usize = 100;
const MAX_REQUEST_BODY_BYTES: usize = 5 * 1024 * 1024;
const MAX_NAME_FILTER_CHARACTERS: usize = 200_000;

/// Shared, read-only application state: the loaded engine behind an Arc.
pub struct AppState {
    pub engine: Arc<Engine>,
    pub dictionary_defaults: Arc<DictionaryDefaults>,
    pub name_filter_services: NameFilterServices,
}

#[derive(Clone, Default)]
pub struct NameFilterServices {
    recognizer: Option<Arc<dyn NameEntityRecognizer>>,
    reviewer: Option<GeminiNameReviewer>,
    startup_warnings: Arc<Vec<String>>,
}

impl NameFilterServices {
    pub fn from_env() -> Self {
        let mut startup_warnings = Vec::new();
        let recognizer = match name_ner::from_env() {
            Ok(recognizer) => recognizer,
            Err(error) => {
                eprintln!("warning: name NER provider was not initialized: {error}");
                startup_warnings.push("ONNX NER is unavailable; check server logs".to_string());
                None
            }
        };
        let reviewer = match GeminiNameReviewer::from_env() {
            Ok(reviewer) => reviewer,
            Err(error) => {
                eprintln!("warning: Gemini name reviewer was not initialized: {error}");
                startup_warnings
                    .push("Gemini fallback is unavailable; check server logs".to_string());
                None
            }
        };
        Self {
            recognizer,
            reviewer,
            startup_warnings: Arc::new(startup_warnings),
        }
    }

    fn ner_configured(&self) -> bool {
        self.recognizer.is_some()
    }

    fn ai_configured(&self) -> bool {
        self.reviewer.is_some()
    }
}

/// Build the router from shared state. Kept separate from socket binding so
/// tests can exercise handlers in-process via `oneshot`.
pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/modes", get(modes))
        .route("/dictionaries/defaults", get(dictionary_defaults))
        .route("/translate", post(translate))
        .route("/translate/batch", post(translate_batch))
        .route("/names/filter", post(filter_names))
        .with_state(state)
        .layer(DefaultBodyLimit::max(MAX_REQUEST_BODY_BYTES))
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DictionaryDefaultsResp<'a> {
    names: &'a str,
    names2: &'a str,
    luat_nhan: &'a str,
    pronouns: &'a str,
    danh_tu: &'a str,
    ho_nguoi: &'a str,
    hau_tu: &'a str,
    ignored_chinese_phrases: &'a str,
}

async fn dictionary_defaults(State(state): State<Arc<AppState>>) -> Response {
    let defaults = state.dictionary_defaults.as_ref();
    let body = DictionaryDefaultsResp {
        names: &defaults.names,
        names2: &defaults.names2,
        luat_nhan: &defaults.luat_nhan,
        pronouns: &defaults.pronouns,
        danh_tu: &defaults.danh_tu,
        ho_nguoi: &defaults.ho_nguoi,
        hau_tu: &defaults.hau_tu,
        ignored_chinese_phrases: &defaults.ignored_chinese_phrases,
    };
    (
        [(header::CACHE_CONTROL, "public, max-age=3600")],
        Json(body),
    )
        .into_response()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TranslateReq {
    text: String,
    mode: String,
    #[serde(default)]
    wrap: bool,
    #[serde(default)]
    pretty: bool,
    #[serde(default)]
    ranges: bool,
    scan_range: Option<usize>,
    translation_algorithm: Option<i32>,
    prioritized_name: Option<bool>,
    dictionaries: Option<DictionarySourcesReq>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DictionarySourcesReq {
    names: Option<String>,
    names2: Option<String>,
    luat_nhan: Option<String>,
    pronouns: Option<String>,
    danh_tu: Option<String>,
    ho_nguoi: Option<String>,
    hau_tu: Option<String>,
    ignored_chinese_phrases: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct NameFilterReq {
    text: String,
    #[serde(default = "default_name_filter_mode")]
    mode: String,
    min_occurrences: Option<usize>,
    min_confidence: Option<f32>,
    max_candidates: Option<usize>,
    max_name_length: Option<usize>,
    include_known: Option<bool>,
    #[serde(default)]
    known_names: HashMap<String, String>,
    #[serde(default)]
    rejected_names: Vec<String>,
    #[serde(default)]
    ner: ProviderReq,
    #[serde(default)]
    ai_fallback: AiFallbackReq,
    dictionaries: Option<DictionarySourcesReq>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProviderReq {
    #[serde(default)]
    enabled: bool,
    min_confidence: Option<f32>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AiFallbackReq {
    #[serde(default)]
    enabled: bool,
    min_confidence: Option<f32>,
    min_rule_confidence: Option<f32>,
    max_rule_confidence: Option<f32>,
    max_candidates: Option<usize>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NameFilterResp {
    candidates: Vec<NameCandidateResp>,
    stats: NameFilterStats,
    capabilities: NameFilterCapabilities,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    warnings: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NameCandidateResp {
    text: String,
    suggested: String,
    entity_type: &'static str,
    score: f32,
    occurrences: usize,
    ranges: Vec<RangeDto>,
    contexts: Vec<String>,
    reasons: Vec<String>,
    sources: Vec<&'static str>,
    known: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NameFilterStats {
    scanned_characters: usize,
    rule_candidates: usize,
    ner_candidates: usize,
    ai_reviewed: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NameFilterCapabilities {
    ner_configured: bool,
    ai_configured: bool,
}

fn default_name_filter_mode() -> String {
    "hybrid".to_string()
}

impl DictionarySourcesReq {
    fn into_overrides(self) -> DictionaryOverrides {
        DictionaryOverrides::from_sources(DictionarySourceOverrides {
            names: self.names.as_deref(),
            names2: self.names2.as_deref(),
            luat_nhan: self.luat_nhan.as_deref(),
            pronouns: self.pronouns.as_deref(),
            danh_tu: self.danh_tu.as_deref(),
            ho_nguoi: self.ho_nguoi.as_deref(),
            hau_tu: self.hau_tu.as_deref(),
            ignored_chinese_phrases: self.ignored_chinese_phrases.as_deref(),
        })
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TranslateResp {
    translated: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_ranges: Option<Vec<RangeDto>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_ranges: Option<Vec<RangeDto>>,
}

#[derive(Serialize)]
struct RangeDto {
    start: usize,
    length: usize,
}

impl From<CharRange> for RangeDto {
    fn from(value: CharRange) -> Self {
        Self {
            start: value.start,
            length: value.length,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BatchReq {
    texts: Vec<String>,
    mode: String,
    #[serde(default)]
    wrap: bool,
    #[serde(default)]
    pretty: bool,
    #[serde(default)]
    ranges: bool,
    scan_range: Option<usize>,
    translation_algorithm: Option<i32>,
    prioritized_name: Option<bool>,
    dictionaries: Option<DictionarySourcesReq>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BatchResp {
    translated: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_ranges: Option<Vec<Vec<RangeDto>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_ranges: Option<Vec<Vec<RangeDto>>>,
}

/// Error carrying an HTTP status and a message; renders as `{"error": ...}`.
struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn bad_request(msg: impl Into<String>) -> Self {
        ApiError {
            status: StatusCode::BAD_REQUEST,
            message: msg.into(),
        }
    }
    fn internal(msg: impl Into<String>) -> Self {
        ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: msg.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(json!({ "error": self.message }))).into_response()
    }
}

/// Mode strings shared with the CLI.
fn parse_mode(s: &str) -> Option<Mode> {
    match s {
        "hanviet" => Some(Mode::HanViet),
        "vietphrase" => Some(Mode::VietPhrase),
        "vietphrase-one" => Some(Mode::VietPhraseOneMeaning),
        _ => None,
    }
}

/// Presentation-only normalization: trim leading whitespace, uppercase first char.
fn prettify(s: &str) -> String {
    let t = s.trim_start();
    let mut chars = t.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn prettify_result(mut result: TranslationResult) -> TranslationResult {
    let pretty_text = prettify(&result.translated_text);
    let trimmed = result.translated_text.trim_start();
    let removed = result.translated_text[..result.translated_text.len() - trimmed.len()]
        .encode_utf16()
        .count();
    let mut chars = trimmed.chars();
    let Some(first) = chars.next() else {
        result.translated_text.clear();
        for range in &mut result.target_ranges {
            range.start = 0;
            range.length = 0;
        }
        return result;
    };
    let old_first_length = first.len_utf16();
    let upper = first.to_uppercase().collect::<String>();
    let new_first_length = upper.encode_utf16().count();
    result.translated_text = pretty_text;

    let map_boundary = |position: usize| {
        let relative = position.saturating_sub(removed);
        if relative == 0 {
            0
        } else if relative <= old_first_length {
            new_first_length
        } else {
            relative - old_first_length + new_first_length
        }
    };
    for range in &mut result.target_ranges {
        let end = map_boundary(range.start.saturating_add(range.length));
        range.start = map_boundary(range.start);
        range.length = end.saturating_sub(range.start);
    }
    result
}

/// Run one translation off the async runtime (engine is synchronous/CPU-bound).
async fn run_translate(
    engine: Arc<Engine>,
    text: String,
    mode: Mode,
    opts: Options,
    pretty: bool,
    dictionaries: Option<Arc<DictionaryOverrides>>,
) -> Result<TranslationResult, ApiError> {
    tokio::task::spawn_blocking(move || {
        let out = match dictionaries {
            Some(dictionaries) => engine
                .translate_with_ranges_and_overrides(&text, mode, &opts, &dictionaries)
                .map_err(|error| error.to_string()),
            None => Ok(engine.translate_with_ranges(&text, mode, &opts)),
        }?;
        Ok(if pretty { prettify_result(out) } else { out })
    })
    .await
    .map_err(|_| ApiError::internal("translate task failed"))
    .and_then(|result: Result<TranslationResult, String>| result.map_err(ApiError::bad_request))
}

async fn prepare_dictionaries(
    dictionaries: Option<DictionarySourcesReq>,
) -> Result<Option<Arc<DictionaryOverrides>>, ApiError> {
    let Some(dictionaries) = dictionaries else {
        return Ok(None);
    };
    tokio::task::spawn_blocking(move || Arc::new(dictionaries.into_overrides()))
        .await
        .map(Some)
        .map_err(|_| ApiError::internal("dictionary parse task failed"))
}

fn request_options(
    wrap: bool,
    scan_range: Option<usize>,
    translation_algorithm: Option<i32>,
    prioritized_name: Option<bool>,
) -> Result<Options, ApiError> {
    let mut options = Options {
        wrap_type: if wrap { 1 } else { 0 },
        ..Options::default()
    };
    if let Some(scan_range) = scan_range {
        if !(1..=MAX_REQUEST_SCAN_RANGE).contains(&scan_range) {
            return Err(ApiError::bad_request(format!(
                "scanRange must be between 1 and {MAX_REQUEST_SCAN_RANGE}"
            )));
        }
        options.scan_range = scan_range;
    }
    if let Some(translation_algorithm) = translation_algorithm {
        if !matches!(translation_algorithm, 0..=2) {
            return Err(ApiError::bad_request(
                "translationAlgorithm must be 0, 1, or 2",
            ));
        }
        options.translation_algorithm = translation_algorithm;
    }
    if let Some(prioritized_name) = prioritized_name {
        options.prioritized_name = prioritized_name;
    }
    Ok(options)
}

async fn modes() -> Json<serde_json::Value> {
    Json(json!({ "modes": ["hanviet", "vietphrase", "vietphrase-one"] }))
}

async fn filter_names(
    State(state): State<Arc<AppState>>,
    Json(req): Json<NameFilterReq>,
) -> Result<Json<NameFilterResp>, ApiError> {
    let scanned_characters = req.text.chars().count();
    if scanned_characters > MAX_NAME_FILTER_CHARACTERS {
        return Err(ApiError::bad_request(format!(
            "text must not exceed {MAX_NAME_FILTER_CHARACTERS} characters"
        )));
    }
    let mode = match req.mode.as_str() {
        "qt" => NameFilterMode::QtCompatible,
        "hybrid" => NameFilterMode::Hybrid,
        other => {
            return Err(ApiError::bad_request(format!(
                "invalid name filter mode: {other}"
            )))
        }
    };
    let default_min_confidence = if mode == NameFilterMode::QtCompatible {
        0.55
    } else {
        0.60
    };
    let min_confidence = confidence(req.min_confidence, default_min_confidence, "minConfidence")?;
    let options = NameFilterOptions {
        mode,
        min_occurrences: bounded_usize(req.min_occurrences, 2, 1, 100, "minOccurrences")?,
        min_score: min_confidence,
        max_candidates: bounded_usize(req.max_candidates, 200, 1, 1_000, "maxCandidates")?,
        max_name_length: bounded_usize(req.max_name_length, 8, 2, 8, "maxNameLength")?,
        include_known: req.include_known.unwrap_or(true),
    };
    let output_max_candidates = options.max_candidates;
    let memory = NameFilterMemory {
        known_names: req.known_names,
        rejected_names: req.rejected_names.into_iter().collect(),
    };
    let dictionaries = prepare_dictionaries(req.dictionaries).await?;
    let text = Arc::new(req.text);
    let engine = state.engine.clone();
    let rule_text = text.clone();
    let rule_memory = memory.clone();
    let rule_dictionaries = dictionaries.clone();
    let mut result = tokio::task::spawn_blocking(move || {
        engine.filter_names(
            &rule_text,
            &options,
            &rule_memory,
            rule_dictionaries.as_deref(),
        )
    })
    .await
    .map_err(|_| ApiError::internal("name filter task failed"))?;
    let rule_candidates = result.candidates.len();
    let mut warnings = state.name_filter_services.startup_warnings.as_ref().clone();

    let mut ner_candidates = 0;
    if req.ner.enabled {
        if let Some(recognizer) = state.name_filter_services.recognizer.clone() {
            let ner_text = text.clone();
            match tokio::task::spawn_blocking(move || recognizer.recognize(&ner_text)).await {
                Ok(Ok(spans)) => {
                    let threshold = confidence(req.ner.min_confidence, 0.65, "ner.minConfidence")?;
                    ner_candidates = merge_ner_candidates(
                        &state.engine,
                        &mut result.candidates,
                        spans,
                        threshold,
                        &memory,
                        dictionaries.as_deref(),
                    );
                }
                Ok(Err(error)) => warnings.push(error),
                Err(_) => warnings.push("NER task failed".to_string()),
            }
        } else {
            warnings.push("NER was requested but no ONNX model is configured".to_string());
        }
    }

    let mut ai_reviewed = 0;
    if req.ai_fallback.enabled {
        if let Some(reviewer) = state.name_filter_services.reviewer.as_ref() {
            let min_rule = confidence(
                req.ai_fallback.min_rule_confidence,
                0.40,
                "aiFallback.minRuleConfidence",
            )?;
            let max_rule = confidence(
                req.ai_fallback.max_rule_confidence,
                0.82,
                "aiFallback.maxRuleConfidence",
            )?;
            if min_rule > max_rule {
                return Err(ApiError::bad_request(
                    "aiFallback.minRuleConfidence must not exceed maxRuleConfidence",
                ));
            }
            let ambiguous: Vec<NameCandidate> = result
                .candidates
                .iter()
                .filter(|candidate| {
                    !candidate.known && (min_rule..=max_rule).contains(&candidate.score)
                })
                .cloned()
                .collect();
            let max_ai_candidates = bounded_usize(
                req.ai_fallback.max_candidates,
                25,
                1,
                50,
                "aiFallback.maxCandidates",
            )?;
            match reviewer.review(&text, &ambiguous, max_ai_candidates).await {
                Ok(decisions) => {
                    ai_reviewed = decisions.len();
                    let decision_threshold = confidence(
                        req.ai_fallback.min_confidence,
                        0.65,
                        "aiFallback.minConfidence",
                    )?;
                    apply_ai_decisions(&mut result.candidates, decisions, decision_threshold);
                }
                Err(error) => warnings.push(error),
            }
        } else {
            warnings.push("AI fallback was requested but Gemini is not configured".to_string());
        }
    }

    result.candidates.sort_by(|left, right| {
        right
            .known
            .cmp(&left.known)
            .then_with(|| right.score.total_cmp(&left.score))
            .then_with(|| right.occurrences.cmp(&left.occurrences))
    });
    result.candidates.truncate(output_max_candidates);
    warnings.sort();
    warnings.dedup();
    let response_candidates = result
        .candidates
        .into_iter()
        .map(|candidate| name_candidate_response(&text, candidate))
        .collect();
    Ok(Json(NameFilterResp {
        candidates: response_candidates,
        stats: NameFilterStats {
            scanned_characters: result.scanned_characters,
            rule_candidates,
            ner_candidates,
            ai_reviewed,
        },
        capabilities: NameFilterCapabilities {
            ner_configured: state.name_filter_services.ner_configured(),
            ai_configured: state.name_filter_services.ai_configured(),
        },
        warnings,
    }))
}

fn confidence(value: Option<f32>, default: f32, field: &str) -> Result<f32, ApiError> {
    let value = value.unwrap_or(default);
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(value)
    } else {
        Err(ApiError::bad_request(format!(
            "{field} must be between 0 and 1"
        )))
    }
}

fn bounded_usize(
    value: Option<usize>,
    default: usize,
    min: usize,
    max: usize,
    field: &str,
) -> Result<usize, ApiError> {
    let value = value.unwrap_or(default);
    if (min..=max).contains(&value) {
        Ok(value)
    } else {
        Err(ApiError::bad_request(format!(
            "{field} must be between {min} and {max}"
        )))
    }
}

fn merge_ner_candidates(
    engine: &Engine,
    candidates: &mut Vec<NameCandidate>,
    spans: Vec<NerNameSpan>,
    threshold: f32,
    memory: &NameFilterMemory,
    dictionaries: Option<&DictionaryOverrides>,
) -> usize {
    let mut grouped: HashMap<String, Vec<NerNameSpan>> = HashMap::new();
    for span in spans.into_iter().filter(|span| span.score >= threshold) {
        if memory.rejected_names.contains(&span.text)
            || engine.contains_name(&span.text, dictionaries)
        {
            continue;
        }
        grouped.entry(span.text.clone()).or_default().push(span);
    }
    let count = grouped.len();
    for (text, spans) in grouped {
        let best = spans
            .iter()
            .max_by(|left, right| left.score.total_cmp(&right.score))
            .expect("group is not empty");
        if let Some(candidate) = candidates
            .iter_mut()
            .find(|candidate| candidate.text == text)
        {
            candidate.score = combined_confidence(candidate.score, best.score);
            if candidate.entity_type == NameEntityType::Unknown {
                candidate.entity_type = best.entity_type;
            }
            for span in spans {
                if !candidate.ranges.contains(&span.range) {
                    candidate.ranges.push(span.range);
                }
            }
            candidate.ranges.sort_by_key(|range| range.start);
            candidate.occurrences = candidate.ranges.len();
            if !candidate.sources.contains(&NameCandidateSource::OnnxNer) {
                candidate.sources.push(NameCandidateSource::OnnxNer);
                candidate
                    .reasons
                    .push("được mô hình ONNX NER xác nhận".to_string());
            }
        } else {
            let ranges = spans.iter().map(|span| span.range).collect::<Vec<_>>();
            candidates.push(NameCandidate {
                text: text.clone(),
                suggested: memory
                    .known_names
                    .get(&text)
                    .cloned()
                    .unwrap_or_else(|| engine.suggest_name(&text)),
                entity_type: best.entity_type,
                score: best.score,
                occurrences: ranges.len(),
                ranges,
                reasons: vec!["được mô hình ONNX NER nhận diện".to_string()],
                sources: vec![NameCandidateSource::OnnxNer],
                known: memory.known_names.contains_key(&text),
            });
        }
    }
    count
}

fn combined_confidence(left: f32, right: f32) -> f32 {
    (1.0 - (1.0 - left) * (1.0 - right)).clamp(0.0, 1.0)
}

fn apply_ai_decisions(
    candidates: &mut Vec<NameCandidate>,
    decisions: Vec<crate::name_ai::AiNameDecision>,
    threshold: f32,
) {
    let decisions: HashMap<_, _> = decisions
        .into_iter()
        .filter(|decision| decision.confidence >= threshold)
        .map(|decision| (decision.text.clone(), decision))
        .collect();
    candidates.retain_mut(|candidate| {
        let Some(decision) = decisions.get(&candidate.text) else {
            return true;
        };
        if !decision.keep {
            return false;
        }
        candidate.score = combined_confidence(candidate.score, decision.confidence);
        if let Some(entity_type) = decision.entity_type.as_deref() {
            candidate.entity_type = parse_entity_type(entity_type);
        }
        if let Some(suggested) = decision
            .suggested
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            candidate.suggested = suggested.to_string();
        }
        if !candidate.sources.contains(&NameCandidateSource::AiFallback) {
            candidate.sources.push(NameCandidateSource::AiFallback);
            candidate
                .reasons
                .push("được AI fallback xác nhận".to_string());
        }
        true
    });
}

fn name_candidate_response(chapter: &str, candidate: NameCandidate) -> NameCandidateResp {
    NameCandidateResp {
        contexts: candidate_contexts(chapter, &candidate.text, 36, 3),
        text: candidate.text,
        suggested: candidate.suggested,
        entity_type: entity_type_name(candidate.entity_type),
        score: candidate.score,
        occurrences: candidate.occurrences,
        ranges: candidate.ranges.into_iter().map(RangeDto::from).collect(),
        reasons: candidate.reasons,
        sources: candidate.sources.into_iter().map(source_name).collect(),
        known: candidate.known,
    }
}

fn source_name(source: NameCandidateSource) -> &'static str {
    match source {
        NameCandidateSource::QtJieba => "qt-jieba",
        NameCandidateSource::Ngram => "ngram",
        NameCandidateSource::ContextRule => "context-rule",
        NameCandidateSource::SurnameRule => "surname-rule",
        NameCandidateSource::SuffixRule => "suffix-rule",
        NameCandidateSource::BookMemory => "book-memory",
        NameCandidateSource::OnnxNer => "onnx-ner",
        NameCandidateSource::AiFallback => "ai-fallback",
    }
}

fn candidate_contexts(chapter: &str, candidate: &str, radius: usize, limit: usize) -> Vec<String> {
    chapter
        .match_indices(candidate)
        .take(limit)
        .map(|(byte_start, value)| {
            let before: String = chapter[..byte_start]
                .chars()
                .rev()
                .take(radius)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            let after: String = chapter[byte_start + value.len()..]
                .chars()
                .take(radius)
                .collect();
            format!("{before}【{value}】{after}")
        })
        .collect()
}

async fn translate_batch(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BatchReq>,
) -> Result<Json<BatchResp>, ApiError> {
    let mode = parse_mode(&req.mode)
        .ok_or_else(|| ApiError::bad_request(format!("invalid mode: {}", req.mode)))?;
    let options = request_options(
        req.wrap,
        req.scan_range,
        req.translation_algorithm,
        req.prioritized_name,
    )?;
    let dictionaries = prepare_dictionaries(req.dictionaries).await?;
    let mut translated = Vec::with_capacity(req.texts.len());
    let mut source_ranges = req.ranges.then(|| Vec::with_capacity(req.texts.len()));
    let mut target_ranges = req.ranges.then(|| Vec::with_capacity(req.texts.len()));
    for text in req.texts {
        let out = run_translate(
            state.engine.clone(),
            text,
            mode,
            options,
            req.pretty,
            dictionaries.clone(),
        )
        .await?;
        translated.push(out.translated_text);
        if let Some(ranges) = &mut source_ranges {
            ranges.push(out.source_ranges.into_iter().map(RangeDto::from).collect());
        }
        if let Some(ranges) = &mut target_ranges {
            ranges.push(out.target_ranges.into_iter().map(RangeDto::from).collect());
        }
    }
    Ok(Json(BatchResp {
        translated,
        source_ranges,
        target_ranges,
    }))
}

async fn translate(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TranslateReq>,
) -> Result<Json<TranslateResp>, ApiError> {
    let mode = parse_mode(&req.mode)
        .ok_or_else(|| ApiError::bad_request(format!("invalid mode: {}", req.mode)))?;
    let options = request_options(
        req.wrap,
        req.scan_range,
        req.translation_algorithm,
        req.prioritized_name,
    )?;
    let dictionaries = prepare_dictionaries(req.dictionaries).await?;
    let out = run_translate(
        state.engine.clone(),
        req.text,
        mode,
        options,
        req.pretty,
        dictionaries,
    )
    .await?;
    let (source_ranges, target_ranges) = if req.ranges {
        (
            Some(out.source_ranges.into_iter().map(RangeDto::from).collect()),
            Some(out.target_ranges.into_iter().map(RangeDto::from).collect()),
        )
    } else {
        (None, None)
    };
    Ok(Json(TranslateResp {
        translated: out.translated_text,
        source_ranges,
        target_ranges,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::name_ai::AiNameDecision;

    fn candidate(text: &str) -> NameCandidate {
        NameCandidate {
            text: text.to_string(),
            suggested: "Cũ".to_string(),
            entity_type: NameEntityType::Unknown,
            score: 0.5,
            occurrences: 1,
            ranges: vec![CharRange {
                start: 0,
                length: 2,
            }],
            reasons: Vec::new(),
            sources: Vec::new(),
            known: false,
        }
    }

    #[test]
    fn ai_decisions_are_bounded_to_candidates_and_threshold() {
        let mut candidates = vec![candidate("萧炎"), candidate("看向")];
        apply_ai_decisions(
            &mut candidates,
            vec![
                AiNameDecision {
                    text: "萧炎".to_string(),
                    keep: true,
                    confidence: 0.9,
                    entity_type: Some("person".to_string()),
                    suggested: Some("Tiêu Viêm".to_string()),
                },
                AiNameDecision {
                    text: "看向".to_string(),
                    keep: false,
                    confidence: 0.95,
                    entity_type: Some("unknown".to_string()),
                    suggested: None,
                },
            ],
            0.65,
        );
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].text, "萧炎");
        assert_eq!(candidates[0].suggested, "Tiêu Viêm");
        assert_eq!(candidates[0].entity_type, NameEntityType::Person);
        assert!(candidates[0]
            .sources
            .contains(&NameCandidateSource::AiFallback));
    }
}
