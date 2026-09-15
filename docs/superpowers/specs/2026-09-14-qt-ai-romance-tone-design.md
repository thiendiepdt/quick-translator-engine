# qt-ai: trục "giọng văn" cho truyện nữ (ngôn tình)

Ngày: 2026-09-14. Phản hồi người dùng: bản dịch qt-ai-gui khô, hợp truyện nam, truyện nữ cần ngọt và hài.

## Chẩn đoán

Base prompt (nguồn `apps/qt-web/src/lib/ai-translation-prompt.ts`) nghiêng hẳn về tiết chế: "độ trung thành cao hơn độ mượt",
cấm "tô màu, thêm chất thơ", cấm loạt từ trang sức ("khẽ", "chợt", "bất giác"…), suffix bắt "không làm câu giàu cảm xúc hơn".
Genre chỉ có hai trục (bối cảnh, tên riêng), không có trục giọng. A/B trên `chuc-tieu-dao` (chương 5, 24) với mục
"Giọng văn: ngôn tình" chèn vào base: thoại tự nhiên hơn (tiểu từ, xưng hô tỷ/muội đúng raw), lời bình có duyên hơn;
lời kể đổi ít. Người dùng duyệt hướng này.

## Thiết kế

- `StoryGenre` thêm `tone: "neutral" | "romance"`, mặc định `neutral` (truyện cũ, story.json thiếu field → không đổi).
  Key base prompt vẫn `setting/names` (9 base), tone không nhân combo.
- qt-web `composeBasePrompt(genre)` chèn `TONES[genre.tone]` (mảng dòng) ngay sau `CORE_PHILOSOPHY`, trước
  `setting.pronouns` (mục "## 1. Đại từ nhân xưng"). `neutral` = mảng rỗng → prompt y hệt cũ từng byte.
- `prompts.json` (gen-golden) thêm `tones: { romance: "<các dòng join \n>" }`. Rust `build_system_prompt` lấy base
  (mặc định hoặc file người dùng sửa ở Bản mặc định) rồi chèn `tones[tone]` trước dòng `## 1. Đại từ nhân xưng`;
  base không có dòng đó (người dùng sửa tay) → nối vào cuối base. Prompt riêng của truyện (`customPrompt`) không chèn.
  Golden case `genre-ancient/han-romance` bắt Rust khớp web từng byte.
- Nội dung mục romance: sắc thái ngọt/hài/chớt nhả là nội dung phải giữ; thoại dùng tiểu từ, từ láy; bảng ví dụ
  khô → đúng giọng (dấu nháy cong); vẫn cấm thêm ý. Văn bản chốt từ A/B ở scratchpad `tone_romance.md`.
- UI: qt-ai-gui Hồ sơ truyện thêm Select "Giọng văn" (Trung tính / Ngôn tình) cạnh Bối cảnh, Tên riêng; qt-web
  dialog cấu hình truyện thêm tương tự. `story-form` thêm `genreTone`.
- AI điền: prompt JSON thêm `genre.tone` với mô tả; merge như hai trục kia.
- CLI TS dùng thẳng `buildAiTranslationSystemPrompt` của qt-web nên nhận tone tự động.

## Kiểm thử

qt-web: normalize tone, composeBasePrompt neutral không đổi hash legacy, romance chứa mục. Core: normalize tone,
chèn tone (có/không anchor, không chèn cho custom prompt), golden romance, api_fill parse tone. GUI: form
round-trip tone, story page có Select Giọng văn gọi `storyDefaults` với tone.
