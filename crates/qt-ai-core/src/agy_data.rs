//! Dọn dữ liệu agy CLI để lại trên đĩa. Mỗi lượt `agy -p` sinh một hội thoại `conversations/<id>.db` (SQLite,
//! vài MB) kèm thư mục `brain/<id>/` trong thư mục dữ liệu của agy và agy không bao giờ tự xoá — dịch vài
//! ngày là vài GB. Hội thoại do app sinh ra nhận ra được từ chính file đó: lượt đầu tiên là prompt của app
//! (`APP_PROMPT_MARK`), hoặc hội thoại rỗng (agy chết trước lượt đầu) mà workspace là một folder truyện.
//! Hội thoại người dùng tự chat bằng agy nằm chung thư mục và không bao giờ bị đụng tới.

use crate::story_fs::story_paths;
use rusqlite::{Connection, OpenFlags, OptionalExtension};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Đoạn cố định trong mọi prompt app gửi cho agy (`build_translate_prompt`, prompt "AI điền hồ sơ" của GUI),
/// đứng ngay sau đường dẫn file workflow. Các hàm dựng prompt phải ghép từ hằng này để việc nhận diện không lệch.
pub const APP_PROMPT_MARK: &str = " (đường dẫn tuyệt đối, tồn tại sẵn, KHÔNG cần tìm kiếm) rồi làm đúng theo nó";
const APP_PROMPT_HEAD: &str = "Mở file ";

/// Hội thoại rỗng, và mọi hội thoại khi dọn cả thư mục, phải nằm yên chừng này mới bị xoá — phiên agy đang
/// chạy ở chỗ khác (app khác, terminal) còn đang ghi vào file của nó.
pub const IDLE_GRACE: Duration = Duration::from_secs(600);

/// `~/.gemini/antigravity-cli` nếu có trên máy.
pub fn default_data_dir() -> Option<PathBuf> {
    let home = std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" })?;
    let dir = PathBuf::from(home).join(".gemini").join("antigravity-cli");
    dir.is_dir().then_some(dir)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conversation {
    pub id: String,
    /// Do app sinh ra (xem đầu file). false = của người dùng hoặc không đọc được — giữ nguyên.
    pub app: bool,
    /// Chưa có lượt nào.
    pub empty: bool,
    /// Thư mục agy chạy trong đó — với hội thoại của app là folder truyện.
    pub workspace: Option<PathBuf>,
    /// Thời gian từ lần ghi cuối vào file hội thoại.
    pub idle: Duration,
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}

fn percent_decode(text: &[u8]) -> String {
    let mut out = Vec::with_capacity(text.len());
    let mut i = 0;
    while i < text.len() {
        let hex = (text[i] == b'%').then(|| text.get(i + 1..i + 3)).flatten();
        match hex.and_then(|h| std::str::from_utf8(h).ok()).and_then(|h| u8::from_str_radix(h, 16).ok()) {
            Some(byte) => {
                out.push(byte);
                i += 3;
            }
            None => {
                out.push(text[i]);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// URI workspace đầu tiên trong blob protobuf `trajectory_metadata_blob.data` → đường dẫn. Chuỗi trong
/// protobuf đứng sau varint độ dài; không khớp thì cắt ở byte điều khiển đầu tiên.
fn workspace_from_blob(blob: &[u8]) -> Option<PathBuf> {
    const SCHEME: &[u8] = b"file://";
    let start = find(blob, b"file:///")?;
    let rest = &blob[start..];
    let mut end = rest.iter().position(|byte| *byte < 0x20).unwrap_or(rest.len());
    if start >= 1 && blob[start - 1] < 0x80 {
        let mut len = blob[start - 1] as usize;
        if start >= 2 && blob[start - 2] >= 0x80 {
            len = (len << 7) | (blob[start - 2] & 0x7f) as usize;
        }
        if len > SCHEME.len() && len <= end {
            end = len;
        }
    }
    let path = percent_decode(&rest[SCHEME.len()..end]);
    // `/D:/Work/x` → `D:/Work/x`; đường dẫn Unix giữ dấu `/` đầu.
    let bytes = path.as_bytes();
    let drive = bytes.len() >= 3 && bytes[0] == b'/' && bytes[1].is_ascii_alphabetic() && bytes[2] == b':';
    let path = if drive { &path[1..] } else { &path[..] };
    (!path.is_empty()).then(|| PathBuf::from(path))
}

/// So đường dẫn bỏ qua dấu `/` `\` và dấu cuối; Windows bỏ qua cả hoa thường.
fn path_key(path: &Path) -> String {
    let text = path.to_string_lossy().replace('\\', "/");
    let text = text.trim_end_matches('/');
    if cfg!(windows) {
        text.to_lowercase()
    } else {
        text.to_string()
    }
}

fn same_path(a: &Path, b: &Path) -> bool {
    path_key(a) == path_key(b)
}

fn is_conversation_id(id: &str) -> bool {
    id.len() == 36 && id.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
}

fn sibling(db: &Path, suffix: &str) -> PathBuf {
    let mut name = db.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

/// `file:///C:/x/y.db?immutable=1` — SQLite đọc thẳng file chính, không khoá, không đụng -wal/-shm.
fn immutable_uri(db: &Path) -> String {
    let mut path = db.to_string_lossy().replace('\\', "/").replace('%', "%25").replace('?', "%3f").replace('#', "%23");
    if !path.starts_with('/') {
        path.insert(0, '/');
    }
    format!("file://{path}?immutable=1")
}

/// Mở file hội thoại mà không để lại dấu vết trong thư mục của agy. File của agy ở chế độ WAL: mở chỉ-đọc
/// kiểu thường vẫn khiến SQLite tạo `-wal`/`-shm` rỗng cạnh từng file và ghi vào `-shm`. Nên: không có WAL
/// (agy đã đóng hội thoại gọn gàng) thì mở `immutable`; còn WAL có dữ liệu (agy đang chạy hoặc bị giết giữa
/// chừng) thì phải đọc qua WAL, và chỉ làm vậy khi `-shm` đã có sẵn để không tạo thêm file nào.
fn open_quietly(db: &Path) -> Option<Connection> {
    let read_only = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let wal_bytes = fs::metadata(sibling(db, "-wal")).map(|meta| meta.len()).unwrap_or(0);
    if wal_bytes == 0 {
        return Connection::open_with_flags(immutable_uri(db), read_only | OpenFlags::SQLITE_OPEN_URI).ok();
    }
    if !sibling(db, "-shm").is_file() {
        return None;
    }
    let conn = Connection::open_with_flags(db, read_only).ok()?;
    let _ = conn.busy_timeout(Duration::from_millis(200));
    Some(conn)
}

/// (payload lượt đầu — None nếu chưa có lượt nào, workspace) của một hội thoại. Không mở/đọc được (agy đang
/// khoá, schema lạ) → None, hội thoại được coi là không phải của app.
fn inspect(db: &Path) -> Option<(Option<Vec<u8>>, Option<PathBuf>)> {
    let conn = open_quietly(db)?;
    let first = conn
        .query_row("select step_payload from steps order by idx limit 1", [], |row| row.get::<_, Option<Vec<u8>>>(0))
        .optional()
        .ok()?
        .map(Option::unwrap_or_default);
    let workspace = conn
        .query_row("select data from trajectory_metadata_blob limit 1", [], |row| row.get::<_, Option<Vec<u8>>>(0))
        .optional()
        .ok()
        .flatten()
        .flatten()
        .and_then(|blob| workspace_from_blob(&blob));
    Some((first, workspace))
}

fn conversation_files(data_dir: &Path, id: &str) -> [PathBuf; 3] {
    let dir = data_dir.join("conversations");
    [dir.join(format!("{id}.db")), dir.join(format!("{id}.db-wal")), dir.join(format!("{id}.db-shm"))]
}

/// Tính theo .db và -wal; -shm không tính vì cả tiến trình chỉ đọc cũng ghi vào nó.
fn idle_of(data_dir: &Path, id: &str) -> Duration {
    let newest = conversation_files(data_dir, id)[..2]
        .iter()
        .filter_map(|path| fs::metadata(path).and_then(|meta| meta.modified()).ok())
        .max();
    newest.and_then(|time| SystemTime::now().duration_since(time).ok()).unwrap_or(Duration::ZERO)
}

/// Mọi hội thoại trong thư mục dữ liệu agy, đã phân loại.
pub fn scan(data_dir: &Path) -> Vec<Conversation> {
    let Ok(entries) = fs::read_dir(data_dir.join("conversations")) else { return vec![] };
    let mut ids: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().to_string_lossy().strip_suffix(".db").map(str::to_string))
        .filter(|id| is_conversation_id(id))
        .collect();
    ids.sort();
    ids.into_iter()
        .map(|id| {
            let inspected = inspect(&conversation_files(data_dir, &id)[0]);
            let (first, workspace) = match inspected {
                Some((first, workspace)) => (Some(first), workspace),
                None => (None, None),
            };
            let empty = matches!(first, Some(None));
            let marked = matches!(&first, Some(Some(payload))
                if find(payload, APP_PROMPT_HEAD.as_bytes()).is_some() && find(payload, APP_PROMPT_MARK.as_bytes()).is_some());
            let in_story = workspace.as_deref().is_some_and(|dir| story_paths(dir).state_json.is_file());
            Conversation { app: marked || (empty && in_story), empty, workspace, idle: idle_of(data_dir, &id), id }
        })
        .collect()
}

fn tree_size(path: &Path) -> u64 {
    let Ok(entries) = fs::read_dir(path) else { return 0 };
    entries
        .filter_map(|entry| entry.ok())
        .map(|entry| match entry.file_type() {
            Ok(kind) if kind.is_dir() => tree_size(&entry.path()),
            _ => entry.metadata().map(|meta| meta.len()).unwrap_or(0),
        })
        .sum()
}

/// Dung lượng một hội thoại: file SQLite (+ wal/shm) và `brain/<id>/`.
fn size_of(data_dir: &Path, id: &str) -> u64 {
    let files: u64 =
        conversation_files(data_dir, id).iter().filter_map(|path| fs::metadata(path).ok()).map(|meta| meta.len()).sum();
    files + tree_size(&data_dir.join("brain").join(id))
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataStats {
    pub app_count: usize,
    pub app_bytes: u64,
    /// Hội thoại không phải của app (người dùng tự chat, hoặc không đọc được) — không bao giờ bị xoá.
    pub other_count: usize,
    pub other_bytes: u64,
}

pub fn stats(data_dir: &Path) -> DataStats {
    let mut stats = DataStats::default();
    for conversation in scan(data_dir) {
        let bytes = size_of(data_dir, &conversation.id);
        if conversation.app {
            stats.app_count += 1;
            stats.app_bytes += bytes;
        } else {
            stats.other_count += 1;
            stats.other_bytes += bytes;
        }
    }
    stats
}

/// Phạm vi một lần dọn. Cả hai chỉ xoá hội thoại của app.
#[derive(Debug, Clone, Copy)]
pub enum Scope<'a> {
    /// Hội thoại của một truyện, gọi ngay sau khi lượt agy của truyện đó thoát.
    Story(&'a Path),
    /// Mọi truyện, trừ các folder đang có phiên chạy; chỉ lấy hội thoại đã nằm yên `IDLE_GRACE`.
    All { busy_roots: &'a [PathBuf] },
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanReport {
    pub removed: usize,
    pub freed_bytes: u64,
    /// Hội thoại của app được giữ lại lần này: truyện đang dịch, vừa ghi xong, hoặc agy còn giữ file.
    pub kept: usize,
}

fn wanted(conversation: &Conversation, scope: Scope<'_>) -> bool {
    let rested = conversation.idle >= IDLE_GRACE;
    match scope {
        Scope::Story(root) => {
            conversation.workspace.as_deref().is_some_and(|dir| same_path(dir, root)) && (!conversation.empty || rested)
        }
        Scope::All { busy_roots } => {
            let busy = conversation.workspace.as_deref().is_some_and(|dir| busy_roots.iter().any(|root| same_path(dir, root)));
            rested && !busy
        }
    }
}

/// Xoá file hội thoại rồi mới tới `brain/<id>/`: file .db còn bị agy giữ (Windows không cho xoá) thì để yên cả hai.
fn remove(data_dir: &Path, id: &str) -> bool {
    let [db, wal, shm] = conversation_files(data_dir, id);
    if fs::remove_file(&db).is_err() {
        return false;
    }
    let _ = fs::remove_file(wal);
    let _ = fs::remove_file(shm);
    let _ = fs::remove_dir_all(data_dir.join("brain").join(id));
    true
}

pub fn clean(data_dir: &Path, scope: Scope<'_>) -> CleanReport {
    let mut report = CleanReport::default();
    for conversation in scan(data_dir).into_iter().filter(|conversation| conversation.app) {
        if let Scope::Story(root) = scope {
            // Hội thoại của truyện khác không thuộc lượt dọn này — không tính là "giữ lại".
            if !conversation.workspace.as_deref().is_some_and(|dir| same_path(dir, root)) {
                continue;
            }
        }
        if !wanted(&conversation, scope) {
            report.kept += 1;
            continue;
        }
        let bytes = size_of(data_dir, &conversation.id);
        if remove(data_dir, &conversation.id) {
            report.removed += 1;
            report.freed_bytes += bytes;
        } else {
            report.kept += 1;
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_tu_blob_theo_do_dai_varint_va_giai_ma_phan_tram() {
        let uri = "file:///D:/Work/truy%E1%BB%87n";
        let mut blob = vec![0x0a, 0xb7, 0x01, 0x0a, uri.len() as u8];
        blob.extend_from_slice(uri.as_bytes());
        blob.extend_from_slice(b"\x12\"file:///D:/Work");
        assert_eq!(workspace_from_blob(&blob), Some(PathBuf::from("D:/Work/truyện")));
        // Độ dài 2 byte (URI ≥ 128 ký tự) và đường dẫn Unix giữ dấu / đầu.
        let long = format!("file:///home/u/{}", "a".repeat(140));
        let mut blob = vec![0x0a, (long.len() & 0x7f) as u8 | 0x80, (long.len() >> 7) as u8];
        blob.extend_from_slice(long.as_bytes());
        blob.extend_from_slice(b"\"tiep");
        assert_eq!(workspace_from_blob(&blob), Some(PathBuf::from(format!("/home/u/{}", "a".repeat(140)))));
        assert_eq!(workspace_from_blob(b"khong co uri"), None);
    }

    #[test]
    fn id_phai_la_uuid_de_khong_bao_gio_tro_ra_ngoai_thu_muc() {
        assert!(is_conversation_id("8ac483ef-1d78-4b97-b82c-56076f2ea818"));
        assert!(!is_conversation_id("") && !is_conversation_id("..") && !is_conversation_id("../../../../../../../../../../../x"));
    }
}
