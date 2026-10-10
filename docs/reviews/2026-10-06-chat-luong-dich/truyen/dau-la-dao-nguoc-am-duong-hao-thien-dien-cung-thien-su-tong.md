# Review: Đấu La: Đảo ngược âm dương, Hạo Thiên điện cùng thiên sứ tông

- Setting: `ancient`. Thể loại thật: đồng nhân Đấu La giễu nhại, xuyên không; về sau là thần giới (Biểu Giới/Lý Giới). Kể ngôi 3.
- Đã dịch 324 chương, đủ đoạn. Chương đọc trọn: 0001, 0163, 0323, 0324 (hai chương cuối là bản dịch mới nhất). Có grep toàn bộ out/.

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 7.0 | Cha–con ấm, đúng vai vế. Trôi ở 父亲 (lúc "Cha", lúc "Ba"), 爷爷 ("gia gia"/"ông nội"), 阿姨 ("a di"/"dì"); glossary ghi cha–con nhưng con–ba |
| Giọng văn | 7.0 | Hán Việt hợp chất Đấu La, giọng hiện đại của nhân vật chính giữ được. Trừ điểm vì thành ngữ dịch sát chữ, "đem…", "bản thân…" |
| Dễ hiểu | 7.5 | Không sót ý, không sai số; còn ít câu convert và một lỗi sai nghĩa |
| Khó (cảm xúc, lóng) | 7.0 | Lóng xử lý tốt, nhưng vài trò đùa bị Hán Việt hóa nên mất duyên |
| Thuật ngữ | 5.5 | Glossary lớn mà tên vẫn bị tách đôi: Vũ/Võ Hồn Điện, Đới/Đái; trật tự Hán/Việt của địa danh lộn xộn |

## Lỗi (nặng → nhẹ)

1. **THUAT_NGU_TROI (cao, hệ thống)**: 武魂殿 ra "Vũ Hồn Điện" 138 lần/54 file, ra "Võ Hồn Điện" 90 lần/30 file. Ví dụ 0189:3: "Tầng lớp cấp cao của Võ Hồn Điện…". Nguyên nhân là glossary tự mâu thuẫn: 武魂殿 = Vũ, nhưng 武魂殿学院 = "Học Viện Võ Hồn Điện" và 武魂 = "võ hồn".
2. **GLOSSARY_PHIEN_AM (cao, hệ thống)**: họ 戴 bị tách đôi. "Đái Trúc Thanh" 86 lần/26 file so với "Đới Trúc Thanh" 102/34; "Đái Mộc Bạch" 43/12. Glossary ghi Đới cho tên nhưng lại có "Đái Duy Tư", "Tinh La Đái gia", "Đái lão đại". Ví dụ 0250:25: "Bọn Đái Trúc Thanh lập tức tản ra" → nên là "Đới".
3. **XUNG_HO_TROI (vừa, hệ thống)**: 0052:59 `父亲，您真强` → "Ba, ba thật mạnh." Trong khi 0167:65 `父亲！` → "Cha!". Đề xuất chốt 爸爸 = ba, 父亲 = cha.
4. **BAN_THAN (vừa, 393 lần/214 file)**: 0001:61 "Trước hết, bản thân quả thực là Thiên Tầm Tật…" → "Trước hết, hắn đúng là Thiên Tầm Tật…".
5. **CALQUE "đem" (vừa, 122 lần/92 file)**: 0324:33 "cuối cùng đem tầm mắt lạnh lùng khóa chặt trên người Thiên Tầm Tật…" → "cuối cùng ánh mắt lạnh lẽo dừng hẳn trên người Thiên Tầm Tật". Lỗi này vẫn có ở chương mới nhất.
6. **SAI_NGHIA (vừa, lẻ)**: 0323:51 `脸色铁青` → "sắc mặt sắt thép", nên là "sắc mặt tái xanh". Lỗi nằm trong chương dịch bằng prompt hiện tại.
7. **CALQUE thành ngữ (vừa, hệ thống)**: 0323:77 "âm trầm gần như có thể rỉ ra nước" (4 file); 0163:73 "đừng tự dát vàng lên mặt mình"; 0163:31 "mặt không đỏ tim không đập" (4 file). Nên lần lượt là "sầm mặt", "đừng tự đề cao mình", "mặt không đổi sắc".
8. **MEME_NGHIA_DEN (vừa, 2 file)**: 白眼狼 → "con sói mắt trắng" (0323:33, 0312:43); nên là "đồ vong ân bội nghĩa".
9. **GLOSSARY_HAN_VIET_NANG (vừa)**: hồn kỹ 鸭肉肠 → "Áp Nhục Tràng" (0038:9) làm mất trò đùa, nên là "Xúc Xích Thịt Vịt". Ngoài ra còn "Kim Thuộc Chưởng Khống" và mục lai trật tự "Chấn Không Liệt Giáp Ong".
10. **CALQUE lượng từ và từ vựng (thấp, hệ thống)**: "một đầu" 25/21, "phiến đại lục" khoảng 10, "chiếm tiện nghi" 5, "công cụ nhân" 2. Ví dụ 0003:77 "trên phiến đại lục này… Kẻ nào dám chiếm tiện nghi…".
11. **THUAT_NGU_TROI địa danh (thấp)**: 0118:13 "rời khỏi Thiên Đấu Đế Quốc… Chu gia của Đế Quốc Tinh La": một câu dùng cả hai trật tự.
12. **GLOSSARY_DANH_XUNG (thấp)**: 0125:57 "Nhị Long a di" → "dì Nhị Long"; 爷爷 ra "gia gia" 70 lần so với "ông nội" 12 lần.
13. **XUNG_HO_VAI_VE (thấp, lẻ)**: 0324:45 Tu La gọi thẳng Sinh Mệnh là "nàng" ("đến cả nàng cũng mặc cho hắn…"), trong khi xưng "ngươi" với các Thần Vương khác; nên dùng "ngươi".
14. **HU_HA (thấp, 3 file)**: 0118:53 `呵呵` → "Hừ hừ", nên là "Ha ha"; 0243 và 0310 `呵` → "Hừ".
15. **AN_THAN_TU (thấp, 6 file)**: 0094:63 "Ân, con hiểu được là tốt." Đây là nhân vật chính giọng hiện đại, nên là "Ừ, con hiểu là tốt rồi."

## Gợi ý

- App kiểm glossary: cảnh báo khi cùng một chữ Hán trong tên riêng có hai cách phiên (戴 Đới/Đái, 武 Vũ/Võ); mục auto mới phải theo mục gốc (武魂殿学院 theo 武魂殿); ép một trật tự cho X Thành và X Đế Quốc.
- Sửa addressing của cặp chính: 爸爸 = ba, 父亲 = cha; đồng bộ hai chiều.
- Prompt cổ đại: thêm danh sách calque cấm ("đem + động từ", "một đầu", "phiến", "chiếm tiện nghi", "bản thân" làm chủ ngữ, "a di") và bảng thành ngữ tương đương (滴出水, 脸不红心不跳, 白眼狼, 贴金, 铁青).
- Truyện giễu nhại: cho phép dịch nghĩa thuần Việt tên hồn kỹ và võ hồn mang tính đùa.
- Bỏ ép 嗯 → "Ân" với lời của nhân vật giọng hiện đại; 呵 → cười nhạt, không phải "Hừ".

## Làm tốt

- Đủ 100% đoạn, không sót chữ Hán.
- Độc thoại hiện đại sống: "Đệt?", "cái cóc khô gì", "Thẩm mỹ bị chó ăn mất rồi à?".
- Lóng Việt hóa khéo: "hóng chuyện đã đời", "gõ cho một vố đau điếng", "trộm gà không thành lại mất nắm gạo".
- Tránh được "tiện nghi" ở 便宜传人 ("kẻ truyền thừa nhặt được").
- Giữ đúng giới tính các Thần Vương ở hai giới theo raw.
