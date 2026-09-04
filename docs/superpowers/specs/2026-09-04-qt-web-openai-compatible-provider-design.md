# qt-web: provider OpenAI-compatible cho Dịch AI

## Mục tiêu

Thêm provider thứ năm `openai` vào qt-web, dùng được cho cả lọc tên lẫn Dịch AI.
Mặc định trỏ vào OpenAI chính chủ (`https://api.openai.com/v1`), nhưng Base URL
sửa được để trỏ sang hub OpenAI-compatible bất kỳ (ví dụ `http://192.0.2.10/v1`
với model `gemini-3.7-flash`).

## Thiết kế

- **Settings** (`ai-settings.ts`): thêm `"openai"` vào `AiProvider`, config riêng
  `openai: { apiKey, model, baseUrl }`, model mặc định `gpt-5.6-sol` cho cả lọc tên
  và dịch. Normalize/migrate như các provider khác.
- **Routing** (`ai-client.ts`): base URL mặc định `https://api.openai.com/v1`,
  model fallback `gpt-5.6-sol`. `completeJson` với `openai` không gửi `temperature`
  (GPT-5/o-series chỉ nhận giá trị mặc định).
- **Stream** (`ai-text-client.ts`): đi đường `chat/completions` sẵn có. Riêng
  `openai` dùng `max_completion_tokens` thay `max_tokens`. Công tắc Thinking không
  áp dụng cho OpenAI; thay vào đó Cài đặt có ô "Mức nghĩ" chọn `reasoning_effort`
  (none/low/medium/high/xhigh/max, mặc định high, lưu ở
  `translation.openaiReasoningEffort`) và gửi thẳng cho mọi model, kể cả model
  hub relay. Stream đọc `delta.reasoning` như `reasoning_content`.
- **Base URL http**: `baseUrlProblem(baseUrl, pageProtocol)` cho phép `http://`
  host bất kỳ khi chính trang đang chạy `http:` (dev local). Trang https giữ
  luật cũ vì trình duyệt chặn mixed-content.
- **UI** (`settings-dialog.tsx`): thêm mục "OpenAI" ở hai dropdown, placeholder
  key/model, tên nhà cung cấp trong ghi chú, ghi chú Thinking.
- **Docs**: README nhắc provider OpenAI-compatible.

## Kiểm thử

Vitest theo khuôn GLM: routing mặc định, body request (không `max_tokens`, không
`temperature`, `reasoning_effort` theo model), parse `delta.reasoning`, settings
round-trip, `baseUrlProblem` theo protocol trang.
