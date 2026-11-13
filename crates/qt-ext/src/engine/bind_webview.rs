//! `webview.open` cho extension. Kiểm quyền và `hosts` trước khi mở, chờ chỗ (1 cửa sổ hiện / 2 cửa sổ ẩn mỗi
//! extension), gọi `WebviewHost` trên blocking pool, lọc kết quả theo `hosts` rồi nạp cookie vào hũ của `http`.
//! Thời gian cửa sổ hiện mở (chờ người dùng) không tính vào giới hạn của lệnh đang chạy.

use super::HostState;
use crate::webview::{finish, gates, parse_request};
use crate::ExtError;
use rquickjs::prelude::Async;
use rquickjs::{Ctx, Exception, Function, Result};
use std::rc::Rc;
use std::time::Instant;

pub(crate) const JS: &str = r#"
  {
    const NEEDS = "NeedsInteraction: ";
    define("webview", freeze({
      open: async (options) => {
        if (options == null || typeof options !== "object") throw new TypeError("webview.open: cần object tuỳ chọn");
        const until = options.until && Object.assign({}, options.until, {
          url: options.until.url instanceof RegExp ? options.until.url.source : options.until.url,
        });
        let raw;
        try {
          raw = await host.webview(JSON.stringify(Object.assign({}, options, { until })));
        } catch (e) {
          const message = String((e && e.message) || e);
          if (message.startsWith(NEEDS)) throw new NeedsInteraction(message.slice(NEEDS.length));
          throw e;
        }
        return deepFreeze(JSON.parse(raw));
      },
    }));
  }
"#;

enum OpenError {
    Type(String),
    Ext(ExtError),
}

/// Đang có cửa sổ hiện: cộng thời gian mở vào phần "chờ người dùng" khi đóng (kể cả khi huỷ giữa chừng).
struct Interactive<'a> {
    state: &'a HostState,
    start: Instant,
}

impl<'a> Interactive<'a> {
    fn begin(state: &'a HostState) -> Self {
        state.interactive.set(state.interactive.get() + 1);
        Interactive { state, start: Instant::now() }
    }
}

impl Drop for Interactive<'_> {
    fn drop(&mut self) {
        self.state.interactive.set(self.state.interactive.get().saturating_sub(1));
        self.state.interactive_time.set(self.state.interactive_time.get() + self.start.elapsed());
    }
}

async fn open(st: &HostState, options: &str) -> std::result::Result<String, OpenError> {
    let web = &st.webview;
    if !web.allowed {
        return Err(OpenError::Ext(ExtError::Extension { message: "extension chưa khai quyền \"webview\" trong manifest".into(), retry: false }));
    }
    let request = parse_request(options, web, st.http.policy().clone(), st.cancel.clone()).map_err(OpenError::Type)?;
    if let Err(denied) = st.http.policy().check(&request.url) {
        return Err(OpenError::Ext(denied));
    }
    let Some(host) = web.host.clone() else {
        return Err(OpenError::Ext(ExtError::NeedsInteraction("trang cần mở cửa sổ trình duyệt — chạy trong app để thao tác".into())));
    };
    let gates = gates(&web.extension_id);
    let semaphore = if request.visible { &gates.visible } else { &gates.hidden };
    let _permit = tokio::select! {
        permit = semaphore.acquire() => permit.map_err(|_| OpenError::Ext(ExtError::Cancelled))?,
        _ = HostState::until_cancelled(st.cancel.clone()) => return Err(OpenError::Ext(ExtError::Cancelled)),
    };
    let _interactive = request.visible.then(|| Interactive::begin(st));
    let sent = request.clone();
    let job = tokio::task::spawn_blocking(move || host.open(sent));
    let raw = tokio::select! {
        joined = job => joined.map_err(|e| OpenError::Ext(ExtError::Script(format!("webview: {e}"))))?.map_err(OpenError::Ext)?,
        // Host thấy cờ huỷ và tự đóng cửa sổ.
        _ = HostState::until_cancelled(st.cancel.clone()) => return Err(OpenError::Ext(ExtError::Cancelled)),
    };
    let (value, cookies) = finish(&request, raw);
    if !cookies.is_empty() {
        st.http.jar().insert(&cookies);
        st.persist_cookies();
    }
    Ok(value.to_string())
}

pub(crate) fn install<'js>(ctx: &Ctx<'js>, state: &Rc<HostState>) -> Result<()> {
    let st = state.clone();
    let webview = Function::new(
        ctx.clone(),
        Async(move |ctx: Ctx<'js>, options: String| {
            let st = st.clone();
            async move {
                st.touch();
                let result = open(&st, &options).await;
                st.touch();
                match result {
                    Ok(json) => Ok(json),
                    Err(OpenError::Type(message)) => Err(Exception::throw_type(&ctx, &format!("webview.open: {message}"))),
                    Err(OpenError::Ext(ExtError::NeedsInteraction(message))) => Err(Exception::throw_message(&ctx, &format!("NeedsInteraction: {message}"))),
                    Err(OpenError::Ext(error)) => Err(Exception::throw_message(&ctx, &st.redact(&error.to_string()))),
                }
            }
        }),
    )?;
    ctx.globals().set("__host_webview", webview)?;
    Ok(())
}
