//! Dọn hội thoại agy: dựng thư mục dữ liệu agy giả (SQLite cùng schema tối thiểu agy dùng) trên tempdir.
use qt_ai_core::agy_data::{clean, scan, stats, Scope, APP_PROMPT_MARK, IDLE_GRACE};
use qt_ai_core::commands::init::run_init;
use qt_ai_core::session::{build_translate_prompt, start_session, LogStream, SessionConfig, SessionEvent, StopReason};
use qt_ai_core::story_fs::{load_state, now_ms, story_paths, ChapterStatus};
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

const TRANSLATE: &str = "11111111-1111-4111-8111-111111111111";
const SETUP: &str = "22222222-2222-4222-8222-222222222222";
const EMPTY: &str = "33333333-3333-4333-8333-333333333333";
const OTHER_STORY: &str = "44444444-4444-4444-8444-444444444444";
const USER_CHAT: &str = "55555555-5555-4555-8555-555555555555";
const USER_EMPTY: &str = "66666666-6666-4666-8666-666666666666";
const NOT_SQLITE: &str = "77777777-7777-4777-8777-777777777777";

fn story_dir(parent: &Path, name: &str) -> PathBuf {
    let root = parent.join(name);
    fs::create_dir_all(root.join("raw")).unwrap();
    fs::write(root.join("raw").join("0001.txt"), "第一章").unwrap();
    run_init(&root, "qt-ai").unwrap();
    root
}

/// `file:///D:/x/y` như agy ghi; blob bọc URI theo kiểu protobuf (tag, độ dài, chuỗi, tag kế).
fn workspace_blob(workspace: &Path) -> Vec<u8> {
    let uri = format!("file:///{}", workspace.to_string_lossy().replace('\\', "/").trim_start_matches('/'));
    let mut blob = vec![0x0a];
    if uri.len() < 128 {
        blob.push(uri.len() as u8);
    } else {
        blob.extend([(uri.len() & 0x7f) as u8 | 0x80, (uri.len() >> 7) as u8]);
    }
    blob.extend_from_slice(uri.as_bytes());
    blob.extend_from_slice(b"\x12\x04main");
    blob
}

/// Một hội thoại: `first` = prompt lượt đầu (None = chưa có lượt nào), kèm `brain/<id>/` 1000 byte.
fn conversation(data: &Path, id: &str, first: Option<&str>, workspace: &Path) {
    fs::create_dir_all(data.join("conversations")).unwrap();
    let conn = Connection::open(data.join("conversations").join(format!("{id}.db"))).unwrap();
    // agy để file ở chế độ WAL; đóng kết nối gọn thì -wal/-shm tự mất.
    conn.execute_batch(
        "pragma journal_mode = wal;
         create table steps (idx integer primary key, step_payload blob);
         create table trajectory_metadata_blob (id text primary key, data blob);",
    )
    .unwrap();
    if let Some(prompt) = first {
        let mut payload = b"\x08\x0e \x03*\x9a\x01".to_vec();
        payload.extend_from_slice(prompt.as_bytes());
        conn.execute("insert into steps (idx, step_payload) values (0, ?1)", [payload]).unwrap();
        conn.execute("insert into steps (idx, step_payload) values (1, ?1)", [b"tra loi".to_vec()]).unwrap();
    }
    conn.execute("insert into trajectory_metadata_blob (id, data) values ('main', ?1)", [workspace_blob(workspace)]).unwrap();
    drop(conn);
    let brain = data.join("brain").join(id).join("scratch");
    fs::create_dir_all(&brain).unwrap();
    fs::write(brain.join("note.md"), vec![b'x'; 1000]).unwrap();
}

/// Lùi mtime file hội thoại cho quá `IDLE_GRACE`.
fn age(data: &Path, id: &str) {
    let file = fs::File::options().write(true).open(data.join("conversations").join(format!("{id}.db"))).unwrap();
    file.set_modified(SystemTime::now() - IDLE_GRACE - Duration::from_secs(60)).unwrap();
}

fn exists(data: &Path, id: &str) -> bool {
    data.join("conversations").join(format!("{id}.db")).exists()
}

struct Fixture {
    _dir: tempfile::TempDir,
    data: PathBuf,
    story_a: PathBuf,
    story_b: PathBuf,
}

fn fixture() -> Fixture {
    let dir = tempfile::Builder::new().prefix("qt-ai-agy-data-").tempdir().unwrap();
    let data = dir.path().join("antigravity-cli");
    let story_a = story_dir(dir.path(), "truyen-a");
    let story_b = story_dir(dir.path(), "truyen-b");
    let plain = dir.path().join("du-an-khac");
    fs::create_dir_all(&plain).unwrap();
    let setup_prompt = format!("Mở file {}{APP_PROMPT_MARK} với input: tên = \"A\"", story_a.join("setup-story.md").display());
    conversation(&data, TRANSLATE, Some(&build_translate_prompt(&story_a)), &story_a);
    conversation(&data, SETUP, Some(&setup_prompt), &story_a);
    conversation(&data, EMPTY, None, &story_a);
    conversation(&data, OTHER_STORY, Some(&build_translate_prompt(&story_b)), &story_b);
    // Người dùng tự mở agy ngay trong folder truyện để hỏi việc khác: không phải của app.
    conversation(&data, USER_CHAT, Some("sửa giúp glossary chương 3"), &story_a);
    conversation(&data, USER_EMPTY, None, &plain);
    fs::write(data.join("conversations").join(format!("{NOT_SQLITE}.db")), "khong phai sqlite").unwrap();
    fs::write(data.join("conversations").join("ghi-chu.db"), "ten khong phai uuid").unwrap();
    Fixture { _dir: dir, data, story_a, story_b }
}

#[test]
fn scan_chi_nhan_hoi_thoai_mo_dau_bang_prompt_cua_app_hoac_rong_trong_folder_truyen() {
    let fx = fixture();
    let found = scan(&fx.data);
    let app: Vec<&str> = found.iter().filter(|c| c.app).map(|c| c.id.as_str()).collect();
    let other: Vec<&str> = found.iter().filter(|c| !c.app).map(|c| c.id.as_str()).collect();
    assert_eq!(app, vec![TRANSLATE, SETUP, EMPTY, OTHER_STORY]);
    assert_eq!(other, vec![USER_CHAT, USER_EMPTY, NOT_SQLITE]);
    let translate = found.iter().find(|c| c.id == TRANSLATE).unwrap();
    assert!(!translate.empty);
    assert_eq!(translate.workspace.as_deref().map(|p| p.to_string_lossy().replace('\\', "/")), Some(fx.story_a.to_string_lossy().replace('\\', "/")));
    assert!(found.iter().find(|c| c.id == EMPTY).unwrap().empty);

    let counted = stats(&fx.data);
    assert_eq!((counted.app_count, counted.other_count), (4, 3));
    assert!(counted.app_bytes > 4000 && counted.other_bytes > 2000, "{counted:?}");
    assert_eq!(stats(&fx.data.join("khong-co")), Default::default());
}

fn file_names(data: &Path) -> Vec<String> {
    let mut names: Vec<String> =
        fs::read_dir(data.join("conversations")).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).collect();
    names.sort();
    names
}

#[test]
fn scan_khong_de_lai_file_nao_va_doc_duoc_hoi_thoai_agy_dang_ghi_do() {
    let fx = fixture();
    let before = file_names(&fx.data);
    assert!(before.iter().all(|name| name.ends_with(".db")), "{before:?}");
    scan(&fx.data);
    stats(&fx.data);
    assert_eq!(file_names(&fx.data), before, "quét không được tạo -wal/-shm trong thư mục của agy");

    // agy đang chạy: lượt đầu mới nằm trong -wal, chưa về file chính. Vẫn phải nhận ra là của app.
    const LIVE: &str = "88888888-8888-4888-8888-888888888888";
    conversation(&fx.data, LIVE, None, &fx.story_b);
    let live = Connection::open(fx.data.join("conversations").join(format!("{LIVE}.db"))).unwrap();
    live.execute_batch("pragma wal_autocheckpoint = 0;").unwrap();
    live.execute("insert into steps (idx, step_payload) values (0, ?1)", [build_translate_prompt(&fx.story_b).into_bytes()]).unwrap();
    assert!(fs::metadata(fx.data.join("conversations").join(format!("{LIVE}.db-wal"))).unwrap().len() > 0);
    let found = scan(&fx.data);
    let seen = found.iter().find(|c| c.id == LIVE).unwrap();
    assert!(seen.app && !seen.empty, "{seen:?}");
    drop(live);
}

#[test]
fn don_theo_truyen_xoa_ngay_hoi_thoai_cua_truyen_do_va_khong_dung_cai_khac() {
    let fx = fixture();
    let report = clean(&fx.data, Scope::Story(&fx.story_a));
    // Hội thoại rỗng vừa tạo được giữ: có thể là lượt agy khác vừa mở trong folder này.
    assert_eq!((report.removed, report.kept), (2, 1), "{report:?}");
    assert!(report.freed_bytes > 2000);
    assert!(!exists(&fx.data, TRANSLATE) && !exists(&fx.data, SETUP));
    assert!(!fx.data.join("brain").join(TRANSLATE).exists());
    for id in [EMPTY, OTHER_STORY, USER_CHAT, USER_EMPTY, NOT_SQLITE] {
        assert!(exists(&fx.data, id), "{id} phải còn");
    }
    assert!(fx.data.join("brain").join(USER_CHAT).join("scratch").join("note.md").exists());
    assert!(fx.data.join("conversations").join("ghi-chu.db").exists());

    age(&fx.data, EMPTY);
    let later = clean(&fx.data, Scope::Story(&fx.story_a));
    assert_eq!((later.removed, later.kept), (1, 0), "{later:?}");
    assert!(!exists(&fx.data, EMPTY) && exists(&fx.data, USER_CHAT) && exists(&fx.data, OTHER_STORY));
}

#[test]
fn don_tat_ca_bo_qua_truyen_dang_dich_va_hoi_thoai_vua_ghi() {
    let fx = fixture();
    // Mọi thứ vừa ghi xong: chưa xoá gì.
    let fresh = clean(&fx.data, Scope::All { busy_roots: &[] });
    assert_eq!((fresh.removed, fresh.kept), (0, 4));

    for id in [TRANSLATE, SETUP, EMPTY, OTHER_STORY, USER_CHAT, USER_EMPTY] {
        age(&fx.data, id);
    }
    let busy = [fx.story_b.clone()];
    let report = clean(&fx.data, Scope::All { busy_roots: &busy });
    assert_eq!((report.removed, report.kept), (3, 1), "{report:?}");
    assert!(exists(&fx.data, OTHER_STORY), "truyện đang dịch giữ nguyên");
    assert!(exists(&fx.data, USER_CHAT) && exists(&fx.data, USER_EMPTY) && exists(&fx.data, NOT_SQLITE));

    let rest = clean(&fx.data, Scope::All { busy_roots: &[] });
    assert_eq!((rest.removed, rest.kept), (1, 0));
    let left = stats(&fx.data);
    assert_eq!((left.app_count, left.other_count), (0, 3));
}

#[test]
fn phien_agy_tu_don_hoi_thoai_cua_truyen_sau_moi_luot() {
    let dir = tempfile::Builder::new().prefix("qt-ai-agy-sweep-").tempdir().unwrap();
    let data = dir.path().join("antigravity-cli");
    let root = story_dir(dir.path(), "truyen");
    let other = story_dir(dir.path(), "truyen-khac");
    fs::write(root.join("fake-agy-mode.txt"), "progress").unwrap();
    conversation(&data, TRANSLATE, Some(&build_translate_prompt(&root)), &root);
    conversation(&data, OTHER_STORY, Some(&build_translate_prompt(&other)), &other);
    conversation(&data, USER_CHAT, Some("câu hỏi riêng"), &root);

    let events = Arc::new(Mutex::new(Vec::new()));
    let sink_events = events.clone();
    let config = SessionConfig {
        root: root.clone(),
        agy: PathBuf::from(env!("CARGO_BIN_EXE_fake-agy")),
        model: None,
        max_sessions: 5,
        poll_interval: Duration::from_millis(50),
        extra_path: None,
        agy_data_dir: Some(data.clone()),
        resume_on_quota: false,
    };
    let handle = start_session(config, Arc::new(move |event| sink_events.lock().unwrap().push(event))).unwrap();
    assert_eq!(handle.join(), StopReason::Finished);

    assert!(!exists(&data, TRANSLATE) && !data.join("brain").join(TRANSLATE).exists());
    assert!(exists(&data, OTHER_STORY) && exists(&data, USER_CHAT));
    let events = events.lock().unwrap();
    assert!(events.iter().any(|e| matches!(
        e,
        SessionEvent::AgyLog { line, stream: LogStream::Stdout } if line.contains("Đã dọn 1 hội thoại agy")
    )));
}

type Events = Arc<Mutex<Vec<SessionEvent>>>;

/// Truyện một chương, agy giả hết quota ở lượt đầu (reset sau `resets`) rồi dịch bình thường.
fn quota_story(resets: &str, resume_on_quota: bool) -> (tempfile::TempDir, PathBuf, PathBuf, SessionConfig, Events) {
    let dir = tempfile::Builder::new().prefix("qt-ai-quota-").tempdir().unwrap();
    let data = dir.path().join("antigravity-cli");
    let root = story_dir(dir.path(), "truyen");
    fs::write(root.join("fake-agy-mode.txt"), "quota").unwrap();
    fs::write(root.join("fake-agy-data-dir.txt"), data.to_string_lossy().as_bytes()).unwrap();
    fs::write(root.join("fake-agy-quota-resets.txt"), resets).unwrap();
    let config = SessionConfig {
        root: root.clone(),
        agy: PathBuf::from(env!("CARGO_BIN_EXE_fake-agy")),
        model: None,
        max_sessions: 2,
        poll_interval: Duration::from_millis(20),
        extra_path: None,
        agy_data_dir: Some(data.clone()),
        resume_on_quota,
    };
    (dir, data, root, config, Arc::new(Mutex::new(Vec::new())))
}

fn sink_of(events: &Events) -> Arc<dyn Fn(SessionEvent) + Send + Sync> {
    let events = events.clone();
    Arc::new(move |event| events.lock().unwrap().push(event))
}

#[test]
fn het_quota_thi_cho_toi_gio_reset_roi_tu_dich_tiep() {
    let (_dir, data, root, config, events) = quota_story("1s", true);
    let started = Instant::now();
    let asked_at = now_ms();
    let handle = start_session(config, sink_of(&events)).unwrap();
    // max_sessions = 2: lượt hết quota không được tính vào giới hạn — tính thì lượt dịch thật là lượt thứ hai,
    // chạy xong sẽ ra MaxSessions chứ không phải Finished.
    assert_eq!(handle.join(), StopReason::Finished);
    assert!(started.elapsed() >= Duration::from_millis(1000), "phải chờ tới giờ reset: {:?}", started.elapsed());

    let events = events.lock().unwrap();
    let waits: Vec<u64> = events.iter().filter_map(|e| if let SessionEvent::QuotaWait { until } = e { Some(*until) } else { None }).collect();
    assert_eq!(waits.len(), 1, "{events:?}");
    // Reset sau 1s kể từ lúc hội thoại được ghi, cộng biên 30 × poll_interval.
    assert!(waits[0] >= asked_at + 1000 && waits[0] <= now_ms(), "{waits:?}");
    assert_eq!(events.iter().filter(|e| matches!(e, SessionEvent::Started { session_no: 1 })).count(), 2);
    assert!(events.iter().any(|e| matches!(e, SessionEvent::AgyLog { line, .. } if line.contains("Hết quota agy"))));
    assert!(matches!(events.last(), Some(SessionEvent::Stopped(StopReason::Finished))));
    let state = load_state(&story_paths(&root)).unwrap();
    assert!(state.chapters.values().all(|chapter| chapter.status == ChapterStatus::Done));
    assert!(!exists(&data, "99999999-9999-4999-8999-999999999999"), "hội thoại hết quota vẫn được dọn sau khi đọc");
}

#[test]
fn het_quota_ma_khong_bat_tu_chay_lai_thi_dung_voi_ly_do_va_gio_reset() {
    let (_dir, _data, root, config, events) = quota_story("2h47m27s", false);
    let asked_at = now_ms();
    let handle = start_session(config, sink_of(&events)).unwrap();
    let StopReason::QuotaExhausted { reset_at } = handle.join() else { panic!("{:?}", events.lock().unwrap()) };
    let expected = asked_at + 10_047_000;
    assert!(reset_at >= expected - 2000 && reset_at <= expected + 10_000, "{reset_at} vs {expected}");
    assert!(!events.lock().unwrap().iter().any(|e| matches!(e, SessionEvent::QuotaWait { .. })));
    let state = load_state(&story_paths(&root)).unwrap();
    assert!(state.chapters.values().all(|chapter| chapter.status == ChapterStatus::Queued));
}

#[test]
fn dang_cho_quota_bam_dung_thi_thoat_ngay() {
    let (_dir, _data, _root, config, events) = quota_story("2h", true);
    let handle = start_session(config, sink_of(&events)).unwrap();
    let deadline = Instant::now() + Duration::from_secs(20);
    while !events.lock().unwrap().iter().any(|e| matches!(e, SessionEvent::QuotaWait { .. })) {
        assert!(Instant::now() < deadline, "không thấy QuotaWait: {:?}", events.lock().unwrap());
        std::thread::sleep(Duration::from_millis(20));
    }
    let cancelled_at = Instant::now();
    handle.cancel();
    assert_eq!(handle.join(), StopReason::UserCancelled);
    assert!(cancelled_at.elapsed() < Duration::from_secs(5));
}
