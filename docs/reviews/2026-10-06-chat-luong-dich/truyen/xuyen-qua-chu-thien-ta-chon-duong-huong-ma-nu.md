# Review: Xuyên Qua Chư Thiên: Ta Chọn Đường Hướng Ma Nữ

- setting: `ancient`. Thể loại đọc được: đồng nhân chư thiên + hệ thống, lời kể meme hiện đại trong thế giới Kim Dung (Tiếu Ngạo → Ỷ Thiên). Nhân vật chính đổi giới nam → nữ từ chương 0030.
- Ngôi kể: ngôi 3. Đã dịch 35 chương. Đọc trọn 0001, 0018, 0034, 0035, rồi grep toàn bộ out/.

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 7.0 | Đổi hắn → nàng đúng mốc, không lẫn lần nào. Nhưng các cặp phụ ở arc Ỷ Thiên bị trôi, và cast.json không có nhân vật chính |
| Giọng văn | 7.5 | Giữ được giọng tưng tửng, cà khịa, chất kiếm hiệp. Đôi chỗ lời kể lọt Hán Việt kiểu convert |
| Dễ hiểu | 7.0 | Phần lớn trôi chảy. Còn vài chỗ sai nghĩa do chọn từ, và một số calque cấu trúc |
| Khó khăn | 7.0 | Nhiều meme xử lý khéo. Vẫn còn vài meme dịch nghĩa đen |
| Thuật ngữ | 6.0 | Tên Kim Dung chuẩn. Thuật ngữ cốt lõi 教唆者 lại ra 3 bản dịch, và có tên phiên âm sai |

## Lỗi (nặng → nhẹ)

1. **THUAT_NGU_TROI, cao, hệ thống.** 教唆者 (Danh Sách 8 của nhân vật chính) có 22 lần trong 9 file raw. Bản dịch ra ba kiểu: "Kẻ Xúi Giục" 9 lần (0016, 0019, 0035), "Giáo Toa Giả" 10 lần (0023–0025), "Xúi Giục Giả" 3 lần (0026, 0030, 0034). Ví dụ 0034:147 "năng lực Xúi Giục Giả", còn 0035:189 "năng lực của Kẻ Xúi Giục". Mục này chưa có trong glossary. Đề xuất chốt một tên.
2. **GLOSSARY_PHIEN_AM, cao.** 丁勉 → "Đinh Mẫn" (13 lần/9 file). Đúng phải là "Đinh Miễn". Tên này còn đụng với 丁敏君 "Đinh Mẫn Quân".
3. **CAST_MAU_THUAN, vừa.** cast.json không có 江夜 và không có mốc đổi giới ở 0030. Bản dịch hiện đúng chỉ vì raw có sẵn 他/她. Ngoài ra 童百雄 và 童百熊 đang là hai mục riêng.
4. **XUNG_HO_VAI_VE, hệ thống.** Sử Hỏa Long gọi Kỷ Hiểu Phù:
   - raw 0034:9 「你年纪轻」 → hiện tại "Nàng tuổi còn trẻ chắc chưa biết" → đề xuất "Cô nương tuổi còn trẻ…".
   - 0033:153 cũng gọi "Nàng".
   - "Nàng" ở ngôi 2 là giọng tình tứ, của riêng Dương Tiêu.
5. **XUNG_HO_VAI_VE.** Kỷ Hiểu Phù nói với bậc trên:
   - 0034:221 「那我与你们一同去」 → hiện tại "Vậy ta cùng đi với các ngươi." → đề xuất "Vậy ta đi cùng hai vị."
   - Ngay dòng 209 cô vừa gọi họ là "hai người", nên chỗ này lệch.
6. **XUNG_HO_VAI_VE.** Dương Tiêu nói về chú ruột:
   - 0035:15 「论辈分他是我的叔父」 → hiện tại "hắn là thúc phụ của ta" → đề xuất "người là thúc phụ của ta".
7. **DAI_TU_MO_HO.** 0034:77 "Hoàng đế Thát Đát kia đều bị người giết tới sáu vị". Cùng kiểu ở dòng 65 và 81. "Bị người giết" dễ đọc thành "bị người ta giết", mất chủ thể Trương Tam Phong.
8. **SAI_NGHIA.** 0035:221 「拳力即将临身」 → hiện tại "quyền lực sắp chạm vào thân" → đề xuất "kình lực cú đấm sắp chạm tới người".
9. **SAI_NGHIA.** 0035:187 「气劲牵引」 → hiện tại "khí kình… dắt mối kéo theo" → đề xuất "dẫn dắt, cuốn theo". "Dắt mối" nghĩa là làm mai, môi giới.
10. **MEME_NGHIA_DEN.** 0018:121:
    - 「花在刀刃上…最大的刀刃」 → hiện tại "tiêu vào lưỡi dao bén nhất… lưỡi dao lớn nhất". Thành ngữ này nghĩa là "tiêu đúng chỗ".
    - 「铺天盖地」 → hiện tại "rải rác rợp trời". "Rải rác" nghĩa là thưa thớt, ngược nghĩa gốc.
11. **MEME_NGHIA_DEN.** 0025:173 「不列颠人…搅屎棍」 → hiện tại "người Briton… cây gậy khuấy phân". Tiêu đề 0025 cũng bị. Thêm 装逼 → "trang bức" 9 lần ở 0016. Đề xuất "phá bĩnh", "làm màu".
12. **CALQUE, hệ thống.**
    - 0035:219 「也好…也罢」 → hiện tại "Hắc hỏa… cũng tốt, lam băng cũng xong".
    - 四处 → "bốn phía phân phong", "bốn phía đồ sát" (3 lần).
    - "ngạnh sinh sinh" (0035:197).
    - "mười mấy năm công phu" (0034:7).
13. **NGUOI_CHUNG, thấp, hệ thống.** 你 chỉ người chung chung bị dịch thành "ngươi":
    - 0035:251 "ngươi nói đây là tu tiên ta cũng tin sái cổ".
    - 0004:109 "trừ phi ngươi là thần tiên".
14. **THUAT_NGU_TROI, thấp.**
    - 倭寇: "giặc Oa" ở 11 file, "Oa khấu" ở 4 file (0016–0020).
    - "Quang Minh tả sứ" và "Quang Minh Tả Sứ" dùng lẫn.
    - Trong cùng 0035 có "Ma Sư" 9 lần và "ma sư" 7 lần.
15. **SAI_SO, thấp.** 三五 → "ba năm" (4 file). Ví dụ 0035:177 "Qua ba năm chiêu", 0001:179 "trong vòng ba năm năm". Đề xuất "dăm ba".

## Gợi ý

- **Glossary, thêm mục.** Đưa vào ngay từ lần đầu xuất hiện:
  - toàn bộ chuỗi Danh Sách của đường hướng Ma Nữ;
  - các danh từ chung hay lặp: 倭寇, 鞑子, 魔师, 光明左使.

  Auto-glossary hiện chỉ bắt tên riêng.
- **Glossary, sửa mục sai.**
  - 丁勉 → Đinh Miễn.
  - 向阳老宅 → nhà cũ Hướng Dương.
  - 吉米仔 → Jimmy.
  - 铁罗汉 đang viết hoa không thống nhất với bản dịch.
- **cast.json.**
  - Luôn tạo mục cho protagonist, có mốc giới tính theo chương (female từ 0030).
  - Gộp các biến thể tên do tác giả viết sai.
- **Luật xưng hô trong prompt.**
  - "Nàng" ở ngôi 2 chỉ dùng cho cặp tình cảm.
  - Người ngoài gọi nữ trẻ là "cô nương".
  - Vai dưới không gọi bậc trên là "ngươi/các ngươi".
  - Không gọi người thân bề trên là "hắn".
  - Tránh cấu trúc "bị người + V".
- **Danh sách calque cần tránh.** Thêm: 也好…也罢, 四处, 硬生生, N年功夫, 三五, 拳力, 顶尖 → "đỉnh tiêm".
- **Setting.** Nên có setting kép (thế giới cổ, người kể hiện đại) để lời kể được dùng từ hiện đại tự nhiên, còn thoại nhân vật vẫn giữ cổ trang.
- **checkRule.** Cảnh báo khi một raw term có từ 2 bản dịch trở lên trong out/.

## Làm tốt

- Đổi hắn → nàng chính xác từ 0030. Các chương 0031–0035 không lẫn "hắn" lần nào.
- Giọng hệ thống cà khịa giữ tốt. Ví dụ "Ký chủ thân mến của ta, sở thích của ngài...... đúng là độc lạ thật".
- Meme xử lý khéo:
  - 润 → "chuồn"
  - 章口就莱 → "mở mồm chém gió"
  - 挂 → "hack"
  - 酸不酸 → "sến súa vừa thôi chứ"
- Glossary tên Kim Dung dày và nhất quán.
- Không sót chữ Hán. "Ân" chỉ xuất hiện 1 lần, không có "Hừ" cho 呵.
- Tiêu đề chương đúng kiểu câu.
