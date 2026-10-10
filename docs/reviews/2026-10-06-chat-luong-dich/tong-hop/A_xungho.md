# Nhóm A — Xưng hô, vai vế, đại từ, bối cảnh

## 1. Tổng quan

- **194 lỗi / 60 truyện**: cao 63, vừa 92, thấp 39. Có 138 lỗi hệ thống (HT) và 56 lỗi lẻ.
- **Theo thẻ**:
  - XUNG_HO_TROI: 60 lỗi / 43 truyện
  - XUNG_HO_VAI_VE: 38 / 35
  - CAST_MAU_THUAN: 32 / 32
  - DAI_TU_MO_HO: 22 / 21
  - BOI_CANH_SAI: 18 / 14
  - NGUOI_CHUNG: 12 / 12
  - XUNG_HO_NGUOI_KE: 12 / 12
- **Số liệu tự đếm lại** trên 64 truyện trong `bookshelf/books/common`:
  - 13.409 cặp `glossary.addressing` trong story.json. Truyện cổ đại có 8.625 cặp, trong đó 1.819 cặp (21%) là `ta–ngươi`.
  - Chỉ 32/64 truyện có cast.json, và chỉ 23 truyện có ít nhất một mốc đổi xưng hô (tổng 291 mốc).
  - Trong 60 truyện bị dính: 29 truyện không có cast.json, 9 truyện có cast.json nhưng 0 mốc. Vậy **38/60 truyện chưa từng ghi được lần đổi xưng hô nào**.
  - Cast.json có từ thiết kế ngày 2026-10-01 (`docs/superpowers/specs/2026-10-01-qt-ai-addressing-system-design.md`). Phần lớn truyện không có cast.json đã dịch xong trước mốc đó (theo mtime của out/).

**Đính chính một nhận định lặp lại trong nhiều báo cáo:** "story.json và cast.json cùng nạp vào prompt nên mâu thuẫn nhau" là **sai với code hiện tại**.
- `build_chapter_prompt` xoá nhóm addressing khỏi glossary (`crates/qt-ai-core/src/prompt.rs:223`). Prompt chỉ in cặp hiệu lực do `effective_addressing` tính, trong đó mốc ở cast thắng (`src/cast.rs:226-245`).
- Kiểm chứng: `the-gioi-tu-tien…/work/0146_….prompt.md:608` in `徐建明→韩雨: ta–Hàn Thái thượng` (giá trị từ cast), không in `ta–ngươi` (giá trị cũ trong story.json).
- Có 262 mục story.json đã bị cast ghi đè. Chúng chỉ còn là dữ liệu cũ gây rối khi người dùng đọc file hoặc xem trong GUI.
- Mâu thuẫn **thật sự có trong prompt** là giữa `style.tone_rules` (luôn được in, `prompt.rs:169-188`) và cặp addressing.

---

## 2. Vấn đề hệ thống

### VĐ1 — Xưng hô bị đóng băng ở lần gặp đầu, mốc đổi gần như không bao giờ được ghi (P0)

**Phạm vi:** 43 lỗi XUNG_HO_TROI về cặp thoại cộng 32 lỗi CAST_MAU_THUAN, tổng **47 truyện**.

**Biểu hiện:**
- Cặp đã yêu nhau hoặc cưới nhau mà vẫn `tôi–cô`, `ta–ngươi`.
- Đã bái sư, nhận chủ hoặc kết nghĩa mà vẫn giữ cặp cũ.
- Xưng hô nhảy qua lại giữa các chương.

**Ví dụ:**
- di-lam-mo-ca 1251:83 「宁宁，你听我解释……」 → "Ninh Ninh, cô nghe tôi giải thích" ⇒ "Ninh Ninh, em nghe anh giải thích". Cảnh trên giường sau khi đã cưới. Mẫu "Ninh Ninh, cô" xuất hiện 155 lần / 125 file, còn "Ninh Ninh, em" chỉ 2 lần.
- cuc-pham-pho-ma 0159:9 「薛郎…你初涉官场」 → "Tiết lang… Ngươi mới bước chân…" ⇒ "Chàng mới…". Lỗi này có 20 lần / 7 file, xen kẽ khoảng 15 file đã dùng "chàng". Story.json có 655 cặp nhưng không cặp nào có mốc.
- ac-nu 1030:223 「师父，我无事」 → "Sư phụ, ta không sao" ⇒ "Sư phụ, con không sao". Các chương 1018–1029 đều xưng "con", nhưng glossary ghi `陈钰→朱媺娖: ta–sư phụ`.

**Nguyên nhân gốc (đã xác minh trong code):**
1. **Story.json chỉ thêm khoá mới, không bao giờ sửa khoá đã có.**
   - `append_auto_glossary` (`src/glossary.rs:139-160`) chỉ ghi khoá mới.
   - `sanitize_extracted` (`glossary.rs:80`) loại bỏ khoá đã tồn tại.
   - Hệ quả: giá trị trong story.json là ảnh chụp lúc hai người gặp nhau lần đầu.
2. **Đường duy nhất để đổi xưng hô là `apply_address_changes` (`cast.rs:656-707`), và đường này rất hẹp.**
   - Model phải tự khai lại cặp, kèm `note`.
   - Không được quay về giá trị liền trước (`cast.rs:691-692`). Vì thế không biểu diễn được quan hệ có hai chế độ như công khai/riêng tư hay trở mặt rồi làm lành.
3. **Harness chỉ tự phát hiện cặp cần xét lại trong trường hợp rất hẹp.**
   - `pairs_to_review` (`cast.rs:354-377`) chỉ bật khi raw có dấu hiệu tình cảm (`INTIMACY_MARKERS`, `cast.rs:328-331`).
   - Đồng thời cặp phải bị coi là "xa lạ" theo `is_distant` (`cast.rs:345-349`), tức chỉ khi tự xưng `tôi` hoặc `ta` + `ngươi/cô nương/công tử…`.
   - Bỏ sót hoàn toàn: bái sư (ac-nu, cai-nay-ngo-khong), nhận chủ/thần phục (hop-hoan, than-dieu), kết nghĩa sư tỷ–đệ (ma-mon), lộ giới tính (livestream).
   - Bỏ sót các cặp `tớ–cậu`, `tao–mày`, `bổn cung–ngươi`, `lão phu–ngươi`.
4. **Câu lệnh trong prompt ép model giữ cặp cũ.**
   - Hiện tại: "Giữ đúng các cặp này" (`cast.rs:411`).
   - Bản cũ, áp cho mọi truyện dịch trước 10/2026: "đã chốt ở chương trước, giữ y hệt" (`prompt.rs:145`).
   - Chính comment trong `cast.rs:351-352` thừa nhận model coi danh sách cặp là lệnh phải giữ.
5. **Cặp chỉ được học khi cả hai bên là tên trong `names`** (`cast.rs:644`, `glossary.rs:29`) **và tên đầy đủ có trong raw.**
   - Vì vậy hệ thống, chó 大黄, cặp chỉ gọi nhau bằng biệt danh như 老刘 (Trương Chí Quốc–Lưu Khánh) và 祂 không bao giờ có cặp.
   - Mỗi chương model lại chọn lại từ đầu (sao-the: 5 kiểu Giang Nịnh→hệ thống).
6. **Truyện có mốc thì mốc chạy đúng.** Các báo cáo khen: tan-the-phuc-hung (56 mốc), ta-cung-khong-muon (28 mốc), xuong-nui ("cô nương" trước ch.79, "nàng" sau). Như vậy cơ chế timeline đúng; vấn đề là nó hiếm khi được kích hoạt.

**Đề xuất sửa:**
- **(a) P0, nhỏ — Cho model thấy tuổi của cặp và lý do được đổi.**
  - Trong `chapter_addressing_section`, in kèm mốc: `甲→乙: X–Y (từ ch.N)` (lấy từ `from`; với giá trị gốc thì lấy chương trong `autoGlossaryLog`).
  - Thay câu "Giữ đúng các cặp này. Chỉ đổi khi…" bằng: *"Đây là cặp gần nhất đã ghi (từ chương N). Nếu thoại trong chương gọi nhau bằng danh xưng khác hẳn cặp này (师父/师尊, 主人, 公子, 夫君/相公/老公/老婆, 姐/哥, 宝贝…) hoặc quan hệ đã đổi, hãy dịch theo quan hệ hiện tại, rồi khai lại cả hai chiều kèm `note`."*
- **(b) P0, vừa — Mở rộng điều kiện tự phát hiện.**
  - Thêm `RELATION_MARKERS` (拜师, 师父, 师尊, 徒儿, 主人, 认主, 臣服, 奴婢, 结拜, 义兄/义妹, 夫君, 相公, 老公, 老婆) bên cạnh `INTIMACY_MARKERS`.
  - Thay `is_distant` bằng `!is_current_relation(target, marker)`. Ví dụ: marker hôn nhân/tình cảm mà cặp không thuộc {anh–em, chị–em, ta–nàng, thiếp–chàng, ông xã/bà xã…} thì đưa vào xét lại; tương tự marker bái sư mà cặp chưa có sư phụ/đồ nhi/con.
- **(c) P1, vừa — Theo dõi trôi xưng hô.**
  - Đuôi lượt dịch (`GLOSSARY_INLINE_INSTRUCTION`, `api_session.rs:50-62`; `commands/next.rs:27-28`) yêu cầu thêm: liệt kê **cặp thực dùng** cho mọi cặp đang có trong mục "Cặp xưng hô đang dùng".
  - Lúc accept: nếu khác cặp hiệu lực ở hai chương liên tiếp mà không có note, ghi mốc `source: auto, pending: true` và báo lên GUI.
  - Bỏ chặn "không quay về giá trị trước" khi có `note`.
- **(d) P1, lớn — Bù timeline cho 29 truyện không có cast.json.**
  - Lệnh `cast backfill-addressing`: lấy khoảng 20 cặp xuất hiện cùng nhau nhiều nhất, trích thoại của cặp từ out/ theo từng khoảng chương, hỏi model ra các mốc, ghi vào cast.json dưới dạng chờ duyệt.
  - Nên làm trước cho các truyện đang dịch tiếp: di-lam-mo-ca, nhuong-nguoi, cuc-pham, thau-huong, tien-tu-lai-xu-long, da-tu-da-phuc, tuyet-doi-rung-dong.
- **(e) P2, nhỏ — Dọn dữ liệu và bổ sung kiểm tra.**
  - GUI đánh dấu mục story.json đã bị cast ghi đè (262 mục).
  - Lint mâu thuẫn giữa `tone_rules` và cặp addressing (am-thinh, rut-dao, ma-mon, ta-tuyet-the, saint-seiya): tên hai bên xuất hiện trong cùng một tone_rule mà thuật ngữ khác → cảnh báo.
  - Kiểm tra hai chiều trong `filter_new_pairs`: A→B tự xưng anh/chị/em thì B→A không được tự xưng `tôi`. Đếm thô hiện có 158 cặp lệch kiểu này ở 21 truyện; sau-khi-chia-tay riêng 8 cặp.
  - Gộp alias tên (周正青/周青, 童百雄/童百熊).

### VĐ2 — Mặc định cổ đại `ta–ngươi` và luật check ép `ta` khiến sai vai vế (P0)

**Phạm vi:** 38 lỗi VAI_VE / 35 truyện, cộng một phần lỗi TROI.

**Ví dụ:**
- thieu-nien 0002:53 「少爷，我应该跟您说过」 → "Thiếu gia, ta nhớ là đã từng nói với ngài" ⇒ "…tôi/nô tỳ…". Câu mở bằng "Thiếu gia/Chủ nhân/Đại nhân" mà tự xưng "ta" có 64 lần / 40 file.
- moi-ngay 0001:137 tạp dịch nói với quản sự: "Vừa rồi ta thấy hắn" ⇒ "tiểu nhân". Truyện này có 191/405 cặp addressing là ta–ngươi.
- rut-dao 0551:173 「我儿，你回来了」 → "Nhi tử của ta, ngươi đã về rồi" ⇒ "Con trai ta, con về rồi đấy à".
- cuc-pham 1141:53: 你们 → "giữa hai người các ngươi, chàng sẽ…".

**Nguyên nhân gốc:**
- **Prompt cổ đại** (nguồn: `apps/qt-web/src/lib/ai-translation-prompt.ts`; `prompts.json` được sinh qua `apps/qt-ai-cli/scripts/gen-golden.ts`):
  - `:335`: 我 trong lời kể **và lời thoại** dùng `ta`.
  - `:346`: 你→`ngươi`, cấm "ông, bà".
  - `:350`: 你们→`các ngươi`.
  - `:355`: "你 nói với người già vẫn là ngươi theo vai vế".
  - Không có câu nào về cách bề dưới tự xưng.
- **Luật check cổ đại** `check.rs:68-72` (bản gốc ở `ai-translation.ts:49`) bắt lỗi `tôi/mình` ở đầu thoại, gợi ý "→ ta".
  - Vòng soát `review()` (`api_session.rs:316-361`) đưa vi phạm này cho model sửa, và chỉ nhận bản soát nếu số vi phạm giảm.
  - Nghi đây là chỗ biến "tôi/nô tỳ" thành "ta".
- **Auto-glossary học lại đúng lỗi của model rồi đóng băng** (VĐ1). Vòng lặp tự củng cố: 21% cặp cổ đại là ta–ngươi.

**Đề xuất sửa:**
- **P0, nhỏ — Thêm khối "Vai vế trong thoại" ngay sau bảng cổ đại (`ai-translation-prompt.ts` sau `:357`):**

  > **Vai vế trong thoại:** `ta–ngươi` chỉ dùng giữa người ngang hàng xa lạ/đối địch hoặc bề trên nói với bề dưới. Bề dưới nói với bề trên (raw có 您, hoặc gọi 少爷/主人/师父/大人/公子/殿下/前辈, hoặc người nói là nô tỳ, thuộc hạ, đệ tử, linh thú) thì tự xưng theo thân phận `tiểu nhân / nô tỳ / thuộc hạ / đệ tử / vãn bối / con`, gọi bằng danh xưng hoặc `ngài / người`; KHÔNG `ta`, KHÔNG `ngươi`. Cha mẹ, sư phụ nói với con, đồ đệ khi thân tình: `ta/cha/mẹ/vi sư – con/đồ nhi`. Tình nhân, phu thê đã thân: `ta–nàng`, `thiếp–chàng`. `你们` theo đúng ngôi hai người nói đang dùng (`hai người`, `các vị`, `các con`, `mọi người`); chỉ dịch `các ngươi` khi người nói gọi từng người là `ngươi`.

  - Sửa `:335`: "`我` trong lời kể dùng `ta`; trong thoại mặc định `ta` trừ khi vai vế ở mục dưới quy định khác".
  - Sửa `:355`: bỏ câu "vẫn là ngươi".
- **P1, nhỏ — Sửa thông điệp luật `check.rs:72`** thành "→ ta, hoặc tự xưng theo thân phận (tiểu nhân/nô tỳ/thuộc hạ/đệ tử/con)".
- **P1, nhỏ — Thêm luật check cổ đại:**
  - Regex: `[“"](?:Thiếu gia|Chủ nhân|Đại nhân|Sư phụ|Sư tôn|Công tử|Điện hạ|Bệ hạ|Tiền bối|Phụ thân|Mẫu thân|Cha|Mẹ|Nương)[,!][^”"]{0,120}(?<!\p{L})(?:ngươi|các ngươi)(?!\p{L})`
  - Thông điệp: "Gọi bề trên bằng danh xưng mà xưng ngươi".

### VĐ3 — Không lưu đại từ ngôi ba cho từng nhân vật, kể cả 祂 và nhân vật đổi giới (P1)

**Phạm vi:** 18 lỗi / 16 truyện, kéo theo một phần DAI_TU_MO_HO.

**Ví dụ:**
- huyen-giam 0079:5: 祂 → "hắn" ⇒ "Ngài". Ở ch.47–78, 祂 được dịch "Ngài" khoảng 130 lần / 18 file; ch.79–80 có 34 chữ 祂 mà 0 lần "Ngài". Sau đó 0080:127 có bốn chữ "hắn" chỉ hai người.
- tien-tu-ta-that 0067:3: 她 (Tề Hoàng) → "hắn" ⇒ "nàng". Sai ở 170 dòng / 41 chương.
- thieu-nien 0072:251: Lore lúc "hắn" lúc "cậu"; 78 chương dùng "cậu".
- Cùng kiểu: Hùng Bá hắn/lão/gã/ông; Gaia bà/nàng; Hứa Du hắn/nó.

**Nguyên nhân gốc:**
- `Character` chỉ có `gender`, `source`, `chapter`, `disputed` (`cast.rs:48-61`). Danh sách "Nhân vật có mặt" chỉ in giới (`cast.rs:398-401`).
- Giới tính là một giá trị duy nhất, không có mốc theo chương. Truyện đổi giới (Kronos từ ch.60, 江夜 từ ch.30) không biểu diễn được.
- Prompt cổ đại để model tự chọn: `他`→`hắn` "hoặc `lão`", `她`→`nàng` "hoặc `bà`" (`ts:355`). Không có luật nào cho 祂/它 (đã grep, không có).

**Đề xuất sửa:**
- **P1, vừa — Mở rộng cast.json** với `Character.narration: [{from, pronoun}]` và `genderChanges: [{from, gender}]`.
  - Đường học: model khai `{"source":"徐建明","category":"names","narration":"lão"}` ở lần đầu, hoặc khi đổi (kèm note).
  - In vào prompt: `- 徐建明 (Từ Kiến Minh): nam; lời kể gọi "lão"`.
- **P1, nhỏ — Thêm luật vào base** (cả ba bối cảnh): *"`祂` (thần linh, tồn tại siêu phàm) dịch một đại từ cố định do story/cast quy định, mặc định `Ngài`; không trộn với `hắn` của người phàm trong cùng đoạn. `它` chỉ thú có linh trí đã nhân cách hóa thì theo đại từ trong danh sách nhân vật."*
- **P2, nhỏ — Thêm luật vào mục "Lời kể gián tiếp":** *"Hai nhân vật cùng giới trong cùng câu hoặc hai câu liền nhau mà đổi chủ thể thì nêu tên hoặc danh xưng, không lặp `hắn`/`nàng`; `y` chỉ dành cho một người trong cảnh."* Luật này nhắm DAI_TU_MO_HO, ví dụ bat-dau-ho-tong 0312:63, moi-ngay 0520:35, truong-sinh-la-meo 0268:171.

### VĐ4 — `ta` của người kể và từ thân tộc lọt sai chỗ: độc thoại, 【】, đạn mạc, lời kể (P0 cho phần check, P1 cho prompt)

**Phạm vi:** 12 lỗi NGUOI_KE / 12 truyện, cộng đạn mạc (trong-sinh-thanh-meo, livestream).

**Ví dụ:**
- da-tu-da-phuc 0251:191 【…我本已经打算做他的女人】 → "ta vốn đã định…" ⇒ "mình". "ta" trong 【】 có 177 lần / 42 file.
- trong-sinh-thanh-meo 0011:71, bình luận cư dân mạng: "Bây giờ ta đi đầu thai làm mèo" ⇒ "Giờ tui…". Gặp ở 396 dòng / 85 file.
- bien-than-nu-dao-si 0091:5: 她 → "Muội bốc lại một nắm" ⇒ "Cô bé…". Khoảng 200 lần / 39 file.

**Nguyên nhân gốc:**
- **Đã xác minh (chắc chắn nhất):** luật check hiện đại `check.rs:132` (bản gốc `apps/qt-web/src/lib/ai-translation.ts:113`), regex `^[^"“”]*tôi[^"“”]*$`.
  - Mọi dòng **không có dấu ngoặc kép** mà chứa "tôi" đều bị báo "→ ta": 【】, tin nhắn, đạn mạc dạng `ID: …`, lời tác giả, độc thoại tự do.
  - Vòng soát `api_session.rs:316-361` ép sửa và chỉ nhận bản soát nếu số vi phạm giảm, nên bản dùng "ta" thắng một cách máy móc.
  - Luật này áp cho mọi truyện hiện đại, kể cả truyện kể ngôi ba.
- `addressing_rules` (`cast.rs:313-320`) đặt mặc định "mình" cho độc thoại, nhưng chỉ khi "đặt trong ngoặc kép", nên 【】 không được phủ.
- Luật "danh xưng chỉ sống trong ngoặc kép" chỉ có ở bảng hiện đại (`ts:567`). Bảng cổ đại và phần cổ đại của mixed không có. Thêm vào đó, tone_rules ghi "gọi … đệ, muội" thì model áp luôn vào lời kể.

**Đề xuất sửa:**
- **P0, nhỏ — Thêm trường `genre.pov` (`first|third`) vào story.json.**
  - Fill suy ra từ `voice` (`api_fill.rs:61`).
  - Luật `check.rs:132` cùng các câu "người kể ngôi một tự xưng ta" trong prompt chỉ bật khi `pov=first`.
  - Trước mắt: sửa regex để bỏ qua dòng bắt đầu bằng `【`, `[`, và dòng dạng `^\S{1,30}[:：]`.
  - Vẫn giữ sở thích của chủ app: người kể ngôi một luôn "ta".
- **P1, nhỏ — Thêm luật vào mục `# Xưng hô`** (`addressing_rules`, áp mọi setting):
  - *"Ý nghĩ trong 【】, chữ nghiêng hoặc độc thoại tự do theo chung luật độc thoại."*
  - *"Bình luận, đạn mạc, tin nhắn của cư dân mạng hiện đại: tự xưng `tôi/tui/mình`, gọi idol `anh ấy/ảnh/cậu ấy`; không `ta`, không `hắn`."*
  - *"Lời kể ngôi ba không dùng từ thân tộc hay kính xưng (muội, đệ, tỷ, huynh, anh, ngài, người) làm đại từ."*

### VĐ5 — Một truyện chỉ có một `setting`; truyện nhiều arc và phần ngoài truyện bị khoá sai (P1)

**Phạm vi:** 7 truyện.

**Ví dụ:**
- xuyen-nhanh-tuyet-sac (ancient), arc hào môn ch.263–289:
  - "nàng" khoảng 484 lần / 23 file, rồi về 0 từ ch.281.
  - "tỷ" 248 lần / 28 file.
  - "Giang tiểu thư, ta cũng có thể cho cổ phần" (0281:131).
  - Có ≥6 cặp addressing kiểu cổ trang ("ta–Tần di").
- lao-ba-qua-manh: truyện đô thị nhưng gắn `ancient`. Có 0 lần "vợ/chồng" trong 636 file, "thê tử" 360 lần; 「老公，你回来啦」 → "Phu quân, anh về rồi à".
- ma-mon ch.1 (prologue hiện đại): "ta thấy khá thích ngươi, chúng ta kết hôn đi".
- Cùng loại:
  - xuyen-nhanh-phao-hoi: "lão sư", "thê tử" trong arc năm 1968 và thập niên 80.
  - cuc-pham 1142 (lời bạt) và su-ton 0071–0072 (lời tác giả): "cảm ơn các ngươi".
  - huyen-giam: không có `genre`, nên rơi về mặc định Ancient (`story.rs:34`).

**Nguyên nhân gốc:**
- `StoryGenre.setting` chỉ có một giá trị (`story.rs:33-37`), và được dùng cho toàn bộ truyện:
  - chọn base (`prompt.rs:195-196`)
  - chọn luật xưng hô (`prompt.rs:221`)
  - chọn luật check (`check.rs:138`)
- Fill chỉ đọc 3 chương đầu, mỗi chương 4.000 ký tự (`api_fill.rs:14-16`), nên không thấy các arc về sau.
- Luật check cổ đại `check.rs:68` cấm "vợ/chồng", và vòng soát lại ép điều đó vào.
- Bối cảnh `mixed` tự chọn bộ xưng hô theo cảnh (`ts:653-686`) nhưng chỉ 3/64 truyện dùng.

**Đề xuất sửa:**
- **P0, nhỏ — Sửa dữ liệu ngay:**
  - xuyen-nhanh ×2 → `mixed`
  - lao-ba → `modern`
  - huyen-giam → điền `genre`
- **P1, vừa — Thêm `genre.arcs: [{from, setting, tone?}]`** và hàm `effective_genre(story, chapter_id)` (giống `effective_addressing`), dùng ở `build_chapter_prompt`, `check_violations` và `BaseStore::glossary`. Cache prompt chỉ vỡ ở ranh giới arc.
- **P1, nhỏ — Văn bản ngoài truyện** (`ts:163`): thêm *"Lời tác giả nói với độc giả luôn là văn hiện đại: tác giả xưng `tôi`, gọi độc giả `các bạn/mọi người`, kể cả truyện cổ đại."* Đồng thời bỏ luật check cổ đại cho chương có tiêu đề khớp `感言|写在最后|后记|请假|完本`.
- **P2, nhỏ — Fill:** lấy mẫu thêm chương giữa và chương cuối; nhắc "xuyên nhanh nhiều thế giới → mixed".

### VĐ6 — 你 phiếm chỉ trong lời kể bị dịch thành `ngươi` (P1, rẻ)

**Phạm vi:** 12 lỗi / 12 truyện, đều cổ đại. Số đếm theo báo cáo:
- truong-sinh-bat-dau: 299 "ngươi" ngoài ngoặc kép / 61 file
- hop-hoan: 133 dòng / 109 file (chưa tách độc thoại hợp lệ)
- the-gioi: khoảng 45 dòng / 37 file
- pham-nhan: khoảng 35 đoạn / 27 file

**Ví dụ:**
- pham-nhan 0018:27 「当你弱小时…」 → "khi ngươi nhỏ yếu" ⇒ "khi người ta yếu".
- xuong-nui 0054:131 「当一个人爱你的时候」 → "yêu ngươi" ⇒ "yêu mình".

**Nguyên nhân gốc:** Bảng cổ đại ánh xạ 你→ngươi không có ngoại lệ (`ts:346`); không có luật nào về đại từ phiếm chỉ (đã grep).

**Đề xuất sửa:**
- Thêm dưới bảng: *"`你` ngoài ngoặc kép không chỉ một nhân vật cụ thể (câu triết lý, ví von, người kể nói với độc giả: 当你…时, 你以为…, 你说…我都信) → `người ta`, `mình`, `ai`, hoặc lược chủ ngữ; KHÔNG `ngươi`."*
- Luật check cổ đại: `^[^"“”]*(?<!\p{L})(?:khi|nếu|dù|chỉ cần|tưởng)\s+ngươi(?!\p{L})` → "你 phiếm chỉ → người ta/mình".

---

## 3. Lỗi lẻ đáng chú ý

- **cai-nay-ngo-khong 0080:65:** raw 「悟空/大师兄！」 dùng "/" để gộp lời hai người, bản dịch chép "/" vào cả đại từ ("cậu/huynh"). Cần luật: tách thành hai lời gọi.
- **dau-la 0052:59:** 父亲→"Ba". Cần chốt 爸爸=ba, 父亲=cha trong addressing.
- **Glossary tự sinh chép từ cảnh ngoại lệ** (cãi nhau hoặc trở mặt) rồi khoá luôn: tuyet-doi-rung-dong 书女士→黎书宇 "tao–mày" làm mẹ–con thành giọng chợ búa; ta-tuyet-the "tao–mày" lấy từ cảnh túm tóc chương 1.
- **Mục addressing tự mâu thuẫn:** dinh-luu-van 宋巧云→宋父 "chị–anh"; lao-ba có ≥4 cặp lệch chiều.
- **hop-hoan 0001:145:** raw bỏ 叔 chuyển sang 你 khi trở mặt, nhưng bản dịch vẫn "thúc–cháu" theo glossary. Prompt nên nói: raw đổi cách gọi trong cảnh thì theo raw.
- **trong-sinh-thanh-cau:** signature_phrase chứa "ngươi" trong truyện hiện đại. Lint signature_phrases theo setting.
- **ta-duong-thanh:** addressing ghi sai tên 徐徽绛 (đúng là 张徽绛); 14/14 cặp chỉ thuộc một arc.
- **Gọi tôn kính bằng "người" ở ngôi ba** (cuc-pham 1140, xuyen-qua 0034:77) dễ đọc thành "người ta"; trong câu bị động nên nêu tên.

## 4. Điều đang làm tốt (đừng sửa nhầm)

- **Truyện có mốc trong cast.json thì xưng hô chạy đúng mốc:** tan-the-phuc-hung ("Chồng à… anh"), tan-the-nguoi-khac, ta-tai-tan-the, saint-seiya (thiếp–chàng), truong-sinh-bat-dau, xuong-nui ("cô nương"→"nàng" từ ch.79), am-thinh. Giữ nguyên `effective_addressing` và cách cast thắng story.json.
- **Prompt chương chỉ in cặp hiệu lực**, đã kiểm chứng ở the-gioi-tu-tien 0146. Đừng quay lại nạp story.json song song.
- **Model bám raw đúng khi glossary sai:** cuu-thuc ("Lâm đạo hữu"), bai-poker ("Diệp đại ca"), ma-mon (chàng 193 / thiếp 221 lần dù addressing ghi ta–ngươi), xuyen-qua-chu-thien (hắn→nàng theo 他/她).
- **Xưng hô cung đình, sư môn, hậu bối–tiền bối phần lớn chuẩn:** bản cung, trẫm–nàng, nô tài, vãn bối–tiền bối, 你→"ngài" với người trên (bat-dau-ho-tong, tuyet-pham, phuong-thon).
- **Luật người kể ngôi một "ta" là sở thích của chủ app.** Chỉ thu hẹp phạm vi áp dụng (pov, 【】, đạn mạc), không bỏ luật.
