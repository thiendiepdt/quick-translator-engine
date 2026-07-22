use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use qt_core::{
    Dictionaries, DictionaryOverrides, DictionarySourceOverrides, Engine, Mode, Options,
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
        "usage: qt translate [--mode <hanviet|vietphrase|vietphrase-one>] \
         [--data-dir DIR] [--wrap] [--scan-range 1..={MAX_SCAN_RANGE}] \
         [--translation-algorithm 0|1|2] [--prioritized-name true|false] \
         [--names-file PATH] [--names2-file PATH] [--luat-nhan-file PATH] \
         [--pronouns-file PATH] [--danh-tu-file PATH] [--ho-nguoi-file PATH] \
         [--hau-tu-file PATH] [--ignored-chinese-phrases-file PATH]"
    );
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
