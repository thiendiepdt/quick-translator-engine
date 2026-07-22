use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use qt_core::{Dictionaries, Engine, Mode, Options};

const MAX_SCAN_RANGE: usize = 100;

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
    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        eprintln!("error: failed to read stdin");
        return ExitCode::FAILURE;
    }
    let out = engine.translate(&input, mode, &options);
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
         [--translation-algorithm 0|1|2] [--prioritized-name true|false]"
    );
}
