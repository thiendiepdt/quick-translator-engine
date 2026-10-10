# Review: Tận Thế: Người Khác Cầu Sinh, Ta Cùng Hầu Gái Đi Nghỉ Dưỡng

- Slug: `tan-the-nguoi-khac-cau-sinh-ta-cung-hau-gai-di-nghi-duong`. Setting `modern`, thực tế là sảng văn mạt thế đô thị + hệ thống hầu gái, kể ngôi ba.
- Đã dịch 164 chương. Đọc trọn 0001, 0083, 0163, 0164 (hai chương cuối là bản mới nhất), có grep toàn bộ out/.

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 7.5 | Trong thân chương ổn. Tiêu đề (titles.json) mặc định "ta" nên sai người nói. Mục addressing trong story.json mâu thuẫn cast.json ở khoảng 11 cặp. |
| Giọng văn | 7 | Giọng đô thị, nhịp nhanh, lóng sống. Vẫn lọt Hán Việt cổ (chấn kinh, điểm hóa, đầu danh trạng, phi gian tất đạo); riêng "chấn kinh" chỉ có ở các chương mới nhất. |
| Dễ hiểu | 7.5 | Phần lớn trôi chảy. 斤 bị dịch thành "cân" nên khối lượng gấp đôi; vài câu què hoặc bị đảo nghĩa. |
| Khó khăn | 7.5 | Chửi thề và thán từ tốt. Còn meme dịch nghĩa đen, 呵 bị dịch thành "Hừ". |
| Thuật ngữ | 7 | Tên nhất quán, không sót chữ Hán. Glossary tự sinh có mục trùng, có nhân vật không tồn tại, danh xưng bị ghi như tên riêng, Hán Việt lệch bối cảnh. |

## Lỗi (nặng → nhẹ)

1. **SAI_SO (cao, hệ thống: 17 lần 斤 trong 10 file, 15 dòng dịch thành "cân")**. Ví dụ 0001:107: `一百斤重压` → "sức nặng một trăm cân" → nên là "năm chục ký". 1 斤 = 0,5 kg nên mọi số liệu bị gấp đôi. Riêng 0114 còn dịch `两百斤` thành "cả trăm cân".
2. **TIEU_DE / xưng hô (cao, hệ thống: 7/48 tiêu đề)**. Ví dụ 0119: `苏小苏，我最近表现不好？` → "Tô Tiểu Tô, dạo này ta thể hiện không tốt sao?" Người nói là Tô Tiểu Tô, thân chương xưng "em". Tương tự 0133 "Gọi ta một tiếng chị Tiểu Tô", 0135 "Chủ nhân nhà ta" (thân chương là "nhà chị"), 0118 "Ta không thích…" (thân chương là "tôi"). Nguyên nhân: tiêu đề được dịch tách khỏi chương.
3. **CAST_MAU_THUAN (cao, hệ thống)**. story.json ghi `许依依→陈南星 tôi–anh`, `刘初夏→陈南星 tôi–cậu`, `柳如烟→白凝冰 em–chị`, `许依依→林若曦 tôi–cô`. cast.json lại có mốc đổi thành em–chủ nhân, tao–mày, em–chị. Hai nguồn nói ngược nhau, nên Lâm Nhược Hi lúc gọi chủ nhân là "anh", lúc là "ngài".
4. **BOI_CANH_SAI (vừa, hệ thống)**. Ví dụ 0163:151: `无事献殷勤，非奸即盗` → "vô sự hiến ân cần, phi gian tất đạo" → nên là "tự dưng ân cần thế này, chẳng gian cũng trộm". Các trường hợp khác: "chấn kinh" (4 lần/4 file, toàn bộ ở 0158–0164), "điểm hóa", "đầu danh trạng", "tứ chi bách cốt", "hàn quang" (11 file). Tất cả trái style.avoid.
5. **DAI_TU_MO_HO (vừa, lẻ)**. 0083:139: `是陈南星“情敌”的女人` → "Là người phụ nữ của 'tình địch' Trần Nam Tinh!" Câu này đọc thành Trần Nam Tinh là tình địch, tức đảo nghĩa. Nên viết rõ là vợ của tình địch của Trần Nam Tinh.
6. **CALQUE (vừa, hệ thống: 14/14 lần 毒死, 9 file)**. Ví dụ "Dám muốn độc chết tao và chủ nhân à?" → nên là "Dám đầu độc tao với chủ nhân à?"
7. **SAI_NGHIA (vừa)**. 0083:151: `空口白牙` → "nói mồm không bằng không chứng" → nên là "chỉ nói suông không bằng chứng".
8. **CALQUE (vừa)**. 0164:83: `虚不受补` → "hư không chịu nổi bổ" → nên là "người đang suy mà ăn đồ bổ quá thì không chịu nổi".
9. **HU_HA (thấp, hệ thống: 8/42 dòng có 呵, 7 file)**. Ví dụ 0049:91: `呵，真是好算计` → "Hừ, tính toán hay lắm." → nên là "Ha, tính toán hay thật đấy." 呵 là cười khẩy, không phải hừ mũi.
10. **THUAT_NGU_TROI (thấp)**. Glossary có cả 沙漠之鹰=Desert Eagle và 沙鹰=Sa Ưng cho cùng một khẩu súng. Ví dụ 0070:41 "khẩu Sa Ưng", trong khi các chương khác dùng "Desert Eagle".
11. **MEME_NGHIA_DEN (thấp)**. 0164:55: `被秒成渣渣` → "bị đánh cho thành cặn bã trong một giây" → nên là "bị hạ gọn trong một nốt nhạc".
12. **GLOSSARY_TRUNG (thấp)**. 0061:17: `白若曦` là lỗi gõ của tác giả (đúng ra là 林若曦). Auto-glossary lại tạo thêm nhân vật "Bạch Nhược Hi" trong names và cast.json.
13. **BAN_THAN (thấp, 155 lần/90 file, một phần dùng đúng)**. Ví dụ 0025:69 "sau khi bản thân ra ngoài" → nên là "lúc mình ra ngoài".
14. **DINH_DANG (thấp)**. 21 file (0041, 0141–0164) dùng ngoặc thẳng "…", 143 file dùng ngoặc cong “…”. Dấu "......" nên chuẩn hoá thành "…".
15. **GLOSSARY_HAN_VIET_NANG (thấp)**. 强哥 → "Cường ca" (46 lần/4 file), 丧尸 → "tang thi". Bối cảnh hiện đại nên dùng "anh Cường", "xác sống/zombie".

## Gợi ý

- Thêm luật đơn vị: 斤 = 0,5 kg, 两 = 50 g; thêm check tự động bắt "cân" khi raw có 斤.
- Dịch tiêu đề cùng thân chương, hoặc đưa ngữ cảnh thoại và cast vào bước dịch titles.json. Với setting modern thì cấm mặc định "ta".
- Khi cast.json có mốc đổi xưng hô thì ghi đè hoặc đánh dấu stale mục tĩnh tương ứng trong story.json. Chỉ nạp một nguồn vào prompt.
- Auto-glossary cần: gộp alias cùng một vật; không tạo nhân vật từ tên xuất hiện 1 lần mà gần trùng tên nhân vật chính; không đưa danh xưng (张大爷, 李老师, 张大姐) vào names; với modern thì chặn "ca/huynh/tang thi".
- Thêm vào prompt modern danh sách Việt hoá: 震惊 → sửng sốt, 点拨 → nhắc khéo, 投名状 → quà ra mắt, 四肢百骸 → khắp người, 分明 → rõ ràng, 毒死 → đầu độc, 呵 → "Ha/Hứ".
- Hậu kỳ chuẩn hoá ngoặc kép cong và "…" cho toàn bộ out/.

## Làm tốt

- Đủ đoạn, không sót chữ Hán, không có "Ân" hay "ngươi". Tiêu đề không viết hoa mọi chữ.
- Lóng và chửi thề sống: "Vãi!", "Mẹ nó", "Bà đây một bạt tai đập chết mày!", "tác giả cùi bắp", "cạn lời".
- Xưng hô giữa hầu gái với nhau và với chủ nhân trong thân chương ổn định, đúng các mốc trong cast.json.
- Lời kể ngôi ba nhanh, giữ được giọng mỉa mai. Chương 1 gần như không cần sửa.
