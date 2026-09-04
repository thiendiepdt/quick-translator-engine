//! Vòng dịch API trên tempdir với model giả (không HTTP).
use qt_ai_core::api::{ApiError, TextModel};
use qt_ai_core::api_session::*;
use qt_ai_core::commands::init::run_init;
use qt_ai_core::session::{SessionEvent, StopReason};
use qt_ai_core::story_fs::{load_state, load_story_config, story_paths, ChapterStatus};
use std::collections::VecDeque;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const RAW: &str = "赵静文抬头看向远方的高塔。\n\n她沉默了很久没有说话。";
const GOOD_1: &str = "Triệu Tĩnh Văn ngẩng đầu nhìn về phía tòa tháp cao ở nơi xa.";
const GOOD_2: &str = "Nàng im lặng rất lâu, không nói một lời nào.";

fn good() -> String {
    format!("[[1]] {GOOD_1}\n\n[[2]] {GOOD_2}")
}

fn story(chapters: usize) -> tempfile::TempDir {
    let dir = tempfile::Builder::new().prefix("qt-ai-api-").tempdir().unwrap();
    fs::create_dir_all(dir.path().join("raw")).unwrap();
    for i in 1..=chapters {
        fs::write(dir.path().join("raw").join(format!("{i:04}.txt")), RAW).unwrap();
    }
    run_init(dir.path(), "qt-ai").unwrap();
    dir
}

/// Kịch bản: mỗi lượt generate lấy một phần tử; hết kịch bản thì trả bản dịch tốt.
struct FakeModel {
    script: Mutex<VecDeque<Result<String, ApiError>>>,
    calls: Mutex<Vec<(String, String)>>,
    glossary_json: String,
    wait_cancel: bool,
}

impl FakeModel {
    fn new(script: Vec<Result<String, ApiError>>) -> Arc<Self> {
        Arc::new(FakeModel {
            script: Mutex::new(script.into()),
            calls: Mutex::new(vec![]),
            glossary_json: r#"{"entries":[]}"#.to_string(),
            wait_cancel: false,
        })
    }
    fn calls(&self) -> Vec<(String, String)> {
        self.calls.lock().unwrap().clone()
    }
}

impl TextModel for FakeModel {
    fn label(&self) -> String {
        "Fake fake-model".into()
    }
    fn generate(&self, system: &str, user: &str, cancel: &AtomicBool, on_progress: &mut dyn FnMut(usize)) -> Result<String, ApiError> {
        self.calls.lock().unwrap().push((system.to_string(), user.to_string()));
        if self.wait_cancel {
            while !cancel.load(Ordering::SeqCst) {
                std::thread::sleep(Duration::from_millis(10));
            }
            return Err(ApiError::Cancelled);
        }
        on_progress(10);
        self.script.lock().unwrap().pop_front().unwrap_or_else(|| Ok(good()))
    }
    fn complete_json(&self, _system: &str, _user: &str) -> Result<String, ApiError> {
        Ok(self.glossary_json.clone())
    }
}

type Events = Arc<Mutex<Vec<SessionEvent>>>;

fn collect() -> (qt_ai_core::session::Sink, Events) {
    let events: Events = Arc::new(Mutex::new(Vec::new()));
    let sink_events = events.clone();
    let sink: qt_ai_core::session::Sink = Arc::new(move |event| sink_events.lock().unwrap().push(event));
    (sink, events)
}

fn config(root: &Path) -> ApiSessionConfig {
    ApiSessionConfig { root: root.to_path_buf(), retry_delay: Duration::from_millis(20) }
}

fn logs(events: &[SessionEvent]) -> Vec<String> {
    events.iter().filter_map(|e| match e {
        SessionEvent::AgyLog { line, .. } => Some(line.clone()),
        _ => None,
    }).collect()
}

fn status(root: &Path, id: &str) -> ChapterStatus {
    load_state(&story_paths(root)).unwrap().chapters[id].status
}

#[test]
fn dich_dat_ngay_thi_accept_ghi_out_va_finished() {
    let dir = story(2);
    let model = FakeModel::new(vec![]);
    let (sink, events) = collect();
    let handle = start_api_session(config(dir.path()), model.clone(), sink).unwrap();
    assert_eq!(handle.join(), StopReason::Finished);
    assert_eq!(status(dir.path(), "0001"), ChapterStatus::Done);
    assert_eq!(status(dir.path(), "0002"), ChapterStatus::Done);
    let out = fs::read_to_string(dir.path().join("out").join("0001.txt")).unwrap();
    assert_eq!(out, format!("{GOOD_1}\n\n{GOOD_2}\n"));
    assert!(!dir.path().join("work").join("0001.draft.md").exists(), "accept dọn work/");
    let calls = model.calls();
    assert_eq!(calls.len(), 2, "mỗi chương một lượt dịch: {calls:?}");
    assert!(calls[0].0.contains("Dịch raw text tiếng Trung"), "system prompt của qt-web");
    assert!(calls[0].1.contains("[[1]] 赵静文") && calls[0].1.contains("[[2]] 她沉默"));
    let events = events.lock().unwrap();
    assert!(matches!(events[0], SessionEvent::Started { session_no: 1 }));
    assert!(events.iter().any(|e| matches!(e, SessionEvent::Progress(p) if p.done == 2)));
    assert!(logs(&events).iter().any(|l| l.contains("0001: chốt (soát 0 lần, 0 cảnh báo")), "{:?}", logs(&events));
    assert!(matches!(events.last(), Some(SessionEvent::Stopped(StopReason::Finished))));
}

#[test]
fn thieu_doan_thi_dich_bo_sung_dung_doan_do() {
    let dir = story(1);
    let model = FakeModel::new(vec![Ok(format!("[[1]] {GOOD_1}")), Ok(format!("[[2]] {GOOD_2}"))]);
    let (sink, _) = collect();
    let handle = start_api_session(config(dir.path()), model.clone(), sink).unwrap();
    assert_eq!(handle.join(), StopReason::Finished);
    let calls = model.calls();
    assert_eq!(calls.len(), 2);
    assert!(calls[1].1.contains("thiếu các đoạn") && calls[1].1.contains("[[2]] 她沉默") && !calls[1].1.contains("[[1]]"));
    let out = fs::read_to_string(dir.path().join("out").join("0001.txt")).unwrap();
    assert_eq!(out, format!("{GOOD_1}\n\n{GOOD_2}\n"));
}

#[test]
fn vi_pham_rule_thi_soat_tren_draft_co_nhan_roi_accept() {
    let dir = story(1);
    let bad = format!("[[1]] Anh ấy ngẩng đầu nhìn về phía tòa tháp cao ở nơi xa.\n\n[[2]] {GOOD_2}");
    let model = FakeModel::new(vec![Ok(bad), Ok(good())]);
    let (sink, events) = collect();
    let handle = start_api_session(config(dir.path()), model.clone(), sink).unwrap();
    assert_eq!(handle.join(), StopReason::Finished);
    let calls = model.calls();
    assert_eq!(calls.len(), 2);
    assert!(calls[1].0.contains("soát tối thiểu") && calls[1].0.contains("GIỮ NGUYÊN mọi nhãn"));
    assert!(calls[1].1.contains("[[1]] Đại từ sai") && calls[1].1.contains("[[1]] Anh ấy"), "{}", calls[1].1);
    let state = load_state(&story_paths(dir.path())).unwrap();
    assert_eq!(state.chapters["0001"].status, ChapterStatus::Done);
    assert_eq!(state.chapters["0001"].review_round, 1);
    assert!(state.chapters["0001"].warnings.is_none());
    assert!(logs(&events.lock().unwrap()).iter().any(|l| l.contains("soát 1 lần")));
}

#[test]
fn ban_soat_te_hon_thi_bo_va_het_vong_chot_kem_canh_bao() {
    let dir = story(1);
    let bad = format!("[[1]] Anh ấy ngẩng đầu nhìn về phía tòa tháp cao ở nơi xa.\n\n[[2]] {GOOD_2}");
    // Ba vòng soát đều trả bản mất nhãn / vẫn lỗi → chốt kèm cảnh báo, không bao giờ treo.
    let model = FakeModel::new(vec![Ok(bad.clone()), Ok("không có nhãn".into()), Ok(bad.clone()), Ok(bad.clone())]);
    let (sink, _) = collect();
    let handle = start_api_session(config(dir.path()), model.clone(), sink).unwrap();
    assert_eq!(handle.join(), StopReason::Finished);
    let state = load_state(&story_paths(dir.path())).unwrap();
    assert_eq!(state.chapters["0001"].status, ChapterStatus::Done);
    assert_eq!(state.chapters["0001"].review_round, 3);
    assert!(state.chapters["0001"].warnings.as_ref().unwrap()[0].contains("Đại từ sai"));
    assert_eq!(model.calls().len(), 4);
}

#[test]
fn glossary_moi_duoc_nap_vao_story_json() {
    let dir = story(1);
    let mut model = FakeModel::new(vec![]);
    Arc::get_mut(&mut model).unwrap().glossary_json =
        r#"{"entries":[{"source":"赵静文","target":"Triệu Tĩnh Văn","category":"names"}]}"#.to_string();
    let (sink, events) = collect();
    let handle = start_api_session(config(dir.path()), model, sink).unwrap();
    assert_eq!(handle.join(), StopReason::Finished);
    let story = load_story_config(&story_paths(dir.path())).unwrap();
    assert_eq!(story.glossary["names"]["赵静文"], "Triệu Tĩnh Văn");
    assert!(logs(&events.lock().unwrap()).iter().any(|l| l.contains("+1 glossary")));
}

#[test]
fn model_tu_choi_thi_skip_kem_ly_do_va_di_tiep() {
    let dir = story(2);
    let model = FakeModel::new(vec![Err(ApiError::Blocked("PROHIBITED_CONTENT".into()))]);
    let (sink, _) = collect();
    let handle = start_api_session(config(dir.path()), model.clone(), sink).unwrap();
    assert_eq!(handle.join(), StopReason::Finished);
    let state = load_state(&story_paths(dir.path())).unwrap();
    assert_eq!(state.chapters["0001"].status, ChapterStatus::Skipped);
    assert_eq!(state.chapters["0001"].reason.as_deref(), Some("model từ chối: PROHIBITED_CONTENT"));
    assert_eq!(state.chapters["0002"].status, ChapterStatus::Done);
    assert_eq!(model.calls().len(), 2, "từ chối không thử lại");
}

#[test]
fn loi_mang_thu_lai_mot_lan_roi_skip_hai_chuong_lien_tiep_thi_dung() {
    let dir = story(3);
    let net = || Err(ApiError::Network { provider: "Gemini", message: "timeout".into() });
    let model = FakeModel::new(vec![net(), net(), net(), net()]);
    let (sink, events) = collect();
    let handle = start_api_session(config(dir.path()), model.clone(), sink).unwrap();
    assert_eq!(handle.join(), StopReason::ApiFailed { message: "Gemini không kết nối được: timeout".into() });
    let state = load_state(&story_paths(dir.path())).unwrap();
    assert_eq!(state.chapters["0001"].status, ChapterStatus::Skipped);
    assert!(state.chapters["0001"].reason.as_deref().unwrap().starts_with("lỗi API: "));
    // Chương thứ hai lỗi liên tiếp → dừng ngay, giữ translating để phiên sau làm tiếp.
    assert_eq!(state.chapters["0002"].status, ChapterStatus::Translating);
    assert_eq!(state.chapters["0003"].status, ChapterStatus::Queued, "dừng trước khi đốt chương 3");
    assert_eq!(model.calls().len(), 4);
    assert!(logs(&events.lock().unwrap()).iter().any(|l| l.contains("thử lại sau")));
}

#[test]
fn loi_mang_mot_lan_roi_thanh_cong_thi_khong_tinh_that_bai() {
    let dir = story(1);
    let model = FakeModel::new(vec![Err(ApiError::Http { provider: "OpenAI", status: 429, message: "slow down".into() })]);
    let (sink, _) = collect();
    let handle = start_api_session(config(dir.path()), model.clone(), sink).unwrap();
    assert_eq!(handle.join(), StopReason::Finished);
    assert_eq!(status(dir.path(), "0001"), ChapterStatus::Done);
    assert_eq!(model.calls().len(), 2);
}

#[test]
fn cancel_giu_chuong_translating_va_phien_sau_lam_tiep_chuong_do() {
    let dir = story(2);
    let mut model = FakeModel::new(vec![]);
    Arc::get_mut(&mut model).unwrap().wait_cancel = true;
    let (sink, _) = collect();
    let handle = start_api_session(config(dir.path()), model, sink).unwrap();
    std::thread::sleep(Duration::from_millis(100));
    handle.cancel();
    assert_eq!(handle.join(), StopReason::UserCancelled);
    assert_eq!(status(dir.path(), "0001"), ChapterStatus::Translating);
    assert!(!dir.path().join("work").join(".session.lock").exists());

    let model = FakeModel::new(vec![]);
    let (sink, _) = collect();
    let handle = start_api_session(config(dir.path()), model.clone(), sink).unwrap();
    assert_eq!(handle.join(), StopReason::Finished);
    assert!(model.calls()[0].1.contains("[[1]] 赵静文"));
    assert_eq!(status(dir.path(), "0001"), ChapterStatus::Done);
    assert_eq!(status(dir.path(), "0002"), ChapterStatus::Done);
}

#[test]
fn hang_doi_rong_thi_finished_ngay_va_lock_chan_phien_thu_hai() {
    let dir = story(0);
    let (sink, _) = collect();
    let handle = start_api_session(config(dir.path()), FakeModel::new(vec![]), sink).unwrap();
    assert_eq!(handle.join(), StopReason::Finished);
    assert_eq!(labeled_draft(&["a".into(), "b".into()]), "[[1]] a\n\n[[2]] b\n");
}
