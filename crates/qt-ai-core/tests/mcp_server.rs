use qt_ai_core::commands::init::run_init;
use qt_ai_core::mcp::serve;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

/// Đưa các request (mỗi dòng một JSON) vào server tới EOF, trả các dòng server ghi ra.
fn session(root: &Path, requests: &[Value]) -> Vec<Value> {
    let input: String = requests.iter().map(|r| format!("{r}\n")).collect();
    let mut out = Vec::new();
    serve(input.as_bytes(), &mut out, root).unwrap();
    String::from_utf8(out).unwrap().lines().map(|line| serde_json::from_str(line).expect(line)).collect()
}

fn one(root: &Path, request: Value) -> Value {
    let mut replies = session(root, &[request]);
    assert_eq!(replies.len(), 1, "{replies:?}");
    replies.remove(0)
}

fn call(name: &str, arguments: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": 7, "method": "tools/call", "params": {"name": name, "arguments": arguments}})
}

fn text(reply: &Value) -> &str {
    reply["result"]["content"][0]["text"].as_str().unwrap_or_else(|| panic!("{reply}"))
}

fn is_error(reply: &Value) -> bool {
    reply["result"]["isError"].as_bool().unwrap_or_else(|| panic!("{reply}"))
}

fn story() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("raw")).unwrap();
    fs::write(dir.path().join("raw").join("0001.txt"), "赵静文抬头看向远方的高塔。\n\n她沉默了很久没有说话。").unwrap();
    run_init(dir.path(), "qt-ai").unwrap();
    dir
}

#[test]
fn initialize_tra_phien_ban_client_xin_neu_biet_khong_thi_ban_moi_nhat() {
    let dir = story();
    let init = |version: &str| {
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize",
               "params": {"protocolVersion": version, "capabilities": {}, "clientInfo": {"name": "antigravity-client"}}})
    };
    let replies = session(dir.path(), &[init("2025-06-18"), json!({"jsonrpc": "2.0", "method": "notifications/initialized"})]);
    assert_eq!(replies.len(), 1, "notification không được trả lời: {replies:?}");
    let result = &replies[0]["result"];
    assert_eq!(replies[0]["id"], json!(1));
    assert_eq!(result["protocolVersion"], json!("2025-06-18"));
    assert!(result["capabilities"]["tools"].is_object());
    assert_eq!(result["serverInfo"]["name"], json!("qt-ai"));

    assert_eq!(one(dir.path(), init("1999-01-01"))["result"]["protocolVersion"], json!("2025-11-25"));
}

#[test]
fn tools_list_du_tool_cua_vong_dich_kem_tham_so_bat_buoc() {
    let dir = story();
    let reply = one(dir.path(), json!({"jsonrpc": "2.0", "id": "a", "method": "tools/list"}));
    assert_eq!(reply["id"], json!("a"));
    let tools = reply["result"]["tools"].as_array().unwrap();
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["qt_status", "qt_next", "qt_check", "qt_accept", "qt_skip", "qt_retry"]);
    let required = |name: &str| tools.iter().find(|t| t["name"] == name).unwrap()["inputSchema"]["required"].clone();
    assert_eq!(required("qt_check"), json!(["id"]));
    assert_eq!(required("qt_accept"), json!(["id"]));
    assert_eq!(required("qt_skip"), json!(["id", "reason"]));
    for tool in tools {
        assert_eq!(tool["inputSchema"]["type"], json!("object"));
        assert!(!tool["description"].as_str().unwrap().is_empty());
    }
}

#[test]
fn vong_dich_next_check_accept_status_qua_tool_khong_can_terminal() {
    let dir = story();
    let root = dir.path();

    let next = one(root, call("qt_next", json!({})));
    assert!(!is_error(&next));
    assert!(text(&next).contains("Chương 0001 → đọc prompt tại:"), "{next}");
    let prompt = fs::read_to_string(root.join("work").join("0001.prompt.md")).unwrap();
    assert!(prompt.contains("qt_check") && !prompt.contains("Chạy: qt-ai check"), "prompt dặn gọi tool, không dặn lệnh terminal");

    fs::write(root.join("work").join("0001.draft.md"), "[[1]] Triệu Tĩnh Văn ngẩng đầu nhìn 高塔.").unwrap();
    let fail = one(root, call("qt_check", json!({"id": "0001"})));
    assert!(is_error(&fail));
    assert!(text(&fail).contains("FAIL") && text(&fail).contains("0001.review.md"), "{fail}");

    fs::write(
        root.join("work").join("0001.draft.md"),
        "[[1]] Triệu Tĩnh Văn ngẩng đầu nhìn về phía tòa tháp cao nơi xa.\n\n[[2]] Nàng im lặng hồi lâu không nói lời nào.",
    )
    .unwrap();
    let pass = one(root, call("qt_check", json!({"id": "0001"})));
    assert!(!is_error(&pass) && text(&pass).contains("PASS"), "{pass}");

    let accept = one(root, call("qt_accept", json!({"id": "0001"})));
    assert!(!is_error(&accept) && text(&accept).contains("Đã chốt"), "{accept}");
    assert!(root.join("out").join("0001.txt").exists());

    assert!(text(&one(root, call("qt_status", json!({})))).contains("done: 1"));

    let empty = one(root, call("qt_next", json!({})));
    assert!(is_error(&empty) && text(&empty).contains("Không còn chương"), "{empty}");
}

#[test]
fn skip_ghi_ly_do_retry_dua_ve_hang_doi() {
    let dir = story();
    let root = dir.path();
    one(root, call("qt_next", json!({})));
    let skip = one(root, call("qt_skip", json!({"id": "0001", "reason": "model từ chối: bộ lọc chặn 3 lượt"})));
    assert!(!is_error(&skip) && text(&skip).contains("Đã skip chương 0001"), "{skip}");
    assert!(fs::read_to_string(root.join("state.json")).unwrap().contains("bộ lọc chặn 3 lượt"));

    let retry = one(root, call("qt_retry", json!({"id": "0001"})));
    assert!(!is_error(&retry) && text(&retry).contains("về hàng đợi"), "{retry}");
}

#[test]
fn tool_la_la_loi_giao_thuc_thieu_tham_so_la_loi_tool_de_model_tu_sua() {
    let dir = story();
    let unknown = one(dir.path(), call("qt_xoa_het", json!({})));
    assert_eq!(unknown["error"]["code"], json!(-32602), "{unknown}");

    let missing = one(dir.path(), call("qt_check", json!({})));
    assert!(is_error(&missing) && text(&missing).contains("id"), "{missing}");
    let missing = one(dir.path(), call("qt_skip", json!({"id": "0001"})));
    assert!(is_error(&missing) && text(&missing).contains("reason"), "{missing}");
}

#[test]
fn method_la_dong_hong_va_ping() {
    let dir = story();
    let replies = session(
        dir.path(),
        &[
            json!({"jsonrpc": "2.0", "id": 1, "method": "server/discover"}),
            json!({"jsonrpc": "2.0", "method": "notifications/cancelled", "params": {}}),
            json!({"jsonrpc": "2.0", "id": 2, "method": "ping"}),
        ],
    );
    assert_eq!(replies.len(), 2, "{replies:?}");
    assert_eq!((replies[0]["id"].clone(), replies[0]["error"]["code"].clone()), (json!(1), json!(-32601)));
    assert_eq!((replies[1]["id"].clone(), replies[1]["result"].clone()), (json!(2), json!({})));

    let mut out = Vec::new();
    serve("{ hỏng\n\n".as_bytes(), &mut out, dir.path()).unwrap();
    let reply: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!((reply["id"].clone(), reply["error"]["code"].clone()), (Value::Null, json!(-32700)));
}

#[test]
fn next_qua_mcp_ghi_ro_tham_so_id_nguyen_van_khi_id_la_ten_file_dai() {
    // Truyện thật đặt tên file raw kiểu "Chương 0039 - 第39章 …": model từng gọi {"id": "0039"} theo ví dụ "0001" → lỗi.
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("raw")).unwrap();
    fs::write(root.join("raw").join("Chương 0039 - 第39章 黑道.txt"), "第39章\n\n他笑了。").unwrap();
    run_init(root, "qt-ai").unwrap();

    let next = one(root, call("qt_next", json!({})));
    assert!(text(&next).contains(r#"{"id":"Chương 0039 - 第39章 黑道"}"#), "{next}");
    let tools = one(root, json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list"}));
    for tool in tools["result"]["tools"].as_array().unwrap() {
        if let Some(id) = tool["inputSchema"]["properties"].get("id") {
            let description = id["description"].as_str().unwrap();
            assert!(description.contains("nguyên văn") && !description.contains("0001"), "{description}");
        }
    }
}

/// Truyện id dài, số thứ tự file lệch số trong tên chương; chương đầu đang dịch dở.
fn story_dich_do() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("raw")).unwrap();
    fs::write(root.join("raw").join("0108 第107章 悲凉.txt"), "第107章\n\n他笑了。").unwrap();
    fs::write(root.join("raw").join("0109 第108章 怒火.txt"), "第108章\n\n她哭了。").unwrap();
    run_init(root, "qt-ai").unwrap();
    assert!(!is_error(&one(root, call("qt_next", json!({})))));
    dir
}

const ID_DICH_DO: &str = r#"{"id":"0108 第107章 悲凉"}"#;

#[test]
fn next_khi_con_chuong_dich_do_dua_san_tham_so_id_nguyen_van() {
    // Phiên mới vào lại chương dở chỉ thấy câu báo lỗi, model tự rút id thành "0108" rồi kẹt cả phiên.
    let dir = story_dich_do();
    let next = one(dir.path(), call("qt_next", json!({})));
    assert!(is_error(&next), "{next}");
    assert!(text(&next).contains("đang translating chưa chốt"), "{next}");
    assert!(text(&next).contains(&format!("Tham số cho qt_check / qt_accept: {ID_DICH_DO}")), "{next}");
}

#[test]
fn goi_sai_id_bao_lai_id_chuong_dich_do_va_khong_doan_chuong_nao() {
    let dir = story_dich_do();
    let root = dir.path();
    let before = fs::read_to_string(root.join("state.json")).unwrap();
    // "0108" là số file của chương dở, "108" lại là số trong tên chương KHÁC: cả hai đều không được đoán.
    for wrong in ["0108", "108", "第107章"] {
        for (tool, args) in [
            ("qt_check", json!({"id": wrong})),
            ("qt_accept", json!({"id": wrong})),
            ("qt_skip", json!({"id": wrong, "reason": "x"})),
            ("qt_retry", json!({"id": wrong})),
        ] {
            let reply = one(root, call(tool, args));
            assert!(is_error(&reply), "{tool} {reply}");
            assert!(text(&reply).contains(&format!("Không có chương \"{wrong}\"")), "{tool} {reply}");
            assert!(text(&reply).contains(ID_DICH_DO), "{tool} {reply}");
        }
    }
    assert_eq!(fs::read_to_string(root.join("state.json")).unwrap(), before, "gọi sai id không được đổi tiến độ");
}

#[test]
fn nhieu_chuong_dich_do_thi_liet_ke_du_id() {
    let dir = story_dich_do();
    let root = dir.path();
    let mut state: Value = serde_json::from_str(&fs::read_to_string(root.join("state.json")).unwrap()).unwrap();
    state["chapters"]["0109 第108章 怒火"]["status"] = json!("translating");
    fs::write(root.join("state.json"), state.to_string()).unwrap();

    let reply = one(root, call("qt_check", json!({"id": "0108"})));
    assert!(text(&reply).contains(ID_DICH_DO) && text(&reply).contains(r#"{"id":"0109 第108章 怒火"}"#), "{reply}");
}

#[test]
fn khong_co_chuong_dich_do_thi_giu_thong_bao_cu() {
    let dir = story();
    let reply = one(dir.path(), call("qt_check", json!({"id": "0108"})));
    assert!(is_error(&reply), "{reply}");
    assert_eq!(text(&reply), "Không có chương 0108 trong state.json.");
}

#[test]
fn next_qua_terminal_van_dan_lenh_check_nhu_cu() {
    let dir = story();
    qt_ai_core::commands::next::run_next(dir.path()).unwrap();
    let prompt = fs::read_to_string(dir.path().join("work").join("0001.prompt.md")).unwrap();
    assert!(prompt.contains("Chạy: qt-ai check 0001") && !prompt.contains("qt_check"));
}
