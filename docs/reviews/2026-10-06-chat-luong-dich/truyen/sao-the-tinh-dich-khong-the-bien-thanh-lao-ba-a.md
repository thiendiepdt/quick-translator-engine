# Review: Sao Thế, Tình Địch Không Thể Biến Thành Lão Bà À?

- **Setting trong story.json:** modern. **Thể loại thật:** đô thị hiện đại, xuyên thư có hệ thống, bách hợp hài, nhiều meme mạng. **Ngôi kể:** ngôi 3 (luật "ta" của người kể không áp).
- **Số chương đã dịch:** 179. **Chương đã đọc trọn:** 0001, 0090, 0178, 0179. Đã grep toàn bộ out/ để đếm các lỗi hệ thống.

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 6.0 | Giang Nịnh xưng với hệ thống trôi qua 5 kiểu. Chữ "ta" lọt vào thoại tự nói. Tiêu đề dùng "ta" trong khi thân chương dùng "tôi". Bạch đã thành người yêu mà vẫn xưng "tôi". |
| Giọng văn | 7.0 | Đúng chất đô thị trẻ, thoại sống. Còn sót calque và Hán Việt kiểu convert. |
| Dễ hiểu | 7.5 | Đọc trơn, gần như không sót ý. 女人 bị dịch máy móc thành "người phụ nữ", vài câu có đại từ mơ hồ. |
| Khó khăn | 7.0 | Nhiều meme xử lý khéo. Vẫn có meme dịch nghĩa đen, 呵 và 嘿嘿 dịch thiếu nhất quán. |
| Thuật ngữ | 6.5 | Tên riêng nhất quán, không sót chữ Hán. Glossary tự sinh còn mục Hán Việt nặng hoặc sai danh xưng. |

## Lỗi (từ nặng đến nhẹ)

1. **XUNG_HO_TROI – cao, hệ thống.** Giang Nịnh xưng với hệ thống mỗi chương một kiểu:
   - 0001: "tôi… cậu"
   - 0063/0068: "ta… mi"
   - 0085/0090: "Mày coi tao…"
   - 0114/0116/0163: "tôi… mày"
   - 0176/0179: "ta"

   Ví dụ ở 0179:129: `我特么是那种肤浅的人？` → "Ta mẹ nó là kiểu người nông cạn thế à?" → đề xuất "Tao mẹ nó là loại người nông cạn thế à?". Nguyên nhân: cast.json không có cặp 江柠↔系统.

2. **XUNG_HO_NGUOI_KE – cao, hệ thống.** Khoảng 12 câu trong 4 file, dồn vào các chương mới nhất. Ở 0176, `还好，我技高一筹。` thành "May thay, bản lĩnh của ta vẫn cao hơn một bậc.", nhưng ngay dòng sau lại là "tôi", rồi đến "Mình". Đề xuất: "May mà tôi vẫn cao tay hơn một bậc." Luật "ta" của truyện ngôi 1 đang lọt sang thoại của truyện ngôi 3.

3. **TIEU_DE – cao, hệ thống.** 13 tiêu đề dùng "ta". Ví dụ: 0141 `我的演技，要好过你！` → "Diễn xuất của ta giỏi hơn cậu!"; 0001 "Ơ, bảo ta nẫng tay trên nữ chính à?"; 0136 "Giang Nịnh, bắt nạt ta đi". Thân các chương này đều dùng "tôi". Bước dịch tiêu đề (titles.json) không có ngữ cảnh xưng hô.

4. **CALQUE 女人 → "người phụ nữ" – vừa, hệ thống.** Cụm "người phụ nữ" xuất hiện 105 lần trong 62 file; 87/182 dòng có 女人 bị dịch theo kiểu này. Ví dụ 0090:157: "Phen này người phụ nữ có bướng bỉnh đến đâu cũng phải rơi nước mắt." → đề xuất "Phen này cô gái có cứng đầu đến mấy cũng phải bật khóc." Với nữ sinh đại học, "người phụ nữ" nghe già và lộ dấu dịch máy.

5. **CAST_MAU_THUAN – vừa, hệ thống.** toneRules ghi Bạch sẽ chuyển sang "tớ–cậu" hoặc "em–chị", nhưng mốc cast ở ch130 chỉ đổi sang "tôi–cậu". Ví dụ 0178:15, Bạch là "cô vợ nhỏ" mà vẫn nói "Tôi… tôi biết có đầu bếp rồi! … tay nghề của tôi". Đề xuất: "Tớ… tớ biết có đầu bếp mà! … tay nghề của tớ thôi, không được à?"

6. **CALQUE "chiếm tiện nghi" – vừa, hệ thống.** 17 lần trong 11 file. Ví dụ 0030:175: `你……又占我便宜！` → "Cô...... Lại chiếm tiện nghi của tôi!" → đề xuất "Cô… lại sàm sỡ tôi nữa rồi!"

7. **CALQUE "Vạn vạn không ngờ" – vừa, hệ thống.** 10 lần trong 9 file, và có thêm "cầu còn không được" (0178:17). Ví dụ 0090:121 → đề xuất "Nào ngờ cô vặn mạnh một cái thì hỏng luôn."

8. **GLOSSARY_HAN_VIET_NANG – vừa.**
   - "kịch tình sát" ở 0001:71: "chỉ có thể kịch tình sát đối với Giang Nịnh thôi" → đề xuất "để cốt truyện 'xử đẹp' Giang Nịnh".
   - "Giang tiểu thư": 103 lần trong 55 file.
   - "Đào lão tiên sinh" dùng song song với "ông Đào".

9. **GLOSSARY_DANH_XUNG – vừa.** 白老幺 → "Bạch Lão Yêu", 101 lần trong 13 file. Ví dụ 0123:49. 老幺 nghĩa là con út, nên đề xuất "chú Út nhà họ Bạch"; "Lão Yêu" đọc lên giống "lão yêu quái".

10. **HU_HA – thấp, hệ thống.** 呵 bị dịch thành "hừ" ở 11/84 dòng. Ví dụ 0001:177: "Giang Nịnh “hừ” một tiếng" → đề xuất "cười khẩy". Riêng 呵呵 ra nhiều kiểu: "Hực!", "Hì hì" (ngay cả trong câu châm chọc), "Hô hô hô".

11. **MEME_NGHIA_DEN – thấp.**
   - 0090:163: 水漫金山 → "tràn ngập ngụa như nước ngập Kim Sơn Tự", đề xuất "sắp thành biển nước đến nơi".
   - 0179:163: 搓板 → "cái ván giặt đồ", đề xuất "ngực phẳng như sân bay".

12. **DINH_DANG – thấp, hệ thống.**
   - Ngoặc kép thẳng `"` xuất hiện trong 25 file, theo từng đợt dịch: 0091–0100 và 0161–0170.
   - Viết hoa ngay sau "......" dù câu chưa kết thúc: 198 lần trong 83 file. Ví dụ 0090:13 "sát người...... Hoàn toàn".
   - Trộn "......" với "...".

13. **DAI_TU_MO_HO – thấp, lẻ.** 0178:157: "Từng ức hiếp bao nhiêu đứa con gái như cô làm sao…" dễ đọc thành "những cô gái giống cô". Đề xuất: "Cô ta từng ức hiếp bao nhiêu cô gái, sao có thể không biết…"

14. **SAI_NGHIA – thấp, lẻ.** 0179:87–89: dòng trước đã dịch "chị Kiến Văn", dòng sau lại viết "Nghe thấy hai chữ “chị gái”", là chữ chưa ai nói. Đề xuất: "Nghe thấy chữ “chị”".

## Gợi ý sửa prompt / quy trình / app

- Ghim cặp xưng hô cho nhân vật phi người, ví dụ 江柠→系统 "tao–mày" và 系统→江柠 "ngài/ký chủ". Auto-addressing hiện chỉ sinh cặp giữa các tên có trong names.
- Chỉ bật luật "người kể xưng ta" khi truyện kể ngôi 1. Với truyện ngôi 3, prompt cần nói rõ: thoại, kể cả khi tự nói một mình hay nói với hệ thống, dùng "tôi/mình".
- Khi dịch hoặc thống nhất tiêu đề, đưa cả cast, addressing và xưng hô của thân chương vào. Có thể dịch tiêu đề sau thân chương, hoặc thêm check so đại từ trong tiêu đề với thân chương.
- Thêm bảng calque cấm cho truyện hiện đại: 女人, 占便宜, 万万没想到, 求之不得, 呵, 嘿嘿.
- Lọc auto-glossary:
  - không coi danh xưng thứ bậc (老幺, 老大, 小姐) là tên riêng;
  - gộp 陶老先生 và 陶爷爷 về một cách gọi;
  - bỏ mục không dùng "học tỷ Thi Thi".
- Khi mốc cast tự sinh lệch với toneRules ("tôi–cậu" so với "tớ–cậu"), báo để người dùng duyệt.
- Thêm bước hậu xử lý chuẩn hoá dấu câu: ngoặc kép cong, dấu "…", không viết hoa sau dấu lửng khi câu chưa kết thúc.

## Làm tốt

- Nhiều meme khó dịch khéo:
  - 宁可我牛天下人 → "Thà để ta cướp người của thiên hạ…"
  - 中场开香槟 → "ăn mừng chiến thắng quá sớm"
  - 卧槽 → "Vãi"
  - 大怨种 → "đứa xui xẻo chịu trận"
- Không còn "Ân" (0 lần) và không sót chữ Hán trong cả 179 chương.
- Đại từ cho Hạ Kiến Văn đổi từ "hắn" sang "cô" đúng thời điểm nhân vật đổi giới tính.
- Thoại gia đình tự nhiên ("mẹ–con", "Nịnh Nịnh"). Cảnh hài giữ được nhịp nhanh và dấu "~".
- Tên riêng nhất quán. Giang Nịnh với Lục Tục xưng "tôi–anh", lạnh nhạt đúng toneRules.
