//! `cache` và `session` cho extension (spec mục 6). Giá trị là chuỗi hoặc dữ liệu JSON, lưu dạng JSON; `ttl`
//! tính bằng giây. Hàm host chạy đồng bộ (SQLite / file nhỏ trên máy), lớp JS bọc thành Promise.

use super::HostState;
use rquickjs::{Ctx, Exception, Function, Result};
use std::rc::Rc;

pub(crate) const JS: &str = r#"
  {
    const keyOf = (key) => {
      if (typeof key !== "string" || key === "") throw new TypeError("key phải là chuỗi không rỗng");
      return key;
    };
    const encodeValue = (value) => {
      const text = value === undefined ? undefined : JSON.stringify(value);
      if (text === undefined) throw new TypeError("value phải là chuỗi hoặc dữ liệu JSON");
      return text;
    };
    const ttlOf = (o) => {
      if (o == null || o.ttl == null) return -1;
      const ttl = Number(o.ttl);
      if (!(ttl >= 0)) throw new TypeError("ttl phải là số giây không âm");
      return Math.floor(ttl);
    };
    const decodeValue = (text) => (text == null ? null : JSON.parse(text));
    define("cache", freeze({
      get: async (key) => decodeValue(host.cacheGet(keyOf(key))),
      set: async (key, value, o) => { host.cacheSet(keyOf(key), encodeValue(value), ttlOf(o)); },
      delete: async (key) => { host.cacheDelete(keyOf(key)); },
    }));
    define("session", freeze({
      get: async (key) => decodeValue(host.sessionGet(keyOf(key))),
      set: async (key, value, o) => { host.sessionSet(keyOf(key), encodeValue(value), ttlOf(o)); },
      delete: async (key) => { host.sessionDelete(keyOf(key)); },
      clear: async () => { host.sessionClear(); },
    }));
  }
"#;

fn ttl(value: f64) -> Option<u64> {
    (value >= 0.0).then_some(value as u64)
}

fn throw<T>(ctx: &Ctx<'_>, st: &HostState, error: crate::ExtError) -> Result<T> {
    Err(Exception::throw_message(ctx, &st.redact(&error.to_string())))
}

pub(crate) fn install<'js>(ctx: &Ctx<'js>, state: &Rc<HostState>) -> Result<()> {
    let globals = ctx.globals();

    let st = state.clone();
    globals.set(
        "__host_cacheGet",
        Function::new(ctx.clone(), move |ctx: Ctx<'js>, key: String| -> Result<Option<String>> {
            st.touch();
            st.with_cache(|c| c.get(&key)).or_else(|e| throw(&ctx, &st, e))
        })?,
    )?;
    let st = state.clone();
    globals.set(
        "__host_cacheSet",
        Function::new(ctx.clone(), move |ctx: Ctx<'js>, key: String, value: String, seconds: f64| -> Result<()> {
            st.touch();
            st.with_cache(|c| c.set(&key, &value, ttl(seconds))).or_else(|e| throw(&ctx, &st, e))
        })?,
    )?;
    let st = state.clone();
    globals.set(
        "__host_cacheDelete",
        Function::new(ctx.clone(), move |ctx: Ctx<'js>, key: String| -> Result<()> {
            st.touch();
            st.with_cache(|c| c.delete(&key)).or_else(|e| throw(&ctx, &st, e))
        })?,
    )?;

    let st = state.clone();
    globals.set(
        "__host_sessionGet",
        Function::new(ctx.clone(), move |key: String| -> Option<String> {
            st.touch();
            st.session.get(&key).map(|v| v.to_string())
        })?,
    )?;
    let st = state.clone();
    globals.set(
        "__host_sessionSet",
        Function::new(ctx.clone(), move |ctx: Ctx<'js>, key: String, value: String, seconds: f64| -> Result<()> {
            st.touch();
            let value: serde_json::Value = serde_json::from_str(&value).map_err(|e| Exception::throw_type(&ctx, &e.to_string()))?;
            st.session.set(&key, value, ttl(seconds)).or_else(|e| throw(&ctx, &st, e))
        })?,
    )?;
    let st = state.clone();
    globals.set(
        "__host_sessionDelete",
        Function::new(ctx.clone(), move |ctx: Ctx<'js>, key: String| -> Result<()> {
            st.touch();
            st.session.delete(&key).or_else(|e| throw(&ctx, &st, e))
        })?,
    )?;
    let st = state.clone();
    globals.set(
        "__host_sessionClear",
        Function::new(ctx.clone(), move |ctx: Ctx<'js>| -> Result<()> {
            st.touch();
            st.http.jar().clear();
            st.session.clear().or_else(|e| throw(&ctx, &st, e))
        })?,
    )?;
    Ok(())
}
