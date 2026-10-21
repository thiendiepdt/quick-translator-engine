//! App chết thì agy chết theo (child_guard): test chạy lại chính exe test làm "app giả" — app giả spawn một tiến
//! trình chạy lâu qua child_guard, in PID rồi thoát ngay (như đóng app khi đang dịch). Tiến trình đó phải chết theo.

use qt_ai_core::child_guard::{bind, prepare};
use qt_ai_core::process::quiet_command;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const ROLE: &str = "QT_AI_CHILD_GUARD_FAKE_APP";

fn long_running() -> Command {
    if cfg!(windows) {
        let mut command = quiet_command("cmd");
        command.args(["/C", "ping -n 60 127.0.0.1 >NUL"]);
        command
    } else {
        let mut command = quiet_command("sleep");
        command.arg("60");
        command
    }
}

fn alive(pid: u32) -> bool {
    if cfg!(windows) {
        let out = quiet_command("tasklist").args(["/FI", &format!("PID eq {pid}"), "/NH", "/FO", "CSV"]).output().unwrap();
        String::from_utf8_lossy(&out.stdout).contains(&format!("\"{pid}\""))
    } else {
        // `kill -0` thấy cả zombie; đọc trạng thái trong /proc để không tính zombie là còn sống.
        // Trạng thái là trường đầu tiên sau `(tên)` — tên có thể chứa dấu cách nên cắt sau dấu `)` cuối.
        std::fs::read_to_string(format!("/proc/{pid}/stat"))
            .map(|stat| stat.rsplit_once(')').and_then(|(_, rest)| rest.split_whitespace().next()).is_none_or(|state| state != "Z"))
            .unwrap_or(false)
    }
}

/// Vai "app giả": chỉ chạy khi được test chính gọi lại với biến môi trường.
#[test]
fn fake_app_spawn_roi_thoat() {
    if std::env::var(ROLE).is_err() {
        return;
    }
    let mut command = long_running();
    command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    prepare(&mut command);
    let child = command.spawn().unwrap();
    bind(&child);
    println!("CHILD_PID={}", child.id());
    // Thoát cứng như app bị tắt: không dọn dẹp, không giết con.
    std::process::exit(0);
}

#[test]
fn app_chet_thi_tien_trinh_con_chet_theo() {
    use std::io::{BufRead, BufReader};
    let exe = std::env::current_exe().unwrap();
    // Không dùng output(): nó chờ pipe đóng, mà tiến trình con mồ côi thừa hưởng pipe sẽ giữ nó tới khi tự chạy xong.
    let mut app = Command::new(exe)
        .args(["fake_app_spawn_roi_thoat", "--exact", "--nocapture", "--test-threads=1"])
        .env(ROLE, "1")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut seen = String::new();
    let mut pid = None;
    for line in BufReader::new(app.stdout.take().unwrap()).lines() {
        let line = line.unwrap();
        if let Some(value) = line.split("CHILD_PID=").nth(1) {
            pid = Some(value.trim().parse::<u32>().unwrap());
            break;
        }
        seen.push_str(&line);
    }
    let pid = pid.unwrap_or_else(|| panic!("app giả không in PID: {seen}"));
    app.wait().unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while alive(pid) && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(100));
    }
    let still = alive(pid);
    if still {
        let _ = quiet_command(if cfg!(windows) { "taskkill" } else { "kill" })
            .args(if cfg!(windows) { vec!["/PID".to_string(), pid.to_string(), "/T".into(), "/F".into()] } else { vec!["-9".into(), pid.to_string()] })
            .output();
    }
    assert!(!still, "tiến trình con PID {pid} còn sống sau khi app giả chết");
}
