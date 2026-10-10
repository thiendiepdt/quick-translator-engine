# Review: Ta dưỡng thành các nữ thần điện tử, chế bá đô thị

- slug: `ta-duong-thanh-cac-nu-than-dien-tu`. Setting trong story.json: `modern`. Thực tế truyện nhiều arc: hiện tại 2025, 2008, Hồng Kông thập niên 70–90, Minh triều, và arc mới nhất là thời Dân Quốc.
- Ngôi kể: ngôi 3. Đã dịch 718 chương. Đã đọc trọn các chương 0001, 0360, 0717, 0718; grep toàn bộ out/ (raw và out khớp từng đoạn ở cả 718 file).

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 7.5 | Lời kể đúng hắn/cô. Thoại arc Dân Quốc tốt. Lâm Nhược Huyên gọi "Bạo Quân" mà tự xưng lúc tôi, lúc em ngay trong một chương. Addressing chỉ ghi cặp của arc mới nhất, có tên gõ sai, không có cast.json. |
| Giọng văn | 8 | Đô thị trẻ, sảng khoái. Arc Dân Quốc đổi sang giọng trang nhã, hợp thời đại. Còn rải rác Hán Việt nặng: khí trường, khuê các nữ tử, thư trai. |
| Dễ hiểu | 8 | Mượt, đủ ý. Lỗi lẻ: calque, một thành ngữ dịch sai, "hắn" mơ hồ khi 自己 nằm trong suy nghĩ của nhân vật khác. |
| Khó khăn | 7.5 | Phần lớn lóng mạng được Việt hoá sống động. 呵 vẫn bị dịch thành "Hừ" khoảng 85% số lần, vài meme dịch nghĩa đen hoặc bị làm nhạt. |
| Thuật ngữ | 6.5 | Glossary hơn 2.200 mục, áp dụng nhất quán, nhưng có phiên âm sai lan rộng, tên lai Hán Việt với thuần Việt, danh xưng tỷ/chị trôi. |

## Lỗi (nặng → nhẹ)

1. **GLOSSARY_PHIEN_AM (cao, hệ thống, 34 lần/10 file).** 0717 dòng 3: `离开鼓浪屿` → "rời Cổ Lãng Tự" → đề xuất "rời đảo Cổ Lãng Dữ". 屿 đọc là "dữ"; "Tự" là chùa. Mục sai bị khoá trong glossary, kể cả item "Tạm Biệt Cổ Lãng Tự".
2. **XUNG_HO_TROI (cao, hệ thống ở ch.0003–0013).** 0008 dòng 29: `我会加倍努力…你等我！` → "Tôi sẽ nỗ lực… Anh đợi tôi nhé!", trong khi dòng 67 cùng chương lại là "Bạo Quân của em". Đề xuất "Em sẽ… Anh chờ em nhé!".
3. **CAST_MAU_THUAN (cao).** story.json có 14 mục addressing, tất cả thuộc arc Trương Huy Giáng. Hai mục ghi sai tên `徐徽绛`. Không có cặp nào của các nữ chính chính (Lâm Nhược Huyên, Tống Giai Như…), không có cast.json. Có vẻ addressing bị ghi đè theo arc.
4. **BAN_THAN (vừa, hệ thống, 720 dòng/397 file, khoảng 20% số câu có 自己).** 0360 dòng 5: "rằng bản thân là người Đại Hạ" → "rằng mình là người Đại Hạ". Cùng chương còn "Bản thân chỉ trả lời một câu hỏi".
5. **HU_HA (vừa, hệ thống, 25/29 dòng mở bằng 呵).** 0360 dòng 127: `呵。那今天就让你见识见识` → "Hừ. Vậy hôm nay sẽ cho đối phương thấy…" → đề xuất "Hà. Vậy hôm nay để mày mở mang tầm mắt…". Bản dịch vừa sai sắc thái, vừa mất lời thách thẳng mặt.
6. **THUAT_NGU_TROI (vừa, hệ thống).** "Vi tỷ" 51 lần nhưng "chị Vi" 7 lần, cho cùng `微姐`: 0450 dòng 123 viết "Gọi là chị Vi!", 0453 dòng 17 viết "Vi tỷ có biết Thắng ca…". "Lương tỷ" và "chị Lương" cũng lẫn lộn.
7. **SAI_NGHIA (vừa, lẻ).** 0717 dòng 9: `纸醉金迷的上流社会` → "chốn thượng lưu vàng thau lẫn lộn say đắm trụy lạc" → "giới thượng lưu xa hoa, ăn chơi phè phỡn".
8. **DAI_TU_MO_HO (vừa, lẻ).** 0360 dòng 123: `跪在自己面前…自己这种人` → "quỳ gối trước mặt hắn… Loại người như hắn". Cả nhị công tử lẫn Từ Vân Chu đều là "hắn" → nên viết "trước mặt mình… Hạng người như mình".
9. **GLOSSARY_PHIEN_AM (vừa).** 0718 dòng 133: `三坊七巷` → "Tam Phường Thất Hẻm" → "Tam Phường Thất Hạng".
10. **GLOSSARY_HAN_VIET_NANG (thấp, 16 lần/14 file).** `黄家花园` → "Hoàng Gia Hoa Viên", dễ đọc thành "hoàng gia" → "vườn nhà họ Hoàng".
11. **CALQUE (thấp).** 0360 dòng 33: "suốt năm ngàn năm trên dưới đều có người" → "có người khắp năm nghìn năm lịch sử".
12. **CALQUE (thấp, 6 lần/6 file).** "chiếm tiện nghi" (占便宜) → tuỳ ngữ cảnh: sàm sỡ / lợi dụng / được lợi.
13. **MEME_NGHIA_DEN (thấp).** 0001 dòng 117: `无中生友` → "'vô trung sinh hữu', bịa ra bạn" → "Lại là kiểu 'tôi có một người bạn' đấy hả?".
14. **XUNG_HO_NGUOI_KE (thấp, 9 lần/5 file).** "anh ta" xuất hiện trong lời kể (0130 dòng 167), trái với toneRules của chính truyện.
15. **TIEU_DE (thấp).** 29 tiêu đề viết "Chương N Tên", 689 tiêu đề viết "Chương N: Tên".

## Gợi ý

- Auto-glossary: kiểm tra phiên âm địa danh thật theo từ điển Hán Việt (屿 = dữ, 巷 = hạng). Cấm tên lai Hán Việt với thuần Việt. Không nhận từ thường làm item (时代→Time, 自然→Nature, 少年, 活着, 童年). Chốt một chính sách cho tên chơi chữ: hiện 敦伦→London, 呆北→Đài Bắc nhưng 林凉 vẫn giữ "Lâm Lương".
- Addressing: không ghi đè theo arc. Giữ các cặp xuyên truyện, có mốc đổi xưng hô. Kiểm tra tên trong addressing phải có trong glossary.names.
- Prompt: 自己 làm chủ ngữ hoặc tân ngữ thì dịch "mình" hoặc bỏ. Thán từ 呵 dịch "Hà/Hờ" hoặc tả "cười khẩy"; chỉ 哼 mới là "Hừ".
- Danh xưng 姐/哥: chốt theo bối cảnh và không lưu như tên riêng. Gộp mục trùng 芝姐/之姐.
- Cho setting theo dải chương (modern / HK retro / Dân Quốc) thay vì một giá trị `modern`.
- Chuẩn hoá tiêu đề "Chương N: …".

## Làm tốt

- Khớp 100% đoạn, không sót chữ Hán, không có "ngươi" chỉ người chung chung, 嗯 dịch thành Ừ/Ừm.
- Arc Dân Quốc rất đạt: thầy trò em–thầy, kế mẫu mẹ–con, thơ 《雨巷》 thành "Ngõ mưa".
- Lóng mạng sống động: liếm cẩu, bố hệ thống, phim con heo Đông Doanh.
- Lời kể giữ hắn/cô nhất quán. Số liệu tiền chính xác.
