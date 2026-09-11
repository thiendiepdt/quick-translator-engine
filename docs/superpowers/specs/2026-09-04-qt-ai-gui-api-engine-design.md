# qt-ai-gui: động cơ dịch bằng API key bên cạnh Antigravity CLI

## Mục tiêu

Người dùng chọn một trong hai động cơ dịch trong Cài đặt:

- **agy** (hiện có): điều khiển Antigravity CLI theo vòng phiên, agent tự dịch/check/accept.
- **api** (mới): app tự gọi model qua HTTP bằng key của người dùng — Gemini chính chủ
  hoặc bất kỳ hub OpenAI-compatible (OpenAI, hub riêng `http://…/v1`) — rồi tự chạy
  cùng vòng next → dịch → check → accept của `qt-ai-core`. Không cần cài agy.

Cùng folder truyện, cùng `state.json`/`work/`/`out/`; đổi động cơ giữa chừng vẫn tiếp được.

## Cấu hình (`config.json` của app)

```jsonc
{
  "engine": "agy" | "api",                 // mặc định "agy"
  "api": {
    "provider": "gemini" | "openai",       // mặc định "gemini"
    "gemini": { "apiKey": "", "model": "gemini-3.7-flash", "baseUrl": "" },
    "openai": { "apiKey": "", "model": "gpt-5.6-sol", "baseUrl": "" },
    "thinking": true,                      // Gemini: thinkingLevel high ↔ minimal
    "reasoningEffort": "high"              // OpenAI: none|low|medium|high|xhigh|max
  }
}
```

Key lưu plain trong config.json ở thư mục cấu hình app (như qt-web lưu localStorage);
config cũ thiếu trường → default, không chặn khởi động.

## Core (`crates/qt-ai-core`)

- `api/` — port của `qt-web/src/lib/ai-text-client.ts` + `completeJson`:
  - `ApiConfig { provider, api_key, model, base_url, thinking, reasoning_effort }`,
    base URL mặc định `https://generativelanguage.googleapis.com` /
    `https://api.openai.com/v1`, cắt `/` cuối.
  - Trait `TextModel { generate(system, user, on_progress, cancel) -> Result<String, ApiError>;
    complete_json(system, user) -> Result<String, ApiError> }` để vòng dịch test được
    bằng model giả.
  - `HttpModel` dùng `reqwest::blocking` (rustls + ring, không cần OpenSSL/cmake):
    Gemini `:streamGenerateContent?alt=sse` với `x-goog-api-key`, safetySettings OFF,
    thinkingConfig theo major version; OpenAI-compatible `chat/completions` stream,
    `max_completion_tokens`, `reasoning_effort`, đọc `delta.content` /
    `reasoning_content` / `reasoning`. JSON: Gemini `generateContent` với
    `responseMimeType: application/json`; OpenAI `response_format: json_object`
    (không `temperature`).
  - `ApiError::{Blocked(reason), Http{status, message}, Network(message), Empty, Cancelled}`.
  - Stream đọc từng dòng SSE; kiểm cờ cancel giữa các chunk (drop response là huỷ).
- `api_session` — vòng dịch không agent, dùng lại `run_next`/`run_check`/`run_accept`/`run_skip`:
  1. Chương translating dở (phiên trước bị huỷ) làm tiếp trước; hết thì `run_next` phát chương
     mới. System prompt lắp lại bằng `build_system_prompt` + `labeled_source_payload` (không đọc
     file prompt của agent).
  2. `generate` → parse nhãn `[[n]]`; đoạn thiếu → gọi bổ sung bằng `labeled_repair_payload`;
     vẫn thiếu → giữ nguyên văn Hán (rule CJK sẽ bắt). Ghi `work/<id>.draft.md` dạng
     `[[n]] đoạn`.
  3. `complete_json` trích tên riêng (prompt `GLOSSARY_EXTRACT` của qt-web) → ghi
     `work/<id>.glossary.json`; lỗi → `{"entries":[]}`.
  4. `run_check`; FAIL còn lượt: thiếu đoạn → bổ sung như bước 2; vi phạm → lượt soát
     tối thiểu trên draft có nhãn (system/user port từ `buildAiTranslationReviewPrompt`,
     liệt kê `[[n]]`), chỉ nhận bản soát nếu đủ nhãn cũ; quá ngắn → dịch lại cả chương,
     giữ bản dài hơn. Lặp tới PASS / chốt kèm cảnh báo / escalate error.
  5. PASS → `run_accept`. Error → sang chương kế. Model từ chối (`Blocked`) →
     `run_skip(id, "model từ chối: …")`.
  - Lỗi API khác Blocked: đợi 5 s thử lại đúng chương một lần; vẫn lỗi → skip chương kèm
    "lỗi API: …"; chương thứ hai liên tiếp lỗi → dừng `StopReason::ApiFailed{message}`, chương đó
    giữ translating để phiên sau làm tiếp.
  - Event: `Started{session_no:1}`, `Progress` sau mỗi thay đổi state, `AgyLog` cho mốc
    ("Chương 0012: dịch 3.2k ký tự, soát 1 lần, +2 glossary") và tiến độ nhận stream
    mỗi ~5 s, `Stopped`. `chapters_per_session`/`max_sessions` không áp dụng.
  - Dùng chung lock `work/.session.lock`, `SessionHandle`, cancel.

## Tauri

- `AppConfig` thêm `engine`, `api` (serde default). `session_start` rẽ theo `engine`:
  agy như cũ; api → `start_api_session(root, ApiConfig, sink)`; thiếu key → lỗi
  `api_key_missing`. `ai_fill_story` vẫn chỉ agy (API mode: nút bị vô hiệu kèm gợi ý).

## UI

- Cài đặt: card "Động cơ dịch" với hai lựa chọn; chọn API thì hiện provider
  (Gemini / OpenAI-compatible), API key (password), model, Base URL, Thinking (Gemini)
  hoặc Mức nghĩ (OpenAI). Lưu chung nút hiện có.
- Cổng "Chưa thấy agy" chỉ chặn khi `engine === "agy"`; có nút "Dùng API key thay thế".
- Toolbar Dịch: engine agy giữ dropdown model agy; engine api hiện nhãn
  "API · Gemini · gemini-3.7-flash" và bấm mở Cài đặt. Log panel đổi nhãn theo engine.
- Schema: `appConfigSchema` thêm `engine`/`api` với default; `stopReason` thêm
  `api_failed { message }`.

## Kiểm thử

Rust: parse SSE Gemini/OpenAI + build body (unit); vòng dịch trên tempdir với model giả
(pass thẳng, thiếu đoạn rồi bổ sung, vi phạm rồi soát, từ chối → skip, lỗi mạng 2 lần →
ApiFailed, cancel). Tauri: config default/migration, session_start từ chối khi thiếu key.
UI: schema default cho config cũ, settings page hiện/ẩn theo engine, toolbar theo engine.
