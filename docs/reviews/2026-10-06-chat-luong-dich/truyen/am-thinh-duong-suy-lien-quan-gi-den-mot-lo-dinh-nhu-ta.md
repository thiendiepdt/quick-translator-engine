# Review: Âm Thịnh Dương Suy? Liên Quan Gì Đến Một Lô Đỉnh Như Ta

- Bối cảnh: tu tiên tà đạo, âm thịnh dương suy, nhiều arc chư thiên/túc thế thân, setting `ancient` (khớp). Kể ngôi 3. Đã dịch 534 chương.
- Đã đọc trọn: 0001 (chương đầu), 0267 (giữa), 0534 và 0386 (dịch gần nhất). Có grep toàn bộ out/ để đếm các lỗi hệ thống.

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 8 | Tỷ–đệ, điện hạ–Tiểu Lý, sư tỷ–đệ đúng và đều. Đại từ ngôi ba của Thiên Toái chân nhân trôi giữa lão, bà, lão nhân. Có chỗ kẻ dưới xưng "bọn ta" với chân nhân. |
| Giọng văn | 8.5 | Cổ phong hợp thể loại, ma mị đúng chỗ. Chỉ lác đác Hán Việt thô, ví dụ "Vô sở bất năng". |
| Dễ hiểu | 8.5 | Trôi chảy, ít văn convert, số đoạn khớp 100%. Hiếm lỗi nghĩa. |
| Khó khăn | 7.5 | Miếng hài được giữ. Nhưng độc thoại 『』 bị biến thành lời thoại, và 呵 bị dịch thành "Hừ". |
| Thuật ngữ | 6.5 | Áp glossary nhất quán, nhưng phiên âm sai ở các tên cốt lõi. |

## Lỗi (nặng → nhẹ)

1. **DINH_DANG (cao, hệ thống)**: 750 trên 2045 dấu 『』 bị mất, ở 151 file, gồm cả 3 chương dịch gần nhất (0533, 0534, 0386).
   - 0386:147: `『她是故意让依莎看到的。』` → “Nàng ta cố ý để Y Sa nhìn thấy.” → nên là 『Nàng ta cố ý để Y Sa nhìn thấy.』
   - Suy nghĩ thầm thành lời nói ra miệng ngay trước mặt Long Nữ. Ý nghĩ "Đáng tiếc thật......" của Tức Dực (dòng 89) cũng thành thoại.
   - Lỗi xảy ra theo từng lô chương, nghĩa là model lúc giữ dấu lúc không, và không có check nào bắt được.
2. **GLOSSARY_PHIEN_AM (vừa, hệ thống)**: 千祟 → "Thiên Toái" (116 lần, 37 file; kèm 上祟 → "Thượng Toái").
   - 祟 đọc là "túy", còn "toái" là chữ 碎. Đề xuất: Thiên Túy chân nhân.
3. **GLOSSARY_PHIEN_AM (vừa, hệ thống)**: 蛊蠹 → "Cổ Đỗ" (34 lần), 养蠹将 → "Dưỡng Đỗ Tướng".
   - 蠹 đọc là "đố". Đề xuất: Cổ Đố.
4. **THUAT_NGU_TROI (vừa)**: 受抚顶 → glossary ghi "Thọ Phủ Đỉnh" (4 lần), bản dịch có lúc tự sửa thành "Thụ Phủ Đỉnh" (2 lần).
   - 受 đọc là "thụ". Một thần thông đang mang hai tên.
5. **HU_HA (vừa, hệ thống)**: 呵 → "Hừ", 32 lần ở khoảng 30 file.
   - 0386:71: “呵。” → “Hừ.” → nên là “Ha.” (cười lạnh).
   - Bản dịch mất sắc thái khinh miệt, ma mị.
6. **XUNG_HO_TROI (vừa, hệ thống)**: đại từ chỉ Thiên Toái chân nhân (nữ, già) trôi qua các chương 0224–0267.
   - 0231:95 gọi "lão nhân", 0235:93 gọi "bà" (`她趺坐…` → "bà ngồi xếp bằng"), 0260 gọi "nó".
   - cast.json không ghi đại từ cho nhân vật này.
7. **SAI_NGHIA (vừa)**: 0386:137: `打击了幼小心灵` → "kích thích tâm hồn non nớt".
   - 打击 là sốc, đả kích; "kích thích" gợi nghĩa khêu gợi. Đề xuất: "tâm hồn non nớt bị giáng một đòn".
8. **CAST_MAU_THUAN (thấp)**: hai chỉ dẫn mâu thuẫn nhau.
   - Addressing ghi 阴月璃→安生 là "ta–ngươi", còn toneRules đòi "tỷ tỷ/hảo đệ đệ".
   - Kết quả là câu trộn hai kiểu, ví dụ 0005:101 "Hảo đệ đệ, ngươi kẹt lại… để tỷ tỷ tới giúp ngươi".
9. **XUNG_HO_VAI_VE (thấp)**: 0534:95: `我等` → "bọn ta". Con chồn vừa xưng "tiểu tu" trước chân nhân. Đề xuất: "bọn tiểu tu".
10. **CALQUE (thấp, hệ thống)**: 女人 trong lời kể → "nữ nhân".
    - Xuất hiện ở 586 dòng kể, trong đó "người nữ nhân" 21 lần.
    - 0001:93 "dáng vẻ dị thường của nữ nhân" → nên là "của nàng".
11. **CALQUE (thấp)**: 0534:55: `狐属幻身` → "huyễn thân hồ thuộc". Dòng 39 cùng chương lại dịch 狐属 là "hồ tộc".
12. **DINH_DANG (thấp, hệ thống)**: 29 file dùng ngoặc kép thẳng " ", ví dụ 0027–0030, 0355–0363, 0486–0495.
    - Có file còn dịch 唔 thành "Ân" (0492:81).
13. **GLOSSARY_TRUNG (thấp)**:
    - 明辰 và 明神 cùng ra "Minh Thần"; 摩罗 và 魔罗 cùng ra "Ma La".
    - Danh xưng bị coi là tên: 张猎户 → "Trương thợ săn".
    - Mục "璃姐" nằm trong addressing nhưng không có dấu →.
14. **MEME_NGHIA_DEN (thấp)**: 0001:5: tag 逆推 → "Nghịch thôi", độc giả Việt không hiểu. Đề xuất: "Nữ chủ động đẩy ngã".
15. **TIEU_DE (thấp)**: 7 tiêu đề viết hoa mọi chữ dù không phải tên riêng, ví dụ "Chương 67: Món Quà Của Số Phận".

## Gợi ý

- Thêm check tự động đếm 『』 và ‘’ ở raw so với out, lệch thì báo lỗi; ghi rõ trong prompt rằng 『』 là độc thoại nội tâm và phải giữ nguyên. Chuẩn hoá " " thành “ ” ở bước hậu xử lý.
- Kiểm phiên âm glossary tự sinh bằng bảng Hán Việt từng chữ. Sửa ngay 千祟, 上祟, 蛊蠹, 养蠹将, 受抚顶, rồi thay hàng loạt trong out/.
- Cảnh báo khi hai khoá glossary ra cùng một bản dịch; chặn danh xưng khỏi mục names.
- Thêm vào cast.json trường "đại từ ngôi ba trong lời kể" cho từng nhân vật.
- Gỡ mâu thuẫn xưng hô của Âm Nguyệt Ly: "tỷ tỷ–đệ" khi mị hoặc, "ta–ngươi" khi lộ sát ý.
- Ghi vào prompt: 呵 là tiếng cười, không phải "Hừ"; 女人 trong lời kể dịch là "nàng/ả", cấm "người nữ nhân".
- Thêm bảng tag/meme web-novel kèm nghĩa Việt; tiêu đề chương chỉ viết hoa chữ đầu và tên riêng.

## Làm tốt

- Văn cổ phong mượt, ma mị đúng chỗ, giọng nội tâm của nam chính hóm hỉnh.
- Giữ được các miếng hài: ngủ đủ 8 tiếng, uống sữa ấm, "bug", "như húp bún", "lừa trọc", "Thí chủ chấp tướng rồi".
- Xưng hô chính đều qua các arc; các mốc đổi xưng hô trong cast.json được tôn trọng.
- Số đoạn khớp 100%, không sót chữ Hán. 嗯 hầu như dịch tự nhiên, chỉ khoảng 15 chỗ còn "Ân".
- Hiểu đúng ý tác giả cả khi tác giả dùng sai từ (取缔 → "thay thế").
