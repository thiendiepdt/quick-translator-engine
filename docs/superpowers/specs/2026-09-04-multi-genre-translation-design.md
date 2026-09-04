# Dịch nhiều thể loại: prompt và rule ghép theo bối cảnh × tên riêng

## Vấn đề

Prompt dịch và bộ check rule mặc định (qt-web `ai-translation-prompt.ts`, `ai-translation.ts`; Rust đọc bản golden `prompts.json`, `check.rs`) là một khối duy nhất viết cho truyện cổ đại / tiên hiệp: đại từ hắn/nàng/ta/ngươi, cấm "vợ/chồng", thán từ `A?`/`Ân`, tên riêng bắt buộc Hán-Việt, bảng tu tiên và cung đình. Dịch truyện đô thị hiện đại hay truyện có nhân vật phương Tây bằng bộ này cho ra văn cổ trang, và vòng soát tự động "sửa" `vợ` thành `thê tử`, `Emily` thành `Ngải Mễ Lệ`.

Lối thoát hiện có chỉ là per-story và toàn-phần: `customPrompt` thay cả base, `checkRules` thay cả rule set. Người dùng phải tự viết lại ~470 dòng prompt. `story.json` không có khái niệm thể loại dù `/setup-story` và AI fill có tra được.

## Mục tiêu

- Chọn thể loại trong Hồ sơ truyện bằng hai trục độc lập; prompt và rule mặc định tự ghép theo lựa chọn.
- Một nguồn chữ duy nhất (qt-web) cho cả qt-web, qt-ai-gui (agy lẫn API) và qt-ai-cli, qua golden như hiện nay.
- Truyện đang dịch (không có `genre`) ra **đúng từng chữ** prompt và rule như trước.

Ngoài phạm vi: dọn rule/prompt lộ truyện cụ thể (`方寸`, Miêu Ảnh Vô Tung), tự đoán thể loại khi mở truyện cũ, ngôn ngữ nguồn khác tiếng Trung.

## Mô hình dữ liệu

```jsonc
// story.json
"genre": { "setting": "ancient" | "modern", "names": "han" | "foreign" | "mixed" }
```

- `setting` quyết định xưng hô, thán từ, từ gia đình, bảng thuật ngữ thời đại, rule set.
- `names` quyết định cách xử lý nhân danh/địa danh phiên âm.
- Thiếu hoặc sai giá trị → normalize về `{ "setting": "ancient", "names": "han" }` ở cả ba nơi có schema: qt-web `ai-story.ts` (`normalizeAiStoryConfig`), Rust `story.rs` (`StoryConfig::from_value`), GUI `schema.ts` (zod default). `init` ghi giá trị mặc định tường minh.
- Ưu tiên không đổi: `customPrompt` không rỗng thay toàn bộ phần base đã ghép; `checkRules` không rỗng thay toàn bộ rule mặc định; rule CJK sót vẫn cứng.
- Hệ thống / game / vô hạn lưu không là trục riêng: mục `【】` và tiếng lóng game nằm trong core, luôn bật.

## Ghép prompt

Base cũ tách thành module, giữ nguyên chữ ở phần tách ra:

| Module | Nội dung |
| --- | --- |
| `core` | Mở đầu, ngữ cảnh tác vụ, suy nghĩ trước khi dịch, ràng buộc hệ thống trừ 11 và 13, triết lý dịch, mục 0 trung thành, mục 2 chống convert (phần chung + bảng dấu hiệu convert), mục 4 dịch câu, mục 5 "Tránh dùng" + thành ngữ, mục 6 khẩu ngữ trừ `蓝星`, mục 7 hệ thống, mục 8 biên tập trừ dòng 14 |
| `settings.ancient` | Ràng buộc 11 (`我`→ta) và 13 (thán từ A?/Ân), mục 1 bảng đại từ + hai ghi chú, bảng tu tiên/cảnh giới/vật phẩm/kiến trúc/không gian hệ thống/tên dễ lẫn âm/viết hoa, mục cổ đại–cung đình–triều Thanh, đảo ngữ cổ phong, mục 5 "TUYỆT ĐỐI CẤM" vợ/chồng, `蓝星`→lam tinh, biên tập dòng 14 |
| `settings.modern` | Viết mới, xem dưới |
| `names.han` | Mục 3 hiện tại (phiên âm Hán-Việt, bảng Kế Duyên / Ji Yuan) |
| `names.foreign` | Viết mới, xem dưới |
| `names.mixed` | Viết mới, xem dưới |
| `suffix` | Không đổi |

Thứ tự ghép: `core` + `settings[setting]` + `names[names]` + thông tin truyện + glossary + style + `suffix`. Số thứ tự đầu mục trong core được đánh lại cho liền mạch; test golden chốt tổ hợp `ancient/han` bằng đúng chuỗi cũ nên mọi khác biệt chữ đều đỏ.

### `settings.modern`

- Đại từ: `他` → anh / anh ta / hắn (hắn chỉ cho nhân vật lạnh, phản diện, hoặc khi style quy định); `她` → cô / cô ta / chị; `我` → tôi trong lời kể ngôi thứ nhất; trong thoại chọn theo quan hệ và tuổi (anh–em, tớ–cậu, tao–mày, cháu–bác), chốt cho từng cặp nhân vật trong phần suy nghĩ và giữ nhất quán cả chương; `他们` → họ / bọn họ; `你们` → các cậu / mọi người theo ngữ cảnh.
- Từ gia đình và xã hội đời thường được ưu tiên: `老公/老婆` → chồng/vợ, `爸/妈` → bố/mẹ, `老师` → thầy/cô, `学长/学姐` → anh/chị khoá trên, `同事` → đồng nghiệp. Cấm phụ thân/mẫu thân/thê tử/phu quân/lang quân trừ khi nhân vật cố tình nói cổ (style.toneRules mở).
- Thán từ hiện đại: `嗯` → Ừ / Ừm, `咦` → Ơ?, `哦` → À / Ồ, `哎呀` → Ôi / Trời ơi, `卧槽` → vãi / chết tiệt, `妈的` → mẹ kiếp / mẹ nó. Không dùng `A?`/`Ân` kiểu cổ.
- Chống "văn convert đô thị": `总裁` → tổng giám đốc (không "tổng tài" trừ glossary), `手机` → điện thoại, `微信/淘宝` giữ WeChat/Taobao, `元` → tệ, `高考` → thi đại học, `公司` → công ty, `小区` → khu chung cư, `打车` → bắt taxi. Thương hiệu, app, mã chứng khoán giữ Latin. Địa danh Trung Quốc vẫn Hán-Việt (Bắc Kinh, Thâm Quyến, Hàng Châu).
- Cấm `ngươi`, `nàng`, `ta` làm đại từ và cấm đảo ngữ cổ phong trong đô thị; ngoại lệ xuyên không / nhân vật cổ trang do `style.toneRules` mở.

### `names.foreign`

- Nhân danh, địa danh, tổ chức phiên âm bằng chữ Hán trả về dạng gốc, tra glossary trước: `艾米丽` → Emily, `迈克尔` → Michael, `纽约` → New York, `霍格沃茨` → Hogwarts. Có tên tiếng Việt quen thuộc thì dùng: `巴黎` → Paris, `美国` → Mỹ, `英国` → Anh, `莫斯科` → Moscow.
- Nhật / Hàn theo dạng phổ biến ở Việt Nam: `鸣人` → Naruto, `佐藤` → Sato, `金秀贤` → Kim Soo-hyun; glossary thắng.
- Tên hư cấu không tra được: phiên Latin gần nhất theo âm, không Hán-Việt hoá, chốt trong phần suy nghĩ và dùng nhất quán; mục tự thêm glossary ghi dạng đã chốt.
- Bảng thuật ngữ phương Tây: `公爵` công tước, `侯爵` hầu tước, `伯爵` bá tước, `骑士` hiệp sĩ, `领主` lãnh chúa, `法师` pháp sư, `魔法` ma pháp, `精灵` tinh linh, `矮人` người lùn, `教皇` giáo hoàng, `神父` cha xứ, `先生/小姐/夫人` → ngài / cô / phu nhân hoặc Mr. / Miss / Mrs. tuỳ giọng, chọn một hệ và giữ nhất quán.

### `names.mixed`

- Tra glossary trước. Họ Hán thuần 2–3 chữ (họ trong Bách gia tính) → Hán-Việt như `names.han`. Chuỗi 3 chữ trở lên có ký tự phiên âm đặc trưng (`尔 斯 克 姆 特 娜 丽 德 洛 布 罗 伊 森`) hoặc địa danh ngoài Trung Quốc → dạng gốc như `names.foreign`.
- Quyết định lần đầu được ghi vào glossary tự thêm để chương sau theo đúng.

## Check rules

Mỗi rule mặc định thêm `setting: "ancient" | "modern" | null` (null = trung lập). Lựa chọn: `neutral` + rule có `setting` khớp truyện. `checkRules` riêng vẫn thay toàn bộ; rule CJK cứng như cũ. Trục `names` không có rule regex.

- Chuyển sang `ancient`: vợ/chồng, anh ấy/anh ta/cô ấy/chị ấy, mình/tôi, Hừm/Ừm, `Ơ?`, bom khói, dao→đao, kho tàng/kho báu→bảo khố, Miêu Ảnh Vô Tông, một tấc vuông, thê tử danh nghĩa.
- Thêm `modern`: `ngươi`, `nàng`, `bọn ta`, thê tử / phu quân / lang quân / phụ thân / mẫu thân, "tổng tài", "nói đạo", cụm đảo ngữ `X Tông công pháp`.
- Còn lại trung lập (dấu câu Trung, từ nối Anh, `…`, chính tả, các bẫy convert từ Hán-Việt).

Chữ ký hàm đổi: web `checkAiTranslationViolations(text, configuredRules?, setting = "ancient")`, Rust `check_violations(text, configured, setting)`; `default_rules_as_check_rules(setting)` và `DEFAULT_AI_CHECK_RULES` thành hàm theo setting. Người gọi: `commands/check.rs`, `api_session.rs`, GUI `story_cmds.rs`, qt-web dialog và vòng dịch.

## Giao diện

- **qt-web** `ai-story-config-dialog.tsx`, tab Thông tin: hai Select "Bối cảnh" (Cổ đại / Hiện đại) và "Tên riêng" (Hán-Việt / Gốc nước ngoài / Hỗn hợp), có mô tả một dòng. Tab Prompt hiện bản đã ghép theo genre khi `customPrompt` rỗng; tab Rules liệt kê rule mặc định theo setting. AI fill nhận thêm `genre` trong JSON trả về, normalize, user duyệt.
- **qt-ai-gui** trang Hồ sơ: khối "Thể loại" mới ngay dưới Thông tin, hai Select, có mục lục neo; form dirty/lưu như các field khác. Command trả `base_prompt` / `check_rules` cho trang Hồ sơ nhận genre để hiện đúng bản mặc định. Không đổi gì ở trang Dịch: Rust ghép theo `story.json` cho cả agy lẫn API.
- **qt-ai-cli / workflow** `/setup-story`: bước tra web điền `genre` (đô thị → modern, nhân vật Âu Mỹ → foreign) rồi trình user duyệt cùng `story.json`. `init` ghi genre mặc định.

## Golden và kiểm thử

- `gen-golden.ts` xuất `prompts.json` dạng `{ core, settings: { ancient, modern }, names: { han, foreign, mixed }, suffix }`. Fixture `prompt.json` thêm ca cho 6 tổ hợp genre và ca customPrompt với genre khác mặc định; `check.json` thêm `defaultRules` theo từng setting và ca hiện đại (`vợ` không vi phạm, `ngươi` vi phạm), ca file cũ không có `genre`.
- qt-web vitest: `composeBasePrompt(genre)` ghép đúng thứ tự; tổ hợp `ancient/han` bằng đúng chuỗi `NOVEL_TRANSLATOR_BASE_PROMPT` cũ (giữ hằng cũ trong test làm mốc, sau đó xoá); `normalizeAiStoryConfig` thiếu genre; `checkAiTranslationViolations` theo setting; dialog test hai Select.
- Rust `cargo test -p qt-ai-core`: golden đọc fixture mới; `story.rs` normalize thiếu/sai genre; `check.rs` chọn rule theo setting.
- GUI: `schema.ts` genre mặc định; story-page test hai Select đổi và lưu; `cargo test -p qt-ai-gui` command trả prompt theo genre.
- Trước PR: `npm --prefix apps/qt-ai-cli run -s golden:check` xanh.
