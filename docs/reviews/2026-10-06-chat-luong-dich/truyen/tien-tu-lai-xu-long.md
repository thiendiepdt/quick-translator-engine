# Review: Tiên tử, lại xù lông nữa là nàng thật sự hơi thiếu đòn yêu! (tien-tu-lai-xu-long)

Thể loại: tiên hiệp hậu cung hài-ngọt, kể ở ngôi 3 (setting `ancient`). Đã dịch 621 chương. Đọc trọn ch1, ch310, ch619, ch620 (hai chương cuối là hai chương có mtime mới nhất) và grep toàn bộ out/.

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 5.5 | Cặp Cố Trì–Khê Vũ trôi giữa ngươi/nàng/đệ ngay trong một cảnh. Phương Tử Nguyệt lúc được kể là "nàng", lúc là "bà". Addressing có 71/113 mục là "ta–ngươi", không có mốc chương, cũng không có cast.json. |
| Giọng văn | 7.5 | Văn tiên hiệp mượt, cảnh ngọt vẫn giữ được nét tinh nghịch. Thỉnh thoảng lọt từ hiện đại. |
| Dễ hiểu | 7.5 | Đa số đủ ý và rõ. Rải rác có câu lắp ghép sai nghĩa, thành ngữ tự bịa, chi tiết thêm thắt. |
| Khó khăn | 7 | Thán từ và nhịp hài khá ổn, nhưng 嗯→"Ân" và 呵→"Hừ" làm câu khô hoặc lệch sắc thái. |
| Thuật ngữ | 6.5 | Glossary lớn, phần lớn được tuân thủ. Riêng tên Mặc Trúc Sanh bị đổi ở 4 chương mới nhất, vài phiên âm sai, Hoả/Hỏa lẫn lộn. |

## Lỗi (nặng → nhẹ)

1. **XUNG_HO_TROI (cao, hệ thống).** Ch310 dòng 159 "Hôn chết nàng!", rồi dòng 173 lại "Ngươi thắng rồi, ta mới là chó con.". Đến ch620, Khê Vũ chuyển sang gọi Cố Trì là "đệ". Trong thoại ở các đoạn có Khê Vũ: "ngươi" 1303 lần, "đệ" 72 lần, trộn theo chương. Đề xuất: sau khi thành đôi thì chốt "nàng/sư tỷ" ↔ "đệ/phu quân".
2. **CAST_MAU_THUAN (cao, hệ thống).** Addressing ghi `方溪雨→顾迟: ta–ngươi` dù hai người đã là vợ chồng. Khoá `长公主` là danh xưng, mà truyện có nhiều công chúa. Không có mốc, không có cast.json, nên model mỗi chương tự đoán.
3. **XUNG_HO_VAI_VE (cao, hệ thống).** Ch619 dòng 13 "Cố Trì tò mò liếc nhìn bà", trong khi lúc này Phương Tử Nguyệt là người tình/cô dâu của hắn. Trong các đoạn có bà: "bà" xuất hiện 92 lần; ch301–500 không có lần nào, ch501–622 có 44 lần. → "nàng".
4. **THUAT_NGU_TROI (cao, hệ thống).** Glossary ghi 墨竹笙 = "Mặc Trúc Sanh" (2185 lần/132 file), nhưng có 74 lần/10 file là "Mặc Trúc Sênh", gồm cả 4 chương dịch mới nhất (ch620 dòng 5). Lượt dịch mới không áp glossary, và bước check không bắt được.
5. **TIEU_DE (vừa, hệ thống).** 6 chương mới nhất có tiêu đề dạng "Chương 619: Những chuyện vụn vặt", còn 614 chương trước là "310 Cố gắng hết sức".
6. **AN_THAN_TU (vừa, hệ thống).** Ch1 dòng 49: 嗯,似乎… → "Ân, dường như…". Đề xuất: "Ừm, xem ra…". "Ân" xuất hiện 506 lần/307 file. Cách dịch cũng không nhất quán: cùng ch620, 嗯? thành "A?" nhưng 嗯 lại thành "Ân".
7. **GLOSSARY_PHIEN_AM (vừa).** 琼玉酿 được phiên thành "Cùng Ngọc Nương", nghe như tên người; đúng là "Quỳnh Ngọc Nhưỡng". 淬剑 là "Tôi Kiếm", không phải "Thúy Kiếm". 炽火 là "Sí Hỏa", không phải "Xích Hoả" (sai 29 lần/20 file, trong khi 炽火灵参 lại đúng là "Sí Hỏa").
8. **SAI_NGHIA (vừa, lẻ).** Ch310 dòng 71: "thì ta đoán người chỉ đoán theo hướng tồi tệ nhất thôi đấy" là câu lắp theo trật tự tiếng Trung. Dòng 75 tự bịa thành ngữ "Hóa giận mất khôn" cho 气急败坏.
9. **HU_HA (vừa, hệ thống).** Ch620 dòng 121: 呵,我卤菜… → "Hừ, món kho…". Đề xuất: "Ha, …". Có 11/34 đoạn chứa 呵 bị dịch thành "Hừ".
10. **CALQUE (vừa, hệ thống).** "chiếm tiện nghi" 62 lần/54 file, "đem" 251/186. Ch619 dòng 17: "Ngươi có loài động vật nhỏ nào khá yêu thích không?". Đề xuất: "Nàng thích con vật nhỏ nào nhất?".
11. **THEM_Y (thấp).** Ch620 dòng 109: "một đĩa cá hồi phi lê phần bụng". Đây là linh ngư hồ băng, nên → "gỏi cá phần bụng".
12. **BAN_THAN (thấp, hệ thống).** "bản thân" 827 lần/404 file. Ch620 dòng 113: "nghĩ xem bản thân thích ăn món gì" → "mình thích ăn gì".
13. **VAN_KHO (thấp).** "viêm màng túi" (囊中羞涩) là lóng hiện đại, trái style.avoid. "hoàn hôn" nên là "thành hôn".
14. **THUAT_NGU_TROI (thấp).** "Hoả" 222 lần, "Hỏa" 102 lần: hai kiểu bỏ dấu Unicode cho cùng tên (ch1 "Hoả Hoàng Tông", ch619 "Hỏa Hoàng Tông"). "Cửu cảnh" 18 lần, "cửu cảnh" 34 lần.

## Gợi ý

- Sinh cast.json có mốc xưng hô theo chương cho các cặp tình cảm. Với cặp đã là đạo lữ, cấm addressing tự sinh chỉ ghi "ta–ngươi".
- Thêm trường "đại từ kể" cho nhân vật lớn tuổi nhưng là người tình (Phương Tử Nguyệt → "nàng").
- Không cho khoá addressing là danh xưng (长公主). App nên cảnh báo khi khoá không có trong glossary.names.
- Thêm checkRule: khi raw có tên trong glossary.names thì bản dịch phải dùng đúng target. Rule này bắt được lỗi "Sênh".
- Cố định format tiêu đề trong code thay vì để prompt quyết, rồi sửa lại ch615–620.
- Luật 嗯: dùng "Ừm/Ừ" khi đồng ý hoặc trầm ngâm, "Hửm?/Hả?" khi hỏi lại. Luật 呵: "Ha/Hơ"; chỉ 哼 mới dịch là "Hừ".
- Kiểm phiên âm glossary tự sinh bằng từ điển Hán Việt, và chuẩn hoá Unicode dấu (Hoả/Hỏa) trước khi lưu.

## Làm tốt

- Đủ đoạn, khớp 100% số đoạn với raw, không còn ký tự Hán trong 621 file.
- Văn tiên hiệp mượt, cảnh ngọt giữ được nhịp tinh nghịch ("Gâu.", "chụt", "Hôn chết nàng!").
- Signature phrase 哈气 → "xù lông" được áp đúng ngữ cảnh.
- Giữ danh xưng tông môn. Cố Trì gọi Quỳ Hoàng là "ngài", rất hợp vai.
- Chữ bị kiểm duyệt (**) được xử lý khéo thành "chỗ nhạy cảm".
