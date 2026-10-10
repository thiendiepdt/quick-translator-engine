# Chương B — Glossary và thuật ngữ

## 1. Tổng quan

- **215 lỗi**, rải ở **cả 61/61 truyện**. 197 lỗi là lỗi hệ thống, 18 lỗi lẻ. Theo mức: cao 48, vừa 97, thấp 70.
- Theo thẻ: THUAT_NGU_TROI 93 (54 truyện), GLOSSARY_PHIEN_AM 48 (32), GLOSSARY_HAN_VIET_NANG 29 (27), GLOSSARY_TRUNG 28 (26), GLOSSARY_DANH_XUNG 17 (17).
- Tự đếm trên `bookshelf/books/common/*/story.json`:
  - Toàn thư viện có **54.707 mục glossary**, trong đó `autoGlossaryLog` ghi **51.721 mục tự sinh** (≈95%).
  - Mục tự sinh chia theo nhóm: names 16.225, addressing 13.257, places 8.520, items 6.233, skills 5.874, creatures 1.515, **common 93 (0,18%)**.
  - Glossary hầu như chỉ là tên riêng. Thuật ngữ chung của truyện gần như không bao giờ được chốt.
- Lượng glossary nạp vào prompt mỗi chương không lớn. Đo ở 20 chương mới nhất của 5 truyện, trung vị là 9–40 mục (thau-huong tối đa 156). Vì vậy model bỏ qua glossary **không phải vì prompt quá tải**.

## 2. Các vấn đề hệ thống (xếp theo ảnh hưởng)

### B1. Auto-glossary khoá cứng âm Hán Việt sai, rồi ép sai cả truyện — 48 lỗi / 32 truyện (cao 15)

**Ví dụ**
- **ky-tuc-xa** ch0547: 童绮 → "Đồng **Khởi**" ⇒ "Đồng **Ỷ**". Lặp 1.602 lần trong 400 file.
- **cuu-thuc** ch0001: 煜儿 → "**Dực** nhi" ⇒ "**Dục** nhi". Lý Dực có 2.263 lần, sai ngay từ story.json lúc lập hồ sơ.
- **moi-ngay** ch0520: 佟仲秋 → "**Đông** Trọng Thu" ⇒ "Đồng Trọng Thu" (406 lần). Ở **thau-huong**, glossary ghi "Tiểu Đông hậu" (29 lần), còn model tự sửa thành "Tiểu Đồng hậu" 219 lần.
- Các ca khác: 祟→Toái, 蠹→Đỗ, 蛟→Kiều, 轩→Huyên, 棘→Cực, 鸩→Trầm, 储→Trừ, 冲→Trùng, 屿→Tự, 迦→Gia, 很→Hứa, 综→Tống, 全→Quan…

**Nguyên nhân gốc (đã thấy trong code/prompt)**
1. Prompt nền dạy sai. `apps/qt-web/src/lib/ai-translation-prompt.ts:719` (vào prompts.json cho mọi base) ghi **"佟 Đông"** trong bảng "Họ một âm nhưng hiếm, không tự bịa". 佟 đọc là Đồng (Đồng Tương Ngọc).
2. Auto-glossary học thẳng từ bản dịch. `glossary.rs:93` chỉ đòi target chép nguyên văn từ bản dịch; chương đầu phiên sai thì cái sai được chốt. Không có bước nào đối chiếu âm Hán Việt.
3. `drop_conflicting_readings` (`glossary.rs:120-136`) chỉ so với âm của các mục names **đã có**, nên âm đầu tiên luôn thắng, kể cả khi âm đó sai. Hàm cũng chỉ xét nhóm names có số tiếng bằng số chữ.
4. Lập hồ sơ (`api_fill.rs:72`, `antigravity/workflows/setup-story.md:35-36`) seed glossary mà không kiểm âm. Đây là gốc của ca 煜 = Dực.

**Đề xuất**
- **(P0, nhỏ)** Sửa `ai-translation-prompt.ts:719` thành `佟 Đồng`, rồi chạy `npm run golden` để sinh lại prompts.json.
- **(P0, vừa)** Thêm bộ kiểm âm Hán Việt:
  - Nhúng bảng âm theo chữ (khoảng 8–10k chữ, mỗi chữ một danh sách âm, lấy từ từ điển Hán Việt) vào `crates/qt-ai-core/data/hanviet.tsv`.
  - Viết `fn reading_mismatches(source, target) -> Vec<(char, String, Vec<String>)>`: khi số chữ bằng số tiếng, mỗi tiếng (đã viết thường) phải nằm trong danh sách âm của chữ tương ứng.
  - Gọi trong `sanitize_extracted`: lệch âm thì **không thêm**, ghi `work/<id>.glossary-rejects.json` để người dùng xem.
  - Cho một danh sách ngoại lệ quen dùng: 耶律 Da Luật, 万俟 Mặc Kỳ, 单 Đơn/Thiện, tên dịch nghĩa (傻姑 Ngốc Cô). Danh sách này nằm trong story.json `readingOverrides`.
- **(P1, nhỏ)** Thêm lệnh `qt-ai glossary-audit <root>` chạy cùng hàm trên cho glossary hiện có. Chỉ riêng các ca trong digest đã có ≥30 mục sai ở 32 truyện cần sửa tay.

### B2. Biến thể tên do tác giả gõ nhầm, mục trùng và glossary tự mâu thuẫn — 44 lỗi / 36 truyện (cao 12)

**Ví dụ**
- **bat-dau-ho-tong** ch0255: 陆停云/陆云停 thành hai mục: "Lục Đình Vân" 45 lần, "Lục Vân Đình" 64 lần.
- **ma-mon** ch0236: 玲儿 và 灵儿 đều ra "Linh Nhi". Hai nhân vật trùng tên Việt, nhiều cảnh có mặt cả hai.
- **nhuong-nguoi** ch0258: 督察局 = "Cục Giám Sát" nhưng 华国督察局 = "Cục Đốc Tra Hoa Quốc". Kết quả một tổ chức có ba tên (603/110/8 lần).
- Cùng kiểu: 赵锐/赵瑞, 秦红锦/秦红棉, 沉默→"Trầm Mặc" (là lỗi gõ của 陈默), 沙漠之鹰 lúc "Sa Mạc Chi Ưng" lúc "Desert Eagle", 《X》/X dịch khác nhau (20 cặp ở giai-tri).

**Nguyên nhân gốc**
- `sanitize_extracted` chỉ chống trùng **khoá** (`glossary.rs:90`). Không có kiểm tra nào xem:
  - target mới có trùng target của khoá khác không;
  - khoá mới chỉ khác khoá cũ một chữ, đảo chữ, khác phồn/giản hay chỉ thêm 《》;
  - khoá dài có chứa khoá ngắn mà target lại không chứa target ngắn (督察局 ⊂ 华国督察局).
- Schema `Glossary = IndexMap<String, StringMap>` (`story.rs:19-20`) không có chỗ cho alias, nên biến thể chỉ có thể thành một mục độc lập.
- Tự đếm: **321 nhóm** gồm các khoá khác nhau cùng một bản Việt, ở 42 truyện. Con số này gồm cả biến thể gộp đúng lẫn va chạm thật như 全昕/关欣 → Quan Hân.

**Đề xuất**
- **(P1, vừa)** Thêm field `glossaryAliases: {biến thể: khoá chuẩn}` vào story.json.
  - `filter_glossary_for_source` (`prompt.rs:117`): raw có biến thể thì nạp mục chuẩn kèm dòng `"陆停云 (biến thể của 陆云停)": "Lục Vân Đình"`.
  - Check coi biến thể là khoá chuẩn.
- **(P1, vừa)** Đặt cổng kiểm trùng trong `append_auto_glossary` (`glossary.rs:139`). Mục mới **không tự thêm** mà vào hàng chờ duyệt khi:
  - (a) target trùng target của khoá khác (so chữ thường, chuẩn NFC);
  - (b) cùng độ dài với một khoá có sẵn và chỉ khác 1 chữ, hoặc là hoán vị của khoá đó.
  - Nếu khoá mới xuất hiện ≤2 lần trong toàn bộ `raw/` trong khi khoá cũ xuất hiện ≥20 lần (dấu hiệu lỗi gõ), tự gộp thành alias. Nhờ vậy 李贤/李显 (hai người thật) không bị gộp nhầm.
- **(P1, nhỏ)** Thêm lint "đầu ngữ": khi khoá A là chuỗi con của khoá B, target(B) phải chứa target(A) (bỏ qua hoa/thường), lệch thì cảnh báo. Lint này bắt được 督察局, 十二长生/十二长生法, 武魂殿/武魂殿学院, 戴 (Đới/Đái), 弑神.
- **(P2, nhỏ)** Gộp khoá `《X》` với `X` khi lưu (bỏ 《》 khỏi khoá).

### B3. Thuật ngữ cốt lõi không bao giờ được chốt (面板, 好感度, 末世…) — 44 lỗi / 32 truyện (cao 10)

**Ví dụ**
- **moi-ngay** ch1044: 面板 lúc "bảng giao diện" (245 lần), lúc "bảng điều khiển" (303 lần). Glossary không có mục này, còn toneRules lại ghi "bảng điều khiển".
- **tam-muoi-tuoi** ch0406 và **saint-seiya** ch0090: 好感度 lúc "độ hảo cảm" lúc "độ thiện cảm" (80/131 dòng; 201/256 lần).
- **tan-the-phuc-hung** ch0001: 末世 ra "mạt thế" 627 lần và "tận thế" 144 lần, lẫn ngay trong chương 1.
- Cùng loại: 春晚 Xuân Vãn/Gala Tết, 丧尸 với 4 kiểu dịch, 截胡, 积分, 大比, 楼主, 成长体, 叮 Ting/Đinh, 检测到 "kiểm trắc".

**Nguyên nhân gốc**
- Cả ba lối trích glossary đều **cấm** thuật ngữ chung:
  - `api_session.rs:35-36` và `:53-54`: category chỉ có names/places/items/creatures/skills, kèm "Bỏ qua từ chung, chức danh, đại từ".
  - `commands/next.rs:22-30` (luồng agy): cùng chỉ dẫn.
  - Nguồn web: `apps/qt-web/src/lib/ai-client.ts:656-662`.
- Nhóm `common` chỉ được điền lúc lập hồ sơ, từ 3 chương đầu cắt còn 4.000 chữ (`api_fill.rs:14-16`). Kết quả là common chỉ chiếm 0,18% mục tự sinh.

**Đề xuất**
- **(P0, nhỏ)** Mở nhóm `common` trong cả 3 chỉ dẫn. Câu đề xuất:

  > "Ngoài tên riêng, khai category \"common\" cho THUẬT NGỮ RIÊNG CỦA TRUYỆN lặp lại: chỉ số/bảng/âm báo hệ thống (面板, 好感度, 积分, 叮, 检测到), khái niệm thế giới (末世, 丧尸, 大比, 成长体), biệt danh và danh xưng cố định của nhân vật (好师父, 夜叉小姐, 方方哥), tên chương trình/tổ chức đời thực (春晚). target chép đúng cách bạn vừa dịch. Không khai từ thường dùng một lần."

- **(P1, vừa)** Quét tần suất trong code khi `init`/fill:
  - Đếm n-gram 2–4 chữ trên toàn `raw/`; lấy cụm có tần suất tài liệu cao trong truyện mà không thuộc danh sách từ phổ thông (bảng ~5k từ nhúng sẵn).
  - Lấy top 30–50 cụm đưa vào prompt fill để model chốt bản dịch vào `common`.
  - Khi harvest, cụm xuất hiện ≥3 lần trong 2 chương liên tiếp mà chưa có mục thì ghi vào hàng chờ.
- **(P1, nhỏ)** Thêm lint story.json: target của glossary không được nằm trong `style.avoid`, và `style.toneRules` không được chốt từ khác cho cùng khoá. Lint này bắt được 善恶值 'thiện ác trị' và 面板.

### B4. Glossary đã chốt và đã nạp mà model vẫn dịch khác — 18 lỗi / 16 truyện (cao 8)

**Ví dụ**
- **thau-huong** ch1278: 万俟卨 → "Vạn Sĩ Tiết" (88 lần, dồn ở ch1274–1281), trong khi glossary ghi "Mặc Kỳ Tiết". Tự kiểm: raw các chương này có 万俟卨 đúng nguyên văn 69 lần, nghĩa là mục đã được nạp vào prompt.
- **tien-tu-lai-xu-long** ch0622: "Mặc Trúc **Sênh**" 74 lần so với glossary "Sanh". Tên nhân vật chính, gồm cả 4 chương mới nhất.
- **trong-sinh-thanh-meo** ch0228: glossary và cast ghim 乐哥 = "anh Lạc" mà vẫn ra "Lạc ca" 303 lần theo từng đợt chương.
- Cùng loại: 卫皎皎 Kiểu/Giảo, 天启 Khải/Khởi, 东海 → "Biển Đông" (glossary ghi Đông Hải), 摸鱼崽.

**Nguyên nhân gốc**
- Check chỉ bắt **lệch dấu thanh**. `check.rs:186-224` (`glossary_drifts`) bỏ dấu rồi so khớp, và chỉ xét target ≥2 tiếng. Sanh/Sênh, Giảo/Kiểu, Khởi/Khải, Vũ/Võ, Mặc Kỳ/Vạn Sĩ, anh Lạc/Lạc ca đều lọt.
- Hàm này mới thêm ngày 2026-09-30 (commit 855963e), nên phần lớn chương cũ chưa từng được kiểm. Toàn bộ state.json hiện có **0 cảnh báo "Tên lệch glossary"**.
- Có bắt được thì vẫn bỏ qua được:
  - Lời dặn soát cho model quyền "Nếu đúng trong ngữ cảnh, giữ nguyên" (`api_session.rs:321`).
  - Model giữ nguyên thì chương được chốt kèm cảnh báo (`api_session.rs:345-347`, `commands/check.rs:181-185`).
- Lời dặn trong prompt yếu: "Các mục này được ưu tiên và phải dùng nhất quán" (`prompt.rs:230`). Model "sửa giùm" âm hiếm (万俟 → Vạn Sĩ) — **nghi** do nó tin âm của nó hơn glossary.
- Bộ lọc nạp chỉ khớp nguyên văn (`prompt.rs:96-114`), nên còn sót thêm:
  - 云隐道观 không kéo được mục 云隐观;
  - 玄黄气 không kéo được 天地玄黄气.

**Đề xuất**
- **(P0, vừa)** Thêm check "thiếu target" mức chương: raw có khoá (≥2 chữ Hán, nhóm khác addressing) mà toàn chương không có target (so sau chuẩn NFC, không phân biệt hoa/thường) thì báo vi phạm. Không xét từng đoạn để tránh báo nhầm khi tên được thay bằng đại từ.
- **(P0, vừa)** Bổ sung so lệch mức đoạn: trong đoạn có khoá mà thiếu target, tìm cụm viết hoa có cùng số tiếng và trùng ≥n−1 tiếng (đã bỏ dấu) với target. Tìm thấy thì code **tự thay luôn** cụm đó bằng target, không cần đưa model soát.
- **(P0, nhỏ)** Vi phạm glossary không được để model "giữ nguyên vì đúng ngữ cảnh". Tách thành mục riêng trong lời dặn soát: "Tên/thuật ngữ BẮT BUỘC theo từ điển — thay đúng như sau: X → Y". Chưa sạch loại này thì không chốt kèm cảnh báo.
- **(P1, nhỏ)** Đổi tiêu đề mục glossary ở `prompt.rs:230` thành:

  > "Từ điển BẮT BUỘC: raw có khoá nào thì bản dịch phải chứa đúng target đó (chính tả, hoa/thường, trật tự từ). Không tự sửa âm kể cả khi bạn cho là sai; nghi sai thì vẫn dùng và khai trong khối [[glossary]] với \"dispute\": \"lý do\"."

  Kèm theo: log các `dispute` ra để người dùng sửa glossary, nhờ đó gom luôn được các ca B1.

### B5. Glossary học muộn, ghi đè cách dịch đúng trước đó — 9 lỗi / 9 truyện (cao 3)

**Ví dụ**
- **giai-tri** ch1024: 群星 ("nhiều ca sĩ") được thêm thành names "Quần Tinh" ở ch1023. Mất cách dịch đúng đã dùng ở 10 file trước.
- **cuu-thuc** ch0078: 清廷 xuất hiện từ ch27 nhưng đến ch137 mới chốt. Trước đó dịch theo đợt: nhà Thanh/Mãn Thanh/Thanh đình.
- **xuong-nui** ch0119: ch118 dịch "đèn cầu trời", ch119 auto chốt "đèn Khẩn Thiên". Cùng kiểu: 咆哮帝 "Gào Thét Đế" bị thay bằng "Bào Hạo Đế".

**Nguyên nhân gốc**: `sanitize_extracted` chỉ nhìn chương hiện tại (`glossary.rs:62-106`), không đối chiếu out/ cũ. Category lạ thì mặc định rơi về `names` (`glossary.rs:96-101`), nên từ chung bị đội lốt tên riêng.

**Đề xuất (P1, vừa)**
- Trước khi thêm mục, lấy ≤5 chương done gần nhất có khoá trong raw, căn theo đoạn, đếm cách dịch đang chiếm ưu thế. Nếu ≠ target mới và xuất hiện ≥2 lần thì giữ bản cũ hoặc đưa vào hàng chờ.
- Category lạ thì **bỏ mục**, không rơi về names.
- **(P2)** Thêm lệnh "rà ngược": khi một mục được chốt hoặc đổi, liệt kê các file out/ đang dùng biến thể và cho thay hàng loạt.

### B6. Danh xưng bị lưu như tên riêng — 19 lỗi / 18 truyện

**Ví dụ**
- **dinh-luu-van** ch0024: 老裴 → "**Lão** Bùi" viết hoa giữa câu (33 lần).
- **xuong-nui** ch0061: 李奶奶 → "Lý nãi nãi" ⇒ "bà Lý".
- **bat-dau-ho-tong**: 龙姑娘 và 龙青青 thành hai nhân vật trong cast.json.
- Cùng loại: "Hạ nữ sĩ", "Trương đạo", "Vi tỷ", "Bạch Lão Yêu".

**Nguyên nhân gốc**
- Chính bảng kính ngữ trong prompt dạy viết hoa: `ai-translation-prompt.ts:415` (cổ đại) `| 老X | Lão X |` và `:601` (hiện đại) `| 老X | Lão X giữa bạn bè …`.
- `:419` ép `X thúc / X di` cho mọi truyện ancient, kể cả các arc hiện đại của truyện xuyên nhanh (ra "Tần di", "Dương a di").
- Lời dặn "Bỏ qua chức danh" chỉ là lời dặn; sanitize không chặn được.
- Tự đếm thô bằng regex: **~667 mục names** có dạng họ + danh xưng (阿姨, 夫人, 姑娘, 哥, 姐, 老X…), trong đó 86 mục dịch là "Lão X" viết hoa. Các mục này còn rơi vào cast qua `learn_genders` (`commands/accept.rs:77-82`).

**Đề xuất**
- **(P0, nhỏ)** Đổi hai dòng bảng thành `lão X` (viết thường; hiện đại: "lão X giữa bạn bè / ông X, bác X khi kính trọng").
- **(P1, nhỏ)** Trong `sanitize_extracted`, loại khoá names khớp `^(老|小|阿)?\p{Han}{1,2}(姑娘|夫人|管家|管事|阿姨|姨|老师|大夫|捕头|长老|女士|先生|小姐|奶奶|爷爷|老头|哥|姐|少|导|总|董|经理)$|^老\p{Han}$`. Cho ngoại lệ khi khoá là biệt hiệu có trong tên truyện hoặc tóm tắt.
- **(P1, nhỏ)** Với setting modern, thêm vào chỉ dẫn trích: "Danh xưng đi kèm họ dịch theo trật tự Việt (cô Hạ, chủ tịch Hạ, đạo diễn Trương, nhà họ Vệ), không lưu vào từ điển."

### B7. Glossary và hồ sơ chốt Hán Việt nặng, trái style.avoid — 29 lỗi / 27 truyện

**Ví dụ**
- **bat-dau-ho-tong** ch0001: creatures 老牛 = "lão ngưu", lặp 213 lần.
- **livestream** ch0001: 纯爷们 = "thuần nam tử hán", 弹幕 = "đạn mạc" (218 lần), trong khi toneRules của truyện nói tránh.
- **tien-tu-ta-that** ch0635: 灵魂纽带 = "Linh Hồn Nữu Đái". Model tránh mục này và tự dịch 5 kiểu.

**Nguyên nhân gốc**: lập hồ sơ đòi "target Hán-Việt" (`setup-story.md:36`, `api_fill.rs:72`). Chỉ dẫn trích không phân biệt setting, cũng không phân biệt biệt danh/meme với tên thật.

**Đề xuất (P1, nhỏ)**
- Sửa câu trong fill và setup thành: "target theo bối cảnh: tên người/địa danh Trung Hán-Việt; biệt danh, meme, tên quà, chức danh và từ đời thường trong truyện hiện đại thì DỊCH NGHĨA (Tóc Vàng, Gà Mờ, đạo diễn Trương)."
- Thêm lint lúc ghi glossary cho truyện modern: target khớp `lão thái thái|tiên sinh|nữ sĩ|a di|nãi nãi|tiểu thư| chi ` thì đưa vào hàng chờ.

### B8. Trôi hoa/thường, trật tự từ và dạng Unicode (thấp, nhiều truyện)

**Ví dụ**: "Thần Nữ/Thần nữ/thần nữ", "Hoài Thủy huyện" lẫn "huyện Hoài Thủy", "Vệ gia" lẫn "nhà họ Vệ", "Hoả" 222 lần lẫn "Hỏa" 102 lần (tien-tu-lai).

**Nguyên nhân**: không có chuẩn hoá khi ghi glossary; check so khớp phân biệt hoa/thường và dạng dấu.

**Đề xuất (P2, nhỏ)**
- Chuẩn hoá vị trí dấu thanh (kiểu mới: hỏa, thủy) ở cả `append_auto_glossary` lẫn bản dịch lúc accept.
- Thêm lint "một trật tự" cho đơn vị hành chính/gia tộc trong từng truyện.

## 3. Lỗi lẻ đáng chú ý

- **tuyet-pham** ch0507: 东海 thành "**Biển Đông**", gây hiểu nhầm địa lý–chính trị với độc giả Việt. Nên đưa vào check cứng cho mọi truyện có glossary 东海.
- **cai-nay-ngo-khong** ch0083: auto-glossary nhặt cả tên người ủng hộ trong lời tác giả (与珏, 极光). Nên bỏ qua các dòng 感谢…打赏 khi trích.
- **ky-quai** story.json: addressing có mục đơn 老爸 = "bố già", vốn là từ lóng chỉ trùm mafia.
- **livestream**: 50/334 ID đạn mạc lặp có hơn một cách dịch (吃瓜不吐籽 có 9 kiểu). ID fan cần được chốt như tên riêng.
- **ky-tuc-xa**: tên 钱桐 "Tiền Đồng" trùng hệt đơn vị tiền "tiền đồng" (520 lần).

## 4. Điều đang làm tốt (đừng sửa nhầm)

- Tên nhân vật, môn phái, chiêu thức chính nhất quán qua hàng nghìn chương ở nhiều truyện: ac-nu (1.028 chương Kim Dung), moi-ngay, tan-the-phuc-hung, nam-nu-de (Hổ Phách Đao 32/32).
- cuc-pham-pho-ma gộp được biến thể lỗi gõ (司马承祯/桢/帧, 回纥/回鹘) về một tên.
- Luật "mục có sẵn luôn thắng, chỉ thêm khoá mới" (`glossary.rs:138-160`) giữ glossary ổn định. Không nên đổi thành cho ghi đè tự động.
- Bộ lọc nạp theo chương (`prompt.rs:117`) giữ prompt gọn (trung vị 9–40 mục) và cache được. Chỉ nên mở rộng cho alias, không nạp toàn bộ glossary.
- `drop_conflicting_readings` chặn đúng kiểu 段 Đoàn/Đoạn. Nên giữ, và bổ sung bộ kiểm âm từ điển (B1) để chặn cả trường hợp âm đầu tiên đã sai.
- Thuật ngữ LMHT, tên tướng gốc Anh, Tốc Biến, Bùa Đỏ… sát cộng đồng. Đây là bằng chứng chốt `common` sớm (B3) cho kết quả tốt.

**Thứ tự làm**
1. P0 nhỏ: sửa prompt 佟/老X; mở nhóm common; lời dặn soát không cho giữ nguyên vi phạm glossary.
2. P0 vừa: check thiếu target mức chương và tự thay cụm gần đúng; bảng âm Hán Việt.
3. P1: alias + cổng kiểm trùng, chống thoái hoá bằng out/ cũ, lọc danh xưng, lint story.json.
