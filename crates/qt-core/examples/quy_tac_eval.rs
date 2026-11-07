//! Cổng bước 1 của spec Quy Tắc (mục 12): chất lượng, tốc độ, RAM trên bộ test bookshelf.
//!
//! cargo run --release -p qt-core --example quy_tac_eval -- <QT2025> <books/common> [chương-mỗi-truyện=12] [lệch=7]
//!
//! Bộ test giống bản thử: mỗi truyện lấy rải đều 12 chương có bản biên tập
//! (`out/` cùng tên với `raw/`, cùng số đoạn), bắt đầu từ chương thứ 8. Mỗi
//! đoạn dịch hai lần: convert hiện tại (có Luật Nhân) và Quy Tắc. Chấm bằng số
//! cụm 2 và 3 từ trùng với bản biên tập (chữ thường, NFC).

use std::alloc::{GlobalAlloc, Layout, System};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::time::{Duration, Instant};

use qt_core::{
    Dictionaries, DictionaryOverrides, DictionarySourceOverrides, Engine, Mode, Options, QuyTac,
};
use regex::Regex;
use unicode_normalization::UnicodeNormalization;

struct Counting;
static LIVE: AtomicIsize = AtomicIsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        LIVE.fetch_add(layout.size() as isize, Ordering::Relaxed);
        System.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size() as isize, Ordering::Relaxed);
        System.dealloc(ptr, layout)
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        LIVE.fetch_add(new_size as isize - layout.size() as isize, Ordering::Relaxed);
        System.realloc(ptr, layout, new_size)
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

fn words(text: &str, word: &Regex) -> Vec<String> {
    let lowered = text.nfc().collect::<String>().to_lowercase();
    word.find_iter(&lowered).map(|m| m.as_str().to_string()).collect()
}

fn grams(words: &[String], n: usize) -> HashMap<String, usize> {
    let mut out = HashMap::new();
    for window in words.windows(n) {
        *out.entry(window.join("\u{1}")).or_insert(0) += 1;
    }
    out
}

fn overlap(text: &HashMap<String, usize>, reference: &HashMap<String, usize>) -> u64 {
    text.iter()
        .map(|(gram, count)| (*count).min(*reference.get(gram).unwrap_or(&0)) as u64)
        .sum()
}

fn non_blank_lines(text: &str) -> Vec<&str> {
    text.lines().map(str::trim).filter(|line| !line.is_empty()).collect()
}

fn story_names(book: &Path) -> String {
    let Ok(text) = std::fs::read_to_string(book.join("story.json")) else {
        return String::new();
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return String::new();
    };
    value
        .get("glossary")
        .and_then(|g| g.get("names"))
        .and_then(|n| n.as_object())
        .map(|names| {
            names
                .iter()
                .map(|(key, value)| format!("{key}={}", value.as_str().unwrap_or("")))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        eprintln!("usage: quy_tac_eval <QT2025> <books/common> [chương-mỗi-truyện=12] [lệch=7]");
        return ExitCode::from(2);
    }
    let data = PathBuf::from(&args[0]);
    let books = PathBuf::from(&args[1]);
    let per_book: usize = args.get(2).and_then(|v| v.parse().ok()).unwrap_or(12);
    let offset: usize = args.get(3).and_then(|v| v.parse().ok()).unwrap_or(7);

    let engine = Engine::from_dicts(Dictionaries::load(&data).expect("nạp QT2025"));
    let before = LIVE.load(Ordering::Relaxed);
    let quy_tac = QuyTac::load(&data, false).expect("nạp Quy Tắc");
    let quy_tac_bytes = LIVE.load(Ordering::Relaxed) - before;
    let engine = engine.with_quy_tac(quy_tac);
    let word = Regex::new(r"\w+").unwrap();
    let off = Options::default();
    let on = Options {
        quy_tac: true,
        ..Options::default()
    };
    let mode = Mode::VietPhraseOneMeaning;

    let mut dirs: Vec<PathBuf> = std::fs::read_dir(&books)
        .expect("đọc thư mục truyện")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.join("raw").is_dir() && path.join("out").is_dir())
        .collect();
    dirs.sort();

    let (mut cur2, mut cur3, mut new2, mut new3) = (0u64, 0u64, 0u64, 0u64);
    let (mut paragraphs, mut changed, mut better, mut worse) = (0usize, 0usize, 0usize, 0usize);
    let (mut cur_time, mut new_time) = (Duration::ZERO, Duration::ZERO);
    let mut chars = 0usize;
    // QT_EVAL_DUMP=<file>: mỗi đoạn một dòng `{hiện tại}\t{Quy Tắc}`, theo thứ tự chấm.
    let mut dump = std::env::var_os("QT_EVAL_DUMP").map(|path| {
        std::io::BufWriter::new(std::fs::File::create(path).expect("tạo file dump"))
    });
    for book in &dirs {
        let names = story_names(book);
        let overrides = if names.is_empty() {
            DictionaryOverrides::default()
        } else {
            DictionaryOverrides::from_sources(DictionarySourceOverrides {
                names: Some(&names),
                ..Default::default()
            })
        };
        let mut files: Vec<PathBuf> = std::fs::read_dir(book.join("raw"))
            .unwrap()
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| {
                path.extension().is_some_and(|ext| ext == "txt")
                    && book.join("out").join(path.file_name().unwrap()).is_file()
            })
            .collect();
        files.sort();
        if files.is_empty() {
            continue;
        }
        let step = (files.len() / per_book).max(1);
        for file in files.iter().skip(offset.min(files.len() - 1)).step_by(step).take(per_book) {
            let (Ok(raw), Ok(reference)) = (
                std::fs::read_to_string(file),
                std::fs::read_to_string(book.join("out").join(file.file_name().unwrap())),
            ) else {
                continue;
            };
            let (raw, reference) = (non_blank_lines(&raw), non_blank_lines(&reference));
            if raw.len() != reference.len() {
                continue;
            }
            for (source, edited) in raw.iter().zip(&reference) {
                chars += source.chars().count();
                let started = Instant::now();
                let current = engine
                    .translate_with_overrides(source, mode, &off, &overrides)
                    .unwrap();
                cur_time += started.elapsed();
                let started = Instant::now();
                let next = engine
                    .translate_with_overrides(source, mode, &on, &overrides)
                    .unwrap();
                new_time += started.elapsed();
                if let Some(dump) = dump.as_mut() {
                    writeln!(dump, "{current}\t{next}").expect("ghi dump");
                }
                let reference_words = words(edited, &word);
                let (ref2, ref3) = (grams(&reference_words, 2), grams(&reference_words, 3));
                let current_words = words(&current, &word);
                let next_words = words(&next, &word);
                let (c2, c3) = (
                    overlap(&grams(&current_words, 2), &ref2),
                    overlap(&grams(&current_words, 3), &ref3),
                );
                let (n2, n3) = (
                    overlap(&grams(&next_words, 2), &ref2),
                    overlap(&grams(&next_words, 3), &ref3),
                );
                cur2 += c2;
                cur3 += c3;
                new2 += n2;
                new3 += n3;
                paragraphs += 1;
                if next != current {
                    changed += 1;
                    if n2 + n3 > c2 + c3 {
                        better += 1;
                    } else if n2 + n3 < c2 + c3 {
                        worse += 1;
                    }
                }
            }
        }
    }

    if let Some(mut dump) = dump {
        dump.flush().expect("ghi dump");
    }
    let gain = |new: u64, cur: u64| (new as f64 - cur as f64) * 100.0 / cur.max(1) as f64;
    let win_rate = better as f64 * 100.0 / (better + worse).max(1) as f64;
    let speed = new_time.as_secs_f64() / cur_time.as_secs_f64().max(1e-9);
    let ns = |time: Duration| time.as_secs_f64() * 1e9 / chars.max(1) as f64;
    let mib = quy_tac_bytes as f64 / (1024.0 * 1024.0);
    println!(
        "{paragraphs} đoạn, {chars} ký tự; đổi {changed} ({:.1}%); tốt hơn {better}, kém hơn {worse}; thắng {win_rate:.1}%",
        changed as f64 * 100.0 / paragraphs.max(1) as f64
    );
    println!("cụm 2 từ: hiện tại {cur2}, Quy Tắc {new2} ({:+.2}%)", gain(new2, cur2));
    println!("cụm 3 từ: hiện tại {cur3}, Quy Tắc {new3} ({:+.2}%)", gain(new3, cur3));
    println!(
        "thời gian: hiện tại {:.0} ns/ký tự, Quy Tắc {:.0} ns/ký tự ({:.0}%)",
        ns(cur_time),
        ns(new_time),
        speed * 100.0
    );
    println!("RAM giữ của QuyTac: {mib:.1} MiB");
    let gates = [
        ("chất lượng: cụm 3 từ >= +10,0%", gain(new3, cur3) >= 10.0),
        ("chất lượng: thắng >= 80%", win_rate >= 80.0),
        ("tốc độ: <= 100% hiện tại", speed <= 1.00),
        ("RAM: <= 10 MiB", mib <= 10.0),
    ];
    let mut ok = true;
    for (name, pass) in gates {
        println!("{} {name}", if pass { "ĐẠT " } else { "HỎNG" });
        ok &= pass;
    }
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
