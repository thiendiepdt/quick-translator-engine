//! Bảng thành ngữ (logic ở `qt_ai_core::idioms`): tải bảng chung từ GitHub mỗi lần mở app, xem trạng thái, sửa
//! bảng của người dùng (đè bảng chung theo khoá, rỗng = tắt mục chung).

use crate::error::{blocking, CmdResult};
use qt_ai_core::base::BaseStore;
use qt_ai_core::idioms::{refresh_remote, IdiomTable, RemoteSource, REMOTE_URL};
use serde::Serialize;
use std::sync::Mutex;

/// Lỗi lần tải gần nhất trong phiên app này (None = chưa tải hoặc tải được).
static LAST_ERROR: Mutex<Option<String>> = Mutex::new(None);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IdiomsInfo {
    pub remote: IdiomTable,
    /// "cache" = bản đã tải, "builtin" = bản đóng trong app (chưa tải được lần nào).
    pub source: &'static str,
    pub fetched_at: Option<u64>,
    pub error: Option<String>,
    pub user: IdiomTable,
    pub url: &'static str,
}

fn info(store: &BaseStore) -> IdiomsInfo {
    let (remote, source) = store.idioms_remote();
    IdiomsInfo {
        remote,
        source: match source {
            RemoteSource::Cache => "cache",
            RemoteSource::Builtin => "builtin",
        },
        fetched_at: store.idioms_fetched_at(),
        error: LAST_ERROR.lock().unwrap().clone(),
        user: store.idioms_user(),
        url: REMOTE_URL,
    }
}

/// Tải bảng chung, nhớ lỗi để Cài đặt hiện; lỗi thì bản cũ giữ nguyên.
pub fn refresh(store: &BaseStore) -> Result<usize, String> {
    let result = refresh_remote(store, REMOTE_URL).map_err(|error| error.to_string());
    *LAST_ERROR.lock().unwrap() = result.as_ref().err().cloned();
    result
}

/// Lúc mở app: tải nền, không chặn khởi động; phiên dịch mở trước khi tải xong dùng bản cũ.
pub fn refresh_in_background() {
    std::thread::spawn(|| {
        let _ = refresh(&BaseStore::from_env());
    });
}

#[tauri::command]
pub async fn idioms_get() -> CmdResult<IdiomsInfo> {
    blocking(|| Ok(info(&BaseStore::from_env()))).await
}

/// Nút "Tải lại": tải ngay; lỗi vẫn trả trạng thái (kèm `error`) để giao diện hiện.
#[tauri::command]
pub async fn idioms_refresh() -> CmdResult<IdiomsInfo> {
    blocking(|| {
        let store = BaseStore::from_env();
        let _ = refresh(&store);
        Ok(info(&store))
    })
    .await
}

#[tauri::command]
pub async fn idioms_save_user(table: IdiomTable) -> CmdResult<IdiomsInfo> {
    blocking(move || {
        let store = BaseStore::from_env();
        let table: IdiomTable =
            table.into_iter().map(|(key, value)| (key.trim().to_string(), value.trim().to_string())).filter(|(key, _)| !key.is_empty()).collect();
        store.save_idioms_user(&table)?;
        Ok(info(&store))
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn luu_bang_nguoi_dung_bo_khoa_rong_va_tra_trang_thai() {
        let dir = tempfile::tempdir().unwrap();
        let store = BaseStore::at(dir.path());
        let before = info(&store);
        assert_eq!(before.source, "builtin");
        assert!(before.remote.contains_key("打草惊蛇"));
        store.save_idioms_user(&IdiomTable::from([("对牛弹琴".to_string(), "đàn gảy tai trâu".to_string())])).unwrap();
        assert_eq!(info(&store).user.len(), 1);
    }
}
