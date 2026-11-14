//! `http.request / get / post` cho extension. Mọi request đi qua `HttpClient` (kiểm hosts từng bước redirect,
//! trần 20 MB, giới hạn theo host, chờ 429). Body phản hồi giữ ở Rust (`RawResponse`), JS đọc qua `text()`,
//! `json()`, `bytes()`.

use super::{HostState, RequestLog};
use crate::net::HttpRequest;
use crate::text;
use rquickjs::class::Trace;
use rquickjs::prelude::{Async, Opt};
use rquickjs::{Ctx, Exception, Function, JsLifetime, Result, TypedArray};
use serde::Deserialize;
use std::rc::Rc;

pub(crate) const JS: &str = r#"
  {
    const pairs = (value) => {
      if (value == null) return [];
      if (Array.isArray(value)) return value.map(([k, v]) => [String(k), String(v)]);
      return Object.keys(value).map((k) => [k, String(value[k])]);
    };
    class Response {
      constructor(raw, encoding) {
        this.status = raw.status;
        this.ok = raw.status >= 200 && raw.status < 300;
        this.url = raw.url;
        this.headers = freeze(JSON.parse(raw.headersJson()));
        Object.defineProperty(this, "_raw", { value: raw });
        Object.defineProperty(this, "_encoding", { value: encoding });
      }
      text(encoding) {
        const label = encoding === undefined ? this._encoding : encoding;
        return label == null ? this._raw.text() : this._raw.text(String(label));
      }
      json() { return JSON.parse(this.text()); }
      bytes() { return this._raw.bytes(); }
    }
    const request = async (input) => {
      const o = typeof input === "string" ? { url: input } : (input || {});
      if (!o.url) throw new TypeError("http.request: thiếu url");
      const headers = pairs(o.headers);
      let bodyText;
      let bodyBytes;
      if (o.body instanceof Uint8Array) bodyBytes = o.body;
      else if (typeof o.body === "string") bodyText = o.body;
      else if (o.body != null) {
        bodyText = JSON.stringify(o.body);
        if (!headers.some(([k]) => k.toLowerCase() === "content-type")) headers.push(["Content-Type", "application/json"]);
      }
      const options = { method: o.method || "GET", url: String(o.url), headers, query: pairs(o.query), timeout: o.timeout, bodyText, cookies: o.cookies === false ? false : undefined };
      // Tham số tuỳ chọn phía Rust không nhận `undefined` truyền tường minh.
      const raw = await (bodyBytes === undefined ? host.http(JSON.stringify(options)) : host.http(JSON.stringify(options), bodyBytes));
      return new Response(raw, o.encoding);
    };
    define("http", freeze({
      request,
      get: (url, o) => request(Object.assign({}, o, { url, method: "GET" })),
      post: (url, body, o) => request(Object.assign({}, o, { url, body, method: "POST" })),
    }));
  }
"#;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RequestOptions {
    method: String,
    url: String,
    #[serde(default)]
    headers: Vec<(String, String)>,
    #[serde(default)]
    query: Vec<(String, String)>,
    timeout: Option<f64>,
    body_text: Option<String>,
    cookies: Option<bool>,
}

#[derive(Trace, JsLifetime)]
#[rquickjs::class]
pub(crate) struct RawResponse {
    #[qjs(skip_trace)]
    status: u16,
    #[qjs(skip_trace)]
    url: String,
    #[qjs(skip_trace)]
    headers: Vec<(String, String)>,
    #[qjs(skip_trace)]
    body: Rc<Vec<u8>>,
    #[qjs(skip_trace)]
    charset: Option<String>,
}

#[rquickjs::methods]
impl RawResponse {
    #[qjs(get)]
    pub fn status(&self) -> u16 {
        self.status
    }

    #[qjs(get)]
    pub fn url(&self) -> String {
        self.url.clone()
    }

    #[qjs(rename = "headersJson")]
    pub fn headers_json(&self) -> String {
        let mut map = serde_json::Map::new();
        for (key, value) in &self.headers {
            let key = key.to_ascii_lowercase();
            match map.get_mut(&key) {
                Some(serde_json::Value::String(existing)) => {
                    existing.push_str(", ");
                    existing.push_str(value);
                }
                _ => {
                    map.insert(key, serde_json::Value::String(value.clone()));
                }
            }
        }
        serde_json::Value::Object(map).to_string()
    }

    pub fn text<'js>(&self, ctx: Ctx<'js>, encoding: Opt<String>) -> Result<String> {
        text::decode(&self.body, encoding.0.as_deref(), self.charset.as_deref()).map_err(|e| Exception::throw_message(&ctx, &e.to_string()))
    }

    pub fn bytes<'js>(&self, ctx: Ctx<'js>) -> Result<TypedArray<'js, u8>> {
        TypedArray::new_copy(ctx, self.body.as_slice())
    }
}

pub(crate) fn install<'js>(ctx: &Ctx<'js>, state: &Rc<HostState>) -> Result<()> {
    let st = state.clone();
    let http = Function::new(
        ctx.clone(),
        Async(move |ctx: Ctx<'js>, options: String, body: Opt<TypedArray<'js, u8>>| {
            let st = st.clone();
            // SAFETY: chép ra Vec ngay, không chạy JS nào trong lúc slice còn sống.
            let bytes: Option<Vec<u8>> = body.0.and_then(|b| unsafe { b.as_bytes() }.map(<[u8]>::to_vec));
            async move {
                st.touch();
                let parsed: RequestOptions = serde_json::from_str(&options).map_err(|e| Exception::throw_type(&ctx, &format!("http.request: {e}")))?;
                let body = bytes.or_else(|| parsed.body_text.map(String::into_bytes));
                let shown = st.on_request.as_ref().map(|_| {
                    let url = match url::Url::parse(parsed.url.trim()) {
                        Ok(mut url) => {
                            if !parsed.query.is_empty() {
                                url.query_pairs_mut().extend_pairs(&parsed.query);
                            }
                            url.to_string()
                        }
                        Err(_) => parsed.url.clone(),
                    };
                    (parsed.method.trim().to_ascii_uppercase(), st.redact(&url), std::time::Instant::now())
                });
                let request = HttpRequest {
                    method: parsed.method,
                    url: parsed.url,
                    headers: parsed.headers,
                    query: parsed.query,
                    body,
                    timeout_ms: parsed.timeout.filter(|t| *t > 0.0).map(|t| t.min(600_000.0) as u64),
                    no_cookies: parsed.cookies == Some(false),
                };
                let result = tokio::select! {
                    result = st.http.send(request) => result,
                    _ = HostState::until_cancelled(st.cancel.clone()) => Err(crate::ExtError::Cancelled),
                };
                st.touch();
                st.persist_cookies();
                if let (Some(sink), Some((method, url, start))) = (&st.on_request, shown) {
                    let (status, error) = match &result {
                        Ok(response) => (Some(response.status), None),
                        Err(error) => (None, Some(st.redact(&error.to_string()))),
                    };
                    sink(RequestLog { method: if method.is_empty() { "GET".into() } else { method }, url, status, ms: start.elapsed().as_millis() as u64, error });
                }
                match result {
                    Ok(response) => Ok(RawResponse {
                        status: response.status,
                        url: response.url,
                        headers: response.headers,
                        body: Rc::new(response.body),
                        charset: response.charset,
                    }),
                    Err(error) => Err(Exception::throw_message(&ctx, &st.redact(&error.to_string()))),
                }
            }
        }),
    )?;
    ctx.globals().set("__host_http", http)?;
    Ok(())
}
