use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use qt_core::{
    parse_dict, Dictionaries, DictionaryOverrides, DictionarySourceOverrides, Engine, Mode,
    NameCandidateSource, NameEntityType, NameFilterMemory, NameFilterMode, NameFilterOptions,
    Options,
};

const MAX_SCAN_RANGE: usize = 100;

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

fn parse_mode(s: &str) -> Option<Mode> {
    match s {
        "hanviet" => Some(Mode::HanViet),
        "vietphrase" => Some(Mode::VietPhrase),
        "vietphrase-one" => Some(Mode::VietPhraseOneMeaning),
        _ => None,
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("names")
        && args.get(1).map(String::as_str) == Some("filter")
    {
        return run_name_filter(&args[2..]);
    }
    // Expected: translate --mode M [engine/presentation options]
    if args.first().map(String::as_str) != Some("translate") {
        print_usage();
        return ExitCode::from(2);
    }
    let mut mode = Mode::VietPhrase;
    let mut data_dir = PathBuf::from("data");
    let mut dictionary_files = DictionaryFiles::default();
    let mut options = Options::default();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--mode" => {
                i += 1;
                match args.get(i).and_then(|s| parse_mode(s)) {
                    Some(m) => mode = m,
                    None => {
                        eprintln!("error: invalid or missing --mode value");
                        return ExitCode::from(2);
                    }
                }
            }
            "--data-dir" => {
                i += 1;
                match args.get(i) {
                    Some(d) => data_dir = PathBuf::from(d),
                    None => {
                        eprintln!("error: --data-dir needs a path");
                        return ExitCode::from(2);
                    }
                }
            }
            "--wrap" => options.wrap_type = 1,
            "--scan-range" => {
                i += 1;
                let Some(value) = args.get(i).and_then(|value| value.parse::<usize>().ok()) else {
                    eprintln!(
                        "error: --scan-range needs an integer between 1 and {MAX_SCAN_RANGE}"
                    );
                    return ExitCode::from(2);
                };
                if !(1..=MAX_SCAN_RANGE).contains(&value) {
                    eprintln!("error: --scan-range must be between 1 and {MAX_SCAN_RANGE}");
                    return ExitCode::from(2);
                }
                options.scan_range = value;
            }
            "--translation-algorithm" => {
                i += 1;
                let Some(value) = args.get(i).and_then(|value| value.parse::<i32>().ok()) else {
                    eprintln!("error: --translation-algorithm needs 0, 1, or 2");
                    return ExitCode::from(2);
                };
                if !matches!(value, 0..=2) {
                    eprintln!("error: --translation-algorithm must be 0, 1, or 2");
                    return ExitCode::from(2);
                }
                options.translation_algorithm = value;
            }
            "--prioritized-name" => {
                i += 1;
                let Some(value) = args.get(i).and_then(|value| value.parse::<bool>().ok()) else {
                    eprintln!("error: --prioritized-name needs true or false");
                    return ExitCode::from(2);
                };
                options.prioritized_name = value;
            }
            option @ ("--names-file"
            | "--names2-file"
            | "--luat-nhan-file"
            | "--pronouns-file"
            | "--danh-tu-file"
            | "--ho-nguoi-file"
            | "--hau-tu-file"
            | "--ignored-chinese-phrases-file") => {
                i += 1;
                let Some(path) = args.get(i) else {
                    eprintln!("error: {option} needs a path");
                    return ExitCode::from(2);
                };
                let path = PathBuf::from(path);
                match option {
                    "--names-file" => dictionary_files.names = Some(path),
                    "--names2-file" => dictionary_files.names2 = Some(path),
                    "--luat-nhan-file" => dictionary_files.luat_nhan = Some(path),
                    "--pronouns-file" => dictionary_files.pronouns = Some(path),
                    "--danh-tu-file" => dictionary_files.danh_tu = Some(path),
                    "--ho-nguoi-file" => dictionary_files.ho_nguoi = Some(path),
                    "--hau-tu-file" => dictionary_files.hau_tu = Some(path),
                    "--ignored-chinese-phrases-file" => {
                        dictionary_files.ignored_chinese_phrases = Some(path)
                    }
                    _ => unreachable!(),
                }
            }
            other => {
                eprintln!("error: unknown argument {other}");
                return ExitCode::from(2);
            }
        }
        i += 1;
    }

    let dicts = match Dictionaries::load(&data_dir) {
        Ok(d) => d,
        Err(e) => {
            eprintln!(
                "error: failed to load dictionaries from {}: {e}",
                data_dir.display()
            );
            return ExitCode::FAILURE;
        }
    };
    let engine = Engine::from_dicts(dicts);
    let dictionary_overrides = match load_dictionary_overrides(&dictionary_files) {
        Ok(overrides) => overrides,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };
    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        eprintln!("error: failed to read stdin");
        return ExitCode::FAILURE;
    }
    let out = match engine.translate_with_overrides(&input, mode, &options, &dictionary_overrides) {
        Ok(out) => out,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };
    let mut stdout = std::io::stdout();
    if stdout.write_all(out.as_bytes()).is_err() {
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn print_usage() {
    eprintln!(
        "usage:\n  qt translate [--mode <hanviet|vietphrase|vietphrase-one>] \
         [--data-dir DIR] [--wrap] [--scan-range 1..={MAX_SCAN_RANGE}] \
         [--translation-algorithm 0|1|2] [--prioritized-name true|false] \
         [--names-file PATH] [--names2-file PATH] [--luat-nhan-file PATH] \
         [--pronouns-file PATH] [--danh-tu-file PATH] [--ho-nguoi-file PATH] \
         [--hau-tu-file PATH] [--ignored-chinese-phrases-file PATH]\n  \
         qt names filter [--mode qt|hybrid] [--data-dir DIR] \
         [--min-occurrences N] [--min-confidence 0..1] [--max-candidates N] \
         [--known-names-file PATH] [--rejected-names-file PATH] [--json] \
         [dictionary file options]"
    );
}

fn run_name_filter(args: &[String]) -> ExitCode {
    let mut data_dir = PathBuf::from("data");
    let mut dictionary_files = DictionaryFiles::default();
    let mut options = NameFilterOptions::default();
    let mut known_names_file = None;
    let mut rejected_names_file = None;
    let mut json_output = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--mode" => {
                i += 1;
                options.mode = match args.get(i).map(String::as_str) {
                    Some("qt") => NameFilterMode::QtCompatible,
                    Some("hybrid") => NameFilterMode::Hybrid,
                    _ => {
                        eprintln!("error: --mode needs qt or hybrid");
                        return ExitCode::from(2);
                    }
                };
            }
            "--data-dir" => {
                i += 1;
                let Some(path) = args.get(i) else {
                    eprintln!("error: --data-dir needs a path");
                    return ExitCode::from(2);
                };
                data_dir = PathBuf::from(path);
            }
            "--min-occurrences" => {
                i += 1;
                let Some(value) = args.get(i).and_then(|value| value.parse::<usize>().ok()) else {
                    eprintln!("error: --min-occurrences needs a positive integer");
                    return ExitCode::from(2);
                };
                if value == 0 {
                    eprintln!("error: --min-occurrences must be at least 1");
                    return ExitCode::from(2);
                }
                options.min_occurrences = value.min(100);
            }
            "--min-confidence" => {
                i += 1;
                let Some(value) = args.get(i).and_then(|value| value.parse::<f32>().ok()) else {
                    eprintln!("error: --min-confidence needs a number between 0 and 1");
                    return ExitCode::from(2);
                };
                if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                    eprintln!("error: --min-confidence must be between 0 and 1");
                    return ExitCode::from(2);
                }
                options.min_score = value;
            }
            "--max-candidates" => {
                i += 1;
                let Some(value) = args.get(i).and_then(|value| value.parse::<usize>().ok()) else {
                    eprintln!("error: --max-candidates needs a positive integer");
                    return ExitCode::from(2);
                };
                options.max_candidates = value.clamp(1, 1_000);
            }
            "--known-names-file" => {
                i += 1;
                let Some(path) = args.get(i) else {
                    eprintln!("error: --known-names-file needs a path");
                    return ExitCode::from(2);
                };
                known_names_file = Some(PathBuf::from(path));
            }
            "--rejected-names-file" => {
                i += 1;
                let Some(path) = args.get(i) else {
                    eprintln!("error: --rejected-names-file needs a path");
                    return ExitCode::from(2);
                };
                rejected_names_file = Some(PathBuf::from(path));
            }
            "--json" => json_output = true,
            option @ ("--names-file"
            | "--names2-file"
            | "--luat-nhan-file"
            | "--pronouns-file"
            | "--danh-tu-file"
            | "--ho-nguoi-file"
            | "--hau-tu-file"
            | "--ignored-chinese-phrases-file") => {
                i += 1;
                let Some(path) = args.get(i) else {
                    eprintln!("error: {option} needs a path");
                    return ExitCode::from(2);
                };
                let path = PathBuf::from(path);
                match option {
                    "--names-file" => dictionary_files.names = Some(path),
                    "--names2-file" => dictionary_files.names2 = Some(path),
                    "--luat-nhan-file" => dictionary_files.luat_nhan = Some(path),
                    "--pronouns-file" => dictionary_files.pronouns = Some(path),
                    "--danh-tu-file" => dictionary_files.danh_tu = Some(path),
                    "--ho-nguoi-file" => dictionary_files.ho_nguoi = Some(path),
                    "--hau-tu-file" => dictionary_files.hau_tu = Some(path),
                    "--ignored-chinese-phrases-file" => {
                        dictionary_files.ignored_chinese_phrases = Some(path)
                    }
                    _ => unreachable!(),
                }
            }
            other => {
                eprintln!("error: unknown argument {other}");
                return ExitCode::from(2);
            }
        }
        i += 1;
    }

    let dictionaries = match Dictionaries::load(&data_dir) {
        Ok(dictionaries) => dictionaries,
        Err(error) => {
            eprintln!(
                "error: failed to load dictionaries from {}: {error}",
                data_dir.display()
            );
            return ExitCode::FAILURE;
        }
    };
    let overrides = match load_dictionary_overrides(&dictionary_files) {
        Ok(overrides) => overrides,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };
    let memory = match load_name_memory(&known_names_file, &rejected_names_file) {
        Ok(memory) => memory,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };
    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        eprintln!("error: failed to read stdin");
        return ExitCode::FAILURE;
    }
    let result =
        Engine::from_dicts(dictionaries).filter_names(&input, &options, &memory, Some(&overrides));
    let output = if json_output {
        let candidates: Vec<_> = result
            .candidates
            .iter()
            .map(|candidate| {
                serde_json::json!({
                    "text": candidate.text,
                    "suggested": candidate.suggested,
                    "entityType": cli_entity_type(candidate.entity_type),
                    "score": candidate.score,
                    "occurrences": candidate.occurrences,
                    "known": candidate.known,
                    "sources": candidate.sources.iter().map(|source| cli_source(*source)).collect::<Vec<_>>()
                })
            })
            .collect();
        serde_json::to_string_pretty(&serde_json::json!({
            "candidates": candidates,
            "scannedCharacters": result.scanned_characters
        }))
        .expect("JSON serialization cannot fail")
    } else {
        result
            .candidates
            .iter()
            .map(|candidate| format!("{}={}", candidate.text, candidate.suggested))
            .collect::<Vec<_>>()
            .join("\n")
    };
    if std::io::stdout().write_all(output.as_bytes()).is_err() {
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn load_name_memory(
    known_names_file: &Option<PathBuf>,
    rejected_names_file: &Option<PathBuf>,
) -> Result<NameFilterMemory, String> {
    let known_names = read_optional_file(known_names_file)?
        .map(|content| parse_dict(&content).into_iter().collect())
        .unwrap_or_default();
    let rejected_names = read_optional_file(rejected_names_file)?
        .map(|content| {
            content
                .lines()
                .map(|line| line.trim().trim_start_matches('\u{feff}'))
                .filter(|line| !line.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    Ok(NameFilterMemory {
        known_names,
        rejected_names,
    })
}

fn cli_entity_type(value: NameEntityType) -> &'static str {
    match value {
        NameEntityType::Person => "person",
        NameEntityType::Location => "location",
        NameEntityType::Organization => "organization",
        NameEntityType::Title => "title",
        NameEntityType::Unknown => "unknown",
    }
}

fn cli_source(value: NameCandidateSource) -> &'static str {
    match value {
        NameCandidateSource::QtJieba => "qt-jieba",
        NameCandidateSource::Ngram => "ngram",
        NameCandidateSource::ContextRule => "context-rule",
        NameCandidateSource::SurnameRule => "surname-rule",
        NameCandidateSource::SuffixRule => "suffix-rule",
        NameCandidateSource::BookMemory => "book-memory",
        NameCandidateSource::BookTitle => "book-title",
        NameCandidateSource::OnnxNer => "onnx-ner",
        NameCandidateSource::AiFallback => "ai-fallback",
    }
}

fn read_optional_file(path: &Option<PathBuf>) -> Result<Option<String>, String> {
    path.as_ref()
        .map(|path| {
            std::fs::read_to_string(path)
                .map_err(|error| format!("failed to read {}: {error}", path.display()))
        })
        .transpose()
}

fn load_dictionary_overrides(files: &DictionaryFiles) -> Result<DictionaryOverrides, String> {
    let names = read_optional_file(&files.names)?;
    let names2 = read_optional_file(&files.names2)?;
    let luat_nhan = read_optional_file(&files.luat_nhan)?;
    let pronouns = read_optional_file(&files.pronouns)?;
    let danh_tu = read_optional_file(&files.danh_tu)?;
    let ho_nguoi = read_optional_file(&files.ho_nguoi)?;
    let hau_tu = read_optional_file(&files.hau_tu)?;
    let ignored_chinese_phrases = read_optional_file(&files.ignored_chinese_phrases)?;

    Ok(DictionaryOverrides::from_sources(
        DictionarySourceOverrides {
            names: names.as_deref(),
            names2: names2.as_deref(),
            luat_nhan: luat_nhan.as_deref(),
            pronouns: pronouns.as_deref(),
            danh_tu: danh_tu.as_deref(),
            ho_nguoi: ho_nguoi.as_deref(),
            hau_tu: hau_tu.as_deref(),
            ignored_chinese_phrases: ignored_chinese_phrases.as_deref(),
        },
    ))
}
