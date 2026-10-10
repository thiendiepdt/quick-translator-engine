# Review: Lão Bà Quá Mạnh, Cùng Ta Cách Li Sinh Sản Làm Sao Bây Giờ

- Slug: `lao-ba-qua-manh-cung-ta-cach-li-sinh-san-lam-sao-bay-gio`. Setting trong story.json: **ancient**. Thể loại đọc được: **đô thị hiện đại có linh khí khôi phục** (Cục Quản lý Di dân, tứ hợp viện, xổ số…) pha tu tiên/Thiên Giới, hài gia đình. Kể ngôi 3. Đã dịch 636 chương.
- Chương đã đọc trọn: 0001 (đầu), 0321 (giữa), 0438 và 0155 (mtime mới nhất). Ngoài ra grep toàn bộ out/.

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 4 | Cặp chính trôi giữa phu quân–thê tử, chàng–thiếp, ông xã–bà xã và anh–em; mẹ xưng ta/ngươi với con; tỷ tỷ và chị lẫn lộn; addressing tự mâu thuẫn |
| Giọng văn | 4.5 | Truyện hiện đại bị khoá cổ đại: nhạc phụ, tiểu di tử, thuộc hạ–ngài trong văn phòng, nữ nhân, Ân |
| Dễ hiểu | 7 | Đúng ý, đủ đoạn; chỉ có calque nhẹ |
| Khó khăn (cảm xúc, hài) | 6.5 | Lóng giữ tốt nhưng chơi chữ và meme bị dịch nghĩa đen |
| Thuật ngữ | 6 | Glossary lớn, áp đều, nhưng có mục tách đôi, sai chính tả, meme bị đóng băng |

## Lỗi chính (nặng → nhẹ)

1. **BOI_CANH_SAI (cao, hệ thống).** Do setting ancient, prompt cấm "vợ/chồng": cả 636 file không có lần nào. 老公 → "Phu quân" 85 dòng/31 file; "thê tử" 360 lần/181 file.
   - ch1 dòng 83: 老公，你回来啦 → "Phu quân, anh về rồi à!" → nên là "Ông xã, anh về rồi à!"
   - Lỗi này đi ngược toneRules của chính story.json.
2. **XUNG_HO_TROI (cao, hệ thống).** Vợ xưng "thiếp" 66 lần/23 file, gọi "chàng" 104 lần/44 file, xen với "em/anh".
   - ch432 dòng 147: "Ân...... phu quân, đều nghe theo chàng." → nên là "Ừm... ông xã, em nghe anh hết."
3. **CALQUE (cao, hệ thống).** Không được dùng "vợ" nên model chế ra từ ghép Hán Việt: nhạc mẫu 71 dòng, nhạc phụ 108 lần, tiểu di tử/đại di tỷ 29 dòng.
   - ch2 dòng 165: "nhạc phụ nhạc mẫu cùng đại cữu ca và tiểu di tử" → nên là "bố vợ, mẹ vợ, anh vợ với em vợ".
4. **XUNG_HO_VAI_VE (cao).** ch438 dòng 19: mẹ mắng con trai và em gái mà xưng "ta… các ngươi".
   - Nên là: "Còn để mẹ bắt gặp hai đứa…"
   - Trái với addressing "mẹ–con".
5. **XUNG_HO_VAI_VE (vừa).** ch5 dòng 151: nhân viên văn phòng nói với trưởng phòng: "Thuộc hạ cùng phu quân… ngài… phu nhân".
   - Nên là: "Em với chồng em… trưởng phòng… chị nhà".
6. **AN_THAN_TU (vừa, hệ thống).** 嗯 → "Ân" 98 dòng/81 file, trong khi chỉ 53 dòng dịch là Ừ/Dạ.
   - ch438 dòng 31: "Ân, về nhà thôi." → nên là "Ừ, về nhà thôi."
7. **XUNG_HO_TROI (vừa, hệ thống).** 姐姐 → "tỷ tỷ" 217 dòng, → "chị" 321 dòng.
   - ch155: "Tỷ tỷ bạo thật!" đứng ngay cạnh "Anh rể đỉnh thật!"
8. **CAST_MAU_THUAN (vừa).** Addressing trong story.json lệch giữa hai chiều:
   - Lưu Nguyên→Tần Nhị gia là "cháu–ông", nhưng chiều ngược lại ghi "bác–cháu".
   - Lưu Niệm→Lưu Tô là "ta–ngươi", nhưng Lưu Tô→Lưu Niệm là "đệ–tỷ".
   - Lưu Tô→Lưu Nguyên ghi "con–ta".
   - Thanh Tuyết và Vi Vi gọi nhau "em" cả hai phía.
9. **XUNG_HO_NGUOI_KE (vừa, hệ thống).** Lời kể gọi Lưu Nguyên là "anh" ~94 lần/55 file, còn lại là "hắn".
   - ch58 dòng 183: "tại sao anh cũng chẳng hề hấn gì?"
10. **CALQUE (vừa, hệ thống).** nữ nhân 146 dòng, nam nhân 231 dòng.
    - ch155: "anh mới là nam nhân nỗ lực nhất" → nên là "người đàn ông chăm chỉ nhất nhà mình".
11. **MEME_NGHIA_DEN (vừa).** Meme và chiêu anime bị glossary chốt thành Hán Việt.
    - ch342: 大玉螺旋丸 → "Đại Ngọc La Toàn Hoàn" (là Rasengan trong Naruto).
    - ch371: 时停 → "Thời Đình" (nên là "Dừng thời gian").
    - 神仙快乐水 → "Thần Tiên Khoái Lạc Thủy".
12. **VAN_KHO (vừa).** ch155: 闹出人命 → "gây ra/tạo ra mạng người", kể cả ở tiêu đề.
    - Câu này vừa không phải tiếng Việt tự nhiên vừa mất nghĩa kép "án mạng / có em bé".
13. **GLOSSARY_TRUNG (thấp).** Thuật ngữ bị tách đôi:
    - Hỷ Kiến 339 lần, Hỉ Kiến 73 lần (glossary tự ghi cả hai kiểu).
    - "chùa Phổ Độ" và "Phổ Độ Tự".
14. **GLOSSARY_PHIEN_AM (thấp).**
    - 词条 → "từ điều" 27 lần ("ngón tay vàng từ điều" ở ch1).
    - 绿蚁 → "Lục Ngĩ", sai chính tả.
    - 金手指 trôi giữa "ngón tay vàng" và "bàn tay vàng".
15. **DINH_DANG (thấp).** 2478 dòng thoại ở 88 file dùng ngoặc thẳng `"`, các chương khác dùng “”.
    - Ví dụ ch98 dòng 3.

Lỗi nhỏ khác: ch438 dòng 35 có cụm lặp "Lúc này lúc này" (此时此刻). Ngoài ra còn calque "Cùng với việc…" (随着, 74 dòng/68 file) và "Tìm chết" (找死).

## Gợi ý

- **Đổi setting.** Chuyển truyện sang modern, hoặc có setting "đô thị tu tiên" hay setting theo arc. Luật ancient (cấm vợ/chồng, bắt dùng Ân) là gốc của hơn nửa số lỗi.
- **Cảnh báo mâu thuẫn trong app.** Báo khi toneRules/avoid của truyện mâu thuẫn với luật setting, hoặc cho toneRules của truyện được đè luật setting.
- **Bảng quan hệ thông gia cho prompt modern.** Bố vợ, mẹ vợ, anh vợ, em vợ/chị vợ, anh rể; cấm "tiểu di tử/đại cữu ca/đại di tỷ".
- **Kiểm addressing hai chiều.** Không cho ghi "ta–ngươi" cho cặp ruột thịt khi setting hiện đại; thêm cast.json có mốc cho các arc Thiên Giới.
- **Thêm checkRules:** thiếp/chàng, nữ nhân/nam nhân, ngoặc thẳng ở đầu dòng, cụm lặp từ.
- **Auto-glossary:**
  - không Hán Việt hoá meme/chiêu anime;
  - gom các biến thể cùng gốc về một cách đọc;
  - soát chính tả mục mới.
- **Chốt đại từ lời kể** cho nhân vật chính.

## Làm tốt

- Đủ đoạn, không sót chữ Hán.
- Lóng và nhịp hài giữ tốt: "OMG", "Vãi chưởng", "Đệt mợ", "chiếc áo bông tri kỷ", "bà cô tổ dân phố".
- Từ ngữ đời sống hiện đại được Việt hoá khéo: tào phớ, tôm hùm đất, kẹo Đại Bạch Thỏ, vé cào.
- Glossary hơn 700 mục được áp nhất quán qua hơn 600 chương. Chương 155 và 321 đọc trơn, đúng nghĩa.
