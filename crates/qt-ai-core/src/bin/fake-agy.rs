//! agy giả cho test runner. Không ship. Kịch bản đọc từ ./fake-agy-mode.txt (cwd = folder truyện):
//!   progress → đánh dấu chương queued đầu tiên thành done (ghi out/<id>.txt), exit 0
//!   noop     → không đổi gì, exit 0
//!   fail     → in stderr, exit 3
//!   hang     → ngủ 20s rồi exit 0 (để test cancel)
//! `--version` in "fake-agy 0.0.1"; `models` in hai model.
use qt_ai_core::story::natural_chapter_compare;
use qt_ai_core::story_fs::{load_state, now_ms, save_state, story_paths, ChapterState, ChapterStatus};
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--version") {
        println!("fake-agy 0.0.1");
        return;
    }
    if args.first().map(String::as_str) == Some("models") {
        println!("fake-model\nfake-model-pro\n");
        return;
    }
    let model = args.iter().position(|a| a == "--model").and_then(|i| args.get(i + 1)).cloned().unwrap_or_default();
    println!("fake: model={model} args={}", args.len());
    let cwd = std::env::current_dir().unwrap();
    let mode = std::fs::read_to_string(cwd.join("fake-agy-mode.txt")).unwrap_or_default();
    match mode.trim() {
        "progress" => mark_one_done(&cwd),
        "fail" => {
            eprintln!("fake: quota exceeded");
            std::process::exit(3);
        }
        "hang" => std::thread::sleep(std::time::Duration::from_secs(20)),
        _ => {}
    }
}

fn mark_one_done(root: &Path) {
    let paths = story_paths(root);
    let mut state = load_state(&paths).unwrap();
    let mut queued: Vec<String> = state
        .chapters
        .iter()
        .filter(|(_, c)| c.status == ChapterStatus::Queued)
        .map(|(id, _)| id.clone())
        .collect();
    queued.sort_by(|a, b| natural_chapter_compare(a, b));
    if let Some(id) = queued.first() {
        std::fs::create_dir_all(&paths.out_dir).unwrap();
        std::fs::write(paths.out_dir.join(format!("{id}.txt")), "Bản dịch giả.\n").unwrap();
        state.chapters.insert(
            id.clone(),
            ChapterState { status: ChapterStatus::Done, review_round: 0, reason: None, warnings: None, updated_at: now_ms() },
        );
        save_state(&paths, &state).unwrap();
        println!("fake: done {id}");
    }
}
