//! CLI cho agent (bên trong agy / Antigravity IDE) gọi từ AGENTS.md. Cùng lệnh, message, exit code
//! với apps/qt-ai-cli/src/main.ts.

use qt_ai_core::commands::accept::run_accept;
use qt_ai_core::commands::check::run_check;
use qt_ai_core::commands::export::{run_export, ExportOptions};
use qt_ai_core::commands::init::run_init;
use qt_ai_core::commands::next::run_next;
use qt_ai_core::commands::retry::run_retry;
use qt_ai_core::commands::skip::run_skip;
use qt_ai_core::commands::status::run_status;
use qt_ai_core::CoreError;
use std::path::{Path, PathBuf};

const USAGE: &str = "qt-ai <lệnh> <thư-mục-truyện> [chương] [cờ]

Lệnh:
  init <root>                        Dựng khung folder truyện + copy AGENTS.md/workflows
  next <root>                        Phát chương kế tiếp, lắp prompt vào work/
  check <root> <id>                  Kiểm tra bản dịch work/<id>.draft.md
  accept <root> <id> [--force]       Chốt chương: ghi out/, merge glossary
  skip <root> <id> --reason <lý do>  Bỏ qua chương (model từ chối...)
  retry <root> <id>                  Đưa chương error/skipped về hàng đợi dịch lại
  export <root> [--from <id>] [--to <id>] [--out <file>]
                                     Gộp các chương done thành một file txt
  status <root>                      Bảng tiến độ";

/// Lệnh chạy chính binary này, render vào AGENTS.md — bọc ngoặc kép vì đường dẫn Windows hay có khoảng trắng.
fn self_command() -> String {
    std::env::current_exe()
        .map(|exe| format!("\"{}\"", exe.display()))
        .unwrap_or_else(|_| "qt-ai".to_string())
}

fn flag<'a>(rest: &'a [String], name: &str) -> Option<&'a String> {
    rest.iter().position(|arg| arg == name).and_then(|index| rest.get(index + 1))
}

fn usage() -> i32 {
    eprintln!("{USAGE}");
    2
}

fn run(argv: &[String]) -> Result<i32, CoreError> {
    let Some(command) = argv.first() else { return Ok(usage()) };
    let Some(root) = argv.get(1).map(Path::new) else { return Ok(usage()) };
    let rest = &argv[2..];
    match command.as_str() {
        "init" => {
            println!("{}", run_init(root, &self_command())?);
            Ok(0)
        }
        "next" => {
            let result = run_next(root)?;
            println!("Chương {} → đọc prompt tại: {}", result.chapter_id, result.prompt_path.display());
            Ok(0)
        }
        "check" => {
            let Some(id) = rest.first() else { return Ok(usage()) };
            let result = run_check(root, id)?;
            if result.pass {
                if result.accepted_with_warnings {
                    println!(
                        "Chương {id} hết vòng review, còn {} vi phạm rule → chốt kèm cảnh báo. Chạy accept.",
                        result.violations.len()
                    );
                } else {
                    println!("Chương {id} PASS (ratio {:.2}) — chạy accept.", result.ratio);
                }
                return Ok(0);
            }
            if result.escalated_to_error {
                eprintln!("Chương {id} quá số vòng review → error. Xem qt-ai status.");
                return Ok(2);
            }
            eprintln!(
                "Chương {id} FAIL: thiếu {} đoạn, {} vi phạm, ratio {:.2}.\nSửa theo: {}",
                result.missing.len(),
                result.violations.len(),
                result.ratio,
                result.review_path.map(|p| p.display().to_string()).unwrap_or_default()
            );
            Ok(1)
        }
        "accept" => {
            let Some(id) = rest.first() else { return Ok(usage()) };
            let result = run_accept(root, id, rest.iter().any(|a| a == "--force"))?;
            println!("Đã chốt {} (+{} glossary mới).", result.out_path.display(), result.added_glossary);
            if !result.warnings.is_empty() {
                println!("Kèm {} cảnh báo — xem qt-ai status.", result.warnings.len());
            }
            Ok(0)
        }
        "skip" => {
            // Không có id, hoặc "--reason" bị hiểu nhầm thành id → usage thay vì lỗi khó hiểu.
            let Some(id) = rest.first().filter(|id| !id.starts_with("--")) else { return Ok(usage()) };
            let reason = rest
                .iter()
                .position(|a| a == "--reason")
                .map(|index| rest[index + 1..].join(" "))
                .unwrap_or_default();
            run_skip(root, id, &reason)?;
            println!("Đã skip chương {id}.");
            Ok(0)
        }
        "retry" => {
            let Some(id) = rest.first() else { return Ok(usage()) };
            run_retry(root, id)?;
            println!("Đã đưa chương {id} về hàng đợi — next sẽ phát lại nó.");
            Ok(0)
        }
        "export" => {
            let options = ExportOptions {
                from: flag(rest, "--from").cloned(),
                to: flag(rest, "--to").cloned(),
                out: flag(rest, "--out").map(PathBuf::from),
            };
            let result = run_export(root, &options)?;
            println!("Đã gộp {} chương → {}", result.ids.len(), result.out_path.display());
            if !result.gaps.is_empty() {
                println!("Hổng {} chương chưa done trong khoảng: {}", result.gaps.len(), result.gaps.join(", "));
            }
            Ok(0)
        }
        "status" => {
            println!("{}", run_status(root)?);
            Ok(0)
        }
        _ => Ok(usage()),
    }
}

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let code = match run(&argv) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{error}");
            2
        }
    };
    std::process::exit(code);
}
