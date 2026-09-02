use qt_ai_core::commands::init::run_init;
use qt_ai_core::session::*;
use qt_ai_core::story_fs::{load_state, story_paths, ChapterStatus};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

fn story(mode: &str, chapters: usize) -> tempfile::TempDir {
    let dir = tempfile::Builder::new().prefix("qt-ai-session-").tempdir().unwrap();
    fs::create_dir_all(dir.path().join("raw")).unwrap();
    for i in 1..=chapters {
        fs::write(dir.path().join("raw").join(format!("{i:04}.txt")), "第一章").unwrap();
    }
    run_init(dir.path(), "qt-ai").unwrap();
    fs::write(dir.path().join("fake-agy-mode.txt"), mode).unwrap();
    dir
}

fn config(root: &Path) -> SessionConfig {
    SessionConfig {
        root: root.to_path_buf(),
        agy: std::path::PathBuf::from(env!("CARGO_BIN_EXE_fake-agy")),
        model: Some("fake-model".into()),
        max_sessions: 50,
        poll_interval: Duration::from_millis(50),
    }
}

fn collect() -> (Arc<dyn Fn(SessionEvent) + Send + Sync>, Arc<Mutex<Vec<SessionEvent>>>) {
    let events = Arc::new(Mutex::new(Vec::new()));
    let sink_events = events.clone();
    let sink: Arc<dyn Fn(SessionEvent) + Send + Sync> =
        Arc::new(move |event| sink_events.lock().unwrap().push(event));
    (sink, events)
}

#[test]
fn progress_dem_dung_tu_state_json() {
    let dir = story("noop", 3);
    let p = read_progress(dir.path()).unwrap();
    assert_eq!(
        p,
        Progress { done: 0, queued: 3, translating: 0, error: 0, skipped: 0, warnings_count: 0, current: None }
    );
}

#[test]
fn chay_toi_het_hang_doi_moi_phien_mot_chuong_roi_finished() {
    let dir = story("progress", 2);
    let (sink, events) = collect();
    let handle = start_session(config(dir.path()), sink).unwrap();
    assert_eq!(handle.join(), StopReason::Finished);
    let events = events.lock().unwrap();
    let starts = events.iter().filter(|e| matches!(e, SessionEvent::Started { .. })).count();
    assert_eq!(starts, 2, "{events:?}");
    assert!(events.iter().any(|e| matches!(
        e,
        SessionEvent::AgyLog { line, stream: LogStream::Stdout } if line.contains("fake: model=fake-model")
    )));
    assert!(events.iter().any(|e| matches!(e, SessionEvent::Progress(p) if p.done == 1 && p.queued == 1)));
    assert!(matches!(events.last(), Some(SessionEvent::Stopped(StopReason::Finished))));
    let state = load_state(&story_paths(dir.path())).unwrap();
    assert!(state.chapters.values().all(|c| c.status == ChapterStatus::Done));
    assert!(!dir.path().join("work").join(".session.lock").exists());
}

#[test]
fn phien_khong_tien_do_thi_ngat() {
    let dir = story("noop", 2);
    let (sink, events) = collect();
    let handle = start_session(config(dir.path()), sink).unwrap();
    assert_eq!(handle.join(), StopReason::NoProgress);
    assert_eq!(events.lock().unwrap().iter().filter(|e| matches!(e, SessionEvent::Started { .. })).count(), 1);
}

#[test]
fn agy_loi_hai_lan_lien_tiep_thi_ngat() {
    let dir = story("fail", 2);
    let (sink, events) = collect();
    let handle = start_session(config(dir.path()), sink).unwrap();
    assert_eq!(handle.join(), StopReason::AgyFailed { code: 3 });
    // Lỗi lần 1 chưa ngắt (có thể rate limit nhất thời), lần 2 mới ngắt → 2 phiên.
    assert_eq!(events.lock().unwrap().iter().filter(|e| matches!(e, SessionEvent::Started { .. })).count(), 2);
    assert!(events.lock().unwrap().iter().any(|e| matches!(e, SessionEvent::AgyLog { stream: LogStream::Stderr, .. })));
}

#[test]
fn cancel_giet_agy_ngay_va_don_lock() {
    let dir = story("hang", 1);
    let (sink, _events) = collect();
    let handle = start_session(config(dir.path()), sink).unwrap();
    std::thread::sleep(Duration::from_millis(300));
    assert!(handle.is_running());
    let started = Instant::now();
    handle.cancel();
    assert_eq!(handle.join(), StopReason::UserCancelled);
    assert!(started.elapsed() < Duration::from_secs(5), "cancel phải giết agy, không đợi nó tự thoát (20s)");
    assert!(!dir.path().join("work").join(".session.lock").exists());
}

#[test]
fn max_sessions_chan_vong_lap() {
    let dir = story("progress", 3);
    let mut cfg = config(dir.path());
    cfg.max_sessions = 1;
    let (sink, _) = collect();
    assert_eq!(start_session(cfg, sink).unwrap().join(), StopReason::MaxSessions);
    assert_eq!(read_progress(dir.path()).unwrap().done, 1);
}

#[test]
fn lock_chan_phien_thu_hai_va_tu_don_lock_mo_coi() {
    let dir = story("hang", 1);
    let (sink, _) = collect();
    let handle = start_session(config(dir.path()), sink.clone()).unwrap();
    std::thread::sleep(Duration::from_millis(200));
    assert!(matches!(
        start_session(config(dir.path()), sink.clone()),
        Err(qt_ai_core::CoreError::SessionLocked { .. })
    ));
    handle.cancel();
    handle.join();
    // lock mồ côi: PID không còn sống → phiên mới tự dọn và chạy
    fs::create_dir_all(dir.path().join("work")).unwrap();
    fs::write(dir.path().join("work").join(".session.lock"), r#"{"pid": 4000000000, "startedAt": 1}"#).unwrap();
    fs::write(dir.path().join("fake-agy-mode.txt"), "noop").unwrap();
    let handle = start_session(config(dir.path()), sink).unwrap();
    assert_eq!(handle.join(), StopReason::NoProgress);
}

#[test]
fn build_translate_prompt_dua_duong_dan_tuyet_doi() {
    let prompt = build_translate_prompt(Path::new("D:\\truyen"));
    assert!(prompt.contains("translate.md") && prompt.contains("KHÔNG cần tìm kiếm") && prompt.contains("D:\\truyen"));
}

#[test]
fn run_once_tra_exit_code() {
    let dir = story("fail", 1);
    let (sink, _) = collect();
    let code = run_once(&config(dir.path()), "prompt bất kỳ", &*sink).unwrap();
    assert_eq!(code, 3);
}

#[test]
fn find_agy_khong_co_thi_agy_missing_co_configured_thi_dung() {
    let dir = tempfile::tempdir().unwrap();
    assert!(matches!(
        qt_ai_core::agy::find_agy(Some(&dir.path().join("khong-co"))),
        Err(qt_ai_core::CoreError::AgyMissing)
    ));
    let fake = std::path::PathBuf::from(env!("CARGO_BIN_EXE_fake-agy"));
    assert_eq!(qt_ai_core::agy::find_agy(Some(&fake)).unwrap(), fake);
    assert_eq!(qt_ai_core::agy::agy_version(&fake).unwrap(), "fake-agy 0.0.1");
    assert_eq!(qt_ai_core::agy::agy_models(&fake).unwrap(), vec!["fake-model", "fake-model-pro"]);
}
