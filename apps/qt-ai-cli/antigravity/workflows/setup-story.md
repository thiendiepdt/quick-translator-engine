# /setup-story — điền hồ sơ truyện trước khi dịch

1. Chạy `{{QT_AI}} init {{STORY_ROOT}}` (an toàn chạy lại).
2. Đọc 2–3 chương đầu trong `raw/` (chỉ để nắm truyện, không dịch).
3. Mở `story.json`, điền bằng tiếng Việt:
   - `name`, `protagonist`, `summary` (3–5 câu, không spoil quá chương đã đọc).
   - `style.voice` (1 câu tả giọng kể), `style.toneRules` (3–5 luật xưng hô/giọng điệu rút từ chính truyện),
     `style.avoid` (những kiểu diễn đạt cần tránh với truyện này).
   - `glossary`: seed các tên riêng gặp trong các chương đã đọc vào đúng nhóm
     (`names`, `places`, `items`, `creatures`, `skills`, `common`, `signature_phrases`) — source chữ Hán, target Hán-Việt.
   - Giữ nguyên các field khác (`customPrompt`, `checkRules`, `autoGlossaryLog`, `autoGlossary`).
4. Trình `story.json` cho người dùng duyệt trước khi chạy /translate.
