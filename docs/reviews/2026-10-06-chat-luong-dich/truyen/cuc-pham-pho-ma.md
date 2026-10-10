# Review: Cực Phẩm Phò Mã (cuc-pham-pho-ma)

- **Setting trong story.json:** ancient
- **Thể loại thực tế:** lịch sử giả tưởng triều Đường thời Võ Tắc Thiên, xuyên không (đặc chủng hiện đại), quyền mưu, quân sự, hậu cung, có cảnh nóng
- **Ngôi kể:** ngôi 3 (lời bạt cuối là ngôi một của tác giả)
- **Số chương đã dịch:** 1142
- **Chương đã đọc:**
  - 1: Lam Điền công tử
  - 571: 诈
  - 1137 (đọc khoảng 70%)
  - 1138: chương kết
  - 写在最后: lời bạt

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 7.0 | Cuối truyện dùng chàng–thiếp ổn. Nhưng ở ch.82–159, Thái Bình gọi "Tiết lang" lúc "chàng" lúc "ngươi". Glossary addressing chốt xưng hô từ lần gặp đầu. 你们 thì thành "các ngươi" một cách máy móc. |
| Giọng văn | 8.3 | Giọng Đường triều trôi chảy, lễ nghi cung đình chuẩn, giọng từng nhân vật phân biệt được. |
| Dễ hiểu | 8.0 | Ít văn convert. Lỗi rải rác do 自己 → "bản thân" và vài câu calque. |
| Khó khăn | 7.0 | Thơ có vần, lóng xử lý tốt. Nhưng "Ân" phủ khắp, thư/thành ngữ để nguyên Hán Việt. |
| Thuật ngữ | 7.3 | Tên riêng nhất quán. Glossary phình to, có mục Hán Việt nặng, phủ công chúa dịch theo hai trật tự. |

## Lỗi chính (nặng → nhẹ)

1. **XUNG_HO_TROI** (cao, hệ thống: 20 lần/7 file, ch.82–159)
   - Raw: 薛郎，我很奇怪…你初涉官场
   - Hiện tại: "Tiết lang, ta rất lấy làm lạ… Ngươi mới bước chân vào chốn quan trường"
   - Đề xuất: "Tiết lang, thiếp lạ lắm… Chàng mới…"
   - Vì sao: ch.94 đã dùng "chàng", sang ch.110/155/159 lại quay về "ngươi". Không có cast.json nên không có mốc xưng hô.
2. **CAST_MAU_THUAN** (cao, hệ thống)
   - Glossary addressing có 655 cặp "A→B", mỗi cặp chỉ một giá trị, ví dụ:
     - 太平公主→薛绍 = "bổn cung–ngươi"
     - 上官婉儿→薛绍 = "ta–Tiết công tử"
     - 薛绍→上官婉儿 = "ta–Thượng Quan đại nhân"
   - Đến chương cuối, model phải tự làm trái glossary. Chỗ nào làm theo thì lệch: Uyển Nhi nghĩ "Ta muốn có con!" rồi vài dòng sau lại xưng "thiếp".
3. **AN_THAN_TU** (vừa, hệ thống: 110/141 câu thoại mở bằng 嗯; 108 lần/95 file)
   - Hiện tại: "“Ân......” Thái Bình công chúa chớp mắt kéo dài một giọng dài thượt"
   - Đề xuất: "“Ừm…” … kéo dài giọng"
   - Vì sao: làm khô cảnh tình tứ ở chương kết; câu còn lặp từ.
4. **XUNG_HO_VAI_VE** (vừa, hệ thống: ~6 lần)
   - Raw: 在你们二人之间
   - Hiện tại: "giữa hai người các ngươi, chàng sẽ làm thế nào?"
   - Vì sao: vợ nói với chồng mà cùng một câu vừa "chàng" vừa "các ngươi".
5. **BOI_CANH_SAI** (vừa, lẻ: lời bạt)
   - Hiện tại: "cảm ơn các ngươi đã theo sát và chở che"
   - Đề xuất: "cảm ơn các bạn"
   - Vì sao: lời bạt hiện đại vẫn bị khoá setting ancient. Xưng "ta" thì đúng sở thích.
6. **BAN_THAN** (vừa, hệ thống: "bản thân hắn/nàng…" 259 lần/217 file)
   - Raw: 必须逼着自己来决定
   - Hiện tại: "muốn ép bản thân phải tự mình quyết định"
   - Đề xuất: "là muốn ép hắn tự quyết"
   - Vì sao: sai chủ thể, đọc ra là Tiết–Thái Bình tự ép mình.
7. **GLOSSARY_HAN_VIET_NANG** (vừa, hệ thống)
   - Raw: 思君，念君，盼君
   - Hiện tại: "Tư quân, niệm quân, phán quân."
   - Đề xuất: "Nhớ chàng, thương chàng, mong chàng."
   - Cùng kiểu: "bào trạch" 19 lần/10 file, "dụng tâm lương khổ" 9, "tâm ngoan thủ lạt" 8, "cao xứ bất thắng hàn".
8. **DINH_DANG** (vừa, hệ thống)
   - 329 file có ngoặc thẳng "…" (176 file toàn bộ, 153 file lẫn), trong khi raw dùng ngoặc “…” 100%.
9. **THUAT_NGU_TROI** (thấp, hệ thống)
   - "Thái Bình công chúa phủ" xuất hiện 22 lần/13 file, trong khi glossary đã chốt "phủ Thái Bình công chúa" (146 lần).
10. **HU_HA** (thấp, hệ thống: 6 lần)
    - Raw: “呵！”李治笑了一声
    - Hiện tại: "“Hừ!” Lý Trị cười một tiếng"
    - Vì sao: "Hừ" mâu thuẫn với "cười".
11. **DAI_TU_MO_HO** (thấp)
    - Ch.1137 dùng "người" làm đại từ cho Võ Tắc Thiên: "triều đường của người"; ch.571 lại dùng "bà".
12. **SAI_NGHIA và CALQUE lẻ** (thấp)
    - 交待 → "một lời ăn tiếng nói"
    - 以此为记 → "Lấy đây làm ghi nhớ"
    - 你好自为之 → "hãy tự giải quyết cho ổn thỏa"
    - 团扇诗 → "khúc quạt đời"

## Gợi ý

- **Xưng hô theo mốc:**
  - Lập cast.json có mốc xưng hô cho các cặp chính, hoặc đổi schema addressing để một cặp có nhiều giá trị theo chương.
  - Luật trong prompt: 你们 phải theo xưng hô ngôi hai mà người nói đang dùng.
- **Thán từ:**
  - Bỏ ép 嗯 → "Ân"; cho phép "Ừm/Ừ" theo ngữ cảnh.
  - 呵 đi với 笑 thì dịch là tiếng cười, không phải "Hừ".
- **Hán Việt:** thư, thơ, thành ngữ mà người đọc phải hiểu nghĩa thì dịch nghĩa thuần Việt. Lọc glossary: "bào trạch" → "chiến hữu".
- **Chương ngoài truyện:** nhận diện lời bạt, cảm ngôn và bỏ khoá setting cổ đại cho các chương này.
- **Hậu xử lý trong app:**
  - Chuẩn hoá ngoặc thẳng sang ngoặc cong; 《》 sang ngoặc kép.
  - Thêm checkRule bắt "Thái Bình công chúa phủ" và "“Hừ!” … cười".
- **自己:** dịch theo chủ thể thật (hắn, mình, chính mình), tránh "bản thân" khi đổi chủ ngữ.

## Làm tốt

- Văn mượt, có không khí Đường triều. Cảnh Võ Tắc Thiên gài bẫy Lý Hiển (ch.571) xưng hô và nhịp rất tốt.
- Thơ có vần; thành ngữ được Việt hoá đúng chỗ.
- Cảnh nóng và lóng hiện đại dịch trung thành mà tự nhiên: mây mưa, thận hư, chấm mỹ nhân.
- 0 chữ Hán sót; số đoạn khớp raw 100%.
- Tên riêng nhất quán suốt 1142 chương; các biến thể lỗi gõ trong raw được gộp đúng.
- Cuối truyện, cặp chính dùng chàng–thiếp ấm áp, tự nhiên.
