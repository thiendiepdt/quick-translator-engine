//! Một lượt chạy extension (spec mục 9): thread riêng với tokio current-thread và một `Engine` mới. Caller gọi
//! đồng bộ; kết quả từng chương gửi về ngay khi xong để caller ghi file luôn. Biến JS dùng chung trong cả lượt.
//!
//! Auth và phiên (spec mục 5, 7): lượt tự nạp auth từ `ExtStore`, chèn `ctx.auth` và `ctx.session` vào mọi lệnh.
//! Extension có `login` mà phiên thiếu hoặc hết hạn thì gọi `login` trước. Lệnh ném `AuthExpired` thì xoá phiên
//! (login và cookie), gọi `login` lại rồi chạy lại đúng các lệnh đó một lần.

use crate::cookies::Jar;
use crate::engine::{Engine, Stores};
use crate::manifest::SettingKind;
use crate::store::{AuthValues, ExtStore, LoginRecord, SessionData};
use crate::{EngineOptions, ExtError, HostPolicy, HttpClient, Manifest, Package};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

/// Số chương mỗi lần gọi hàm gộp `chapters`.
pub const BATCH_SIZE: usize = 20;
const MAX_BATCH_LIMIT: Duration = Duration::from_secs(600);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TocItem {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub locked: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume: Option<String>,
    /// Dữ liệu riêng của extension, trả lại nguyên vẹn khi gọi `chapter`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChapterFormat {
    Text,
    Html,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChapterContent {
    pub content: String,
    pub title: Option<String>,
    pub format: ChapterFormat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChapterOutcome {
    pub id: String,
    /// Vị trí trong mục lục, từ 1.
    pub index: usize,
    pub result: Result<ChapterContent, ExtError>,
}

#[derive(Clone)]
pub struct SessionOptions {
    pub engine: EngineOptions,
    /// Chế độ developer: cho gọi `localhost` / IP nội bộ đã khai trong `hosts`.
    pub allow_local: bool,
    pub toc_limit: Duration,
    pub chapter_limit: Duration,
    pub login_limit: Duration,
    /// Dữ liệu của extension (auth, cache, session data). `None` = không auth; cache và session chỉ trong bộ nhớ.
    pub store: Option<ExtStore>,
}

impl Default for SessionOptions {
    fn default() -> Self {
        SessionOptions {
            engine: EngineOptions::default(),
            allow_local: false,
            toc_limit: Duration::from_secs(300),
            chapter_limit: Duration::from_secs(120),
            login_limit: Duration::from_secs(120),
            store: None,
        }
    }
}

enum Command {
    Toc { ctx: Value, reply: mpsc::Sender<Result<Vec<TocItem>, ExtError>> },
    Chapters { ctx: Value, items: Vec<(usize, TocItem)>, reply: mpsc::Sender<ChapterOutcome> },
    Login { ctx: Value, reply: mpsc::Sender<Result<Option<LoginRecord>, ExtError>> },
}

pub struct Session {
    commands: tokio::sync::mpsc::UnboundedSender<Command>,
    cancel: Arc<AtomicBool>,
    thread: Mutex<Option<JoinHandle<()>>>,
}

fn bad(message: String) -> ExtError {
    ExtError::BadResult(message)
}

fn parse_toc(value: Value) -> Result<Vec<TocItem>, ExtError> {
    let Value::Array(list) = value else {
        return Err(bad("toc phải trả về một mảng".into()));
    };
    let mut seen = HashSet::new();
    let mut items = Vec::with_capacity(list.len());
    for (i, entry) in list.into_iter().enumerate() {
        let Value::Object(mut object) = entry else {
            return Err(bad(format!("toc[{i}] phải là object")));
        };
        let id = match object.remove("id") {
            Some(Value::String(s)) => s.trim().to_string(),
            Some(Value::Number(n)) => n.to_string(),
            _ => String::new(),
        };
        if id.is_empty() {
            return Err(bad(format!("toc[{i}].id phải là chuỗi hoặc số không rỗng")));
        }
        if !seen.insert(id.clone()) {
            return Err(bad(format!("toc[{i}].id \"{id}\" bị trùng")));
        }
        let title = match object.remove("title") {
            Some(Value::String(s)) => s.trim().to_string(),
            _ => return Err(bad(format!("toc[{i}].title phải là chuỗi"))),
        };
        let locked = match object.remove("locked") {
            None | Some(Value::Null) => false,
            Some(Value::Bool(b)) => b,
            _ => return Err(bad(format!("toc[{i}].locked phải là true/false"))),
        };
        let volume = match object.remove("volume") {
            None | Some(Value::Null) => None,
            Some(Value::String(s)) => Some(s.trim().to_string()).filter(|s| !s.is_empty()),
            _ => return Err(bad(format!("toc[{i}].volume phải là chuỗi"))),
        };
        let data = object.remove("data").filter(|v| !v.is_null());
        items.push(TocItem { id, title, locked, volume, data });
    }
    Ok(items)
}

fn parse_content(value: Value, at: &str) -> Result<ChapterContent, ExtError> {
    let Value::Object(object) = value else {
        return Err(bad(format!("{at} phải trả về object {{ content }}")));
    };
    let content = match object.get("content") {
        Some(Value::String(s)) => s.clone(),
        _ => return Err(bad(format!("{at}.content phải là chuỗi"))),
    };
    let title = match object.get("title") {
        None | Some(Value::Null) => None,
        Some(Value::String(s)) => Some(s.trim().to_string()).filter(|s| !s.is_empty()),
        _ => return Err(bad(format!("{at}.title phải là chuỗi"))),
    };
    let format = match object.get("format") {
        None | Some(Value::Null) => ChapterFormat::Text,
        Some(Value::String(s)) if s == "text" => ChapterFormat::Text,
        Some(Value::String(s)) if s == "html" => ChapterFormat::Html,
        _ => return Err(bad(format!("{at}.format chỉ có \"text\" hoặc \"html\""))),
    };
    Ok(ChapterContent { content, title, format })
}

/// `expiresAt` nhỏ hơn mốc này coi là giây (Date.now() / 1000), lớn hơn là mili giây.
const SECONDS_BELOW: f64 = 1e11;

fn parse_login(value: Value) -> Result<LoginRecord, ExtError> {
    let object = match value {
        Value::Null => return Ok(LoginRecord { token: None, expires_at: None, data: Value::Null }),
        Value::Object(object) => object,
        _ => return Err(ExtError::LoginFailed("login phải trả về object { token, expiresAt?, data? }".into())),
    };
    let token = match object.get("token") {
        None | Some(Value::Null) => None,
        Some(Value::String(s)) => Some(s.clone()).filter(|s| !s.is_empty()),
        Some(Value::Number(n)) => Some(n.to_string()),
        Some(_) => return Err(ExtError::LoginFailed("login: token phải là chuỗi".into())),
    };
    let expires_at = match object.get("expiresAt") {
        None | Some(Value::Null) => None,
        Some(Value::Number(n)) => {
            let at = n.as_f64().unwrap_or(0.0);
            Some(if at < SECONDS_BELOW { at * 1000.0 } else { at } as i64)
        }
        Some(_) => return Err(ExtError::LoginFailed("login: expiresAt phải là số (Date.now() + …)".into())),
    };
    Ok(LoginRecord { token, expires_at, data: object.get("data").cloned().unwrap_or(Value::Null) })
}

/// Lỗi của `login` → lỗi báo người dùng. Huỷ và "cần mở app" giữ nguyên.
fn login_error(error: ExtError) -> ExtError {
    match error {
        ExtError::Cancelled | ExtError::NeedsInteraction(_) | ExtError::LoginFailed(_) => error,
        ExtError::AuthExpired(message) | ExtError::Extension { message, .. } => ExtError::LoginFailed(message),
        other => ExtError::LoginFailed(other.to_string()),
    }
}

fn item_arg(index: usize, item: &TocItem) -> Value {
    let mut value = serde_json::to_value(item).expect("TocItem → JSON");
    value["index"] = json!(index);
    value
}

type OnChapter<'a> = dyn FnMut(usize, &TocItem, Result<ChapterContent, ExtError>) + 'a;

async fn run_chapters(engine: &Engine, opts: &SessionOptions, concurrency: usize, ctx: Value, items: Vec<(usize, TocItem)>, send: &mut OnChapter<'_>) {
    if engine.has_export("chapters") {
        let groups: Vec<Vec<(usize, TocItem)>> = items.chunks(BATCH_SIZE).map(<[_]>::to_vec).collect();
        let calls: Vec<Vec<Value>> = groups.iter().map(|g| vec![ctx.clone(), Value::Array(g.iter().map(|(i, t)| item_arg(*i, t)).collect())]).collect();
        let limit = (opts.chapter_limit * BATCH_SIZE as u32).min(MAX_BATCH_LIMIT);
        engine
            .call_many("chapters", calls, concurrency, limit, &mut |g, result| {
                let group = &groups[g];
                match result {
                    Err(error) => group.iter().for_each(|(i, t)| send(*i, t, Err(error.clone()))),
                    Ok(Value::Array(list)) => {
                        let mut by_id: HashMap<String, Value> = HashMap::new();
                        for entry in list {
                            let id = match entry.get("id") {
                                Some(Value::String(s)) => s.clone(),
                                Some(Value::Number(n)) => n.to_string(),
                                _ => continue,
                            };
                            by_id.insert(id, entry);
                        }
                        for (i, t) in group {
                            let result = match by_id.remove(&t.id) {
                                None => Err(ExtError::Extension { message: "hàm chapters không trả chương này".into(), retry: true }),
                                Some(entry) => match entry.get("error") {
                                    Some(error) if !error.is_null() => {
                                        let message = error.as_str().map(str::to_string).unwrap_or_else(|| error.to_string());
                                        Err(ExtError::Extension { message, retry: true })
                                    }
                                    _ => parse_content(entry, &format!("chapters[{}]", t.id)),
                                },
                            };
                            send(*i, t, result);
                        }
                    }
                    Ok(_) => group.iter().for_each(|(i, t)| send(*i, t, Err(bad("chapters phải trả về một mảng".into())))),
                }
            })
            .await;
    } else {
        let calls: Vec<Vec<Value>> = items.iter().map(|(i, t)| vec![ctx.clone(), item_arg(*i, t)]).collect();
        engine
            .call_many("chapter", calls, concurrency, opts.chapter_limit, &mut |k, result| {
                let (index, item) = &items[k];
                send(*index, item, result.and_then(|v| parse_content(v, "chapter")));
            })
            .await;
    }
}

/// Phần chạy trên thread của lượt: engine, auth, phiên.
struct Runner<'a> {
    engine: Engine,
    opts: &'a SessionOptions,
    manifest: &'a Manifest,
    auth: Option<AuthValues>,
}

impl Runner<'_> {
    fn session(&self) -> &SessionData {
        &self.engine.state().session
    }

    fn has_login(&self) -> bool {
        self.engine.has_export("login")
    }

    /// `ctx` của caller + `auth` + `session` (phiên login còn hạn, hoặc null).
    fn ctx(&self, base: &Value, with_session: bool) -> Value {
        let mut ctx = base.clone();
        if let Value::Object(map) = &mut ctx {
            map.insert("auth".into(), self.auth.as_ref().map(AuthValues::to_json).unwrap_or(Value::Null));
            let session = if with_session { self.session().login().map(|l| l.to_json()) } else { None };
            map.insert("session".into(), session.unwrap_or(Value::Null));
        }
        let secret_settings: Vec<String> = self
            .manifest
            .settings
            .iter()
            .filter(|f| f.kind == SettingKind::Secret)
            .filter_map(|f| ctx.get("settings").and_then(|s| s.get(&f.key)).and_then(Value::as_str).map(str::to_string))
            .collect();
        self.engine.state().add_secrets(secret_settings);
        ctx
    }

    /// Có `login` mà phiên thiếu, hết hạn hoặc `force` thì gọi `login`. `None` = extension không có `login`.
    async fn login(&self, base: &Value, force: bool) -> Result<Option<LoginRecord>, ExtError> {
        if !self.has_login() {
            return Ok(None);
        }
        if !force {
            if let Some(record) = self.session().login() {
                return Ok(Some(record));
            }
        }
        let value = self.engine.call("login", vec![self.ctx(base, false)], self.opts.login_limit).await.map_err(login_error)?;
        let record = parse_login(value)?;
        self.engine.state().add_secrets(record.token.clone());
        self.session().set_login(record.clone())?;
        Ok(Some(record))
    }

    /// Sau `AuthExpired`: xoá phiên (login và cookie, giữ giá trị `session.set`) rồi đăng nhập lại.
    async fn relogin(&self, base: &Value) -> Result<(), ExtError> {
        self.engine.state().http.jar().clear();
        self.session().clear_login()?;
        self.login(base, true).await.map(|_| ())
    }

    fn can_relogin(&self, error: &ExtError) -> bool {
        matches!(error, ExtError::AuthExpired(_)) && self.has_login() && !self.engine.state().cancelled()
    }

    async fn toc(&self, base: &Value) -> Result<Vec<TocItem>, ExtError> {
        self.login(base, false).await?;
        match self.engine.call("toc", vec![self.ctx(base, true)], self.opts.toc_limit).await {
            Err(error) if self.can_relogin(&error) => {
                self.relogin(base).await?;
                self.engine.call("toc", vec![self.ctx(base, true)], self.opts.toc_limit).await.and_then(parse_toc)
            }
            result => result.and_then(parse_toc),
        }
    }

    async fn chapters(&self, base: &Value, items: Vec<(usize, TocItem)>, reply: &mpsc::Sender<ChapterOutcome>) {
        let send = |index: usize, item: &TocItem, result: Result<ChapterContent, ExtError>| {
            let _ = reply.send(ChapterOutcome { id: item.id.clone(), index, result });
        };
        if let Err(error) = self.login(base, false).await {
            items.iter().for_each(|(i, t)| send(*i, t, Err(error.clone())));
            return;
        }
        let concurrency = self.manifest.limits.concurrency;
        let mut expired: Vec<(usize, TocItem)> = Vec::new();
        run_chapters(&self.engine, self.opts, concurrency, self.ctx(base, true), items, &mut |index, item, result| match result {
            Err(error) if self.can_relogin(&error) => expired.push((index, item.clone())),
            result => send(index, item, result),
        })
        .await;
        if expired.is_empty() {
            return;
        }
        if let Err(error) = self.relogin(base).await {
            expired.iter().for_each(|(i, t)| send(*i, t, Err(error.clone())));
            return;
        }
        run_chapters(&self.engine, self.opts, concurrency, self.ctx(base, true), expired, &mut |index, item, result| send(index, item, result)).await;
    }
}

/// Dựng engine của lượt: auth và session data từ `ExtStore` (nếu có), hũ cookie nạp từ session data.
async fn build_runner<'a>(package: &'a Package, opts: &'a SessionOptions) -> Result<Runner<'a>, ExtError> {
    let manifest = &package.manifest;
    let store = opts.store.as_ref();
    let auth = match store {
        Some(store) => store.auth(manifest)?,
        None => None,
    };
    if manifest.auth.as_ref().is_some_and(|a| a.required) && auth.is_none() {
        return Err(ExtError::AuthMissing);
    }
    let version = store.map(ExtStore::auth_version).unwrap_or_default();
    let session = Rc::new(SessionData::open(store.map(ExtStore::session_path), &version));
    let jar = Arc::new(Jar::from_cookies(session.cookies()));
    let http = Rc::new(HttpClient::with_jar(HostPolicy::new(&manifest.hosts, opts.allow_local), manifest.limits, jar)?);
    let mut engine_opts = opts.engine.clone();
    engine_opts.secrets.extend(auth.iter().flat_map(|a| a.secret_values(manifest)));
    engine_opts.secrets.extend(session.login().and_then(|l| l.token));
    let stores = Stores { cache: store.map(ExtStore::cache_path), session };
    let engine = Engine::new(package, http, engine_opts, stores).await?;
    Ok(Runner { engine, opts, manifest, auth })
}

impl Session {
    pub fn start(package: Package, opts: SessionOptions) -> Result<Session, ExtError> {
        let cancel = opts.engine.cancel.clone();
        let (commands, mut receiver) = tokio::sync::mpsc::unbounded_channel::<Command>();
        let (ready_tx, ready_rx) = mpsc::channel::<Result<(), ExtError>>();
        let thread = std::thread::Builder::new()
            .name(format!("qt-ext {}", package.manifest.id))
            .spawn(move || {
                let runtime = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
                    Ok(rt) => rt,
                    Err(e) => {
                        let _ = ready_tx.send(Err(ExtError::Script(e.to_string())));
                        return;
                    }
                };
                runtime.block_on(async move {
                    let runner = match build_runner(&package, &opts).await {
                        Ok(runner) => runner,
                        Err(e) => {
                            let _ = ready_tx.send(Err(e));
                            return;
                        }
                    };
                    let _ = ready_tx.send(Ok(()));
                    while let Some(command) = receiver.recv().await {
                        match command {
                            Command::Toc { ctx, reply } => {
                                let _ = reply.send(runner.toc(&ctx).await);
                            }
                            Command::Chapters { ctx, items, reply } => runner.chapters(&ctx, items, &reply).await,
                            Command::Login { ctx, reply } => {
                                let _ = reply.send(runner.login(&ctx, true).await);
                            }
                        }
                    }
                });
            })
            .map_err(|e| ExtError::Script(e.to_string()))?;
        match ready_rx.recv() {
            Ok(Ok(())) => Ok(Session { commands, cancel, thread: Mutex::new(Some(thread)) }),
            Ok(Err(e)) => {
                let _ = thread.join();
                Err(e)
            }
            Err(_) => Err(ExtError::Script("luồng extension dừng bất ngờ".into())),
        }
    }

    /// Đăng nhập lại ngay (nút "Đăng nhập"). `None` = extension không có hàm `login`.
    pub fn login(&self, ctx: &Value) -> Result<Option<LoginRecord>, ExtError> {
        let (reply, result) = mpsc::channel();
        self.commands.send(Command::Login { ctx: ctx.clone(), reply }).map_err(|_| ExtError::Cancelled)?;
        result.recv().unwrap_or(Err(ExtError::Cancelled))
    }

    pub fn toc(&self, ctx: &Value) -> Result<Vec<TocItem>, ExtError> {
        let (reply, result) = mpsc::channel();
        self.commands.send(Command::Toc { ctx: ctx.clone(), reply }).map_err(|_| ExtError::Cancelled)?;
        result.recv().unwrap_or(Err(ExtError::Cancelled))
    }

    /// `items`: (vị trí trong mục lục từ 1, mục). `on_each` chạy trên thread của caller, đúng một lần cho mỗi mục.
    pub fn chapters(&self, ctx: &Value, items: &[(usize, TocItem)], on_each: &mut dyn FnMut(ChapterOutcome)) {
        let (reply, results) = mpsc::channel();
        let mut remaining: HashMap<String, usize> = items.iter().map(|(i, t)| (t.id.clone(), *i)).collect();
        if self.commands.send(Command::Chapters { ctx: ctx.clone(), items: items.to_vec(), reply }).is_ok() {
            for outcome in results.iter() {
                remaining.remove(&outcome.id);
                on_each(outcome);
            }
        }
        // Luồng extension chết giữa chừng: mục chưa có kết quả coi như bị huỷ.
        let mut rest: Vec<(String, usize)> = remaining.into_iter().collect();
        rest.sort_by_key(|(_, i)| *i);
        for (id, index) in rest {
            on_each(ChapterOutcome { id, index, result: Err(ExtError::Cancelled) });
        }
    }

    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.cancel();
        let (closed, _) = tokio::sync::mpsc::unbounded_channel();
        drop(std::mem::replace(&mut self.commands, closed));
        if let Some(thread) = self.thread.lock().ok().and_then(|mut t| t.take()) {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kiem_ket_qua_toc_va_chapter() {
        let items = parse_toc(json!([{ "id": 7, "title": " a ", "locked": true, "volume": "", "data": { "u": 1 } }, { "id": "8", "title": "b" }])).unwrap();
        assert_eq!(items[0], TocItem { id: "7".into(), title: "a".into(), locked: true, volume: None, data: Some(json!({ "u": 1 })) });
        assert!(parse_toc(json!({})).is_err());
        assert!(parse_toc(json!([{ "id": "", "title": "a" }])).unwrap_err().to_string().contains("toc[0].id"));
        assert!(parse_toc(json!([{ "id": "1", "title": 3 }])).unwrap_err().to_string().contains("title"));
        assert!(parse_toc(json!([{ "id": "1", "title": "a" }, { "id": 1, "title": "b" }])).unwrap_err().to_string().contains("trùng"));
        assert_eq!(parse_content(json!({ "content": "" }), "chapter").unwrap().content, "", "nội dung trống để mốc 2 quyết");
        assert!(parse_content(json!({ "content": "x", "format": "md" }), "chapter").is_err());
        assert!(parse_content(json!("x"), "chapter").is_err());
    }

    #[test]
    fn doc_ket_qua_login() {
        let record = parse_login(json!({ "token": "a", "expiresAt": 1_700_000_000, "data": { "u": 1 } })).unwrap();
        assert_eq!((record.token.as_deref(), record.expires_at, record.data), (Some("a"), Some(1_700_000_000_000), json!({ "u": 1 })), "giây → ms");
        assert_eq!(parse_login(json!({ "expiresAt": 1_700_000_000_000i64 })).unwrap().expires_at, Some(1_700_000_000_000));
        assert_eq!(parse_login(Value::Null).unwrap().token, None, "login chỉ đặt cookie thì không trả gì");
        assert!(parse_login(json!("tok")).is_err());
        assert!(parse_login(json!({ "expiresAt": "mai" })).is_err());
        assert_eq!(login_error(ExtError::Extension { message: "sai".into(), retry: true }), ExtError::LoginFailed("sai".into()));
        assert_eq!(login_error(ExtError::Cancelled), ExtError::Cancelled);
    }
}
