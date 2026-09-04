# Bối cảnh hỗn hợp: truyện xuyên qua lại cổ đại ↔ hiện đại, đô thị tu tiên

Bổ sung cho `2026-09-04-multi-genre-translation-design.md`. Không đổi gì ở hai bối cảnh sẵn có.

## Vấn đề

`genre.setting` chỉ có `ancient | modern`, chọn tĩnh cho cả truyện. Truyện xuyên qua lại giữa hai thời, hoặc hiện đại nhưng có tu luyện (đô thị tu tiên), vấp:

- Prompt chỉ có một bộ xưng hô/thán từ; module hiện đại thiếu bảng tu tiên/cung đình.
- Check rule của bối cảnh đã chọn bắt nhầm nửa còn lại (`vợ`/`tôi` trong ancient, `ngươi`/`nàng` trong modern), vòng soát sửa sai.

## Thiết kế

- `genre.setting` thêm giá trị `"mixed"`; nhãn "Hỗn hợp / xuyên qua lại", gợi ý "Chọn xưng hô theo cảnh; không bắt lỗi xưng hô". Cả ba schema (qt-web `ai-story.ts`, Rust `story.rs`, GUI `schema.ts`) mở rộng enum; giá trị lạ vẫn về `ancient`.
- Module prompt `mixed` điền đủ 6 slot của `SettingModule`; ancient/han không đổi một byte:
  - `constraints`: xác định thời của từng cảnh qua tín hiệu trong raw (điện thoại, xe, công ty, trường học → hiện đại; cung điện, tu vi, đan dược, tước vị → cổ đại) rồi dùng bộ xưng hô và thán từ tương ứng; chốt theo cặp nhân vật trong từng cảnh; đổi cảnh đổi cả bộ; không lai trong cùng câu; nhân vật xuyên không giữ thói quen xưng hô cũ nếu raw thể hiện vậy. Thán từ: cảnh cổ dùng `A?`/`Ân`, cảnh hiện đại dùng `Ừ`/`Ơ?`/`À`.
  - `pronouns`: hai bảng đại từ (cổ đại rồi hiện đại, chữ lấy từ hai module sẵn có) kèm ghi chú chọn bảng theo cảnh.
  - `terms`: bảng tu tiên + cung đình (nguyên văn ancient) rồi bảng đô thị (nguyên văn modern).
  - `inversion`: chỉ đảo ngữ cổ phong trong cảnh cổ đại.
  - `vocabulary`: cảnh cổ đại dùng thê tử/phu quân/phụ thân, cảnh hiện đại dùng vợ/chồng/bố mẹ.
  - `editing`: soát mỗi cảnh dùng đúng một bộ xưng hô, không trộn.
- Rule: `mixed` chỉ chạy rule trung lập (không tag). Logic lọc `tag === undefined || tag === setting` ở web và Rust đã cho kết quả này; không thêm rule mới.
- Golden: `prompts.json` 9 base (`mixed/han`, `mixed/foreign`, `mixed/mixed` thêm); `check.json.defaultRules.mixed` và case `mixed-default` (văn bản có cả `vợ` lẫn `ngươi` → không vi phạm xưng hô); `prompt.json` thêm 3 case.
- UI: qt-web và GUI hiện mục thứ ba nhờ map hằng; test: rule tab với mixed không có rule xưng hô (web), chọn Hỗn hợp lưu đúng (GUI).
- Workflow `/setup-story`: chọn `mixed` khi tóm tắt có xuyên không qua lại hoặc đô thị tu tiên.

## Kiểm thử

- Web: `composeBasePrompt({setting:"mixed", names:"han"})` chứa cả `| 我          | **ta**` lẫn `**anh** / **anh ta**`, chứa "### Tu tiên / Xianxia" và "### Đô thị / Hiện đại"; 9 tổ hợp cho 9 chuỗi khác nhau; `defaultAiCheckRules("mixed")` không có rule tag; `checkAiTranslationViolations(text, undefined, "mixed")` cho `vợ`, `ngươi`, `Ừm` qua nhưng vẫn bắt dấu câu Trung.
- Rust: golden khớp; `base_prompt` có 9 base khác nhau; `story.rs` normalize `"mixed"`.
- GUI: story-form/schema nhận `mixed`; story-page chọn Hỗn hợp gọi `storyDefaults` với `mixed`.
