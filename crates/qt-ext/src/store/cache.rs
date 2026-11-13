//! Cache của extension trên SQLite (spec mục 7): `ttl` tính bằng giây, tối đa 50 MB, đầy thì xoá mục dùng lâu
//! nhất. Thứ tự dùng là bộ đếm tăng dần (`used`), không phụ thuộc đồng hồ. Mở bằng WAL + busy_timeout để hai
//! lượt tải cùng extension dùng chung file được.

use super::{io_error, now_ms};
use crate::ExtError;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::time::Duration;

pub const CACHE_QUOTA: u64 = 50 * 1024 * 1024;
pub const MAX_CACHE_ENTRY: usize = 5 * 1024 * 1024;
pub const MAX_KEY: usize = 512;

pub struct Cache {
    conn: Connection,
    quota: u64,
}

fn db_error(error: rusqlite::Error) -> ExtError {
    ExtError::Store(format!("cache: {error}"))
}

pub(crate) fn check_key(key: &str) -> Result<(), ExtError> {
    if key.is_empty() || key.chars().count() > MAX_KEY {
        Err(ExtError::Store(format!("khoá phải dài 1–{MAX_KEY} ký tự")))
    } else {
        Ok(())
    }
}

const NEXT_USED: &str = "(SELECT COALESCE(MAX(used), 0) + 1 FROM entries)";

impl Cache {
    pub fn open(path: &Path) -> Result<Cache, ExtError> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(io_error(dir))?;
        }
        let conn = Connection::open(path).map_err(db_error)?;
        conn.busy_timeout(Duration::from_secs(5)).map_err(db_error)?;
        conn.query_row("PRAGMA journal_mode = WAL", [], |_| Ok(())).map_err(db_error)?;
        Cache::init(conn)
    }

    pub fn memory() -> Result<Cache, ExtError> {
        Cache::init(Connection::open_in_memory().map_err(db_error)?)
    }

    fn init(conn: Connection) -> Result<Cache, ExtError> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS entries (key TEXT PRIMARY KEY, value TEXT NOT NULL, size INTEGER NOT NULL, expires INTEGER, used INTEGER NOT NULL);
             CREATE INDEX IF NOT EXISTS entries_used ON entries(used);",
        )
        .map_err(db_error)?;
        Ok(Cache { conn, quota: CACHE_QUOTA })
    }

    pub fn with_quota(mut self, quota: u64) -> Self {
        self.quota = quota;
        self
    }

    pub fn get(&self, key: &str) -> Result<Option<String>, ExtError> {
        self.get_at(key, now_ms())
    }

    /// `now`: mốc thời gian (ms) — cho test.
    #[doc(hidden)]
    pub fn get_at(&self, key: &str, now: i64) -> Result<Option<String>, ExtError> {
        check_key(key)?;
        let row: Option<(String, Option<i64>)> =
            self.conn.query_row("SELECT value, expires FROM entries WHERE key = ?1", [key], |r| Ok((r.get(0)?, r.get(1)?))).optional().map_err(db_error)?;
        match row {
            None => Ok(None),
            Some((_, Some(expires))) if expires <= now => {
                self.delete(key)?;
                Ok(None)
            }
            Some((value, _)) => {
                self.conn.execute(&format!("UPDATE entries SET used = {NEXT_USED} WHERE key = ?1"), [key]).map_err(db_error)?;
                Ok(Some(value))
            }
        }
    }

    /// `ttl`: giây; `None` = giữ tới khi bị xoá.
    pub fn set(&self, key: &str, value: &str, ttl: Option<u64>) -> Result<(), ExtError> {
        self.set_at(key, value, ttl, now_ms())
    }

    #[doc(hidden)]
    pub fn set_at(&self, key: &str, value: &str, ttl: Option<u64>, now: i64) -> Result<(), ExtError> {
        check_key(key)?;
        let size = key.len() + value.len();
        if size > MAX_CACHE_ENTRY {
            return Err(ExtError::Store(format!("mục cache quá {} MB", MAX_CACHE_ENTRY / 1024 / 1024)));
        }
        let expires = ttl.map(|t| now.saturating_add((t.min(i64::MAX as u64 / 1000) as i64).saturating_mul(1000)));
        self.conn
            .execute(
                &format!(
                    "INSERT INTO entries (key, value, size, expires, used) VALUES (?1, ?2, ?3, ?4, {NEXT_USED})
                     ON CONFLICT(key) DO UPDATE SET value = excluded.value, size = excluded.size, expires = excluded.expires, used = excluded.used"
                ),
                params![key, value, size as i64, expires],
            )
            .map_err(db_error)?;
        self.evict(now)
    }

    fn evict(&self, now: i64) -> Result<(), ExtError> {
        if self.total_bytes()? <= self.quota {
            return Ok(());
        }
        self.conn.execute("DELETE FROM entries WHERE expires IS NOT NULL AND expires <= ?1", [now]).map_err(db_error)?;
        let mut total = self.total_bytes()?;
        if total <= self.quota {
            return Ok(());
        }
        let mut victims = Vec::new();
        {
            let mut stmt = self.conn.prepare("SELECT key, size FROM entries ORDER BY used ASC").map_err(db_error)?;
            let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))).map_err(db_error)?;
            for row in rows {
                let (key, size) = row.map_err(db_error)?;
                if total <= self.quota {
                    break;
                }
                total = total.saturating_sub(size.max(0) as u64);
                victims.push(key);
            }
        }
        for key in victims {
            self.conn.execute("DELETE FROM entries WHERE key = ?1", [key]).map_err(db_error)?;
        }
        Ok(())
    }

    pub fn delete(&self, key: &str) -> Result<(), ExtError> {
        check_key(key)?;
        self.conn.execute("DELETE FROM entries WHERE key = ?1", [key]).map_err(db_error)?;
        Ok(())
    }

    /// Tổng cỡ khoá + giá trị đang giữ.
    pub fn total_bytes(&self) -> Result<u64, ExtError> {
        let total: i64 = self.conn.query_row("SELECT COALESCE(SUM(size), 0) FROM entries", [], |r| r.get(0)).map_err(db_error)?;
        Ok(total.max(0) as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn het_ttl_thi_mat() {
        let cache = Cache::memory().unwrap();
        cache.set_at("a", "1", Some(10), 1_000).unwrap();
        cache.set_at("b", "2", None, 1_000).unwrap();
        assert_eq!(cache.get_at("a", 10_999).unwrap().as_deref(), Some("1"));
        assert_eq!(cache.get_at("a", 11_000).unwrap(), None);
        assert_eq!(cache.total_bytes().unwrap(), 2, "mục hết hạn bị xoá khi đọc");
        assert_eq!(cache.get_at("b", i64::MAX).unwrap().as_deref(), Some("2"));
        cache.set_at("b", "22", Some(1), 1_000).unwrap();
        assert_eq!(cache.get_at("b", 1_500).unwrap().as_deref(), Some("22"), "ghi đè cập nhật giá trị và ttl");
        cache.delete("b").unwrap();
        assert_eq!(cache.get("b").unwrap(), None);
    }

    #[test]
    fn day_thi_xoa_muc_dung_lau_nhat() {
        let cache = Cache::memory().unwrap().with_quota(30);
        for key in ["a", "b", "c"] {
            cache.set(key, "123456789", None).unwrap();
        }
        assert_eq!(cache.total_bytes().unwrap(), 30);
        assert!(cache.get("a").unwrap().is_some());
        cache.set("d", "123456789", None).unwrap();
        assert_eq!(cache.get("b").unwrap(), None, "b dùng lâu nhất");
        for key in ["a", "c", "d"] {
            assert!(cache.get(key).unwrap().is_some(), "{key}");
        }
    }

    #[test]
    fn het_han_bi_xoa_truoc_khi_day() {
        let cache = Cache::memory().unwrap().with_quota(21);
        cache.set_at("a", "123456789", None, 0).unwrap();
        cache.set_at("cu", "123456789", Some(1), 0).unwrap();
        cache.set_at("b", "123456789", None, 5_000).unwrap();
        assert_eq!(cache.get_at("a", 5_000).unwrap().as_deref(), Some("123456789"), "mục hết hạn bị xoá trước, a dù dùng lâu nhất vẫn còn");
    }

    #[test]
    fn tu_choi_khoa_va_gia_tri_qua_co() {
        let cache = Cache::memory().unwrap();
        assert!(cache.set("", "x", None).is_err());
        assert!(cache.get(&"k".repeat(MAX_KEY + 1)).is_err());
        assert!(cache.set("big", &"x".repeat(MAX_CACHE_ENTRY), None).unwrap_err().to_string().contains("5 MB"));
    }

    #[test]
    fn mo_lai_file_con_du_lieu() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("x/cache.sqlite");
        Cache::open(&path).unwrap().set("k", "{\"a\":1}", None).unwrap();
        let again = Cache::open(&path).unwrap();
        assert_eq!(again.get("k").unwrap().as_deref(), Some("{\"a\":1}"));
        let other = Cache::open(&path).unwrap();
        other.set("k2", "v", None).unwrap();
        assert_eq!(again.get("k2").unwrap().as_deref(), Some("v"), "hai kết nối cùng file");
    }
}
