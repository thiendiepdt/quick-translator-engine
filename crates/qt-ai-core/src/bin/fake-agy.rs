//! agy giả cho test runner. Không ship. Kịch bản đọc từ ./fake-agy-mode.txt (cwd = folder truyện):
//!   progress → đánh dấu chương queued đầu tiên thành done (ghi out/<id>.txt), exit 0
//!   noop     → không đổi gì, exit 0
//!   fail     → in stderr, exit 3
//!   hang     → ngủ 20s rồi exit 0 (để test cancel)
//!   quota    → lần gọi đầu: ghi một hội thoại chết vì hết quota (RESOURCE_EXHAUSTED … Resets in <T>) vào thư mục
//!              dữ liệu agy ghi trong ./fake-agy-data-dir.txt rồi exit 1; các lần sau chạy như progress.
//!              <T> lấy từ ./fake-agy-quota-resets.txt, mặc định 1s.
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
    let flag = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned().unwrap_or_default();
    let model = flag("--model");
    let print_timeout = flag("--print-timeout");
    println!("fake: model={model} print_timeout={print_timeout} args={}", args.len());
    let cwd = std::env::current_dir().unwrap();
    let mode = std::fs::read_to_string(cwd.join("fake-agy-mode.txt")).unwrap_or_default();
    match mode.trim() {
        "progress" => mark_one_done(&cwd),
        "fail" => {
            eprintln!("fake: quota exceeded");
            std::process::exit(3);
        }
        "hang" => std::thread::sleep(std::time::Duration::from_secs(20)),
        "quota" => {
            let hit = cwd.join("fake-agy-quota-hit.txt");
            if hit.exists() {
                mark_one_done(&cwd);
            } else {
                std::fs::write(&hit, "1").unwrap();
                write_quota_conversation(&cwd, &flag("-p"));
                eprintln!("fake: Individual quota reached");
                std::process::exit(1);
            }
        }
        _ => {}
    }
}

/// Hội thoại như agy thật để lại khi hết quota: lượt đầu là prompt của app, workspace là folder truyện, lượt
/// sau mang nguyên văn lỗi của máy chủ.
fn write_quota_conversation(root: &Path, prompt: &str) {
    let data_dir = std::fs::read_to_string(root.join("fake-agy-data-dir.txt")).unwrap();
    let conversations = Path::new(data_dir.trim()).join("conversations");
    std::fs::create_dir_all(&conversations).unwrap();
    let resets = std::fs::read_to_string(root.join("fake-agy-quota-resets.txt")).unwrap_or_else(|_| "1s".to_string());
    let uri = format!("file:///{}", root.to_string_lossy().replace('\\', "/").trim_start_matches('/'));
    let mut workspace = vec![0x0a];
    if uri.len() < 128 {
        workspace.push(uri.len() as u8);
    } else {
        workspace.extend([(uri.len() & 0x7f) as u8 | 0x80, (uri.len() >> 7) as u8]);
    }
    workspace.extend_from_slice(uri.as_bytes());
    workspace.push(0x12);
    let error = format!(
        "RESOURCE_EXHAUSTED (code 429): Individual quota reached. Please upgrade your subscription to increase your limits. Resets in {}.",
        resets.trim()
    );
    let conn = rusqlite::Connection::open(conversations.join("99999999-9999-4999-8999-999999999999.db")).unwrap();
    conn.execute_batch(
        "pragma journal_mode = wal;
         create table steps (idx integer primary key, step_payload blob);
         create table trajectory_metadata_blob (id text primary key, data blob);",
    )
    .unwrap();
    conn.execute("insert into steps (idx, step_payload) values (0, ?1)", [prompt.as_bytes().to_vec()]).unwrap();
    conn.execute("insert into steps (idx, step_payload) values (1, ?1)", [error.into_bytes()]).unwrap();
    conn.execute("insert into trajectory_metadata_blob (id, data) values ('main', ?1)", [workspace]).unwrap();
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
