# Review: Kỳ Quái: Bắt Đầu Đập Tan Bát Xích Phu Nhân

- **Setting trong story.json:** modern. **Thể loại thực tế:** đô thị kinh dị hiện đại kết hợp hệ thống săn quỷ và võ đạo quốc thuật, sảng văn, giọng hài tưng tửng.
- **Ngôi kể:** ngôi 3, nên luật "ta" cho người kể không áp dụng.
- **Đã dịch:** 137 chương.
- **Chương đã đọc trọn:** 0001, 0069, 0136, 0137. Các lỗi nghi là hệ thống đã được grep trên toàn bộ out/.

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 7.5 | Các cặp chính khớp glossary: Sài Vi Mị gọi "sếp–anh", đối đầu thì "tao–mày", nhân vật chính gọi Mộ Dung Thành là "ông". Trừ điểm vì người cha khách hàng cũng bị gọi "hắn", và có chỗ trộn "lão thân" với "tôi" (0128). |
| Giọng văn | 7.5 | Nhịp nhanh, hợp bối cảnh hiện đại, không lạc từ cổ trang. Trừ điểm vì Hán Việt convert ("khủng bố", "cự lực", "khí tức", "vạn thú chi vương") dồn vào các cảnh đánh nhau ở chương mới. |
| Dễ hiểu | 7.5 | Phần lớn câu trôi, ý đủ. Còn vài chỗ sai sắc thái ("hủy diệt nhân đạo") và vài câu giữ trật tự từ tiếng Trung. |
| Khó khăn (lóng, meme) | 7.5 | Lóng xử lý tốt. Trừ điểm vì 苟 thành "cẩu" khó hiểu và 卧槽 bị dịch phẳng. |
| Thuật ngữ | 6.0 | Glossary tự mâu thuẫn (Hàng Ma/Trừ ma), có phiên âm sai, nhiều thuật ngữ bị trôi giữa các chương. |

## Lỗi (xếp từ nặng đến nhẹ)

1. **CALQUE: 恐怖 → "khủng bố"** (cao, hệ thống, 62 lần/22 file, dồn ở 0099–0137)
   - Ví dụ 0136 dòng 17: 恐怖威压 → "uy áp khủng bố hung hãn tàn bạo…"
   - Nên dịch: "luồng uy áp kinh hoàng…"
   - "Khủng bố" trong tiếng Việt mang nghĩa khủng bố chính trị. Lỗi này tăng dần ở các chương mới, tức prompt hiện tại không chặn.
2. **GLOSSARY_TRUNG: văn phòng Hàng Ma/Trừ ma** (vừa, hệ thống)
   - Ví dụ 0136 dòng 87: 降魔事务所 → "văn phòng Hàng Ma". Nên dịch "văn phòng Trừ ma".
   - Glossary có 3 mục cho cùng một nơi: 徐氏降魔事务所 = Trừ ma họ Từ, 降魔事务所 = Hàng Ma, 除魔事务所 = trừ ma.
   - Kết quả: "Hàng Ma" 9 lần/7 file, "đại sư hàng ma" 4 lần, trong khi "trừ ma" 44 lần/17 file.
3. **DINH_DANG: dấu ngoặc lẫn lộn** (vừa, hệ thống)
   - Ngoặc kép thẳng `"…"` xuất hiện ở 46/137 file, trong khi bản gốc chỉ dùng “ ”.
   - Bảng hệ thống dùng `[..]` thay cho 【】 ở 4 file (0048–0051).
   - Ví dụ 0101 dòng 149.
4. **SAI_NGHIA: 人道毁灭** (vừa, 2 file)
   - 0137 dòng 49 dịch thành "hủy diệt nhân đạo"; 0096 dòng 13 dịch thành "tiêu diệt theo chủ nghĩa nhân đạo".
   - Nghĩa đúng là "xử lý nhân đạo / cho chết êm".
5. **MEME_NGHIA_DEN: 苟 → "cẩu"** (vừa, 3 file: 0050, 0063, 0104)
   - Ví dụ 0063 dòng 83: "Hắn đã quen cẩu... à không, quen vững vàng rồi." Nên dịch: "quen rúc đầu... à nhầm, quen chắc ăn rồi".
   - Chương 136 lại dịch rất hay ("nấp một xó chờ đến lúc vô địch"), nên cách xử lý không thống nhất.
6. **TIEU_DE: viết hoa không thống nhất** (vừa, hệ thống)
   - Khoảng 10 tiêu đề viết hoa mọi chữ dù chỉ là cụm từ thường, ví dụ "Kịch Chiến", "Giao Phong", "Lục Đục Nội Bộ", "Tàn Sát". Các chương khác lại viết hoa chữ đầu.
   - titles.json lưu sẵn các bản này nên lỗi bị khoá lại.
7. **GLOSSARY_PHIEN_AM: 小咪 = "Tiểu Mị"** (vừa, 24 lần/8 file)
   - 咪 không đọc là "Mị", và tên mèo dễ lẫn với Sài Vi Mị. Nên dùng "Mimi" hoặc "Tiểu Mi".
8. **XUNG_HO_VAI_VE: người cha khách hàng bị gọi "hắn"** (thấp, ch136–137)
   - Mộ Dung Thành là người cha trung niên đáng thương nhưng bị gọi "hắn", ngang với nhân vật chính và phản diện.
   - Ví dụ 0137 dòng 7: "lúc này hắn cũng toát mồ hôi như tắm". Nên dùng "ông ta".
9. **THUAT_NGU_TROI: 鬼婴** (thấp)
   - 0118 dịch "quỷ sơ sinh", 0137 lại dùng "Quỷ Anh" ở tiêu đề và "quỷ anh" trong thân bài.
10. **GLOSSARY_PHIEN_AM: 撑死鬼 → "Xưng Tử Quỷ"** (thấp, lỗi lẻ ở 0137 dòng 95)
    - 撑 không đọc là "xưng". Cùng câu còn trộn tên Hán Việt với tên thuần Việt. Nên dịch "Quỷ bội thực".
11. **CALQUE: 便宜老爹 → "ông bố hời"** (thấp, 55 lần/18 file)
    - Đây là cách nói của giới convert. Nên chốt một cách thuần Việt như "ông bố trên trời rơi xuống".
12. **THUAT_NGU_TROI: 三线城市** (thấp)
    - Có chỗ dịch "thành phố cấp ba" (0001, 0023, 0128), có chỗ "tuyến ba/tuyến hai" (11 file).
13. **THUAT_NGU_TROI: tiếng hệ thống 叮** (thấp)
    - Phần lớn là "Ting" (142 lần), nhưng có "Đinh" ở 0048 và 0051.
    - Ở 0048, tên nhiệm vụ cũng lệch: "Chân Tướng Hậu Trường", trong khi ch40 dùng "Chân Tướng Sau Màn".
14. **HU_HA: 呵呵 → "Hừ hừ"** (thấp)
    - Ví dụ 0019 dòng 43: tiếng cười hả hê bị dịch thành tiếng hừ khinh bỉ.
    - Cùng một 呵呵 còn có lúc thành "Hơ hơ", "Hắc hắc", "Ha ha".
15. **GLOSSARY_DANH_XUNG: mục addressing sai dạng** (thấp)
    - glossary.addressing và cast.json chứa 老爸 = "bố già" và 宿主 = "ký chủ". Đây không phải cặp A→B.
    - "Bố già" còn mang nghĩa trùm mafia, sẽ sai nếu được áp vào bản dịch.

## Gợi ý

- **Prompt cho setting modern:** thêm danh sách từ cấm kèm từ thay thế:
  - 恐怖: không dùng "khủng bố", thay bằng kinh hoàng / khủng khiếp.
  - 人道毁灭: dịch "xử lý nhân đạo".
  - 苟: không dùng "cẩu", thay bằng rúc đầu / ẩn nhẫn.
- **Bộ sinh glossary tự động:** phát hiện các mục trùng nghĩa (降魔/除魔事务所; 林氏 lúc "họ Lâm" lúc "Lâm Thị") và buộc chốt một bản dịch. Kiểm tra phiên âm bằng từ điển chữ đơn trước khi nạp.
- **Kiểm tra addressing:** chỉ cho phép khoá dạng A→B.
- **Hậu xử lý tự động:** chuẩn hoá ngoặc kép về “ ”, bảng hệ thống về 【】, và tiếng 叮 về một dạng duy nhất.
- **Tiêu đề chương:** chỉ viết hoa chữ đầu và tên riêng, rồi sửa lại titles.json.
- **Đại từ ngôi ba:** "hắn" dùng cho nhân vật chính, phản diện hoặc người ngang hàng; nam trung niên, người lớn tuổi, khách hàng thì dùng "ông ta".
- **Tên quái vật Nhật/Âu:** chốt một luật (giữ tên gốc hay dùng Hán Việt), tránh cảnh "Trinh Tử" đứng cạnh "Kayako" và "Hanako".

## Làm tốt

- Không sót chữ Hán; bảng hệ thống dịch đủ và gọn.
- Lóng và chơi chữ khéo: "vãi chưởng" (挖槽), "con sen" (铲屎官), "ăn chùa" (白嫖), "tiền tỉ dễ gần" (平亿近人), "cưỡi tên lửa" (串火箭).
- Xưng hô theo vai vế rõ ràng; độc thoại của nhân vật chính dùng "mình" đúng toneRules.
- Không có "Ân" cho 嗯, không có "ngươi" lạc vào bối cảnh hiện đại, "bản thân hắn" xuất hiện ít.
- Cảnh đánh nhau dồn dập, giữ được không khí rùng rợn.
