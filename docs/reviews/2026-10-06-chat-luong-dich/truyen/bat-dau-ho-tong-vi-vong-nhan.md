# Review: Bắt Đầu Hộ Tống Vị Vong Nhân, Lấy Được Kim Cương Bất Hoại

- Slug: `bat-dau-ho-tong-vi-vong-nhan` · setting `ancient` · võ hiệp cổ trang có hệ thống, kể ngôi 3
- 312 chương đã dịch. Đọc trọn: 0001, 0156, 0311, 0312 (hai chương cuối là hai chương mới dịch nhất). Lỗi nghi có tính hệ thống đã được grep trên toàn bộ out/.

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 7.5 | Các cặp chính hợp lý, lời kể không dùng anh/em/tôi. Bị trừ vì Vô Tướng công tử bị gọi lần lượt gã→hắn→y trong một cảnh, có chỗ hai chữ "y" liền nhau chỉ hai người, và cast.json gần như rỗng. |
| Giọng văn | 7.5 | Giọng võ hiệp gọn. Còn Hán Việt nặng (lão ngưu, sở sở khả liên, nhược nữ tử) và mẫu "đem…". |
| Dễ hiểu | 7.5 | Đọc trôi, ít sót ý. Có lỗi nghĩa (放肆, 一倍, 榜首) và cụm dịch từng chữ. |
| Khó khăn | 7 | Thoại hài giữ được nhịp. Thán từ khô ("Ân?", 呵 dịch thành "Hừ"), thành ngữ dịch theo nghĩa đen. |
| Thuật ngữ | 6 | Glossary lớn nên tên phần lớn nhất quán. Nhưng chỗ tác giả gõ nhầm bị tách thành hai tên, cấp sao trôi giữa hai cách dịch, danh xưng bị nhận làm tên, có mục rác "Qidian". |

## Lỗi (nặng → nhẹ)

1. **THUAT_NGU_TROI (cao, hệ thống)**. Tác giả gõ nhầm 陆云停/陆停云, glossary tạo hai mục nên ra hai tên: "Lục Đình Vân" (45 lần/6 file) và "Lục Vân Đình" (64 lần/15 file). Ví dụ 0255:5 `陆停云说话时…` → "Lục Đình Vân khi nói chuyện…", nên sửa thành "Lục Vân Đình…". 毒蛇坳/拗 cũng bị tách: Độc Xà Ao (12 lần) / Độc Xà Áo (4 lần).
2. **SAI_SO (cao)**. 0312:143 `多了将近一倍` → "nhiều hơn gần một nửa". Đúng phải là "nhiều gần gấp đôi".
3. **AN_THAN_TU (vừa, hệ thống, 127 lần/94 file)**. 0001:143 `嗯？` → "Ân?". Nên dùng "Hửm?"; khi đáp ứng thì "Ừ./Được.".
4. **SAI_NGHIA (vừa)**. 0311:29 `放肆！` → "Phóng túng!". Nên là "Láo xược!", vì "phóng túng" trong tiếng Việt là buông thả, trác táng.
5. **DAI_TU_MO_HO (vừa)**. 0312:63–65: "Y chậm rãi thở ra…" (chỉ Vô Tướng), ngay sau đó "còn chưa đợi y kịp có động tác" (chỉ kiếm khách). Nên gọi tên thay cho đại từ.
6. **THUAT_NGU_TROI (vừa, hệ thống)**. Cấp sao của hệ thống dịch hai kiểu: "một/hai/ba/bốn sao" (6 lần/5 file) và "nhị tinh/tam tinh" (4 lần/3 file). Ví dụ 0021:157 "(Chu Uyển Thanh, nhị tinh)", nên là "hai sao". Mẫu câu trong khung 【任务目标】 cũng đổi trật tự giữa các chương.
7. **HU_HA (vừa, hệ thống)**. 呵 xuất hiện 121 lần ở raw và hay bị dịch thành "Hừ" (tổng 33 chữ "Hừ"/30 file). Ví dụ 0001:351 `呵，时间？` → "Hừ, thời gian?", nên là "Hà, thời gian à?".
8. **GLOSSARY_HAN_VIET_NANG (vừa, hệ thống, 213 lần/78 file)**. Glossary ép 老牛 → "lão ngưu". Ví dụ 0001:109 "Lão ngưu đi rất chậm", nên là "Con bò già đi rất chậm".
9. **CALQUE (vừa, hệ thống)**. "đem" 92 lần/72 file, "Theo việc/sự" 10 lần. Ví dụ 0008:21 "không kìm được đem lời trong lòng nói ra", nên là "…nói ra lời trong lòng".
10. **GLOSSARY_DANH_XUNG (vừa, hệ thống)**. cast.json có hai nhân vật 龙姑娘 và 龙青青 nhưng thực ra là một người. Glossary còn chứa 刘夫人, 杨姑娘, 周大夫, 孙管家, 顾长老, 老王…
11. **VAN_KHO (thấp)**. 0311:81 "Nhìn mỹ nhân sở sở khả liên… thái độ… đều mềm lòng xuống", nên là "mỹ nhân yếu đuối đáng thương… ai nấy đều mềm lòng". Cụm "kẻ nhược nữ tử/kẻ phụ nhân" xuất hiện 7 lần trong 2 chương mới nhất.
12. **MEME_NGHIA_DEN (thấp)**. 0001:353 `喝西北风` → "húp gió tây bắc", nên là "hít gió mà sống à?".
13. **CALQUE (thấp)**. 0312:177 `四分五裂` → "bốn mảnh năm mảnh", nên là "tan thành từng mảnh". Cùng kiểu với "ba năm năm năm" (三年五年) và "Ba ngày thời gian".
14. **KHAC (thấp, hệ thống)**. Lời tác giả và đơn xin nghỉ (0095, 0232, 0267, 0306) bị dịch như chương truyện. Từ 0095, glossary còn lấy thêm mục `起点 → Qidian`; chuỗi 起点 cũng nằm trong "一起点灯" ở chương 0103.
15. **TIEU_DE (thấp)**. Khoảng 20 tiêu đề viết hoa mọi chữ, ví dụ "Chương 14: Mưa Chẳng Tưới Cỏ Không Rễ". Các chương 0146–0149 thiếu dấu ":".

Lỗi lẻ khác: "một giây trước" (一秒) ở 0311:135, dùng đơn vị hiện đại trong bối cảnh cổ; 榜首 → "tân thủ lĩnh Tiềm Long Bảng" ở 0156:203; Mai Hoa phu nhân trộn "ta/thiếp thân" trong cùng một lời thoại; dấu ngoặc kép lúc dùng `"`, lúc dùng `“”` (4 file trộn cả hai).

## Gợi ý

- Glossary tự sinh: dò biến thể gần trùng (đảo chữ 云停/停云, chữ đồng âm 坳/拗) rồi gộp về một đích, đồng thời báo cho người dùng.
- Đưa thuật ngữ hệ thống vào glossary common: 一星… → "một sao…", kèm mẫu câu cố định cho khung 【】.
- Không nhận mục "họ + danh xưng" làm tên riêng, cả trong glossary lẫn cast.json.
- Bỏ luật ép 嗯 → "Ân". Chỉ 哼 mới dịch là "Hừ"; 呵 là cười nhạt.
- Đổi 老牛 thành "con bò già". Liệt kê trong prompt những thành ngữ không được phiên âm, và cấm mẫu "đem X + động từ".
- Phát hiện chương lời tác giả (感言, 请假, 上架) để bỏ qua, không trích thuật ngữ từ đó.
- Thêm luật đại từ cho cảnh có hai nhân vật nam: gọi tên thay vì dùng hai lần "y"/"hắn". Cấm đơn vị "giây" trong bối cảnh cổ.
- Hậu kỳ: chuẩn hoá dấu ngoặc kép và cách viết hoa tiêu đề.
- Ghi mốc xưng hô vào cast.json. Ví dụ Lý Tú Nương: ta–Lâm huynh đệ ở chương 1 → công tử ở chương 53 → thiếp–Lâm lang về sau.

## Làm tốt

- Không sót chữ Hán. Tuân thủ style.avoid: không có vợ/chồng/cư nhiên.
- Thoại hài giữ được nhịp: "Con gái con lứa", "nhạt toẹt"; cặp cha–con Minh Nguyệt và Vô Trần nghe tự nhiên.
- Xưng hô vãn bối–tiền bối chuẩn, 你 được chuyển thành "ngài" khi nói với người trên.
- Tên riêng và chiêu thức phần lớn khớp glossary; cảnh đánh nhau bám sát nguyên tác, ít sót ý.
