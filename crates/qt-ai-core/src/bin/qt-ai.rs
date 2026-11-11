//! CLI cho agent (bên trong agy / Antigravity IDE) gọi từ AGENTS.md. Cùng lệnh, message, exit code
//! với apps/qt-ai-cli/src/main.ts.

use qt_ai_core::agent_reply::{self, Reply};
use qt_ai_core::commands::delete::run_delete;
use qt_ai_core::commands::export::{run_export, ExportOptions};
use qt_ai_core::commands::init::run_init;
use qt_ai_core::commands::next::Via;
use qt_ai_core::commands::retry::run_retry_range;
use qt_ai_core::commands::sync_progress::run_sync_progress;
use qt_ai_core::CoreError;
use std::path::{Path, PathBuf};

const USAGE: &str = "qt-ai <lệnh> <thư-mục-truyện> [chương] [cờ]

Lệnh:
  init <root>                        Dựng khung folder truyện + copy AGENTS.md/workflows
  next <root>                        Phát chương kế tiếp, lắp prompt vào work/
  check <root> <id>                  Kiểm tra bản dịch work/<id>.draft.md
  accept <root> <id> [--force]       Chốt chương: ghi out/, merge glossary
  skip <root> <id> --reason <lý do>  Bỏ qua chương (model từ chối...)
  retry <root> <id>                  Đưa chương về hàng đợi dịch lại (done: out/<id>.txt → .bak)
  retry <root> --all | --from <id> --to <id>   Dịch lại nhiều chương (bỏ qua chương đang queued)
  delete <root> <id> [<id>…]         Xoá chương khỏi truyện: gỡ state, xoá raw/ + work/ (out/ giữ nguyên)
  export <root> [--from <id>] [--to <id>] [--out <file>]
                                     Gộp các chương done thành một file txt
  sync-progress <root>               Đánh dấu đã dịch các chương có bản dịch trong out/ (state.json lệch sau sync Drive)
  vn-check <root>                    Quét nội dung nhạy cảm về Việt Nam
  status <root>                      Bảng tiến độ
  mcp <root>                         Server MCP (stdio) cho agy: tool qt_next/qt_check/qt_accept/… thay lệnh terminal";

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

/// In lời đáp như CLI xưa nay: mã 0 ra stdout, khác 0 ra stderr; trả mã thoát.
fn emit(reply: Reply) -> i32 {
    if reply.code == 0 {
        println!("{}", reply.text);
    } else {
        eprintln!("{}", reply.text);
    }
    reply.code
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
        "next" => Ok(emit(agent_reply::next(root, Via::Shell)?)),
        "check" => {
            let Some(id) = rest.first() else { return Ok(usage()) };
            Ok(emit(agent_reply::check(root, id)?))
        }
        "accept" => {
            let Some(id) = rest.first() else { return Ok(usage()) };
            Ok(emit(agent_reply::accept(root, id, rest.iter().any(|a| a == "--force"))?))
        }
        "skip" => {
            // Không có id, hoặc "--reason" bị hiểu nhầm thành id → usage thay vì lỗi khó hiểu.
            let Some(id) = rest.first().filter(|id| !id.starts_with("--")) else { return Ok(usage()) };
            let reason = rest
                .iter()
                .position(|a| a == "--reason")
                .map(|index| rest[index + 1..].join(" "))
                .unwrap_or_default();
            Ok(emit(agent_reply::skip(root, id, &reason)?))
        }
        "retry" => {
            let ranged = rest.iter().any(|arg| arg == "--all" || arg == "--from" || arg == "--to");
            if ranged {
                let outcome = run_retry_range(root, flag(rest, "--from").map(String::as_str), flag(rest, "--to").map(String::as_str))?;
                println!(
                    "Đã đưa {} chương về hàng đợi ({} bản dịch cũ giữ .bak, {} vốn đã queued).",
                    outcome.retried.len(),
                    outcome.backed_up.len(),
                    outcome.already_queued.len()
                );
                return Ok(0);
            }
            let Some(id) = rest.first() else { return Ok(usage()) };
            Ok(emit(agent_reply::retry(root, id)?))
        }
        "delete" => {
            if rest.is_empty() {
                return Ok(usage());
            }
            let outcome = run_delete(root, rest)?;
            println!(
                "Đã xoá {} chương ({} bản dịch trong out/ giữ nguyên).",
                outcome.removed.len(),
                outcome.kept_outputs.len()
            );
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
        "sync-progress" => {
            let outcome = run_sync_progress(root)?;
            println!(
                "Đã đánh dấu {} chương đã dịch theo out/; {} chương ghi đã dịch nhưng chưa thấy bản dịch (giữ nguyên — có thể Drive chưa sync xong).",
                outcome.marked_done.len(),
                outcome.missing.len()
            );
            Ok(0)
        }
        "vn-check" => {
            let paths = qt_ai_core::story_fs::story_paths(root);
            let lexicon = qt_ai_core::vn_lexicon::Lexicon::load(&qt_ai_core::base::BaseStore::from_env());
            let started = std::time::Instant::now();
            let summary = qt_ai_core::vn_check::refresh(&paths, &lexicon)?;
            println!(
                "Kiểm tra VN: {} chỗ ({} chưa xét/cần xử lý), quét {} chương, {} ms",
                summary.hits,
                summary.open,
                summary.scanned,
                started.elapsed().as_millis()
            );
            for hit in qt_ai_core::vn_check::load_file(&paths).hits.iter().filter(|h| h.status != qt_ai_core::vn_check::Status::Ok) {
                println!("  {} [{:?}] {} ×{}", hit.id, hit.side, hit.matched, hit.count);
            }
            Ok(0)
        }
        "status" => Ok(emit(agent_reply::status(root)?)),
        "mcp" => {
            let (stdin, stdout) = (std::io::stdin(), std::io::stdout());
            qt_ai_core::mcp::serve(stdin.lock(), stdout.lock(), root).map_err(|e| CoreError::Internal(e.to_string()))?;
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
