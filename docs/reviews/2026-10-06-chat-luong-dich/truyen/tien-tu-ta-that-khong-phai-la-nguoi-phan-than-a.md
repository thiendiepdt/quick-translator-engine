# Review: Tiên tử, ta thật không phải là ngươi phân thân a

- Setting: `ancient`. Thực tế là truyện tu tiên hài, nam bị nữ hóa thành phân thân của tiên tử, kể ngôi 3. Đã dịch 958/958 chương.
- Chương đọc trọn: 0001, 0480 (giữa), 0635 và 0636 (dịch mới nhất, 2026-10-02). Các lỗi nghi là hệ thống đều đã grep trên toàn bộ out/.

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| xung_ho | 6.5 | Thoại trong chương đúng vai vế. Đại từ kể cho Tề Hoàng trôi hắn/nàng theo lô chương; có chỗ lời kể gọi quan huyện là "ngài"; cast.json gần như rỗng. |
| giong_van | 7.5 | Đúng chất tu tiên hài, câu trôi. Còn mùi convert ("tầng lớp cao tầng", "đem…"). |
| de_hieu | 7.5 | Rõ nghĩa, không sót ý. Có sai nghĩa lẻ tẻ. |
| kho_khan | 7 | Thành ngữ chuyển khéo, nhưng câu dẫn thoại lặp lại hay vụng; 呵呵 thành "Hừ hừ", 嗯 thành "Ân". |
| thuat_ngu | 6 | Tên riêng ổn định. Thuật ngữ cốt lõi 灵魂纽带 trôi 5 kiểu; glossary có phiên âm sai (曾, 敲) và giữ nguyên lóng 钉子户. |

## Lỗi (nặng → nhẹ)

1. **XUNG_HO_TROI (cao, hệ thống, 170 dòng/41 chương)**: ch.0067 dòng 3, raw `她掀开自己裙摆`, bản dịch đang là "…hắn vén tà váy của mình lên". Nên sửa thành "nàng vén tà váy lên". Lỗi dồn theo cụm chương 0063–0097, 0302–0311, 0760–0781, trong khi 3811 dòng ở các chương khác dùng "nàng". Nguyên nhân: toneRules "giữ đại từ hắn cho Hoàng Kỳ" mâu thuẫn với raw, và cast.json không ghi mốc chuyển đại từ.
2. **THUAT_NGU_TROI (cao, hệ thống)**: 灵魂纽带 (102 lần) được dịch thành "liên kết linh hồn" 48 lần, "dây liên kết linh hồn" 35, "mối dây linh hồn" ~10, "sợi dây linh hồn" 3, "Linh Hồn Nữu Đái" 2. Glossary chốt "Linh Hồn Nữu Đái", vừa nặng vừa khó hiểu nên model hầu như bỏ qua. Riêng chương mới nhất (0635 dòng 3) lại dùng nó: "Linh Hồn Nữu Đái xuất thể". Nên đổi thành "sợi dây linh hồn thoát khỏi cơ thể".
3. **SAI_NGHIA (cao, lẻ)**: 0480 dòng 23, `老妇人已经年迈` bị dịch thành "lão phụ thân này tuổi tác đã cao", tức "ông bố già". Nên sửa thành "lão thân/bà già này tuổi đã cao".
4. **GLOSSARY_PHIEN_AM (vừa, hệ thống)**: họ 曾 ra "Tằng" 112 lần/18 file và "Tăng" 40 lần/7 file. Glossary tự mâu thuẫn: 曾校 Tằng Hiệu nhưng 曾寀 Tăng Thái. Họ này đọc đúng là **Tăng**. Ví dụ 0480 dòng 25: "Bà lão họ Tằng kia".
5. **CALQUE "tầng lớp cao tầng" (vừa, 17 lần/15 file)**: 0480 dòng 3, "gây sự chú ý của tầng lớp cao tầng tu tiên giả". Nên sửa thành "các bậc cao tầng của giới tu tiên". Ở 0620 còn có "Sáu tầng lớp cao cấp còn lại", dùng chữ "tầng lớp" cho người cụ thể là sai.
6. **CALQUE "đem" (vừa, 528 lần/343 file)**: 0480 dòng 119, "Trương Chí cải trang đem tin tức làm lộ ra". Nên sửa thành "cải trang đi tiết lộ tin tức".
7. **VAN_KHO (vừa, 21 lần, ~12 cách dịch)**: 语不惊人死不休. Ví dụ 0636 dòng 171: "Trần Khải không nói lời kinh người thì thề không bỏ qua." Ở 0068, 0074, 0764 còn có "khiến người ta không kinh ngạc thì không chịu dừng". Nên chốt một cách: "buông một câu khiến người ta sững sờ".
8. **HU_HA (vừa, 52 dòng)**: 呵呵 bị dịch thành "Hừ hừ", ví dụ 0005 dòng 145: “Hừ hừ, bà nghe ai nói thế?”. Nên dùng "Ha ha" hoặc "Hừm" cười khẩy.
9. **MEME_NGHIA_DEN (vừa, 21 lần)**: 修仙钉子户 ra "đinh tử hộ tu tiên" (0023 dòng 27). Đây là lóng chỉ kẻ bám lì, đồng thời là biệt danh của nhân vật chính. Nên dịch "kẻ bám trụ tu tiên mãi không lên nổi".
10. **GLOSSARY_PHIEN_AM (vừa, 7 lần)**: 敲山锤 ra "Thao Sơn Chùy" (0401 dòng 125). 敲 đọc đúng là "xao", nên sửa thành "Xao Sơn Chùy".
11. **DINH_DANG (vừa, hệ thống)**: 170 file chỉ dùng ngoặc thẳng `"…"`, 52 file trộn hai kiểu ngoặc, đổi theo từng lô chương (vd 0012 dòng 77).
12. **XUNG_HO_VAI_VE (thấp, 25 dòng/19 file)**: lời kể ngôi ba gọi nhân vật là "ngài": "bởi vì ngài vừa mới mạo hiểm về nông thôn thị sát dân tình" (0028 dòng 9). Nên dùng "y/ông ta".
13. **BAN_THAN (thấp, 1397 lần/665 file)**: 0636 dòng 189, "nếu bản thân gả cho Trần Khải". Nên sửa thành "nếu mình gả cho Trần Khải".
14. **AN_THAN_TU (thấp, 63 lần/52 file)**: “Ân!” (0017 dòng 57). Giọng light novel nên dùng "Ừm/Vâng/Dạ". Bản dịch cũng đã dùng các từ này ở hơn nửa số chỗ, nên chữ "Ân" còn lại là do prompt ép.
15. **NGUOI_CHUNG (thấp, lẻ tẻ)**: 0016 dòng 11, "chỉ cần ngươi đủ xuất sắc…". Nên sửa thành "chỉ cần đủ xuất sắc thì ai cũng…".

Lỗi phụ: 地外 ra "ngoài mặt đất" (6 lần, 0479–0480), nghĩa đúng là ra ngoài hành tinh/thiên ngoại. Glossary còn trùng biến thể: 平安县/平安县城, 龙运矿坑/矿洞.

## Gợi ý sửa

- Thêm vào cast.json mục 黄奇/齐凰 kèm mốc đại từ, hoặc luật "theo đúng 他/她 của raw". Bỏ dòng toneRules "giữ hắn cho Hoàng Kỳ".
- Lọc phiên âm của glossary tự sinh theo từ điển Hán Việt cho họ và chữ đa âm (曾→Tăng, 敲→Xao); gộp các biến thể trùng.
- Chốt thuật ngữ cốt lõi bằng thuần Việt khi Hán Việt khó hiểu (灵魂纽带, 钉子户). App nên cảnh báo khi một mục glossary bị model bỏ qua quá 50%.
- Đưa vào signaturePhrases các câu dẫn thoại lặp lại (语不惊人死不休, 呵呵, 嗯), kể cả khi setting=ancient.
- Thêm vào prompt danh sách cần né: "tầng lớp cao tầng", "đem … làm …", "bản thân" khi "mình" tự nhiên hơn, và "ngài/ngươi" trong lời kể.
- Thêm checkRule bắt ngoặc thẳng `"`, chuẩn hóa sang “ ” khi accept.

## Làm tốt

- Thoại đúng vai vế và có màu: lão gia–thiếp, người hầu xưng "tiểu nhân", ăn mày xưng "con".
- Thành ngữ chuyển khéo: "ăn cây táo rào cây sung", "ăn cơm nhà vác tù và hàng tổng", "ở hiền gặp lành", "quyền huynh thế phụ".
- Chơi chữ được giải nghĩa: "Khương Phức" đồng âm với "phục" trong phục quốc.
- Giọng tự giễu của chương 1 còn nguyên ("Cha già ơi!", "Mẹ ơi là mẹ!").
- Đủ đoạn ở cả 958 file, không sót chữ Hán, tiêu đề chương viết hoa đúng.
