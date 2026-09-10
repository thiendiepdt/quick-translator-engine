# qt-ai: base prompt / rule / glossary người dùng sửa được

Ngày 2026-09-11. Bổ sung cho `2026-09-04-multi-genre-translation-design.md` (9 base prompt theo genre, rule theo
bối cảnh).

## Mục tiêu

Người dùng sửa được bản mặc định của app — base prompt từng genre, bộ rule từng bối cảnh, và một kho glossary
chung từng bối cảnh — mà không phải chép ra từng truyện. Truyện đang dùng mặc định tự ăn theo; truyện có prompt
riêng / rule riêng vẫn thắng như nay. Bản sửa phải được thấy bởi cả app, phiên API lẫn `qt-ai next` chạy trong
phiên agy.

## 1. Lưu trữ: file rời trong thư mục app

Thư mục base `<app_config_dir>/base/` (Windows `%APPDATA%\com.vn-converter.qt-ai-gui\base\`, Linux
`~/.config/com.vn-converter.qt-ai-gui/base/`, macOS `~/Library/Application Support/com.vn-converter.qt-ai-gui/base/`):

```
base/
  prompts/<setting>-<names>.md   ancient-han.md … 9 tổ hợp, markdown nguyên văn thay cho base cứng
  rules/<setting>.json           ancient.json / modern.json / mixed.json — mảng {pattern, flags?, message}
  glossary/<setting>.json        cùng cấu trúc glossary truyện: {names:{}, places:{}, …, addressing:{}}
```

- Không có file = dùng bản cứng trong binary (prompts.json / DEFAULT_RULES / rỗng). "Về mặc định" = xoá file.
- File rules là **bộ đã lọc theo bối cảnh** (như `default_rules_as_check_rules(setting)` trả ra), người dùng sửa
  danh sách đó. Rule bắt buộc "còn Hán tự" không nằm trong file, luôn chạy.
- File glossary: nhóm thiếu coi như rỗng; entry key/value trống bị bỏ khi ghép (như glossary truyện).
- Không nhét vào `config.json` (9 prompt ≈ 300 KB sẽ được ghi lại mỗi lần đổi recent) và không gộp một file
  `base.json` (sửa tay / chia sẻ từng phần khó hơn).

## 2. Core Rust đọc base — module `qt_ai_core::base`

- `pub fn base_dir() -> Option<PathBuf>`: ưu tiên env `QT_AI_BASE_DIR`; không có thì tự tính
  `<config_dir>/com.vn-converter.qt-ai-gui/base` theo hệ: Windows `%APPDATA%`, macOS `$HOME/Library/Application
  Support`, còn lại `$XDG_CONFIG_HOME` hoặc `$HOME/.config`. Identifier là hằng `APP_IDENTIFIER` trong core (GUI
  test đối chiếu với `tauri.conf.json`).
- `pub fn base_prompt_for(genre) -> String`: file `prompts/<key>.md` (đọc được, không rỗng sau trim) → nội dung;
  không → `prompt::base_prompt(genre)`. Đọc mỗi lần gọi, không cache (sửa xong dịch ngay).
- `pub fn base_rules_for(setting) -> Vec<CheckRule>`: file `rules/<setting>.json` parse được → bộ đó; không →
  `default_rules_as_check_rules(setting)`. File hỏng → bỏ qua như không có (ghi log stderr một dòng).
- `pub fn base_glossary_for(setting) -> Glossary`: file `glossary/<setting>.json` → glossary; không → rỗng.
- `pub fn base_source(kind, key) -> BaseSource { Builtin | File(PathBuf) }` để UI hiện nhãn.
- Ghi/xoá: `save_base_prompt(genre, text)`, `save_base_rules(setting, rules)`, `save_base_glossary(setting,
  glossary)`, `reset_base_*` (xoá file). Ghi atomic (`write_atomic`), tạo thư mục khi thiếu. Text trống → xoá file.

Điểm nối vào chỗ ghép:

- `prompt::build_system_prompt`: `base` = custom_prompt trống → `base_prompt_for(genre)`; `workspace` glossary
  do caller truyền = `base_glossary_for(genre.setting)` (hai caller: `commands/next.rs`, `api_session.rs`). Thứ tự
  ghép glossary giữ nguyên: base làm nền, truyện đè khi trùng key, rồi lọc theo chương.
- `check::check_violations(text, configured, setting)`: `configured` rỗng → `base_rules_for(setting)` thay vì
  `rules_for(setting)`. `MANDATORY_RULES` vẫn nối sau.
- `qt-ai next/check` (CLI) không đổi chữ ký — tự thấy base qua `base_dir()`.

## 3. GUI

- Tauri command mới (`base_cmds.rs`): `base_get(kind, key)` → `{ text | rules | glossary, source: "builtin" |
  "file" }`, `base_save(kind, key, payload)`, `base_reset(kind, key)`. `story_defaults(genre)` trả
  `base_prompt_for` / `base_rules_for` kèm `promptSource`, `rulesSource` để Hồ sơ truyện ghi nhãn.
- Lúc khởi động, app đặt env `QT_AI_BASE_DIR` = `<app_config_dir>/base` (`AppState.base_dir`); tiến trình agy
  con thừa hưởng env, phiên API và `story_defaults` trong app dùng `AppState.base_dir` — mọi bên cùng thư mục
  dù core tự tính có lệch.
- Trang Cài đặt: card **"Bản mặc định"** với ba nút mở ba dialog toàn màn (`w-[min(96vw,64rem)]`, nội dung cuộn):
  - **Prompt mặc định**: hai ô Bối cảnh + Tên riêng (dùng lại `Choice`/Select như tab Thông tin), nhãn "bản cứng"
    hay "đã sửa", editor Plate (`PlatePromptEditor`, lazy như prompt-editor), nút Lưu (ghi file) và Về mặc định
    (xoá file, editor remount nạp bản cứng). Đổi genre khi đang dirty → hỏi bỏ thay đổi.
  - **Rule mặc định**: chọn bối cảnh, bảng rule dùng lại phần bảng của `CheckRulesEditor` (tách thành
    `RuleTable` thuần props để hai nơi dùng), Thêm/Xoá dòng, Lưu, Về mặc định.
  - **Glossary mặc định**: chọn bối cảnh, dùng lại `GlossaryEditor` (đang gắn RHF `useFormContext` → dialog bọc
    `FormProvider` với form riêng có field `glossary`), Lưu, Về mặc định.
- `useStoryDefaults` bỏ cache theo phiên app: nạp lại khi genre đổi **và** sau khi dialog base lưu/xoá
  (store có `baseVersion` tăng mỗi lần lưu; hook đưa vào deps). Prompt tab / Rule tab của truyện dùng mặc định
  hiện "mặc định của app (đã sửa)" khi `source === "file"`.
- Hồ sơ truyện tab Glossary thêm dòng nhỏ: "Kho chung <bối cảnh>: N mục (Cài đặt → Bản mặc định)".

## 4. Kiểm thử

- Core: `base` với `QT_AI_BASE_DIR` trỏ tempdir — không file → builtin; có file → nội dung file; file rỗng/hỏng
  → builtin; save/reset ghi và xoá đúng đường dẫn; `base_dir()` tính theo env từng hệ (test đặt `APPDATA`/`HOME`
  giả trong tiến trình test, chạy tuần tự vì env chung — dùng một test gộp). `build_system_prompt` với base
  prompt file và glossary file: prompt riêng của truyện vẫn thắng, glossary truyện đè key trùng, lọc theo chương
  vẫn chạy. `check_violations` với file rule: bộ file thay bộ cứng, rule Hán tự vẫn có.
- Tauri crate: `base_cmds` inner functions với tempdir (test crate này hiện không chạy được trên máy Win10 dev —
  chỉ biên dịch qua `cargo clippy --all-targets`); `session_config` đặt `base_dir`.
- GUI (vitest): ba dialog — mở nạp đúng nội dung/nhãn, Lưu gọi `base_save` đúng payload rồi tăng
  `baseVersion`, Về mặc định gọi `base_reset`; `useStoryDefaults` nạp lại khi `baseVersion` đổi; card Cài đặt
  có ba nút.

## Ngoài phạm vi

- CLI TypeScript `apps/qt-ai-cli` và qt-web không đọc base này.
- Export/import gói base để chia sẻ; diff giữa bản sửa và bản cứng khi app cập nhật (hiện chỉ có nhãn "đã sửa").
- Sửa theo module (port logic ghép từ qt-web).
