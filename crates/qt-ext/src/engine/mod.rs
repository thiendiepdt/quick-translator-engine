//! Môi trường QuickJS cho một lượt chạy extension: giới hạn RAM, stack, thời gian JS chạy liên tục; module chỉ
//! từ gói; API app gắn qua `prelude.js`. Gọi hàm export của `main.js` và đổi lỗi JS sang `ExtError`.
//!
//! Một `Engine` chỉ dùng trên một thread (AsyncRuntime không `Send`) — `Session` lo phần thread.

pub(crate) mod bind_crypto;
pub(crate) mod bind_html;
pub(crate) mod bind_http;
pub(crate) mod bind_misc;
pub(crate) mod bind_store;
pub(crate) mod loader;

use crate::store::{Cache, SessionData};
use crate::{ExtError, HttpClient, Package};
use futures_util::stream::{FuturesUnordered, StreamExt};
use rquickjs::function::Args;
use rquickjs::promise::MaybePromise;
use rquickjs::{AsyncContext, AsyncRuntime, CatchResultExt, CaughtError, Ctx, Function, Module, Object, Value};
use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

const PRELUDE: &str = include_str!("prelude.js");
const STACK_LIMIT: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

pub type LogSink = Arc<dyn Fn(LogLevel, String) + Send + Sync>;
pub type ProgressSink = Arc<dyn Fn(u64, u64, Option<String>) + Send + Sync>;

#[derive(Clone)]
pub struct EngineOptions {
    pub memory_limit: usize,
    /// JS chạy liên tục (không chờ I/O của app) quá mức này thì bị ngắt.
    pub cpu_slice: Duration,
    pub cancel: Arc<AtomicBool>,
    /// Giá trị cần che (`***`) trong log và thông báo lỗi.
    pub secrets: Vec<String>,
    pub log: LogSink,
    pub progress: ProgressSink,
}

impl Default for EngineOptions {
    fn default() -> Self {
        EngineOptions {
            memory_limit: 256 * 1024 * 1024,
            cpu_slice: Duration::from_secs(30),
            cancel: Arc::new(AtomicBool::new(false)),
            secrets: Vec::new(),
            log: Arc::new(|_, _| {}),
            progress: Arc::new(|_, _, _| {}),
        }
    }
}

/// Thay mọi giá trị bí mật (từ 4 ký tự) bằng `***`.
pub fn redact(text: &str, secrets: &[String]) -> String {
    let mut out = text.to_string();
    for secret in secrets.iter().filter(|s| s.chars().count() >= 4) {
        out = out.replace(secret.as_str(), "***");
    }
    out
}

/// Tên hàm → nhãn trong `ExtError::Timeout`.
fn label(name: &str) -> &'static str {
    match name {
        "toc" => "toc",
        "chapter" => "chapter",
        "chapters" => "chapters",
        "login" => "login",
        _ => "call",
    }
}

/// Chỗ lưu `cache` và `session` của một lượt.
pub(crate) struct Stores {
    /// `None` = cache trong bộ nhớ, mất khi hết lượt.
    pub cache: Option<PathBuf>,
    pub session: Rc<SessionData>,
}

impl Stores {
    pub fn memory() -> Stores {
        Stores { cache: None, session: Rc::new(SessionData::open(None, "")) }
    }
}

/// Trạng thái dùng chung giữa engine và các hàm host.
pub(crate) struct HostState {
    pub activity: Rc<Cell<Instant>>,
    pub cancel: Arc<AtomicBool>,
    /// Giá trị cần che; thêm được giữa lượt (token của `login`).
    secrets: RefCell<Vec<String>>,
    pub log: LogSink,
    pub progress: ProgressSink,
    pub http: Rc<HttpClient>,
    pub session: Rc<SessionData>,
    cache_path: Option<PathBuf>,
    cache: RefCell<Option<Cache>>,
}

impl HostState {
    pub fn secrets(&self) -> Vec<String> {
        self.secrets.borrow().clone()
    }

    pub fn redact(&self, text: &str) -> String {
        redact(text, &self.secrets.borrow())
    }

    pub fn add_secrets(&self, values: impl IntoIterator<Item = String>) {
        let mut list = self.secrets.borrow_mut();
        for value in values {
            if !value.is_empty() && !list.contains(&value) {
                list.push(value);
            }
        }
        // Chuỗi dài trước để chuỗi chứa chuỗi khác vẫn che hết.
        list.sort_by_key(|s| std::cmp::Reverse(s.chars().count()));
    }

    /// Mở cache lần đầu dùng (file của extension, hoặc bộ nhớ).
    pub fn with_cache<T>(&self, f: impl FnOnce(&Cache) -> Result<T, ExtError>) -> Result<T, ExtError> {
        let mut slot = self.cache.borrow_mut();
        if slot.is_none() {
            *slot = Some(match &self.cache_path {
                Some(path) => Cache::open(path)?,
                None => Cache::memory()?,
            });
        }
        f(slot.as_ref().expect("vừa mở"))
    }

    /// Hũ cookie có cookie mới thì lưu vào session data.
    pub fn persist_cookies(&self) {
        let jar = self.http.jar();
        if jar.take_dirty() {
            if let Err(error) = self.session.set_cookies(jar.snapshot()) {
                (self.log)(LogLevel::Warn, format!("không lưu được cookie: {error}"));
            }
        }
    }

    pub fn touch(&self) {
        self.activity.set(Instant::now());
    }

    pub fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }

    /// Xong khi lượt bị huỷ — dùng để cắt ngang hàm host đang chờ I/O.
    pub async fn until_cancelled(cancel: Arc<AtomicBool>) {
        while !cancel.load(Ordering::SeqCst) {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }
}

pub(crate) struct Engine {
    _runtime: AsyncRuntime,
    context: AsyncContext,
    exports: Vec<String>,
    state: Rc<HostState>,
}

fn script_error(message: String, secrets: &[String]) -> ExtError {
    ExtError::Script(redact(&message, secrets))
}

/// Lỗi bắt được từ JS → `ExtError`. `cancel`: lượt đang bị huỷ (lỗi "interrupted" khi đó là do huỷ).
fn map_caught(caught: CaughtError<'_>, cancel: bool, secrets: &[String]) -> ExtError {
    match caught {
        CaughtError::Exception(exception) => {
            let message = exception.message().unwrap_or_default();
            let object = exception.as_object();
            let name: Option<String> = object.get("name").ok().flatten();
            let clean = redact(&message, secrets);
            match name.as_deref() {
                _ if message == "interrupted" => {
                    if cancel {
                        ExtError::Cancelled
                    } else {
                        ExtError::Interrupted
                    }
                }
                _ if message == "out of memory" => ExtError::OutOfMemory,
                Some("AuthExpired") => ExtError::AuthExpired(clean),
                Some("Locked") => ExtError::Locked(clean),
                Some("NeedsInteraction") => ExtError::NeedsInteraction(clean),
                Some("ExtensionError") => {
                    let retry: bool = object.get::<_, Option<bool>>("retry").ok().flatten().unwrap_or(false);
                    ExtError::Extension { message: clean, retry }
                }
                _ => {
                    let stack = exception.stack().unwrap_or_default();
                    let head = match name.as_deref() {
                        Some(n) if !n.is_empty() => format!("{n}: {message}"),
                        _ => message,
                    };
                    script_error(if stack.trim().is_empty() { head } else { format!("{head}\n{}", stack.trim_end()) }, secrets)
                }
            }
        }
        CaughtError::Value(value) => {
            let text = value.ctx().json_stringify(value.clone()).ok().flatten().and_then(|s| s.to_string().ok()).unwrap_or_else(|| format!("{value:?}"));
            script_error(format!("extension ném giá trị không phải Error: {text}"), secrets)
        }
        CaughtError::Error(error) => {
            if cancel && error.to_string().contains("interrupted") {
                return ExtError::Cancelled;
            }
            script_error(error.to_string(), secrets)
        }
    }
}

fn to_json<'js>(ctx: &Ctx<'js>, value: Value<'js>) -> Result<serde_json::Value, ExtError> {
    if value.is_undefined() {
        return Ok(serde_json::Value::Null);
    }
    let text = ctx.json_stringify(value).map_err(|e| ExtError::BadResult(format!("không đổi được kết quả sang JSON: {e}")))?;
    match text {
        None => Ok(serde_json::Value::Null),
        Some(s) => {
            let s = s.to_string().map_err(|e| ExtError::BadResult(e.to_string()))?;
            serde_json::from_str(&s).map_err(|e| ExtError::BadResult(e.to_string()))
        }
    }
}

impl Engine {
    pub(crate) async fn new(package: &Package, http: Rc<HttpClient>, opts: EngineOptions, stores: Stores) -> Result<Engine, ExtError> {
        let runtime = AsyncRuntime::new().map_err(|e| ExtError::Script(e.to_string()))?;
        runtime.set_memory_limit(opts.memory_limit).await;
        runtime.set_max_stack_size(STACK_LIMIT).await;
        let activity = Rc::new(Cell::new(Instant::now()));
        {
            let activity = activity.clone();
            let cancel = opts.cancel.clone();
            let slice = opts.cpu_slice;
            runtime
                .set_interrupt_handler(Some(Box::new(move || cancel.load(Ordering::SeqCst) || activity.get().elapsed() > slice)))
                .await;
        }
        runtime.set_loader(loader::MemResolver(package.files.clone()), loader::MemLoader(package.files.clone())).await;
        let context = AsyncContext::full(&runtime).await.map_err(|e| ExtError::Script(e.to_string()))?;
        let state = Rc::new(HostState {
            activity,
            cancel: opts.cancel.clone(),
            secrets: RefCell::new(Vec::new()),
            log: opts.log.clone(),
            progress: opts.progress.clone(),
            http,
            session: stores.session,
            cache_path: stores.cache,
            cache: RefCell::new(None),
        });
        state.add_secrets(opts.secrets.iter().cloned());

        let prelude = PRELUDE.replace("/*BINDINGS*/", &[bind_http::JS, bind_html::JS, bind_crypto::JS, bind_store::JS].concat());
        let setup_state = state.clone();
        context
            .with(|ctx| -> Result<(), ExtError> {
                bind_misc::install(&ctx, &setup_state).map_err(|e| ExtError::Script(e.to_string()))?;
                bind_http::install(&ctx, &setup_state).map_err(|e| ExtError::Script(e.to_string()))?;
                bind_html::install(&ctx, &setup_state).map_err(|e| ExtError::Script(e.to_string()))?;
                bind_crypto::install(&ctx, &setup_state).map_err(|e| ExtError::Script(e.to_string()))?;
                bind_store::install(&ctx, &setup_state).map_err(|e| ExtError::Script(e.to_string()))?;
                ctx.eval::<(), _>(prelude).catch(&ctx).map_err(|e| map_caught(e, false, &[]))?;
                Ok(())
            })
            .await?;

        // Nạp main.js (chạy code ở mức module) dưới cùng giới hạn thời gian.
        state.touch();
        let secrets = state.secrets();
        let cancel = state.cancel.clone();
        let exports = context
            .async_with(async |ctx| -> Result<Vec<String>, ExtError> {
                let caught = |e| map_caught(e, cancel.load(Ordering::SeqCst), &secrets);
                let namespace: Object = Module::import(&ctx, crate::package::ENTRY).catch(&ctx).map_err(caught)?.into_future::<Object>().await.catch(&ctx).map_err(caught)?;
                let mut names = Vec::new();
                for key in namespace.keys::<String>() {
                    names.push(key.map_err(|e| ExtError::Script(e.to_string()))?);
                }
                Ok(names)
            })
            .await?;
        Ok(Engine { _runtime: runtime, context, exports, state })
    }

    pub(crate) fn has_export(&self, name: &str) -> bool {
        self.exports.iter().any(|e| e == name)
    }

    pub(crate) fn state(&self) -> &HostState {
        &self.state
    }

    /// Gọi một hàm export, chờ tối đa `limit`.
    pub(crate) async fn call(&self, name: &str, args: Vec<serde_json::Value>, limit: Duration) -> Result<serde_json::Value, ExtError> {
        let mut out = Err(ExtError::Cancelled);
        self.call_many(name, vec![args], 1, limit, &mut |_, result| out = result).await;
        out
    }

    /// Gọi `name` cho từng bộ tham số, tối đa `concurrency` lệnh song song trên cùng môi trường JS (biến toàn
    /// cục dùng chung). `on_each(vị trí, kết quả)` chạy ngay khi từng lệnh xong. Lượt bị huỷ: lệnh chưa chạy và
    /// lệnh đang chờ I/O đều trả `Cancelled`.
    pub(crate) async fn call_many(
        &self,
        name: &str,
        calls: Vec<Vec<serde_json::Value>>,
        concurrency: usize,
        limit: Duration,
        on_each: &mut dyn FnMut(usize, Result<serde_json::Value, ExtError>),
    ) {
        let state = self.state.clone();
        if !self.has_export(name) {
            let error = ExtError::Script(format!("main.js không export hàm {name}"));
            for index in 0..calls.len() {
                on_each(index, Err(error.clone()));
            }
            return;
        }
        let tag = label(name);
        self.context
            .async_with(async |ctx| {
                let caught = |e: CaughtError<'_>, st: &HostState| map_caught(e, st.cancelled(), &st.secrets());
                let namespace: Object = match Module::import(&ctx, crate::package::ENTRY).catch(&ctx) {
                    Ok(promise) => match promise.into_future::<Object>().await.catch(&ctx) {
                        Ok(ns) => ns,
                        Err(e) => {
                            let error = caught(e, &state);
                            (0..calls.len()).for_each(|i| on_each(i, Err(error.clone())));
                            return;
                        }
                    },
                    Err(e) => {
                        let error = caught(e, &state);
                        (0..calls.len()).for_each(|i| on_each(i, Err(error.clone())));
                        return;
                    }
                };
                let function: Function = match namespace.get(name) {
                    Ok(f) => f,
                    Err(e) => {
                        let error = ExtError::Script(e.to_string());
                        (0..calls.len()).for_each(|i| on_each(i, Err(error.clone())));
                        return;
                    }
                };
                let internal: Object = ctx.globals().get("__qt_internal").expect("prelude đã chạy");
                let freeze: Function = internal.get("deepFreeze").expect("deepFreeze");

                let run_one = |index: usize, args: Vec<serde_json::Value>| {
                    let ctx = ctx.clone();
                    let function = function.clone();
                    let freeze = freeze.clone();
                    let state = state.clone();
                    async move {
                        state.touch();
                        let work = async {
                            let mut js_args = Args::new(ctx.clone(), args.len());
                            for arg in &args {
                                let text = serde_json::to_string(arg).map_err(|e| ExtError::Script(e.to_string()))?;
                                let value = ctx.json_parse(text).map_err(|e| ExtError::Script(e.to_string()))?;
                                let frozen: Value = freeze.call((value,)).map_err(|e| ExtError::Script(e.to_string()))?;
                                js_args.push_arg(frozen).map_err(|e| ExtError::Script(e.to_string()))?;
                            }
                            let returned: Value = function.call_arg(js_args).catch(&ctx).map_err(|e| map_caught(e, state.cancelled(), &state.secrets()))?;
                            let value: Value = MaybePromise::from_value(returned).into_future::<Value>().await.catch(&ctx).map_err(|e| map_caught(e, state.cancelled(), &state.secrets()))?;
                            to_json(&ctx, value)
                        };
                        let cancel = HostState::until_cancelled(state.cancel.clone());
                        let result = tokio::select! {
                            biased;
                            _ = cancel => Err(ExtError::Cancelled),
                            timed = tokio::time::timeout(limit, work) => timed.unwrap_or(Err(ExtError::Timeout(tag))),
                        };
                        (index, result)
                    }
                };

                let mut pending = calls.into_iter().enumerate();
                let mut running = FuturesUnordered::new();
                for (index, args) in pending.by_ref().take(concurrency.max(1)) {
                    running.push(run_one(index, args));
                }
                while let Some((index, result)) = running.next().await {
                    on_each(index, result);
                    if let Some((next, args)) = pending.next() {
                        if state.cancelled() {
                            on_each(next, Err(ExtError::Cancelled));
                            for (rest, _) in pending.by_ref() {
                                on_each(rest, Err(ExtError::Cancelled));
                            }
                        } else {
                            running.push(run_one(next, args));
                        }
                    }
                }
            })
            .await;
    }
}

#[doc(hidden)]
pub mod testing {
    //! Gọi một hàm export trên tokio current-thread — cho test của crate.
    use super::*;
    use crate::HostPolicy;

    pub fn call(package: &Package, name: &str, args: Vec<serde_json::Value>, opts: EngineOptions) -> Result<serde_json::Value, ExtError> {
        call_with_limit(package, name, args, opts, Duration::from_secs(60))
    }

    pub fn call_with_limit(package: &Package, name: &str, args: Vec<serde_json::Value>, opts: EngineOptions, limit: Duration) -> Result<serde_json::Value, ExtError> {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("tokio");
        runtime.block_on(async {
            let http = Rc::new(HttpClient::new(HostPolicy::new(&package.manifest.hosts, true), package.manifest.limits)?);
            let engine = Engine::new(package, http, opts, Stores::memory()).await?;
            engine.call(name, args, limit).await
        })
    }
}
