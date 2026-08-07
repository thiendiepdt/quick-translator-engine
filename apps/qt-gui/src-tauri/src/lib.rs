use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use qt_core::{
    CharRange, Dictionaries, DictionaryDefaults, DictionaryOverrides, DictionaryPatches,
    DictionarySourceOverrides, Engine, Mode, NameCandidate, NameCandidateSource, NameEntityType,
    NameFilterMemory, NameFilterMode, NameFilterOptions, Options, TranslationResult,
};
use serde::{Deserialize, Serialize};
use tauri::State;

const MAX_SCAN_RANGE: usize = 100;
const MAX_TEXT_FILE_BYTES: u64 = 16 * 1024 * 1024;
const MAX_NAME_FILTER_CHARACTERS: usize = 200_000;

struct LoadedEngine {
    engine: Engine,
    defaults: DictionaryDefaults,
    data_dir: PathBuf,
}

#[derive(Default)]
struct AppState {
    loaded: RwLock<Option<Arc<LoadedEngine>>>,
}

impl AppState {
    fn new(loaded: Option<LoadedEngine>) -> Self {
        Self {
            loaded: RwLock::new(loaded.map(Arc::new)),
        }
    }

    fn snapshot(&self) -> Result<Arc<LoadedEngine>, String> {
        self.loaded
            .read()
            .map_err(|_| "Engine state is unavailable".to_string())?
            .clone()
            .ok_or_else(|| "Chưa nạp thư mục dữ liệu QT2025".to_string())
    }

    fn replace(&self, loaded: LoadedEngine) -> Result<(), String> {
        *self
            .loaded
            .write()
            .map_err(|_| "Engine state is unavailable".to_string())? = Some(Arc::new(loaded));
        Ok(())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EngineStatus {
    ready: bool,
    data_dir: Option<String>,
    message: String,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TranslationRequest {
    text: String,
    mode: String,
    #[serde(default)]
    wrap: bool,
    #[serde(default)]
    pretty: bool,
    #[serde(default)]
    ranges: bool,
    scan_range: usize,
    translation_algorithm: i32,
    prioritized_name: bool,
    dictionaries: Option<DictionarySourcesRequest>,
    dictionary_patches: Option<DictionaryPatchesRequest>,
}

#[derive(Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DictionarySourcesRequest {
    names: Option<String>,
    names2: Option<String>,
    luat_nhan: Option<String>,
    pronouns: Option<String>,
    danh_tu: Option<String>,
    ho_nguoi: Option<String>,
    hau_tu: Option<String>,
    ignored_chinese_phrases: Option<String>,
}

impl DictionarySourcesRequest {
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

#[derive(Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DictionaryPatchesRequest {
    #[serde(default)]
    viet_phrase: HashMap<String, String>,
    #[serde(default)]
    chinese_phien_am_words: HashMap<String, String>,
}

impl DictionaryPatchesRequest {
    fn into_patches(self) -> Result<DictionaryPatches, String> {
        if self.viet_phrase.keys().any(String::is_empty) {
            return Err("VietPhrase patch không được có key rỗng".to_string());
        }
        let mut chinese_phien_am_words = HashMap::new();
        for (key, value) in self.chinese_phien_am_words {
            let mut chars = key.chars();
            let Some(ch) = chars.next() else {
                return Err("Key phiên âm phải có đúng một ký tự".to_string());
            };
            if chars.next().is_some() {
                return Err("Key phiên âm phải có đúng một ký tự".to_string());
            }
            chinese_phien_am_words.insert(ch, value);
        }
        Ok(DictionaryPatches {
            vietphrase: self.viet_phrase.into_iter().collect(),
            chinese_phien_am_words: chinese_phien_am_words.into_iter().collect(),
        })
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TranslationResponse {
    translated: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_ranges: Option<Vec<RangeDto>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_ranges: Option<Vec<RangeDto>>,
}

#[derive(Clone, Copy, Serialize)]
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DictionaryDefaultsResponse {
    names: String,
    names2: String,
    luat_nhan: String,
    pronouns: String,
    danh_tu: String,
    ho_nguoi: String,
    hau_tu: String,
    ignored_chinese_phrases: String,
}

impl From<DictionaryDefaults> for DictionaryDefaultsResponse {
    fn from(value: DictionaryDefaults) -> Self {
        Self {
            names: value.names,
            names2: value.names2,
            luat_nhan: value.luat_nhan,
            pronouns: value.pronouns,
            danh_tu: value.danh_tu,
            ho_nguoi: value.ho_nguoi,
            hau_tu: value.hau_tu,
            ignored_chinese_phrases: value.ignored_chinese_phrases,
        }
    }
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NameFilterRequest {
    text: String,
    mode: String,
    min_occurrences: usize,
    min_confidence: f32,
    max_candidates: usize,
    known_names: HashMap<String, String>,
    rejected_names: Vec<String>,
    dictionaries: Option<DictionarySourcesRequest>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NameFilterResponse {
    candidates: Vec<NameCandidateResponse>,
    stats: NameFilterStats,
    capabilities: NameFilterCapabilities,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    warnings: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NameCandidateResponse {
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

// GUI chạy hoàn toàn local nên không có AI provider; giữ cùng shape với
// qt-api để component không phân nhánh theo nguồn response.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NameFilterCapabilities {
    ai_configured: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenedTextFile {
    path: String,
    name: String,
    content: String,
}

#[tauri::command]
fn engine_status(state: State<'_, AppState>) -> EngineStatus {
    match state.snapshot() {
        Ok(loaded) => EngineStatus {
            ready: true,
            data_dir: Some(loaded.data_dir.to_string_lossy().into_owned()),
            message: "Engine local đã sẵn sàng".to_string(),
        },
        Err(message) => EngineStatus {
            ready: false,
            data_dir: None,
            message,
        },
    }
}

#[tauri::command]
async fn load_engine(data_dir: String, state: State<'_, AppState>) -> Result<EngineStatus, String> {
    let path = PathBuf::from(data_dir);
    let loaded = tauri::async_runtime::spawn_blocking(move || load_from_directory(&path))
        .await
        .map_err(|error| format!("Không thể khởi chạy tác vụ nạp engine: {error}"))??;
    let data_dir = loaded.data_dir.to_string_lossy().into_owned();
    state.replace(loaded)?;
    Ok(EngineStatus {
        ready: true,
        data_dir: Some(data_dir),
        message: "Đã nạp engine và từ điển local".to_string(),
    })
}

#[tauri::command]
async fn dictionary_defaults(
    state: State<'_, AppState>,
) -> Result<DictionaryDefaultsResponse, String> {
    let loaded = state.snapshot()?;
    Ok(loaded.defaults.clone().into())
}

#[tauri::command]
async fn translate(
    request: TranslationRequest,
    state: State<'_, AppState>,
) -> Result<TranslationResponse, String> {
    let loaded = state.snapshot()?;
    tauri::async_runtime::spawn_blocking(move || translate_local(&loaded.engine, request))
        .await
        .map_err(|error| format!("Tác vụ dịch bị dừng: {error}"))?
}

#[tauri::command]
async fn filter_names(
    request: NameFilterRequest,
    state: State<'_, AppState>,
) -> Result<NameFilterResponse, String> {
    let loaded = state.snapshot()?;
    tauri::async_runtime::spawn_blocking(move || filter_names_local(&loaded.engine, request))
        .await
        .map_err(|error| format!("Tác vụ lọc name bị dừng: {error}"))?
}

#[tauri::command]
async fn read_text_file(path: String) -> Result<OpenedTextFile, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let path = PathBuf::from(path);
        let metadata = std::fs::metadata(&path)
            .map_err(|error| format!("Không đọc được thông tin file: {error}"))?;
        if !metadata.is_file() {
            return Err("Đường dẫn đã chọn không phải file".to_string());
        }
        if metadata.len() > MAX_TEXT_FILE_BYTES {
            return Err("File văn bản vượt quá 16 MiB".to_string());
        }
        let bytes =
            std::fs::read(&path).map_err(|error| format!("Không đọc được file: {error}"))?;
        let content = String::from_utf8(bytes)
            .map_err(|_| "File phải dùng mã hóa UTF-8 (có thể có BOM)".to_string())?;
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("chapter.txt")
            .to_string();
        Ok(OpenedTextFile {
            path: path.to_string_lossy().into_owned(),
            name,
            content,
        })
    })
    .await
    .map_err(|error| format!("Tác vụ đọc file bị dừng: {error}"))?
}

#[tauri::command]
async fn write_text_file(path: String, content: String) -> Result<(), String> {
    if content.len() as u64 > MAX_TEXT_FILE_BYTES {
        return Err("Nội dung vượt quá 16 MiB".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || {
        std::fs::write(PathBuf::from(path), content)
            .map_err(|error| format!("Không ghi được file: {error}"))
    })
    .await
    .map_err(|error| format!("Tác vụ ghi file bị dừng: {error}"))?
}

fn load_from_directory(data_dir: &Path) -> Result<LoadedEngine, String> {
    let canonical = data_dir
        .canonicalize()
        .map_err(|error| format!("Thư mục dữ liệu không tồn tại: {error}"))?;
    if !canonical.is_dir() {
        return Err("Đường dẫn dữ liệu không phải thư mục".to_string());
    }
    let (dictionaries, defaults) = Dictionaries::load_with_defaults(&canonical).map_err(|error| {
        format!(
            "Không nạp được từ điển tại {}: {error}. Cần Resources/ChinesePhienAmWords.txt và VietPhrase/VietPhrase.txt",
            canonical.display()
        )
    })?;
    Ok(LoadedEngine {
        engine: Engine::from_dicts(dictionaries),
        defaults,
        data_dir: canonical,
    })
}

fn find_default_data_directory() -> Option<PathBuf> {
    if let Ok(configured) = std::env::var("QT_DATA_DIR") {
        let path = PathBuf::from(configured);
        if is_data_directory(&path) {
            return Some(path);
        }
    }

    let mut starts = Vec::new();
    if let Ok(current) = std::env::current_dir() {
        starts.push(current);
    }
    if let Ok(executable) = std::env::current_exe() {
        if let Some(parent) = executable.parent() {
            starts.push(parent.to_path_buf());
        }
    }

    for start in starts {
        for ancestor in start.ancestors().take(8) {
            for candidate in [
                ancestor.to_path_buf(),
                ancestor.join("QT2025"),
                ancestor.join("data"),
            ] {
                if is_data_directory(&candidate) {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

fn is_data_directory(path: &Path) -> bool {
    path.join("Resources/ChinesePhienAmWords.txt").is_file()
        && path.join("VietPhrase/VietPhrase.txt").is_file()
}

fn parse_mode(value: &str) -> Result<Mode, String> {
    match value {
        "hanviet" => Ok(Mode::HanViet),
        "vietphrase" => Ok(Mode::VietPhrase),
        "vietphrase-one" => Ok(Mode::VietPhraseOneMeaning),
        other => Err(format!("Mode dịch không hợp lệ: {other}")),
    }
}

fn request_options(request: &TranslationRequest) -> Result<Options, String> {
    if !(1..=MAX_SCAN_RANGE).contains(&request.scan_range) {
        return Err(format!("scanRange phải nằm trong 1..={MAX_SCAN_RANGE}"));
    }
    if !matches!(request.translation_algorithm, 0..=2) {
        return Err("translationAlgorithm phải là 0, 1 hoặc 2".to_string());
    }
    Ok(Options {
        wrap_type: i32::from(request.wrap),
        translation_algorithm: request.translation_algorithm,
        prioritized_name: request.prioritized_name,
        scan_range: request.scan_range,
    })
}

fn prepare_overrides(
    dictionaries: Option<DictionarySourcesRequest>,
    patches: Option<DictionaryPatchesRequest>,
) -> Result<Option<DictionaryOverrides>, String> {
    if dictionaries.is_none() && patches.is_none() {
        return Ok(None);
    }
    let patches = patches
        .map(DictionaryPatchesRequest::into_patches)
        .transpose()?
        .unwrap_or_default();
    Ok(Some(
        dictionaries
            .map(DictionarySourcesRequest::into_overrides)
            .unwrap_or_default()
            .with_patches(patches),
    ))
}

fn translate_local(
    engine: &Engine,
    request: TranslationRequest,
) -> Result<TranslationResponse, String> {
    let mode = parse_mode(&request.mode)?;
    let options = request_options(&request)?;
    let overrides = prepare_overrides(request.dictionaries, request.dictionary_patches)?;
    let output = match overrides.as_ref() {
        Some(overrides) => engine
            .translate_with_ranges_and_overrides(&request.text, mode, &options, overrides)
            .map_err(|error| error.to_string())?,
        None => engine.translate_with_ranges(&request.text, mode, &options),
    };
    let output = if request.pretty {
        prettify_result(output)
    } else {
        output
    };
    let (source_ranges, target_ranges) = if request.ranges {
        (
            Some(
                output
                    .source_ranges
                    .into_iter()
                    .map(RangeDto::from)
                    .collect(),
            ),
            Some(
                output
                    .target_ranges
                    .into_iter()
                    .map(RangeDto::from)
                    .collect(),
            ),
        )
    } else {
        (None, None)
    };
    Ok(TranslationResponse {
        translated: output.translated_text,
        source_ranges,
        target_ranges,
    })
}

fn prettify_result(mut result: TranslationResult) -> TranslationResult {
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
    result.translated_text = upper + chars.as_str();

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

fn filter_names_local(
    engine: &Engine,
    request: NameFilterRequest,
) -> Result<NameFilterResponse, String> {
    let scanned_characters = request.text.chars().count();
    if scanned_characters > MAX_NAME_FILTER_CHARACTERS {
        return Err(format!(
            "Chương không được vượt quá {MAX_NAME_FILTER_CHARACTERS} ký tự"
        ));
    }
    let mode = match request.mode.as_str() {
        "qt" => NameFilterMode::QtCompatible,
        "hybrid" => NameFilterMode::Hybrid,
        other => return Err(format!("Mode lọc name không hợp lệ: {other}")),
    };
    if !(0.0..=1.0).contains(&request.min_confidence) {
        return Err("minConfidence phải nằm trong 0..=1".to_string());
    }
    if !(1..=100).contains(&request.min_occurrences) {
        return Err("minOccurrences phải nằm trong 1..=100".to_string());
    }
    if !(1..=1_000).contains(&request.max_candidates) {
        return Err("maxCandidates phải nằm trong 1..=1000".to_string());
    }

    let overrides = request
        .dictionaries
        .map(DictionarySourcesRequest::into_overrides);
    let options = NameFilterOptions {
        mode,
        min_occurrences: request.min_occurrences,
        min_score: request.min_confidence,
        max_candidates: request.max_candidates,
        max_name_length: 8,
        include_known: true,
    };
    let memory = NameFilterMemory {
        known_names: request.known_names.into_iter().collect(),
        rejected_names: request.rejected_names.into_iter().collect(),
    };
    let mut result = engine.filter_names(&request.text, &options, &memory, overrides.as_ref());
    result.candidates.sort_by(|left, right| {
        right
            .known
            .cmp(&left.known)
            .then_with(|| right.score.total_cmp(&left.score))
            .then_with(|| right.occurrences.cmp(&left.occurrences))
    });
    result.candidates.truncate(request.max_candidates);
    let rule_candidates = result.candidates.len();
    Ok(NameFilterResponse {
        candidates: result
            .candidates
            .into_iter()
            .map(|candidate| name_candidate_response(&request.text, candidate))
            .collect(),
        stats: NameFilterStats {
            scanned_characters: result.scanned_characters,
            rule_candidates,
            ai_extracted_candidates: 0,
            ai_reviewed: 0,
        },
        capabilities: NameFilterCapabilities {
            ai_configured: false,
        },
        warnings: Vec::new(),
    })
}

fn name_candidate_response(chapter: &str, candidate: NameCandidate) -> NameCandidateResponse {
    NameCandidateResponse {
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

fn entity_type_name(value: NameEntityType) -> &'static str {
    match value {
        NameEntityType::Person => "person",
        NameEntityType::Location => "location",
        NameEntityType::Organization => "organization",
        NameEntityType::Title => "title",
        NameEntityType::Unknown => "unknown",
    }
}

fn source_name(value: NameCandidateSource) -> &'static str {
    match value {
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
            let before = chapter[..byte_start]
                .chars()
                .rev()
                .take(radius)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<String>();
            let after = chapter[byte_end..].chars().take(radius).collect::<String>();
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

pub fn run() {
    let initial = find_default_data_directory()
        .as_deref()
        .and_then(|path| load_from_directory(path).ok());
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new(initial))
        .invoke_handler(tauri::generate_handler![
            engine_status,
            load_engine,
            dictionary_defaults,
            translate,
            filter_names,
            read_text_file,
            write_text_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running Quick Translator");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_required_data_files() {
        let missing = std::env::temp_dir().join("qt-gui-missing-data");
        assert!(!is_data_directory(&missing));
    }

    #[test]
    fn prettify_preserves_parallel_range_count() {
        let result = prettify_result(TranslationResult {
            translated_text: "  hello".to_string(),
            source_ranges: vec![CharRange {
                start: 0,
                length: 1,
            }],
            target_ranges: vec![CharRange {
                start: 2,
                length: 5,
            }],
        });
        assert_eq!(result.translated_text, "Hello");
        assert_eq!(result.source_ranges.len(), result.target_ranges.len());
        assert_eq!(
            result.target_ranges[0],
            CharRange {
                start: 0,
                length: 5
            }
        );
    }

    #[test]
    fn translates_with_checkout_qt2025_data_in_all_modes() {
        let data_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../QT2025");
        let loaded = load_from_directory(&data_dir).expect("checkout QT2025 data should load");

        for mode in ["hanviet", "vietphrase", "vietphrase-one"] {
            let response = translate_local(
                &loaded.engine,
                TranslationRequest {
                    text: "他的眼球很好。".to_string(),
                    mode: mode.to_string(),
                    wrap: false,
                    pretty: true,
                    ranges: true,
                    scan_range: 30,
                    translation_algorithm: 1,
                    prioritized_name: true,
                    dictionaries: None,
                    dictionary_patches: None,
                },
            )
            .unwrap_or_else(|error| panic!("{mode} translation failed: {error}"));

            assert!(!response.translated.is_empty(), "{mode} output is empty");
            assert_eq!(
                response.source_ranges.as_ref().map(Vec::len),
                response.target_ranges.as_ref().map(Vec::len),
                "{mode} range arrays must stay parallel"
            );
        }
    }
}
