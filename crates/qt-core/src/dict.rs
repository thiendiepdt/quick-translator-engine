//! Dictionary loading, merging, and QT's priority rules.

/// Parse `key=value` lines the way TranslatorEngine does:
/// - a leading BOM on the first line is stripped
/// - split on '=' into ALL parts; keep the line only if exactly 2 parts result
///   (so a value containing '=' drops the line, matching QT)
/// - lines starting with '#' are comments (skipped)
pub fn parse_dict(content: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (i, raw) in content.lines().enumerate() {
        let line = if i == 0 { raw.trim_start_matches('\u{feff}') } else { raw };
        if line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() == 2 {
            out.push((parts[0].to_string(), parts[1].to_string()));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bom_and_skips_bad_lines() {
        let content = "\u{feff}一=nhất\n# comment\n二=nhị\na=b=c\nnoequals\n";
        let got = parse_dict(content);
        assert_eq!(got, vec![
            ("一".to_string(), "nhất".to_string()),
            ("二".to_string(), "nhị".to_string()),
        ]);
    }
}
