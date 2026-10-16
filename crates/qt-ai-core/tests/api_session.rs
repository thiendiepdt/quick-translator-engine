//! Vòng dịch API trên tempdir với model giả (không HTTP).
use qt_ai_core::api::{fix_encoding, ApiError, ApiStep, Generated, TextModel, Usage};
use qt_ai_core::api_session::*;
use qt_ai_core::commands::init::run_init;
use qt_ai_core::session::{SessionEvent, StopReason};
use qt_ai_core::story_fs::{load_state, load_story_config, save_story_config, story_paths, work_file, ChapterStatus, WorkKind};
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
    /// Bỏ hậu kiểm mã hoá — như agent/động cơ không qua HttpModel: chữ lỗi mã hoá lọt tới check.
    raw_encoding: bool,
    /// Usage giả mỗi lượt (None = hub không báo).
    usage: Option<Usage>,
}

impl FakeModel {
    fn new(script: Vec<Result<String, ApiError>>) -> Arc<Self> {
        Arc::new(FakeModel {
            script: Mutex::new(script.into()),
            calls: Mutex::new(vec![]),
            glossary_json: r#"{"entries":[]}"#.to_string(),
            wait_cancel: false,
            raw_encoding: false,
            usage: None,
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
    fn generate(&self, _: ApiStep, system: &str, user: &str, cancel: &AtomicBool, on_progress: &mut dyn FnMut(usize)) -> Result<Generated, ApiError> {
        self.calls.lock().unwrap().push((system.to_string(), user.to_string()));
        if self.wait_cancel {
            while !cancel.load(Ordering::SeqCst) {
                std::thread::sleep(Duration::from_millis(10));
            }
            return Err(ApiError::Cancelled);
        }
        on_progress(10);
        let text = self.script.lock().unwrap().pop_front().unwrap_or_else(|| Ok(good()))?;
        let generated = Generated { text, usage: self.usage, repaired_encoding: false };
        if self.raw_encoding {
            return Ok(generated);
        }
        // Như HttpModel: mọi kết quả qua hậu kiểm mã hoá.
        fix_encoding(generated)
    }
    fn complete_json(&self, _: ApiStep, _system: &str, _user: &str, _cancel: &AtomicBool) -> Result<Generated, ApiError> {
        fix_encoding(Generated { text: self.glossary_json.clone(), usage: self.usage, repaired_encoding: false })
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
    ApiSessionConfig { root: root.to_path_buf(), retry_delay: Duration::from_millis(5), retry_max: Duration::from_millis(20) }
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
    // Ba vòng soát đều trả bản mất nhãn / đổi chữ nhưng vẫn lỗi → chốt kèm cảnh báo, không bao giờ treo.
    let bad2 = bad.replace("Anh ấy", "Cô ấy");
    let bad3 = bad.replace("Anh ấy", "Chị ấy");
    let model = FakeModel::new(vec![Ok(bad.clone()), Ok("không có nhãn".into()), Ok(bad2), Ok(bad3)]);
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
fn ban_soat_y_het_ban_cu_thi_chot_kem_canh_bao_ngay_khong_dot_them_vong() {
    let dir = story(1);
    // "não hải" dính rule trung lập; model xét ngữ cảnh và giữ nguyên → lặp thêm chỉ tốn tiền.
    let bad = format!("[[1]] Trong não hải hắn hiện lên tòa tháp cao.\n\n[[2]] {GOOD_2}");
    let model = FakeModel::new(vec![Ok(bad.clone()), Ok(bad.clone())]);
    let (sink, events) = collect();
    let handle = start_api_session(config(dir.path()), model.clone(), sink).unwrap();
    assert_eq!(handle.join(), StopReason::Finished);
    assert_eq!(model.calls().len(), 2, "dịch + đúng một lượt soát: {:?}", model.calls().len());
    let state = load_state(&story_paths(dir.path())).unwrap();
    assert_eq!(state.chapters["0001"].status, ChapterStatus::Done);
    assert_eq!(state.chapters["0001"].review_round, 1);
    assert!(state.chapters["0001"].warnings.as_ref().unwrap()[0].contains("não hải"));
    let out = fs::read_to_string(dir.path().join("out").join("0001.txt")).unwrap();
    assert!(out.starts_with("Trong não hải hắn"));
    let lines = logs(&events.lock().unwrap());
    assert!(lines.iter().any(|l| l.contains("model giữ nguyên")), "{lines:?}");
    assert!(lines.iter().any(|l| l.contains("soát 1 lần, 1 cảnh báo")), "{lines:?}");
}

/// Dựng mojibake như hub hỏng: mỗi byte UTF-8 đọc thành một ký tự Windows-1252 (byte chưa định nghĩa → U+0081…).
fn to_mojibake(text: &str) -> String {
    const HIGH: [char; 32] = [
        '\u{20AC}', '\u{81}', '\u{201A}', '\u{192}', '\u{201E}', '\u{2026}', '\u{2020}', '\u{2021}', '\u{2C6}', '\u{2030}',
        '\u{160}', '\u{2039}', '\u{152}', '\u{8D}', '\u{17D}', '\u{8F}', '\u{90}', '\u{2018}', '\u{2019}', '\u{201C}',
        '\u{201D}', '\u{2022}', '\u{2013}', '\u{2014}', '\u{2DC}', '\u{2122}', '\u{161}', '\u{203A}', '\u{153}', '\u{9D}',
        '\u{17E}', '\u{178}',
    ];
    text.bytes().map(|b| if (0x80..0xA0).contains(&b) { HIGH[(b - 0x80) as usize] } else { b as char }).collect()
}

#[test]
fn hub_tra_mojibake_sua_duoc_thi_log_va_chot_ban_dung() {
    let dir = story(1);
    let model = FakeModel::new(vec![Ok(to_mojibake(&good()))]);
    let (sink, events) = collect();
    let handle = start_api_session(config(dir.path()), model.clone(), sink).unwrap();
    assert_eq!(handle.join(), StopReason::Finished);
    assert_eq!(model.calls().len(), 1, "sửa tại chỗ, không tốn lượt gọi lại");
    let out = fs::read_to_string(dir.path().join("out").join("0001.txt")).unwrap();
    assert_eq!(out, format!("{GOOD_1}\n\n{GOOD_2}\n"));
    let lines = logs(&events.lock().unwrap());
    assert!(lines.iter().any(|l| l == "0001: phản hồi API bị lỗi mã hoá — đã tự sửa"), "{lines:?}");
}

#[test]
fn hub_tra_mojibake_khong_sua_duoc_thi_goi_lai_dung_luot_do_khong_bo_chuong() {
    let dir = story(1);
    let broken = format!("[[1]] {} â€ x â€ y â€ z\n\n[[2]] {}", to_mojibake(GOOD_1), to_mojibake(GOOD_2));
    let model = FakeModel::new(vec![Ok(broken.clone()), Ok(broken), Ok(good())]);
    let (sink, events) = collect();
    let handle = start_api_session(config(dir.path()), model.clone(), sink).unwrap();
    assert_eq!(handle.join(), StopReason::Finished);
    assert_eq!(model.calls().len(), 3, "gọi lại lượt dịch tới khi hub trả chữ đúng");
    assert_eq!(status(dir.path(), "0001"), ChapterStatus::Done);
    let out = fs::read_to_string(dir.path().join("out").join("0001.txt")).unwrap();
    assert_eq!(out, format!("{GOOD_1}\n\n{GOOD_2}\n"));
    let lines = logs(&events.lock().unwrap());
    assert!(lines.iter().any(|l| l.contains("mojibake") && l.contains("thử lại lần 2")), "{lines:?}");
}

#[test]
fn draft_con_loi_ma_hoa_thi_dich_lai_tu_raw_dung_doan_do_khong_dua_vao_soat() {
    let dir = story(1);
    let mut model = FakeModel::new(vec![Ok(format!("[[1]] {GOOD_1}\n\n[[2]] {}", to_mojibake(GOOD_2))), Ok(format!("[[2]] {GOOD_2}"))]);
    Arc::get_mut(&mut model).unwrap().raw_encoding = true;
    let (sink, events) = collect();
    let handle = start_api_session(config(dir.path()), model.clone(), sink).unwrap();
    assert_eq!(handle.join(), StopReason::Finished);
    let calls = model.calls();
    assert_eq!(calls.len(), 2, "{calls:?}");
    assert!(calls[1].0.contains("Dịch raw text tiếng Trung"), "dịch lại bằng prompt dịch, không phải soát");
    assert!(calls[1].1.contains("[[2]] 她沉默") && !calls[1].1.contains("[[1]]"), "{}", calls[1].1);
    let out = fs::read_to_string(dir.path().join("out").join("0001.txt")).unwrap();
    assert_eq!(out, format!("{GOOD_1}\n\n{GOOD_2}\n"));
    let lines = logs(&events.lock().unwrap());
    assert!(lines.iter().any(|l| l == "0001: chữ lỗi mã hoá ở 1 đoạn — dịch lại"), "{lines:?}");
}

#[test]
fn con_han_tu_thi_dich_lai_dung_doan_do_khong_dua_vao_soat() {
    let dir = story(1);
    let bad = format!("[[1]] Triệu Tĩnh Văn tai濡目染 ngẩng đầu nhìn về phía tòa tháp cao.\n\n[[2]] {GOOD_2}");
    // Model soát kiểu cũ sẽ trả y nguyên → trước đây chốt kèm cảnh báo, Hán tự lọt ra out.
    let model = FakeModel::new(vec![Ok(bad.clone()), Ok(format!("[[1]] {GOOD_1}"))]);
    let (sink, events) = collect();
    let handle = start_api_session(config(dir.path()), model.clone(), sink).unwrap();
    assert_eq!(handle.join(), StopReason::Finished);
    let calls = model.calls();
    assert_eq!(calls.len(), 2);
    assert!(calls[1].1.contains("[[1]] 赵静文抬头看向远方的高塔。"), "gửi lại raw đoạn 1: {}", calls[1].1);
    assert!(!calls[1].1.contains("[[2]]"), "chỉ dịch lại đoạn còn Hán: {}", calls[1].1);
    let state = load_state(&story_paths(dir.path())).unwrap();
    assert_eq!(state.chapters["0001"].status, ChapterStatus::Done);
    assert!(state.chapters["0001"].warnings.is_none());
    let out = fs::read_to_string(dir.path().join("out").join("0001.txt")).unwrap();
    assert!(out.starts_with(GOOD_1), "{out}");
    let lines = logs(&events.lock().unwrap());
    assert!(lines.iter().any(|l| l.contains("0001: còn chữ Hán ở 1 đoạn — dịch lại")), "{lines:?}");
}

#[test]
fn han_tu_dich_lai_mai_van_con_thi_error_khong_ghi_out() {
    let dir = story(1);
    let bad = format!("[[1]] Triệu Tĩnh Văn tai濡目染 ngẩng đầu nhìn về phía tòa tháp cao.\n\n[[2]] {GOOD_2}");
    let still = "[[1]] Triệu Tĩnh Văn tai濡目染 ngẩng đầu nhìn về phía tòa tháp cao.".to_string();
    let model = FakeModel::new(vec![Ok(bad), Ok(still.clone()), Ok(still.clone()), Ok(still)]);
    let (sink, _) = collect();
    let handle = start_api_session(config(dir.path()), model.clone(), sink).unwrap();
    assert_eq!(handle.join(), StopReason::Finished);
    assert_eq!(model.calls().len(), 4, "dịch + 3 lượt dịch lại đoạn còn Hán");
    let state = load_state(&story_paths(dir.path())).unwrap();
    assert_eq!(state.chapters["0001"].status, ChapterStatus::Error);
    assert!(!dir.path().join("out").join("0001.txt").exists());
}

#[test]
fn dich_lai_doan_han_bi_chan_thi_error_khong_chot_kem_canh_bao() {
    let dir = story(1);
    let bad = format!("[[1]] Triệu Tĩnh Văn tai濡目染 ngẩng đầu nhìn về phía tòa tháp cao.\n\n[[2]] {GOOD_2}");
    let blocked = || Err(ApiError::Blocked("content_filter".into()));
    let model = FakeModel::new(vec![Ok(bad), blocked(), blocked(), blocked()]);
    let (sink, _) = collect();
    let handle = start_api_session(config(dir.path()), model.clone(), sink).unwrap();
    assert_eq!(handle.join(), StopReason::Finished);
    let state = load_state(&story_paths(dir.path())).unwrap();
    assert_eq!(state.chapters["0001"].status, ChapterStatus::Error, "{:?}", state.chapters["0001"]);
    assert!(!dir.path().join("out").join("0001.txt").exists());
}

#[test]
fn ban_soat_dua_han_tu_vao_thi_bo_du_giam_tong_vi_pham() {
    let dir = story(1);
    // 2 vi phạm mềm → bản soát còn 1 vi phạm nhưng là Hán tự: tổng giảm vẫn phải bỏ, Hán tự nặng hơn mọi lỗi mềm.
    let bad = format!("[[1]] Anh ấy ngẩng đầu, não hải trống rỗng, nhìn tòa tháp cao nơi xa.\n\n[[2]] {GOOD_2}");
    let worse = format!("[[1]] Hắn ngẩng đầu, 脑海 trống rỗng, nhìn tòa tháp cao nơi xa.\n\n[[2]] {GOOD_2}");
    let model = FakeModel::new(vec![Ok(bad), Ok(worse), Ok(good())]);
    let (sink, events) = collect();
    let handle = start_api_session(config(dir.path()), model.clone(), sink).unwrap();
    assert_eq!(handle.join(), StopReason::Finished);
    let lines = logs(&events.lock().unwrap());
    assert!(lines.iter().any(|l| l.contains("bản soát không giảm vi phạm")), "{lines:?}");
    let out = fs::read_to_string(dir.path().join("out").join("0001.txt")).unwrap();
    assert!(!out.contains("脑海"), "{out}");
}

#[test]
fn ten_viet_gan_dung_glossary_model_giu_nguyen_thi_soat_tiep_khong_chot_som() {
    let dir = story(1);
    let paths = story_paths(dir.path());
    let mut story = load_story_config(&paths).unwrap();
    story.glossary.entry("names".to_string()).or_default().insert("赵静文".into(), "Triệu Tĩnh Văn".into());
    save_story_config(&paths, &story).unwrap();
    let wrong = format!("[[1]] Triệu Tịch Văn ngẩng đầu nhìn về phía tòa tháp cao ở nơi xa.\n\n[[2]] {GOOD_2}");
    // Dịch → soát 1 model giữ nguyên tên sai → soát 2 sửa đúng.
    let model = FakeModel::new(vec![Ok(wrong.clone()), Ok(wrong), Ok(good())]);
    let (sink, events) = collect();
    let handle = start_api_session(config(dir.path()), model.clone(), sink).unwrap();
    assert_eq!(handle.join(), StopReason::Finished);
    let calls = model.calls();
    assert_eq!(calls.len(), 3, "dịch + hai lượt soát");
    assert!(calls[1].1.contains("Tên theo từ điển — BẮT BUỘC sửa"), "{}", calls[1].1);
    assert!(calls[1].1.contains("赵静文 là `Triệu Tĩnh Văn`, bản dịch viết `Triệu Tịch Văn`"), "{}", calls[1].1);
    let logs = format!("{:?}", events.lock().unwrap());
    assert!(logs.contains("model giữ nguyên tên sai so với từ điển — soát tiếp"), "{logs}");
    let state = load_state(&paths).unwrap();
    assert_eq!(state.chapters["0001"].status, ChapterStatus::Done);
    assert!(state.chapters["0001"].warnings.is_none(), "{:?}", state.chapters["0001"].warnings);
    let out = fs::read_to_string(dir.path().join("out").join("0001.txt")).unwrap();
    assert!(out.starts_with(GOOD_1), "{out}");
}

#[test]
fn ten_lech_dau_glossary_thi_code_tu_sua_khong_ton_luot_soat() {
    let dir = story(1);
    let paths = story_paths(dir.path());
    let mut story = load_story_config(&paths).unwrap();
    story.glossary.entry("names".to_string()).or_default().insert("赵静文".into(), "Triệu Tĩnh Văn".into());
    save_story_config(&paths, &story).unwrap();
    let drift = format!("[[1]] Triệu Tịnh Văn ngẩng đầu nhìn về phía tòa tháp cao ở nơi xa.\n\n[[2]] {GOOD_2}");
    let model = FakeModel::new(vec![Ok(drift)]);
    let (sink, events) = collect();
    let handle = start_api_session(config(dir.path()), model.clone(), sink).unwrap();
    assert_eq!(handle.join(), StopReason::Finished);
    assert_eq!(model.calls().len(), 1, "chỉ lượt dịch, không có lượt soát");
    let logs = format!("{:?}", events.lock().unwrap());
    assert!(logs.contains("tự sửa theo glossary: Triệu Tịnh Văn → Triệu Tĩnh Văn"), "{logs}");
    let state = load_state(&paths).unwrap();
    assert_eq!(state.chapters["0001"].status, ChapterStatus::Done);
    assert!(state.chapters["0001"].warnings.is_none(), "{:?}", state.chapters["0001"].warnings);
    let out = fs::read_to_string(dir.path().join("out").join("0001.txt")).unwrap();
    assert!(out.starts_with(GOOD_1), "{out}");
}

#[test]
fn sai_gioi_theo_cast_thi_soat_va_nhan_ban_sua() {
    let dir = story(1);
    let paths = story_paths(dir.path());
    fs::write(dir.path().join("raw").join("0001.txt"), "赵静文抬头看向远方的高塔。\n\n“赵老师沉默了很久没有说话。”").unwrap();
    let mut story = load_story_config(&paths).unwrap();
    story.glossary.entry("names".to_string()).or_default().insert("赵静文".into(), "Triệu Tĩnh Văn".into());
    save_story_config(&paths, &story).unwrap();
    let mut cast = qt_ai_core::cast::Cast::default();
    cast.characters.insert(
        "赵静文".into(),
        qt_ai_core::cast::Character { gender: Some(qt_ai_core::cast::Gender::Female), ..Default::default() },
    );
    qt_ai_core::cast::save_cast(&paths, &cast).unwrap();
    let wrong = format!("[[1]] {GOOD_1}\n\n[[2]] “Thầy Triệu im lặng rất lâu, không nói một lời nào.”");
    let fixed = format!("[[1]] {GOOD_1}\n\n[[2]] “Cô Triệu im lặng rất lâu, không nói một lời nào.”");
    let model = FakeModel::new(vec![Ok(wrong), Ok(fixed)]);
    let (sink, _) = collect();
    let handle = start_api_session(config(dir.path()), model.clone(), sink).unwrap();
    assert_eq!(handle.join(), StopReason::Finished);
    let calls = model.calls();
    assert_eq!(calls.len(), 2, "dịch + một lượt soát");
    assert!(calls[0].0.contains("- 赵静文 (Triệu Tĩnh Văn): nữ"), "prompt dịch có bảng nhân vật");
    assert!(calls[1].1.contains("[[2]] Sai giới: 赵老师 là 赵静文 (nữ) → `cô Triệu`"), "{}", calls[1].1);
    let state = load_state(&paths).unwrap();
    assert_eq!(state.chapters["0001"].status, ChapterStatus::Done);
    assert!(state.chapters["0001"].warnings.is_none(), "{:?}", state.chapters["0001"].warnings);
    let out = fs::read_to_string(dir.path().join("out").join("0001.txt")).unwrap();
    assert!(out.contains("Cô Triệu"), "{out}");
}

#[test]
fn khoi_glossary_khai_gioi_va_doi_cap_thi_ghi_cast_json_va_log() {
    let dir = story(1);
    let paths = story_paths(dir.path());
    fs::write(dir.path().join("raw").join("0001.txt"), "赵静文抬头看向李明。\n\n她沉默了很久没有说话。").unwrap();
    let mut story = load_story_config(&paths).unwrap();
    story.glossary.entry("names".to_string()).or_default().insert("李明".into(), "Lý Minh".into());
    story.glossary.entry("addressing".to_string()).or_default().insert("赵静文→李明".into(), "ta–ngươi".into());
    save_story_config(&paths, &story).unwrap();
    let with_block = format!(
        "{}\n\n[[glossary]]\n{{\"entries\":[{{\"source\":\"赵静文\",\"target\":\"Triệu Tĩnh Văn\",\"category\":\"names\",\"gender\":\"nữ\"}},{{\"source\":\"赵静文→李明\",\"target\":\"muội–huynh\",\"category\":\"addressing\",\"note\":\"thành người yêu\"}}]}}",
        good()
    );
    let model = FakeModel::new(vec![Ok(with_block)]);
    let (sink, events) = collect();
    let handle = start_api_session(config(dir.path()), model.clone(), sink).unwrap();
    assert_eq!(handle.join(), StopReason::Finished);
    let calls = model.calls();
    assert!(calls[0].1.contains("\"gender\": \"nam\" hoặc \"nữ\""), "chỉ dẫn khai giới: {}", calls[0].1);
    assert!(calls[0].1.contains("quan hệ hai người đã khác lúc chốt cặp") && calls[0].1.contains("\"note\""), "chỉ dẫn khai đổi cặp");
    let cast = qt_ai_core::cast::load_cast(&paths);
    assert_eq!(cast.gender_of("赵静文"), Some(qt_ai_core::cast::Gender::Female));
    assert_eq!(cast.addressing["赵静文→李明"].changes[0].target, "muội–huynh");
    let lines = logs(&events.lock().unwrap());
    assert!(lines.iter().any(|l| l.contains("0001: đổi xưng hô 赵静文→李明: ta–ngươi → muội–huynh (thành người yêu)")), "{lines:?}");
}

#[test]
fn log_token_tung_luot_va_tong_khi_chot() {
    let dir = story(1);
    let bad = format!("[[1]] Anh ấy ngẩng đầu nhìn về phía tòa tháp cao ở nơi xa.\n\n[[2]] {GOOD_2}");
    let mut model = FakeModel::new(vec![Ok(bad), Ok(good())]);
    Arc::get_mut(&mut model).unwrap().usage = Some(Usage { input: 1000, cached: 600, output: 200, thoughts: 50 });
    let (sink, events) = collect();
    let handle = start_api_session(config(dir.path()), model.clone(), sink).unwrap();
    assert_eq!(handle.join(), StopReason::Finished);
    let lines = logs(&events.lock().unwrap());
    assert!(lines.iter().any(|l| l == "0001: dịch — vào 1.0k (cache 600) · ra 200 · nghĩ 50"), "{lines:?}");
    assert!(lines.iter().any(|l| l.starts_with("0001: trích glossary — vào")), "{lines:?}");
    assert!(lines.iter().any(|l| l.starts_with("0001: soát lần 1 — vào")), "{lines:?}");
    // 3 lượt × 1250 token = 3750 → "3.8k".
    assert!(lines.iter().any(|l| l.contains("0001: chốt (soát 1 lần, 0 cảnh báo, +0 glossary) · 3.8k token")), "{lines:?}");
}

#[test]
fn hub_khong_bao_usage_thi_khong_log_token() {
    let dir = story(1);
    let model = FakeModel::new(vec![]);
    let (sink, events) = collect();
    let handle = start_api_session(config(dir.path()), model, sink).unwrap();
    assert_eq!(handle.join(), StopReason::Finished);
    let lines = logs(&events.lock().unwrap());
    assert!(lines.iter().any(|l| l.ends_with("+0 glossary)")), "{lines:?}");
    assert!(!lines.iter().any(|l| l.contains(" token") || l.contains(" — vào")), "{lines:?}");
}

#[test]
fn glossary_kem_luot_dich_thi_khong_goi_trich_rieng() {
    let dir = story(1);
    let with_block = format!(
        "{}\n\n[[glossary]]\n{{\"entries\":[{{\"source\":\"赵静文\",\"target\":\"Triệu Tĩnh Văn\",\"category\":\"names\"}}]}}",
        good()
    );
    let mut model = FakeModel::new(vec![Ok(with_block)]);
    // Nếu app vẫn gọi lượt trích riêng thì glossary này sẽ lọt vào story.json.
    Arc::get_mut(&mut model).unwrap().glossary_json =
        r#"{"entries":[{"source":"高塔","target":"Cao Tháp","category":"places"}]}"#.to_string();
    let (sink, events) = collect();
    let handle = start_api_session(config(dir.path()), model.clone(), sink).unwrap();
    assert_eq!(handle.join(), StopReason::Finished);
    let calls = model.calls();
    assert_eq!(calls.len(), 1);
    assert!(calls[0].1.contains("[[glossary]]") && calls[0].1.contains("\"addressing\""), "payload dịch yêu cầu khối glossary");
    assert!(calls[0].1.contains("phải phiên cùng âm"), "chỉ dẫn glossary: tên mới theo âm entry sẵn có");
    assert!(
        calls[0].1.contains("Cặp xưng hô khai cả khi không có tên mới; chỉ trả {\"entries\": []} khi không có cả tên mới lẫn cặp xưng hô mới."),
        "cặp xưng hô không bị bỏ khi chương không có tên mới"
    );
    let out = fs::read_to_string(dir.path().join("out").join("0001.txt")).unwrap();
    assert_eq!(out, format!("{GOOD_1}\n\n{GOOD_2}\n"), "khối glossary không lọt vào bản dịch");
    let story = load_story_config(&story_paths(dir.path())).unwrap();
    assert_eq!(story.glossary["names"]["赵静文"], "Triệu Tĩnh Văn");
    assert!(story.glossary.get("places").and_then(|p| p.get("高塔")).is_none(), "không có lượt trích riêng");
    let lines = logs(&events.lock().unwrap());
    assert!(lines.iter().any(|l| l.contains("glossary kèm lượt dịch — 1 đề xuất")), "{lines:?}");
    assert!(lines.iter().any(|l| l.contains("+1 glossary")), "{lines:?}");
}

#[test]
fn split_glossary_block_tach_khoi_cuoi_va_chiu_rao_markdown() {
    let (text, entries) = split_glossary_block("[[1]] a\n\n[[2]] b\n\n[[glossary]]\n```json\n{\"entries\":[{\"source\":\"x\"}]}\n```");
    assert_eq!(text, "[[1]] a\n\n[[2]] b");
    assert_eq!(entries.unwrap()[0]["source"], "x");
    let (text, entries) = split_glossary_block("[[1]] a\n\n[[glossary]]\nkhông phải json");
    assert_eq!(text, "[[1]] a");
    assert!(entries.is_none());
    let (text, entries) = split_glossary_block("[[1]] a");
    assert_eq!((text, entries.is_none()), ("[[1]] a", true));
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
fn soat_bi_tu_choi_thi_chot_kem_canh_bao_khong_skip_chuong_da_dich() {
    let dir = story(1);
    let bad = format!("[[1]] Anh ấy ngẩng đầu nhìn về phía tòa tháp cao ở nơi xa.\n\n[[2]] {GOOD_2}");
    let model = FakeModel::new(vec![Ok(bad), Err(ApiError::Blocked("content_filter".into()))]);
    let (sink, events) = collect();
    let handle = start_api_session(config(dir.path()), model.clone(), sink).unwrap();
    assert_eq!(handle.join(), StopReason::Finished);
    assert_eq!(model.calls().len(), 2, "dịch + một lượt soát bị chặn, không soát thêm");
    let state = load_state(&story_paths(dir.path())).unwrap();
    assert_eq!(state.chapters["0001"].status, ChapterStatus::Done);
    assert!(state.chapters["0001"].warnings.is_some());
    let lines = logs(&events.lock().unwrap());
    assert!(lines.iter().any(|l| l.contains("0001: model từ chối soát (content_filter) — chốt kèm cảnh báo")), "{lines:?}");
    assert!(lines.iter().any(|l| l.contains("soát 1 lần, 1 cảnh báo")), "{lines:?}");
}

#[test]
fn dich_bo_sung_bi_tu_choi_thi_giu_nguyen_van_han_doan_thieu() {
    let dir = story(1);
    let model = FakeModel::new(vec![Ok(format!("[[1]] {GOOD_1}")), Err(ApiError::Blocked("SAFETY".into()))]);
    let (sink, events) = collect();
    let handle = start_api_session(config(dir.path()), model.clone(), sink).unwrap();
    assert_eq!(handle.join(), StopReason::Finished);
    let state = load_state(&story_paths(dir.path())).unwrap();
    assert_ne!(state.chapters["0001"].status, ChapterStatus::Skipped, "{:?}", state.chapters["0001"]);
    let lines = logs(&events.lock().unwrap());
    assert!(lines.iter().any(|l| l.contains("0001: dịch bổ sung bị từ chối (SAFETY) — giữ nguyên văn Hán đoạn thiếu")), "{lines:?}");
}

#[test]
fn model_tra_khong_nhan_thi_ly_do_skip_va_log_co_dau_output() {
    let dir = story(1);
    let refusal = "Xin lỗi, tôi không thể dịch nội dung này vì có đề cập tới tự sát.\nBạn có thể cung cấp đoạn khác.";
    // Hai lượt trong translate_full + một lượt thử lại ngoài = 4 lần gọi đều không nhãn.
    let model = FakeModel::new((0..4).map(|_| Ok(refusal.to_string())).collect());
    let (sink, events) = collect();
    let handle = start_api_session(config(dir.path()), model.clone(), sink).unwrap();
    assert_eq!(handle.join(), StopReason::Finished);
    let state = load_state(&story_paths(dir.path())).unwrap();
    assert_eq!(state.chapters["0001"].status, ChapterStatus::Skipped);
    let reason = state.chapters["0001"].reason.clone().unwrap();
    assert!(reason.contains("model trả: «Xin lỗi, tôi không thể dịch nội dung này vì có đề cập tới tự sát. Bạn") && reason.ends_with("…»"), "{reason}");
    let logs = logs(&events.lock().unwrap());
    assert!(logs.iter().any(|l| l.contains("0001: model trả không nhãn, đầu output: «Xin lỗi")), "{logs:?}");
}

#[test]
fn thoi_gian_cho_goi_lai_tang_gap_doi_toi_tran_roi_giu_tran() {
    let wait = |attempt| qt_ai_core::api_session::retry_wait(Duration::from_secs(5), Duration::from_secs(60), attempt);
    let got: Vec<u64> = (1..=7).map(|attempt| wait(attempt).as_secs()).collect();
    assert_eq!(got, vec![5, 10, 20, 40, 60, 60, 60]);
    assert_eq!(wait(1000).as_secs(), 60, "không tràn số khi provider chết lâu");
}

#[test]
fn provider_chet_lau_thi_goi_lai_toi_khi_song_khong_bo_qua_chuong() {
    let dir = story(2);
    let errors: Vec<Result<String, ApiError>> = vec![
        Err(ApiError::Network { provider: "Gemini", message: "timeout".into() }),
        Err(ApiError::Http { provider: "Gemini", status: 503, message: "model quá tải".into() }),
        Err(ApiError::Http { provider: "Gemini", status: 429, message: "hết hạn mức phút".into() }),
        Err(ApiError::Stream("đứt giữa chừng".into())),
        Err(ApiError::Http { provider: "Gemini", status: 503, message: "model quá tải".into() }),
        Err(ApiError::Http { provider: "Gemini", status: 503, message: "model quá tải".into() }),
        Err(ApiError::Http { provider: "Gemini", status: 503, message: "model quá tải".into() }),
    ];
    let model = FakeModel::new(errors);
    let (sink, events) = collect();
    let handle = start_api_session(config(dir.path()), model.clone(), sink).unwrap();
    assert_eq!(handle.join(), StopReason::Finished);
    assert_eq!(status(dir.path(), "0001"), ChapterStatus::Done, "không bỏ qua chương vì provider chết");
    assert_eq!(status(dir.path(), "0002"), ChapterStatus::Done);
    assert_eq!(model.calls().len(), 9, "7 lượt lỗi + 2 chương dịch được");
    let lines = logs(&events.lock().unwrap());
    assert!(lines.iter().any(|l| l == "0001: dịch: Gemini trả 503: model quá tải — thử lại lần 7 sau 20 ms"), "{lines:?}");
    assert!(lines.iter().any(|l| l.contains("thử lại lần 1 sau 5 ms")), "{lines:?}");
    assert!(!lines.iter().any(|l| l.contains("bỏ qua")), "{lines:?}");
}

#[test]
fn loi_tam_thoi_o_buoc_soat_chi_goi_lai_buoc_soat_khong_dich_lai_ca_chuong() {
    let dir = story(1);
    let bad = format!("[[1]] Anh ấy ngẩng đầu nhìn về phía tòa tháp cao ở nơi xa.\n\n[[2]] {GOOD_2}");
    let overloaded = || Err(ApiError::Http { provider: "Gemini", status: 503, message: "model quá tải".into() });
    let model = FakeModel::new(vec![Ok(bad), overloaded(), overloaded(), Ok(good())]);
    let (sink, _) = collect();
    let handle = start_api_session(config(dir.path()), model.clone(), sink).unwrap();
    assert_eq!(handle.join(), StopReason::Finished);
    assert_eq!(status(dir.path(), "0001"), ChapterStatus::Done);
    let calls = model.calls();
    assert_eq!(calls.len(), 4, "1 lượt dịch + 3 lượt soát (2 lỗi), không dịch lại cả chương");
    assert!(calls[1..].iter().all(|(system, user)| system.contains("soát tối thiểu") && !user.contains("[[1]] 赵静文")), "các lượt sau đều là soát");
}

#[test]
fn bam_dung_khi_dang_cho_goi_lai_thi_dung_ngay_chuong_giu_translating() {
    let dir = story(1);
    let errors = (0..100).map(|_| Err(ApiError::Network { provider: "Gemini", message: "timeout".into() })).collect();
    let model = FakeModel::new(errors);
    let (sink, _) = collect();
    let slow = ApiSessionConfig { root: dir.path().to_path_buf(), retry_delay: Duration::from_secs(60), retry_max: Duration::from_secs(60) };
    let handle = start_api_session(slow, model.clone(), sink).unwrap();
    std::thread::sleep(Duration::from_millis(150));
    let started = std::time::Instant::now();
    handle.cancel();
    assert_eq!(handle.join(), StopReason::UserCancelled);
    assert!(started.elapsed() < Duration::from_secs(5), "không đợi hết 60 giây");
    assert_eq!(status(dir.path(), "0001"), ChapterStatus::Translating, "phiên sau dịch tiếp chương này");
    assert_eq!(model.calls().len(), 1);
}

#[test]
fn loi_cau_hinh_400_thi_dung_ngay_khong_thu_lai_khong_skip_chuong_ve_queued() {
    let dir = story(2);
    let model = FakeModel::new(vec![Err(ApiError::Http {
        provider: "OpenAI",
        status: 400,
        message: "Model 'gpt-x' không khả dụng".into(),
    })]);
    let (sink, events) = collect();
    let handle = start_api_session(config(dir.path()), model.clone(), sink).unwrap();
    assert_eq!(
        handle.join(),
        StopReason::ApiFailed { message: "OpenAI trả 400: Model 'gpt-x' không khả dụng".into() }
    );
    let state = load_state(&story_paths(dir.path())).unwrap();
    assert_eq!(state.chapters["0001"].status, ChapterStatus::Queued, "không skip, không kẹt translating");
    assert!(state.chapters["0001"].reason.is_none());
    assert_eq!(state.chapters["0002"].status, ChapterStatus::Queued);
    assert_eq!(model.calls().len(), 1, "lỗi cấu hình không thử lại");
    assert!(!work_file(&story_paths(dir.path()), "0001", WorkKind::Prompt).exists(), "dọn work/ khi trả về hàng đợi");
    assert!(logs(&events.lock().unwrap()).iter().any(|l| l.contains("lỗi cấu hình API")));
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
