# qt-ai-gui — Design

**Ngày:** 2026-09-01
**Mục tiêu:** App desktop cho người dùng bình thường dịch truyện hàng loạt bằng quota Antigravity: mở folder truyện, điền hồ sơ, bấm Dịch, xem tiến độ, xử lý chương lỗi, export. Bản chất vẫn là gọi Antigravity CLI (`agy`) theo vòng phiên như `apps/qt-ai-cli/scripts/auto-translate.ps1`, nhưng phần "não" (state machine, prompt, check, glossary) được **port sang Rust** trong crate `qt-ai-core` — không cần Node trên máy người dùng.

Spec kế thừa `2026-08-31-antigravity-translation-harness-design.md` (format folder truyện, vòng next → dịch → check → accept, luật AGENTS.md).

## Quyết định nền

| Quyết định | Chọn | Lý do / phương án đã bỏ |
|---|---|---|
| Đặt ở đâu | App riêng `apps/qt-ai-gui` | Gắn với agy + quota Antigravity; không ép người dùng có `qt-core` + data QT2025 như `qt-gui`. Bỏ: workspace mới trong qt-gui. |
| Logic dịch chạy kiểu gì | Port sang Rust (`crates/qt-ai-core`) | Người dùng chọn: không phụ thuộc Node, không sidecar. Rủi ro drift với qt-web xử bằng golden fixtures (dưới). Bỏ: sidecar exe từ TS (Bun/Node SEA); chạy logic trong webview qua Tauri fs. |
| Bản TS `qt-ai-cli` | Giữ song song | Cùng format folder truyện → dùng chéo. Về sau có thể thêm `src/bin/qt-ai.rs` từ core để thay, ngoài phạm vi spec này. |
| Nền tảng | Windows trước | Như qt-gui; bundle NSIS + MSI. |

## Kiến trúc

```
apps/qt-ai-gui/            Tauri 2 shell + React UI (vỏ, không chứa logic dịch)
  src/                     React 19, Tailwind 4, shadcn (copy components/ui từ qt-gui), zustand, RHF + zod
  src-tauri/               Tauri commands mỏng: gọi qt-ai-core, đẩy event lên UI
crates/qt-ai-core/         Rust lib thuần, không biết Tauri
  story      story.json (AiStoryConfig) serde + normalize + natural_chapter_compare   <- ai-story.ts
  paragraphs tách đoạn, payload [[n]], repair payload, parse, strip                  <- ai-paragraphs.ts
  prompt     base prompt nguyên văn, merge/filter glossary, lắp system prompt         <- ai-translation(-prompt).ts
  check      rule mặc định + checkRules + rule bắt buộc CJK (fancy-regex)            <- checkAiTranslationViolations
  glossary   sanitize, append, resolve autoGlossary                                  <- ai-glossary.ts
  story_fs   state.json/story.json/raw/out/work, atomic write, .bak, validate        <- story-fs.ts
  commands   init/next/check/accept/skip/retry/export/status                         <- commands/*.ts
  session    runner phiên agy: spawn, stream log, watch state.json, cầu dao, cancel  <- auto-translate.ps1
  templates  AGENTS.md + workflows, include_str! từ apps/qt-ai-cli/antigravity (một nguồn)
agy (ngoài repo)           Antigravity CLI, người dùng tự cài + đăng nhập Google
```

Cả `crates/qt-ai-core` và `apps/qt-ai-gui/src-tauri` thêm vào `members` của Cargo workspace gốc (cạnh `qt-core`, `qt-gui/src-tauri`). Dependency mới: `fancy-regex`, `serde_json` với feature `preserve_order`.

Nguyên tắc:

- **Folder truyện là hợp đồng chung.** `state.json` version 1 (kể cả `settings`, `warnings`), `story.json` đúng schema web, `raw/`, `out/<id>.txt`, `work/`, `AGENTS.md`, `.agent/workflows/`. GUI, bản TS CLI và Antigravity IDE dùng chéo trên cùng folder. GUI vẫn ghi `AGENTS.md` + workflows để `agy` nhặt luật y như hiện tại.
- **`qt-ai-core` không biết Tauri**, test bằng `cargo test`; Tauri chỉ là consumer.
- **UI không tự đọc file.** Mọi thay đổi đi qua core (một chỗ giữ atomic write + validate); UI nhận event.
- **Luồng dữ liệu:** UI → `invoke(cmd)` → core đổi file → core phát event → zustand store → render.

## Session runner (module `session`)

Chạy trên thread riêng, giao tiếp qua channel → Tauri event.

Vòng lặp một lần Bắt đầu:

```
loop (tối đa max_sessions, mặc định 50):
  counts = đọc state.json
  queued == 0 && translating == 0            -> Finished
  spawn agy -p <prompt> --dangerously-skip-permissions [--model M], cwd = folder truyện
  trong lúc chạy: stdout/stderr theo dòng -> AgyLog; poll state.json mỗi 2s -> Progress khi đổi
  agy thoát:
    settled (done+error+skipped) không tăng  -> Stopped(NoProgress)
    exit code != 0 hai lần liên tiếp         -> Stopped(AgyFailed)
```

Prompt cho agy giữ nội dung hiện tại của auto-translate.ps1: đường dẫn tuyệt đối tới `.agent/workflows/translate.md`, ghi rõ "tồn tại sẵn, KHÔNG cần tìm kiếm", kèm đường dẫn folder truyện.

- **Dừng:** cancel → kill cả process tree của agy (Windows: `taskkill /T /F` theo PID hoặc job object) → Stopped(UserCancelled). Chương đang dở vẫn `translating`; `next` lần sau phát lại (thiết kế sẵn có).
- **Phát hiện agy:** khởi động chạy `agy --version`; thiếu → màn hướng dẫn cài (`irm https://antigravity.google/cli/install.ps1 | iex`) + "Kiểm tra lại"; có → `agy models` đổ dropdown. Cài đặt cho trỏ đường dẫn agy tay.
- **AI điền hồ sơ:** runner chế độ một lượt: spawn `agy -p` với prompt setup-story (tên + link từ form), chờ xong, đọc lại `story.json`, hiện diff, người dùng Áp dụng/Bỏ — không tự ghi đè. Trước khi chạy, core sao lưu `story.json` để "Bỏ" khôi phục được.
- **Lock:** `work/.session.lock` chứa PID + thời điểm; app thứ hai mở cùng folder → chỉ xem, nút Bắt đầu vô hiệu kèm lý do. Lock mồ côi (PID không còn sống) thì tự dọn.
- **Một runner / một cửa sổ / một truyện.** Nhiều truyện song song = nhiều cửa sổ.

Event lên UI: `Progress{done, queued, translating, error, skipped, current, warnings_count}`, `AgyLog{line, stream}`, `SessionState{Idle | Running{session_no} | Stopped{reason}}`.

## Giao diện (React)

Ba màn hình:

1. **Chọn truyện:** "Mở folder truyện" (Tauri dialog) + danh sách mở gần đây (app config). Chưa có `state.json` → hỏi "Khởi tạo?" → `init`. Thiếu `raw/` → báo rõ.
2. **Bàn dịch** (chính, 2 cột):
   - Trái: bảng chương — id, badge trạng thái, số cảnh báo, reviewRound; lọc theo trạng thái; click chọn.
   - Phải, tab: **Chương** (bản dịch `out/` hoặc draft đang dở, danh sách cảnh báo, nút Retry / Skip kèm lý do / Accept --force / Mở folder) · **Log agy** (stream, auto-scroll, xoá) · **Hồ sơ truyện**.
   - Thanh trên: `done/tổng` + progress bar, **Bắt đầu / Dừng**, dropdown model, **Export** (dialog khoảng + đích, báo chương hổng).
3. **Hồ sơ truyện:** RHF + zod theo schema `story.json`: tên, link nguồn, nhân vật chính, tóm tắt, customPrompt, style (voice / toneRules mỗi dòng một / avoid), glossary 7 nhóm bảng CN→VN thêm/xoá dòng (port `ai-story-config-dialog.tsx`), checkRules (regex/flags/message), autoGlossary inherit/on/off. **AI điền** → diff → Áp dụng/Bỏ. Import/Export JSON (dán config từ qt-web).
4. **Cài đặt** (dialog): đường dẫn agy (auto/tay), model mặc định, số phiên tối đa → app config; chương/phiên, số vòng review, ratio tối thiểu → `state.json.settings` của truyện đang mở.

State: một store zustand `useStoryStore` (truyện đang mở, chapters map, session state, log ring buffer 2000 dòng, settings). Chỉ cập nhật từ event Tauri và kết quả `invoke`. Form RHF local, submit mới `invoke("save_story")`. Không dùng react-query.

## Port Rust — điểm cần chú ý

- `AiStoryConfig` serde `camelCase`, alias đọc `tone_rules`/`signature_phrases`; normalize bỏ field lạ như web; ghi ra giữ key order ổn định.
- Prompt phải ra **đúng byte** với TS: base prompt copy nguyên văn; `serde_json` pretty 2 space thay `JSON.stringify(_, null, 2)` — key order theo insertion (feature `preserve_order`); filter glossary kể cả rule bỏ họ tên 3–4 chữ Hán và giữ nguyên `signature_phrases`.
- Check rules: web dùng lookbehind `(?<!\p{L})` → `fancy-regex`; regex `checkRules` hỏng thì bỏ qua rule đó như web.
- Messages tiếng Việt của commands giữ nguyên câu chữ bản TS (agent đọc quen, workflow so khớp chuỗi "chốt kèm cảnh báo", "quá số vòng review → error").

## Xử lý lỗi

- Core trả `Result<_, CoreError>`; enum: `StoryNotFound`, `InvalidState`, `InvalidStory`, `AgyMissing`, `AgyFailed{code, stderr_tail}`, `SessionLocked{pid}`, `Io`.
- Tauri serialize `{kind, message}`; UI toast message, dùng `kind` để quyết định hành động (AgyMissing → mở hướng dẫn cài).
- Thread runner bọc `catch_unwind`; panic → Stopped(Internal), app không sập.

## Test

- **Golden parity với qt-web:** `apps/qt-ai-cli/scripts/gen-golden.ts` chạy code TS thật trên 5 story config mẫu (rỗng / đủ glossary 7 nhóm / customPrompt / checkRules riêng / glossary tên 4 chữ Hán) và vài bản dịch mẫu → `crates/qt-ai-core/tests/fixtures/*.json` gồm: prompt đầy đủ, kết quả parse nhãn, danh sách vi phạm, glossary sau sanitize. Rust test so từng byte. Fixtures commit vào repo; CI job gen lại + `git diff --exit-code` bắt drift.
- **State machine:** cargo test trên tempdir: init → next → check → accept trọn vòng, resume khi mất `work/*.prompt.md`, retry/skip/export, hết vòng review chỉ còn vi phạm → done kèm warnings, thiếu đoạn → error. Tương đương bộ vitest hiện có.
- **Runner:** `fake-agy` (script ghi vào state.json rồi thoát theo kịch bản) để test cầu dao NoProgress/AgyFailed, cancel kill process, lock.
- **Frontend:** vitest + testing-library cho store reducer từ event và zod schema form, như qt-gui.

## Build / phân phối

`npm run tauri build` → NSIS + MSI như qt-gui. Không bundle Node hay sidecar. Máy người dùng cần WebView2 (bootstrapper tự tải) và `agy` tự cài + đăng nhập.

## Lộ trình

1. `crates/qt-ai-core`: story_fs + story + paragraphs + prompt + golden parity (bằng chứng port đúng trước khi làm gì khác).
2. check + glossary + commands, đủ bộ test state machine.
3. session runner + fake-agy.
4. `apps/qt-ai-gui` shell: chọn truyện, bàn dịch, Bắt đầu/Dừng, log, tiến độ.
5. Hồ sơ truyện + AI điền + Export + Cài đặt.
6. Chạy thử trên truyện thật đang dịch (`nam-nu-de`), so kết quả với bản TS.

## Ngoài phạm vi

Tách file lớn thành chương, tải truyện từ link nguồn, EPUB, macOS/Linux, nhiều phiên song song trên một truyện, thay bản TS CLI, workspace glossary của qt-web (từ điển convert), gỡ tên tự thêm qua UI (sửa tay `story.json`).
