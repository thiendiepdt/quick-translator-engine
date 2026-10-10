# Nhóm D: Sai nghĩa, sai số, định dạng, tiêu đề, toàn vẹn raw↔out

## 1. Tổng quan

Digest có **177 lỗi ở 62 slug**, trong đó 19 lỗi mức cao.

| Thẻ | Lỗi | Số truyện | Cao / vừa / thấp | Gắn HT |
|---|---|---|---|---|
| SAI_NGHIA | 82 | 50 | 9 / 49 / 24 | 5 |
| DINH_DANG | 36 | 32 | 1 / 9 / 26 | 34 |
| TIEU_DE | 29 | 29 | 2 / 7 / 20 | 21 |
| KHAC | 11 | 10 | 2 / 2 / 7 | 7 |
| SAI_SO | 10 | 9 | 4 / 3 / 3 | 6 |
| THEM_Y | 6 | 6 | 1 / 2 / 3 | 2 |
| SOT_Y | 3 | 3 | 0 / 0 / 3 | 0 |

Nhóm này có hai kiểu lỗi khác nhau:

- **Sai nghĩa:** phần lớn là lỗi lẻ, nhưng lặp lại theo vài khuôn nhận ra được.
- **Định dạng và tiêu đề:** gần như toàn là lỗi hệ thống, đổi theo từng phiên dịch 10 chương.

Ngoài các số trong digest, tôi đếm thêm bằng script trên toàn thư viện (64 truyện, 27.066 file out):

- 4.944 file chỉ dùng ngoặc thẳng `"`, 1.531 file trộn hai kiểu ngoặc; 58/64 truyện có ít nhất một file ngoặc thẳng.
- 122.102 cụm `......`.
- Khoảng 1.180 tiêu đề viết hoa mọi chữ (heuristic: ≥4 chữ đều viết hoa).
- 127 tiêu đề "Chương N" thiếu dấu `:`.
- 25 file (7 truyện) có số chương trong tiêu đề out khác số trong raw.

**Check hiện tại gần như không bắt được gì ở nhóm này.** Trong 27.066 chương done, chỉ 42 chương có warnings, và 40 trong số đó thuộc dinh-luu-van, mà phần lớn lại là cảnh báo sai.

## 2. Các vấn đề hệ thống, xếp theo ảnh hưởng

### V1. Raw bị đổi sau khi dịch mà state vẫn "done" (P0, công sức vừa)

**Mô tả.** Đã xác minh (sự thật số 4) ở phuong-thon-dao-chu:

- raw/ được nhập lại lúc 06/10 07:39 và mất ch382.
- out/0392 là "Chương 382: Vị trí các chòm tinh tú", trong khi raw/0392 bây giờ là `第383章 道合陷囹圄`. State vẫn ghi done.
- out/0364–0378 mang tên chương lệch một bậc. Ví dụ out/0363 "Tàn hồn biến đổi theo thời gian" lại là tên của raw/0364 `残魂因时变`.
- 0370/0371 hoán đổi đơn xin nghỉ: raw/0371 là `请假条`, nhưng out/0371 là "Chương 362: Phép lấy sát nuôi sát".

**Tôi tìm thêm được một ca thứ hai:** truong-sinh-la-meo.

- raw/0252 hiện là bản trùng của raw/0251 (`第251章 兴奋了`), mtime 05/10.
- out/0252 là "Chương 252: Miêu Tổ là tiên thú" (30/09). Nghĩa là ch252 thật đã biến mất khỏi raw mà không ai hay.

**Quy mô:** 6 truyện có gần như toàn bộ raw mới hơn out (mtime raw > out):

| Truyện | File raw mới hơn out |
|---|---|
| bat-dau-ho-tong | 312/312 |
| dau-la-dao-nguoc | 322/324 |
| ma-mon | 65/238 |
| nam-nu-de | 286/288 |
| phuong-thon | 392/393 |
| truong-sinh-la-meo | 265/268 |

Cả 6 truyện đều dùng id theo vị trí `chuong-NNNN`. Chỉ cần raw tải lại thiếu hoặc thừa một chương là mọi id phía sau đều lệch. Ở 4 truyện còn lại, số đoạn và số chương vẫn khớp, nên nhiều khả năng nội dung không đổi, nhưng chưa xác minh được.

**Nguyên nhân gốc (đã thấy trong code):**

- `ChapterState` (`crates/qt-ai-core/src/story_fs.rs:73-82`) chỉ lưu status, review_round, reason, warnings, updated_at. Không có dấu vân tay nào của raw.
- `raw_drift` (`commands/init.rs:15-22`) và `run_init` (`init.rs:35-55`) chỉ so **tập id**: raw đổi nội dung mà giữ tên file thì không ai biết.
- `accept` ghi Done (`commands/accept.rs:108-117`) mà không lưu gì về raw.
- Import trong GUI không ghi đè (`apps/qt-ai-gui/src-tauri/src/library_cmds.rs:168-171`). Vì vậy nghi raw bị ghi đè từ bên ngoài app, tức crawler.

**Đề xuất:**

1. Thêm `raw_hash: Option<String>` vào `ChapterState`, là hash của raw đã chuẩn hoá xuống dòng. `run_accept` ghi giá trị này.
2. Đổi `raw_drift` để trả về `(added, removed, changed)`.
   - `run_init` đưa chương có `changed` về Queued, kèm `reason: "raw đổi sau khi dịch"`.
   - Bản dịch cũ được chuyển sang `out/<id>.stale.txt`, không xoá.
   - "Quét lại thư viện" (`library_cmds.rs:229-237`) báo thêm cột changed.
3. Backfill cho 27k chương cũ chưa có hash bằng hai phép so rẻ:
   - Số `第N章` trong raw ≠ số `Chương N` trong out: 25 file. Trong đó có 2 ca drift; phần còn lại là model tự đổi số, ví dụ tien-tu-ta 0758: raw `第七百七十八章` → out "Chương 758"; lol 0133: raw 132 → out 133.
   - Số đoạn raw ≠ số đoạn out: chỉ 4/27.066 file, gồm phuong-thon 0371 và 0392, truong-sinh-la-meo 0075 và 0252.

   Cả hai phép đều ít báo nhầm. Chương khớp thì ghi hash luôn.
4. Đưa phép so số chương vào `check` thành lỗi cứng: số trong tiêu đề out phải bằng số trong raw. Hàm `parse_source_title` ở `titles.rs:147` đã có sẵn.
5. Đề xuất phía crawler: dùng id ổn định (số chương hoặc id nguồn, như các truyện có id `0031_7449698198106407486`), không dùng số thứ tự file.

### V2. Số và đơn vị: 亿, 万, 斤, 一倍, 百亿 (P0, công sức nhỏ cho prompt, vừa cho check)

**Mô tả.** 10 lỗi SAI_SO ở 9 truyện, 4 lỗi mức cao. Lỗi này làm sai hẳn con số thật của cốt truyện. Đếm lại toàn thư viện, chỉ tính file mà raw cùng chương có chữ tương ứng:

| Lỗi | Modern | Ancient |
|---|---|---|
| Số + "ức" khi raw có 亿 | 123 lần / 28 file / 7 truyện | 113 lần / 53 file / 10 truyện |
| "cân" khi raw có 斤 | 161 lần / 120 file / 21 truyện | — |
| "<số> vạn" | 441 lần / 10 truyện | — |
| raw 百亿 so với out "trăm tỷ" | 182 so với 98 | — |

- **ức:** ví dụ thinh-tien-sinh-cuu-ta "Số dư điểm hành thiện: 9,6 ức".
- **cân:** 斤 = 0,5 kg nên số bị gấp đôi.
- **百亿 → "trăm tỷ":** "trăm tỷ" phải đến từ 千亿, nên đa số chỗ này sai bậc.

**Ví dụ:**

- di-lam-mo-ca ch1247: `筹集了一百亿给他买别墅` → "gom góp một trăm ức" ⇒ "gom góp mười tỷ".
- dinh-luu-van ch0002: `即可获得百亿资产` → "khối tài sản trăm tỷ" ⇒ "mười tỷ". summary trong story.json cũng ghi "trăm tỷ", nên prompt kéo lỗi theo.
- bat-dau-ho-tong ch0312: `足足多了将近一倍` → "nhiều hơn gần một nửa" ⇒ "nhiều gần gấp đôi". moi-ngay ch0001 cũng sai cùng kiểu: `快了一倍` → "nhanh hơn hẳn một nửa".
- tan-the-nguoi-khac ch0001: `一百斤重压` → "một trăm cân" ⇒ "năm chục ký".

**Nguyên nhân gốc (đã thấy):** prompt không có luật nào về số hay đơn vị. Grep `亿|斤|一倍|小时|米` trong `apps/qt-web/src/lib/ai-translation-prompt.ts` không ra kết quả nào. Model tự chọn cách viết, và cách chọn đổi theo phiên.

**Đề xuất:**

- **Prompt.** Thêm mục "### Số và đơn vị" vào `CORE_PUNCT` (`ai-translation-prompt.ts`, sau dòng 217), rồi chạy gen-golden để sinh lại `prompts.json`:

  > "万 = mười nghìn, 亿 = một trăm triệu: 20万 → 200 nghìn, 3亿 → 300 triệu, 一百亿/百亿 → 10 tỷ, 千亿 → 100 tỷ. KHÔNG viết 'ức'; bối cảnh hiện đại không viết '<số> vạn'. 多/快/大了一倍 = gấp đôi (không phải 'một nửa'). Bối cảnh hiện đại: 斤 = nửa ký (200斤 → 100 ký), 小时 = tiếng (không 'canh giờ'), 米 = mét. 三五 = dăm ba."

- **Check.** Cần một loại luật mới: luật có điều kiện theo raw. Hiện `check_violations` (`check.rs:250-283`) chỉ chạy regex trên out. Đề xuất:
  - Thêm `raw: Option<String>` vào `CheckRule` (`story.rs`).
  - Trong `check_chapter` (`commands/check.rs:138`), chạy các luật này theo từng cặp `paragraphs[i]` ↔ `parsed[i]`; nhãn [[n]] đã căn sẵn hai bên.
  - Đặt nhóm luật này ngoài `DEFAULT_RULES` (luật ở đó bị bỏ khi truyện có checkRules riêng, xem `check.rs:252-263`), và nguồn thật phải ở `CHECK_RULES` (`apps/qt-web/src/lib/ai-translation.ts:42`) vì có golden test.

  Luật cụ thể (raw ⇒ out):

  | Raw | Out | Mức |
  |---|---|---|
  | `亿` | `(?:\d|mười|trăm|nghìn|ngàn|mấy|vài|một|hai|ba|bốn|năm|sáu|bảy|tám|chín)\s+ức(?!\p{L})` | lỗi |
  | `百亿` (raw không có `千亿`) | `trăm t[ỷỉ]` | lỗi |
  | `一倍` | `một nửa\|gấp rưỡi` | lỗi |
  | `万` | `\d[\d.,]*\s*vạn(?!\p{L})` (modern) | cảnh báo |
  | `斤` | `(?<!\p{L})cân(?!\p{L})` (modern) | cảnh báo |
  | `小时` | `canh giờ` | cảnh báo |
  | `三五` | `ba năm` | cảnh báo |
  | `\d+\s*米` | `trượng\|thước` | cảnh báo |

- **Sửa dữ liệu:** sửa summary của dinh-luu-van.

### V3. Sai nghĩa lặp theo khuôn: chặn được một phần bằng check, phần còn lại cần soát (P1)

Đây là thẻ lớn nhất của nhóm: 82 lỗi SAI_NGHIA ở 50 truyện. Tôi chia thành 4 khuôn.

**(a) "Bạn giả Hán-Việt" và từ đa nghĩa.** Khoảng 55 lỗi. Model chọn từ Việt cùng âm Hán-Việt nhưng khác nghĩa, hoặc chọn sai nghĩa của từ đa nghĩa:

- 打击 → "kích thích" (am-thinh ch0386)
- 放肆 → "phóng túng" (bat-dau-ho-tong ch0311)
- 铁青 → "sắt thép"
- 交待 → "lời ăn tiếng nói" (cuc-pham 1142, thau-huong 2509)
- 头颅 → "đầu lâu" (ta-tai-tan-the: 4 file)
- 龙眼 → "mắt rồng"
- 老家 → "lão gia"
- 前台 → "tiền tuyến"
- 拳力 → "quyền lực"
- 总不能 → "tổng thể"
- 欣慰 → "an ủi"
- 退休 → "giải ngũ"
- 谨 → "cẩn trọng"
- 老妇人 → "lão phụ thân" (tien-tu-ta ch0480, mức cao)
- 樱花 → "hoa đào"

**(b) Ghép nhầm thành ngữ Việt, hoặc bịa lai Hán-Việt:**

- 一个头两个大 → "một đầu hai thứ tóc": thau-huong, 5 lần / 5 file.
- 藏着掖着 → "giấu đầu lòi đuôi"
- 纸醉金迷 → "vàng thau lẫn lộn"
- 欲哭无泪 → "dở khóc dở cười"
- "tội có thích đáng", "Hảo nào", "Bất hẹn mà cùng", "nói mồm không bằng không chứng", "u mê ám muội"

**(c) Đảo chủ thể hoặc logic.** Đây là các lỗi cao nhất về hậu quả:

- saint-seiya ch0055: gán việc "lo cho con gái" sang Oceanus.
- xuc-tu ch0204: `周屹被俘虏的不止是身体` → "Thứ bị Chu Ngật bắt làm tù binh" (đảo chủ thể).
- huyen-giam ch0079: một người bị tách thành hai.
- saint-seiya ch0001: câu hỏi tu từ `哪里还管得了` thành câu khẳng định.
- 你讨厌了啦 → "đệ ghét ghê cơ".

**(d) Văn hoá, meme, chơi chữ, lỗi gõ trong raw:**

- 我打尼玛 → "Tao đập con mẹ mày" (lol ch0158, mức cao).
- 随你呗 → "Tùy bố thôi ạ" (tuyet-doi ch0087, mức cao).
- 石楠花, chơi chữ 青/清.
- Raw gõ nhầm 逆星宫 (đúng là 逆星盟) và 确实 (đúng là 却是), model dịch theo lỗi gõ.
- 阿青 → "A Khang" (thau-huong 2509), dù glossary đã có 阿青 → A Thanh.

**Nguyên nhân gốc:** do model; prompt không gây ra. Prompt đã có luật trung thành và luật về mức độ (`ai-translation-prompt.ts:176-182`). Khâu kiểm hiện có:

- Regex trên out (`check.rs:65-133`): đã có tiền lệ cho khuôn (a), ví dụ "não hải", "vô ngữ", "lãnh tình", nhưng chỉ xét out, không xét raw.
- Lệch glossary, nhưng chỉ bắt lệch dấu thanh (`check.rs:186-224`), nên "A Khang" lọt.
- Xưng hô, thiếu đoạn, còn chữ Hán, tỉ lệ độ dài tối thiểu.
- Vòng review (`commands/check.rs:65-87`) chỉ đưa model sửa đúng các vi phạm trên. **Không có bước nào đọc đối chiếu nghĩa.**

**Đề xuất:**

- **Khuôn (a) và (b): làm được bằng check (P1, nhỏ khi đã có luật raw-có-điều-kiện ở V2).**
  - Thêm bảng "bẫy đồng âm" (raw ⇒ out cấm):
    - `头颅|脑袋|人头` ⇒ `đầu lâu` (raw không có `骷髅|头骨`)
    - `放肆` ⇒ `phóng túng`
    - `铁青` ⇒ `sắt thép`
    - `打击` ⇒ `kích thích`
    - `交[待代]` ⇒ `lời ăn tiếng nói`
    - `龙眼` ⇒ `mắt rồng`
    - `老家` ⇒ `lão gia`
    - `前台` ⇒ `tiền tuyến`
    - `拳力` ⇒ `quyền lực`
    - `总不能` ⇒ `tổng thể`
    - `欣慰` ⇒ `an ủi`
    - `退休` ⇒ `giải ngũ`
    - `一个头两个大` ⇒ `hai thứ tóc`
    - `藏着掖着` ⇒ `giấu đầu lòi đuôi`
    - `纸醉金迷` ⇒ `vàng thau`
    - `欲哭无泪` ⇒ `dở khóc dở cười`
    - `翻.{0,2}白眼` ⇒ `trợn tròn`
    - `樱花` ⇒ `(?<!anh )hoa đào`
    - `老妇人` ⇒ `phụ thân`
    - `牵引` ⇒ `dắt mối`
  - Thêm vào `DEFAULT_RULES` các cụm chỉ cần xét out: `tội có thích đáng|(?<!\p{L})Hảo nào|Bất hẹn mà|nói mồm không bằng|ám muội nữa`.
  - Giới hạn của cách này: bảng chỉ chặn được những cặp đã gặp. Nên cho GUI một nút "thêm vào bảng bẫy" từ trang đọc chương.
- **Mở rộng kiểm glossary (P1, vừa):** khi raw có key nhóm names (≥2 chữ Hán) mà out không có target, và out lại có một cụm viết hoa ngoài glossary thì cảnh báo. Luật này bắt được "A Khang" và "cổ quốc Islan".
- **Khuôn (c) và (d): cần một bước soát nghĩa (P2, lớn).** Sau khi check pass, chọn ra các đoạn rủi ro:
  - raw có `被` kèm hai tên riêng;
  - câu hỏi tu từ `哪里还|难道|岂`;
  - thành ngữ 4 chữ nằm trong một danh sách thành ngữ;
  - số có `亿|万|倍`;
  - đoạn có tỉ lệ độ dài lệch.

  Gửi các cặp song ngữ có nhãn [[n]] cho một prompt soát chỉ trả JSON `{n, loại, sửa}`, rồi biến kết quả thành một vòng `review.md`. Không soát cả chương để giữ chi phí thấp.

### V4. Định dạng thoại đổi theo phiên 10 chương, prompt tự phá 『』 (P1, công sức nhỏ)

**Mô tả.** 36 lỗi DINH_DANG ở 32 truyện, 34 lỗi gắn HT. Ngoặc thẳng dồn thành từng khối liền nhau:

- cai-nay ch11–40
- cuu-thuc 0011–0020, 0051–0070
- tuyet-doi 0063–0072
- ky-tuc-xa 106–222

Khối 10 chương trùng đúng `chapters_per_session: 10` (`story_fs.rs:100`). Nghi mỗi phiên agent mới chọn lại kiểu ngoặc.

**Ví dụ:**

- am-thinh ch0386: `『她是故意让依莎看到的。』` → "“Nàng ta cố ý…”". Suy nghĩ thầm thành lời nói ra miệng; 750/2045 dấu 『』 bị mất (tôi đếm lại: out còn 1295 so với raw 2045).
- cai-nay ch0001: `“呃啊——！”` → "“Ách a — !”" (169 lần / 54 file).
- tam-muoi-tuoi ch0021: 1.446 dòng ở 82 file mở thoại bằng `"`.

**Nguyên nhân gốc (đã thấy trong prompt):**

- `ai-translation-prompt.ts:214`: "Ngoặc thoại `「」` / `『』` / `“”` đều thành `“”`". Chính prompt ra lệnh xoá 『』.
- `:215`: `——` thành ` — ` có khoảng trắng hai bên, nên `——！` ra "— !". Điều này trái với luật ngay dòng sau, `:216` "Không có khoảng trắng trước dấu câu".
- Luật "Không trộn hai kiểu ngoặc" (`:214`) không có check nào ép.
- Dấu sáu chấm là **thiết kế có chủ ý**: prompt `:212-213` và check `check.rs:127` đều bắt `……` → `......`. Reviewer ở ít nhất 10 truyện lại coi đây là lỗi; toàn thư viện có 122.102 cụm.

**Đề xuất:**

- **Chuẩn hoá tất định trong `run_accept`,** trước `format_translation` (`accept.rs:42`). Không giao việc này cho model:
  - Trong mỗi đoạn có số `"` chẵn, thay xen kẽ bằng `“`/`”`; số lẻ thì cảnh báo.
  - `\s+—\s+([!?.,])` → `—$1`.
  - Thêm luật bắt buộc vào `MANDATORY_RULES` (`check.rs:227`): `^"|\s"` → "Ngoặc thẳng".
- **Sửa prompt `:214`:**

  > "`「」`/`“”` thành `“”`; `『』` giữ nguyên `『』` (độc thoại nội tâm/trích dẫn đặc biệt) — không đổi thành ngoặc kép."

  Kèm luật check theo cặp: số `『` trong raw ≠ số `『` trong out thì báo lỗi. Sửa `:215`: "`——` đứng trước dấu câu thì viết liền: `——！` → `—!`".
- **Dấu sáu chấm là quyết định của user.** Nếu chọn "...", phải đổi đồng thời `ai-translation-prompt.ts:212-213`, `:315` và `check.rs:127` (cả golden ở `ai-translation.ts`), rồi chuẩn hoá hàng loạt trong out/. Kèm theo, sửa luôn lỗi viết hoa sau dấu lửng giữa câu (sao-the: 198 lần / 83 file).

### V5. Tiêu đề chương: viết hoa mọi chữ, thiếu ":", "ta" lệch giọng, lời xin phiếu (P1/P2)

**Mô tả.** 29 lỗi TIEU_DE ở 29 truyện. Tiêu đề viết hoa mọi chữ cũng đi theo lô phiên:

- di-lam-mo-ca 0958–0967, 1019–1028 ("đúng hai phiên 10 chương")
- bien-than-nu-dao-si 114–123
- moi-ngay 1043–1045
- thau-huong: 165/2501 tiêu đề

**Ví dụ:**

- moi-ngay 1044: → "Đột Phá, Dung Đạo Cảnh Trung Kỳ" ⇒ "Đột phá, Dung Đạo cảnh trung kỳ".
- tan-the-nguoi-khac 0119: `苏小苏，我最近表现不好？` → "dạo này ta thể hiện…" ⇒ "em".
- truong-sinh-la-meo 0076: → "(Cầu đặt mua đầu!!!)" lẫn vào tiêu đề.
- tuyet-doi-rung-dong: 69/87 tiêu đề dài hơn 80 ký tự, vì raw không có tên chương nên đoạn mở đầu dính vào dòng `第N章`.

**Nguyên nhân gốc:**

- Luật tiêu đề `ai-translation-prompt.ts:160` có mẫu `Chương N: Tiêu đề` nhưng **không nói gì về cách viết hoa**. Model viết hoa là do model (sự thật số 3).
- Chữ "ta" trong tiêu đề là thoại: do luật người kể "ta" (sự thật số 3). Thêm vào đó, luật check `check.rs:132` chạy trên cả dòng tiêu đề, vì tiêu đề là đoạn [[1]] của `final_text`, không có ngoặc kép. Nên "Để tôi!" bị ép thành "ta" ngay trong vòng review.
- Lời xin phiếu trong tiêu đề là do `:163`: lời tác giả "dịch nguyên văn".

**Đề xuất:**

- **Prompt `:160`:** thêm

  > "Tiêu đề viết hoa kiểu câu: chỉ chữ đầu và tên riêng/mục glossary. Tiêu đề là câu nói của nhân vật thì xưng hô như trong thoại của nhân vật đó. Bỏ phần ghi chú tác giả trong ngoặc (求票/求订阅/补字数/小修/加更)."

- **Code:**
  - `check.rs:132` bỏ qua dòng 1, và chỉ bật khi `style.voice` là ngôi một.
  - `accept` tự sửa `^Chương (\d+(?:-\d+)?)[.\s]+` → `Chương $1: `. Không áp cho truyện đã có quy ước khác (tien-tu-lai-xu-long dùng "619 …").
  - Check cảnh báo Title Case: `^Chương \d+: (?:\p{Lu}\S*\s+){3,}\p{Lu}`, trừ khi phần tên trùng một target trong glossary.
  - Kiểm số chương: xem V1.

  Công sức nhỏ.

### V6. Chương lời tác giả: dịch như truyện, làm bẩn glossary, có chỗ model bịa thêm (P2, vừa)

**Ví dụ:**

- bat-dau-ho-tong: 4 chương ngoài truyện, sinh glossary `起点→Qidian`.
- nam-nu-de ch0024.
- ma-mon ch191/233/238.
- truong-sinh-la-meo ch0075: raw 411 ký tự, out 2.628, tỉ lệ 6,4 (trung vị 3,75). Model bịa thêm khoảng 4 đoạn xin phiếu. Tôi đếm lại: out có 27 đoạn so với raw 21.

**Nguyên nhân gốc:**

- `:163` bắt dịch nguyên văn lời tác giả.
- Không có nhận diện nào: grep `感言|请假|上架` trong code không ra kết quả.
- `check` chỉ có ngưỡng tỉ lệ dưới (`commands/check.rs:151`, `min_length_ratio 0.75`); không có ngưỡng trên.

**Đề xuất:**

- `next` nhận diện tiêu đề raw khớp `上架感言|完本感言|请假|写在最后|新书|单章` và gợi ý `skip`, hoặc dịch theo preset "lời tác giả hiện đại".
- `accept` không trích glossary từ các chương này.
- Thêm ngưỡng trên: ratio > 2× trung vị của truyện, hoặc một đoạn dài hơn 8× đoạn raw và quá 200 ký tự, thì cảnh báo "nghi thêm ý". Ngưỡng này cũng bắt được kiểu độn chữ ở trong-sinh-cau.

### V7. Check bắn nhầm và luật "avoid" không được kiểm (P1, nhỏ)

**Ví dụ:** dinh-luu-van có 48 cảnh báo sai (sự thật từ digest); chương 3 chạy đủ 3 vòng review vì câu đúng "con ngươi đen láy".

**Nguyên nhân gốc:**

- `check.rs:128` `(?<!\p{L})(?:ngươi|…)` khớp với "con ngươi".
- `check.rs:132` áp cho mọi truyện modern, bất kể ngôi kể.
- `style.avoid` chỉ vào prompt (`prompt.rs:180`), không thành luật check. Vì vậy phuong-thon vẫn có 92 lần "mà thôi". "khẽ" thêm vô cớ cũng không được kiểm (xuong-nui: khoảng 103 dòng).

**Đề xuất:**

- Sửa `:128` thành `(?<!con\s)(?<!\p{L})(?:ngươi|…)`.
- Gắn `:132` với ngôi kể (xem V5).
- Tự sinh check rule từ mỗi mục `avoid` là cụm chữ thuần, ở mức cảnh báo.
- Luật raw-vắng: out có `(?<!\p{L})khẽ(?!\p{L})` mà raw cùng đoạn không có `轻|微|悄|低声|浅|淡` thì cảnh báo.

## 3. Lỗi lẻ đáng chú ý

- **Glossary chép nguyên vào truyện:** signature_phrases bị chèn nguyên văn làm gãy câu (bien-thanh-my ch0001); giá trị glossary chứa "/" hai phương án lọt vào truyện (lol ch0129 "Tung Tăng / Nhảy Múa").
  - **Đề xuất:** `sanitize_extracted` từ chối target có `/`.
- **Mục glossary sai** làm lẫn thực thể: `亚斯兰古国 => cổ quốc Islan`; 昆大 → "Côn Đảo".
- **Đạn mạc tự bọc thêm ngoặc kép:** 434 dòng (trong-sinh-meo).
- **Tin nhắn bọc hai lớp** "“【…】”" (tuyet-doi).
- **Sót ý nhỏ:** sót 女 trong `汝妻女吾养之`; sót 睡觉 trong ID khán giả; nói giảm câu chửi thô (xuyen-nhanh 0280).

## 4. Điều đang làm tốt, không sửa nhầm

- **Căn đoạn bằng nhãn [[n]] rất chắc:** chỉ 4/27.066 file lệch số đoạn, và 2 trong số đó là do raw drift. Nhờ vậy làm được luật theo cặp raw↔out ở V2/V3, và dùng số đoạn để phát hiện raw drift.
- **Phần lớn đầu ra đúng chuẩn:** 20.520 file dùng ngoặc cong đúng; 26.405 tiêu đề đúng mẫu "Chương N", chỉ 127 thiếu ":".
- **Nhiều chỗ dịch đúng:** 两百亿/八千亿 dịch đúng bậc (dinh-luu-van); 铁青 các chỗ khác dịch "tái mét"; 阿青 = "A Thanh" đúng 455/456 lần; 我玩尼玛 dịch đúng.
- **`titles.rs` ép tiêu đề lặp nhất quán:** không đụng vào khi không chắc. Giữ nguyên cách này; đề xuất ở V5 chỉ thêm, không thay.
- **Luật out-only kiểu "não hải", "vô ngữ" hoạt động** (không thấy tái phát trong digest). Bảng bẫy đồng âm ở V3 là phần mở rộng của cơ chế này.

## 5. Tóm tắt: check chặn được gì, cái gì cần soát

| Loại | Chặn tự động | Cách |
|---|---|---|
| Raw đổi sau khi dịch | Có | hash raw + so số chương/số đoạn (V1) |
| 亿/万/斤/倍/小时/三五 | Có (gần hết) | luật raw⇒out (V2) + prompt |
| Bạn giả Hán-Việt, thành ngữ ghép nhầm | Một phần (chỉ cặp đã biết) | bảng bẫy (V3a/b) |
| Tên lệch glossary khác chữ | Một phần | mở rộng glossary check |
| Ngoặc, 『』, "— !", ":" | Có, nên tự sửa | chuẩn hoá trong accept (V4/V5) |
| Title Case, "ta" trong tiêu đề | Cảnh báo | regex + bỏ dòng 1 khỏi luật ngôi kể |
| Thêm ý dài / bịa lời tác giả | Có | ngưỡng trên tỉ lệ độ dài (V6) |
| Đảo chủ thể, tu từ, meme, chơi chữ, lỗi gõ raw | Không | soát nghĩa có chọn lọc (V3, P2) |
