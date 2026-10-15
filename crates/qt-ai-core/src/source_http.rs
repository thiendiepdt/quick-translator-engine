//! Tải thông tin sách (tác giả, ảnh bìa) từ trang nguồn cho export EPUB — cách gọi chép từ novelkit
//! (`src-tauri/src/novel_source.rs`). Text chỉ tải từ danh sách host dưới đây; ảnh bìa từ host https công
//! khai bất kỳ (tác giả JJWXC dán ảnh từ khắp nơi), kiểm lại từng bước redirect. Phần bóc dữ liệu nằm ở
//! frontend (DOMParser). Không phải proxy chung.

use reqwest::blocking::Client;
use reqwest::Url;
use serde::Serialize;
use std::sync::OnceLock;
use std::time::Duration;

const DESKTOP_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";
const MOBILE_UA: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 16_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1";

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_IMAGE_BYTES: usize = 10 * 1024 * 1024;
// CDN ảnh bìa (byteimg nhất là) chập chờn khi gọi từ ngoài Trung Quốc: lần đầu có khi 10 giây hoặc rớt, lần
// sau trả trong chưa tới một giây.
const IMAGE_TIMEOUT: Duration = Duration::from_secs(45);
const IMAGE_ATTEMPTS: usize = 2;
const IMAGE_RETRY_DELAY: Duration = Duration::from_millis(600);

/// Host trang thông tin sách được phép tải, và cách tự giới thiệu với host đó.
struct TextSource {
    host: &'static str,
    user_agent: &'static str,
    referer: Option<&'static str>,
}

const TEXT_SOURCES: [TextSource; 6] = [
    // Trang sách có `window.__INITIAL_STATE__`.
    TextSource { host: "fanqienovel.com", user_agent: DESKTOP_UA, referer: Some("https://fanqienovel.com/") },
    // Bản mobile: www.qidian.com trả HTTP 202 kèm trang dò bot thay vì trang sách.
    TextSource { host: "m.qidian.com", user_agent: MOBILE_UA, referer: None },
    // API chi tiết sách QQ đọc sách, công khai, không chữ ký.
    TextSource { host: "detailadr.reader.qq.com", user_agent: MOBILE_UA, referer: None },
    TextSource { host: "www.qimao.com", user_agent: DESKTOP_UA, referer: None },
    TextSource { host: "m.sfacg.com", user_agent: MOBILE_UA, referer: None },
    // API app: JSON UTF-8 (trang web là GB18030 không khai charset).
    TextSource { host: "app.jjwxc.net", user_agent: MOBILE_UA, referer: None },
];

/// Tên miền chỉ phân giải được trong mạng nội bộ.
const LOCAL_DOMAIN_SUFFIXES: [&str; 7] = [".localhost", ".local", ".internal", ".intranet", ".lan", ".home", ".corp"];

#[derive(Debug, Clone, Serialize)]
pub struct SourceImage {
    pub bytes: Vec<u8>,
    pub mime: String,
}

fn is_public_domain(domain: &str) -> bool {
    let domain = domain.trim_end_matches('.').to_ascii_lowercase();
    domain.contains('.') && !LOCAL_DOMAIN_SUFFIXES.iter().any(|suffix| domain.ends_with(suffix))
}

/// https + tên miền công khai (không IP, không localhost, không tên nội bộ) — áp cả cho từng bước redirect,
/// để URL lấy từ trang bên thứ ba không trỏ app vào máy trong mạng nội bộ.
fn check_public_https(url: &Url) -> Result<(), String> {
    if url.scheme() != "https" {
        return Err("Chỉ hỗ trợ URL https".to_string());
    }
    // `domain()` là None khi host là IP (mọi cách viết đã được parser chuẩn hoá).
    match url.domain() {
        Some(domain) if is_public_domain(domain) => Ok(()),
        _ => Err(format!("Máy chủ ảnh không hợp lệ: {}", url.host_str().unwrap_or_default())),
    }
}

fn text_client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        crate::api::install_crypto_provider();
        Client::builder()
            // Qimao gửi header gập dòng kiểu cũ.
            .http1_allow_obsolete_multiline_headers_in_responses(true)
            .timeout(REQUEST_TIMEOUT)
            .build()
            .expect("reqwest client")
    })
}

fn image_client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        crate::api::install_crypto_provider();
        Client::builder()
            .timeout(IMAGE_TIMEOUT)
            .redirect(reqwest::redirect::Policy::custom(|attempt| {
                if attempt.previous().len() >= 5 {
                    attempt.error("quá nhiều redirect")
                } else if check_public_https(attempt.url()).is_ok() {
                    attempt.follow()
                } else {
                    // Trả nguyên 3xx: báo thành tải thất bại.
                    attempt.stop()
                }
            }))
            .build()
            .expect("reqwest client")
    })
}

fn text_source(url: &str) -> Result<(Url, &'static TextSource), String> {
    let parsed = Url::parse(url.trim()).map_err(|e| format!("URL không hợp lệ: {e}"))?;
    if parsed.scheme() != "https" {
        return Err("Chỉ hỗ trợ URL https".to_string());
    }
    let host = parsed.host_str().unwrap_or_default().to_ascii_lowercase();
    let source = TEXT_SOURCES.iter().find(|s| s.host == host).ok_or_else(|| format!("Nguồn không được hỗ trợ: {host}"))?;
    Ok((parsed, source))
}

fn image_url(url: &str) -> Result<Url, String> {
    let parsed = Url::parse(url.trim()).map_err(|e| format!("URL không hợp lệ: {e}"))?;
    check_public_https(&parsed)?;
    Ok(parsed)
}

/// Loại ảnh theo byte đầu, không theo Content-Type (host không do mình chọn).
pub fn sniff_image_mime(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("image/jpeg")
    } else if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        Some("image/png")
    } else if bytes.starts_with(b"GIF8") {
        Some("image/gif")
    } else if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some("image/webp")
    } else {
        None
    }
}

/// Trang / API thông tin sách từ một host trong danh sách.
pub fn fetch_text(url: &str) -> Result<String, String> {
    let (parsed, source) = text_source(url)?;
    let mut request = text_client()
        .get(parsed)
        .header("User-Agent", source.user_agent)
        .header("Accept", "text/html,application/json;q=0.9,*/*;q=0.8")
        .header("Accept-Language", "zh-CN,zh;q=0.9");
    if let Some(referer) = source.referer {
        request = request.header("Referer", referer);
    }
    let response = request.send().map_err(|e| format!("Không tải được dữ liệu từ {}: {e}", source.host))?;
    let status = response.status();
    // 202 là cách các trang này trả trang dò bot: không có dữ liệu sách.
    if !status.is_success() || status.as_u16() == 202 {
        return Err(format!("{} trả về HTTP {status}", source.host));
    }
    response.text().map_err(|e| format!("Không đọc được phản hồi từ {}: {e}", source.host))
}

/// Lỗi tải ảnh: `Transient` đáng thử lại, `Fatal` thì không.
#[derive(Debug, PartialEq)]
enum ImageError {
    Transient(String),
    Fatal(String),
}

/// 5xx và 429 lúc có lúc không trên CDN; mã khác thử lại cũng vậy.
fn status_error(status: reqwest::StatusCode) -> ImageError {
    let message = format!("máy chủ ảnh trả về HTTP {status}");
    if status.is_server_error() || status.as_u16() == 429 {
        ImageError::Transient(message)
    } else {
        ImageError::Fatal(message)
    }
}

fn try_fetch_image(url: Url) -> Result<SourceImage, ImageError> {
    let response = image_client()
        .get(url)
        .header("User-Agent", DESKTOP_UA)
        .send()
        .map_err(|e| ImageError::Transient(e.to_string()))?;
    if !response.status().is_success() {
        return Err(status_error(response.status()));
    }
    // Rớt kết nối giữa chừng cũng tạm thời như không mở được kết nối.
    let bytes = response.bytes().map_err(|e| ImageError::Transient(e.to_string()))?;
    if bytes.len() > MAX_IMAGE_BYTES {
        return Err(ImageError::Fatal("ảnh quá lớn (trên 10MB)".to_string()));
    }
    let mime = sniff_image_mime(&bytes)
        .ok_or_else(|| ImageError::Fatal("dữ liệu tải về không phải ảnh JPEG, PNG, WebP hay GIF".to_string()))?;
    Ok(SourceImage { bytes: bytes.to_vec(), mime: mime.to_string() })
}

/// Tải một ảnh bìa, thử lại lỗi tạm thời (mạng, 5xx, 429).
pub fn fetch_image(url: &str) -> Result<SourceImage, String> {
    let parsed = image_url(url)?;
    let mut last = String::new();
    for attempt in 1..=IMAGE_ATTEMPTS {
        match try_fetch_image(parsed.clone()) {
            Ok(image) => return Ok(image),
            Err(ImageError::Fatal(message)) => return Err(format!("Không tải được ảnh bìa: {message}")),
            Err(ImageError::Transient(message)) => {
                last = message;
                if attempt < IMAGE_ATTEMPTS {
                    std::thread::sleep(IMAGE_RETRY_DELAY);
                }
            }
        }
    }
    Err(format!("Không tải được ảnh bìa sau {IMAGE_ATTEMPTS} lần thử: {last}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_chi_nhan_dung_host_trong_danh_sach() {
        assert_eq!(text_source("https://fanqienovel.com/page/7069948840148732967").unwrap().1.host, "fanqienovel.com");
        assert!(text_source("https://m.qidian.com/book/1010868264/").is_ok());
        assert!(text_source("https://detailadr.reader.qq.com/book/queryDetailPage?bid=59366432").is_ok());
        assert!(text_source("https://www.qimao.com/shuku/10021482/").is_ok());
        assert!(text_source("https://m.sfacg.com/b/759334/").is_ok());
        assert!(text_source("https://app.jjwxc.net/androidapi/novelbasicinfo?novelId=4468196").is_ok());
        assert!(text_source("https://example.com/").is_err());
        assert!(text_source("https://fanqienovel.com.evil.example/page/1").is_err());
        assert!(text_source("https://evil.example/?u=m.qidian.com").is_err());
        assert!(text_source("http://m.qidian.com/book/1/").is_err());
        assert!(text_source("file:///C:/Windows/win.ini").is_err());
        assert!(text_source("not a url").is_err());
    }

    #[test]
    fn anh_nhan_host_https_cong_khai_bat_ky() {
        assert!(image_url("https://p6-novel.byteimg.com/novel-pic/abc~tplv-shrink:640:0.image").is_ok());
        assert!(image_url("https://bookcover.reader.qq.com/cover/836/58257836/t9_58257836.webp").is_ok());
        assert!(image_url("https://pic.rmb.bdstatic.com/bjh/portrait/e8210b76.jpeg").is_ok());
    }

    #[test]
    fn anh_tu_choi_ip_localhost_ten_noi_bo_va_http() {
        for url in [
            "http://p6-novel.byteimg.com/origin/novel-pic/abc",
            "file:///C:/Windows/win.ini",
            "https://127.0.0.1/a.jpg",
            "https://192.168.1.1/a.jpg",
            "https://[::1]/a.jpg",
            "https://2130706433/a.jpg",
            "https://0x7f.0.0.1/a.jpg",
            "https://localhost/a.jpg",
            "https://router/a.jpg",
            "https://nas.local/a.jpg",
            "https://git.corp/a.jpg",
            "https://printer.lan./a.jpg",
        ] {
            assert!(image_url(url).is_err(), "{url}");
        }
    }

    #[test]
    fn chi_thu_lai_loi_phia_may_chu() {
        use reqwest::StatusCode;
        assert!(matches!(status_error(StatusCode::BAD_GATEWAY), ImageError::Transient(_)));
        assert!(matches!(status_error(StatusCode::TOO_MANY_REQUESTS), ImageError::Transient(_)));
        assert!(matches!(status_error(StatusCode::NOT_FOUND), ImageError::Fatal(_)));
        assert!(matches!(status_error(StatusCode::FORBIDDEN), ImageError::Fatal(_)));
    }

    #[test]
    fn nhan_dien_loai_anh_theo_byte_dau() {
        assert_eq!(sniff_image_mime(&[0xFF, 0xD8, 0xFF, 0xE0]), Some("image/jpeg"));
        assert_eq!(sniff_image_mime(&[0x89, b'P', b'N', b'G', 0x0D]), Some("image/png"));
        assert_eq!(sniff_image_mime(b"GIF89a"), Some("image/gif"));
        assert_eq!(sniff_image_mime(b"RIFF\x00\x00\x00\x00WEBPVP8 "), Some("image/webp"));
        assert_eq!(sniff_image_mime(b"<html>"), None);
    }

    /// Gọi thật trang nguồn: cargo test -p qt-ai-core live_ -- --ignored --nocapture
    #[test]
    #[ignore]
    fn live_nguon_tra_du_lieu_sach() {
        let qq = fetch_text("https://detailadr.reader.qq.com/book/queryDetailPage?bid=58257836&sex=1&qmk=1,3,7,9,10").expect("qq");
        assert!(qq.contains("introinfo"));
        let fanqie = fetch_text("https://fanqienovel.com/page/7445622960850750526").expect("fanqie");
        assert!(fanqie.contains("__INITIAL_STATE__"));
        let cover = fetch_image("https://bookcover.reader.qq.com/cover/836/58257836/t9_58257836.webp").expect("cover");
        assert_eq!(cover.mime, "image/webp");
    }
}
