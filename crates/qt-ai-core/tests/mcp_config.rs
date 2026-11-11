use qt_ai_core::mcp_config::{config_path, has_server, install, prepare, uninstall, SERVER_NAME};
use serde_json::json;
use std::fs;
use std::path::Path;

fn read(root: &Path) -> serde_json::Value {
    serde_json::from_str(&fs::read_to_string(config_path(root)).unwrap()).unwrap()
}

fn write(root: &Path, text: &str) {
    fs::create_dir_all(root.join(".agents")).unwrap();
    fs::write(config_path(root), text).unwrap();
}

#[test]
fn install_tao_file_tro_lenh_mcp_vao_dung_thu_muc_truyen_lan_hai_khong_ghi_lai() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let exe = Path::new(r"C:\app\qt-ai.exe");
    assert!(!has_server(root));

    assert!(install(root, exe).unwrap());
    let server = &read(root)["mcpServers"][SERVER_NAME];
    assert_eq!(server["command"], json!(exe.display().to_string()));
    assert_eq!(server["args"], json!(["mcp", root.display().to_string()]));
    assert!(has_server(root));

    assert!(!install(root, exe).unwrap(), "đã đúng thì không ghi lại (đỡ Drive sync vô ích)");
}

#[test]
fn install_giu_server_va_khoa_khac_cua_nguoi_dung_doi_lenh_khi_app_doi_cho() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(root, r#"{"mcpServers": {"khac": {"command": "node", "args": ["x.js"]}}, "riêng": 1}"#);

    install(root, Path::new(r"C:\cu\qt-ai.exe")).unwrap();
    assert!(install(root, Path::new(r"D:\moi\qt-ai.exe")).unwrap());
    let config = read(root);
    assert_eq!(config["mcpServers"][SERVER_NAME]["command"], json!(r"D:\moi\qt-ai.exe"));
    assert_eq!(config["mcpServers"]["khac"], json!({"command": "node", "args": ["x.js"]}));
    assert_eq!(config["riêng"], json!(1));
    let keys: Vec<&String> = config["mcpServers"].as_object().unwrap().keys().collect();
    assert_eq!(keys, ["khac", SERVER_NAME], "server có sẵn giữ chỗ, qt-ai thêm vào cuối");
}

#[test]
fn file_hong_thi_khong_ghi_de_va_coi_nhu_chua_co_server() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    for broken in ["{ không phải json", r#"{"mcpServers": []}"#, "[1, 2]"] {
        write(root, broken);
        assert!(install(root, Path::new("qt-ai.exe")).is_err(), "{broken}");
        assert!(!has_server(root), "{broken}");
        assert!(!uninstall(root).unwrap(), "{broken}");
        assert_eq!(fs::read_to_string(config_path(root)).unwrap(), broken, "không đụng file của người dùng");
    }
}

#[test]
fn uninstall_file_chi_con_qt_ai_thi_xoa_ca_file_lan_thu_muc_agents() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    install(root, Path::new("qt-ai.exe")).unwrap();

    assert!(uninstall(root).unwrap());
    assert!(!config_path(root).exists());
    assert!(!root.join(".agents").exists());
    assert!(!has_server(root));
    assert!(!uninstall(root).unwrap(), "chưa có file thì không làm gì");
}

#[test]
fn uninstall_con_server_khac_thi_giu_file_chi_bo_muc_qt_ai() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(root, r#"{"mcpServers": {"khac": {"command": "node"}}}"#);
    install(root, Path::new("qt-ai.exe")).unwrap();

    assert!(uninstall(root).unwrap());
    assert_eq!(read(root), json!({"mcpServers": {"khac": {"command": "node"}}}));
}

#[test]
fn uninstall_giu_thu_muc_agents_khi_trong_do_con_thu_khac() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    install(root, Path::new("qt-ai.exe")).unwrap();
    fs::create_dir_all(root.join(".agents").join("plugins")).unwrap();

    assert!(uninstall(root).unwrap());
    assert!(!config_path(root).exists());
    assert!(root.join(".agents").join("plugins").is_dir());
}

fn initialized_story() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("raw")).unwrap();
    fs::write(dir.path().join("raw").join("0001.txt"), "第一章").unwrap();
    qt_ai_core::commands::init::run_init(dir.path(), "qt-ai").unwrap();
    dir
}

fn agents_md(root: &Path) -> String {
    fs::read_to_string(root.join("AGENTS.md")).unwrap()
}

#[test]
fn chuan_bi_phien_co_binary_thi_ghi_server_va_mau_goi_tool_khong_co_thi_go_ve_mau_lenh() {
    let dir = initialized_story();
    let root = dir.path();
    let exe = root.join("app").join("qt-ai.exe");

    assert!(prepare(root, Some(&exe), "qt-ai").unwrap());
    assert!(has_server(root));
    assert!(agents_md(root).contains("qt_next"));

    assert!(!prepare(root, None, "qt-ai").unwrap(), "tắt MCP (hoặc chưa build binary qt-ai)");
    assert!(!root.join(".agents").exists());
    assert!(!agents_md(root).contains("qt_next"));
}

#[test]
fn chuan_bi_phien_file_mcp_hong_thi_chay_lenh_terminal_khong_dung_file() {
    let dir = initialized_story();
    let root = dir.path();
    write(root, "{ hỏng");

    assert!(!prepare(root, Some(Path::new("qt-ai.exe")), "qt-ai").unwrap());
    assert_eq!(fs::read_to_string(config_path(root)).unwrap(), "{ hỏng");
    assert!(!agents_md(root).contains("qt_next"));
}
