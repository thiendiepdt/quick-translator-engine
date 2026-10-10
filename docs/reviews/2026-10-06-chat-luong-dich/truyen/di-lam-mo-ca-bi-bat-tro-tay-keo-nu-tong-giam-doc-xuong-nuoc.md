# Review: Đi Làm Mò Cá Bị Bắt, Trở Tay Kéo Nữ Tổng Giám Đốc Xuống Nước

- **Setting:** modern. Thực tế đây là truyện hài công sở + hệ thống, kèm ngôn tình ngọt (từ đồng nghiệp thành người yêu rồi cưới). Kể ngôi 3.
- **Đã dịch:** 1252/1252 chương. **Đã đọc kỹ 4 chương:** 0001, 0627, 1251, 1252 (chương cuối là lời tác giả). Các lỗi nghi có tính hệ thống đều đã grep đếm trên toàn bộ out/.

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 5.0 | Tần Tầm gọi Hạ Ninh bằng "tôi–cô" từ chương 1 tới hết truyện, kể cả lúc đã yêu, đã cưới. Hạ Ninh thì gọi lại là "anh", nên hai bên lệch vai. |
| Giọng văn | 7.5 | Giọng hiện đại, tưng tửng, đúng chất hài công sở. Còn vài vệt Hán Việt kiểu convert: "Hạ nữ sĩ", "Hạ chủ tịch", "đại nhân mỹ nữ tổng giám đốc". |
| Dễ hiểu | 7.5 | Dịch sát nghĩa, đủ đoạn. Điểm trừ chính là con số: 亿 để thành "ức", 万 để thành "vạn". |
| Khó khăn | 7.0 | Meme phần lớn xử lý khéo. Vài chỗ dịch theo chữ nên mất ẩn ý (开车, 石楠花). |
| Thuật ngữ | 5.5 | Glossary có mục trùng hoặc lệch: một bệnh viện hai tên, một nhân vật hai cách viết. Có chương không theo glossary. |

## Lỗi (xếp từ nặng đến nhẹ)

1. **XUNG_HO_TROI. Nặng, hệ thống:** "Ninh Ninh, cô" xuất hiện 155 lần/125 file, còn "Ninh Ninh, em" chỉ 2 lần.
   - Ví dụ ở ch.1251 dòng 83: `宁宁，你听我解释……` → "Ninh Ninh, cô nghe tôi giải thích......" → nên là "Ninh Ninh, em nghe anh giải thích......".
   - Đây là cảnh trên giường sau khi đã cưới. Addressing `秦寻→夏宁: tôi–cô` chốt từ ch.1, không có cast.json và không có mốc nào để đổi.
2. **XUNG_HO_VAI_VE. Nặng:** ch.627 dòng 69: `你，我是怎么都看不腻的。` → "Còn cô thì tôi ngắm thế nào cũng không thấy chán." → nên là "Còn em thì anh ngắm thế nào cũng không chán."
   - Cùng cảnh đó Hạ Ninh gọi "anh" và lời bài hát dịch "anh–em". Chỉ riêng thoại của Tần Tầm là "tôi–cô".
3. **SAI_SO (亿 → "ức"). Nặng, hệ thống:** 118 lần/18 file.
   - Ví dụ ch.1247 dòng 87: `筹集了一百亿` → "gom góp một trăm ức" → nên là "gom góp mười tỷ".
   - Người đọc Việt không đổi được "ức" ra số thật.
4. **SAI_SO (万 → "vạn"). Vừa, hệ thống:** "số + vạn" 340 lần/122 file, chạy song song với "nghìn tệ"/"triệu tệ".
   - Ví dụ ch.1248 dòng 63: `2000，不过你要还20万` → "2.000 tệ, nhưng mà chị phải trả 20 vạn." Trong một câu có hai hệ đơn vị.
5. **GLOSSARY_PHIEN_AM. Vừa:** mục `海星妇幼保健院` → "Hải Hưng" bị phiên âm sai (星 = Tinh), còn mục `海星妇幼保健医院` → "Hải Tinh". Một bệnh viện mang hai tên ("Hải Hưng" 9 lần, "Hải Tinh" 29 lần). Ví dụ ở ch.1214 dòng 13.
6. **GLOSSARY_TRUNG. Vừa:** tác giả viết 米嘻儿 và 咪嘻儿 cho cùng một cô gái, glossary tách thành "Mễ Hy Nhi" (460 lần) và "Mễ Hi Nhi" (349 lần). Ví dụ ở ch.1161 dòng 57.
7. **THUAT_NGU_TROI. Vừa:** glossary đã chốt 摸鱼崽 = "Nhóc Mò Cá", nhưng chương mới nhất 1251 dịch thành "Mò Cá Tể" 7 lần (dòng 113). Glossary không được ép.
8. **GLOSSARY_DANH_XUNG. Vừa:** 女士 → "nữ sĩ" 35 lần/17 file; "Hạ chủ tịch" 63 lần. Ngay addressing cũng có "tôi–Hạ nữ sĩ".
   - Ví dụ ch.315 dòng 71: "Hạ nữ sĩ, cô định làm gì đây?" → nên là "Cô Hạ, cô định làm gì vậy?".
   - "Nữ sĩ" trong tiếng Việt nghĩa là nhà thơ, nhà văn nữ.
9. **CALQUE "người phụ nữ". Vừa:** 573 lần/350 file.
   - Ví dụ ch.1251 dòng 267: "Kỳ biến ngẫu bất biến, ting ting ting, đúng là người phụ nữ nhạt nhẽo!" → nên là "Lẻ đổi chẵn giữ, ting ting ting, đúng là con nhỏ chán phèo!".
   - Cả câu khẩu quyết 奇变偶不变 cũng bị phiên âm Hán Việt nên mất nghĩa.
10. **TIEU_DE. Vừa, theo phiên:** 20 tiêu đề viết hoa mọi chữ, thuộc 0958–0967 và 1019–1028 (đúng hai lô 10 chương). Ví dụ ch.962: "Có Thể Đã Phạm Pháp? Mẹ Kiếp! Nhắm Vào Ta Đấy À?!"
11. **CALQUE. Nhẹ:** ch.1 dòng 85: "thì sao ta có thể đặt đại nhân mỹ nữ tổng giám đốc bên cạnh cậu chứ?" → nên là "thì chị đời nào dám xếp bà sếp tổng xinh đẹp ngồi cạnh cậu?".
    - Các vệt tương tự: "đại nhân" 33 lần, "chiếm tiện nghi" 28 lần, "sao tác" (炒作) 4 lần.
12. **MEME_NGHIA_DEN. Nhẹ:** ch.1252 dòng 75: `小小的开了一下车，不会关很久` → "lái xe một chút thôi, sẽ không bị nhốt lâu đâu" → nên là "viết hơi 'nóng' một tí, chắc không bị khoá lâu đâu".
13. **KHAC. Nhẹ:** ch.627 dòng 161: mùi "hoa thạch nam" là ám chỉ tình dục quen thuộc trên mạng Trung. Dịch trơn thì mất câu đùa bậy.

## Gợi ý

- **Mốc xưng hô cho cặp chính:** dùng cast.json hoặc timeline (đồng nghiệp: tôi–cô; từ khi hẹn hò: anh–em). Bước auto-glossary cần được phép *cập nhật* addressing khi quan hệ đổi, chứ không chỉ thêm mục mới.
- **Luật số liệu cho bối cảnh modern:**
  - Quy đổi 万/亿 ra nghìn/triệu/tỷ, cấm dùng "ức".
  - Thêm checkRule bằng regex: `\d+ ức`, `(trăm|nghìn|mươi) ức`, `\d+ vạn`.
- **Khử trùng glossary:** cảnh báo khi hai source chỉ khác nhau 1 chữ mà target lại khác nhau. Sửa mục "Hải Hưng".
- **Prompt cho danh xưng:** 女士 → "cô/bà + họ"; 董/总 → "chủ tịch/sếp + họ". Không để "nữ sĩ" hay "tiên sinh" lọt vào addressing.
- **Check theo glossary:** báo khi một source trong glossary có mặt trong raw mà target không có trong out (ví dụ 摸鱼崽).
- **Tiêu đề:** tự động kiểm tra Title Case.
- **Prompt modern:**
  - 女人 trong lời mỉa mai dịch là "đàn bà / con nhỏ / cô ta", không mặc định "người phụ nữ".
  - Meme mạng thì chuyển nghĩa sang cách nói Việt tương đương.

## Làm tốt

- Đủ đoạn, căn dòng khớp. Không còn ký tự Hán nào trong out/.
- Không dùng "ngươi" hay "nàng". "Ân" chỉ 5 lần. "bản thân hắn/cô" chỉ khoảng 82 lần trên 4592 lần 自己 trong raw.
- Nhiều meme dịch khéo: 真的6 → "đỉnh thật", 草泥马 → "alpaca phi nước đại", 水文 → "câu chữ câu chương", 楼凤 → "gái gọi tại gia".
- Giọng tưng tửng, nhịp nhanh, thoại công sở tự nhiên.
