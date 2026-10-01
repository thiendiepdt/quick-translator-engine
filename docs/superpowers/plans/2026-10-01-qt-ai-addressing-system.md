# Hệ thống xưng hô qt-ai — kế hoạch thực thi

> Spec: `docs/superpowers/specs/2026-10-01-qt-ai-addressing-system-design.md`. Thực thi inline trong
> một phiên, TDD từng task (test đỏ trước), commit chỉ khi user bảo.

**Goal:** Bảng nhân vật (giới tính) + xưng hô theo mốc chương + luật xưng hô + check, không đổi schema `story.json`.

**Architecture:** Lớp mới `cast` trong qt-ai-core bọc quanh các hàm port từ qt-web (không sửa hàm port).
Dữ liệu ở `cast.json`. Prompt chương = `build_system_prompt` (đã rút nhóm addressing) + mục `# Xưng hô`
chèn trước suffix.

**Tech Stack:** Rust (qt-ai-core, serde_json, regex), Tauri commands, React + zod + vitest.

## Global Constraints

- Không sửa `build_system_prompt`, `filter_glossary_for_source`, `sanitize_extracted`,
  `append_auto_glossary`, `DEFAULT_RULES`, `prompts.json` — golden phải xanh nguyên.
- `story.json` không thêm field. `cast.json` thiếu/hỏng = rỗng.
- Comment, tên test tiếng Việt như code xung quanh. Màu GUI dùng theme token.

## Task 1 — `cast.rs`: dữ liệu + đọc/ghi

- Create `crates/qt-ai-core/src/cast.rs`; modify `lib.rs`, `story_fs.rs` (`StoryPaths.cast_json`).
- Produces: `Gender {Male, Female}`, `EntrySource {Auto, User}`, `Character`, `AddressChange`,
  `AddressTimeline`, `Cast`, `load_cast(&StoryPaths) -> Cast`, `save_cast(&StoryPaths, &Cast) -> Result<()>`.
- Test: round-trip; file thiếu/hỏng → rỗng; bỏ mục sai schema.

## Task 2 — giới của từ xưng hô + chuẩn hoá cặp

- `cast.rs`: `term_gender(&str) -> Option<Gender>`, `normalize_pair(&str) -> Option<(String, String)>`
  (đảo cặp ngược), `pair_gender_conflict(key, target, &Cast) -> bool`.
- Test: thầy Hạ/cô/chị/sư tỷ/Tống công tử; đệ tử, cậu, em trung tính; `mày–tao` → `tao–mày`.

## Task 3 — cặp hiệu lực theo chương + lọc theo chương

- `cast.rs`: `effective_addressing(&StringMap, &Cast, chapter_id) -> StringMap`,
  `chapter_addressing(&StringMap, source) -> StringMap` (cả hai bên có mặt).
- Test: mốc đứng trước/sau chương; dịch lại chương cũ; một bên vắng thì bỏ.

## Task 4 — nhân vật có mặt + mục prompt `# Xưng hô`

- `cast.rs`: `present_characters(names, &Cast, source) -> Vec<(han, viet, Gender)>`,
  `addressing_section(...) -> String`; `prompt.rs`: `build_chapter_prompt(workspace, story, cast, chapter_id, source)`.
- Nối vào `next.rs` (`write_prompt_file`) và `api_session.rs` (`translate_chapter`).
- Test: không cast + không addressing → luật chung vẫn có, golden `build_system_prompt` không đổi;
  có cast → liệt kê đúng người; 贺老师 kéo theo 贺静昭; cặp một bên vắng không xuất hiện.

## Task 5 — check: sai giới + lời gọi thô

- `cast.rs`: `gender_violations(paragraphs, parsed, names, &Cast) -> Vec<ParagraphIssue>`,
  `vocative_violations(paragraphs, parsed) -> Vec<ParagraphIssue>`; nối vào `commands/check.rs`
  và điểm soát trong `api_session.rs::review`.
- Test: "thầy Hạ" cho nữ bị bắt; hai người cùng họ trong chương → bỏ qua; "Đàn em, cậu…" bị bắt,
  "đàn chị" ngoài ngoặc không bị bắt; vòng dịch nhận bản soát sửa giới.

## Task 6 — accept: học giới, siết cặp mới, nhận đổi cặp

- `cast.rs`: `apply_extracted(entries, raw, story, &mut Cast, chapter_id) -> CastUpdate`;
  `glossary.rs`: lọc cặp mới (hai bên là tên, hợp giới, đảo chiều); `commands/accept.rs` ghi `cast.json`.
- Chỉ dẫn model: `GLOSSARY_INLINE_INSTRUCTION`, `GLOSSARY_EXTRACT_SYSTEM_PROMPT`, `agent_instructions`.
- Test: học giới có căn cứ; không căn cứ → bỏ; khai ngược → disputed; cặp `保安→…` bị bỏ; cặp trái giới
  bị bỏ; đổi cặp có note → `changes`; ghim → không đổi; đảo về giá trị trước → bỏ; dịch lại cùng chương
  → thay mục cùng `from`.

## Task 7 — quét bù giới tính

- Create `crates/qt-ai-core/src/cast_scan.rs`: `collect_snippets`, `build_scan_prompt`, `scan_genders(paths, model, cancel) -> Result<ScanOutcome>`.
- Test với model giả: chỉ hỏi tên chưa có giới, lô 40, chỉ điền chỗ trống, JSON hỏng bỏ qua lô.

## Task 8 — Tauri + GUI

- `story_cmds.rs`: `load_cast`, `save_cast`, `clean_cast_addressing`; `save_story` ghim cặp user sửa.
  `session_cmds.rs`: `scan_cast` (động cơ API).
- Frontend: `lib/schema.ts` + `lib/api.ts` + `components/cast-panel.tsx` (+ test) gắn vào `story-page.tsx`.

## Task 9 — chạy thử thật

- Bản sao hai truyện trong scratchpad; quét giới; dịch lại ~20 chương có lỗi đã biết; so trước/sau;
  báo số liệu, chỉnh luật nếu cần.
