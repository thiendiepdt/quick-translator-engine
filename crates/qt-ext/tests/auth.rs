//! Mốc 3: cache / session cho JS, hũ cookie giữa các lượt, auth, login, AuthExpired, che khoá.

mod common;

use common::{package, Route, TestServer};
use qt_ext::{AccountStatus, EngineOptions, ExtError, ExtStore, SecretBackend, Session, SessionOptions, TocItem};
use serde_json::{json, Map, Value};
use std::sync::{Arc, Mutex};

const ID: &str = "io.github.dev.test";

fn manifest(extra: Value) -> String {
    let mut m = json!({ "id": ID, "name": "Test", "version": "1.0.0", "apiVersion": 1, "description": "d",
        "developer": { "name": "dev" }, "match": ["^http"], "hosts": ["127.0.0.1"], "limits": { "concurrency": 3 } });
    for (k, v) in extra.as_object().unwrap() {
        m[k] = v.clone();
    }
    m.to_string()
}

fn auth_manifest(required: bool) -> String {
    manifest(json!({
        "auth": { "required": required, "schemes": [{ "id": "apiKey", "label": "API key", "fields": [{ "key": "apiKey", "label": "API key", "type": "secret" }] }] },
        "settings": [{ "key": "token", "label": "Token", "type": "secret" }]
    }))
}

struct Env {
    _dir: tempfile::TempDir,
    store: ExtStore,
    logs: Arc<Mutex<Vec<String>>>,
}

fn env() -> Env {
    let dir = tempfile::tempdir().unwrap();
    let store = ExtStore::with_backend(dir.path(), ID, SecretBackend::memory());
    Env { _dir: dir, store, logs: Arc::default() }
}

impl Env {
    fn opts(&self) -> SessionOptions {
        let logs = self.logs.clone();
        let engine = EngineOptions { log: Arc::new(move |_, m| logs.lock().unwrap().push(m)), ..Default::default() };
        SessionOptions { allow_local: true, store: Some(self.store.clone()), engine, ..Default::default() }
    }

    fn save_key(&self, files: &[(&str, &str)], key: &str) {
        let pkg = package(files);
        let values: Map<String, Value> = json!({ "apiKey": key }).as_object().unwrap().clone();
        self.store.save_auth(&pkg.manifest, "apiKey", &values).unwrap();
    }
}

fn first_title(session: &Session, ctx: &Value) -> String {
    session.toc(ctx).unwrap().remove(0).title
}

fn items(ids: &[&str]) -> Vec<(usize, TocItem)> {
    ids.iter().enumerate().map(|(i, id)| (i + 1, TocItem { id: id.to_string(), title: id.to_string(), locked: false, volume: None, data: None })).collect()
}

fn run_chapters(session: &Session, ctx: &Value, ids: &[&str]) -> Vec<(String, Result<String, ExtError>)> {
    let mut out = Vec::new();
    session.chapters(ctx, &items(ids), &mut |o| out.push((o.id, o.result.map(|c| c.content))));
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

#[test]
fn cache_va_session_giu_qua_hai_luot() {
    let env = env();
    let main = r#"
        export async function toc(ctx) {
          const before = { c: await cache.get("c"), s: await session.get("s"), t: await session.get("t") };
          await cache.set("c", { n: 1 });
          await session.set("s", "giá trị");
          await session.set("t", 1, { ttl: 0 });
          let caught = "";
          try { await cache.set("big", "x".repeat(6 * 1024 * 1024)); } catch (e) { caught = e.message; }
          let bad = "";
          try { await session.set("u", undefined); } catch (e) { bad = e.name; }
          return [{ id: "1", title: JSON.stringify({ before, caught, bad }) }];
        }
        export async function chapter() { return { content: "x" }; }"#;
    let files = [("main.js", main)];
    let one: Value = serde_json::from_str(&first_title(&Session::start(package(&files), env.opts()).unwrap(), &json!({}))).unwrap();
    assert_eq!(one["before"], json!({ "c": null, "s": null, "t": null }));
    assert!(one["caught"].as_str().unwrap().contains("5 MB"), "{one}");
    assert_eq!(one["bad"], json!("TypeError"));
    let two: Value = serde_json::from_str(&first_title(&Session::start(package(&files), env.opts()).unwrap(), &json!({}))).unwrap();
    assert_eq!(two["before"], json!({ "c": { "n": 1 }, "s": "giá trị", "t": null }), "ttl 0 thì mất, còn lại giữ qua lượt");
    assert!(env.store.cache_bytes() > 0);
}

#[test]
fn cookie_luu_giua_hai_luot_cookies_false_va_dang_xuat() {
    let server = TestServer::start(vec![Route::sequence("/login", vec![(200, vec![("Set-Cookie".into(), "sid=abc; Path=/".into())], "ok".into())]), Route::echo("/check")]);
    let env = env();
    let main = r#"
        export async function toc(ctx) {
          if (ctx.login) await http.get(ctx.base + "/login");
          const jar = await (await http.get(ctx.base + "/check")).text();
          const plain = await (await http.get(ctx.base + "/check", { cookies: false })).text();
          return [{ id: "1", title: JSON.stringify({ jar: jar.includes("sid=abc"), plain: plain.includes("sid=abc") }) }];
        }
        export async function chapter() { return { content: "x" }; }"#;
    let files = [("main.js", main)];
    let run = |login: bool| -> Value {
        let session = Session::start(package(&files), env.opts()).unwrap();
        serde_json::from_str(&first_title(&session, &json!({ "base": server.url(""), "login": login }))).unwrap()
    };
    assert_eq!(run(true), json!({ "jar": true, "plain": false }));
    assert_eq!(run(false), json!({ "jar": true, "plain": false }), "cookie giữ qua lượt sau");
    assert!(env.store.session_status().has_data);
    env.store.logout().unwrap();
    assert_eq!(run(false), json!({ "jar": false, "plain": false }), "đăng xuất xoá cookie");
}

const LOGIN_MAIN: &str = r#"
    export async function login(ctx) {
      await http.get(ctx.base + "/login");
      if (ctx.session !== null) throw new Error("login nhận ctx.session phải là null");
      return { token: "tok-" + ctx.auth.values.apiKey, expiresAt: Date.now() + 3600000, data: { n: 1 } };
    }
    export async function toc(ctx) {
      log.info("dùng " + ctx.auth.values.apiKey + " / " + ctx.session.token + " / " + ctx.settings.token);
      return [{ id: "1", title: ctx.session.token + " " + ctx.session.data.n }];
    }
    export async function chapter() { return { content: "x" }; }"#;

#[test]
fn login_mot_lan_dung_lai_phien_doi_auth_thi_dang_nhap_lai_va_che_khoa() {
    let server = TestServer::start(vec![Route::ok("/login", "ok")]);
    let env = env();
    let manifest = auth_manifest(true);
    let files = [("manifest.json", manifest.as_str()), ("main.js", LOGIN_MAIN)];
    env.save_key(&files, "key-12345");
    let ctx = json!({ "base": server.url(""), "settings": { "token": "cai-dat-bi-mat" } });

    assert_eq!(first_title(&Session::start(package(&files), env.opts()).unwrap(), &ctx), "tok-key-12345 1");
    assert_eq!(first_title(&Session::start(package(&files), env.opts()).unwrap(), &ctx), "tok-key-12345 1");
    assert_eq!(server.hits("/login"), 1, "lượt sau dùng lại phiên còn hạn");
    assert!(env.store.session_status().logged_in);

    env.save_key(&files, "key-99999");
    assert_eq!(first_title(&Session::start(package(&files), env.opts()).unwrap(), &ctx), "tok-key-99999 1");
    assert_eq!(server.hits("/login"), 2, "đổi auth thì phiên cũ mất");

    let logs = env.logs.lock().unwrap().join("\n");
    assert!(!logs.contains("key-") && !logs.contains("tok-") && !logs.contains("cai-dat-bi-mat"), "{logs}");
    assert!(logs.contains("dùng *** / *** / ***"), "{logs}");
}

#[test]
fn thieu_auth_bat_buoc_va_login_loi() {
    let env = env();
    let manifest = auth_manifest(true);
    let files = [("manifest.json", manifest.as_str()), ("main.js", LOGIN_MAIN)];
    assert_eq!(Session::start(package(&files), env.opts()).err(), Some(ExtError::AuthMissing));

    let failing = r#"
        export async function login() { throw new ExtensionError("sai mật khẩu"); }
        export async function toc() { return []; }
        export async function chapter() { return { content: "x" }; }"#;
    let files = [("manifest.json", manifest.as_str()), ("main.js", failing)];
    env.save_key(&files, "key-1");
    let session = Session::start(package(&files), env.opts()).unwrap();
    assert_eq!(session.toc(&json!({})), Err(ExtError::LoginFailed("sai mật khẩu".into())));
    assert_eq!(session.login(&json!({})), Err(ExtError::LoginFailed("sai mật khẩu".into())));
}

#[test]
fn nut_dang_nhap() {
    let server = TestServer::start(vec![Route::ok("/login", "ok")]);
    let env = env();
    let manifest = auth_manifest(false);
    let files = [("manifest.json", manifest.as_str()), ("main.js", LOGIN_MAIN)];
    env.save_key(&files, "key-1");
    let session = Session::start(package(&files), env.opts()).unwrap();
    let record = session.login(&json!({ "base": server.url("") })).unwrap().unwrap();
    assert_eq!(record.token.as_deref(), Some("tok-key-1"));
    session.login(&json!({ "base": server.url("") })).unwrap();
    assert_eq!(server.hits("/login"), 2, "nút Đăng nhập luôn đăng nhập lại");

    let no_login = [("main.js", "export async function toc() { return []; }\nexport async function chapter() { return { content: 'x' }; }")];
    assert_eq!(Session::start(package(&no_login), env.opts()).unwrap().login(&json!({})), Ok(None));
}

const ACCOUNT_MAIN: &str = r#"
    export async function login(ctx) {
      if (ctx.request && ctx.request.mode === "login") return { token: "tk", account: "Nguyễn A" };
      return { guest: true };
    }
    export async function account(ctx) {
      const s = ctx.session;
      if (!s || s.guest) return { loggedIn: false, message: "đang dùng kiểu khách" };
      if (ctx.expired) throw new AuthExpired("phiên hết hạn");
      return { loggedIn: true, name: s.account };
    }
    export async function toc() { return [{ id: 1, title: "a" }]; }
    export async function chapter() { return { content: "x" }; }"#;

#[test]
fn tai_khoan_trang_nguon_kiem_khong_mo_dang_nhap() {
    let env = env();
    let manifest = manifest(json!({ "account": { "title": "Tài khoản Nguồn" } }));
    let files = [("manifest.json", manifest.as_str()), ("main.js", ACCOUNT_MAIN)];
    let session = Session::start(package(&files), env.opts()).unwrap();

    let status = session.account(&json!({})).unwrap();
    assert_eq!(status, AccountStatus { logged_in: false, name: None, message: Some("đang dùng kiểu khách".into()), checked: true });
    assert!(!env.store.session_status().logged_in, "kiểm tài khoản không tự gọi login");

    session.toc(&json!({})).unwrap();
    assert!(env.store.session_status().guest, "lượt tải chưa đăng nhập → phiên khách");
    assert!(!session.account(&json!({})).unwrap().logged_in);
    assert!(env.store.session_status().guest, "phiên khách không bị xoá khi kiểm");

    session.login(&json!({ "request": { "mode": "login" } })).unwrap();
    let saved = env.store.session_status();
    assert_eq!((saved.account.as_deref(), saved.guest), (Some("Nguyễn A"), false));
    assert_eq!(session.account(&json!({})).unwrap().name.as_deref(), Some("Nguyễn A"));

    let expired = session.account(&json!({ "expired": true })).unwrap();
    assert_eq!((expired.logged_in, expired.message.as_deref()), (false, Some("phiên hết hạn")));
    assert!(!env.store.session_status().logged_in, "phiên chết phía máy chủ → xoá để lượt sau đăng nhập lại");
}

#[test]
fn khong_co_ham_account_thi_theo_phien_da_luu() {
    let server = TestServer::start(vec![Route::ok("/login", "ok")]);
    let env = env();
    let manifest = auth_manifest(false);
    let files = [("manifest.json", manifest.as_str()), ("main.js", LOGIN_MAIN)];
    env.save_key(&files, "key-1");
    let session = Session::start(package(&files), env.opts()).unwrap();
    assert_eq!(session.account(&json!({})).unwrap(), AccountStatus::default());
    session.login(&json!({ "base": server.url("") })).unwrap();
    assert_eq!(session.account(&json!({})).unwrap(), AccountStatus { logged_in: true, ..Default::default() });
    assert_eq!(server.hits("/login"), 1, "kiểm không gọi login");
}

const EXPIRING_MAIN: &str = r#"
    let first = null;
    export async function login(ctx) {
      await http.get(ctx.base + "/login");
      return { token: "t" + Math.random() };
    }
    export async function toc() { return []; }
    export async function chapter(ctx, ch) {
      if (first === null) first = ctx.session.token;
      const stale = ctx.session.token === first;
      if (ch.id !== "ok" && (stale || ctx.always)) { await sleep(30); throw new AuthExpired("token hết hạn"); }
      return { content: ch.id + (stale ? " cũ" : " mới") };
    }"#;

#[test]
fn auth_expired_dang_nhap_lai_dung_mot_lan_cho_ca_lo() {
    let server = TestServer::start(vec![Route::ok("/login", "ok")]);
    let env = env();
    let files = [("main.js", EXPIRING_MAIN)];
    let session = Session::start(package(&files), env.opts()).unwrap();
    let out = run_chapters(&session, &json!({ "base": server.url("") }), &["a", "b", "c", "ok"]);
    assert_eq!(
        out,
        vec![("a".into(), Ok("a mới".into())), ("b".into(), Ok("b mới".into())), ("c".into(), Ok("c mới".into())), ("ok".into(), Ok("ok cũ".into()))]
    );
    assert_eq!(server.hits("/login"), 2, "đăng nhập đầu lượt + đúng một lần sau AuthExpired");
}

#[test]
fn auth_expired_lan_hai_thanh_loi_va_khong_login_thi_loi_luon() {
    let server = TestServer::start(vec![Route::ok("/login", "ok")]);
    let env = env();
    let files = [("main.js", EXPIRING_MAIN)];
    let session = Session::start(package(&files), env.opts()).unwrap();
    let out = run_chapters(&session, &json!({ "base": server.url(""), "always": true }), &["a", "b"]);
    assert!(out.iter().all(|(_, r)| matches!(r, Err(ExtError::AuthExpired(_)))), "{out:?}");
    assert_eq!(server.hits("/login"), 2);

    let no_login = r#"
        export async function toc() { throw new AuthExpired("hết hạn"); }
        export async function chapter() { throw new AuthExpired("hết hạn"); }"#;
    let session = Session::start(package(&[("main.js", no_login)]), env.opts()).unwrap();
    assert_eq!(session.toc(&json!({})), Err(ExtError::AuthExpired("hết hạn".into())));
    assert!(matches!(run_chapters(&session, &json!({}), &["a"])[0].1, Err(ExtError::AuthExpired(_))));
}

#[test]
fn auth_expired_o_toc_thi_dang_nhap_lai_roi_chay_lai() {
    let server = TestServer::start(vec![Route::ok("/login", "ok")]);
    let env = env();
    let main = r#"
        let n = 0;
        export async function login(ctx) { await http.get(ctx.base + "/login"); return { token: "t" + (++n) }; }
        export async function toc(ctx) {
          if (ctx.session.token === "t1") throw new AuthExpired("hết hạn");
          return [{ id: "1", title: ctx.session.token }];
        }
        export async function chapter() { return { content: "x" }; }"#;
    let session = Session::start(package(&[("main.js", main)]), env.opts()).unwrap();
    assert_eq!(first_title(&session, &json!({ "base": server.url("") })), "t2");
}
