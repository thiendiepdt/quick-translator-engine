# Review: Thỉnh Tiên Sinh Cứu Ta (thinh-tien-sinh-cuu-ta)

Setting `ancient`; thực tế là huyền huyễn tu tiên xuyên không, giọng dí dỏm. Truyện kể ngôi 3, đã dịch 1353 chương.
Đã đọc trọn 4 chương, đối chiếu với raw: 0001 (chương đầu), 0677 (chương giữa, nhiều thoại), 1352 và 1353 (hai chương sửa gần nhất).

## Điểm
| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 8 | Hứa Khinh Chu–Tiên giữ ta/nàng–ngươi đều suốt 0677; Hứa Khinh Chu–Vô Ưu là ta–con; Tiểu Bạch gọi "Lão Hứa" đúng glossary. |
| Giọng văn | 7 | Đúng chất tiên hiệp, giữ được nhịp hài; có chỗ Hán Việt nặng ("phần thưởng phong hậu") và "bản thân hắn" dày đặc. |
| Dễ hiểu | 6 | Còn câu convert và vài chỗ sai nghĩa từ đa nghĩa. |
| Khô khan | 7 | Phần lớn còn sống, nhưng thán từ "Ân" và mấy thành ngữ bị dịch phẳng làm thoại đơ. |
| Thuật ngữ | 7 | Phần lớn khớp glossary; riêng 天启 dịch hai kiểu Thiên Khải/Thiên Khởi. |

## Lỗi (nặng → nhẹ)
1. **AN_THAN_TU, hệ thống, nặng.** 0677 dòng 7: `嗯...姑娘为何叹气？` → "Ân... Cô nương vì sao lại thở dài?" → nên là "Ừm... Cô nương sao lại thở dài?"; còn `“嗯？”` đáp lời (dòng 15, 111, 127) → "Hửm?". Toàn bộ out/ có 495 chữ "Ân" và 0 chữ "Ừm". Gốc lỗi: luật 11 (cổ đại) trong `prompts.json` bắt dùng `Ân`, mâu thuẫn với bảng ví dụ giọng văn trong cùng file.
2. **THUAT_NGU_TROI, hệ thống, nặng.** 1352 dòng 89: `天启六神千帝` → "sáu vị thần cùng ngàn vị đế của Thiên Khải" → nên là "Lục Thần cùng ngàn vị đế của Thiên Khởi". Glossary ghi 天启 = Thiên Khởi và 六神 = Lục Thần. "Thiên Khải" có ở 15 file, "Thiên Khởi" ở 57 file; chương 1346 còn đúng, sang 1352 lại lệch.
3. **BAN_THAN, hệ thống.** 0677 dòng 55: `自己这情况…自己已经够努力了` → "Khổ nỗi tình hình của bản thân hắn quả thực có chút đặc thù, vả lại, bản thân hắn đã đủ nỗ lực rồi." → nên là "Khổ nỗi tình cảnh của hắn quả thật đặc thù, vả lại hắn cũng đã đủ cố gắng rồi." Toàn bộ có 475 lần "bản thân hắn"; gốc lỗi là ví dụ mặc định trong luật "Lời kể gián tiếp" của prompt.
4. **LAP_TU, lẻ.** 1353 dòng 57: `此时此刻。` → "Lúc này lúc này." → nên là "Giờ phút này." Lặp từ vô nghĩa; cùng kiểu có ở 0605 dòng 39.
5. **SAI_NGHIA, lẻ.** 0677 dòng 189: `这笔账很容易算明白` → "Món nợ cảnh giới này…" → nên là "Bài toán cảnh giới… tính ra rất dễ". Chữ 账 ở đây là phép tính, không phải món nợ.
6. **SAI_NGHIA, lẻ.** 0677 dòng 209: `欣慰一笑` → "nở nụ cười đầy an ủi" → nên là "mỉm cười mãn nguyện". 欣慰 là thấy được an lòng, không phải đi an ủi người khác.
7. **SAI_NGHIA, lẻ.** 1352 dòng 159: `“看着，很厉害，很强。”` → "Nhìn xem, rất lợi hại, rất mạnh." → nên là "Trông có vẻ... rất lợi hại, rất mạnh." 看着 nghĩa là "trông".
8. **SAI_NGHIA, lẻ.** 1353 dòng 137: `生生将黑暗之剑挡下` → "gắng gượng cản được" → nên là "chặn đứng được". Dịch "gắng gượng" đảo ngược thế mạnh của Thiên Môn.
9. **HU_HA, khá phổ biến.** 1353 dòng 91: `灾玩味的笑道：“呵...` → "Tai cười đầy thích thú: “Hừ..." → nên là "Ha...". Tương tự 1352 dòng 197, tiếng cười tự giễu `呵！` → "hừ!". "Hừ..." có ở 95 file.
10. **CALQUE, lặp lại.** 1353 dòng 111: "cũng chỉ là một cái chỉ tay nhẹ nhàng khi nhấc tay lên" → nên là "chỉ là nhấc tay khẽ điểm một cái". Chương 0001 cũng có kiểu này: dòng 195 "đối diện liền có Lưu thị hôm nọ đi tới", dòng 199 "Hai người đối với Hứa Khinh Chu hết lời cảm tạ".
11. **VAN_KHO, lẻ.** 0677 dòng 39: `哪壶不开提哪壶` → "đừng chuyện gì không muốn nhắc lại cứ khơi ra" → nên là "đừng có xát muối vào chỗ đau được không?". Thành ngữ bị giải nghĩa dài dòng, mất câu cà khịa.
12. **CALQUE, lẻ.** 0001 dòng 3: "Trùng non này có nét nhấp nhô của trùng non này" → nên là "Núi này có cái trập trùng của núi này…".
13. **THUAT_NGU_TROI, lẻ.** 1353 dòng 133 viết thường "thiên môn", các dòng sau lại viết hoa "Thiên Môn".

## Gợi ý sửa prompt / quy trình
1. Sửa luật 11 (thán từ cổ đại) trong `prompts.json`: bỏ yêu cầu bắt buộc `Ân`. Dùng 嗯 → Ừm / Ừ, 嗯？ → Hửm?; giữ `A?` chỉ cho 啊?. Thêm 呵 → "ha / hà" và nói rõ "Hừ" chỉ dùng cho 哼.
2. Viết lại luật 自己 trong lời kể: mặc định dịch "hắn / mình / chính hắn"; "bản thân hắn" chỉ dùng khi cần nhấn mạnh.
3. Thêm vào bước `check`: báo khi raw có key glossary mà bản dịch dùng biến thể khác; bắt cụm từ lặp liền nhau; bắt cùng một thuật ngữ lúc hoa lúc thường.
4. Thêm mục "từ đa nghĩa hay dịch nhầm" vào prompt, gồm 这笔账, 欣慰, 看着, 生生, 哪壶不开提哪壶.

## Chỗ làm TỐT
- Không thấy sót đoạn; khung 【】, tiêu đề "Chương N:" và dấu "——" được giữ nguyên.
- Nhịp hài giữ tốt: "giải ưu cho nàng, không thể giải sầu cho chàng", "Dở hơi.", "điểm danh chuyên cần đủ ngày đủ buổi", "cụ tổ của Thiên Môn".
- Điển cố dịch đẹp: 唯愿孩儿愚且鲁 → "Chỉ nguyện con trẻ khờ và dại, chẳng tai chẳng bệnh bước trường sinh."
- Tên riêng khớp glossary: Nhược Thủy, Thương Minh, Chúng Sinh Hòa, Đế Đài, Côn Bằng, Mộng Yểm.
- Xưng hô sư đồ và quan hệ Hứa Khinh Chu – Tiên nhất quán.
