# qt-ai-gui: dịch nhiều truyện song song

Ngày 2026-09-09. Bổ sung cho `2026-09-09-qt-ai-gui-library-design.md`.

## Mục tiêu

Một cửa sổ app chạy phiên dịch cho nhiều truyện cùng lúc; mỗi truyện tối đa một phiên; tổng số phiên bị chặn
bởi cấu hình để không tự bắn 429 vào API hub hay ngốn quota agy.

## 1. Rust: bảng phiên theo truyện

- `AppState.sessions: Mutex<SessionRegistry>` (module `session_registry.rs`, thuần Rust, test bằng handle giả qua
  trait `SessionLike`). Khoá theo `root_key` (bỏ hoa thường, dấu `/` `\` cuối) — khớp `pathKey` bên TS.
- `AppConfig.max_parallel: u32` (mặc định 2; UI giới hạn 1–5).
- `session_start(root, model)`: `check_can_start` → `session_locked` nếu truyện đó đang chạy, `session_limit`
  ("Đang chạy N/M truyện…") nếu đã đủ. Phiên kết thúc tự bị dọn khỏi bảng mỗi lần tra.
- `session_stop(root)`: gỡ khỏi bảng rồi cancel + join ngoài lock. `session_state()` → `{ running: [root…] }`.
- Event `session-event` phát `{ root, ...SessionEvent }` (`RootedEvent` với `serde(flatten)`).
- `open_story`, `story_snapshot`, `create_story`, `rescan_story`, `import_chapters`, `ai_fill_story` kiểm
  "đang chạy" theo root của mình.

## 2. Store: tách theo truyện

- `sessions`, `progress`, `logs`, `roots` là map theo `pathKey(root)`; `applySessionEvent(root, event)`;
  `clearLogs(root)`. Selector hiện tại: `selectCurrentSession/Running/Progress/Logs` đọc theo `root` đang mở,
  trả tham chiếu ổn định khi rỗng. `runningRoots(state)` là hàm thuần (dùng trong `useMemo`, không làm selector).
- `openStory`: `snapshot.sessionRunning` đồng bộ trạng thái truyện đó (Rust bảo chạy mà store chưa biết →
  running/sessionNo 0; Rust bảo không chạy mà store còn running → idle); truyện khác giữ nguyên.
- `closeStory` không đụng phiên. Nút "Về danh sách truyện" không còn khoá.

## 3. Giao diện

- Hook `useSessionEvents` parse `root`, chỉ nạp lại snapshot khi event thuộc truyện đang mở; toast kèm tên folder.
- Picker: tiêu đề có "Đang dịch N/M truyện"; dòng đang dịch có nhãn "Đang dịch · <chương>", tiến độ live đè số
  đọc từ đĩa, nút Dừng gọi `session_stop(root)` rồi đánh dấu `stopped/user_cancelled` ngay trong store.
- Toolbar, reader, hồ sơ, cài đặt dùng selector hiện tại nên chỉ tác động lên truyện đang mở. Cài đặt → App có
  "Số truyện dịch song song".

## Kiểm thử

Rust `session_registry` (chạy tới max, từ chối cùng root, tự dọn, take đúng root); store (event hai truyện không
lẫn, ring buffer log riêng, selector theo root, đóng truyện giữ phiên); hook (event truyện khác không nạp
snapshot, thiếu root bị bỏ); picker (dòng đang dịch + Dừng đúng root); translate-page (Quét lại khoá theo truyện).
