# Review: Xuyên Nhanh: Tuyệt Sắc Yêu Nữ Dựa Vào Sinh Con Sống Lâu Trăm Tuổi

- Setting trong story.json: `ancient`. Thể loại thật: xuyên nhanh nữ tần. Khung hiện đại ở bệnh viện mở đầu mỗi arc; phần lớn là arc cổ đại; arc **hào môn hiện đại** chạy trọn ch.263–302 (40 chương); phiên ngoại là tu tiên. Kể ở ngôi 3.
- Tổng 436 chương đã dịch. Đọc kỹ 4 chương: ch.1 (hiện đại), ch.280 (hào môn, hiện đại), ch.434 và ch.435 (phiên ngoại tu tiên, có mtime mới nhất). Grep toàn bộ out/ để đếm.

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 5.0 | Arc cổ đại ổn. Arc hiện đại thì hỏng: nhân vật gọi "tỷ", xưng "ta", gọi "Giang tiểu thư"; người kể gọi nữ chính là "nàng" ở ch.263–280 rồi đổi sang "cô" từ ch.281 |
| Giọng văn | 6.0 | Cổ trang và tu tiên trơn tru. Nhưng do setting khoá cứng, từ cổ trang (nam nhân, nữ nhân, phu thê, Hạ phụ, Tần di, chính thê) tràn vào 40 chương hiện đại |
| Dễ hiểu | 7.5 | Đọc trôi, ít văn convert; còn rải rác "đem…", "một phần sắc mặt", "bồi nàng" |
| Khó khăn | 7.5 | Meme dịch tốt; có chỗ làm nhẹ câu chửi tục; "Ân" đọc khô |
| Thuật ngữ | 5.5 | Có phiên âm sai, danh xưng hiện đại bị biến thành tên riêng, tiêu đề một arc dịch ra 2–4 kiểu |

## Lỗi (xếp từ nặng đến nhẹ)

1. **BOI_CANH_SAI (cao, hệ thống)**: arc hiện đại dùng "tỷ" 248 lần/28 file, riêng "Tình Tuyết tỷ" 36 lần/21 file.
   - Ch.280 L23 `你进去睡吧，我睡沙发` → "Tỷ vào trong ngủ đi, em ngủ sô pha."
   - Đề xuất: "Chị vào trong ngủ đi, em ngủ sofa."
   - Ch.283 còn có câu "tỷ với anh trai em".
2. **CAST_MAU_THUAN (cao, hệ thống)**: phần addressing do app tự sinh chốt sẵn các cặp xưng hô sau: `夏以樾→江晴雪 = em–Tình Tuyết tỷ`, `夏以朗→秦姨 = ta–Tần di`, `夏父→夏母 = ta–bà`.
   - Các cặp này trái với toneRules ("hiện đại dùng tôi/cô/anh").
   - Đây là nguyên nhân gây ra lỗi 1 và lỗi 4.
3. **XUNG_HO_TROI (cao, hệ thống)**: người kể gọi nữ chính là "nàng" khoảng 484 lần ở ch.263–280, rồi không dùng lần nào ở ch.281–293 vì đã chuyển sang "cô".
   - Ch.280 L9 "Nàng ném qua một chiếc khăn lông…" → "Cô ném cho anh…"
4. **BOI_CANH_SAI (cao, hệ thống)**: nhân vật hiện đại xưng "ta" và gọi "Giang tiểu thư" (20 lần).
   - Ch.281 L131 `江小姐，我也可以给股份啊` → "Giang tiểu thư, ta cũng có thể cho cổ phần mà…"
   - Đề xuất: "Cô Giang, tôi cũng có thể cho cô cổ phần mà…"
5. **XUNG_HO_VAI_VE (vừa)**: ngay ch.1 L73, bà mẹ cãi nhau ở bệnh viện xưng "Để ta vào, ta là mẹ ruột của nó".
   - Đề xuất: "Cho tôi vào, tôi là mẹ ruột nó…"
6. **BOI_CANH_SAI (vừa, hệ thống)**: 男人/女人 dịch thành "nam nhân/nữ nhân" 132 lần trong arc hào môn.
   - Ch.289 L61 "một nữ nhân đã ly hôn như con" → "con là đàn bà đã ly hôn…"
7. **THUAT_NGU_TROI (vừa, hệ thống)**: "Hạ phụ/Hạ mẫu" xuất hiện 86 lần, trong khi "ba Hạ/mẹ Hạ" xuất hiện 124 lần. Ngay trong ch.280 đã lẫn cả hai (L59 so với L63).
   - Đề xuất: thống nhất thành "ông Hạ/bà Hạ".
8. **GLOSSARY_DANH_XUNG (vừa, hệ thống)**: glossary chốt các danh xưng thành tên riêng: 秦姨 = Tần di (30 lần), 黄阿姨 = Hoàng a di, 林老师 = Lâm lão sư, 夏老爷子 = Hạ lão gia tử.
   - Đề xuất: "dì Tần", "cô Hoàng", "cô giáo Lâm", "ông cụ Hạ".
9. **AN_THAN_TU (vừa, hệ thống)**: 嗯 → "Ân" 103/129 lần, trong 90 file. Có 3 lần rơi vào cảnh hiện đại, ví dụ ch.282 L113 "Ân, một lần là đủ rồi." → nên là "Ừ, …".
10. **TIEU_DE (vừa, hệ thống)**: cùng một tên arc nhưng dịch nhiều kiểu.
    - 天煞孤星摄政王: "Nhiếp chính vương thiên sát cô tinh" (22 chương) và "Thiên Sát Cô Tinh Nhiếp Chính Vương" (38 chương, viết hoa mọi chữ).
    - 糙汉将军: 2 kiểu. 风流女帝: 2 kiểu. 风流俏寡妇: 4 kiểu.
    - Ghi chú tác giả 补字数 bị đưa vào tiêu đề thành "(bù số chữ)".
11. **GLOSSARY_PHIEN_AM (vừa)**: 孪生丹 = "Lao Sinh Đan" nhưng 孪生子丹 = "Loan Sinh Tử Đan" (đúng phải là "loan"). 江眉翎 = "Giang Môi Linh" (đúng phải là Mi Linh).
12. **XUNG_HO_TROI (thấp)**: ch.435 L181 "Sư tỷ, bất kể nàng muốn làm gì, ta đều bồi nàng."
    - Đề xuất: "…ta cũng ở bên tỷ."
    - Câu gốc vừa gọi "sư tỷ" vừa gọi "nàng"; "bồi" là calque của 陪.
13. **SOT_Y (thấp)**: ch.280 L141 `跑到正妻头上拉屎拉尿` → "trèo lên đầu chính thê mà tác oai tác quái". Câu chửi thô bị làm nhẹ, lại dùng "chính thê" là từ cổ trang.
14. **CALQUE (thấp)**: ch.435 L21 "cho hắn một phần sắc mặt" → "cho hắn chút mặt mũi là được đằng chân lân đằng đầu". Riêng "đem" xuất hiện 180 lần/134 file.
15. **LAP_TU (thấp)**: ch.435 L53 "dở sống dở dở chết".

## Gợi ý

- **Setting theo arc:** cho story.json khai báo setting riêng cho từng arc (ví dụ ch.263–302 là modern), kèm cờ "khung hiện đại" cho đoạn bệnh viện. Khi đó prompt mới bật bộ luật hiện đại.
- **Lọc addressing tự sinh:** ở arc hiện đại, chặn hoặc cảnh báo các giá trị cổ trang (tỷ, ta, tiểu thư, X phụ/X mẫu, X di, lão sư). Thêm bước tách danh xưng ra khỏi tên riêng.
- **Chốt đại từ người kể:** ghi rõ đại từ cho nữ chính theo từng arc, và thêm check đếm "nàng/cô" theo từng chương.
- **Bỏ ép "Ân":** cho dùng "Ừ/Ừm/Vâng" tuỳ vai; ít nhất cấm "Ân" ở cảnh hiện đại.
- **Tiêu đề:** chạy lại tính năng thống nhất tiêu đề lặp (commit cd752eb) cho truyện này, và bỏ ghi chú tác giả khỏi tiêu đề.
- **Phiên âm glossary:** kiểm tra lại bằng từ điển Hán Việt và gộp các mục trùng nhau.
- **Lặp chữ:** thêm check bắt lặp chữ liền nhau như "dở dở", trừ các trường hợp hợp lệ như "được được được".

## Làm tốt

- Lời kể cổ trang và tu tiên trơn, nhịp nhanh, đúng chất sảng văn nữ tần.
- Meme dịch có hồn: 装逼 → "làm màu", 母猪系统 → "hệ thống heo nái", 躺平摆烂 → "nằm ườn buông xuôi".
- Thoại của hệ thống Tiểu Phúc Tử nhí nhảnh, đúng toneRules.
- Không sót chữ Hán nào trong 436 file; số liệu đúng; số dòng khớp với raw.
- Xưng hô cung đình (bản vương, nô tỳ, lão nô, trẫm–nàng) nhất quán.
