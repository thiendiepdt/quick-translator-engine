const USAGE = `qt-ai <lệnh> <thư-mục-truyện> [chương] [cờ]

Lệnh:
  init <root>                Dựng khung folder truyện + copy AGENTS.md/workflows
  next <root>                Phát chương kế tiếp, lắp prompt vào work/
  check <root> <id>          Kiểm tra bản dịch work/<id>.draft.md
  accept <root> <id> [--force]  Chốt chương: ghi out/, merge glossary
  skip <root> <id> --reason <lý do>  Bỏ qua chương (model từ chối...)
  status <root>              Bảng tiến độ`;

export function main(_argv: string[]): number {
  console.log(USAGE);
  return 1;
}

if (import.meta.url === `file://${process.argv[1]}`) {
  process.exit(main(process.argv.slice(2)));
}
