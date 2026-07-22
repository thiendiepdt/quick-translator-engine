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
        "他=tha\n很=ngận\n好=hảo\n",
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
            "--scan-range",
            "30",
            "--translation-algorithm",
            "1",
            "--prioritized-name",
            "true",
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

    let mut child = Command::new(bin)
        .args([
            "translate",
            "--mode",
            "vietphrase-one",
            "--data-dir",
            dir.to_str().unwrap(),
            "--scan-range",
            "5",
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
    assert_eq!(String::from_utf8_lossy(&out.stdout), "丁格尔斯泰特");

    let custom_names = dir.join("custom-names.txt");
    std::fs::write(&custom_names, "丁格尔斯泰特=Custom Name/Alternative\n").unwrap();
    let mut child = Command::new(bin)
        .args([
            "translate",
            "--mode",
            "vietphrase-one",
            "--data-dir",
            dir.to_str().unwrap(),
            "--names-file",
            custom_names.to_str().unwrap(),
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
    assert_eq!(String::from_utf8_lossy(&out.stdout), " Custom Name");

    let custom_luat_nhan = dir.join("custom-luat-nhan.txt");
    let custom_pronouns = dir.join("custom-pronouns.txt");
    std::fs::write(&custom_luat_nhan, "{n}很好={n} rất tốt\n").unwrap();
    std::fs::write(&custom_pronouns, "他=hắn\n").unwrap();
    let mut child = Command::new(bin)
        .args([
            "translate",
            "--mode",
            "vietphrase-one",
            "--data-dir",
            dir.to_str().unwrap(),
            "--luat-nhan-file",
            custom_luat_nhan.to_str().unwrap(),
            "--pronouns-file",
            custom_pronouns.to_str().unwrap(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all("他很好".as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();

    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), " hắn rất tốt");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn cli_rejects_invalid_engine_options() {
    let output = Command::new(env!("CARGO_BIN_EXE_qt"))
        .args([
            "translate",
            "--translation-algorithm",
            "3",
            "--prioritized-name",
            "false",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "error: --translation-algorithm must be 0, 1, or 2\n"
    );
}
