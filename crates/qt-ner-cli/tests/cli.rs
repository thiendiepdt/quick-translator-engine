use std::io::Write;
use std::process::{Command, Stdio};

fn create_data_dir(name: &str) -> std::path::PathBuf {
    let directory = std::env::temp_dir().join(format!("qt-ner-cli-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(directory.join("Resources")).unwrap();
    std::fs::create_dir_all(directory.join("VietPhrase")).unwrap();
    std::fs::write(
        directory.join("Resources/ChinesePhienAmWords.txt"),
        "来=lai\n人=nhân\n名=danh\n为=vi\n萧=tiêu\n炎=viêm\n走=tẩu\n",
    )
    .unwrap();
    std::fs::write(directory.join("Resources/HoNguoi.txt"), "萧=Tiêu\n").unwrap();
    std::fs::write(directory.join("VietPhrase/VietPhrase.txt"), "").unwrap();
    directory
}

#[test]
fn filters_stdin_as_names2_without_optional_providers() {
    let directory = create_data_dir("names2");
    let mut child = Command::new(env!("CARGO_BIN_EXE_qt-ner-cli"))
        .args([
            "filter",
            "--data-dir",
            directory.to_str().unwrap(),
            "--no-ner",
            "--min-occurrences",
            "1",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all("来人名为萧炎。萧炎走来。".as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("萧炎=Tiêu Viêm"));
    let _ = std::fs::remove_dir_all(directory);
}

#[test]
fn emits_the_shared_api_response_as_json() {
    let directory = create_data_dir("json");
    let mut child = Command::new(env!("CARGO_BIN_EXE_qt-ner-cli"))
        .args([
            "filter",
            "--data-dir",
            directory.to_str().unwrap(),
            "--no-ner",
            "--min-occurrences",
            "1",
            "--json",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all("来人名为萧炎。萧炎走来。".as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(output.status.success());
    let body: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(body["capabilities"]["nerConfigured"], false);
    assert!(body["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .any(|candidate| candidate["text"] == "萧炎"));
    let _ = std::fs::remove_dir_all(directory);
}
