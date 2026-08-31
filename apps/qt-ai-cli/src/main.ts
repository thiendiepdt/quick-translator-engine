import { runAccept } from "./commands/accept.ts";
import { runCheck } from "./commands/check.ts";
import { runInit } from "./commands/init.ts";
import { runNext } from "./commands/next.ts";
import { runSkip } from "./commands/skip.ts";
import { runStatus } from "./commands/status.ts";

const USAGE = `qt-ai <lệnh> <thư-mục-truyện> [chương] [cờ]

Lệnh:
  init <root>                        Dựng khung folder truyện + copy AGENTS.md/workflows
  next <root>                        Phát chương kế tiếp, lắp prompt vào work/
  check <root> <id>                  Kiểm tra bản dịch work/<id>.draft.md
  accept <root> <id> [--force]       Chốt chương: ghi out/, merge glossary
  skip <root> <id> --reason <lý do>  Bỏ qua chương (model từ chối...)
  status <root>                      Bảng tiến độ`;

export function main(argv: string[]): number {
  const [command, root, ...rest] = argv;
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
          console.log(`Chương ${id} PASS (ratio ${result.ratio.toFixed(2)}) — chạy accept.`);
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
        return 0;
      }
      case "skip": {
        const id = rest[0];
        const reasonIndex = rest.indexOf("--reason");
        const reason = reasonIndex >= 0 ? rest.slice(reasonIndex + 1).join(" ") : "";
        if (!root || !id) break;
        runSkip(root, id, reason);
        console.log(`Đã skip chương ${id}.`);
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

if (import.meta.url === `file://${process.argv[1]}`) {
  process.exit(main(process.argv.slice(2)));
}
