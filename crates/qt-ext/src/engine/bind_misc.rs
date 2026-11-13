//! Hàm host nhóm chung: log, progress, sleep. Các nhóm khác (http, html, crypto, encoding) gắn ở file riêng
//! và nối lớp JS của mình vào `bindings_js()`.

use super::{redact, HostState, LogLevel};
use rquickjs::prelude::{Async, Opt};
use rquickjs::{Ctx, Error, Function, Result};
use std::rc::Rc;
use std::time::Duration;

/// Lớp JS của mọi nhóm API, chèn vào chỗ `/*BINDINGS*/` của prelude (thấy `host`, `define`, `freeze`).
pub(crate) fn bindings_js() -> String {
    String::new()
}

pub(crate) fn install(ctx: &Ctx<'_>, state: &Rc<HostState>) -> Result<()> {
    let globals = ctx.globals();

    let st = state.clone();
    globals.set(
        "__host_log",
        Function::new(ctx.clone(), move |level: String, message: String| {
            st.touch();
            let level = match level.as_str() {
                "warn" => LogLevel::Warn,
                "error" => LogLevel::Error,
                _ => LogLevel::Info,
            };
            (st.log)(level, redact(&message, &st.secrets));
        })?,
    )?;

    let st = state.clone();
    globals.set(
        "__host_progress",
        Function::new(ctx.clone(), move |done: f64, total: f64, message: Opt<String>| {
            st.touch();
            (st.progress)(done.max(0.0) as u64, total.max(0.0) as u64, message.0.map(|m| redact(&m, &st.secrets)));
        })?,
    )?;

    let st = state.clone();
    globals.set(
        "__host_sleep",
        Function::new(
            ctx.clone(),
            Async(move |ms: f64| {
                let st = st.clone();
                async move {
                    st.touch();
                    let wait = Duration::from_millis(ms.clamp(0.0, 600_000.0) as u64);
                    tokio::select! {
                        _ = tokio::time::sleep(wait) => {}
                        _ = HostState::until_cancelled(st.cancel.clone()) => {}
                    }
                    st.touch();
                    Ok::<(), Error>(())
                }
            }),
        )?,
    )?;
    Ok(())
}
