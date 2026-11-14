//! Hai extension mẫu trong docs/extensions/examples chạy được thật: chép sang thư mục tạm, đổi `hosts` / `match`
//! sang 127.0.0.1 rồi chạy với máy chủ giả.

mod common;

use common::{demo_server, Route, TestServer};
use qt_ext::{EngineOptions, ExtError, ExtStore, Package, SecretBackend, Session, SessionOptions, TocItem};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

fn examples() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/extensions/examples")
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap().flatten() {
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn local_copy(name: &str) -> (tempfile::TempDir, Package) {
    let source = examples().join(name);
    assert!(Package::load_dir(&source, false).is_ok(), "manifest gốc của mẫu {name} hợp lệ");
    let dir = tempfile::tempdir().unwrap();
    copy_dir(&source, dir.path());
    let mut manifest: Value = serde_json::from_str(&std::fs::read_to_string(dir.path().join("manifest.json")).unwrap()).unwrap();
    manifest["hosts"] = json!(["127.0.0.1"]);
    manifest["match"] = json!(["^http"]);
    std::fs::write(dir.path().join("manifest.json"), manifest.to_string()).unwrap();
    let package = Package::load_dir(dir.path(), true).unwrap();
    (dir, package)
}

fn opts(store: Option<ExtStore>, logs: Arc<Mutex<Vec<String>>>) -> SessionOptions {
    let engine = EngineOptions { log: Arc::new(move |_, m| logs.lock().unwrap().push(m)), ..Default::default() };
    SessionOptions { allow_local: true, store, engine, ..Default::default() }
}

fn chapters(session: &Session, ctx: &Value, toc: &[TocItem], picks: &[usize]) -> Vec<Result<(String, Option<String>), ExtError>> {
    let items: Vec<(usize, TocItem)> = picks.iter().map(|i| (*i, toc[*i - 1].clone())).collect();
    let mut out = Vec::new();
    session.chapters(ctx, &items, &mut |o| out.push((o.index, o.result.map(|c| (c.content, c.title)))));
    out.sort_by_key(|(i, _)| *i);
    out.into_iter().map(|(_, r)| r).collect()
}

#[test]
fn mau_html_gbk() {
    let server = demo_server();
    let (_dir, package) = local_copy("html-gbk");
    let logs: Arc<Mutex<Vec<String>>> = Arc::default();
    let session = Session::start(package.clone(), opts(None, logs.clone())).unwrap();
    let ctx = json!({ "rawUrl": server.url("/book/1"), "settings": { "maxPages": 50 } });
    let toc = session.toc(&ctx).unwrap();
    assert_eq!(toc.len(), 6);
    assert_eq!(toc[0].title, "第1章 起");
    assert!(toc[4].locked, "chương VIP");
    let got = chapters(&session, &ctx, &toc, &[1, 6]);
    let (content, title) = got[0].clone().unwrap();
    assert!(content.contains("<p>天下大势，分久必合。</p>"), "{content}");
    assert_eq!(title.as_deref(), Some("第1章"));
    assert_eq!(got[1], Err(ExtError::Extension { message: "chương không còn trên trang nguồn".into(), retry: false }));

    let short = Session::start(package, opts(None, logs.clone())).unwrap();
    assert_eq!(short.toc(&json!({ "rawUrl": server.url("/book/1"), "settings": { "maxPages": 1 } })).unwrap().len(), 3);
    assert!(logs.lock().unwrap().iter().any(|l| l.contains("dừng ở 1 trang mục lục")));
}

fn api_server() -> TestServer {
    let json_header = || vec![("Content-Type".to_string(), "application/json".to_string())];
    TestServer::start(vec![
        Route::sequence(
            "/auth/token",
            vec![(200, json_header(), r#"{"token":"t1","expiresIn":3600}"#.into()), (200, json_header(), r#"{"token":"t2","expiresIn":3600}"#.into())],
        ),
        Route::json("/books/7/chapters?page=1&lang=zh-CN", r#"{"pages":2,"items":[{"id":101,"title":"第1章","vip":false},{"id":102,"title":"第2章","vip":false}]}"#),
        Route::json("/books/7/chapters?page=2&lang=zh-CN", r#"{"pages":2,"items":[{"id":103,"title":"第3章","vip":true,"purchased":false}]}"#),
        Route::sequence("/chapters/101", vec![(401, vec![], String::new()), (200, json_header(), r#"{"title":"第1章","content":"内容一"}"#.into())]),
        Route::json("/chapters/102", r#"{"title":"第2章","content":"内容二"}"#),
        Route::sequence("/chapters/103", vec![(403, vec![], String::new())]),
    ])
}

#[test]
fn mau_api_key_login_auth_expired_cache() {
    let server = api_server();
    let (_dir, package) = local_copy("api-key");
    let config = tempfile::tempdir().unwrap();
    let store = ExtStore::with_backend(config.path(), &package.manifest.id, SecretBackend::memory());
    let logs: Arc<Mutex<Vec<String>>> = Arc::default();
    assert_eq!(Session::start(package.clone(), opts(Some(store.clone()), logs.clone())).err(), Some(ExtError::AuthMissing));

    let key = json!({ "apiKey": "key-123" }).as_object().unwrap().clone();
    store.save_auth(&package.manifest, "apiKey", &key).unwrap();
    let ctx = json!({ "rawUrl": "https://truyen-mau.example/novel/7", "settings": { "apiBase": server.url(""), "lang": "zh-CN" } });

    let session = Session::start(package.clone(), opts(Some(store.clone()), logs.clone())).unwrap();
    let toc = session.toc(&ctx).unwrap();
    assert_eq!(toc.iter().map(|t| t.id.as_str()).collect::<Vec<_>>(), ["101", "102", "103"]);
    assert!(toc[2].locked, "VIP chưa mua");
    let got = chapters(&session, &ctx, &toc, &[1, 2, 3]);
    assert_eq!(got[0], Ok(("内容一".into(), Some("第1章".into()))), "401 → đăng nhập lại → chạy lại");
    assert_eq!(got[1], Ok(("内容二".into(), Some("第2章".into()))));
    assert!(matches!(got[2], Err(ExtError::Locked(_))));
    assert_eq!(server.hits("/auth/token"), 2);

    let again = Session::start(package, opts(Some(store.clone()), logs.clone())).unwrap();
    assert_eq!(again.toc(&ctx).unwrap().len(), 3);
    assert!(logs.lock().unwrap().iter().any(|l| l.contains("mục lục lấy từ cache (3 chương)")), "{:?}", logs.lock().unwrap());
    assert_eq!(server.hits("/books/7/chapters?page=1&lang=zh-CN"), 1);
    assert_eq!(server.hits("/auth/token"), 2, "lượt sau dùng lại phiên còn hạn");
    assert!(store.session_status().logged_in);
}
