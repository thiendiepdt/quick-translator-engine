use std::io::Write;
use std::process::{Command, Stdio};

// Runs the built `qt` binary with a tiny data dir written to a temp folder.
#[test]
fn cli_translates_hanviet_and_long_vietphrase_over_stdin() {
    // Arrange a minimal data dir
    let dir = std::env::temp_dir().join(format!("qtcli-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("Resources")).unwrap();
    std::fs::create_dir_all(dir.join("VietPhrase")).unwrap();
    std::fs::create_dir_all(dir.join("Names2")).unwrap();
    std::fs::write(
        dir.join("Resources/ChinesePhienAmWords.txt"),
        "他=tha\n好=hảo\n",
    )
    .unwrap();
    std::fs::write(dir.join("Names.txt"), "丁格尔斯泰特=Dingelstedt\n").unwrap();
    std::fs::write(dir.join("Names2/123.txt"), "").unwrap();
    std::fs::write(dir.join("VietPhrase/VietPhrase.txt"), "").unwrap();

    let bin = env!("CARGO_BIN_EXE_qt");
    let mut child = Command::new(bin)
        .args([
            "translate",
            "--mode",
            "hanviet",
            "--data-dir",
            dir.to_str().unwrap(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all("他好".as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();

    assert!(out.status.success());
    // Faithful engine output: leading space, lowercase first word.
    assert_eq!(String::from_utf8_lossy(&out.stdout), " tha hảo");

    let mut child = Command::new(bin)
        .args([
            "translate",
            "--mode",
            "vietphrase-one",
            "--data-dir",
            dir.to_str().unwrap(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all("丁格尔斯泰特".as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();

    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), " Dingelstedt");

    let _ = std::fs::remove_dir_all(&dir);
}
