//! Dựng file EPUB 3 (kèm toc.ncx cho máy đọc cũ) từ các chương đã dịch.
//!
//! Thứ tự đọc: bìa → trang tiêu đề → trang Mục lục (chính là nav.xhtml, mỗi dòng là link tới chương) → các
//! chương, mỗi chương một file để truyện nghìn chương mở và lật trang không đơ. `mimetype` đứng đầu, không nén.

use crate::error::{CoreError, Result};
use std::io::{Cursor, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

#[derive(Debug, Clone, PartialEq)]
pub struct EpubChapter {
    pub title: String,
    pub paragraphs: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct EpubBook {
    pub title: String,
    /// Tên gốc tiếng Trung (rỗng = không có).
    pub original_title: String,
    pub author: String,
    /// Bút danh gốc chữ Hán (rỗng = không có): trang tiêu đề ghi `Tác giả: Hán-Việt (gốc)`.
    pub author_original: String,
    pub protagonist: String,
    pub summary: String,
    /// `urn:uuid:…` — giữ cố định theo truyện để app đọc nhận ra bản export lại là cùng một sách.
    pub identifier: String,
    /// `YYYY-MM-DDThh:mm:ssZ`.
    pub modified: String,
    pub cover_jpeg: Option<Vec<u8>>,
    pub chapters: Vec<EpubChapter>,
}

/// Dòng đầu là tiêu đề (`Chương N: …`), mỗi dòng không trống còn lại là một đoạn.
pub fn chapter_from_text(text: &str) -> EpubChapter {
    let mut lines = text.lines().map(str::trim).filter(|line| !line.is_empty());
    let title = lines.next().unwrap_or_default().to_string();
    EpubChapter { title, paragraphs: lines.map(str::to_string).collect() }
}

/// `"Phương Thốn Đạo Chủ (方寸道主)"` → (`Phương Thốn Đạo Chủ`, `方寸道主`); ngoặc không chứa chữ Hán thì giữ nguyên tên.
pub fn split_story_name(name: &str) -> (String, String) {
    let name = name.trim();
    for (open, close) in [(" (", ')'), (" （", '）'), ("(", ')'), ("（", '）')] {
        if let Some(start) = name.rfind(open) {
            let inner = &name[start + open.len()..];
            if let Some(inner) = inner.strip_suffix(close) {
                if inner.chars().any(is_han) {
                    return (name[..start].trim().to_string(), inner.trim().to_string());
                }
            }
        }
    }
    (name.to_string(), String::new())
}

fn is_han(c: char) -> bool {
    matches!(c, '\u{3400}'..='\u{4DBF}' | '\u{4E00}'..='\u{9FFF}' | '\u{F900}'..='\u{FAFF}')
}

/// UUID dạng chữ từ hash FNV-1a của khoá (tên truyện + link) — không cần ngẫu nhiên, cần cố định.
pub fn identifier_for(key: &str) -> String {
    let hash = |seed: u64| {
        key.bytes().fold(seed, |acc, byte| (acc ^ u64::from(byte)).wrapping_mul(0x100000001b3))
    };
    let (a, b) = (hash(0xcbf29ce484222325), hash(0x84222325cbf29ce4));
    let hex = format!("{a:016x}{b:016x}");
    format!("urn:uuid:{}-{}-5{}-a{}-{}", &hex[0..8], &hex[8..12], &hex[13..16], &hex[17..20], &hex[20..32])
}

/// Thời điểm hiện tại theo UTC, dạng `dcterms:modified` yêu cầu.
pub fn utc_now() -> String {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    format_utc(secs)
}

fn format_utc(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    // Đổi số ngày từ 1970-01-01 sang năm-tháng-ngày (thuật toán civil_from_days của Howard Hinnant).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z", rem / 3600, rem % 3600 / 60, rem % 60)
}

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            // Ký tự điều khiển không hợp lệ trong XML 1.0.
            c if (c as u32) < 0x20 && !matches!(c, '\t' | '\n' | '\r') => {}
            c => out.push(c),
        }
    }
    out
}

pub fn chapter_file(index: usize) -> String {
    format!("text/c{:05}.xhtml", index + 1)
}

const STYLE: &str = "body { margin: 0 4%; line-height: 1.6; }
h1, h2 { text-align: center; line-height: 1.3; }
h2 { margin: 1.5em 0 1em; }
p { text-indent: 1.5em; margin: 0 0 0.6em; text-align: justify; }
.center { text-align: center; text-indent: 0; }
.original { font-size: 1.1em; }
.meta { text-indent: 0; text-align: center; margin: 0.3em 0; }
.summary { margin-top: 2em; }
nav#toc ol { list-style: none; padding: 0; margin: 0; }
nav#toc li { margin: 0.35em 0; }
nav#toc a { text-decoration: none; }
.cover { margin: 0; padding: 0; text-align: center; }
.cover img { max-width: 100%; max-height: 100%; }
";

/// `css`: đường dẫn style.css tính từ file trang (`style.css` ở gốc OEBPS, `../style.css` trong `text/`).
fn xhtml(title: &str, css: &str, body: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<!DOCTYPE html>\n<html xmlns=\"http://www.w3.org/1999/xhtml\" xmlns:epub=\"http://www.idpf.org/2007/ops\" xml:lang=\"vi\" lang=\"vi\">\n<head>\n<meta charset=\"utf-8\"/>\n<title>{}</title>\n<link rel=\"stylesheet\" type=\"text/css\" href=\"{css}\"/>\n</head>\n<body>\n{body}</body>\n</html>\n",
        escape(title)
    )
}

fn page(title: &str, body: &str) -> String {
    xhtml(title, "style.css", body)
}

fn chapter_page(chapter: &EpubChapter) -> String {
    let mut body = format!("<section epub:type=\"chapter\">\n<h2>{}</h2>\n", escape(&chapter.title));
    for paragraph in &chapter.paragraphs {
        body.push_str(&format!("<p>{}</p>\n", escape(paragraph)));
    }
    body.push_str("</section>\n");
    xhtml(&chapter.title, "../style.css", &body)
}

fn title_page(book: &EpubBook) -> String {
    let mut body = format!("<section epub:type=\"titlepage\">\n<h1>{}</h1>\n", escape(&book.title));
    if !book.original_title.is_empty() {
        body.push_str(&format!("<p class=\"meta original\">{}</p>\n", escape(&book.original_title)));
    }
    let author = match (book.author.is_empty(), book.author_original.is_empty()) {
        (false, false) if book.author != book.author_original => format!("{} ({})", book.author, book.author_original),
        (false, _) => book.author.clone(),
        (true, _) => book.author_original.clone(),
    };
    if !author.is_empty() {
        body.push_str(&format!("<p class=\"meta\">Tác giả: {}</p>\n", escape(&author)));
    }
    if !book.protagonist.is_empty() {
        body.push_str(&format!("<p class=\"meta\">Nhân vật chính: {}</p>\n", escape(&book.protagonist)));
    }
    body.push_str(&format!("<p class=\"meta\">{} chương</p>\n", book.chapters.len()));
    let summary: Vec<&str> = book.summary.lines().map(str::trim).filter(|line| !line.is_empty()).collect();
    if !summary.is_empty() {
        body.push_str("<div class=\"summary\">\n");
        for line in summary {
            body.push_str(&format!("<p>{}</p>\n", escape(line)));
        }
        body.push_str("</div>\n");
    }
    body.push_str("</section>\n");
    page(&book.title, &body)
}

fn cover_page(title: &str) -> String {
    page(title, "<div class=\"cover\"><img src=\"images/cover.jpg\" alt=\"Bìa\"/></div>\n")
}

/// Trang Mục lục: vừa là nav của EPUB 3 (menu mục lục của app đọc), vừa nằm trong luồng đọc.
fn nav_page(book: &EpubBook) -> String {
    let mut body = String::from("<nav epub:type=\"toc\" id=\"toc\">\n<h1>Mục lục</h1>\n<ol>\n");
    for (index, chapter) in book.chapters.iter().enumerate() {
        body.push_str(&format!("<li><a href=\"{}\">{}</a></li>\n", chapter_file(index), escape(&chapter.title)));
    }
    body.push_str("</ol>\n</nav>\n<nav epub:type=\"landmarks\" hidden=\"hidden\">\n<ol>\n");
    if book.cover_jpeg.is_some() {
        body.push_str("<li><a epub:type=\"cover\" href=\"cover.xhtml\">Bìa</a></li>\n");
    }
    body.push_str("<li><a epub:type=\"toc\" href=\"nav.xhtml\">Mục lục</a></li>\n");
    body.push_str(&format!("<li><a epub:type=\"bodymatter\" href=\"{}\">Bắt đầu đọc</a></li>\n", chapter_file(0)));
    body.push_str("</ol>\n</nav>\n");
    page("Mục lục", &body)
}

fn ncx(book: &EpubBook) -> String {
    let mut points = String::new();
    for (index, chapter) in book.chapters.iter().enumerate() {
        points.push_str(&format!(
            "<navPoint id=\"np{n}\" playOrder=\"{n}\"><navLabel><text>{}</text></navLabel><content src=\"{}\"/></navPoint>\n",
            escape(&chapter.title),
            chapter_file(index),
            n = index + 1
        ));
    }
    format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<ncx xmlns=\"http://www.daisy.org/z3986/2005/ncx/\" version=\"2005-1\" xml:lang=\"vi\">\n<head>\n<meta name=\"dtb:uid\" content=\"{}\"/>\n<meta name=\"dtb:depth\" content=\"1\"/>\n<meta name=\"dtb:totalPageCount\" content=\"0\"/>\n<meta name=\"dtb:maxPageNumber\" content=\"0\"/>\n</head>\n<docTitle><text>{}</text></docTitle>\n<navMap>\n{points}</navMap>\n</ncx>\n",
        escape(&book.identifier),
        escape(&book.title)
    )
}

fn opf(book: &EpubBook) -> String {
    let has_cover = book.cover_jpeg.is_some();
    let mut metadata = format!(
        "<dc:identifier id=\"book-id\">{}</dc:identifier>\n<dc:title>{}</dc:title>\n<dc:language>vi</dc:language>\n",
        escape(&book.identifier),
        escape(&book.title)
    );
    // App đọc xếp sách theo dòng này: chỉ một tên, ưu tiên Hán-Việt.
    let creator = if book.author.is_empty() { &book.author_original } else { &book.author };
    if !creator.is_empty() {
        metadata.push_str(&format!("<dc:creator>{}</dc:creator>\n", escape(creator)));
    }
    if !book.summary.trim().is_empty() {
        metadata.push_str(&format!("<dc:description>{}</dc:description>\n", escape(book.summary.trim())));
    }
    metadata.push_str(&format!("<meta property=\"dcterms:modified\">{}</meta>\n", escape(&book.modified)));
    if has_cover {
        // Cách khai bìa của EPUB 2 — Kindle và app đọc cũ chỉ hiểu dòng này.
        metadata.push_str("<meta name=\"cover\" content=\"cover-image\"/>\n");
    }

    let mut manifest = String::from(
        "<item id=\"nav\" href=\"nav.xhtml\" media-type=\"application/xhtml+xml\" properties=\"nav\"/>\n<item id=\"ncx\" href=\"toc.ncx\" media-type=\"application/x-dtbncx+xml\"/>\n<item id=\"css\" href=\"style.css\" media-type=\"text/css\"/>\n<item id=\"title\" href=\"title.xhtml\" media-type=\"application/xhtml+xml\"/>\n",
    );
    let mut spine = String::new();
    if has_cover {
        manifest.push_str("<item id=\"cover-image\" href=\"images/cover.jpg\" media-type=\"image/jpeg\" properties=\"cover-image\"/>\n<item id=\"cover\" href=\"cover.xhtml\" media-type=\"application/xhtml+xml\"/>\n");
        spine.push_str("<itemref idref=\"cover\"/>\n");
    }
    spine.push_str("<itemref idref=\"title\"/>\n<itemref idref=\"nav\"/>\n");
    for index in 0..book.chapters.len() {
        manifest.push_str(&format!(
            "<item id=\"c{n}\" href=\"{}\" media-type=\"application/xhtml+xml\"/>\n",
            chapter_file(index),
            n = index + 1
        ));
        spine.push_str(&format!("<itemref idref=\"c{}\"/>\n", index + 1));
    }
    let mut guide = String::new();
    if has_cover {
        guide.push_str("<reference type=\"cover\" title=\"Bìa\" href=\"cover.xhtml\"/>\n");
    }
    guide.push_str("<reference type=\"toc\" title=\"Mục lục\" href=\"nav.xhtml\"/>\n");
    format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<package xmlns=\"http://www.idpf.org/2007/opf\" version=\"3.0\" unique-identifier=\"book-id\" xml:lang=\"vi\">\n<metadata xmlns:dc=\"http://purl.org/dc/elements/1.1/\">\n{metadata}</metadata>\n<manifest>\n{manifest}</manifest>\n<spine toc=\"ncx\">\n{spine}</spine>\n<guide>\n{guide}</guide>\n</package>\n"
    )
}

const CONTAINER: &str = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<container version=\"1.0\" xmlns=\"urn:oasis:names:tc:opendocument:xmlns:container\">\n<rootfiles>\n<rootfile full-path=\"OEBPS/content.opf\" media-type=\"application/oebps-package+xml\"/>\n</rootfiles>\n</container>\n";

fn zip_error(error: impl std::fmt::Display) -> CoreError {
    CoreError::Internal(format!("Không ghi được file EPUB: {error}"))
}

pub fn build_epub(book: &EpubBook) -> Result<Vec<u8>> {
    if book.chapters.is_empty() {
        return Err(CoreError::InvalidState("Không có chương nào để đưa vào EPUB.".to_string()));
    }
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let deflated = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let put = |zip: &mut ZipWriter<Cursor<Vec<u8>>>, name: &str, options: SimpleFileOptions, data: &[u8]| -> Result<()> {
        zip.start_file(name, options).map_err(zip_error)?;
        zip.write_all(data).map_err(zip_error)
    };
    // Spec OCF: `mimetype` là file đầu tiên, không nén, không extra field.
    put(&mut zip, "mimetype", stored, b"application/epub+zip")?;
    put(&mut zip, "META-INF/container.xml", deflated, CONTAINER.as_bytes())?;
    put(&mut zip, "OEBPS/content.opf", deflated, opf(book).as_bytes())?;
    put(&mut zip, "OEBPS/toc.ncx", deflated, ncx(book).as_bytes())?;
    put(&mut zip, "OEBPS/nav.xhtml", deflated, nav_page(book).as_bytes())?;
    put(&mut zip, "OEBPS/style.css", deflated, STYLE.as_bytes())?;
    put(&mut zip, "OEBPS/title.xhtml", deflated, title_page(book).as_bytes())?;
    if let Some(cover) = &book.cover_jpeg {
        put(&mut zip, "OEBPS/cover.xhtml", deflated, cover_page(&book.title).as_bytes())?;
        // JPEG đã nén sẵn: nén lại chỉ tốn thời gian.
        put(&mut zip, "OEBPS/images/cover.jpg", stored, cover)?;
    }
    for (index, chapter) in book.chapters.iter().enumerate() {
        put(&mut zip, &format!("OEBPS/{}", chapter_file(index)), deflated, chapter_page(chapter).as_bytes())?;
    }
    Ok(zip.finish().map_err(zip_error)?.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    fn sample(cover: bool) -> EpubBook {
        EpubBook {
            title: "Phương Thốn Đạo Chủ".into(),
            original_title: "方寸道主".into(),
            author: "Phẫn Nộ Đích Ô Tặc".into(),
            author_original: "愤怒的乌贼".into(),
            protagonist: "Lý Thuận".into(),
            summary: "Dòng 1 <b>&</b>\nDòng 2".into(),
            identifier: identifier_for("Phương Thốn Đạo Chủ|https://book.qq.com/book-detail/58257836"),
            modified: "2026-10-15T01:02:03Z".into(),
            cover_jpeg: cover.then(|| vec![0xFF, 0xD8, 0xFF, 0xE0, 0, 1]),
            chapters: vec![
                chapter_from_text("Chương 1: Ngô nhật tam tỉnh thân\n\n“Thằng què, xảy ra chuyện lớn rồi!”\n\nTrốn?\n"),
                chapter_from_text("Chương 2: A & B <c>\n\nĐoạn một."),
            ],
        }
    }

    fn entries(bytes: &[u8]) -> zip::ZipArchive<Cursor<&[u8]>> {
        zip::ZipArchive::new(Cursor::new(bytes)).unwrap()
    }

    fn read(archive: &mut zip::ZipArchive<Cursor<&[u8]>>, name: &str) -> String {
        let mut text = String::new();
        archive.by_name(name).unwrap().read_to_string(&mut text).unwrap();
        text
    }

    #[test]
    fn tach_tieu_de_va_doan() {
        let chapter = chapter_from_text("Chương 1: Kiếp này làm mèo\n\nĐoạn một.\n\n  Đoạn hai.  \n");
        assert_eq!(chapter.title, "Chương 1: Kiếp này làm mèo");
        assert_eq!(chapter.paragraphs, vec!["Đoạn một.", "Đoạn hai."]);
    }

    #[test]
    fn tach_ten_goc_trong_ngoac() {
        assert_eq!(split_story_name("Phương Thốn Đạo Chủ (方寸道主)"), ("Phương Thốn Đạo Chủ".into(), "方寸道主".into()));
        assert_eq!(split_story_name("Trường Sinh Là Mèo"), ("Trường Sinh Là Mèo".into(), String::new()));
        assert_eq!(split_story_name("Tên (bản dịch)"), ("Tên (bản dịch)".into(), String::new()));
        assert_eq!(split_story_name("Tên（长生）"), ("Tên".into(), "长生".into()));
    }

    #[test]
    fn identifier_co_dinh_theo_khoa_va_dung_dang_uuid() {
        let a = identifier_for("x");
        assert_eq!(a, identifier_for("x"));
        assert_ne!(a, identifier_for("y"));
        let uuid = a.strip_prefix("urn:uuid:").unwrap();
        let parts: Vec<usize> = uuid.split('-').map(str::len).collect();
        assert_eq!(parts, vec![8, 4, 4, 4, 12]);
    }

    #[test]
    fn dinh_dang_gio_utc() {
        assert_eq!(format_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(format_utc(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(format_utc(1_791_853_323), "2026-10-13T01:02:03Z");
    }

    #[test]
    fn mimetype_dung_dau_khong_nen_khong_extra_field() {
        let bytes = build_epub(&sample(true)).unwrap();
        // Local file header: chữ ký PK\x03\x04, phương thức nén ở byte 8 (0 = stored), độ dài tên ở 26, extra ở 28.
        assert_eq!(&bytes[0..4], b"PK\x03\x04");
        assert_eq!(u16::from_le_bytes([bytes[8], bytes[9]]), 0);
        assert_eq!(u16::from_le_bytes([bytes[26], bytes[27]]), 8);
        assert_eq!(u16::from_le_bytes([bytes[28], bytes[29]]), 0);
        assert_eq!(&bytes[30..38], b"mimetype");
        assert_eq!(&bytes[38..58], b"application/epub+zip");
    }

    #[test]
    fn muc_luc_tro_dung_tung_chuong_va_moi_file_deu_co() {
        let bytes = build_epub(&sample(true)).unwrap();
        let mut archive = entries(&bytes);
        let nav = read(&mut archive, "OEBPS/nav.xhtml");
        let ncx = read(&mut archive, "OEBPS/toc.ncx");
        let opf = read(&mut archive, "OEBPS/content.opf");
        assert!(nav.contains("<li><a href=\"text/c00001.xhtml\">Chương 1: Ngô nhật tam tỉnh thân</a></li>"));
        assert!(nav.contains("<li><a href=\"text/c00002.xhtml\">Chương 2: A &amp; B &lt;c&gt;</a></li>"));
        assert!(ncx.contains("<content src=\"text/c00002.xhtml\"/>"));
        // Trang Mục lục nằm trong luồng đọc, ngay sau trang tiêu đề.
        assert!(opf.contains("<itemref idref=\"cover\"/>\n<itemref idref=\"title\"/>\n<itemref idref=\"nav\"/>\n<itemref idref=\"c1\"/>"));
        assert!(opf.contains("properties=\"cover-image\"") && opf.contains("<meta name=\"cover\" content=\"cover-image\"/>"));
        assert!(opf.contains("<dc:creator>Phẫn Nộ Đích Ô Tặc</dc:creator>"));
        assert!(opf.contains("<dc:description>Dòng 1 &lt;b&gt;&amp;&lt;/b&gt;\nDòng 2</dc:description>"));
        // Mọi href trong manifest đều có file thật.
        for href in opf.split("href=\"").skip(1).map(|rest| rest.split('"').next().unwrap()) {
            assert!(archive.by_name(&format!("OEBPS/{href}")).is_ok(), "thiếu {href}");
        }
        let chapter = read(&mut archive, "OEBPS/text/c00001.xhtml");
        assert!(chapter.contains("<h2>Chương 1: Ngô nhật tam tỉnh thân</h2>"));
        assert!(chapter.contains("<p>“Thằng què, xảy ra chuyện lớn rồi!”</p>\n<p>Trốn?</p>"));
        assert!(chapter.contains("href=\"../style.css\""));
        let title = read(&mut archive, "OEBPS/title.xhtml");
        assert!(title.contains("<h1>Phương Thốn Đạo Chủ</h1>") && title.contains("方寸道主") && title.contains("2 chương"));
        assert!(title.contains("<p class=\"meta\">Tác giả: Phẫn Nộ Đích Ô Tặc (愤怒的乌贼)</p>"));
    }

    #[test]
    fn khong_bia_thi_khong_co_trang_bia() {
        let bytes = build_epub(&sample(false)).unwrap();
        let mut archive = entries(&bytes);
        let opf = read(&mut archive, "OEBPS/content.opf");
        assert!(!opf.contains("cover"));
        assert!(archive.by_name("OEBPS/cover.xhtml").is_err());
        assert!(!read(&mut archive, "OEBPS/nav.xhtml").contains("cover.xhtml"));
    }

    #[test]
    fn khong_co_chuong_thi_bao_loi() {
        let mut book = sample(false);
        book.chapters.clear();
        assert!(build_epub(&book).is_err());
    }
}
