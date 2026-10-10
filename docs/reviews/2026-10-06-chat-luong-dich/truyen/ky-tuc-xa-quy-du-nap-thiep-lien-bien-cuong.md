# Review: Ký Túc Xá Quỷ Dữ: Nạp Thiếp Liền Biến Cường

- Slug: `ky-tuc-xa-quy-du-nap-thiep-lien-bien-cuong` · setting `modern` · kể ngôi 3 · 1051 chương đã dịch
- Thể loại đọc được: mạt thế sinh tồn kết hợp game hệ thống trong ký túc xá đại học, về sau thành chiến tranh giữa các tòa/Titan/ma tộc, có arc "Đại Võ Hiệp"; giọng hài, sảng văn, harem.
- Đã đọc trọn 4 chương: 0001, 0526, 1050, 1051 (hai chương cuối là hai chương dịch gần nhất). Đã grep toàn bộ out/ để đếm các lỗi lặp lại.

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 7 | Đại từ hiện đại tự nhiên, không có "ngươi". Nhưng Đồng Khởi tự xưng lúc "tôi" lúc "em" với Hứa Lãng, còn các Titan ngang hàng lúc "tôi–cậu" lúc "tao–mày". |
| Giọng văn | 7.5 | Hợp thể loại, bảng hệ thống gọn. Điểm trừ: "lâu chủ" mang giọng kiếm hiệp, vài cụm Hán Việt nặng ("Tâm linh chi lực"). |
| Dễ hiểu | 8 | Đọc trơn, các đoạn khớp 100% với bản gốc. Còn lẻ tẻ câu "Theo…", "bản thân", thành ngữ dịch lệch. |
| Khó khăn | 7.5 | Meme trong phòng chat dịch khéo. Nhưng 呵 bị đổi thành "Hừ", và chữ lặp gây cười 又又又又 bị dịch phẳng. |
| Thuật ngữ | 6 | Tên nữ chính bị phiên âm sai 1602 lần. 楼主, 祖血, 模拟梦境, 巨人 mỗi từ dịch nhiều kiểu. Glossary phình to nhưng vẫn thiếu thuật ngữ cốt lõi. |

## Lỗi (nặng → nhẹ)

1. **GLOSSARY_PHIEN_AM, cao, hệ thống (1602 lần/400 file).** 童绮 → "Đồng Khởi". Chữ 绮 đọc Hán Việt là "Ỷ", nên tên đúng là **Đồng Ỷ**. Glossary tự sinh đã khóa sai tên một nữ chính xuyên suốt truyện.
2. **XUNG_HO_TROI, cao, hệ thống.** Ở 0547:17, Đồng Khởi gọi "Anh Lãng" nhưng lại xưng "tôi" ("Tuệ Tuệ có hỏi tôi…"). Đến 0567:43 cô lại xưng "Em cảm thấy…". Đếm cả truyện được khoảng 16 câu xưng "tôi" và 13 câu xưng "em". Glossary chỉ ghi "tôi–anh"/"tôi–cô", không có mốc đổi xưng hô và cũng không có cast.json. → Nên chốt "em–anh".
3. **THUAT_NGU_TROI, vừa.** Ở 1051:61, 祖血池 / 祖血 → "Hồ Huyết Tổ" / "Huyết Tổ". Glossary đã chốt "Tổ Huyết Trì" (các chương 1029/1031/1035 có dùng), vậy mà chương này không theo. Thêm nữa, đảo thành "Huyết Tổ" làm đổi nghĩa sang "thủy tổ huyết tộc". → "hồi sinh trong Tổ Huyết Trì… bị tổ huyết ăn mòn".
4. **DINH_DANG, vừa, hệ thống.** Ngoặc thoại không thống nhất: 177 file chỉ dùng ngoặc thẳng `"`, 119 file trộn hai kiểu, 874 file dùng “ ”. Ví dụ 0526:1 `"Quỷ Trảm!"`.
5. **HU_HA, vừa, hệ thống (53 trên 71 câu có 呵 đứng riêng).** Ở 1051:1, “呵，这五个人，真是有意思…” → “Hừ, năm người kia thật thú vị…”. Đây là tiếng cười giễu, nên dịch "Ha, …".
6. **THUAT_NGU_TROI, vừa, hệ thống.** 楼主 dịch thành "lâu chủ" 166 lần, "chủ tòa" 29 lần, "trưởng tòa" 1 lần. Từ này không có trong glossary, và "lâu chủ" mang giọng kiếm hiệp. Ví dụ 0526:43 "Chào lâu chủ!" → "Chào chủ tòa!".
7. **XUNG_HO_VAI_VE, vừa.** Ở 1050:24, các Titan cổ xưa gọi nhau là "老东西" mà lại nói "chúng tôi đến tìm cậu". Trong khi đó glossary ghi 钢铁↔海神 là "tao–mày". → "bọn ta đến tìm ông bàn chuyện".
8. **BAN_THAN, vừa, hệ thống (1562 lần/764 file).** Ở 1050:43, "Chuyện bản thân lén lút liên lạc…" → "Chuyện hắn lén liên lạc…". Lỗi tương tự ở 0526:21 và 0894.
9. **VAN_KHO, vừa.** Ở 1051:43, 又又又又一次 → "lại thêm một lần nữa". Cụm lặp này xuất hiện 5 lần, 3 lần bị dịch phẳng, trong khi tiêu đề chương 0685 lại giữ được "lại lại lại lại".
10. **CALQUE, thấp.** Ở 0526:63, "Theo ánh mắt tăng lên, tiếng bàn tán… nổi lên theo" → "Ánh mắt đổ dồn càng nhiều, tiếng bàn tán càng rộ lên". Cũng gặp ở 0794, 0841 ("Theo cái mở mắt của Hứa Lãng"), 0976.
11. **SAI_NGHIA, thấp.** Ở 1051:46, 一触即发 → "Đại chiến hết sức căng thẳng", nên là "Đại chiến chực bùng nổ". Ở 0526:23, 似是而非 → "nửa đúng nửa sai", nên là "vừa giống vừa khác".
12. **CALQUE, thấp.** Ở 0001:89, "mày Hứa Lãng nhíu chặt" → "Hứa Lãng nhíu chặt mày".
13. **GLOSSARY_HAN_VIET_NANG, thấp.** Ở 1050:41, "Tâm linh chi lực" → "Sức mạnh tâm linh".
14. **THUAT_NGU_TROI, thấp.** 模拟梦境 dịch thành "Mộng Cảnh Mô Phỏng" 92 lần, viết thường 28 lần, "giấc mơ mô phỏng" 13 lần. 巨人 dịch thành "cự nhân" 153 lần, "người khổng lồ" 43 lần.
15. **GLOSSARY_TRUNG, thấp.** Glossary có 3445 mục và 16 nhóm mục trùng bản dịch. Tên 钱桐 "Tiền Đồng" trùng với 铜币 "tiền đồng" (520 lần). Glossary còn nhận cả cụm thường như "cửa gỗ tồi tàn", "tòa ký túc xá".
16. **DINH_DANG, thấp.** Dấu ba chấm bị viết thành "......" (6 chấm) 3862 lần.

## Gợi ý sửa prompt/quy trình

- Auto-glossary: đối chiếu từng chữ với bảng Hán Việt khi thêm tên riêng và cảnh báo khi lệch. Sửa 童绮 → Đồng Ỷ rồi thay hàng loạt.
- Chốt thuật ngữ cốt lõi theo tần suất trong raw: 楼主 "chủ tòa", 巨人 "người khổng lồ", 祖血 "tổ huyết". Chặn auto-glossary nhận cụm mô tả thường và mục trùng bản dịch.
- Lập cast.json có mốc xưng hô (童绮→许浪 "em–anh"; các Titan ngang hàng dùng chung một kiểu).
- Thêm hậu xử lý tất định: đổi ngoặc thẳng thành “ ”, "......" thành "…". Thêm check rule báo khi một file trộn hai kiểu ngoặc.
- Prompt: 呵 dịch theo ngữ cảnh, không mặc định "Hừ"; câu có 随着 thì viết lại, không mở đầu bằng "Theo…"; giữ chữ lặp gây cười; 自己 trong độc thoại dịch là "hắn/mình".
- Check rule báo khi mục glossary bị viết thường hoặc bị thay bằng từ đồng nghĩa.

## Làm tốt

- Các đoạn khớp 100% với bản gốc ở cả 1051 chương, không sót chữ Hán.
- Bảng hệ thống và thuộc tính gọn, đúng văn phong game.
- Meme và chơi chữ trong chat dịch khéo ("ôm chân Phật… lũ cuồng chân", "tụt sạch điểm SAN").
- 嗯 không bị dịch thành "Ân" mà thành Ừ/Ừm/Vâng hợp cảnh; không dùng "ngươi" sai bối cảnh.
- Tiêu đề chương đúng định dạng, không viết hoa mọi chữ.
