//! Giải mã trang nguồn. Trang Trung hay khai sai hoặc không khai charset, nên thứ tự đoán là: label extension
//! ép → charset trong header → `<meta charset>` trong 2 KB đầu → BOM → chardetng theo nội dung.

use crate::ExtError;
use encoding_rs::Encoding;
use regex::Regex;
use std::sync::OnceLock;

fn encoding_for(label: &str) -> Result<&'static Encoding, ExtError> {
    Encoding::for_label(label.trim().as_bytes()).ok_or_else(|| ExtError::Script(format!("không biết bảng mã \"{label}\"")))
}

/// `text/html; charset=GBK` → `gbk`.
pub fn charset_from_content_type(content_type: &str) -> Option<String> {
    content_type.split(';').skip(1).find_map(|part| {
        let (key, value) = part.split_once('=')?;
        (key.trim().eq_ignore_ascii_case("charset")).then(|| value.trim().trim_matches(['"', '\'']).to_ascii_lowercase())
    })
}

fn meta_charset(bytes: &[u8]) -> Option<&'static Encoding> {
    static META: OnceLock<Regex> = OnceLock::new();
    let regex = META.get_or_init(|| Regex::new(r#"(?i)<meta[^>]+charset\s*=\s*["']?([\w-]+)"#).expect("regex meta"));
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(2048)]);
    regex.captures(&head).and_then(|c| Encoding::for_label(c[1].as_bytes()))
}

fn sniff(bytes: &[u8]) -> &'static Encoding {
    if let Some((encoding, _)) = Encoding::for_bom(bytes) {
        return encoding;
    }
    if std::str::from_utf8(bytes).is_ok() {
        return encoding_rs::UTF_8;
    }
    let mut detector = chardetng::EncodingDetector::new(chardetng::Iso2022JpDetection::Deny);
    detector.feed(bytes, true);
    detector.guess(None, chardetng::Utf8Detection::Allow)
}

/// `label`: `None` hoặc `"auto"` thì đoán; khác thì ép theo label đó.
pub fn decode(bytes: &[u8], label: Option<&str>, header_charset: Option<&str>) -> Result<String, ExtError> {
    let encoding = match label.map(str::trim).filter(|l| !l.is_empty() && !l.eq_ignore_ascii_case("auto")) {
        Some(label) => encoding_for(label)?,
        None => header_charset
            .and_then(|c| Encoding::for_label(c.trim().as_bytes()))
            .or_else(|| meta_charset(bytes))
            .unwrap_or_else(|| sniff(bytes)),
    };
    // decode() tự bỏ BOM khớp với bảng mã.
    let (text, _, _) = encoding.decode(bytes);
    Ok(text.into_owned())
}

pub fn encode(text: &str, label: &str) -> Result<Vec<u8>, ExtError> {
    let encoding = encoding_for(label)?;
    // encoding_rs không mã hoá ra UTF-16 (output_encoding trả UTF-8) — tự làm cho đủ.
    if encoding == encoding_rs::UTF_16LE {
        return Ok(text.encode_utf16().flat_map(u16::to_le_bytes).collect());
    }
    if encoding == encoding_rs::UTF_16BE {
        return Ok(text.encode_utf16().flat_map(u16::to_be_bytes).collect());
    }
    let (bytes, _, _) = encoding.encode(text);
    Ok(bytes.into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doan_ma_hoa_theo_thu_tu() {
        let (gbk, _, _) = encoding_rs::GBK.encode("第一章 天下");
        assert_eq!(decode(&gbk, None, Some("gbk")).unwrap(), "第一章 天下");
        let page = [b"<html><head><meta charset=\"gbk\"></head><body>".as_slice(), &gbk, b"</body>"].concat();
        assert!(decode(&page, None, None).unwrap().contains("第一章 天下"));
        let page = [b"<meta http-equiv=\"Content-Type\" content=\"text/html; charset=gb2312\">".as_slice(), &gbk].concat();
        assert!(decode(&page, None, None).unwrap().contains("第一章 天下"));
        assert_eq!(decode("天下".as_bytes(), None, None).unwrap(), "天下");
        assert_eq!(decode(&gbk, Some("gb18030"), Some("utf-8")).unwrap(), "第一章 天下", "label ép thắng header");
        let long = "他走进了房间，看见桌上放着一封信。".repeat(20);
        let (g, _, _) = encoding_rs::GBK.encode(&long);
        assert_eq!(decode(&g, Some("auto"), None).unwrap(), long, "chardetng");
        let trad = "他走進了房間，看見桌上放著一封信。".repeat(20);
        let (big5, _, _) = encoding_rs::BIG5.encode(&trad);
        assert!(decode(&big5, None, None).unwrap().starts_with("他走進了房間"));
        assert!(decode(b"x", Some("klingon"), None).is_err());
        let bom = [&[0xef, 0xbb, 0xbf][..], "天".as_bytes()].concat();
        assert_eq!(decode(&bom, None, None).unwrap(), "天");
    }

    #[test]
    fn encode_va_content_type() {
        assert_eq!(encode("天", "gbk").unwrap(), vec![0xcc, 0xec]);
        assert_eq!(encode("天", "utf-16le").unwrap(), vec![0x29, 0x59]);
        assert_eq!(charset_from_content_type("text/html; charset=GBK").as_deref(), Some("gbk"));
        assert_eq!(charset_from_content_type("text/html; Charset=\"utf-8\"").as_deref(), Some("utf-8"));
        assert_eq!(charset_from_content_type("text/html"), None);
    }
}
