//! Session data của extension (spec mục 6–7): giá trị có `ttl`, phiên `login`, hũ cookie. Gắn với auth: file
//! ghi mã phiên bản auth lúc tạo, lệch thì coi như trống. Tối đa 1 MB.
//!
//! Ghi: khoá chung trong process → đọc lại file → áp đúng thay đổi → bỏ mục hết hạn → kiểm trần → ghi an toàn.
//! Nhờ vậy hai lượt tải cùng extension không xoá giá trị của nhau (hũ cookie thì lượt ghi sau thắng).

use super::cache::check_key;
use super::{now_ms, write_atomic};
use crate::ExtError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub const SESSION_LIMIT: usize = 1024 * 1024;
/// Phiên login coi như hết hạn sớm hơn chừng này để không gửi token sắp hết hạn.
const LOGIN_MARGIN_MS: i64 = 30_000;

pub type Cookie = cookie_store::Cookie<'static>;

static WRITE_LOCK: Mutex<()> = Mutex::new(());

/// Kết quả `login` của extension.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// Mốc hết hạn (ms từ 1970); `None` = tới khi bị xoá.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
    #[serde(default)]
    pub data: Value,
    /// Tên tài khoản để hiện cho người dùng (extension có khối `account`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    /// Phiên khách: chưa đăng nhập tài khoản trang nguồn, vẫn tải được phần không cần tài khoản.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub guest: bool,
}

impl LoginRecord {
    pub fn valid_at(&self, now: i64) -> bool {
        self.expires_at.is_none_or(|at| now < at - LOGIN_MARGIN_MS)
    }

    /// `ctx.session` đưa cho extension.
    pub fn to_json(&self) -> Value {
        serde_json::json!({ "token": self.token, "expiresAt": self.expires_at, "data": self.data, "account": self.account, "guest": self.guest })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStatus {
    pub logged_in: bool,
    pub expires_at: Option<i64>,
    /// Có gì trong session data (phiên, cookie, giá trị).
    pub has_data: bool,
    /// Tên tài khoản của phiên (theo `login`).
    pub account: Option<String>,
    /// Phiên khách (chưa đăng nhập tài khoản trang nguồn).
    pub guest: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct Entry {
    value: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    expires: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct SessionFile {
    auth_version: String,
    values: BTreeMap<String, Entry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    login: Option<LoginRecord>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    cookies: Vec<Cookie>,
}

fn fresh(auth_version: &str) -> SessionFile {
    SessionFile { auth_version: auth_version.to_string(), ..Default::default() }
}

fn purge(file: &mut SessionFile, now: i64) {
    file.values.retain(|_, e| e.expires.is_none_or(|at| at > now));
    if file.login.as_ref().is_some_and(|l| l.expires_at.is_some_and(|at| at <= now)) {
        file.login = None;
    }
    file.cookies.retain(|c| !c.is_expired());
}

fn read(path: &Path, auth_version: &str, now: i64) -> SessionFile {
    let mut file: SessionFile = std::fs::read_to_string(path).ok().and_then(|text| serde_json::from_str(&text).ok()).unwrap_or_default();
    if file.auth_version != auth_version {
        file = fresh(auth_version);
    }
    purge(&mut file, now);
    file
}

fn expiry(ttl: Option<u64>, now: i64) -> Option<i64> {
    ttl.map(|t| now.saturating_add((t.min(i64::MAX as u64 / 1000) as i64).saturating_mul(1000)))
}

pub struct SessionData {
    path: Option<PathBuf>,
    auth_version: String,
    file: RefCell<SessionFile>,
}

impl SessionData {
    /// `path`: `None` = chỉ giữ trong bộ nhớ (test, chạy thử).
    pub fn open(path: Option<PathBuf>, auth_version: &str) -> SessionData {
        let file = match &path {
            Some(p) => read(p, auth_version, now_ms()),
            None => fresh(auth_version),
        };
        SessionData { path, auth_version: auth_version.to_string(), file: RefCell::new(file) }
    }

    fn update(&self, op: impl FnOnce(&mut SessionFile)) -> Result<(), ExtError> {
        let now = now_ms();
        let _guard = WRITE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut next = match &self.path {
            Some(p) => read(p, &self.auth_version, now),
            None => self.file.borrow().clone(),
        };
        op(&mut next);
        purge(&mut next, now);
        let json = serde_json::to_vec(&next).map_err(|e| ExtError::Store(e.to_string()))?;
        if json.len() > SESSION_LIMIT {
            return Err(ExtError::Store(format!("session data vượt {} MB", SESSION_LIMIT / 1024 / 1024)));
        }
        if let Some(p) = &self.path {
            write_atomic(p, &json)?;
        }
        *self.file.borrow_mut() = next;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        let now = now_ms();
        self.file.borrow().values.get(key).filter(|e| e.expires.is_none_or(|at| at > now)).map(|e| e.value.clone())
    }

    /// `ttl`: giây; `None` = giữ tới khi bị xoá.
    pub fn set(&self, key: &str, value: Value, ttl: Option<u64>) -> Result<(), ExtError> {
        check_key(key)?;
        let expires = expiry(ttl, now_ms());
        self.update(|f| {
            f.values.insert(key.to_string(), Entry { value, expires });
        })
    }

    pub fn delete(&self, key: &str) -> Result<(), ExtError> {
        check_key(key)?;
        self.update(|f| {
            f.values.remove(key);
        })
    }

    /// Xoá hết: giá trị, phiên login, cookie.
    pub fn clear(&self) -> Result<(), ExtError> {
        self.update(|f| *f = fresh(&f.auth_version))
    }

    /// Phiên login còn hạn.
    pub fn login(&self) -> Option<LoginRecord> {
        let now = now_ms();
        self.file.borrow().login.clone().filter(|l| l.valid_at(now))
    }

    pub fn set_login(&self, record: LoginRecord) -> Result<(), ExtError> {
        self.update(|f| f.login = Some(record))
    }

    /// Xoá phiên login và cookie, giữ giá trị `session.set`.
    pub fn clear_login(&self) -> Result<(), ExtError> {
        self.update(|f| {
            f.login = None;
            f.cookies.clear();
        })
    }

    pub fn cookies(&self) -> Vec<Cookie> {
        self.file.borrow().cookies.clone()
    }

    pub fn set_cookies(&self, cookies: Vec<Cookie>) -> Result<(), ExtError> {
        self.update(|f| f.cookies = cookies)
    }

    pub fn status(&self) -> SessionStatus {
        let file = self.file.borrow();
        let login = self.login();
        SessionStatus {
            logged_in: login.is_some(),
            expires_at: login.as_ref().and_then(|l| l.expires_at),
            has_data: !file.values.is_empty() || file.login.is_some() || !file.cookies.is_empty(),
            account: login.as_ref().and_then(|l| l.account.clone()),
            guest: login.is_some_and(|l| l.guest),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn temp() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("x/session.json");
        (dir, path)
    }

    #[test]
    fn gia_tri_ttl_va_xoa() {
        let data = SessionData::open(None, "v1");
        data.set("a", json!({ "k": 1 }), None).unwrap();
        data.set("b", json!("x"), Some(0)).unwrap();
        assert_eq!(data.get("a"), Some(json!({ "k": 1 })));
        assert_eq!(data.get("b"), None, "ttl 0 = hết hạn ngay");
        data.delete("a").unwrap();
        assert_eq!(data.get("a"), None);
        assert!(data.set("", json!(1), None).is_err());
    }

    #[test]
    fn tran_1_mb() {
        let data = SessionData::open(None, "v1");
        data.set("nho", json!("x"), None).unwrap();
        let err = data.set("to", json!("x".repeat(SESSION_LIMIT)), None).unwrap_err();
        assert!(err.to_string().contains("1 MB"));
        assert_eq!(data.get("to"), None);
        assert_eq!(data.get("nho"), Some(json!("x")), "lỗi thì giữ nguyên dữ liệu cũ");
    }

    #[test]
    fn lech_phien_ban_auth_thi_trong() {
        let (_dir, path) = temp();
        SessionData::open(Some(path.clone()), "v1").set("a", json!(1), None).unwrap();
        assert_eq!(SessionData::open(Some(path.clone()), "v1").get("a"), Some(json!(1)));
        let other = SessionData::open(Some(path.clone()), "v2");
        assert_eq!(other.get("a"), None);
        other.set("b", json!(2), None).unwrap();
        assert_eq!(SessionData::open(Some(path), "v1").get("b"), None);
    }

    #[test]
    fn hai_handle_cung_file_khong_mat_khoa_cua_nhau() {
        let (_dir, path) = temp();
        let one = SessionData::open(Some(path.clone()), "v");
        let two = SessionData::open(Some(path.clone()), "v");
        one.set("a", json!(1), None).unwrap();
        two.set("b", json!(2), None).unwrap();
        let again = SessionData::open(Some(path), "v");
        assert_eq!((again.get("a"), again.get("b")), (Some(json!(1)), Some(json!(2))));
        assert_eq!(two.get("a"), Some(json!(1)), "ghi xong thì thấy cả thay đổi của handle khác");
    }

    #[test]
    fn phien_login_het_han_va_xoa_phien() {
        let (_dir, path) = temp();
        let data = SessionData::open(Some(path.clone()), "v");
        assert_eq!(data.status(), SessionStatus::default());
        let far = now_ms() + 3_600_000;
        data.set_login(LoginRecord { token: Some("tk".into()), expires_at: Some(far), data: json!({ "uid": 7 }), ..Default::default() }).unwrap();
        data.set("giu", json!(true), None).unwrap();
        assert_eq!(data.login().unwrap().token.as_deref(), Some("tk"));
        assert_eq!(data.status(), SessionStatus { logged_in: true, expires_at: Some(far), has_data: true, ..Default::default() });
        assert_eq!(SessionData::open(Some(path.clone()), "v").login().unwrap().data, json!({ "uid": 7 }));

        data.set_login(LoginRecord { token: Some("tk".into()), expires_at: Some(now_ms() + 10_000), ..Default::default() }).unwrap();
        assert!(data.login().is_none(), "còn dưới 30 giây coi như hết hạn");

        data.clear_login().unwrap();
        assert_eq!(data.get("giu"), Some(json!(true)), "xoá phiên giữ giá trị session.set");
        data.clear().unwrap();
        assert_eq!(data.get("giu"), None);
        assert!(!data.status().has_data);
    }

    #[test]
    fn luu_cookie() {
        let (_dir, path) = temp();
        let url = url::Url::parse("https://a.test/").unwrap();
        let cookie = Cookie::parse("sid=abc; Path=/", &url).unwrap().into_owned();
        let data = SessionData::open(Some(path.clone()), "v");
        data.set_cookies(vec![cookie]).unwrap();
        let back = SessionData::open(Some(path), "v").cookies();
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].value(), "abc");
    }
}
