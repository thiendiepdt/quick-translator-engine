//! HTTP surface for the qt-core engine: state, router, handlers, DTOs.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;

use qt_core::{CharRange, Engine, Mode, Options, TranslationResult};

const MAX_REQUEST_SCAN_RANGE: usize = 100;

/// Shared, read-only application state: the loaded engine behind an Arc.
pub struct AppState {
    pub engine: Arc<Engine>,
}

/// Build the router from shared state. Kept separate from socket binding so
/// tests can exercise handlers in-process via `oneshot`.
pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/modes", get(modes))
        .route("/translate", post(translate))
        .route("/translate/batch", post(translate_batch))
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
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
) -> Result<TranslationResult, ApiError> {
    tokio::task::spawn_blocking(move || {
        let out = engine.translate_with_ranges(&text, mode, &opts);
        if pretty {
            prettify_result(out)
        } else {
            out
        }
    })
    .await
    .map_err(|_| ApiError::internal("translate task failed"))
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
    let mut translated = Vec::with_capacity(req.texts.len());
    let mut source_ranges = req.ranges.then(|| Vec::with_capacity(req.texts.len()));
    let mut target_ranges = req.ranges.then(|| Vec::with_capacity(req.texts.len()));
    for text in req.texts {
        let out = run_translate(state.engine.clone(), text, mode, options, req.pretty).await?;
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
    let out = run_translate(state.engine.clone(), req.text, mode, options, req.pretty).await?;
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
