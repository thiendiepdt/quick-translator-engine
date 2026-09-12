# qt-ai: mức nghĩ (effort) cấu hình theo từng bước gọi API

Ngày: 2026-09-12. Phạm vi: `crates/qt-ai-core` (api, api_session, api_fill), `apps/qt-ai-gui` (Rust
app_config + frontend Cài đặt). Mặc định giữ đúng hành vi hiện tại.

## Hiện trạng

Động cơ API có một nút chung cho cả phiên: Gemini `thinking: bool` (thinkingLevel high ↔ minimal, 2.5 là
budget -1 ↔ 0), OpenAI `reasoningEffort` (none…max). Nút này chỉ áp cho lượt **stream** (`generate`): dịch,
dịch lại, bù đoạn, soát. Lượt **JSON** (`complete_json`: trích glossary, AI điền hồ sơ) không gửi tham số
nghĩ nào — model tự quyết.

## Thiết kế

### Bước (`ApiStep`)

| Bước | Hàm | Mặc định (= hiện tại) |
|---|---|---|
| `translate` | dịch, dịch lại, dịch bổ sung đoạn thiếu | `high` |
| `review` | soát vi phạm | `high` |
| `glossary` | trích glossary (JSON) | `low` (chốt 2026-09-12: chỉ đối chiếu tên, cần nhanh và rẻ; hub coi "không gửi" là high) |
| `fill` | AI điền hồ sơ (JSON) | `high` (chạy một lần mỗi truyện) |

### Giá trị effort

Một chuỗi cho mỗi bước, `""` = không gửi tham số (model mặc định).

- Gemini: `minimal | low | medium | high`. Model 3.x → `thinkingLevel`; 2.5 → `thinkingBudget` 0 khi
  `minimal`, -1 (tự động) khi khác; 2.0 trở xuống bỏ qua. `""` → không có `thinkingConfig`.
- OpenAI-compatible: `none | low | medium | high | xhigh | max` → `reasoning_effort`. `""` → không gửi.

Lượt JSON giờ cũng gửi tham số nghĩ khi bước đó có giá trị khác `""`. Mặc định của hai lượt JSON đổi so
với trước (trước không gửi gì): glossary `low`, fill `high`. Config cũ chuyển đổi cũng nhận hai mặc định này.

### Core

```rust
pub enum ApiStep { Translate, Review, Glossary, Fill }
#[serde(rename_all = "camelCase", default)]
pub struct StepEfforts { translate, review, glossary, fill: String }   // Default = high/high/low/high
impl StepEfforts { fn get(&self, step) -> &str; fn legacy(provider, thinking, reasoning_effort) -> Self }
pub struct ApiConfig { provider, api_key, model, base_url, effort: StepEfforts }
ApiConfig::with_efforts(provider, key, model, base_url, StepEfforts)
ApiConfig::resolve(provider, key, model, base_url, thinking, reasoning_effort)  // giữ cho test cũ, gọi legacy()
```

`TextModel::generate` và `complete_json` nhận thêm `step: ApiStep` đầu tiên. `HttpModel` tra
`config.effort.get(step)` khi dựng body. `api_session`: translate_full/repair_missing → Translate, review →
Review, harvest_glossary → Glossary. `api_fill::fill_story` → Fill. Model giả trong test bỏ qua tham số.

### GUI Rust (`app_config.rs`)

`ProviderCredentials` thêm `effort: StepEfforts` (serde default). `ApiSettings` bỏ `thinking` /
`reasoning_effort` khỏi dữ liệu lưu; giữ hai trường legacy `Option` chỉ đọc (`skip_serializing`) để
`AppConfig::load` chuyển đổi: `thinking: false` → gemini translate/review = `minimal`; `reasoningEffort` khác
rỗng → openai translate/review = giá trị đó. `resolve()` dùng `effort` của provider đang chọn.

### Frontend

- `schema.ts`: `GEMINI_EFFORTS`, `OPENAI_EFFORTS`, `API_STEPS`, `DEFAULT_STEP_EFFORTS`; credentials theo
  provider có `effort` với enum tương ứng; bỏ `thinking`/`reasoningEffort` (zod strip khi config cũ còn).
- `engine-form.ts`: `geminiEffort`, `openaiEffort` (object 4 khoá) thay `thinking`/`reasoningEffort`.
- Cài đặt → Động cơ dịch: dưới key/model/base URL của provider đang chọn là bảng "Mức nghĩ theo bước",
  4 dòng (Dịch, Soát, Trích glossary, AI điền hồ sơ), mỗi dòng một Select. Mục `""` hiện là "Mặc định
  model" (Radix Select không nhận value rỗng nên UI dùng sentinel `default`). Hint nêu cách map theo
  provider. Khoá khi phiên đang chạy như các ô khác.

## Test

- Core unit: `generation_config(model, effort)` các thế hệ; body OpenAI stream/JSON có/không
  `reasoning_effort` theo bước; `StepEfforts::legacy`.
- `tests/api_http.rs`: một ca dựng `ApiConfig::with_efforts` với translate `high`, glossary `low` → body
  stream và JSON mang đúng giá trị từng bước.
- GUI Rust: config cũ có `thinking:false` / `reasoningEffort:"xhigh"` load ra effort đã chuyển; serialize
  không còn hai trường cũ.
- Frontend: schema default/migration (config cũ vẫn parse), engine-form round-trip, settings-page hiện bảng
  4 dòng và đổi giá trị ghi vào form.

## Ngoài phạm vi

Effort riêng cho agy (agy không có tham số này), effort theo từng truyện.
