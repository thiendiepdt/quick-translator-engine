//! Helper test của qt-ext: máy chủ giả ở `qt_ext::testkit`, ở đây là gói tạm và tuỳ chọn lượt chạy.
#![allow(dead_code)]

#[allow(unused_imports)]
pub use qt_ext::testkit::{demo_server, gbk, Resp, Route, Step, TestServer};

pub const TEST_MANIFEST: &str = r#"{ "id": "io.github.dev.test", "name": "Test", "version": "1.0.0", "apiVersion": 1,
    "description": "Extension cho test", "developer": { "name": "dev" }, "match": ["^http"],
    "hosts": ["127.0.0.1"], "limits": { "concurrency": 3 } }"#;

/// Gói tạm từ danh sách file (manifest mặc định nếu không có), nạp ở chế độ developer.
pub fn package(files: &[(&str, &str)]) -> qt_ext::Package {
    let dir = tempfile::tempdir().unwrap();
    if !files.iter().any(|(name, _)| *name == "manifest.json") {
        std::fs::write(dir.path().join("manifest.json"), TEST_MANIFEST).unwrap();
    }
    for (name, text) in files {
        let path = dir.path().join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    qt_ext::Package::load_dir(dir.path(), true).unwrap()
}

pub fn demo_package() -> qt_ext::Package {
    qt_ext::Package::load_dir(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/demo"), true).unwrap()
}

pub fn session_opts() -> qt_ext::SessionOptions {
    qt_ext::SessionOptions { allow_local: true, ..Default::default() }
}
