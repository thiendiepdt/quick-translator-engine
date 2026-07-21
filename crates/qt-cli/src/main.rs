use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use qt_core::{Dictionaries, Engine, Mode, Options};

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
    // Expected: translate --mode M [--data-dir D] [--wrap]
    if args.first().map(String::as_str) != Some("translate") {
        eprintln!("usage: qt translate --mode <hanviet|vietphrase|vietphrase-one> [--data-dir DIR] [--wrap]");
        return ExitCode::from(2);
    }
    let mut mode = Mode::VietPhrase;
    let mut data_dir = PathBuf::from("data");
    let mut wrap = false;
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
            "--wrap" => wrap = true,
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
            eprintln!("error: failed to load dictionaries from {}: {e}", data_dir.display());
            return ExitCode::FAILURE;
        }
    };
    let engine = Engine::from_dicts(dicts);
    let opts = Options { wrap_type: if wrap { 1 } else { 0 }, ..Options::default() };

    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        eprintln!("error: failed to read stdin");
        return ExitCode::FAILURE;
    }
    let out = engine.translate(&input, mode, &opts);
    let mut stdout = std::io::stdout();
    if stdout.write_all(out.as_bytes()).is_err() {
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
