# Quy Tắc — hệ quy tắc mới thay Luật Nhân

Ngày: 2026-11-06. Trạng thái: chờ duyệt. Bản thử Python, bộ luật nháp và số đo nằm trong
[2026-11-06-quy-tac/](2026-11-06-quy-tac/).

## 1. Mục tiêu

Làm bản convert đọc xuôi hơn rõ rệt, nhanh hơn, và để người dùng tự viết quy tắc được mà
không cần biết regex. Luật Nhân bị bỏ khỏi runtime; ai có luật riêng thì chạy công cụ migrate.

Giữ giọng convert: hệ này chỉ đổi trật tự từ và chọn nghĩa theo ngữ cảnh. Nghĩa chỉ được chọn
trong các nghĩa VietPhrase đã có cho chính từ đó, không thêm từ mới (`事情` vẫn là "sự tình").
Mặc định chỉ đổi khi nghĩa đầu sai nghĩa trong ngữ cảnh đó (`刚刚` "vừa vặn" → "vừa"). Đổi theo văn
phong (`似乎` "tựa hồ" → "dường như") nằm trong nhóm `[giọng-mượt]`, tắt mặc định (mục 9).

## 2. Vì sao phải thay

Đo trên 2.700 chương bookshelf, 7 triệu ký tự:

| Vấn đề của Luật Nhân | Số đo |
|---|---|
| Tốn thời gian | ~49% thời gian dịch (3.043 so với 1.555 ns/ký tự) |
| Luật chết | 136/286 luật không khớp lần nào |
| Phủ hẹp | 2,4 lần khớp / 1.000 ký tự; trong đó 55% là `{h}{t}`, 16% là tiêu đề chương |
| Bỏ sót chỗ sai nhiều nhất | `X的Y` gặp 25,1 lần / 1.000 ký tự, Luật Nhân không xử lý |
| Liệt kê từng chữ | `第{s}X` khoảng 50 luật, `比{n}…` khoảng 15 luật |
| Bộ `{0}` bị lờ | vp-tra-sua (395 luật), vp-lac-mai-trang (269 luật) không được nạp |

## 3. Kết quả bản thử

Bộ test: 87.816 đoạn chưa dùng để viết luật. Mốc so sánh: output thô của convert hiện tại, có đủ
Luật Nhân. Tham chiếu: bản đã biên tập trong bookshelf (raw và out khớp từng đoạn ở 99,9% chương).

| | Cụm 2 từ trùng | Cụm 3 từ trùng |
|---|---|---|
| Tắt Luật Nhân | 603.005 | 283.117 |
| Convert hiện tại | 604.979 | 284.658 |
| Quy tắc mới, bản 2026-11-06 (161 quy tắc) | 628.191 (+3,84%) | 312.907 (+9,92%) |
| Quy tắc mới, gộp ý `rules.txt` (180 quy tắc) | 628.592 (+3,90%) | 313.136 (+10,00%) |
| Quy tắc mới, gộp thêm ý bộ luật số thứ hai (194 quy tắc) | 628.795 (+3,94%) | 313.379 (+10,09%) |
| **Mặc định**: thêm sửa nghĩa sai (204 quy tắc) | 630.946 (**+4,29%**) | 315.386 (**+10,79%**) |
| Mặc định + bật `[giọng-mượt]` (212 quy tắc) | 633.530 (+4,72%) | 317.419 (+11,51%) |

Bản mặc định: 43,7% số đoạn đổi, 17.598 đoạn tốt hơn, 3.647 đoạn kém hơn, 17.127 đoạn hoà. Tỉ lệ
thắng 82,8%. Cách gộp ở mục 8 và 9. Câu mẫu thời gian: [demo_thoi_gian.txt](2026-11-06-quy-tac/demo_thoi_gian.txt).
Toàn bộ Luật Nhân chỉ góp +0,5% cụm 3 từ. Thêm cả 174 luật `{n}` cũ vào bộ luật mới hầu như
không tăng điểm; bộ nháp chỉ giữ 24 luật đặc thù còn thắng.

Giới hạn của số đo:
- Đây là độ trùng cụm từ, số gián tiếp, không chứng minh câu đúng ngữ pháp.
- Bản biên tập viết lại nhiều, nên một phần ca "kém" thật ra ngang hoặc tốt hơn, và ngược lại.
- Bản thử lấy nghĩa khi tách cụm bằng tra từ điển đơn giản, khác engine thật. Bản Rust phải đo lại.
- Tốc độ và RAM chưa đo.

Câu mẫu, chạy thật qua bản thử ([demo_out.txt](2026-11-06-quy-tac/demo_out.txt)):

| Nguồn | Hiện tại | Mới |
|---|---|---|
| 他站在高台上 | Hắn đứng tại đài cao bên trên | Hắn đứng trên đài cao |
| 这是掌门的命令 | Đây là chưởng môn mệnh lệnh | Đây là mệnh lệnh của chưởng môn |
| 他走进来的时候 | Hắn đi tới thời điểm | Khi hắn đi tới |
| 站在她身后的女孩 | Đứng tại sau lưng nàng nữ hài | Nữ hài đứng sau lưng nàng |
| 第二层的第四道门 | Thứ hai tầng thứ bốn đạo cửa | Cửa thứ tư của tầng thứ hai |
| 2015年6月11日，晚上九点半 | Năm 2015 ngày 11 tháng 6, buổi tối chín giờ nửa | Ngày 11 tháng 6 năm 2015, chín giờ rưỡi tối |

## 4. Luồng xử lý

```
văn bản
  → chuẩn hoá                (như cũ)
  → nhận số, ngày giờ        (nhóm [số], quét trên ký tự)
  → tách từ, tra nghĩa       (vòng dịch hiện có, bỏ nhánh Luật Nhân, giữ {h}{t})
  → tách hư từ               (mục 7)
  → gắn nhãn                 (mục 6)
  → các nhóm quy tắc         (theo thứ tự trong file)
  → ráp câu                  (dùng lại TranslationOutput: viết hoa, khoảng trắng, range)
```

Vòng dịch không ghi thẳng ra chuỗi nữa mà sinh `Vec<Token>`. `Token` giữ chữ nguồn, nghĩa, nhãn,
khoảng nguồn. Bước ráp phát lại token qua `TranslationOutput`, nên quy tắc viết hoa và khoảng
trắng vẫn nằm một chỗ.

Mỗi nhóm quy tắc là một tầng. Mẫu khớp được thì cả khúc gộp thành một token mang nhãn kết quả,
tầng sau coi nó như một từ. Đây là chuỗi transducer hữu hạn nối tiếp: mỗi token chỉ tra bảng chỉ
mục theo chữ đầu và nhãn, không chạy regex, không có cửa sổ trượt.

## 5. Ngôn ngữ quy tắc

Một file text, `QuyTac.txt`. Mặc định nhúng trong binary; có file trong thư mục dữ liệu thì dùng
file đó; request có thể gửi bộ riêng.

```text
[lớp]
vị-trí: 上=trên 里=trong 中=trong 下=dưới 外=ngoài 中央=giữa
thân-vị: 身上=trên_người 手中=trong_tay 身后=sau_lưng 眼中=trong_mắt
buổi: 上午=sáng 下午=chiều 晚上=tối
đv-thời: 天=ngày 年=năm 分钟=phút 小时=tiếng

[số]
{số}年{số}月{số}(日|号) = ngày {3} tháng {2} năm {1|năm} -> thời-gian
{buổi}{số}点半 = {2} giờ rưỡi {1} -> thời-gian
百分之{số} = {1} phần trăm -> m
{số:1-3}{đv-thời}(后|之后|以后) = {1} {2} sau -> thời-gian

[của lặp=3]
{người} {thân-vị} = {thân-vị} {người} -> nơi-cụm
{tên} 的 {danh-từ}+ = {2} của {1}
{tính-từ} 的 {danh-từ}+ = {2} {1}
这个 {danh-từ}+ | !的 = {1} này
{cụm} {vị-trí} | !{danh-từ} = {vị-trí} {cụm} -> nơi-cụm

[khung]
比 {cụm} (更|还|要)? {tính-từ} = {tính-từ} hơn {cụm} -> tính-cụm
{vế-động} 的时候 = khi {vế-động}
连 {đoạn} (都|也) = ngay cả {đoạn} cũng

[nghĩa]
^ | 可是 | = Nhưng -> lit
在 {động-từ} = đang {động-từ} -> v-cụm
变得 = trở nên -> v

[giọng-mượt tắt]
似乎 = dường như -> d
```

### Cú pháp

Một dòng: `[trước |] mẫu [| sau] = khuôn [-> nhãn]`.

| Phần tử mẫu | Ý nghĩa |
|---|---|
| `的`, `的时候` | chữ cố định; khớp một token hoặc nhiều token liền nhau ghép đúng thành chữ đó |
| `(更\|还)`, `(更\|还)?` | một trong các chữ; `?` là tuỳ chọn |
| `{nhãn}` | một token có nhãn đó, hoặc thuộc lớp từ đó |
| `{nhãn}+` | 1 đến 3 token liền nhau |
| `{số:1-2}`, `{năm:3-4}` | chỉ trong `[số]`: khoá số giới hạn độ dài; năm / chữ số đọc từng chữ (mục 8) |
| `{vế}`, `{vế-động}` | khúc bên trái tới ranh giới vế; `-động` đòi có động từ |
| `{mệnh-đề}` | `[chủ ngữ] [phó từ] động từ [tân ngữ]`, dựng từ phải sang trái |
| `{đoạn}` | khúc ngắn nhất trong vế, tới phần tử kế tiếp |

Ngữ cảnh đặt ngoài hai dấu `|`, không bị thay: `^` đầu vế, `$` cuối vế, `!` phủ định.

Khuôn dùng `{1}`, `{2}` theo thứ tự khoá, hoặc tên nhãn khi nhãn đó chỉ xuất hiện một lần. Chữ cố
định không được đánh số. Tham chiếu tới khoá không tồn tại là lỗi khi nạp file, không im lặng
ra chuỗi rỗng.

Bộ lọc số: `{1|số}` chữ số, `{1|thứ}` số thứ tự (nhất, hai, tư), `{1|năm}` đọc từng chữ số năm.
Không lọc thì giữ kiểu chữ của nguồn (mục 8).

Ưu tiên trong một nhóm: mẫu dài hơn, rồi nhiều chữ cố định hơn, rồi thứ tự trong file. `lặp=N`
chạy lại nhóm tối đa N lần, để `A的B的C` và `这个美丽的女孩` gộp dần được. Số lượt tối đa là 20;
lớn hơn là lỗi khi nạp file, kèm số dòng (mỗi lượt quét lại cả đoạn). Nhóm có cờ `tắt`
(`[giọng-mượt tắt]`) không chạy, trừ khi request hoặc cấu hình bật tên nhóm đó.

File quy tắc và file từ loại (`dict.txt`, `TuLoai.txt`) mở đầu bằng BOM UTF-8 thì BOM được bỏ.

Dòng `mẫu = ` không có gì sau dấu `=` (dấu cách cuối bị cắt thành `mẫu =`) vẫn đọc được, như bản
thử: cả dòng là mẫu, tức `mẫu` rồi chữ cố định `=`, và khuôn rỗng. Nó không xoá chữ `mẫu`; muốn
bỏ một từ thì viết quy tắc cách khác.

Ví dụ kiêm kiểm thử, viết ngay dưới quy tắc:

```text
{tên} 的 {danh-từ}+ = {2} của {1}
# vd: 叶妩的手 => tay của Diệp Vũ
```

### Nhãn có sẵn

`tên`, `đại-từ`, `người` (tên hoặc đại từ), `danh-từ`, `cụm`, `động-từ`, `danh-động`, `hệ-từ`,
`tính-từ`, `thành-ngữ`, `số`, `số-lượng`, `thời-gian`, `nơi`, `nơi-cụm`, `phó-từ`, `dấu`.

## 6. Từ loại

Nguồn là `Resources/dict.txt` (kiểu jieba, 349k từ). Nguồn này nhiễu: `美丽`, `山峰` bị gắn địa
danh, `普通` bị gắn danh từ riêng. Thử suy từ loại thô từ ngữ cảnh đã bị bỏ vì gán sai cả hư từ
(`很`, `也是` thành danh từ). Cách đang dùng:

- Nhãn riêng của jieba mà nghĩa VietPhrase viết thường thì hạ xuống từ thường (8.944 từ có
  nhãn riêng mà nghĩa viết thường, 3.757 từ gặp trong corpus cần sửa).
- Trong nhóm đó, từ hay đứng sau `很/非常/太…` là tính từ.
- Token đến từ Names hoặc names của request luôn là `tên`, không phải đoán.
- Token ngoài bảng: suy từ các từ con. Có động từ thì là cụm động từ, không thì lấy từ cuối.
- Token kết thúc bằng `是` là `hệ-từ`. `时候`, `时` là từ khung, không phải danh từ.

Kết quả ghi vào `TuLoai.txt` (`từ<TAB>nhãn`), người dùng sửa được, đè lên `dict.txt`. Nhãn nằm
trong `CompactDict` bằng cách lấy 8 bit của độ dài value, nên không thêm RAM theo từng mục.

## 7. Tách hư từ

VietPhrase có rất nhiều cụm dính hư từ và đóng băng trật tự sai: `美丽的少女`, `高台上`, `她的`,
`的人，`, `都不知道`. Bước này tách chúng để quy tắc đảo được:

- Dấu câu dính cuối token: tách.
- `的` ở đầu, cuối, hoặc giữa token: tách, trừ `的时候`, `的确`, `目的`, `有的`...
- Đuôi vị trí có nghĩa dạng "… bên trên/bên trong/phía dưới": tách.
- Đầu `都/也` khi phần sau là từ có sẵn: tách, để khung `连…都` khớp được.
- Thành ngữ (`i`, `l`) không tách.

**Điều kiện bắt buộc:** chỉ tách khi nghĩa của cụm đúng bằng nghĩa từng phần ghép theo thứ tự tiếng
Trung. Cụm nào nghĩa khác đi tức là đã dịch tay theo trật tự tiếng Việt thì giữ nguyên
(`不好的预感` = "dự cảm không tốt"). Không có điều kiện này thì các cụm đó bị phá thật, đã thấy trong câu mẫu.

## 8. Số, ngày giờ

Nhóm `[số]` quét trên ký tự trước khi tách từ. Khoảng trắng quanh số được bỏ qua (`第 12 章`).

**Luật số thắng bước tách từ.** Một khúc được nhận khi nó bắt đầu ở ranh giới token và từ điển
không có cụm nào dài hơn bắt đầu cùng chỗ. Đuôi khúc được phép cắt giữa một token; phần dư tách lại
bằng từ điển cùng token kế tiếp. Ví dụ `三点十五分出发`: engine đang tách `分出|发` ("phân ra phát"),
luật giờ cắt ở `分` rồi tách lại `出发` thành "xuất phát". Còn `两点一线` có sẵn cụm trong từ điển
nên luật thập phân nhường lại.

Khoá số có thể giới hạn độ dài: `{số:1-2}` nghĩa là 1–2 ký tự số, để `<số>点` không nuốt `一点`
(một chút). Khoá `{năm:a-b}` chỉ nhận chữ số đơn và đọc từng chữ (`二〇一九` → 2019, phần thập phân
`一四` → một bốn). Khoá năm 3–4 ký tự chỉ nhận chữ số Hán, hoặc số Ả Rập trông như năm
(1500–2099), vì `500年`, `5000年` là khoảng thời gian "500 năm".

Kiểu viết lấy theo bản biên tập. Số Hán ra chữ (98,8% với số 1–10), số Ả Rập giữ nguyên. Câu mẫu
chạy thật: [demo_so.txt](2026-11-06-quy-tac/demo_so.txt).

| Nguồn | Ra | Bản biên tập |
|---|---|---|
| 三百五十岁 | ba trăm năm mươi tuổi | chữ 79%, số 7% |
| 二〇一九年, 2015年6月11日 | năm 2019, ngày 11 tháng 6 năm 2015 | năm + chữ số 88% |
| 晚上九点半, 三点一刻 | chín giờ rưỡi tối, ba giờ mười lăm phút | |
| 三个小时, 半个小时 | ba tiếng, nửa tiếng | tiếng 88% |
| 百分之五十 | năm mươi phần trăm | chữ 92%, `%` 6% |
| 三点一四, 百分之三点五 | ba phẩy một bốn, ba phẩy năm phần trăm | |
| 第四层 | tầng thứ tư | tầng thứ N 90% |
| 第一次, 第一个 + danh từ | lần đầu, … đầu tiên | 90%, 81% |
| 第二天 | hôm sau | 83% |
| 三块钱, 五元 | ba tệ, năm tệ | tệ 96% |
| 十两银子 | mười lượng bạc | 100% |
| 三倍, 十多岁 | gấp ba lần, hơn mười tuổi | gấp 61% |
| 三公里, 五厘米 | ba cây số, năm xen-ti-mét | cây số 79%, xen-ti-mét 100% |
| 第12章 | Chương 12: | |

Hai chỗ giữ nguyên nghĩa VietPhrase vì bản biên tập theo nó: `三阶` = tam giai (80%), `下品灵石`
= hạ phẩm linh thạch (76%). Lượng từ giữ lại: `五百块灵石` = năm trăm khối linh thạch (86%).

`万` đọc là vạn, `亿` là ức, `千` là ngàn. Bộ lọc: `{1|số}` chữ số, `{1|thứ}` số thứ tự,
`{1|năm}` đọc năm, `{2|từng-chữ}` đọc từng chữ số. Dùng lại `number.rs`.

### Tham khảo `rules.txt` (2026-11-07)

`rules.txt` là bộ 102 luật số / đơn vị / tiền trên ký tự, của người khác. Tao chuyển nguyên bộ
sang cú pháp này rồi đo từng luật trên dev (96.369 đoạn):

| Cách nạp | Luật chạy được | Đoạn thắng | Đoạn thua | Cụm 3 từ, cả bộ |
|---|---|---|---|---|
| Giữ nguyên, ra số Ả Rập | 64 | 83 | 2.289 | +10,75% → +10,01% |
| Đổi số sang kiểu của nguồn | 64 | 194 | 404 | +10,75% → +10,64% |

Thua chủ yếu vì quy ước đầu ra khác bản biên tập: số Ả Rập (`三十六岁` → "36 tuổi"), bỏ lượng
từ (`块灵石`), `km`, `cm`, "tầng N", "giai N", `块钱` = "đồng". Ngoài ra `<n>个人` và `<n>天`
làm hỏng `一个人` (một mình) và `一天` (một ngày nọ).

Những thứ đã lấy:
- **Ý tưởng:** khoá có giới hạn độ dài, khoá năm đọc từng chữ, luật số thắng bước tách từ nhưng
  nhường cho cụm từ điển dài hơn, sau `点` bắt buộc có đơn vị, mục ghi rõ luật nào không bật mặc
  định và vì sao.
- **Luật, viết lại đầu ra theo bản biên tập:** thập phân, phần trăm thập phân, `点一刻/三刻/整`,
  `小时` = tiếng, `元/块钱` = tệ, `两银子/白银` = lượng bạc, `多岁/来岁/岁左右`, `倍`, `公里`,
  `厘米`, `亩`, `个多月`, đồng vàng / đồng bạc / đồng tiền / văn tiền / quan tiền / đĩnh bạc.

Không lấy: cú pháp `<a|b>` lựa chọn có thứ tự (chưa có ca nào cần), và các khoá theo từ điển
`<ne>/<pn>/<vp>/<hv>`, vì `{tên}`, `{đại-từ}`, `{cụm}` đã bao các khoá đó.

Sau khi gộp, tao bỏ luật số nào trên dev có thắng ít hơn thua, dưới 60%, hoặc chỉ toàn hoà.
Bỏ cả `第?{số}号` (20 thắng / 80 thua), `{số}个月`, `几十`… (VietPhrase đã dịch giống) và
`{số-lượng} 的 {danh-từ}` (57%).

### Tham khảo bộ luật số thứ hai (2026-11-07)

Trong `corpus/rules/` có thêm hai file luật số / đơn vị / thời gian, một bản đầy đủ và một bản bỏ
187 luật tên/đại từ. Chuyển nguyên bộ rồi đo trên dev, nền là bản đã gộp `rules.txt` (cụm 3 từ
+10,89%):

| Cách nạp | Luật chạy được | Thắng | Thua | Cụm 3 từ |
|---|---|---|---|---|
| Giữ nguyên, số Ả Rập | 110 / 587 luật số | 91 | 1.215 | +10,42% |
| Đổi số sang kiểu nguồn | 110 / 587 luật số | 181 | 440 | +10,81% |
| Luật tên/đại từ | **10 / 185** | 16 | 6 | |

477 luật số không khớp lần nào. Phần lớn là bản lặp cho chữ phồn thể (`點`, `個`, `時`), mà engine
đã đổi phồn sang giản trước. Phần còn lại là tổ hợp kích thước (`长N米宽N米高N米`). 175 luật câu cứng
kiểu `<ne|pn>妻子现在的情况怎么样了` không khớp lần nào, đúng như Luật Nhân.

Lấy ý, viết lại đầu ra (dev: thắng / thua):

| Luật | Ví dụ | Dev |
|---|---|---|
| `{số}{đv-thời}(后\|之后\|以后)` | 五分钟后 → năm phút sau | 54 / 2 |
| `{số}{đv-thời}(前\|之前\|以前)` | 三天前 → ba ngày trước | 27 / 3 |
| `{số}(多\|余){đv-thời}`, `…多` | 十多天 → hơn mười ngày | 24 / 5 |
| `{số}{đv-thời}(内\|之内)` | 三年之内 → trong vòng ba năm | 15 / 0 |
| `{số}{đv-thời}左右` | 三天左右 → khoảng ba ngày | 4 / 0 |
| `{buổi}{số}点多` | 晚上九点多钟 → hơn chín giờ tối | 6 / 0 |
| `{số}点到{số}点`, khoảng ngày | 三点到五点 → từ ba giờ đến năm giờ | 2 / 0 |
| `个半小时`, `个半月`, `天半` | 一个半小时 → một tiếng rưỡi | 5 / 0 |
| `四周`, `周围`, `附近` vào lớp vị-trí | 城墙周围 → xung quanh tường thành | +30 / +13 |
| `公元前{số}年` | năm hai trăm trước Công nguyên | |

Lớp `đv-thời`: ngày, năm, tháng, tuần, giây, phút, tiếng, canh giờ, thế kỷ, quý, ngày đêm. Luật
trần `{số}{đv-thời}` thua (45 / 136) nên không đưa vào.

Lấy thêm một quy tắc của engine từ phần đầu bộ này (bản thử chưa có): khoá số không được bắt đầu
giữa một chuỗi số dài hơn, để `三十二点` không khớp thành `十二点`.

### Tham khảo danh sách tên và thành ngữ (2026-11-07)

**Chinese-Names-Corpus** (Apache-2.0). Đã tải về đo, chưa đưa vào repo:
- **Danh sách tên** (1,2 triệu tên Trung, 25 vạn tên cổ, 42 vạn tên phiên âm, 17 vạn tên Nhật):
  không dùng để gắn nhãn. Trong các token chưa phải tên mà có trong danh sách, khoảng 90% là từ
  thường (`包括`, `自然`, `可能`). Tên thật thì nghĩa VietPhrase đã viết hoa, là tín hiệu đang dùng.
  Có thể có ích cho `/names/filter`, nhưng đó là việc khác.
- **Thành ngữ** (4,7 vạn mục): gắn lại nhãn thành ngữ cho 24.323 mục đang bị jieba gắn danh từ hoặc
  tên riêng, rồi đo trên test: số cụm 3 từ trùng gần như không đổi (giảm 5 trên 313 nghìn). Không đưa vào.

**ChineseNLPCorpus:** dữ liệu không nằm trong repo (phải email xin tác giả), không ghi giấy phép.
Không dùng.

## 9. Chọn nghĩa theo ngữ cảnh

Sau khi đã có các nhóm đảo trật tự, tao cho máy thử đổi chỗ mọi cặp token kề nhau trên 20.000 đoạn
dev. Chỉ còn 4 loại cặp thắng ≥ 70%, toàn ca lẻ, tức là phần đảo ngắn gần hết dư địa. Phần còn lại
nằm ở nghĩa: 485 từ hay gặp mà VietPhrase có sẵn một nghĩa khác hợp bản biên tập hơn nghĩa đầu.

Luật chọn nghĩa nằm trong nhóm `[nghĩa]`, chỉ dùng nghĩa đã có trong VietPhrase của từ đó. Hai nhóm:

**Sửa nghĩa sai, bật mặc định.** Nghĩa đầu sai nghĩa trong ngữ cảnh đó (dev: thắng / thua):

| Luật | Hiện tại → mới | Dev |
|---|---|---|
| `看着` | nhìn xem → nhìn | 653 / 2 |
| `变得` | thay đổi đến → trở nên | 312 / 0 |
| `正 + động từ`, `正在` | ngay tại → đang | 239 / 2 |
| `力量` | lực lượng → sức mạnh | 150 / 6 |
| `谁` | người nào → ai | 133 / 4 |
| `既然` | tất nhiên → đã | 127 / 0 |
| `身上` | trên thân → trên người | 120 / 9 |
| `刚刚` | vừa vặn → vừa | 39 / 1 |
| `động từ + 出来` | … đi ra → … ra | 26 / 3 |

Đây là nhóm lời nhất sau `的`. Số trên test ở mục 3.

**Văn phong `[giọng-mượt tắt]`**, file [QuyTac-giong-muot.txt](2026-11-06-quy-tac/QuyTac-giong-muot.txt).
Số trên test khi bật ở mục 3.

| Luật | Hiện tại → mới | Dev |
|---|---|---|
| `似乎` | tựa hồ → dường như | 487 / 11 |
| `而且` | mà còn → hơn nữa | 522 / 23 |
| `此刻` | giờ phút này → lúc này | 490 / 44 |
| `一股` | một cỗ → một luồng | 172 / 16 |
| `仿佛` | phảng phất → như | 147 / 29 |
| `依旧` | vẫn như cũ → vẫn | 134 / 14 |
| `原本` | nguyên bản → vốn | 81 / 1 |
| `眼睛` | con mắt → mắt | 19 / 3 |

Đã thử và bỏ vì thua: `什么` → "gì" (30%), `但是` → "nhưng" (51%), `里`/`上` đứng một mình (31% / 43%),
`这` + `是` → "đây" (63%), `怎么` → "sao" (68%), `{người} 的` cuối câu → "của …" (52%).

Mới chọn tay khoảng 20 trong 485 từ. Phần còn lại giao cho `qt quy-tac dao` (mục 11): máy sinh
luật, mày duyệt, rồi phân vào nhóm sửa nghĩa sai hoặc `[giọng-mượt]`.

## 10. Tích hợp

**Chuẩn hoá NFC khi nạp từ điển.** QT2025 có 11.071 dòng VietPhrase và 150 dòng Names không ở dạng
NFC. Vì vậy `也` → "cũng", `能` → "có thể", `得` → "đến" xuất ra dạng dấu tách rời (0,92% token ở
dev). Nhìn thì giống hệt, nhưng tìm, thay và so chuỗi đều hụt, kể cả bước kiểm của harness AI. Mọi
nghĩa được chuẩn hoá NFC lúc nạp, cả từ điển gửi kèm request. Việc này độc lập với Quy Tắc. Nó đổi
byte đầu ra, nên kiểm thử đối chiếu QT phải so sau khi chuẩn hoá NFC.

**Engine.** `Options` thêm `quy_tac: bool` (mặc định bật), `tat_nhom: Vec<String>` và
`bat_nhom: Vec<String>` (bật nhóm có cờ `tắt`, ví dụ `giọng-mượt`).
`vietphrase-one` chạy đủ. `vietphrase` nhiều nghĩa chỉ chạy `[số]` và `{h}{t}`. `hanviet` không chạy.
`Engine` và `LiteEngine` dùng chung bộ máy.

**Bỏ Luật Nhân.** Xoá `luat_nhan.rs` và phụ thuộc `fancy-regex`. Hàm `{h}{t}` giữ lại, chuyển vào
vòng tách từ. `scan_range` mặc định xem lại vì lý do giữ 30 chỉ còn là Luật Nhân.

**Migrate.** `qt quy-tac migrate LuatNhan.txt` đổi `{n}`/`{0}` thành `{người}`, `{s}` thành `{số}`,
khai triển `[x]?`, `(a|b)`. Trong một bản phát hành, API vẫn nhận `dictionaries.luatNhan` và tự
migrate khi nhận request; sau đó bỏ. `/dictionaries/defaults` trả `quyTac` thay `luatNhan`.

**API.** `dictionaries.quyTac`, `dictionaries.tuLoai`, `options.quyTac`, `options.tatNhom`,
`options.batNhom`. Quy tắc sai cú pháp trả 400 kèm số dòng.

**Range.** Sau khi đảo, range vẫn là cặp song song, xếp theo thứ tự đích, nên range nguồn không còn
tăng dần. `apps/qt-web/src/lib/ranges.ts` và bản qt-gui phải sắp một bản sao theo nguồn trước khi
dựng đoạn cho khung nguồn. Chữ trong khuôn lấy range của chữ cố định đầu tiên của mẫu; mẫu không
có chữ cố định thì lấy cả khúc.

**App.** Hộp thoại từ điển đổi tab Luật Nhân thành Quy Tắc. Dòng vẫn dạng `mẫu = khuôn` nên bảng
sửa hiện có dùng lại được. Cài đặt có công tắc "Giọng mượt" bật nhóm `giọng-mượt`.

**Không đổi.** qt-ai-core không phụ thuộc qt-core, nên harness AI không bị ảnh hưởng.

## 11. Công cụ

| Lệnh | Việc |
|---|---|
| `qt quy-tac kiem-tra` | nạp file, báo lỗi cú pháp, chạy mọi dòng `# vd:` |
| `qt quy-tac giai "câu"` | in token, nhãn, quy tắc nào chạy theo thứ tự |
| `qt quy-tac do --books DIR` | bảng thắng/thua từng quy tắc: bỏ riêng quy tắc đó ra rồi so với bản biên tập |
| `qt quy-tac dao --books DIR` | đào luật chọn nghĩa (bên dưới) |
| `qt quy-tac migrate FILE` | đổi Luật Nhân sang quy tắc mới |
| `qt quy-tac tu-loai` | sinh `TuLoai.txt` |

`do` chính là thứ làm hệ này dễ dùng: viết thêm luật, chạy một lệnh, biết luật đó được hay mất.

**`dao`.** Chia bookshelf thành dev / test theo chương. Lấy từ gặp ≥ 80 lần mà có ≥ 2 nghĩa
VietPhrase. Với mỗi nghĩa khác nghĩa đầu, sinh luật trần (`X = nghĩa`) và luật theo ngữ cảnh: trước
động từ, trước danh từ, đầu vế, cuối vế. Đo từng luật trên dev bằng cách bỏ riêng luật đó ra như
`do`. Giữ luật có ≥ 40 đoạn phân định và thắng ≥ 80%. Mỗi từ chỉ giữ luật hẹp nhất còn đạt, rồi xác
nhận lại trên test. Kết quả ra TSV: luật, thắng / thua, 3 câu ví dụ, và nghĩa đầu có mặt trong bản
biên tập bao nhiêu phần trăm. Máy không tự quyết luật nào là sửa nghĩa sai hay văn phong; mày duyệt
và đánh dấu từng dòng, luật được duyệt mới vào file.

## 12. Hiệu năng

Cổng phải qua. Lý do kỳ vọng: bỏ regex gỡ đi phần 49% thời gian, còn các tầng chỉ tra bảng theo
token. Số đo thật ở bước 1 nằm dưới bảng.

| Chỉ tiêu | Cổng |
|---|---|
| Thời gian dịch | không quá 100% hiện tại, cùng corpus, có names (ban đầu 70%, xem dưới) |
| RAM lite idle | tăng không quá 10 MB |
| Đỉnh RAM mỗi request | tăng không quá 20% |

**Số đo bước 1 (2026-11-07).** Đo bằng `cargo run --release -p qt-core --example quy_tac_eval -- QT2025
<books/common>` trên bộ test bookshelf (12 chương mỗi truyện, từ chương thứ 8), dịch từng đoạn, có
names riêng của truyện. Tốc độ đo công bằng: mỗi đường một lượt riêng qua mọi đoạn, 8 lượt đổi thứ
tự xen kẽ, cổng lấy tổng thời gian các lượt:

| | Hiện tại (có Luật Nhân) | Quy Tắc | Cổng |
|---|---|---|---|
| Đoạn / ký tự | 104.633 / 2.820.178 | | |
| Cụm 2 từ trùng bản biên tập | 726.262 | 756.873 (+4,21%) | |
| Cụm 3 từ trùng bản biên tập | 340.401 | 376.809 (+10,70%) | ≥ +10,0%: đạt |
| Đoạn đổi / tốt hơn / kém hơn | | 46.555 / 20.652 / 4.228, thắng 83,0% | ≥ 80%: đạt |
| Thời gian (tổng 8 lượt) | 2.566 ns/ký tự | 2.353 ns/ký tự (91,7%; trung vị 91,1%) | ≤ 100%: đạt |
| RAM `QuyTac` giữ sau khi nạp | | 9.397.215 byte (9,0 MiB) | ≤ 10 MiB: đạt |

- Bookshelf đang được sửa nên số đoạn đổi theo ngày. Bản chụp cố định 103.803 đoạn (2.798.883 ký
  tự) cho cụm 3 từ +10,70%, thắng 83,0%, và output y nguyên từng byte qua mọi lượt tối ưu.
- Các số 87%, 91% (và 206% ở bản đầu) đo trước đây là bộ đo ghép đôi bị lệch: mỗi đoạn dịch đường
  hiện tại rồi Quy Tắc ngay sau, nên Quy Tắc luôn chạy trên cache đã ấm; allocator đếm byte còn cộng
  một lần trên mọi lần cấp phát, phạt đường cấp phát nhiều hơn. Đo công bằng, bản trước lượt tối
  ưu thứ ba ra 104,6–106,5%, chưa qua cổng.
- Bản đầu tiên giữ 11,4 MiB. Lượt tối ưu một và hai: lọc chữ cố định một lần mỗi nhóm, bit khoá
  trên token, lọc regex `[số]`, bảng từ loại tự dựng với nhãn gộp. Lượt ba: token mượn chuỗi thay
  vì chép, bỏ lượt nhóm mà không quy tắc nào đủ khoá bắt buộc, bớt tra bảng từ loại, ráp câu một
  lượt quét. Mỗi bước so dump output của mọi đoạn với bản trước, giống nhau từng byte.
- Cổng 70% dựa trên giả định các tầng quy tắc chỉ tốn nửa phần Luật Nhân. Thực tế, sau tối ưu, chúng
  vẫn tốn khoảng 84% phần đó, và phần vòng dịch dùng chung đã chiếm ~52% đường hiện tại. Không xuống
  70% được mà giữ nguyên output, nên user đổi cổng thành "không chậm hơn đường hiện tại" (2026-11-07).
- Đỉnh RAM mỗi request chưa đo, để bước 4 khi nối lite / Lambda. Riêng lúc nạp, bảng từ loại cấp phát
  đỉnh ~5,9 MB.
- Số tuyệt đối dao động theo tải máy (cùng code từng đo 4.168 ns/ký tự cho đường hiện tại); tỉ lệ ổn
  định hơn.

## 13. Kiểm thử

- Kiểm thử đơn vị cho bộ phân tích cú pháp, khớp mẫu, bộ lọc số, tách hư từ.
- Mọi dòng `# vd:` chạy trong `cargo test`.
- Bộ dev để viết luật, bộ test để báo số; không chỉnh luật theo bộ test.
- Đối chiếu Rust với bản thử Python, rồi đo lại toàn bộ bằng `qt quy-tac do`.
- Đo cả trên `QT-hop-nhat`, không chỉ QT2025.
- Câu mẫu cố định đọc bằng mắt mỗi lần đổi bộ máy. Số tổng từng che các lỗi như
  `这是掌门的命令` → "Mệnh lệnh đây là chưởng môn", nên không được chỉ nhìn số.
- Kiểm thử đang khoá output Luật Nhân viết lại; kiểm thử đối chiếu QT chạy với `quy_tac: false`.

## 14. Các bước

0. **Chuẩn hoá NFC khi nạp từ điển.** Độc lập, làm trước được.
1. **Bộ máy trong qt-core, mặc định tắt.** Token, từ loại, tách hư từ, phân tích quy tắc, khớp mẫu,
   nhóm `[số]`, nhóm có cờ `tắt`. Luật Nhân vẫn còn. Cổng: khớp bản thử, qua cổng hiệu năng.
2. **Công cụ CLI** (`kiem-tra`, `giai`, `do`, `migrate`, `tu-loai`) và bộ luật mặc định, đo lại trên
   bộ test.
3. **`qt quy-tac dao`**, chạy một lượt, mày duyệt danh sách, luật được duyệt vào file.
4. **Bật mặc định**, nối API, lite, Lambda, app, sửa range, công tắc giọng mượt.
5. **Xoá Luật Nhân** và `fancy-regex`, cập nhật docs.

Mỗi bước commit riêng khi được bảo. Deploy là việc riêng.

## 15. Rủi ro và giới hạn đã biết

- **Đảo sai khó đọc hơn để nguyên.** Tắt được toàn bộ hoặc từng nhóm. Luật dưới 60% không vào mặc định.
- **Mệnh đề dài chưa ổn.** `{mệnh-đề}` thắng khoảng 68%. `V + N的N` nhập nhằng, mặc định dùng "của"
  (coi là mệnh đề chỉ thắng 36%).
- **Chưa làm:** đảo `把` (thắng 61%), `的话` (22%), `怎么 + động từ` (50%), `出了宝塔后` vẫn sai.
- **Ranh giới sửa nghĩa sai / văn phong là phán đoán.** Ví dụ `看着` "nhìn xem" → "nhìn" tao xếp vào
  sửa nghĩa sai. Mày đổi nhóm được bằng cách chuyển dòng giữa hai file.
- **Từ loại còn nhiễu.** Có thể soát thêm `TuLoai.txt` bằng subagent, nhưng tốn và phải hỏi trước.
- **Bộ từ điển khác cho kết quả khác** vì tách cụm phụ thuộc VietPhrase.
- **Bộ luật viết theo văn phong bookshelf**, thể loại khác có thể khác.
