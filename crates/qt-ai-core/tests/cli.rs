use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn qt_ai(args: &[&str], cwd: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_qt-ai")).args(args).current_dir(cwd).output().expect("chạy được binary qt-ai")
}
fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}
fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

#[test]
fn khong_co_lenh_in_usage_exit_2() {
    let dir = tempfile::tempdir().unwrap();
    let out = qt_ai(&[], dir.path());
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("qt-ai <lệnh> <thư-mục-truyện>"));
    assert!(stderr(&out).contains("export <root>"));
    let out = qt_ai(&["status"], dir.path());
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn init_next_check_accept_status_export_qua_binary() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("raw")).unwrap();
    fs::write(root.join("raw").join("0001.txt"), "赵静文抬头看向远方的高塔。\n\n她沉默了很久没有说话。").unwrap();

    let out = qt_ai(&["init", "."], root); // root tương đối theo cwd
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(stdout(&out).contains("1 chương (1 mới thêm vào hàng đợi)"));
    let agents = fs::read_to_string(root.join("AGENTS.md")).unwrap();
    assert!(agents.contains("qt-ai") && !agents.contains("{{QT_AI}}"));

    let root_str = root.to_str().unwrap();
    let out = qt_ai(&["next", root_str], root);
    assert_eq!(out.status.code(), Some(0));
    assert!(stdout(&out).contains("Chương 0001 → đọc prompt tại:"));

    fs::write(root.join("work").join("0001.draft.md"), "[[1]] Triệu Tĩnh Văn ngẩng đầu nhìn 高塔.").unwrap();
    let out = qt_ai(&["check", root_str, "0001"], root);
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("FAIL") && stderr(&out).contains("0001.review.md"));

    fs::write(
        root.join("work").join("0001.draft.md"),
        "[[1]] Triệu Tĩnh Văn ngẩng đầu nhìn về phía tòa tháp cao nơi xa.\n\n[[2]] Nàng im lặng hồi lâu không nói lời nào.",
    )
    .unwrap();
    let out = qt_ai(&["check", root_str, "0001"], root);
    assert_eq!(out.status.code(), Some(0));
    assert!(stdout(&out).contains("PASS"));

    let out = qt_ai(&["accept", root_str, "0001"], root);
    assert_eq!(out.status.code(), Some(0));
    assert!(stdout(&out).contains("Đã chốt") && root.join("out").join("0001.txt").exists());

    let out = qt_ai(&["status", root_str], root);
    assert!(stdout(&out).contains("done: 1"));

    let out = qt_ai(&["export", root_str], root);
    assert_eq!(out.status.code(), Some(0));
    assert!(stdout(&out).contains("Đã gộp 1 chương"));
    assert!(root.join("export").join("0001-0001.txt").exists());

    let out = qt_ai(&["next", root_str], root);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("Không còn chương"));
}

#[test]
fn skip_thieu_id_in_usage_thay_vi_hieu_nham_reason() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("raw")).unwrap();
    fs::write(root.join("raw").join("0001.txt"), "第一章").unwrap();
    let root_str = root.to_str().unwrap();
    qt_ai(&["init", root_str], root);
    qt_ai(&["next", root_str], root);
    let out = qt_ai(&["skip", root_str, "--reason", "x"], root);
    assert_eq!(out.status.code(), Some(2));
    assert!(fs::read_to_string(root.join("state.json")).unwrap().contains("\"translating\""));
    let out = qt_ai(&["skip", root_str, "0001", "--reason", "model", "từ chối"], root);
    assert_eq!(out.status.code(), Some(0));
    assert!(fs::read_to_string(root.join("state.json")).unwrap().contains("model từ chối"));
    let out = qt_ai(&["retry", root_str, "0001"], root);
    assert_eq!(out.status.code(), Some(0));
    assert!(stdout(&out).contains("về hàng đợi"));
}
