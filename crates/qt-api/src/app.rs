//! HTTP surface for the qt-core engine: state, router, handlers, DTOs.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::{DefaultBodyLimit, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;

use qt_core::{
    CharRange, DictionaryDefaults, DictionaryOverrides, DictionaryPatches,
    DictionarySourceOverrides, Engine, Mode, NameCandidate, NameCandidateSource, NameEntityType,
    NameFilterDocument, NameFilterMemory, NameFilterMode, NameFilterOptions, Options,
    TranslationResult,
};

use crate::name_ai::{
    build_ai_http_client, entity_type_name, parse_entity_type, AiBaseUrls, AiExtractedEntity,
    AiNameProvider,
};

const MAX_REQUEST_SCAN_RANGE: usize = 100;
const MAX_REQUEST_BODY_BYTES: usize = 5 * 1024 * 1024;
const MAX_NAME_FILTER_CHARACTERS: usize = 200_000;

/// Shared, read-only application state: the loaded engine behind an Arc.
pub struct AppState {
    pub engine: Arc<Engine>,
    pub dictionary_defaults: Arc<DictionaryDefaults>,
    pub name_filter_services: NameFilterServices,
}

/// Total wall-clock budget shared by all AI calls in one request. Must stay
/// under the Lambda timeout (120s) with headroom for rules and merging.
const DEFAULT_AI_DEADLINE_SECONDS: u64 = 100;

/// Request-independent pieces of the AI integration. The server holds no
/// provider API key — every request brings its own credentials — so this is
/// just the shared HTTP client, the operator-controlled provider endpoints
/// and the AI time budget.
#[derive(Clone)]
pub struct NameFilterServices {
    http: reqwest::Client,
    ai_base_urls: AiBaseUrls,
    ai_deadline: Duration,
    startup_warnings: Arc<Vec<String>>,
}

impl Default for NameFilterServices {
    fn default() -> Self {
        Self {
            http: build_ai_http_client(),
            ai_base_urls: AiBaseUrls::default(),
            ai_deadline: Duration::from_secs(DEFAULT_AI_DEADLINE_SECONDS),
            startup_warnings: Arc::default(),
        }
    }
}

impl NameFilterServices {
    pub fn from_env() -> Self {
        let mut startup_warnings = Vec::new();
        let ai_deadline = match crate::name_ai::non_empty_env("QT_AI_DEADLINE_SECONDS") {
            Some(value) => match value.parse::<u64>() {
                Ok(seconds) => Duration::from_secs(seconds.clamp(5, 600)),
                Err(_) => {
                    eprintln!("warning: QT_AI_DEADLINE_SECONDS is not a number, using default");
                    startup_warnings.push(
                        "QT_AI_DEADLINE_SECONDS is invalid; the default AI deadline is used"
                            .to_string(),
                    );
                    Duration::from_secs(DEFAULT_AI_DEADLINE_SECONDS)
                }
            },
            None => Duration::from_secs(DEFAULT_AI_DEADLINE_SECONDS),
        };
        Self {
            http: build_ai_http_client(),
            ai_base_urls: AiBaseUrls::from_env(),
            ai_deadline,
            startup_warnings: Arc::new(startup_warnings),
        }
    }

    fn ai_deadline(&self) -> Duration {
        self.ai_deadline
    }

    /// Build the per-request provider from client-supplied credentials.
    fn ai_provider_from(&self, creds: &AiCredentialsReq) -> Result<AiNameProvider, String> {
        AiNameProvider::from_credentials(
            self.http.clone(),
            &self.ai_base_urls,
            &creds.provider,
            &creds.api_key,
            creds.model.as_deref(),
        )
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
    dictionary_patches: Option<DictionaryPatchesReq>,
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

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DictionaryPatchesReq {
    #[serde(default)]
    viet_phrase: HashMap<String, String>,
    #[serde(default)]
    chinese_phien_am_words: HashMap<String, String>,
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
    /// Deprecated: the ONNX NER provider was removed. Accepted so older
    /// clients still parse; enabling it only produces a warning.
    #[serde(default)]
    ner: ProviderReq,
    /// Caller-supplied AI credentials; required for aiExtract/aiFallback.
    /// Used only for this request — never stored or logged.
    ai: Option<AiCredentialsReq>,
    #[serde(default)]
    ai_extract: ProviderReq,
    #[serde(default)]
    ai_fallback: AiFallbackReq,
    dictionaries: Option<DictionarySourcesReq>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AiCredentialsReq {
    provider: String,
    api_key: String,
    model: Option<String>,
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
    ai_extracted_candidates: usize,
    ai_reviewed: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NameFilterCapabilities {
    ai_configured: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    ai_provider: Option<&'static str>,
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

impl DictionaryPatchesReq {
    fn into_patches(self) -> Result<DictionaryPatches, ApiError> {
        let mut chinese_phien_am_words = HashMap::new();
        for (key, value) in self.chinese_phien_am_words {
            let mut chars = key.chars();
            let Some(ch) = chars.next() else {
                return Err(ApiError::bad_request(
                    "dictionaryPatches.chinesePhienAmWords keys must contain exactly one character",
                ));
            };
            if chars.next().is_some() {
                return Err(ApiError::bad_request(
                    "dictionaryPatches.chinesePhienAmWords keys must contain exactly one character",
                ));
            }
            chinese_phien_am_words.insert(ch, value);
        }
        if self.viet_phrase.keys().any(|key| key.is_empty()) {
            return Err(ApiError::bad_request(
                "dictionaryPatches.vietPhrase keys must not be empty",
            ));
        }
        Ok(DictionaryPatches {
            vietphrase: self.viet_phrase,
            chinese_phien_am_words,
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
    dictionary_patches: Option<DictionaryPatchesReq>,
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
    patches: Option<DictionaryPatchesReq>,
) -> Result<Option<Arc<DictionaryOverrides>>, ApiError> {
    if dictionaries.is_none() && patches.is_none() {
        return Ok(None);
    }
    let patches = patches
        .map(DictionaryPatchesReq::into_patches)
        .transpose()?;
    tokio::task::spawn_blocking(move || {
        let overrides = dictionaries
            .map(DictionarySourcesReq::into_overrides)
            .unwrap_or_default()
            .with_patches(patches.unwrap_or_default());
        Arc::new(overrides)
    })
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
    // Every option is validated before rules run or a paid provider call is
    // made, including options for features the request leaves disabled.
    let ai_extract_threshold = confidence(
        req.ai_extract.min_confidence,
        0.65,
        "aiExtract.minConfidence",
    )?;
    let ai_review_threshold = confidence(
        req.ai_fallback.min_confidence,
        0.65,
        "aiFallback.minConfidence",
    )?;
    let ai_min_rule_confidence = confidence(
        req.ai_fallback.min_rule_confidence,
        0.40,
        "aiFallback.minRuleConfidence",
    )?;
    let ai_max_rule_confidence = confidence(
        req.ai_fallback.max_rule_confidence,
        0.82,
        "aiFallback.maxRuleConfidence",
    )?;
    if ai_min_rule_confidence > ai_max_rule_confidence {
        return Err(ApiError::bad_request(
            "aiFallback.minRuleConfidence must not exceed maxRuleConfidence",
        ));
    }
    let ai_max_review_candidates = bounded_usize(
        req.ai_fallback.max_candidates,
        25,
        1,
        50,
        "aiFallback.maxCandidates",
    )?;
    // Client-supplied credentials build the per-request provider; the server
    // itself holds no AI key. Malformed credentials fail here, before rules
    // run or the caller's key is spent.
    let ai_provider = match req.ai.as_ref() {
        Some(creds) => Some(
            state
                .name_filter_services
                .ai_provider_from(creds)
                .map_err(ApiError::bad_request)?,
        ),
        None => None,
    };
    let options = NameFilterOptions {
        mode,
        min_occurrences: bounded_usize(req.min_occurrences, 2, 1, 100, "minOccurrences")?,
        min_score: min_confidence,
        max_candidates: bounded_usize(req.max_candidates, 200, 1, 1_000, "maxCandidates")?,
        max_name_length: bounded_usize(req.max_name_length, 8, 2, 8, "maxNameLength")?,
        include_known: req.include_known.unwrap_or(true),
    };
    let output_max_candidates = options.max_candidates;
    let include_known = options.include_known;
    // One shared deadline for all AI work in this request keeps the handler
    // inside the Lambda/client timeout even when extract and review both run.
    let ai_deadline = tokio::time::Instant::now() + state.name_filter_services.ai_deadline();
    let memory = NameFilterMemory {
        known_names: req.known_names,
        rejected_names: req.rejected_names.into_iter().collect(),
    };
    let dictionaries = prepare_dictionaries(req.dictionaries, None).await?;
    let text = Arc::new(req.text);
    let engine = state.engine.clone();
    let rule_text = text.clone();
    let rule_memory = memory.clone();
    let rule_dictionaries = dictionaries.clone();
    let (mut result, document) = tokio::task::spawn_blocking(move || {
        let document =
            Arc::new(engine.prepare_name_filter_document(&rule_text, rule_dictionaries.as_deref()));
        let result = engine.filter_names_in_document(
            &document,
            &options,
            &rule_memory,
            rule_dictionaries.as_deref(),
        );
        (result, document)
    })
    .await
    .map_err(|_| ApiError::internal("name filter task failed"))?;
    let rule_candidates = result.candidates.len();
    let mut warnings = state.name_filter_services.startup_warnings.as_ref().clone();

    if req.ner.enabled {
        warnings.push(
            "ner is deprecated: the ONNX NER provider was removed, use aiExtract".to_string(),
        );
    }

    let mut ai_extracted_candidates = 0;
    if req.ai_extract.enabled {
        if let Some(provider) = ai_provider.as_ref() {
            let extraction = provider.extract(document.text(), ai_deadline).await;
            warnings.extend(extraction.warnings);
            ai_extracted_candidates = merge_extracted_candidates(
                &state.engine,
                &mut result.candidates,
                extraction.entities,
                ExtractionMerge {
                    threshold: ai_extract_threshold,
                    include_known,
                },
                &memory,
                dictionaries.as_deref(),
                &document,
            );
        } else {
            warnings.push(
                "AI extract was requested but the request has no AI credentials (ai.apiKey)"
                    .to_string(),
            );
        }
    }

    let mut ai_reviewed = 0;
    if req.ai_fallback.enabled {
        if let Some(reviewer) = ai_provider.as_ref() {
            let ambiguous: Vec<NameCandidate> = result
                .candidates
                .iter()
                .filter(|candidate| {
                    !candidate.known
                        && (ai_min_rule_confidence..=ai_max_rule_confidence)
                            .contains(&candidate.score)
                })
                .cloned()
                .collect();
            match reviewer
                .review(
                    document.text(),
                    &ambiguous,
                    ai_max_review_candidates,
                    ai_deadline,
                )
                .await
            {
                Ok(decisions) => {
                    ai_reviewed = decisions.len();
                    apply_ai_decisions(&mut result.candidates, decisions, ai_review_threshold);
                }
                Err(error) => warnings.push(error),
            }
        } else {
            warnings.push(
                "AI fallback was requested but the request has no AI credentials (ai.apiKey)"
                    .to_string(),
            );
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
            ai_extracted_candidates,
            ai_reviewed,
        },
        // Per-request: whether this request carried valid AI credentials.
        capabilities: NameFilterCapabilities {
            ai_configured: ai_provider.is_some(),
            ai_provider: ai_provider.as_ref().map(AiNameProvider::provider_name),
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

/// Request-scoped knobs for merging AI-extracted entities.
struct ExtractionMerge {
    threshold: f32,
    include_known: bool,
}

/// Merge AI-extracted entities into the rule candidates. Entities carry only
/// text (no spans), so occurrences are located in the scan document and
/// mapped back to the caller's original UTF-16 input.
fn merge_extracted_candidates(
    engine: &Engine,
    candidates: &mut Vec<NameCandidate>,
    entities: Vec<AiExtractedEntity>,
    merge: ExtractionMerge,
    memory: &NameFilterMemory,
    dictionaries: Option<&DictionaryOverrides>,
    document: &NameFilterDocument,
) -> usize {
    let text = document.text();
    let mut utf16_starts: HashMap<usize, usize> = HashMap::new();
    let mut utf16_offset = 0usize;
    for (byte_start, character) in text.char_indices() {
        utf16_starts.insert(byte_start, utf16_offset);
        utf16_offset += character.len_utf16();
    }

    let mut merged = 0usize;
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for entity in entities {
        if entity.confidence < merge.threshold {
            continue;
        }
        let entity_text = entity.text.trim();
        if entity_text.is_empty()
            || !seen.insert(entity_text.to_string())
            || memory.rejected_names.contains(entity_text)
            || engine.contains_name(entity_text, dictionaries)
        {
            continue;
        }
        let known_value = memory.known_names.get(entity_text);
        // Rules already honored includeKnown; AI must not re-surface the
        // candidates the caller asked to hide.
        if known_value.is_some() && !merge.include_known {
            continue;
        }
        let entity_utf16_length = entity_text.encode_utf16().count();
        let ranges: Vec<CharRange> = text
            .match_indices(entity_text)
            .filter_map(|(byte_start, _)| {
                utf16_starts.get(&byte_start).and_then(|start| {
                    document.map_range(CharRange {
                        start: *start,
                        length: entity_utf16_length,
                    })
                })
            })
            .collect();
        if ranges.is_empty() {
            continue;
        }
        merged += 1;
        let entity_type = entity
            .entity_type
            .as_deref()
            .map(parse_entity_type)
            .unwrap_or(NameEntityType::Unknown);
        let suggested = entity
            .suggested
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        if let Some(candidate) = candidates
            .iter_mut()
            .find(|candidate| candidate.text == entity_text)
        {
            candidate.score = combined_confidence(candidate.score, entity.confidence);
            if candidate.entity_type == NameEntityType::Unknown {
                candidate.entity_type = entity_type;
            }
            if let Some(suggested) = suggested {
                if !candidate.known {
                    candidate.suggested = suggested.to_string();
                }
            }
            for range in ranges {
                if !candidate.ranges.contains(&range) {
                    candidate.ranges.push(range);
                }
            }
            candidate.ranges.sort_by_key(|range| range.start);
            candidate.occurrences = candidate.ranges.len();
            if !candidate.sources.contains(&NameCandidateSource::AiFallback) {
                candidate.sources.push(NameCandidateSource::AiFallback);
                candidate.reasons.push("được AI xác nhận".to_string());
            }
        } else {
            let is_known = known_value.is_some();
            let mut sources = vec![NameCandidateSource::AiFallback];
            let mut reasons = vec!["được AI trích xuất từ chương".to_string()];
            if is_known {
                sources.push(NameCandidateSource::BookMemory);
                reasons.push("đã được duyệt trong bộ nhớ truyện".to_string());
            }
            candidates.push(NameCandidate {
                text: entity_text.to_string(),
                suggested: known_value
                    .cloned()
                    .or_else(|| suggested.map(str::to_string))
                    .unwrap_or_else(|| engine.suggest_name(entity_text)),
                entity_type,
                // Known candidates keep the rules invariant of score 1.0.
                score: if is_known {
                    1.0
                } else {
                    entity.confidence.clamp(0.0, 1.0)
                },
                occurrences: ranges.len(),
                ranges,
                reasons,
                sources,
                known: is_known,
            });
        }
    }
    merged
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
        contexts: candidate_contexts(chapter, &candidate.ranges, 36, 3),
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
        NameCandidateSource::BookTitle => "book-title",
        NameCandidateSource::AiFallback => "ai-fallback",
    }
}

fn candidate_contexts(
    chapter: &str,
    ranges: &[CharRange],
    radius: usize,
    limit: usize,
) -> Vec<String> {
    ranges
        .iter()
        .take(limit)
        .filter_map(|range| {
            let (byte_start, byte_end) = utf16_range_to_bytes(chapter, *range)?;
            let value = &chapter[byte_start..byte_end];
            let before: String = chapter[..byte_start]
                .chars()
                .rev()
                .take(radius)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            let after: String = chapter[byte_end..].chars().take(radius).collect();
            Some(format!("{before}【{value}】{after}"))
        })
        .collect()
}

fn utf16_range_to_bytes(text: &str, range: CharRange) -> Option<(usize, usize)> {
    let end = range.start.checked_add(range.length)?;
    let mut utf16_offset = 0usize;
    let mut byte_start = (range.start == 0).then_some(0);
    let mut byte_end = (end == 0).then_some(0);
    for (byte_offset, ch) in text.char_indices() {
        if utf16_offset == range.start {
            byte_start.get_or_insert(byte_offset);
        }
        utf16_offset += ch.len_utf16();
        let next_byte = byte_offset + ch.len_utf8();
        if utf16_offset == end {
            byte_end.get_or_insert(next_byte);
            break;
        }
        if utf16_offset > end {
            return None;
        }
    }
    if utf16_offset == range.start {
        byte_start.get_or_insert(text.len());
    }
    if utf16_offset == end {
        byte_end.get_or_insert(text.len());
    }
    Some((byte_start?, byte_end?))
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
    let dictionaries = prepare_dictionaries(req.dictionaries, req.dictionary_patches).await?;
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
    let dictionaries = prepare_dictionaries(req.dictionaries, req.dictionary_patches).await?;
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

    #[test]
    fn extracted_entities_map_to_raw_text_and_skip_ignored_phrases() {
        let dictionaries = DictionaryDefaults {
            ignored_chinese_phrases: "本章完".to_string(),
            ..Default::default()
        }
        .build_dictionaries("本=bản\n章=chương\n完=hoàn\n萧=tiêu\n炎=viêm", "");
        let engine = Engine::from_dicts(dictionaries);
        let document = engine.prepare_name_filter_document("本章完萧炎", None);
        assert_eq!(document.text(), "\n\n\n萧炎");

        let mut candidates = Vec::new();
        let merged = merge_extracted_candidates(
            &engine,
            &mut candidates,
            vec![
                AiExtractedEntity {
                    text: "萧炎".to_string(),
                    entity_type: Some("person".to_string()),
                    suggested: Some("Tiêu Viêm".to_string()),
                    confidence: 0.9,
                },
                AiExtractedEntity {
                    text: "本章完".to_string(),
                    entity_type: None,
                    suggested: None,
                    confidence: 0.9,
                },
            ],
            ExtractionMerge {
                threshold: 0.65,
                include_known: true,
            },
            &NameFilterMemory::default(),
            None,
            &document,
        );

        assert_eq!(merged, 1);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].text, "萧炎");
        assert_eq!(candidates[0].suggested, "Tiêu Viêm");
        assert_eq!(candidates[0].entity_type, NameEntityType::Person);
        assert_eq!(
            candidates[0].ranges,
            vec![CharRange {
                start: 3,
                length: 2,
            }]
        );
    }

    fn extraction_engine_and_document() -> (Engine, NameFilterDocument) {
        let dictionaries = DictionaryDefaults::default().build_dictionaries("萧=tiêu\n炎=viêm", "");
        let engine = Engine::from_dicts(dictionaries);
        let document = engine.prepare_name_filter_document("萧炎来了", None);
        (engine, document)
    }

    fn extracted_entity(text: &str) -> AiExtractedEntity {
        AiExtractedEntity {
            text: text.to_string(),
            entity_type: Some("person".to_string()),
            suggested: None,
            confidence: 0.9,
        }
    }

    #[test]
    fn extraction_merge_skips_known_names_when_include_known_is_false() {
        let (engine, document) = extraction_engine_and_document();
        let memory = NameFilterMemory {
            known_names: HashMap::from([("萧炎".to_string(), "Tiêu Viêm".to_string())]),
            rejected_names: Default::default(),
        };
        let mut candidates = Vec::new();
        let merged = merge_extracted_candidates(
            &engine,
            &mut candidates,
            vec![extracted_entity("萧炎")],
            ExtractionMerge {
                threshold: 0.65,
                include_known: false,
            },
            &memory,
            None,
            &document,
        );
        assert_eq!(merged, 0);
        assert!(candidates.is_empty());
    }

    #[test]
    fn extraction_merge_keeps_known_score_invariant() {
        let (engine, document) = extraction_engine_and_document();
        let memory = NameFilterMemory {
            known_names: HashMap::from([("萧炎".to_string(), "Tiêu Viêm".to_string())]),
            rejected_names: Default::default(),
        };
        let mut candidates = Vec::new();
        let merged = merge_extracted_candidates(
            &engine,
            &mut candidates,
            vec![extracted_entity("萧炎")],
            ExtractionMerge {
                threshold: 0.65,
                include_known: true,
            },
            &memory,
            None,
            &document,
        );
        assert_eq!(merged, 1);
        assert_eq!(candidates.len(), 1);
        assert!(candidates[0].known);
        assert_eq!(candidates[0].score, 1.0);
        assert_eq!(candidates[0].suggested, "Tiêu Viêm");
        assert!(candidates[0]
            .sources
            .contains(&NameCandidateSource::BookMemory));
    }
}
