# Review: Nhường Ngươi Thổ Lộ, Ngươi Tìm Tới Hắc Đạo Thiên Kim?

**Thể loại & cách đọc:** setting `modern`, kể ngôi ba; đô thị hài hước kết hợp hệ thống và hắc đạo/thế gia. Đã dịch 719 chương. Đọc trọn 4 chương 0001, 0360, 0718, 0719 (0718 và 0719 có mtime mới nhất), rồi grep toàn bộ out/ để đếm các lỗi lặp. "dòng" là chỉ số đoạn không rỗng, tính từ 0 (tiêu đề là 0).

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 6.0 | Cặp chính Tô Giang–An Nhu kẹt ở mốc chương 1, nên lấy nhau rồi vẫn lúc tớ–cậu, lúc anh–em |
| Giọng văn | 7.5 | Giọng đô thị hài, khẩu ngữ tốt; còn vài vết cổ trang và tên skill Hán Việt nặng |
| Dễ hiểu | 8.0 | Trôi chảy; rải rác văn convert ("đem…", "Của cậu không phải là vấn đề…") |
| Khó khăn | 7.5 | Chửi thề, thán từ, cà khịa dịch sống; meme mạng thì bị phiên âm hoặc dịch nghĩa đen |
| Thuật ngữ | 5.5 | 督察局 ra 3 tên, glossary tự mâu thuẫn, "Vệ gia" lẫn "nhà họ X", phiên âm sai |

## Lỗi (nặng → nhẹ)

1. **XUNG_HO_TROI (cao, hệ thống)**: Tô Giang và An Nhu. Ở 0718 đ.108: 我作为你的妻子… → "cậu có từng nghĩ… **tớ** với tư cách là vợ **cậu**…", vậy mà sang đ.114 đã thành "Là do **em** chọn mà, bà xã". Ở 0719 đ.17: "ông xã thiên tài của **tôi**, **cậu** có bằng lái xe chưa?". Kiểu lẫn này còn gặp ở 0348, 0379, 0670; riêng 0517 thì dùng "anh". → Đề xuất: sau khi cưới, dùng anh–em từ đầu đến cuối.
2. **CAST_MAU_THUAN (cao, hệ thống)**: truyện không có cast.json. Trong addressing, `苏江→安柔: tớ–cậu` và `安柔→苏江: tôi–cậu` được chốt từ chương 1 và không bao giờ cập nhật. Thêm một cặp lệch: `安兴昌→苏江: bố–con` nhưng `苏江→安兴昌: cháu–chú`.
3. **THUAT_NGU_TROI (cao, hệ thống)**: 督察局 xuất hiện 771 lần, ra "Cục Giám Sát" 603, "Cục Đốc Tra" 110 (20 file), "Cục Đốc Sát" 8. Nguyên nhân là glossary tự mâu thuẫn: 督察局 ⇒ Giám Sát, trong khi 华国/京城督察局 ⇒ Đốc Tra. Ví dụ 0258 đ.52: "tôi muốn tách khỏi Cục Đốc Tra".
4. **THUAT_NGU_TROI (vừa, hệ thống)**: 卫家 ra "Vệ gia" 169 lần so với "nhà họ Vệ" 46 lần; ngoài ra còn "Phong gia" 62, "An gia" 46. Ở 0189 đ.15, "nhà họ Mai… người của Vệ gia" nằm ngay trong cùng một câu.
5. **GLOSSARY_PHIEN_AM (vừa)**: 中二飞刀仔 thành "Nhóc Phi Đao **Trúng Nhị**" (0164), chỗ khác lại là "thằng nhóc phi đao chuunibyou" (0449, 0452). → Nên chốt một dạng, ví dụ "Nhóc Phi Đao Ảo Tưởng".
6. **MEME_NGHIA_DEN (vừa)**: meme 哈基嗣/哈基宇 (từ 哈基米) ra ba cách viết "Ha Cơ Tự", "Ha-ki Tự", "Haji Vũ" (0548, 0562, 0621), nên mất cái meme và người đọc dễ tưởng là ba người.
7. **XUNG_HO_TROI (vừa, lẻ)**: 0001 đ.57, 我特么哪知道我喜欢你什么 → "Mẹ nó ông đây biết tớ thích cậu cái gì chứ?". Câu lẫn "ông đây" với "tớ", lại mất nghĩa phủ định. → "Mẹ nó, ông đây biết quái đâu mình thích cậu ở điểm nào?"
8. **CALQUE (vừa)**: 0719 đ.38, "**Của cậu** không phải là vấn đề có sao hay không" → "Không phải chuyện có sao hay không đâu…". Cũng trong chương đó, Hạ Văn đang xưng "tôi" thì đến đ.44 nhảy sang "em".
9. **CALQUE (thấp, hệ thống)**: "đem" 146 lần/128 file, ví dụ 0026 đ.74 "rảnh rỗi là Tô Giang lại **đem đi** chơi game"; "Theo sự hiểu biết" 3 lần.
10. **MEME_NGHIA_DEN (thấp, hệ thống)**: 打了鸡血 → "tiêm máu gà" 6 lần/6 file (0360 đ.78). → "hăng máu hẳn lên".
11. **GLOSSARY_HAN_VIET_NANG (thấp)**: tên skill "Bộc Đạn Cuồng Nhân", "Thổ Mộc Công Trình Tinh Thông" đi ngược style.avoid. Thứ tự từ cũng lẫn ("Tinh Thông Cờ Bạc"), và 闷棍 có hai tên "Côn Đập Lén"/"Gậy Lén".
12. **BOI_CANH_SAI (thấp)**: 大手一挥 → "vung/phất **tay áo**" 9/39 dòng (0718 đ.35), một hình ảnh cổ trang lạc vào cảnh hiện đại. → "khoát tay".
13. **CALQUE (thấp)**: 大姐头 → "Đại tỷ đầu" 6 lần (0348, 0718) → "chị đại".
14. **HU_HA (thấp)**: tiếng ngáy 哼哼哼 → "Hừ hừ hừ" (0001 đ.9) → "Khò khò khò". 呵 → "Hừ" ở 50/233 dòng (0555, 0620: "Hừ hừ").
15. **TIEU_DE/DINH_DANG (thấp)**: 29 tiêu đề viết hoa mọi chữ (khoảng ch 51–80); 152 file dùng ngoặc thẳng `"`, 572 file dùng “”.

Lỗi nhẹ khác: "bản thân hắn/cô/gã" 79 lần/68 file (ví dụ 0027 đ.70 "to hơn cả bản thân hắn một vòng"); "người phụ nữ" 58 lần/29 file, nhiều chỗ nên thay bằng "cô ta/ả"; "Quán lão" 7 lần lẫn "Quan lão" 5 lần (冠文林).

## Gợi ý

- Sinh cast.json có mốc theo chương cho truyện dài. Auto-glossary addressing phải ghi đè được giá trị cũ, kèm số chương, thay vì chỉ giữ giá trị đầu tiên.
- Thêm checkRule: cảnh báo khi một cặp nhân vật dùng hai bộ đại từ trong cùng một chương, và khi "vợ/bà xã/ông xã" đi cùng "tớ/cậu".
- Dọn glossary: gộp mọi biến thể 督察局/监察局 về "Cục Giám Sát"; chuẩn hoá "X家" thành "nhà họ X"; sửa 中二飞刀仔; chốt một dạng cho 哈基嗣/宇. Khi thêm mục mới thì kiểm tra xung đột với mục cha/con và với style.avoid.
- Thêm vào prompt bối cảnh modern một bảng lóng mạng: 打了鸡血, 中二, 卷, 大姐头, 哈基米…; cấm thêm "tay áo" khi raw là 大手一挥; quy định tượng thanh 呼噜/哼哼 (ngáy) → "khò khò", và 呵呵 không mặc định là "Hừ hừ".
- Hậu kiểm định dạng: chuẩn hoá ngoặc “”, tiêu đề viết hoa kiểu câu.

## Làm tốt

- 嗯 dịch tự nhiên (Ừ/Vâng/Dạ), không có "Ân", không sót chữ Hán, không có "ngươi".
- Chửi thề và cà khịa sống động: "Vãi...... chưởng!", "đỉnh vãi", "Đứa mẹ nào bảo…", "thân trâu ngựa".
- Thông báo hệ thống gọn, máy móc; Tô Giang gọi hệ thống là "mi" rất hợp giọng.
- Đủ đoạn 1:1 với raw, gần như không sót ý.
