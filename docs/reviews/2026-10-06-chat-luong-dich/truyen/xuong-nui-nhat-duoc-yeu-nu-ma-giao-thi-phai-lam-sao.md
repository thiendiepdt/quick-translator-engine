# Review: Xuống Núi Nhặt Được Yêu Nữ Ma Giáo Thì Phải Làm Sao?

- **Setting:** ancient. Thể loại thật: tiên hiệp cổ trang, ngọt, hài tưng tửng; truyện kể ngôi 3.
- **Số chương:** 130 chương đã dịch.
- **Chương đã đọc trọn:** 0001, 0065, 0129, 0130 (đối chiếu raw từng dòng), sau đó grep toàn bộ out/.

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 7.5 | Cặp chính ổn: Lục Vân gọi "cô nương", từ ch.79 chuyển "nàng". Đại từ kể cho trưởng bối trôi lão/hắn/ông; lẻ tẻ có chỗ Lục Vân gọi Bạch Chỉ là "ngươi". |
| Giọng văn | 7.5 | Giọng cổ trang hài nhẹ, thành ngữ Việt tốt. Nhưng "khẽ" chèn dày và "Ân." lặp làm văn đều, khô. |
| Dễ hiểu | 8 | Đúng nghĩa, đủ đoạn, không sót chữ Hán. Còn calque nhẹ ("Mà…" đầu câu, "bản thân" trơ trọi). |
| Khó khăn | 6.5 | Câu đáp 嗯 đặc trưng của nữ chính bị đóng khuôn "Ân."/"A?"; meme 牛头人 dịch đen. |
| Thuật ngữ | 7 | Bám glossary tốt, nhưng auto-glossary gây trôi (đèn cầu trời → đèn Khẩn Thiên), lấy danh xưng làm tên, có mục trùng. |

## Đo "Ân" (theo yêu cầu)

- Raw có **188** chữ 嗯. Bản dịch có **158** chữ "Ân" (153/187 dòng có 嗯, tức 82%), nằm trong 83/130 file.
- 14 câu "嗯？" thành **"A?"**; tính cả các chỗ khác thì "A?" xuất hiện 22 lần.
- Riêng lô 0021–0030 dùng Ừ/Vâng/Ưm/Hử (12 "Ừ"). Lô này cũng là lô dùng ngoặc thẳng, có lẽ dịch bằng prompt cũ. Ngoài lô đó, gần như 100% 嗯 thành "Ân".
- **Mật độ: 7,4 "Ân"/10.000 chữ**, gấp 2,4 lần truyện đứng thứ hai trong thư viện (tien-tu-lai-xu-long: 3,1).
- Hai nguyên nhân cộng lại:
  - Raw dày 嗯: 146/186 câu là lời đáp cụt của Bạch Chỉ.
  - Prompt **tự mâu thuẫn**. Ràng buộc hệ thống số 11 bắt 嗯→"Ân", 嗯?→"A?", và tự tuyên bố không luật nào được ghi đè. Trong khi đó mục "Giọng văn hài" của cùng prompt dạy 嗯→Ừm/Ừ, và lấy đúng câu "Ân, cả bàn…" làm ví dụ "Khô (tránh)". Model theo luật 11.

## Lỗi (nặng → nhẹ)

1. **AN_THAN_TU** (cao, hệ thống). Lỗi chính đã đo ở mục trên.
   - Ví dụ 0065:169–173: `“嗯。”` → "Ân." … "Ân." ngay giữa cảnh ôm nhau. Nên là "Ừm." / "Ừ."
2. **AN_THAN_TU, "A?"** (vừa, hệ thống, 22 lần).
   - 0126:57: `“嗯？”` → "A?". Nên là "Hửm?", vì Bạch Chỉ đang thắc mắc chứ không giật mình.
3. **BAN_THAN** (cao, hệ thống). "bản thân" 219 lần/93 file; 154 lần đứng trơ trọi trong lời kể.
   - 0001:89: "Một tràng thao tác này của bản thân đúng là hơi quá đà thật..". Nên là "Pha xử lý vừa rồi của hắn đúng là hơi quá tay thật..".
4. **THUAT_NGU_TROI** (cao). 祈天灯 ở ch.118 dịch "đèn cầu trời". Sang ch.119, auto-glossary chốt "đèn Khẩn Thiên" và dùng 18 lần.
5. **NGUOI_CHUNG** (vừa, khoảng 24 dòng/17 file).
   - 0054:131: "Khi một người yêu ngươi, dẫu chỉ phải đợi thêm một giây…". Nên là "Khi một người yêu mình…".
6. **XUNG_HO_TROI, đại từ kể cho trưởng bối** (vừa, hệ thống).
   - Hứa Dịch An: lão 42 / hắn 26 / ông 15. Ví dụ 0130:135: "ông thật sự chẳng hiểu", nên là "lão".
   - cast.json không có trường đại từ kể cho nhân vật.
7. **THEM_Y "khẽ"** (vừa, hệ thống). "khẽ" 373 lần; khoảng 103 dòng raw không có 轻/微.
   - 0001:13: "khẽ thở dài một tiếng" cho `叹了一口气`.
8. **CALQUE "Mà…" đầu câu** (vừa). 144 câu/83 file.
   - 0065:7: "Mà Bạch Chỉ cũng từng bước từng bước…".
9. **DINH_DANG** (vừa). 16 file dùng ngoặc thẳng "…", trong đó có 2 chương mới nhất 0129 và 0130.
10. **MEME_NGHIA_DEN** (vừa, lẻ). 牛头人 → "ngưu đầu nhân" 16 lần ở ch.2, kể cả tiêu đề "Ngưu đầu nhân phải chết". Đây là meme NTR, nên dịch theo nghĩa meme; glossary đang chốt cách dịch đen này.
11. **GLOSSARY_DANH_XUNG** (vừa).
    - 0061:143: "Vâng, chúng cháu về rồi, Lý nãi nãi." Nên là "bà Lý".
    - Cùng kiểu: "Hoàng lão đầu" (61 lần), 王小姐, 陈捕头, 聂门主, 顾将军 bị coi là tên.
12. **GLOSSARY_TRUNG** (thấp).
    - 锻/断魂炎 tác giả gõ nhầm trong cùng đoạn, auto-glossary tách thành 2 tên: Đoán/Đoạn Hồn Viêm (0027:67/73).
    - Trùng 瑶灵仙子/瑶灵; 春庭雨 có ở 2 nhóm.
    - 楼 lúc "Lầu" lúc "Lâu".
13. **XUNG_HO_TROI** (vừa, lẻ).
    - 0065:111: Lục Vân nói "Sao ta lại không muốn đi cùng ngươi chứ...". Nên là "cô nương".
14. **TIEU_DE** (thấp). 23 tiêu đề viết hoa mọi chữ (0004–0040).
    - "Tâm An Liền Là Chốn Về" vừa viết hoa vừa calque; nên là "Lòng an là nhà".
15. **SAI_NGHIA** (thấp).
    - 0110:87: 难怪 → "Hảo nào", không phải từ tiếng Việt; nên là "Thảo nào".
    - Lẻ thêm 0129:151: "Ta phải mớm chút cảm xúc đã chứ..." dịch phẳng câu đùa.

## Gợi ý

1. Sửa mâu thuẫn trong prompt:
   - Nới luật 11 cho tone witty/ngọt: 嗯。→ Ừ/Ừm, 嗯？→ Hửm?.
   - Chỉ giữ "Ân" ở cảnh trang trọng hoặc khi raw ghi 嗯了一声.
2. Thêm checkRule:
   - Cảnh báo khi "Ân"/"A?" vượt khoảng 2/10.000 chữ.
   - Đếm "bản thân" trơ trọi, "khẽ" không có nguồn, câu mở đầu "Mà ".
3. Auto-glossary:
   - Tra out/ các chương trước khi chốt mục mới.
   - Không nhận danh xưng làm tên.
   - Gộp biến thể do tác giả gõ nhầm.
   - Thống nhất 楼 = Lâu.
4. cast.json: thêm đại từ kể cho trưởng bối (Hứa Dịch An, Mạc Tam Thanh → "lão").
5. Bộ dò quan hệ: không gắn dấu hiệu "道侣" cho mọi cặp cùng xuất hiện trong chương. Prompt 0131 đang gắn cho 张亦生↔陆云.
6. Glossary common: bỏ mục 牛头人 → "ngưu đầu nhân".
7. check: ép ngoặc “” và tiêu đề viết hoa kiểu câu.

## Làm tốt

- Đủ đoạn, khớp dòng 1:1, không sót chữ Hán ở cả 130 chương.
- Mốc chuyển xưng hô cô nương → nàng ở ch.79 chạy đúng.
- Thành ngữ Việt hóa đẹp: "ruồi không bâu trứng lành", "nước sông không phạm nước giếng", "con giun trong bụng".
- Giữ được parody "Ngạo Tiếu Giang Hồ" và "Lệnh Hồ sư huynh của phái Hoa Sơn"; tiêu đề "Kiều thê nhà ai vò võ phòng không" hay.
- Giọng từng nhân vật phân biệt rõ: "Hứa sư bá, người…", "Lục huynh, huynh…", Trương Diệc Sinh nói tưng tửng.
