//! Lượt tải raw trọn vòng: máy chủ giả + extension mẫu của qt-ext (mục lục GBK 2 trang, 6 chương: chương 5 khoá,
//! chương 6 extension ném lỗi tạm).

mod common;

use qt_ai_core::commands::init::run_init;
use qt_ai_core::raw_download::matching::MatchedBy;
use qt_ai_core::raw_download::registry::Installed;
use qt_ai_core::raw_download::run::{download, load_toc, login, test_chapter, test_toc, ChapterResult, Event, Mode, RowStatus, RunOptions};
use qt_ai_core::raw_download::{load_download, save_download, set_story_settings, DownloadConfig};
use qt_ai_core::story_fs::{load_story_config, save_story_config, story_paths};
use qt_ext::testkit::{demo_extension_dir, demo_server, TestServer};
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

fn options() -> RunOptions {
    let package = qt_ext::Package::load_dir(&demo_extension_dir(), true).unwrap();
    let mut opts = RunOptions::new(Installed { package, dir: demo_extension_dir(), dev: true });
    opts.retry_delays = vec![Duration::ZERO, Duration::ZERO];
    opts
}

fn story(server: &TestServer, raw: &[(&str, &str)]) -> tempfile::TempDir {
    common::isolate_base();
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("raw")).unwrap();
    for (id, text) in raw {
        std::fs::write(dir.path().join("raw").join(format!("{id}.txt")), text).unwrap();
    }
    run_init(dir.path(), "qt-ai").unwrap();
    let paths = story_paths(dir.path());
    let mut config = load_story_config(&paths).unwrap();
    config.name = "Demo".into();
    config.source_url = server.url("/book/1");
    save_story_config(&paths, &config).unwrap();
    dir
}

fn no_cancel() -> Arc<AtomicBool> {
    Arc::new(AtomicBool::new(false))
}

fn raw(root: &Path, id: &str) -> String {
    std::fs::read_to_string(root.join("raw").join(format!("{id}.txt"))).unwrap()
}

#[test]
fn truyen_moi_tai_chuong_moi_khoa_loi_va_so_chuong() {
    let server = demo_server();
    let dir = story(&server, &[]);
    let opts = options();

    let (toc, shown) = load_toc(dir.path(), &opts, no_cancel()).unwrap();
    assert_eq!(toc.len(), 6);
    assert_eq!(shown.by, MatchedBy::Empty);
    assert_eq!(shown.template, "{chuong:4}_{chapter-id}");
    assert_eq!(shown.next_name.as_deref(), Some("0001_1"));
    assert_eq!(shown.rows[4].status, RowStatus::Locked);
    assert_eq!(shown.rows[0].status, RowStatus::Missing);

    let mut events = Vec::new();
    let summary = download(dir.path(), &opts, Some(toc), Mode::New, no_cancel(), &mut |e| events.push(e)).unwrap();
    assert_eq!(summary.saved, vec![1, 2, 3, 4]);
    assert_eq!(summary.short, vec![1, 2, 3, 4], "chương mẫu dưới 50 ký tự vẫn được ghi");
    assert!(summary.locked.is_empty(), "Tải chương mới bỏ qua chương khoá");
    assert_eq!(summary.failed.len(), 1);
    assert_eq!((summary.failed[0].index, summary.failed[0].reason.as_str()), (6, "hỏng"));
    assert!(events.iter().any(|e| matches!(e, Event::Log(m) if m.contains("Thử lại"))), "lỗi tạm được thử lại");
    assert_eq!(events.first(), Some(&Event::Toc { total: 5 }));

    assert_eq!(raw(dir.path(), "0001_1"), "第1章\n\n天下大势，分久必合。\n第1段\n");
    let ledger = load_download(&story_paths(dir.path())).chapters;
    assert_eq!(ledger.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>(), ["1=0001_1", "2=0002_2", "3=0003_3", "4=0004_4"]);

    // Lần 2: không tải lại chương đã có; chương lỗi vẫn thiếu nên thử lại.
    let again = download(dir.path(), &opts, None, Mode::New, no_cancel(), &mut |_| {}).unwrap();
    assert!(again.saved.is_empty());
    assert_eq!(again.failed.iter().map(|f| f.index).collect::<Vec<_>>(), vec![6]);

    // Tải lại một chương đã có: ghi đè đúng file cũ, có backup.
    let mut results = Vec::new();
    let single = download(dir.path(), &opts, None, Mode::Single("2".into()), no_cancel(), &mut |e| {
        if let Event::Chapter { result, .. } = e {
            results.push(result);
        }
    })
    .unwrap();
    assert_eq!(single.saved, vec![2]);
    assert_eq!(results, vec![ChapterResult::Saved { file: "0002_2".into(), short: true, replaced: true }]);
    let backups: Vec<_> = std::fs::read_dir(dir.path().join("work/raw-backup")).unwrap().flatten().collect();
    assert_eq!(backups.len(), 1);
    assert!(backups[0].file_name().to_string_lossy().starts_with("0002_2."));

    // Chọn chương khoá: vẫn gọi extension; extension mẫu không chặn nên tải được.
    let locked = download(dir.path(), &opts, None, Mode::Selected(vec!["5".into()]), no_cancel(), &mut |_| {}).unwrap();
    assert_eq!(locked.saved, vec![5]);
}

#[test]
fn truyen_cu_doan_mau_va_noi_tiep() {
    let server = demo_server();
    let dir = story(&server, &[("chuong-0001", "第1章 起\n\n旧内容"), ("chuong-0002", "第2章 承\n\n旧内容")]);
    let opts = options();
    let (toc, shown) = load_toc(dir.path(), &opts, no_cancel()).unwrap();
    assert_eq!(shown.by, MatchedBy::Order);
    assert!(shown.mismatches.is_empty());
    assert_eq!(shown.template, "chuong-{chuong:4}");
    assert_eq!(shown.next_name.as_deref(), Some("chuong-0003"));
    assert!(shown.next_sorts_after);
    assert_eq!(shown.last_index, Some(2));

    let summary = download(dir.path(), &opts, Some(toc), Mode::New, no_cancel(), &mut |_| {}).unwrap();
    assert_eq!(summary.saved, vec![3, 4]);
    assert!(dir.path().join("raw/chuong-0004.txt").exists());
    assert!(raw(dir.path(), "chuong-0001").contains("旧内容"), "chương cũ không bị đụng");
    let ledger = load_download(&story_paths(dir.path())).chapters;
    assert_eq!(ledger.get("1").map(String::as_str), Some("chuong-0001"), "chương cũ vào sổ theo thứ tự đã khớp");
    assert_eq!(ledger.get("4").map(String::as_str), Some("chuong-0004"));
}

#[test]
fn lech_tieu_de_va_moc() {
    let server = demo_server();
    let dir = story(&server, &[("chuong-0001", "第3章 转\n\nx"), ("chuong-0002", "第4章 合\n\ny")]);
    let mut opts = options();
    let (_, shown) = load_toc(dir.path(), &opts, no_cancel()).unwrap();
    assert!(!shown.mismatches.is_empty());
    assert_eq!(shown.mismatches[0].file_title, "第3章 转");
    opts.anchor = Some(4);
    let (toc, fixed) = load_toc(dir.path(), &opts, no_cancel()).unwrap();
    assert!(fixed.mismatches.is_empty());
    assert_eq!(fixed.rows[2].file.as_deref(), Some("chuong-0001"));
    let summary = download(dir.path(), &opts, Some(toc), Mode::New, no_cancel(), &mut |_| {}).unwrap();
    assert!(summary.saved.is_empty(), "sau mục 4 chỉ còn chương khoá và chương lỗi");
    assert_eq!(summary.failed.len(), 1);
}

#[test]
fn mau_rieng_cua_truyen_va_huy() {
    let server = demo_server();
    let dir = story(&server, &[]);
    let paths = story_paths(dir.path());
    save_download(&paths, &DownloadConfig { file_template: "c{chuong:3} {chapterTitle}".into(), ..Default::default() }).unwrap();
    let opts = options();
    let summary = download(dir.path(), &opts, None, Mode::Selected(vec!["1".into()]), no_cancel(), &mut |_| {}).unwrap();
    assert_eq!(summary.saved, vec![1]);
    assert!(dir.path().join("raw/c001 第1章 起.txt").exists());

    let cancel = Arc::new(AtomicBool::new(true));
    let cancelled = download(dir.path(), &opts, None, Mode::New, cancel, &mut |_| {});
    assert!(cancelled.is_err() || cancelled.unwrap().cancelled, "huỷ trước khi chạy thì dừng ngay");
}

#[test]
fn chua_co_link() {
    let server = demo_server();
    let dir = story(&server, &[]);
    let paths = story_paths(dir.path());
    let mut config = load_story_config(&paths).unwrap();
    config.source_url = String::new();
    save_story_config(&paths, &config).unwrap();
    let error = load_toc(dir.path(), &options(), no_cancel()).unwrap_err().to_string();
    assert!(error.contains("link tải raw"), "{error}");
}

const AUTH_MAIN: &str = r#"
    let expiredOnce = false;
    export async function login(ctx) {
      await http.get(ctx.rawUrl.replace(/\/book\/1$/, "/login"));
      return { token: "t-" + ctx.auth.values.apiKey + "-" + Math.random() };
    }
    export async function toc(ctx) { return [1, 2, 3].map((n) => ({ id: String(n), title: "第" + n + "章" })); }
    export async function chapter(ctx, ch) {
      if (ch.id === "2" && !expiredOnce) { expiredOnce = true; throw new AuthExpired("hết hạn"); }
      const s = ctx.settings;
      return { content: "第" + ch.id + "章\n" + [s.server, s.lang, s.proxy, ctx.session.token.startsWith("t-key-1-")].join(",") };
    }"#;

fn auth_extension() -> (tempfile::TempDir, Installed) {
    let ext = tempfile::tempdir().unwrap();
    let manifest = serde_json::json!({
        "id": "io.github.dev.authdemo", "name": "Auth", "version": "1.0.0", "apiVersion": 1, "description": "d",
        "developer": { "name": "dev" }, "match": ["^http"], "hosts": ["127.0.0.1"], "limits": { "concurrency": 2 },
        "auth": { "required": true, "schemes": [{ "id": "apiKey", "label": "API key", "fields": [{ "key": "apiKey", "label": "API key", "type": "secret" }] }] },
        "settings": [
            { "key": "server", "label": "Máy chủ", "type": "select", "choices": ["cn", "global"], "default": "cn" },
            { "key": "lang", "label": "Ngôn ngữ", "type": "text", "default": "zh" },
            { "key": "proxy", "label": "Proxy", "type": "text", "scope": "story" }
        ]
    });
    std::fs::write(ext.path().join("manifest.json"), manifest.to_string()).unwrap();
    std::fs::write(ext.path().join("main.js"), AUTH_MAIN).unwrap();
    let package = qt_ext::Package::load_dir(ext.path(), true).unwrap();
    let installed = Installed { package, dir: ext.path().to_path_buf(), dev: true };
    (ext, installed)
}

#[test]
fn auth_login_cai_dat_chung_va_truyen_auth_expired() {
    let server = TestServer::start(vec![qt_ext::testkit::Route::ok("/login", "ok")]);
    let dir = story(&server, &[]);
    let config_root = tempfile::tempdir().unwrap();
    let (_ext, installed) = auth_extension();
    let manifest = installed.package.manifest.clone();
    let store = qt_ext::ExtStore::with_backend(config_root.path(), &manifest.id, qt_ext::SecretBackend::memory());
    let map = |v: serde_json::Value| v.as_object().unwrap().clone();

    let mut opts = RunOptions::new(installed);
    opts.retry_delays = vec![Duration::ZERO, Duration::ZERO];
    opts.session.store = Some(store.clone());
    let missing = load_toc(dir.path(), &opts, no_cancel()).unwrap_err().to_string();
    assert!(missing.contains("cần auth"), "{missing}");

    store.save_auth(&manifest, "apiKey", &map(serde_json::json!({ "apiKey": "key-1" }))).unwrap();
    store.save_settings(&manifest, &map(serde_json::json!({ "server": "global" }))).unwrap();
    let paths = story_paths(dir.path());
    let mut config = load_download(&paths);
    assert!(set_story_settings(&mut config, &manifest, &map(serde_json::json!({ "proxy": true }))).unwrap_err().contains("Proxy"));
    set_story_settings(&mut config, &manifest, &map(serde_json::json!({ "proxy": "p1", "server": "bỏ qua" }))).unwrap();
    save_download(&paths, &config).unwrap();
    assert_eq!(load_download(&paths).settings["io.github.dev.authdemo"], map(serde_json::json!({ "proxy": "p1" })));

    let summary = download(dir.path(), &opts, None, Mode::New, no_cancel(), &mut |_| {}).unwrap();
    assert_eq!(summary.saved, vec![1, 2, 3], "{summary:?}");
    assert_eq!(raw(dir.path(), "0002_2"), "第2章\n\nglobal,zh,p1,true\n");
    assert_eq!(server.hits("/login"), 2, "đăng nhập đầu lượt + một lần sau AuthExpired");

    let record = login(Some(dir.path()), &opts, no_cancel()).unwrap().unwrap();
    assert!(record.token.unwrap().starts_with("t-key-1-"));
    assert_eq!(server.hits("/login"), 3);
    assert!(store.session_status().logged_in);
}

#[test]
fn chay_thu_toc_va_chapter_khong_can_truyen() {
    let server = demo_server();
    let opts = options();
    let url = server.url("/book/1");
    let toc = test_toc(&opts, &url, no_cancel()).unwrap();
    assert!(toc.ok, "{toc:?}");
    assert_eq!(toc.result.as_array().unwrap().len(), 6);
    assert_eq!(toc.requests.len(), 2, "mục lục 2 trang");
    assert!(toc.requests.iter().all(|r| r.status == Some(200) && r.method == "GET"));
    assert!(toc.requests[1].url.ends_with("/book/1/list?page=2"), "{:?}", toc.requests[1]);
    assert!(toc.logs.iter().any(|l| l.message.contains("trang mục lục 2")), "{:?}", toc.logs);

    let items: Vec<qt_ext::TocItem> = serde_json::from_value(toc.result.clone()).unwrap();
    let one = test_chapter(&opts, &url, items[0].clone(), 1, no_cancel()).unwrap();
    assert!(one.ok, "{one:?}");
    assert!(one.result["content"].as_str().unwrap().contains("天下大势"));
    assert_eq!(one.result["saved"], serde_json::json!("第1章\n\n天下大势，分久必合。\n第1段\n"));
    assert_eq!(one.requests.len(), 1);

    let bad = test_chapter(&opts, &url, items[5].clone(), 6, no_cancel()).unwrap();
    assert!(!bad.ok);
    assert_eq!(bad.error.as_deref(), Some("hỏng"));
}
