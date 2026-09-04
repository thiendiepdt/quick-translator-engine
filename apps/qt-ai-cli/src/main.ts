import { isAbsolute, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { runAccept } from "./commands/accept.ts";
import { runCheck } from "./commands/check.ts";
import { runExport } from "./commands/export.ts";
import { runInit } from "./commands/init.ts";
import { runNext } from "./commands/next.ts";
import { runRetry } from "./commands/retry.ts";
import { runSkip } from "./commands/skip.ts";
import { runStatus } from "./commands/status.ts";

const USAGE = `qt-ai <lệnh> <thư-mục-truyện> [chương] [cờ]

Lệnh:
  init <root>                        Dựng khung folder truyện + copy AGENTS.md/workflows
  next <root>                        Phát chương kế tiếp, lắp prompt vào work/
  check <root> <id>                  Kiểm tra bản dịch work/<id>.draft.md
  accept <root> <id> [--force]       Chốt chương: ghi out/, merge glossary
  skip <root> <id> --reason <lý do>  Bỏ qua chương (model từ chối...)
  retry <root> <id>                  Đưa chương error/skipped về hàng đợi dịch lại
  export <root> [--from <id>] [--to <id>] [--out <file>]
                                     Gộp các chương done thành một file txt
  status <root>                      Bảng tiến độ`;

/**
 * `npm --prefix apps/qt-ai-cli run -s qt-ai -- <lệnh> <root>` chạy với
 * cwd = thư mục package CLI (không phải cwd của người gõ lệnh), nên một
 * `root` tương đối (vd. `.`, `../my-story`) sẽ resolve nhầm vào trong repo.
 * npm luôn set INIT_CWD = cwd gốc trước khi đổi cwd — dùng nó làm base khi
 * root không tuyệt đối; fallback về `cwd` (process.cwd()) khi không có
 * INIT_CWD (vd. gọi trực tiếp `tsx src/main.ts` hoặc trong test).
 */
export function resolveRootArg(root: string, initCwd: string | undefined, cwd: string): string {
  if (isAbsolute(root)) return root;
  return resolve(initCwd ?? cwd, root);
}

export function main(argv: string[]): number {
  const [command, rootArg, ...rest] = argv;
  const root = rootArg === undefined
    ? undefined
    : resolveRootArg(rootArg, process.env.INIT_CWD, process.cwd());
  try {
    switch (command) {
      case "init": {
        if (!root) break;
        console.log(runInit(root));
        return 0;
      }
      case "next": {
        if (!root) break;
        const result = runNext(root);
        console.log(`Chương ${result.chapterId} → đọc prompt tại: ${result.promptPath}`);
        return 0;
      }
      case "check": {
        const id = rest[0];
        if (!root || !id) break;
        const result = runCheck(root, id);
        if (result.pass) {
          console.log(
            result.acceptedWithWarnings
              ? `Chương ${id} hết vòng review, còn ${result.violations.length} vi phạm rule → chốt kèm cảnh báo. Chạy accept.`
              : `Chương ${id} PASS (ratio ${result.ratio.toFixed(2)}) — chạy accept.`,
          );
          return 0;
        }
        if (result.escalatedToError) {
          console.error(`Chương ${id} quá số vòng review → error. Xem qt-ai status.`);
          return 2;
        }
        console.error(
          `Chương ${id} FAIL: thiếu ${result.missing.length} đoạn, ${result.violations.length} vi phạm, ratio ${result.ratio.toFixed(2)}.\nSửa theo: ${result.reviewPath}`,
        );
        return 1;
      }
      case "accept": {
        const id = rest[0];
        if (!root || !id) break;
        const result = runAccept(root, id, { force: rest.includes("--force") });
        console.log(`Đã chốt ${result.outPath} (+${result.addedGlossary} glossary mới).`);
        if (result.warnings.length > 0) {
          console.log(`Kèm ${result.warnings.length} cảnh báo — xem qt-ai status.`);
        }
        return 0;
      }
      case "skip": {
        const id = rest[0];
        // Không có id, hoặc "--reason" bị hiểu nhầm thành id khi id thực sự bị
        // thiếu (`qt-ai skip <root> --reason x`) → rơi xuống usage thay vì lỗi
        // khó hiểu từ runSkip.
        if (!root || !id || id.startsWith("--")) break;
        const reasonIndex = rest.indexOf("--reason");
        const reason = reasonIndex >= 0 ? rest.slice(reasonIndex + 1).join(" ") : "";
        runSkip(root, id, reason);
        console.log(`Đã skip chương ${id}.`);
        return 0;
      }
      case "retry": {
        const id = rest[0];
        if (!root || !id) break;
        runRetry(root, id);
        console.log(`Đã đưa chương ${id} về hàng đợi — next sẽ phát lại nó.`);
        return 0;
      }
      case "export": {
        if (!root) break;
        const flag = (name: string): string | undefined => {
          const index = rest.indexOf(name);
          return index >= 0 ? rest[index + 1] : undefined;
        };
        const result = runExport(root, { from: flag("--from"), to: flag("--to"), out: flag("--out") });
        console.log(`Đã gộp ${result.ids.length} chương → ${result.outPath}`);
        if (result.gaps.length > 0) {
          console.log(`Hổng ${result.gaps.length} chương chưa done trong khoảng: ${result.gaps.join(", ")}`);
        }
        return 0;
      }
      case "status": {
        if (!root) break;
        console.log(runStatus(root));
        return 0;
      }
    }
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    return 2;
  }
  console.error(USAGE);
  return 2;
}

// So sánh qua pathToFileURL vì trên Windows argv[1] là `D:\...\main.ts`
// còn import.meta.url là `file:///D:/...` — nối chuỗi thô không bao giờ khớp.
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  process.exit(main(process.argv.slice(2)));
}
