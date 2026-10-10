# Review: Nhiều Con Nhiều Phúc, ta đi bệnh viện góp 500 lần

- Slug: `da-tu-da-phuc-ta-di-benh-vien-gop-500-lan`. Setting: `modern`. Thể loại thực tế: đô thị + hệ thống, hài tục, nhiều độc thoại 【】. Truyện kể **ngôi ba**.
- 251 chương đã dịch. Đọc trọn 4 chương: 0001, 0126, 0250, 0251 (0250/0251 là hai chương dịch thật mới nhất; 32 file có mtime 13/09 chỉ là đợt chạm hàng loạt). Có grep toàn bộ out/.

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 5.5 | Các cặp chính đổi qua lại tôi/cô ↔ anh/em, anh ↔ cậu; không có cast.json; độc thoại trộn ta/mình/tao |
| Giọng văn | 7.5 | Khẩu ngữ đô thị sống, ít Hán Việt nặng; còn rải rác "đại hán", "kinh diễm", "Hạo Nam huynh" |
| Dễ hiểu | 8.5 | Bám nghĩa, đủ đoạn, đổi tiền đúng; chỉ vài calque lẻ |
| Khó khăn | 7 | Lóng game/thành ngữ tốt; 呵→"Hừ" 65%, 卧槽 gần như luôn "Vãi", meme dịch nghĩa đen |
| Thuật ngữ | 5 | Tên kỹ năng trôi 3–4 kiểu, biệt danh các con không theo một hệ, glossary tách đôi/trùng mục |

## Lỗi (nặng → nhẹ)

1. **XUNG_HO_TROI – cao, hệ thống.** Trần Mặc→Cố Thanh Dao: tôi/cô (0069, 0126, 0200) xen anh/em (0065, 0068, 0197, 0244…). 0126 dòng 73: `你和你爸…你们盛恒` → "cô với bố cô… Thịnh Hằng của các người" → đề xuất "em với bố em… Thịnh Hằng nhà em". Hai người đã là tình nhân mà vẫn "các người" nghe thù địch.
2. **XUNG_HO_TROI – cao, hệ thống.** Trì Phương Hoa→Trần Mặc: "anh" (0088–0096, 0131) rồi "cậu" (0121, 0126). 0126 dòng 167, cảnh thân mật: "Trần Mặc, cậu có thấy tôi ở trên giường... hơi nhạt nhẽo không?" → "em có thấy chị…". Trần Mặc gọi cô "chị Phương Hoa", tự xưng "em" ngay trong chương đó.
3. **CAST_MAU_THUAN – cao, hệ thống.** Không có cast.json. 267 mục addressing trong story.json chỉ ghi xưng hô lần gặp đầu (`陈默→顾清瑶: tôi–cô`, `池芳华→陈默: tôi–Trần tiên sinh`), không có mốc đổi, dù đến 0250 Thanh Dao đã gọi "Chồng à". Đây là nguyên nhân của lỗi 1–2.
4. **THUAT_NGU_TROI – cao, hệ thống.** 透视之眼: Thấu Thị Chi Nhãn 60 lần / Mắt Thấu Thị 6 / Mắt Nhìn Xuyên Thấu 5. 透视眼: thấu thị 36 / mắt nhìn xuyên thấu 16. 读心术: Độc Tâm Thuật 57 / thuật đọc tâm 17. 中医精通 có 4 kiểu. Ví dụ 0250 dòng 143: "trung y tinh thông kết hợp với Mắt Nhìn Xuyên Thấu" → "Tinh thông Đông y cộng thêm Thấu Thị Chi Nhãn". Glossary có mục mà vẫn trôi.
5. **THUAT_NGU_TROI – vừa.** Các con đánh số không theo một hệ: 小十一 = Tiểu Thập Nhất; 小十二 = "bé thứ mười hai" (0217–0218) hoặc Tiểu Thập Nhị (0235); 小十三 = Tiểu Mười Ba (0249–0250).
6. **XUNG_HO_TROI – vừa, hệ thống.** Nhân vật hiện đại nghĩ thầm trong 【】 xưng "ta" 177 lần/42 file. 0251 dòng 191, Cố Vãn Tình: "ta vốn đã định làm người phụ nữ của cậu ấy" → "mình vốn đã định…". 0107 dòng 79 còn trộn "ta" và "mình" trong cùng một câu. Luật "ta" chỉ áp cho người kể ngôi một.
7. **XUNG_HO_NGUOI_KE – vừa, hệ thống.** Độc thoại của Trần Mặc lúc "mình" (chương 1: "Là do mình không muốn tìm chắc?"), lúc "ta" (khoảng 145 lần/72 file), ví dụ 0251 dòng 93: "bà ngoại của con ta". signaturePhrase `我有儿子了 = Ta có con trai rồi` kéo model về "ta".
8. **HU_HA – vừa, hệ thống.** 呵 → "Hừ" ở 95/147 dòng (74 file). Ví dụ 0251 dòng 89: "Hừ, phân lượng của hồ sơ cấp SSS…" → "Chà, hồ sơ cấp SSS đúng là nặng ký thật."
9. **GLOSSARY_TRUNG – vừa.** Tác giả gõ 赵锐/赵瑞 cho cùng một ông chủ quán nướng, nên chương 0010 có cả "Triệu Duệ" lẫn "Triệu Thụy". Ngoài ra có 3 mục 格斗精通 với 3 tên dịch khác nhau, và 香江 lúc là "Hương Cảng", lúc là "Hương Giang".
10. **MEME_NGHIA_DEN – thấp.** 马爸爸 → "bố Mã" (0134 dòng 119), nên ghi rõ là Jack Ma. 老登 → "lão đăng" (15 lần, 6 file đầu), các chương sau lại thành "lão già".
11. **GLOSSARY_DANH_XUNG – thấp.** 陈阿姨, 秦老, 郑老, 郝经理, 陆少, 老顾董 bị lưu như tên riêng. 许少 lúc là "Hứa thiếu" (32 lần), lúc là "Cậu Hứa" viết hoa giữa câu (0126 dòng 123).
12. **XUNG_HO_TROI – thấp, lẻ.** 0251 dòng 79: Tần Vân Hà tự xưng "bác", hai câu sau lại xưng "dì".
13. **TIEU_DE – thấp.** 0054: "Anh cưới ta", trong khi thân chương dịch "anh cưới tôi".
14. **DAI_TU_MO_HO – thấp.** 0251 dòng 3: "xem con trai của hắn" dễ hiểu nhầm là con Lương Kiến Trung, thật ra là con Trần Mặc do Cố Vãn Tình sinh.
15. **CALQUE – thấp, lẻ.** 千亿集团 → "tập đoàn nghìn ức" (0054 dòng 187) → "trăm tỷ". Ngoài ra còn "Theo sự trưởng thành", "đại hán", "kinh diễm".

## Gợi ý

- Lập cast.json có mốc xưng hô cho các cặp tình cảm chính. Auto-glossary addressing cần ghi "từ chương" hoặc ghi đè khi quan hệ đổi.
- Prompt modern + ngôi ba: độc thoại 【】 và độc thoại tự do dùng mình/tao/tôi, không dùng "ta". Thêm checkRule bắt "ta" ngôi một trong 【】. Sửa signaturePhrase thành "Mình có con trai rồi".
- Checker cho mục skills: raw có từ khoá glossary mà out không có bản dịch đã chốt thì cảnh báo. Thêm alias 透视眼.
- Dọn glossary: gộp 赵锐/赵瑞 và các mục 格斗精通; thống nhất 香江; chuyển danh xưng (老/少/经理/阿姨) thành luật chung; thêm 小十三 theo mẫu Tiểu Thập X.
- Luật thán từ: 呵 → Ha/Chà/Hơ/cười khẩy, "Hừ" chỉ dành cho 哼; dịch 卧槽 đa dạng hơn. Quy đổi 万/亿, cấm để "vạn", "ức".

## Làm tốt

- Đủ đoạn ở 251/251 chương, không sót chữ Hán, tiêu đề đúng định dạng.
- Không có "Ân" cho 嗯, không có "ngươi", "bản thân hắn" ít.
- Thoại cãi vã nông thôn giữ được độ thô; lóng game ("sp"), thành ngữ ("ba ba trong rọ", "ngậm bồ hòn làm ngọt") tự nhiên.
- Quy đổi tiền chính xác; thông báo hệ thống gọn, đúng giọng máy.
