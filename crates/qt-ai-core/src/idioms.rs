//! Bảng thành ngữ: chữ Hán → cách dịch bắt buộc (nhiều cách ngăn bằng ` / `). Bảng chung là `data/thanh-ngu.json`
//! của repo trên GitHub — app tải mỗi lần mở, lưu `base/idioms/remote.json` để phiên agy (`qt-ai next`) và phiên
//! API cùng thấy; chưa tải được lần nào thì dùng bản chụp đóng trong binary lúc build. Bảng người dùng
//! `base/idioms/user.json` đè theo khoá; giá trị rỗng = tắt mục đó của bảng chung.
//!
//! Chương có thành ngữ nào (so chữ Hán, bỏ dấu câu) thì prompt có mục đó; bản dịch thiếu mọi cách dịch đã cho thì
//! cảnh báo khi chốt chương, không đưa vào soát.

use crate::base::BaseStore;
use crate::error::{CoreError, Result};
use indexmap::IndexMap;
use std::path::PathBuf;
use std::time::{Duration, UNIX_EPOCH};

pub type IdiomTable = IndexMap<String, String>;

pub const REMOTE_URL: &str = "https://raw.githubusercontent.com/thiendiepdt/quick-translator-engine/main/data/thanh-ngu.json";
const BUILTIN: &str = include_str!("../../../data/thanh-ngu.json");
const FETCH_TIMEOUT: Duration = Duration::from_secs(20);
/// Vế ngắn hơn thì quá dễ trùng chữ thường (河东 trong tên đất).
const MIN_CLAUSE_CHARS: usize = 4;
pub const IDIOM_ISSUE: &str = "Thành ngữ trong bảng";

/// Object JSON chữ Hán → cách dịch; khoá/giá trị được trim, khoá rỗng bỏ. Giá trị rỗng giữ lại (bảng người dùng
/// dùng nó để tắt mục bảng chung).
pub fn parse_table(text: &str) -> Result<IdiomTable> {
    let raw: IndexMap<String, String> = serde_json::from_str(text)
        .map_err(|error| CoreError::InvalidState(format!("Bảng thành ngữ không đúng dạng {{\"chữ Hán\": \"cách dịch\"}}: {error}")))?;
    Ok(raw
        .into_iter()
        .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
        .filter(|(key, _)| !key.is_empty())
        .collect())
}

pub fn builtin() -> IdiomTable {
    parse_table(BUILTIN).expect("data/thanh-ngu.json hỏng")
}

/// Bảng chung rồi bảng người dùng đè lên; mục rỗng sau khi đè bị bỏ.
pub fn merge(remote: &IdiomTable, user: &IdiomTable) -> IdiomTable {
    let mut merged = remote.clone();
    for (key, value) in user {
        merged.insert(key.clone(), value.clone());
    }
    merged.retain(|_, value| !value.is_empty());
    merged
}

/// Chỉ giữ chữ Hán: so khớp không phụ thuộc dấu câu / khoảng trắng (三十年河东三十年河西 = 三十年河东，三十年河西).
fn han_only(text: &str) -> String {
    text.chars().filter(|c| crate::check::contains_han(c.encode_utf8(&mut [0; 4]))).collect()
}

fn clauses(key: &str) -> Vec<String> {
    key.split(['，', ',', '；', ';', '、', '。'])
        .map(han_only)
        .filter(|clause| clause.chars().count() >= MIN_CLAUSE_CHARS)
        .collect()
}

/// Raw có cả câu, hoặc một vế (≥4 chữ) của câu nhiều vế — prompt đưa cả câu để model tự cắt theo raw.
fn touches(key: &str, han_raw: &str) -> bool {
    let whole = han_only(key);
    (!whole.is_empty() && han_raw.contains(&whole)) || clauses(key).iter().any(|clause| han_raw.contains(clause))
}

/// Mục có mặt trong chương, bỏ khoá trùng glossary (glossary thắng thành ngữ).
pub fn idioms_for_source(table: &IdiomTable, source_text: &str, glossary_keys: &[&str]) -> IdiomTable {
    let han_raw = han_only(source_text);
    table
        .iter()
        .filter(|(key, value)| !value.is_empty() && !glossary_keys.contains(&key.as_str()) && touches(key, &han_raw))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}

/// Mục "Thành ngữ trong chương" của prompt; rỗng khi chương không có thành ngữ nào trong bảng.
pub fn prompt_section(idioms: &IdiomTable) -> String {
    if idioms.is_empty() {
        return String::new();
    }
    format!(
        "\n# Thành ngữ trong chương\n\nRaw có thành ngữ nào dưới đây thì BẮT BUỘC dịch đúng cách đã cho: được chỉnh hoa/thường, trợ từ cho xuôi câu; nhiều cách ngăn bằng ` / ` thì chọn một cách hợp câu; raw chỉ dùng một vế thì dịch đúng vế đó. Chỉ dịch khác khi raw dùng nghĩa đen của từng chữ, không phải thành ngữ.\n\n{}\n",
        crate::prompt::json_pretty(idioms)
    )
}

/// Chữ thường, mọi thứ không phải chữ/số thành một khoảng trắng — so cách dịch không phụ thuộc dấu câu, hoa/thường.
fn plain(text: &str) -> String {
    let lowered: String = text.to_lowercase().chars().map(|c| if c.is_alphanumeric() { c } else { ' ' }).collect();
    lowered.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdiomIssue {
    /// Chỉ số 0-based của nhãn [[n]].
    pub paragraph: usize,
    pub message: String,
}

/// Đoạn raw có nguyên câu thành ngữ mà đoạn dịch cùng nhãn không có cách dịch nào trong bảng. Chỉ xét nguyên câu:
/// raw dùng một vế thì bản dịch đúng cũng chỉ có một vế.
pub fn idiom_issues(raw: &[String], translated: &[Option<String>], table: &IdiomTable) -> Vec<IdiomIssue> {
    let mut issues = Vec::new();
    for (paragraph, source) in raw.iter().enumerate() {
        let Some(Some(text)) = translated.get(paragraph) else { continue };
        let han_raw = han_only(source);
        let text = plain(text);
        for (key, value) in table {
            let whole = han_only(key);
            if value.is_empty() || whole.is_empty() || !han_raw.contains(&whole) {
                continue;
            }
            let options: Vec<&str> = value.split(" / ").map(str::trim).filter(|o| !o.is_empty()).collect();
            if !options.iter().any(|option| text.contains(&plain(option))) {
                issues.push(IdiomIssue { paragraph, message: format!("{IDIOM_ISSUE}: {key} → {value}") });
            }
        }
    }
    issues
}

/// Bảng chung đang dùng là bản đã tải hay bản đóng trong binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteSource {
    Cache,
    Builtin,
}

impl BaseStore {
    fn idioms_file(&self, name: &str) -> Option<PathBuf> {
        self.dir().map(|dir| dir.join("idioms").join(name))
    }

    pub fn idioms_remote_path(&self) -> Option<PathBuf> {
        self.idioms_file("remote.json")
    }

    pub fn idioms_user_path(&self) -> Option<PathBuf> {
        self.idioms_file("user.json")
    }

    /// Bản đã tải (file đọc được, không rỗng) hoặc bản đóng sẵn.
    pub fn idioms_remote(&self) -> (IdiomTable, RemoteSource) {
        let cached = self
            .idioms_remote_path()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .and_then(|text| parse_table(&text).ok())
            .filter(|table| !table.is_empty());
        match cached {
            Some(table) => (table, RemoteSource::Cache),
            None => (builtin(), RemoteSource::Builtin),
        }
    }

    /// Lúc tải bảng chung về lần cuối (ms), theo giờ sửa file.
    pub fn idioms_fetched_at(&self) -> Option<u64> {
        let modified = std::fs::metadata(self.idioms_remote_path()?).ok()?.modified().ok()?;
        Some(modified.duration_since(UNIX_EPOCH).ok()?.as_millis() as u64)
    }

    pub fn idioms_user(&self) -> IdiomTable {
        self.idioms_user_path()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .and_then(|text| parse_table(&text).ok())
            .unwrap_or_default()
    }

    pub fn idioms(&self) -> IdiomTable {
        merge(&self.idioms_remote().0, &self.idioms_user())
    }

    pub fn save_idioms_remote(&self, table: &IdiomTable) -> Result<()> {
        self.write(self.idioms_remote_path(), &format!("{}\n", crate::prompt::json_pretty(table)))
    }

    pub fn save_idioms_user(&self, table: &IdiomTable) -> Result<()> {
        self.write(self.idioms_user_path(), &format!("{}\n", crate::prompt::json_pretty(table)))
    }
}

/// Tải bảng chung; bảng rỗng coi như lỗi (không đè bản đã tải bằng bảng rỗng).
pub fn fetch_remote(url: &str) -> Result<IdiomTable> {
    crate::api::install_crypto_provider();
    let client = reqwest::blocking::Client::builder()
        .timeout(FETCH_TIMEOUT)
        .build()
        .map_err(|error| CoreError::InvalidState(format!("Không tạo được kết nối: {error}")))?;
    let response = client
        .get(url)
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|error| CoreError::InvalidState(format!("Không tải được bảng thành ngữ: {error}")))?;
    let text = response.text().map_err(|error| CoreError::InvalidState(format!("Không đọc được bảng thành ngữ: {error}")))?;
    let table = parse_table(&text)?;
    if table.is_empty() {
        return Err(CoreError::InvalidState("Bảng thành ngữ tải về rỗng.".into()));
    }
    Ok(table)
}

/// Tải bảng chung và lưu vào base/; lỗi thì giữ bản cũ.
pub fn refresh_remote(store: &BaseStore, url: &str) -> Result<usize> {
    let table = fetch_remote(url)?;
    store.save_idioms_remote(&table)?;
    Ok(table.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(entries: &[(&str, &str)]) -> IdiomTable {
        entries.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    #[test]
    fn ban_dong_san_doc_duoc_va_co_cau_kinh_dien() {
        assert_eq!(builtin().get("三十年河东，三十年河西").map(String::as_str), Some("ba mươi năm hà đông, ba mươi năm hà tây"));
    }

    #[test]
    fn bang_nguoi_dung_de_theo_khoa_rong_la_tat() {
        let remote = table(&[("打草惊蛇", "đánh rắn động cỏ"), ("九死一生", "thập tử nhất sinh")]);
        let user = table(&[("打草惊蛇", "rút dây động rừng"), ("九死一生", ""), ("对牛弹琴", "đàn gảy tai trâu")]);
        assert_eq!(
            merge(&remote, &user),
            table(&[("打草惊蛇", "rút dây động rừng"), ("对牛弹琴", "đàn gảy tai trâu")])
        );
        assert!(parse_table("[1]").is_err());
        assert_eq!(parse_table(r#"{" 打草惊蛇 ": " a ", "": "b"}"#).unwrap(), table(&[("打草惊蛇", "a")]));
    }

    #[test]
    fn loc_theo_chuong_bo_dau_cau_khop_tung_ve_va_nhuong_glossary() {
        let all = table(&[
            ("三十年河东，三十年河西", "ba mươi năm hà đông, ba mươi năm hà tây"),
            ("打草惊蛇", "đánh rắn động cỏ"),
            ("九死一生", "thập tử nhất sinh"),
        ]);
        let found = idioms_for_source(&all, "正所谓三十年河东三十年河西，莫欺少年穷！", &[]);
        assert_eq!(found.keys().collect::<Vec<_>>(), vec!["三十年河东，三十年河西"]);
        let half = idioms_for_source(&all, "三十年河东，他如今落魄了。", &[]);
        assert_eq!(half.len(), 1, "một vế cũng đưa vào prompt");
        assert!(idioms_for_source(&all, "河东郡守", &[]).is_empty(), "vế dưới 4 chữ không tính");
        assert!(idioms_for_source(&all, "九死一生之地", &["九死一生"]).is_empty(), "glossary thắng");
        assert!(prompt_section(&IdiomTable::new()).is_empty());
        assert!(prompt_section(&found).contains("\"三十年河东，三十年河西\": \"ba mươi năm hà đông, ba mươi năm hà tây\""));
    }

    #[test]
    fn canh_bao_khi_ban_dich_khong_co_cach_dich_nao_trong_bang() {
        let all = table(&[("三十年河东，三十年河西", "ba mươi năm hà đông, ba mươi năm hà tây"), ("不可思议", "khó tin / không thể tin nổi")]);
        let raw = vec![
            "三十年河东，三十年河西！".to_string(),
            "三十年河东三十年河西。".to_string(),
            "真是不可思议。".to_string(),
            "三十年河东，他落魄了。".to_string(),
        ];
        let out = vec![
            Some("Ba mươi năm sông Đông, ba mươi năm sông Tây!".to_string()),
            Some("Ba mươi năm Hà Đông — ba mươi năm Hà Tây.".to_string()),
            Some("Thật không thể tin nổi.".to_string()),
            Some("Ba mươi năm hà đông, hắn sa cơ rồi.".to_string()),
        ];
        let issues = idiom_issues(&raw, &out, &all);
        assert_eq!(issues.iter().map(|i| i.paragraph).collect::<Vec<_>>(), vec![0], "{issues:?}");
        assert!(issues[0].message.starts_with(IDIOM_ISSUE));
    }

    #[test]
    fn luu_va_doc_lai_tu_base() {
        let dir = tempfile::tempdir().unwrap();
        let store = BaseStore::at(dir.path());
        assert_eq!(store.idioms_remote().1, RemoteSource::Builtin);
        assert!(store.idioms_fetched_at().is_none());
        store.save_idioms_remote(&table(&[("打草惊蛇", "đánh rắn động cỏ")])).unwrap();
        store.save_idioms_user(&table(&[("对牛弹琴", "đàn gảy tai trâu")])).unwrap();
        assert_eq!(store.idioms_remote().1, RemoteSource::Cache);
        assert!(store.idioms_fetched_at().is_some());
        assert_eq!(store.idioms().len(), 2);
        // Bản tải về hỏng thì quay về bản đóng sẵn.
        std::fs::write(store.idioms_remote_path().unwrap(), "{hỏng").unwrap();
        assert_eq!(store.idioms_remote().1, RemoteSource::Builtin);
    }
}
