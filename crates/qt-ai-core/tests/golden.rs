//! So từng byte với fixtures do apps/qt-ai-cli/scripts/gen-golden.ts sinh từ code TS thật.
use qt_ai_core::paragraphs::*;
use qt_ai_core::prompt::{
    build_system_prompt, filter_glossary_for_source, glossary_entry_matches_source, TranslationGlossary,
};
use qt_ai_core::story::{natural_chapter_compare, Glossary, StoryConfig};
use serde::Deserialize;
use serde_json::Value;

fn fixture<T: for<'de> Deserialize<'de>>(text: &str) -> T {
    serde_json::from_str(text).expect("fixture JSON hợp lệ")
}

#[derive(Deserialize)]
struct StoryFixture {
    normalize: Vec<NormalizeCase>,
    sort: SortCase,
}
#[derive(Deserialize)]
struct NormalizeCase {
    input: Value,
    output: Value,
}
#[derive(Deserialize)]
struct SortCase {
    input: Vec<String>,
    sorted: Vec<String>,
}

#[test]
fn story_normalize_va_sort_khop_ts() {
    let f: StoryFixture = fixture(include_str!("fixtures/story.json"));
    for case in f.normalize {
        let got = serde_json::to_value(StoryConfig::normalize(&case.input)).unwrap();
        assert_eq!(got, case.output);
    }
    let mut ids = f.sort.input.clone();
    ids.sort_by(|a, b| natural_chapter_compare(a, b));
    assert_eq!(ids, f.sort.sorted);
}

#[derive(Deserialize)]
struct ParagraphsFixture {
    #[serde(rename = "paragraphsOf")]
    paragraphs_of: Vec<ParagraphsOfCase>,
    #[serde(rename = "sourcePayload")]
    source_payload: Vec<SourcePayloadCase>,
    #[serde(rename = "repairPayload")]
    repair_payload: Vec<RepairPayloadCase>,
    parse: Vec<ParseCase>,
    strip: Vec<InOut>,
    format: Vec<InOut>,
}
#[derive(Deserialize)]
struct ParagraphsOfCase {
    text: String,
    result: Vec<String>,
}
#[derive(Deserialize)]
struct SourcePayloadCase {
    paragraphs: Vec<String>,
    payload: String,
}
#[derive(Deserialize)]
struct RepairPayloadCase {
    paragraphs: Vec<String>,
    missing: Vec<usize>,
    payload: String,
}
#[derive(Deserialize)]
struct ParseCase {
    output: String,
    count: usize,
    result: Option<Vec<Option<String>>>,
}
#[derive(Deserialize)]
struct InOut {
    input: String,
    output: String,
}

#[test]
fn paragraphs_khop_ts() {
    let f: ParagraphsFixture = fixture(include_str!("fixtures/paragraphs.json"));
    for c in f.paragraphs_of {
        assert_eq!(paragraphs_of(&c.text), c.result, "paragraphs_of {:?}", c.text);
    }
    for c in f.source_payload {
        assert_eq!(labeled_source_payload(&c.paragraphs), c.payload);
    }
    for c in f.repair_payload {
        assert_eq!(labeled_repair_payload(&c.paragraphs, &c.missing), c.payload);
    }
    for c in f.parse {
        assert_eq!(parse_labeled_translation(&c.output, c.count), c.result, "parse {:?}", c.output);
    }
    for c in f.strip {
        assert_eq!(strip_markers(&c.input), c.output);
    }
    for c in f.format {
        assert_eq!(format_translation(&c.input), c.output, "format {:?}", c.input);
    }
}

#[derive(Deserialize)]
struct PromptFixture {
    cases: Vec<PromptCase>,
}
#[derive(Deserialize)]
struct PromptCase {
    name: String,
    story: Option<Value>,
    source: Option<String>,
    prompt: String,
}

#[test]
fn prompt_khop_tung_byte_voi_web() {
    let f: PromptFixture = fixture(include_str!("fixtures/prompt.json"));
    for case in f.cases {
        let story = case.story.as_ref().map(StoryConfig::normalize);
        let got = build_system_prompt(&TranslationGlossary::new(), story.as_ref(), case.source.as_deref());
        assert_eq!(got, case.prompt, "case {}", case.name);
    }
}

#[derive(Deserialize)]
struct GlossaryFixture {
    matches: Vec<MatchCase>,
    filter: Vec<FilterCase>,
    sanitize: Vec<SanitizeCase>,
    append: Vec<AppendCase>,
}
#[derive(Deserialize)]
struct MatchCase {
    source: String,
    text: String,
    result: bool,
}
#[derive(Deserialize)]
struct FilterCase {
    glossary: Glossary,
    source: String,
    result: Glossary,
}
#[derive(Deserialize)]
#[allow(dead_code)]
struct SanitizeCase {
    entries: Value,
    raw: String,
    translation: String,
    #[serde(rename = "existingKeys")]
    existing_keys: Vec<String>,
    result: Value,
}
#[derive(Deserialize)]
#[allow(dead_code)]
struct AppendCase {
    story: Value,
    pairs: Value,
    chapter: String,
    result: Value,
}

#[test]
fn glossary_matches_va_filter_khop_web() {
    let f: GlossaryFixture = fixture(include_str!("fixtures/glossary.json"));
    for c in f.matches {
        assert_eq!(glossary_entry_matches_source(&c.source, &c.text), c.result, "{} in {}", c.source, c.text);
    }
    for c in f.filter {
        assert_eq!(filter_glossary_for_source(&c.glossary, &c.source), c.result);
    }
}
