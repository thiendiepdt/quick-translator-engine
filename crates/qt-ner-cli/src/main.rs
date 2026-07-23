use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use axum::body::Body;
use axum::http::Request;
use http_body_util::BodyExt;
use qt_core::parse_dict;
use serde::Serialize;
use serde_json::{json, Map, Value};
use tower::ServiceExt;

#[derive(Default)]
struct DictionaryFiles {
    names: Option<PathBuf>,
    names2: Option<PathBuf>,
    luat_nhan: Option<PathBuf>,
    pronouns: Option<PathBuf>,
    danh_tu: Option<PathBuf>,
    ho_nguoi: Option<PathBuf>,
    hau_tu: Option<PathBuf>,
    ignored_chinese_phrases: Option<PathBuf>,
}

struct CliOptions {
    data_dir: PathBuf,
    input: Option<PathBuf>,
    mode: String,
    min_occurrences: Option<usize>,
    min_confidence: Option<f32>,
    max_candidates: Option<usize>,
    max_name_length: Option<usize>,
    include_known: Option<bool>,
    known_names_file: Option<PathBuf>,
    rejected_names_file: Option<PathBuf>,
    dictionary_files: DictionaryFiles,
    ner_enabled: bool,
    ner_min_confidence: Option<f32>,
    ai_enabled: bool,
    ai_min_confidence: Option<f32>,
    ai_min_rule_confidence: Option<f32>,
    ai_max_rule_confidence: Option<f32>,
    ai_max_candidates: Option<usize>,
    json_output: bool,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("data"),
            input: None,
            mode: "hybrid".to_string(),
            min_occurrences: None,
            min_confidence: None,
            max_candidates: None,
            max_name_length: None,
            include_known: None,
            known_names_file: None,
            rejected_names_file: None,
            dictionary_files: DictionaryFiles::default(),
            ner_enabled: true,
            ner_min_confidence: None,
            ai_enabled: false,
            ai_min_confidence: None,
            ai_min_rule_confidence: None,
            ai_max_rule_confidence: None,
            ai_max_candidates: None,
            json_output: false,
        }
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let options = match parse_arguments(&arguments) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("error: {error}");
            print_usage();
            return ExitCode::from(2);
        }
    };

    match run(options).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn parse_arguments(arguments: &[String]) -> Result<CliOptions, String> {
    if arguments.first().map(String::as_str) != Some("filter") {
        return Err("expected the `filter` command".to_string());
    }
    let mut options = CliOptions::default();
    let mut index = 1;
    while index < arguments.len() {
        let argument = arguments[index].as_str();
        match argument {
            "--data-dir" => {
                options.data_dir = PathBuf::from(next_value(arguments, &mut index, argument)?);
            }
            "--input" => {
                options.input = Some(PathBuf::from(next_value(arguments, &mut index, argument)?));
            }
            "--mode" => {
                let value = next_value(arguments, &mut index, argument)?;
                if !matches!(value, "qt" | "hybrid") {
                    return Err("--mode needs qt or hybrid".to_string());
                }
                options.mode = value.to_string();
            }
            "--min-occurrences" => {
                options.min_occurrences = Some(parse_usize(
                    next_value(arguments, &mut index, argument)?,
                    argument,
                )?);
            }
            "--min-confidence" => {
                options.min_confidence = Some(parse_confidence(
                    next_value(arguments, &mut index, argument)?,
                    argument,
                )?);
            }
            "--max-candidates" => {
                options.max_candidates = Some(parse_usize(
                    next_value(arguments, &mut index, argument)?,
                    argument,
                )?);
            }
            "--max-name-length" => {
                options.max_name_length = Some(parse_usize(
                    next_value(arguments, &mut index, argument)?,
                    argument,
                )?);
            }
            "--include-known" => {
                options.include_known = Some(parse_bool(
                    next_value(arguments, &mut index, argument)?,
                    argument,
                )?);
            }
            "--known-names-file" => {
                options.known_names_file =
                    Some(PathBuf::from(next_value(arguments, &mut index, argument)?));
            }
            "--rejected-names-file" => {
                options.rejected_names_file =
                    Some(PathBuf::from(next_value(arguments, &mut index, argument)?));
            }
            "--ner" => options.ner_enabled = true,
            "--no-ner" => options.ner_enabled = false,
            "--ner-min-confidence" => {
                options.ner_min_confidence = Some(parse_confidence(
                    next_value(arguments, &mut index, argument)?,
                    argument,
                )?);
            }
            "--ai-fallback" => options.ai_enabled = true,
            "--ai-min-confidence" => {
                options.ai_min_confidence = Some(parse_confidence(
                    next_value(arguments, &mut index, argument)?,
                    argument,
                )?);
            }
            "--ai-min-rule-confidence" => {
                options.ai_min_rule_confidence = Some(parse_confidence(
                    next_value(arguments, &mut index, argument)?,
                    argument,
                )?);
            }
            "--ai-max-rule-confidence" => {
                options.ai_max_rule_confidence = Some(parse_confidence(
                    next_value(arguments, &mut index, argument)?,
                    argument,
                )?);
            }
            "--ai-max-candidates" => {
                options.ai_max_candidates = Some(parse_usize(
                    next_value(arguments, &mut index, argument)?,
                    argument,
                )?);
            }
            "--json" => options.json_output = true,
            dictionary_option @ ("--names-file"
            | "--names2-file"
            | "--luat-nhan-file"
            | "--pronouns-file"
            | "--danh-tu-file"
            | "--ho-nguoi-file"
            | "--hau-tu-file"
            | "--ignored-chinese-phrases-file") => {
                let path = PathBuf::from(next_value(arguments, &mut index, dictionary_option)?);
                match dictionary_option {
                    "--names-file" => options.dictionary_files.names = Some(path),
                    "--names2-file" => options.dictionary_files.names2 = Some(path),
                    "--luat-nhan-file" => options.dictionary_files.luat_nhan = Some(path),
                    "--pronouns-file" => options.dictionary_files.pronouns = Some(path),
                    "--danh-tu-file" => options.dictionary_files.danh_tu = Some(path),
                    "--ho-nguoi-file" => options.dictionary_files.ho_nguoi = Some(path),
                    "--hau-tu-file" => options.dictionary_files.hau_tu = Some(path),
                    "--ignored-chinese-phrases-file" => {
                        options.dictionary_files.ignored_chinese_phrases = Some(path);
                    }
                    _ => unreachable!(),
                }
            }
            other => return Err(format!("unknown argument {other}")),
        }
        index += 1;
    }
    Ok(options)
}

fn next_value<'a>(
    arguments: &'a [String],
    index: &mut usize,
    option: &str,
) -> Result<&'a str, String> {
    *index += 1;
    arguments
        .get(*index)
        .map(String::as_str)
        .ok_or_else(|| format!("{option} needs a value"))
}

fn parse_usize(value: &str, option: &str) -> Result<usize, String> {
    value
        .parse()
        .map_err(|_| format!("{option} needs a positive integer"))
}

fn parse_confidence(value: &str, option: &str) -> Result<f32, String> {
    let value = value
        .parse::<f32>()
        .map_err(|_| format!("{option} needs a number between 0 and 1"))?;
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(value)
    } else {
        Err(format!("{option} must be between 0 and 1"))
    }
}

fn parse_bool(value: &str, option: &str) -> Result<bool, String> {
    value
        .parse()
        .map_err(|_| format!("{option} needs true or false"))
}

async fn run(options: CliOptions) -> Result<(), String> {
    let text = read_input(options.input.as_ref())?;
    let state = qt_ner_api::load_state(&options.data_dir).map_err(|error| {
        format!(
            "failed to load dictionaries from {}: {error}",
            options.data_dir.display()
        )
    })?;
    let request_json = build_request(&options, text)?;
    let request = Request::builder()
        .method("POST")
        .uri("/names/filter")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&request_json).map_err(
            |error| format!("failed to serialize request: {error}"),
        )?))
        .map_err(|error| format!("failed to build request: {error}"))?;
    let response = qt_ner_api::build_router(state)
        .oneshot(request)
        .await
        .map_err(|error| format!("name filter request failed: {error}"))?;
    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .map_err(|error| format!("failed to read response: {error}"))?
        .to_bytes();
    if !status.is_success() {
        let message = serde_json::from_slice::<Value>(&body)
            .ok()
            .and_then(|value| value["error"].as_str().map(str::to_string))
            .unwrap_or_else(|| String::from_utf8_lossy(&body).into_owned());
        return Err(format!("name filter returned {status}: {message}"));
    }
    let response: Value = serde_json::from_slice(&body)
        .map_err(|error| format!("invalid name filter response: {error}"))?;
    write_output(&response, options.json_output)
}

fn build_request(options: &CliOptions, text: String) -> Result<Value, String> {
    let known_names = options
        .known_names_file
        .as_ref()
        .map(|path| read_file(path).map(|content| parse_dict(&content).into_iter().collect()))
        .transpose()?
        .unwrap_or_else(HashMap::<String, String>::new);
    let rejected_names = options
        .rejected_names_file
        .as_ref()
        .map(|path| {
            read_file(path).map(|content| {
                content
                    .lines()
                    .map(|line| line.trim().trim_start_matches('\u{feff}'))
                    .filter(|line| !line.is_empty())
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
        })
        .transpose()?
        .unwrap_or_default();
    let dictionaries = load_dictionary_files(&options.dictionary_files)?;
    let mut request = json!({
        "text": text,
        "mode": options.mode,
        "knownNames": known_names,
        "rejectedNames": rejected_names,
        "ner": {
            "enabled": options.ner_enabled
        },
        "aiFallback": {
            "enabled": options.ai_enabled
        }
    });
    insert_optional(&mut request, "minOccurrences", options.min_occurrences);
    insert_optional(&mut request, "minConfidence", options.min_confidence);
    insert_optional(&mut request, "maxCandidates", options.max_candidates);
    insert_optional(&mut request, "maxNameLength", options.max_name_length);
    insert_optional(&mut request, "includeKnown", options.include_known);
    insert_nested_optional(
        &mut request,
        "ner",
        "minConfidence",
        options.ner_min_confidence,
    );
    insert_nested_optional(
        &mut request,
        "aiFallback",
        "minConfidence",
        options.ai_min_confidence,
    );
    insert_nested_optional(
        &mut request,
        "aiFallback",
        "minRuleConfidence",
        options.ai_min_rule_confidence,
    );
    insert_nested_optional(
        &mut request,
        "aiFallback",
        "maxRuleConfidence",
        options.ai_max_rule_confidence,
    );
    insert_nested_optional(
        &mut request,
        "aiFallback",
        "maxCandidates",
        options.ai_max_candidates,
    );
    if !dictionaries.is_empty() {
        request["dictionaries"] = Value::Object(dictionaries);
    }
    Ok(request)
}

fn insert_optional<T: Serialize>(request: &mut Value, field: &str, value: Option<T>) {
    if let Some(value) = value {
        request[field] = json!(value);
    }
}

fn insert_nested_optional<T: Serialize>(
    request: &mut Value,
    object: &str,
    field: &str,
    value: Option<T>,
) {
    if let Some(value) = value {
        request[object][field] = json!(value);
    }
}

fn load_dictionary_files(files: &DictionaryFiles) -> Result<Map<String, Value>, String> {
    let mut dictionaries = Map::new();
    for (field, path) in [
        ("names", &files.names),
        ("names2", &files.names2),
        ("luatNhan", &files.luat_nhan),
        ("pronouns", &files.pronouns),
        ("danhTu", &files.danh_tu),
        ("hoNguoi", &files.ho_nguoi),
        ("hauTu", &files.hau_tu),
        ("ignoredChinesePhrases", &files.ignored_chinese_phrases),
    ] {
        if let Some(path) = path {
            dictionaries.insert(field.to_string(), Value::String(read_file(path)?));
        }
    }
    Ok(dictionaries)
}

fn read_input(path: Option<&PathBuf>) -> Result<String, String> {
    match path {
        Some(path) => read_file(path),
        None => {
            let mut input = String::new();
            std::io::stdin()
                .read_to_string(&mut input)
                .map_err(|error| format!("failed to read stdin: {error}"))?;
            Ok(input)
        }
    }
}

fn read_file(path: &PathBuf) -> Result<String, String> {
    std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))
}

fn write_output(response: &Value, json_output: bool) -> Result<(), String> {
    let output = if json_output {
        serde_json::to_string_pretty(response)
            .map_err(|error| format!("failed to serialize output: {error}"))?
    } else {
        for warning in response["warnings"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            eprintln!("warning: {warning}");
        }
        response["candidates"]
            .as_array()
            .ok_or_else(|| "response has no candidates array".to_string())?
            .iter()
            .filter_map(|candidate| {
                Some(format!(
                    "{}={}",
                    candidate["text"].as_str()?,
                    candidate["suggested"].as_str()?
                ))
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let mut stdout = std::io::stdout();
    stdout
        .write_all(output.as_bytes())
        .and_then(|_| {
            if output.is_empty() || output.ends_with('\n') {
                Ok(())
            } else {
                stdout.write_all(b"\n")
            }
        })
        .map_err(|error| format!("failed to write stdout: {error}"))
}

fn print_usage() {
    eprintln!(
        "usage: qt-ner-cli filter [--data-dir DIR] [--input FILE] \
         [--mode qt|hybrid] [--min-occurrences N] [--min-confidence 0..1] \
         [--max-candidates N] [--max-name-length 2..8] [--include-known true|false] \
         [--known-names-file FILE] [--rejected-names-file FILE] \
         [--ner|--no-ner] [--ner-min-confidence 0..1] \
         [--ai-fallback] [--ai-min-confidence 0..1] \
         [--ai-min-rule-confidence 0..1] [--ai-max-rule-confidence 0..1] \
         [--ai-max-candidates N] [dictionary file options] [--json]"
    );
}
