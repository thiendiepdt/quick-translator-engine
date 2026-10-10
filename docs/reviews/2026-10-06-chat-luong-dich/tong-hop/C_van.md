# C_van: văn convert, meme dịch nghĩa đen, văn khô

Nguồn prompt thật là `apps/qt-web/src/lib/ai-translation-prompt.ts` (viết tắt **TS:dòng**). `crates/qt-ai-core/prompts/prompts.json` chỉ là bản sinh từ file này bằng `npm run golden` trong `apps/qt-ai-cli`. Sau khi sửa TS phải sinh lại prompts.json và cập nhật các fixture `crates/qt-ai-core/tests/fixtures/{prompt,check}.json` cùng `apps/qt-web/src/lib/ai-translation.test.ts`. Luật check nằm ở `apps/qt-web/src/lib/ai-translation.ts` (CHECK_RULES) và có bản chép y hệt ở `crates/qt-ai-core/src/check.rs:65-133`. Hiện không truyện nào tự đặt checkRules (64/64 rỗng) và trong `%APPDATA%` cũng không có file base nào người dùng sửa, nên mọi truyện đều chạy đúng bộ prompt và bộ rule cứng mặc định.

## 1. Tổng quan

- Có **313 lỗi**, dính đủ **61/61 truyện**. Theo mức: cao 22, vừa 172, thấp 119. Theo tính chất: 211 lỗi hệ thống, 102 lỗi lẻ.
- Theo thẻ: CALQUE 118 (61 truyện), MEME_NGHIA_DEN 70 (50), AN_THAN_TU 31 (30), BAN_THAN 31 (31), HU_HA 30 (30), VAN_KHO 23 (22), LAP_TU 9, KHO_KHAN 1.
- **Phát hiện chính:** những lỗi dày nhất là các lỗi prompt hoặc check *ra lệnh* làm, chứ không phải model tự nghĩ ra. Gồm: "Ân", "bản thân hắn", "nữ nhân", "lão tử", "kiểm trắc", "liếm chó", "hắc hắc". Riêng thán từ "Ân" còn bị bộ check **ép ngược lại**. Nhóm calque cú pháp (đem…, Theo sự…, chiếm tiện nghi) thì khác: prompt có cấm nhưng chỉ cấm chung chung, nên không ăn thua.

Tôi đếm lại trên toàn bộ `out/` của 64 truyện:

| Mẫu | cổ đại | hiện đại | ghi chú |
|---|---|---|---|
| dòng raw có 嗯 → dịch "Ân" | **3.383/6.100 (55%)** | **0/4.018** | mixed 0/157; chỉ tính file có số đoạn khớp |
| dòng có 呵 (không có 哼) → "Hừ" | 663/4.831 (14%) | 478/2.241 (21%) | tổng 1.151/7.119 |
| "bản thân + hắn/nàng/cô/y/lão/gã/nó" | 4.477 | 1.751 | "bản thân" nói chung: 41.310 |
| "nữ nhân" | 10.461 | 87 | "người nữ nhân" 195 lần/25 truyện |
| "người phụ nữ" | 116 | 3.938 | |
| "lão tử" | 916 | 335 | |
| "kiểm trắc" | 214 | 117 | 38 truyện |
| "liếm chó" | 1 | 118 | |
| "chiếm (hết) tiện nghi" | 470 | 82 | 42 truyện |
| "Theo sự/việc…" đầu câu | 317 | 69 | 42 truyện |
| "đem " | 7.703 | 1.910 | có lẫn "đem lòng" là cách dùng đúng |

Các mẫu đổi hẳn theo setting ("Ân", "nữ nhân", "người phụ nữ", "liếm chó") đều trùng với chỗ hai bộ prompt viết khác nhau. Đó là bằng chứng nguyên nhân nằm ở prompt.

## 2. Vấn đề hệ thống, xếp theo ảnh hưởng

### V1. 嗯 bị ép thành "Ân", 嗯? thành "A?" (P0, công sức nhỏ)

**Phạm vi:** 30 truyện, đều là cổ đại hoặc không khai genre (`story.rs:34` mặc định Ancient). Truyện hiện đại không có lần nào. 嗯 ra "Ân" ở 55% số dòng; riêng cụm "ân một tiếng" có 324 lần.

**Ví dụ:**
- xuong-nui ch0065:169 「“嗯。”/“我想要试试。”/“嗯。”」 → "Ân." … "Ân." ⇒ "Ừm." … "Ừ." Truyện này có mật độ 7,4 "Ân"/10.000 chữ, cao gấp 2,4 lần truyện đứng thứ hai.
- nam-nu-de ch0286:89 「嗯~」 → "Ân~" ⇒ "Ưm~". Đây là tiếng rên, dịch "Ân" phá hỏng cảnh.
- phuong-thon-dao-chu ch0038:105 「嗯？吴旷，慎言！」 → "A? …" ⇒ "Hửm? …". Truyện này có 65/73 lần 嗯? dịch thành "A?".

**Nguyên nhân gốc (đã xác minh trong code):**
1. TS:336, tức luật 11 của base cổ đại, ra lệnh *"khi trầm ngâm/đáp nhẹ dùng `Ân`, không dùng `Ừm/Hừm`"*, và 嗯? thì dùng "A?". Base mixed cũng có luật tương tự ở TS:656.
2. Luật 11 nằm trong khối *"Ràng buộc hệ thống — không quy tắc nào bên dưới được ghi đè"*. Mục giọng witty có dạy 嗯→Ừm/Ừ (TS:843) và còn lấy đúng câu "Ân, cả bàn…" làm ví dụ cột "Khô (tránh)" (TS:852). Nhưng mục này được `insert_tone` chèn *bên dưới* khối đó (`prompt.rs:52`). Phần style của truyện cũng nằm bên dưới và còn tự giới hạn "trong giới hạn trung thành" (`prompt.rs:183`). Kết quả là luật 11 thắng. 8/30 truyện bị lỗi này đặt tone witty.
3. **Check ép ngược:** `check.rs:84` và `ai-translation.ts:61` báo lỗi khi bản dịch có "Hừm/Ừm" trong truyện cổ đại, với thông điệp "→ Ân". Luồng Antigravity bảo agent "sửa theo review.md chứ không cãi" (AGENTS.md của từng truyện). Luồng API thì loại bản soát nào làm tăng số vi phạm (`api_session.rs:349-362`). Hệ quả: khi model tự viết đúng "Ừm", hệ thống bị đẩy quay về "Ân".

**Đề xuất sửa:**
- TS:336, TS:656: thay luật 11 bằng câu sau: *"11. Thán từ dịch theo chức năng trong câu, bằng thán từ tiếng Việt thật (bảng 'Thán từ & tiếng cười' ở mục 5). Không phiên âm Hán-Việt thán từ (`Ân`, `Hắc`, `Kiệt`). Thoại cổ trang chỉ tránh từ quá hiện đại (`Okay`, `Wow`)."* Đưa bảng chi tiết ra khỏi khối không-ghi-đè để tone và style chỉnh được.
- Xoá rule "Hừm/Ừm → Ân" ở `check.rs:84`, `ai-translation.ts:61` và các fixture (`ai-translation.test.ts:165,261`, `check.json:85,804,963`). Thay bằng hai rule áp mọi setting:
  - `(?<!\p{L})[Ââ]n(?=\s*(?:[,.!?~…]|$))` với thông điệp "嗯 → Ừ/Ừm/Ưm/Hửm, không phiên 'Ân'".
  - `[Ââ]n”? một tiếng` với thông điệp "→ 'ừ/ừm' một tiếng".
- Tuỳ chọn: sửa hàng loạt có kiểm soát 3.383 dòng cũ, theo các mẫu `“Ân.”`→`“Ừ.”`, `Ân~`→`Ưm~`, `Ân?`→`Hửm?` (công sức vừa).

**Bảng "Thán từ & tiếng cười"** (dùng chung cho mọi base, đặt ở mục 5):

| Raw | Chức năng | Dùng | Cấm |
|---|---|---|---|
| 嗯。/嗯， | đáp, đồng ý | Ừ / Ừm / Được / Vâng, Dạ (bề dưới) | Ân |
| 嗯…… | ngẫm nghĩ, ngập ngừng | Ừm… / Hừm… | Ân… |
| 嗯？ | hỏi lại, thắc mắc nhẹ | Hửm? / Hả? / Sao? | A?, Ân? |
| 嗯~ / 嗯…(rên) | rên, nũng | Ưm~ / Ưm… | Ân |
| “嗯”了一声 | | khẽ “ừm” một tiếng | ân một tiếng |
| 咦？ / 啊？ | giật mình | A? / Hả? / Ơ? (hiện đại) | |
| 呵 / 呵呵 | cười khẩy, cười nhạt | Ha / Hà / Ha ha / Hề hề, hoặc "cười khẩy" | **Hừ**, Hì hì |
| 呵呵 (hiền, xã giao, người già) | | Ha ha / Hà hà / Khà khà | Hừ hừ |
| 哼 | hừ mũi, giận, dỗi | Hừ / Hứ | |
| 哼哼 (tiếng ngáy) | | Khò khò | Hừ hừ |
| 嘿嘿 | cười gian / cười khoái chí, ngượng | hắc hắc / hì hì, hề hề | |
| 桀桀 | cười gian ác | khặc khặc / khà khà | kiệt kiệt |
| 哈哈笑道 | | cười ha hả | ha ha cười |
| 啊 cuối câu | | mà / đấy / chứ / thật, hoặc bỏ | a |
| 呃 / 额 / 唔 | | Ờ / Ờm / Ừm | Ân |

Kèm một dòng luật: *"Câu dẫn có 笑 (cười, cười lạnh, cười khẽ) thì thán từ đi trước nó là tiếng cười, không được là 'Hừ'."*

### V2. 自己 → "bản thân (hắn)" (P0, nhỏ)

**Phạm vi:** 31 truyện, cả cổ đại (19) lẫn hiện đại (11). Có 6.278 lần "bản thân + đại từ"; ví dụ thau-huong có 752 lần/606 file, thinh-tien-sinh có 475 lần.

**Ví dụ:**
- pham-nhan ch0203:107 「都在自己的庇护下」 → "dưới sự che chở của bản thân hắn" ⇒ "… của ta". Đây là độc thoại ngôi một, dịch như vậy làm đảo nghĩa.
- cuc-pham-pho-ma ch1138:23 「必须逼着自己来决定」 → "muốn ép bản thân phải tự mình quyết định" ⇒ "là muốn ép hắn phải tự quyết". Bản dịch hiện tại đổi luôn chủ thể của hành động.
- ta-cung-khong-muon ch0001:59 「自己如今是淬体境二重」 → "bản thân hiện tại là…" ⇒ "giờ ta đang ở…".

**Nguyên nhân gốc (đã xác minh):** luật "Lời kể gián tiếp" đưa sẵn `bản thân hắn` (TS:357, cổ đại và mixed) và `bản thân cô` (TS:567, hiện đại) làm ví dụ mẫu cho 自己. Model lấy luôn ví dụ đó làm công thức. Thêm vào đó, prompt không có câu nào cấm "bản thân" đứng làm chủ ngữ.

**Đề xuất sửa:** thay đoạn ở TS:357 và TS:567, giữ ý chống đổi ngôi:

> **`自己` trong lời kể:** `自己` là đại từ hồi chỉ, không phải danh từ "bản thân". Ngoài ngoặc kép, dịch bằng đại từ ngôi ba của nhân vật đang được kể (`hắn`/`nàng`, hiện đại `hắn`/`cô`) hoặc **lược** khi chủ thể đã rõ. Chỉ dùng `chính hắn`/`tự mình` khi raw nhấn đối lập (他自己也…, 自己动手). KHÔNG dùng `bản thân` làm chủ ngữ, sau giới từ (`của bản thân`, `với bản thân hắn`) hay ghép đại từ (`bản thân hắn`). `bản thân` chỉ dùng cho 自身/本身 khi nghĩa là "tự thân". Độc thoại của người kể ngôi một: `ta`. Không đổi ngôi giữa một mạch kể liền nhau (giữ câu cũ).

**Lưu ý:** nhiều reviewer đề xuất dùng "mình" thay cho "bản thân". Không nên đưa "mình" vào truyện cổ đại, vì nó va với `check.rs:72` (bắt "mình/tôi") và với quy ước người kể xưng "ta".

**Check mới (cảnh báo mềm):**
- `(?<!\p{L})bản thân (?:hắn|nàng|cô|y|lão|gã|nó)(?!\p{L})`
- `(?:^|[.!?“"]\s*)Bản thân(?! là)`

### V3. Calque cú pháp và lượng từ: luật có nhưng không ăn (P1, vừa)

**Phạm vi:** CALQUE có 118 lỗi ở cả 61 truyện. Nổi nhất là "đem…" (~9.700 lần, có lẫn "đem lòng" dùng đúng), "Theo sự/việc" 389 lần/42 truyện, "chiếm tiện nghi" 552 lần/42 truyện. hop-hoan có 1.443/2.485 dòng chứa 一阵 bị dịch "một phen/hồi/trận". the-gioi-tu-tien có 297 lần "trực tiếp".

**Ví dụ:**
- rut-dao ch0276:77 「将所有残存真气…尽数灌入这一刀」 → "đem tất cả chân khí… rót hết vào" ⇒ "dồn hết chỗ chân khí… vào nhát đao này".
- ta-tai-tan-the ch0001:41 「随着武器消失」 → "Theo sự biến mất của vũ khí" ⇒ "Khẩu súng vừa biến mất".
- sao-the ch0030:175 「又占我便宜」 → "Lại chiếm tiện nghi của tôi" ⇒ "lại sàm sỡ tôi nữa rồi".

**Nguyên nhân gốc:**
- Prompt **có** luật đúng hướng: luật 8 "Bắt buộc dựng lại cú pháp", §0 "Không bê khung câu Trung", và "Phép thử bắt buộc".
- Nhưng các bảng mẫu "Chống văn convert" (TS:108) và "Dấu hiệu câu vẫn còn là convert" (TS:130) chỉ liệt kê những mẫu hiếm (u u thở dài, tinh thần đại chấn…). Không bảng nào nêu 把/将, 随着, 占便宜, 一阵, 一丝, 一层, 直接, 好一个, 好家伙, 既然, 总不能.
- Bộ check có hơn 60 rule nhưng không rule nào bắt mấy mẫu này (`check.rs:65-133`).
- Dòng "nhận dạng → **kiểm trắc**" trong bảng "Tránh dùng" (TS:237) còn *ra lệnh* dùng "kiểm trắc". Đây là gốc của "Kiểm trắc thấy…" (xuyen-sach 35 lần; toàn thư viện 335 lần ở 38 truyện).

**Đề xuất sửa:**
- Bổ sung các dòng sau vào bảng TS:130 (dùng chung mọi base).
- Xoá dòng "nhận dạng | kiểm trắc", thay bằng `检测 (thông báo hệ thống) | Phát hiện / Quét thấy | kiểm trắc`.

| Raw | Tự nhiên | KHÔNG |
|---|---|---|
| 把/将 + O + V | đưa động từ lên trước: 把话说出来 → nói ra lời trong lòng | đem O + V |
| 随着 X, … | X vừa…, thì… / khi X… | Theo sự/việc X; Cùng với việc |
| 占便宜 | lợi dụng / được hời / sàm sỡ / nhận vơ vai trên (theo cảnh) | chiếm tiện nghi |
| 便宜老爹/爸妈 | ông bố trên trời rơi xuống | tiện nghi lão cha, bố hời |
| 一阵/一丝/一层/一股 + cảm xúc | bỏ lượng từ: 一阵无辜 → ra vẻ vô tội; 一丝虚弱 → thoáng yếu ớt | một phen/một tia/một tầng/một cỗ + cảm xúc |
| 一头 (thú), 这片 (vùng), 这方 (thế giới) | một con / vùng này / cõi này | một đầu, phiến, phương |
| 直接 (trạng từ nhấn) | thẳng / luôn / liền, hoặc bỏ | trực tiếp |
| 好一个 X | Hay cho X / X giỏi lắm | Được một cái X, Tốt cho X |
| 好家伙 | Chà / Trời đất / Ghê thật | Khá lắm |
| 既然…就… | đã… thì… | Đã là + đại từ |
| 总不能 | đâu thể / chẳng lẽ | tổng không thể |
| 而 đầu câu | bỏ, hoặc dùng "Còn" | Mà… |
| 微妙 / 恐怖 / 憋屈 / 毒死 / 万万没想到 | khó tả / kinh hoàng / ấm ức / đầu độc / nào ngờ | vi diệu / khủng bố / nghẹn khuất / độc chết / vạn vạn không ngờ |
| X队 / X导 / X主编 / X同学 | đội trưởng X / đạo diễn X / tổng biên tập X / tên hoặc "em X" | X đội, X đạo, bạn học X |
| 流量 / 操作 (dàn xếp) / 晚上好 | lượt xem, độ hot / xoay xở / chào | lưu lượng / thao tác / buổi tối tốt lành |

**Check mới (cảnh báo mềm, mọi setting):**
- `(?:^|[.!?“"]\s*)Theo (?:sự|việc)\b`
- `chiếm (?:hết )?tiện nghi`
- `(?<!\p{L})đem\s(?!lòng)`: chỉ cảnh báo khi mật độ vượt khoảng 1 lần/1.500 chữ. Cần thêm ngưỡng đếm vào `check_violations`, vì hiện hàm này chỉ khớp theo từng dòng.
- `một (?:tia|tầng|cỗ) (?:\p{L}+ ){0,2}(?:ghét|khinh|vui|buồn|lo|sợ|giận|yếu)`

### V4. Meme, lóng, chơi chữ bị dịch nghĩa đen (P1, vừa)

**Phạm vi:** 70 lỗi ở 50 truyện. Ví dụ: "gân gà" 101 lần/26 truyện, "liếm chó" 118 lần, "Lầu trên" 42 lần (bien-than-nu-dao-si).

**Ví dụ:**
- bai-poker ch0134:69 「他们围攻个锤子」 → "vây công cái búa ấy" ⇒ "Vây công cái con khỉ ấy!"
- tam-muoi-tuoi ch0023:49 「舔狗」 → "kẻ liếm chó" ⇒ "liếm cẩu / kẻ lụy tình". Cách dịch hiện tại lật nghĩa thành người đi liếm chó.
- trong-sinh-thanh-cau ch0012:217 「救命恩狗」 → "chú chó ân nhân cứu mạng" ⇒ "ân… à nhầm, ân 'cẩu' cứu mạng".

**Nguyên nhân gốc:**
- **Đã xác minh:** dòng `舔狗 | liếm chó / kẻ bám đuôi` ở TS:621 (base hiện đại) chính là gốc của "liếm chó".
- **Nghi mạnh, dựa vào câu chữ prompt:** các luật về trung thành không có ngoại lệ cho meme.
  - "KHÔNG giải thích ẩn dụ, thuật ngữ" (TS:81).
  - "KHÔNG giải thích thuật ngữ trong ngoặc đơn" (TS:302).
  - Dòng 金手指 ghi "Giữ nguyên hình ảnh, không giải thích" (TS:280).
  - Luật 13 "Glossary thắng thành ngữ, kể cả khi câu đang chơi chữ" (TS:48).

  Mục thành ngữ có dạy dịch nghĩa bóng (TS:251), nhưng chỉ nói về thành ngữ và điển cố, không đả động gì tới lóng mạng hay meme. Với chơi chữ, prompt hiện không để model một cách hợp lệ nào: đã cấm giải thích, lại bắt giữ tên glossary. Bản tự sinh glossary không loại meme và biệt danh (`api_session.rs:31-62`), nên các cụm như 大实力, 牛头人, 修仙钉子户 bị chốt thành tên riêng. Phần này xem thêm nhóm B.

**Đề xuất sửa:** thêm mục mới vào §6 (TS:274), dùng chung mọi base:

> ### Meme, lóng mạng, chơi chữ
> - Lóng mạng và meme là thành ngữ: dịch **nghĩa bóng** bằng cách nói người Việt hiểu ngay. Dịch nghĩa đen từng chữ (gân gà, sói mắt trắng, lầu trên, cái búa) là **sai nghĩa**, không phải trung thành. Tiếng Việt có lóng tương đương quen thuộc (vả mặt, quay xe, làm màu, chị đại) thì dùng.
> - Chơi chữ đồng âm hoặc biến tên (整人菌子↔正人君子, 恩狗↔恩人, 无中生友, 河童↔贺桐): dựng lại cú chơi chữ bằng tiếng Việt. Không dựng lại được thì giữ ý đùa, và **được** chèn một cụm giải nghĩa ngắn ngay trong câu. Đây là ngoại lệ duy nhất của luật "không giải thích".
> - Biệt danh và meme không phải tên riêng: không phiên Hán-Việt viết hoa (Tiểu Lạt Tiêu, Bức Vương), không khai vào glossary names.

Sửa thêm:
- TS:81 thành "…không giải thích ẩn dụ *văn chương của tác giả*; meme/lóng xem mục 6".
- Bổ sung vào luật 13: "trừ khi mục glossary là meme hoặc biệt danh bị phiên nhầm, khi đó báo để sửa glossary".
- TS:621 sửa thành `舔狗 | liếm cẩu / kẻ lụy tình | liếm chó`.

**Bảng lóng cố định** (bổ sung vào bảng ở TS:613 và dùng cho cả base cổ đại):

| Raw | Dùng | Raw | Dùng |
|---|---|---|---|
| 鸡肋 | vô dụng, chẳng được tích sự gì | 白眼狼 | đồ vô ơn |
| 个锤子 / 懂个锤 | cái con khỉ / biết cái khỉ gì | 楼上 | bạn ở trên |
| 真香 | quay xe | 画(大)饼 | hứa suông, vẽ bánh vẽ |
| 打了鸡血 | hăng máu | 喝西北风 | nhịn đói, hít gió mà sống |
| 抱大腿 | bám đùi, bám chỗ dựa | 领盒饭 | đi đời, hết vai |
| 麻了 | cạn lời, đơ người | 破防 | sụp đổ tâm lý |
| 开车 (viết văn) | viết cảnh nóng | 小黑屋 | bị khoá truyện |
| 老司机 | tay lão luyện | 6 / 666 | đỉnh / bó tay (theo cảnh) |
| 欧气 | nhân phẩm bùng nổ | 爆率 | tỉ lệ rơi |
| 路边一条 | đồ ven đường ai thèm để ý | 绿 / 戴绿帽 | bị cắm sừng |
| 牛头人 | NTR, kẻ cướp người yêu | 装逼 / 逼王 | làm màu / vua làm màu |
| 抽象 (mạng) | dị, tấu hài | 蹦迪 | quẩy |
| 彩虹屁 | khen nức nở | 虎狼之词 | ăn nói bạo |
| 我信你个鬼 | tin ngươi mới có ma | 出来混迟早要还的 | ra giang hồ sớm muộn cũng phải trả |
| 钉子户 | kẻ bám trụ lì | 老登 | lão già khọm |
| 大冤种 | đứa số nhọ | 就这? | Có thế thôi á? |
| 小辣椒 | cô nàng ớt cay | …的节奏 | coi như chắc… |

### V5. 呵 → "Hừ", tiếng cười lẫn lộn (P1, nhỏ cho prompt, vừa cho check)

**Phạm vi:** 30 truyện. 呵 bị dịch thành "Hừ" ở 1.151/7.119 dòng, gặp ở cả cổ đại lẫn hiện đại.

**Ví dụ:**
- tam-muoi-tuoi ch0008:20 「气极反笑：“呵…”」 → "giận quá hóa cười: 'Hừ…'". Câu dẫn nói cười mà thán từ lại là hừ, tự mâu thuẫn.
- tuyet-doi-rung-dong ch0028:229 「“呵……”他轻笑一声」 → "Hừ… Hắn bật cười khẽ" ⇒ "Ha…".
- nhuong-nguoi ch0001:9 「哼哼哼（鼾声）」 → "Hừ hừ hừ" ⇒ "Khò khò khò".

**Nguyên nhân gốc:** cả ba base đều **không có** dòng nào cho 呵 (grep prompts.json không ra). Bảng tượng thanh lại ghi `嘿嘿 → hắc hắc` (TS:270), trong khi mục witty ghi `嘿嘿 → hì hì` (TS:843), hai chỗ mâu thuẫn nhau. Việc 呵 gần như luôn ra "Hừ" thì tôi chỉ ghi là **nghi** do thói quen văn convert của model.

**Đề xuất sửa:**
- Dùng bảng thán từ ở V1.
- Thêm một check có đối chiếu raw: hàm mới `laugh_mismatches(raw, translated)` trong `check.rs`, viết theo kiểu `glossary_drifts`. Hàm báo khi đoạn raw có 呵 mà không có 哼 hay 冷哼, còn đoạn dịch lại có `\bHừ`. Gắn vào `commands/check.rs:141` và vào hàm `score` ở `api_session.rs:349`.

### V6. Từ register bị bảng prompt chốt cứng (P1, nhỏ)

**Phạm vi:**
- 女人 → "nữ nhân" ở cổ đại (10.461 lần), sinh ra cả "người nữ nhân" (195 lần).
- 女人 → "người phụ nữ" ở hiện đại (3.938 lần). Ví dụ: "Này người phụ nữ kia" cho câu cửa miệng 女人，你….
- 老子 → "lão tử" (hop-hoan 69/76 lần) dù toneRules của truyện đã dặn "ông đây".

**Nguyên nhân gốc (đã xác minh):**
- TS:531 chốt `女人 → nữ nhân/nữ tử`, kèm `check.rs:69` bắt "phụ nữ/đàn bà" trong truyện cổ đại.
- TS:351 chốt `老子 → lão tử`.
- Base hiện đại không có dòng nào cho 女人, nên model rơi về "người phụ nữ".

**Đề xuất sửa:**
- TS:531 sửa thành: "女人: lần đầu giới thiệu hoặc nói về giới → nữ nhân/nữ tử; đã biết là ai → nàng / ả / cô ta theo thái độ người kể; cấm 'người nữ nhân'".
- Base hiện đại thêm dòng: "女人 → cô / cô ta / con nhỏ / bà chị / đàn bà theo giọng; 女人，你… (giọng tổng tài) → Cô gái à / Này cô kia".
- TS:351 sửa thành "lão tử / ông đây (style truyện quyết định)".
- Đổi thông điệp của `check.rs:69` thành "→ nữ tử/nàng/ả".
- Ca lao-ba cũng thuộc dạng này: truyện hiện đại bị đặt setting cổ đại nên dính lệnh cấm "vợ/chồng" (TS:522, `check.rs:68`), sinh ra "nhạc mẫu, tiểu di tử". Cách sửa là đổi setting của truyện (nhóm A/setting).

## 3. Chủ trương "trung thành hơn mượt, cấm tô màu" có làm văn khô?

**Kết luận: không có bằng chứng cho thấy bản thân chủ trương này gây khô. Văn khô đến từ ba thứ cụ thể:**
1. **Luật cứng cụ thể đặt sai tầng:** "Ân" nằm trong khối "không được ghi đè" nên thắng cả tone lẫn style (V1). Thêm các bảng register (nữ nhân, lão tử, kiểm trắc) và một check ép ngược. Bằng chứng: cùng một model nhưng ở base hiện đại không bị ép thì 嗯 ra "Ân" 0/4.018 dòng.
2. **Thiếu ngoại lệ cho meme và chơi chữ:** "không giải thích ẩn dụ", "không ngoặc chú thích", "glossary thắng cả chơi chữ" khiến model không còn cách hợp lệ nào ngoài dịch nghĩa đen (V4). Đây là chỗ chủ trương trung thành *có góp phần*. Mức độ chắc chắn: suy từ câu chữ prompt, chưa chạy thử A/B.
3. **Calque cú pháp không phải do trung thành:** prompt đã ra lệnh dựng lại câu (luật 8). Lỗi còn tồn tại vì thiếu ví dụ đích danh và thiếu check (V3).

Có một chỗ tôi chỉ ghi là **nghi**: luật biên tập "chỉ sửa trong phạm vi nhỏ nhất" (TS: mục 8.1) cộng với luật 4 "ưu tiên đúng hơn trơn tru" có thể làm lượt soát ngại dựng lại câu "đem…". Chưa có số liệu để khẳng định.

**Không nên** nới lỏng "cấm tô màu". Ca ky-tuc-xa 又又又又 bị gọt thành "một lần nữa" cho thấy model vốn đã có xu hướng làm phẳng; còn các truyện được khen "văn mượt" thì vẫn chạy cùng bộ luật đó.

## 4. Lỗi lẻ đáng chú ý

- **Lặp máy, check chưa bắt:** "Lúc này lúc này" (thinh-tien-sinh 0605, 1353), "dở sống dở dở chết", "tầng lớp cao tầng" (17 lần). Đề xuất rule `(?<!\p{L})(\p{L}+ \p{L}+) \1(?!\p{L})` làm cảnh báo mềm.
- 去你的 → "Đi chết đi" trong cảnh làm nũng (tam-muoi-tuoi, 6 dòng).
- Thành ngữ dịch sai nghĩa: "thuận buồm xuôi gió" dùng cho 得心应手, "rải rác" dùng cho 铺天盖地.
- Tên Nhật: 桑 → "tang" (thau-huong). Nên dịch là "-san".

## 5. Điều đang làm tốt (để không sửa nhầm)

- Thán từ ở base hiện đại đã ổn: tam-muoi-tuoi dịch 嗯 thành "Vâng/Ừm/Vâng ạ" và không có "Ân". Bảng ở TS:549 dùng được; chỉ cần đưa phần tương ứng sang base cổ đại.
- Nhiều meme đã được giải mã đúng, chứng tỏ model làm được khi không bị cấm: 杂鱼 → "Tép riu", 细狗 → "Đồ yếu xìu", 添点颜色 → "đội nón xanh", 出个屁头 → "Ra mặt cái con khỉ", 落井下石 → "dậu đổ bìm leo".
- Thành ngữ Việt hoá đắt ("đánh chó còn phải ngó mặt chủ", "chém cỏ trừ tận gốc"). Arc cổ đại và arc hiện đại tách giọng rõ (cai-nay-ngo-khong). Các truyện ít "bản thân" (sau-khi-chia-tay, dinh-luu-van) cho thấy model không tự sinh ra lỗi này khi prompt không mớm.
- Giữ nguyên các nguyên tắc: không thêm ý, giữ đủ đoạn, giữ lặp có chủ ý, và bảng "Tránh dùng" (trừ dòng "kiểm trắc").
