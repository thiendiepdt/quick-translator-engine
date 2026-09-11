#!/usr/bin/env bash
# Vòng lặp dịch tự động qua Antigravity CLI (agy): mỗi phiên agy -p là một
# context sạch, thay cho việc tự tay mở conversation mới trong IDE.
# Trong lúc agy chạy, script poll state.json để in tiến độ từng chương.
# Bản Linux/Ubuntu của auto-translate.ps1.
# Dùng: ./auto-translate.sh --root /path/ten-truyen [--max-sessions 50] [--poll-seconds 5]
set -euo pipefail

usage() {
  echo "Dùng: $0 --root <thư mục truyện> [--max-sessions N] [--poll-seconds N]" >&2
  exit 2
}

ROOT=""
MAX_SESSIONS=50
POLL_SECONDS=5
while [[ $# -gt 0 ]]; do
  case "$1" in
    --root|-r) ROOT="${2:-}"; shift 2 ;;
    --max-sessions|-m) MAX_SESSIONS="${2:-}"; shift 2 ;;
    --poll-seconds|-p) POLL_SECONDS="${2:-}"; shift 2 ;;
    -h|--help) usage ;;
    *) echo "Tham số lạ: $1" >&2; usage ;;
  esac
done
[[ -n "$ROOT" ]] || usage
[[ -d "$ROOT" ]] || { echo "Không tìm thấy thư mục truyện: $ROOT" >&2; exit 1; }

CLI_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ROOT="$(cd "$ROOT" && pwd)"
STATE_JSON="$ROOT/state.json"
[[ -f "$STATE_JSON" ]] || { echo "Không có state.json trong $ROOT" >&2; exit 1; }

# Đọc thẳng state.json (ghi atomic nên không bao giờ đọc trúng file dở);
# nhanh hơn nhiều so với gọi qt-ai status qua npm mỗi lần poll.
# In ra: done queued translating settled current
get_counts() {
  node -e '
    const s = JSON.parse(require("fs").readFileSync(process.argv[1], "utf8"));
    const c = { queued: 0, translating: 0, done: 0, error: 0, skipped: 0 };
    let current = "";
    for (const [id, ch] of Object.entries(s.chapters ?? {})) {
      c[ch.status] = (c[ch.status] ?? 0) + 1;
      if (ch.status === "translating") current = id;
    }
    // settled: chương đã xử lý xong, không quay lại
    console.log([c.done, c.queued, c.translating, c.done + c.error + c.skipped, current].join(" "));
  ' "$STATE_JSON"
}

format_counts() {
  local done queued translating settled current
  read -r done queued translating settled current <<<"$1"
  local line="done $done, queued $queued"
  [[ -n "$current" ]] && line+=", đang dịch chương $current"
  echo "$line"
}

# Đường dẫn tuyệt đối để agent mở thẳng file, không đi search filesystem
WORKFLOW_PATH="$ROOT/.agent/workflows/translate.md"
PROMPT="Mở file $WORKFLOW_PATH (đường dẫn tuyệt đối, tồn tại sẵn, KHÔNG cần tìm kiếm) rồi làm đúng theo nó: dịch tới khi chạm giới hạn chương/phiên hoặc hết hàng đợi thì dừng. Thư mục truyện: $ROOT"

AGY_PID=""
cleanup() {
  # Ctrl+C: giết agy đang chạy rồi vẫn in status
  if [[ -n "$AGY_PID" ]] && kill -0 "$AGY_PID" 2>/dev/null; then
    kill "$AGY_PID" 2>/dev/null || true
    wait "$AGY_PID" 2>/dev/null || true
  fi
  npm --prefix "$CLI_DIR" run -s qt-ai -- status "$ROOT" || true
}
trap cleanup EXIT

for ((i = 1; i <= MAX_SESSIONS; i++)); do
  before="$(get_counts)"
  read -r b_done b_queued b_translating b_settled _ <<<"$before"
  if [[ "$b_queued" -eq 0 && "$b_translating" -eq 0 ]]; then
    echo "Hết hàng đợi — done $b_done."
    break
  fi
  echo "=== Phiên $i — $(format_counts "$before") ==="

  # agy chạy nền (cwd = folder truyện để tự nhặt AGENTS.md), script poll tiến độ
  (cd "$ROOT" && exec agy -p "$PROMPT" --dangerously-skip-permissions) &
  AGY_PID=$!
  last="$(format_counts "$before")"
  while kill -0 "$AGY_PID" 2>/dev/null; do
    sleep "$POLL_SECONDS"
    now="$(format_counts "$(get_counts 2>/dev/null || echo "$before")")" || continue
    if [[ "$now" != "$last" ]]; then
      echo "[$(date +%H:%M:%S)] $now"
      last="$now"
    fi
  done
  wait "$AGY_PID" || true
  AGY_PID=""

  after="$(get_counts)"
  read -r _ _ _ a_settled _ <<<"$after"
  if [[ "$a_settled" -le "$b_settled" ]]; then
    echo "CẢNH BÁO: Phiên $i không chốt thêm chương nào (kẹt/refuse/hết quota?) — dừng để khỏi đốt quota. Xem work/ và chạy status." >&2
    break
  fi
done
