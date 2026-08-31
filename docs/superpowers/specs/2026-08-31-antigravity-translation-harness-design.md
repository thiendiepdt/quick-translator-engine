# Antigravity Translation Harness — Design

**Ngày:** 2026-08-31
**Mục tiêu:** Tận dụng quota Antigravity gói Ultra để dịch truyện hàng loạt, thay cho chi phí API của qt-web. Migrate 1-1 pipeline dịch AI + quản lý glossary/story từ qt-web sang bộ công cụ CLI + skill/workflow chạy trên Antigravity. Output là file txt/md, không cần import ngược vào web.

## Bối cảnh & quyết định nền

- Phần "não" của tính năng dịch web nằm trong 5 module logic thuần (không dính DOM): `ai-translation-prompt.ts`, `ai-translation.ts`, `ai-paragraphs.ts`, `ai-glossary.ts`, `ai-story.ts`. Các phần web-only (`ai-client.ts`, `ai-text-client.ts`, `ai-settings.ts`, `ai-story-fill.ts`, UI) **không port** — Antigravity chính là provider.
- Chống hết context window bằng thiết kế **file-state-driven**, không cần sub-agent thật (Antigravity không có cơ chế spawn sub-agent kiểu Claude Code): mỗi chương là một đơn vị công việc, tiến độ nằm trong `state.json` + file output; phiên agent nào mở lên cũng resume được.
- Rủi ro đã chấp nhận:
  1. Prompt dịch chỉ là user-level dưới system prompt của Antigravity → nội dung nặng đô có thể bị refuse nhiều hơn bản gọi API; xử bằng `qt-ai skip` rồi dịch riêng các chương đó bằng web/API.
  2. Agent dịch dài hay tóm tắt bớt → enforce bằng script (nhãn đoạn + parse), không tin agent.
  3. Dùng quota IDE cho dịch hàng loạt là vùng xám ToS Google; quota Ultra vẫn có rate limit.

## Kiến trúc

**Nguyên tắc: không đụng code qt-web.** CLI import thẳng source từ qt-web qua tsconfig path alias (`@/*` → `../qt-web/src/*`), chạy bằng `tsx`, không cần build. Web và CLI dùng cùng một file nên không bao giờ drift. (Đã cân nhắc tách `packages/qt-ai-core` — sạch hơn về lâu dài nhưng phải sửa hàng chục import trong qt-web, chưa đáng.)

### Cấu trúc mới trong repo

```
apps/qt-ai-cli/
  package.json, tsconfig.json      # tsx + vitest, alias @/ → ../qt-web/src
  src/
    main.ts                        # entry: qt-ai <lệnh>
    story-fs.ts                    # đọc/ghi story.json, state.json, raw/, out/, work/
    commands/                      # init, next, check, accept, skip, status
  antigravity/                     # template copy vào workspace truyện
    AGENTS.md                      # luật cho agent Antigravity
    workflows/
      setup-story.md               # điền story.json từ các chương đầu
      translate.md                 # vòng lặp dịch batch
```

### Cấu trúc folder truyện (ngoài repo — chính là workspace Antigravity mở)

```
story.json        # AiStoryConfig y hệt schema web: glossary 7 nhóm, style,
                  # customPrompt, checkRules, autoGlossaryLog, autoGlossary
state.json        # per-chapter: queued|translating|error|done|skipped,
                  # reviewRound, lý do skip/error, thống kê
raw/0001.txt …    # chương gốc
out/0001.md …     # bản dịch đã chốt
work/             # prompt lắp sẵn, bản nháp, đề xuất glossary, báo cáo check
                  # của chương đang dịch
AGENTS.md
.agent/workflows/ # copy từ template
```

`story.json` cùng schema với web nên config/glossary export từ web dán vào là chạy.

## Lệnh CLI

Mỗi lệnh idempotent; mọi ghi state theo kiểu atomic (write temp + rename); mọi lệnh validate schema `story.json`/`state.json` trước khi chạy.

- **`qt-ai init <folder>`** — quét `raw/`, tạo `state.json` (mỗi chương `queued`), tạo `story.json` rỗng đúng schema, copy template AGENTS.md + workflows.
- **`qt-ai next`** — chọn chương `queued` kế tiếp, lắp prompt hoàn chỉnh bằng `buildAiTranslationSystemPrompt` + `filterTranslationGlossaryForSource` + `labeledAiSourcePayload` (cùng code với web), ghi `work/NNNN.prompt.md`, đánh dấu `translating`. Từ chối phát chương mới khi còn chương `translating` chưa chốt.
- *(agent dịch)* — đọc prompt, ghi `work/NNNN.draft.md` (giữ nhãn đoạn) + `work/NNNN.glossary.json` (đề xuất tên riêng mới, có category).
- **`qt-ai check NNNN`** — hàng rào deterministic: `parseLabeledAiTranslation` bắt thiếu đoạn/sai thứ tự; `checkAiTranslationViolations` (rule mặc định + `checkRules` riêng của truyện); sanity tỉ lệ độ dài: fail khi tổng ký tự bản dịch < 75% tổng ký tự raw (tiếng Việt thường dài hơn Hán tự, dưới ngưỡng này gần như chắc chắn dịch thiếu; ngưỡng config được trong `story.json`). Fail → sinh `work/NNNN.review.md` từ `buildAiTranslationReviewPrompt`, `reviewRound++`; quá 2 vòng → chương thành `error`, đi tiếp. Exit code khác 0 khi fail.
- **`qt-ai accept NNNN`** — chỉ nhận khi check pass (`--force` để ghi đè có chủ đích): strip nhãn → `formatAiTranslation` → `out/NNNN.md`; đề xuất glossary qua `sanitizeExtractedGlossary` (source phải có trong raw, target phải có nguyên văn trong bản dịch, chỉ key mới) → merge `story.json` + ghi `autoGlossaryLog`; backup `story.json.bak` trước khi ghi; chương → `done`; dọn `work/`.
- **`qt-ai skip NNNN --reason <lý do>`** — cho chương bị model từ chối: ghi lý do, chương → `skipped`, đi tiếp. Xử riêng sau bằng web/API.
- **`qt-ai status`** — bảng tiến độ, danh sách chương `error`/`skipped`.

## Phía Antigravity

- **`/setup-story`** (tương đương `fillAiStoryConfig` bên web): agent đọc vài chương đầu trong `raw/`, điền `name`, `protagonist`, `summary`, `style`, seed glossary vào `story.json`; user duyệt trước khi dịch.
- **`/translate`**: vòng lặp `qt-ai next` → dịch → `qt-ai check` (sửa theo `work/NNNN.review.md`, tối đa 2 vòng) → `qt-ai accept` → chương tiếp.
- **AGENTS.md enforce vệ sinh context:** không đọc lại `out/` các chương đã xong; không giữ bản dịch cũ trong context; nếu bị chặn bởi policy thì `qt-ai skip` chứ không chế lại nội dung; dịch tối đa **K chương/phiên** (mặc định 10, chỉnh sau pilot) rồi dừng, báo user mở phiên mới — resume tự nhiên qua `state.json`.

## Xử lý lỗi & sự cố

- Phiên chết giữa chừng → `next` phát lại đúng chương đang `translating` (file `work/` còn nguyên).
- Agent quên `accept` → `next` từ chối phát chương mới.
- `story.json` bị ghi hỏng → validate schema ở mọi lệnh + `story.json.bak` mỗi lần accept.
- Chạy song song nhiều agent: **không hỗ trợ** trong bản đầu (glossary tự học phải tuần tự); state ghi atomic để giảm rủi ro nếu lỡ xảy ra.

## Testing (TDD, vitest riêng trong `apps/qt-ai-cli`)

- **State machine:** `next` phát đúng chương và chặn khi còn `translating`; `accept` bị chặn khi chưa pass check; `skip`; resume sau khi phiên chết giữa chừng.
- **Check gating:** thiếu đoạn/sai thứ tự → fail + sinh review file; quá 2 vòng → `error`; rule per-story nạp đúng.
- **Accept:** sanitize glossary end-to-end (đề xuất bịa bị loại, đề xuất thật vào `story.json` + `autoGlossaryLog`); backup; output strip nhãn + format.
- **Prompt parity:** snapshot test khẳng định prompt CLI lắp ra giống hệt web với cùng story config — bằng chứng 1-1.
- Test hiện có của qt-web giữ nguyên, không sửa.
- Verify trước khi báo xong: `vitest` + `tsc --noEmit` trong `apps/qt-ai-cli`.

## Lộ trình chạy thử

1. **Dry-run không cần Antigravity:** Claude đóng vai agent, chạy trọn vòng `next` → dịch → `check` → `accept` trên 2-3 chương corpus `phuong-thon-dao-chu` (đã có glossary gold) để nắn CLI + workflow.
2. **Pilot trên Antigravity thật:** mở folder truyện làm workspace, `/setup-story` rồi `/translate`; đo: số chương/phiên trước khi đầy context, tỉ lệ fail check, tỉ lệ refuse.
3. Chỉnh K và AGENTS.md theo số đo rồi mới chạy hàng loạt.

## Ngoài phạm vi

- Import bản dịch ngược vào qt-web, đóng gói EPUB.
- Chạy nhiều agent song song.
- Port UI, engine convert VietPhrase, provider client/API key.
- Tự động hóa headless Antigravity (vận hành là mở IDE, ra lệnh workflow).
