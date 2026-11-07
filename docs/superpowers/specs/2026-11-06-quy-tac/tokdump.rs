//! Xuất token (zh, vi) của engine theo từng đoạn, kèm đoạn đã biên tập tương ứng.
//! Một dòng JSON mỗi đoạn: {"b":book,"tok":[[zh,vi],...],"ref":"..."}.
use qt_core as qt;
use serde_json::json;
use std::io::Write;

fn lines(text: &str) -> Vec<(usize, usize)> {
    // (start, end) theo chỉ số ký tự, chỉ giữ dòng có chữ.
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut pos = 0usize;
    let mut blank = true;
    for ch in text.chars() {
        if ch == '\n' {
            if !blank {
                out.push((start, pos));
            }
            start = pos + 1;
            blank = true;
        } else if !ch.is_whitespace() {
            blank = false;
        }
        pos += ch.len_utf16();
    }
    if !blank {
        out.push((start, pos));
    }
    out
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dir = std::path::PathBuf::from(&args[1]);
    let books = std::path::PathBuf::from(&args[2]);
    let per_book: usize = args[3].parse().unwrap();
    let mut out = std::io::BufWriter::new(std::fs::File::create(&args[4]).unwrap());
    let offset: usize = args[5].parse().unwrap();

    let engine = qt::Engine::from_dicts(qt::Dictionaries::load(&dir).unwrap());
    let opts = qt::Options::default();
    let mut dirs: Vec<_> = std::fs::read_dir(&books).unwrap().flatten().map(|e| e.path())
        .filter(|p| p.join("raw").is_dir() && p.join("out").is_dir()).collect();
    dirs.sort();
    let (mut chapters, mut paragraphs, mut skipped) = (0usize, 0usize, 0usize);
    for d in &dirs {
        let book = d.file_name().unwrap().to_string_lossy().into_owned();
        let names_src = std::fs::read_to_string(d.join("story.json")).ok()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            .and_then(|v| v.get("glossary").and_then(|g| g.get("names")).cloned())
            .and_then(|n| n.as_object().cloned())
            .map(|m| m.iter().map(|(k, v)| format!("{}={}", k, v.as_str().unwrap_or(""))).collect::<Vec<_>>().join("\n"))
            .unwrap_or_default();
        let src = qt::DictionarySourceOverrides { names: (!names_src.is_empty()).then_some(names_src.as_str()), luat_nhan: std::env::var("QT_KEEP_LN").is_ok().then_some(()).map_or(Some(""), |_| None), ..Default::default() };
        let overrides = qt::DictionaryOverrides::from_sources(src);
        let mut files: Vec<_> = std::fs::read_dir(d.join("raw")).unwrap().flatten().map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "txt") && d.join("out").join(p.file_name().unwrap()).is_file()).collect();
        files.sort();
        if files.is_empty() { continue; }
        let step = (files.len() / per_book).max(1);
        for f in files.iter().skip(offset.min(files.len() - 1)).step_by(step).take(per_book) {
            let Ok(text) = std::fs::read_to_string(f) else { continue };
            let Ok(reference) = std::fs::read_to_string(d.join("out").join(f.file_name().unwrap())) else { continue };
            let ref_lines: Vec<&str> = reference.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
            let spans = lines(&text);
            if spans.len() != ref_lines.len() { skipped += 1; continue; }
            let result = engine
                .translate_with_ranges_and_overrides(&text, qt::Mode::VietPhraseOneMeaning, &opts, &overrides)
                .unwrap();
            let src_chars: Vec<u16> = text.encode_utf16().collect();
            let tgt_chars: Vec<u16> = result.translated_text.encode_utf16().collect();
            let mut pairs: Vec<(usize, usize, usize, usize)> = result.source_ranges.iter().zip(&result.target_ranges)
                .map(|(s, t)| (s.start, s.length, t.start, t.length)).collect();
            pairs.sort();
            let mut cursor = 0usize;
            for (line, (start, end)) in spans.iter().enumerate() {
                let mut tokens: Vec<(String, String)> = Vec::new();
                let mut pos = *start;
                while cursor < pairs.len() && pairs[cursor].0 < *end {
                    let (s, l, ts, tl) = pairs[cursor];
                    cursor += 1;
                    if s < pos { continue; }
                    if s > pos {
                        tokens.push((String::from_utf16_lossy(&src_chars[pos..s]), String::new()));
                    }
                    let stop = (s + l).min(src_chars.len());
                    let tstop = (ts + tl).min(tgt_chars.len());
                    tokens.push((String::from_utf16_lossy(&src_chars[s..stop]), String::from_utf16_lossy(&tgt_chars[ts..tstop])));
                    pos = stop;
                }
                if pos < *end {
                    tokens.push((String::from_utf16_lossy(&src_chars[pos..*end]), String::new()));
                }
                writeln!(out, "{}", json!({"b": book, "tok": tokens, "ref": ref_lines[line]})).unwrap();
                paragraphs += 1;
            }
            chapters += 1;
        }
    }
    println!("{chapters} chapters, {paragraphs} paragraphs, skipped {skipped}");
}
