use std::collections::HashSet;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

use qt_core::{
    parse_dict, Dictionaries, Engine, NameFilterMemory, NameFilterMode, NameFilterOptions,
};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 3 {
        eprintln!(
            "usage: cargo run --release -p qt-core --example name_filter_bench -- \
             <data-dir> <chapter.txt> <gold-names.txt> [iterations] [qt|hybrid]"
        );
        return ExitCode::from(2);
    }
    let iterations = args
        .get(3)
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(20)
        .max(1);
    let mode = match args.get(4).map(String::as_str).unwrap_or("hybrid") {
        "qt" => NameFilterMode::QtCompatible,
        "hybrid" => NameFilterMode::Hybrid,
        other => {
            eprintln!("error: unsupported mode {other}");
            return ExitCode::from(2);
        }
    };
    let dictionaries = match Dictionaries::load(Path::new(&args[0])) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("error: failed to load dictionaries: {error}");
            return ExitCode::FAILURE;
        }
    };
    let chapter = match std::fs::read_to_string(&args[1]) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("error: failed to read chapter: {error}");
            return ExitCode::FAILURE;
        }
    };
    let gold = match std::fs::read_to_string(&args[2]) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("error: failed to read gold names: {error}");
            return ExitCode::FAILURE;
        }
    };
    let gold: HashSet<String> = parse_dict(&gold).into_iter().map(|(key, _)| key).collect();
    let engine = Engine::from_dicts(dictionaries);
    let options = NameFilterOptions {
        mode,
        ..Default::default()
    };
    let started = Instant::now();
    let mut result = None;
    for _ in 0..iterations {
        result = Some(engine.filter_names(&chapter, &options, &NameFilterMemory::default(), None));
    }
    let elapsed = started.elapsed();
    let result = result.expect("at least one iteration");
    let predicted: HashSet<_> = result
        .candidates
        .iter()
        .map(|candidate| candidate.text.clone())
        .collect();
    let true_positive = predicted.intersection(&gold).count();
    let precision = ratio(true_positive, predicted.len());
    let recall = ratio(true_positive, gold.len());
    let f1 = if precision + recall > 0.0 {
        2.0 * precision * recall / (precision + recall)
    } else {
        0.0
    };
    let average_ms = elapsed.as_secs_f64() * 1_000.0 / iterations as f64;
    println!("mode={mode:?}");
    println!("chapter_chars={}", chapter.chars().count());
    println!("iterations={iterations}");
    println!("average_ms={average_ms:.3}");
    println!("predicted={}", predicted.len());
    println!("gold={}", gold.len());
    println!("precision={precision:.4}");
    println!("recall={recall:.4}");
    println!("f1={f1:.4}");
    ExitCode::SUCCESS
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}
