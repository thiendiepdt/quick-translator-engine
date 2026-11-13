//! Hũ cookie của một lượt chạy: `CookieStore` của reqwest trên `cookie_store`, nạp từ session data. Có cookie
//! mới thì bật cờ để engine lưu lại vào session data sau request.

use crate::store::session_data::Cookie;
use cookie_store::RawCookie;
use reqwest::header::HeaderValue;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::RwLock;
use url::Url;

#[derive(Default)]
pub(crate) struct Jar {
    store: RwLock<cookie_store::CookieStore>,
    dirty: AtomicBool,
}

impl Jar {
    pub fn from_cookies(cookies: Vec<Cookie>) -> Jar {
        let store = cookie_store::CookieStore::from_cookies(cookies.into_iter().map(Ok::<_, ()>), false).unwrap_or_default();
        Jar { store: RwLock::new(store), dirty: AtomicBool::new(false) }
    }

    /// Cookie còn hạn (kể cả cookie phiên) — để lưu.
    pub fn snapshot(&self) -> Vec<Cookie> {
        self.store.read().unwrap_or_else(|e| e.into_inner()).iter_unexpired().cloned().collect()
    }

    /// Có cookie mới từ lần hỏi trước không.
    pub fn take_dirty(&self) -> bool {
        self.dirty.swap(false, Ordering::SeqCst)
    }

    pub fn clear(&self) {
        self.store.write().unwrap_or_else(|e| e.into_inner()).clear();
        self.dirty.store(false, Ordering::SeqCst);
    }
}

impl reqwest::cookie::CookieStore for Jar {
    fn set_cookies(&self, cookie_headers: &mut dyn Iterator<Item = &HeaderValue>, url: &Url) {
        let cookies: Vec<RawCookie<'static>> =
            cookie_headers.filter_map(|v| std::str::from_utf8(v.as_bytes()).ok()).filter_map(|s| RawCookie::parse(s.to_string()).ok()).collect();
        if cookies.is_empty() {
            return;
        }
        self.store.write().unwrap_or_else(|e| e.into_inner()).store_response_cookies(cookies.into_iter(), url);
        self.dirty.store(true, Ordering::SeqCst);
    }

    fn cookies(&self, url: &Url) -> Option<HeaderValue> {
        let header = self.store.read().unwrap_or_else(|e| e.into_inner()).get_request_values(url).map(|(name, value)| format!("{name}={value}")).collect::<Vec<_>>().join("; ");
        if header.is_empty() {
            return None;
        }
        HeaderValue::from_str(&header).ok()
    }
}
