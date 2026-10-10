# Review: Bài Poker Phong Yêu: Ta Có Sáu Mươi Bốn Tấm Trảm Ma Lệnh

- Setting trong story.json: `ancient` (tone witty). Thể loại thật: huyền huyễn dị giới giả cổ đại, sảng văn hài, nam chính xuyên không mang lóng hiện đại. Kể ngôi 3.
- 203 chương đã dịch. Chương đã đọc trọn: 0001, 0102, 0202, 0203 (2 chương cuối là bản dịch mới nhất). Các lỗi nghi hệ thống đã được grep trên toàn bộ out/.

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 8.5 | ta–ngươi nhất quán, hắn/gã/lão/nàng chia đúng vai; có một chỗ "của mình" mơ hồ; cast.json lệch glossary ở 2 cặp |
| Giọng văn | 8.0 | Văn trôi, chiến đấu dứt khoát, giữ nét trêu chọc; vài cụm Hán Việt nặng |
| Dễ hiểu | 8.0 | Hầu như không có câu tối nghĩa hay sót ý; còn "đem…" dày và vài chỗ lệch sắc thái |
| Chỗ khó | 7.0 | Thán từ và lóng yếu: 呵 thành "Hừ", 嗯 thành "Ân", 鸡肋 thành "gân gà", 锤子 thành "cái búa" |
| Thuật ngữ | 7.0 | Tên riêng chuẩn, không sót chữ Hán; nhưng 猎魔公会, quân bài và Đế Đô trôi cách viết; tiêu đề viết hoa; glossary có mục trùng |

## Lỗi (nặng → nhẹ)

1. **HU_HA, cao, hệ thống (29/32 dòng có 呵).** 0056:189 `“呵，我赢了！”` → "Hừ, ta thắng rồi!" → nên là "Ha, ta thắng rồi!". 呵 là cười khẩy, còn "Hừ" là bực bội, nên cảm xúc bị đổi ngược.
2. **MEME_NGHIA_DEN, vừa, hệ thống ("gân gà" 20 lần/16 file).** 0202:3 `读心术还十分鸡肋` → "thuật đọc tâm còn vô cùng gân gà" → nên là "còn khá vô dụng". Đây là dịch theo chữ, và máy hay tự thêm "vô dụng" phía sau để giải thích.
3. **MEME_NGHIA_DEN, vừa (3 lần/3 file).** 0134:69 `他们围攻个锤子。` → "Bọn chúng vây công cái búa ấy." → nên là "Vây công cái con khỉ ấy!".
4. **AN_THAN_TU, vừa, hệ thống (23 lần/21 file).** 0202:109 `“嗯。”` → "Ân." → nên là "Ừ.". Thoại của nam chính tưng tửng mà bị khô và cổ lỗ.
5. **THUAT_NGU_TROI, vừa, hệ thống.** 猎魔公会: "công hội liệp ma" khoảng 170 lần/38 file, đến khoảng chương 142 thì chuyển sang "Liệp Ma Công Hội" (50 lần/14 file) vì glossary thêm mục muộn và viết hoa như tên riêng. Ví dụ 0203:187. Ngoài ra "đế đô" viết thường 25 lần/12 file, lẫn với "Đế Đô".
6. **CALQUE, vừa, hệ thống ("đem" 52 lần/45 file).** 0202:87 → "đem bầu rượu chuyển sang chỗ Tiêu Vô Du" → nên là "đẩy bầu rượu sang chỗ Tiêu Vô Du".
7. **DAI_TU_MO_HO, vừa, lẻ.** 0202:115 `自己的女儿…` → "con gái của mình mập mờ với hắn" → nên là "con gái ông ta". Câu theo điểm nhìn Diệp Dương nên dễ đọc nhầm.
8. **SAI_NGHIA, thấp.** 0203:99 `可能要栽` → "sắp gặp xui xẻo" → nên là "sắp ngã ngựa".
9. **TIEU_DE, thấp, hệ thống (9 tiêu đề, 0083–0091).** "Chương 86: Lại Gặp Liệp Ma Sư Đế Tướng" bị viết hoa mọi chữ.
10. **THUAT_NGU_TROI, thấp (8 lần/4 file).** Quân bài phần lớn ghi số ("Hồng Đào 9"), nhưng có chỗ ghi chữ: "Hồng Đào Năm" (0116), "Mai Hoa Tám", "Hồng Đào Bảy".
11. **CAST_MAU_THUAN, thấp.** 魏叔→叶炀 trong glossary là "Diệp tiên sinh", trong cast là "Diệp công tử". 纪岩→叶炀 trong glossary là "ta–ngươi", trong cast là "Diệp đại ca" với mốc 0063, trong khi bản dịch đã dùng từ 0024. Bản dịch vẫn đúng nhờ bám raw.
12. **VAN_KHO, thấp.** 0203:183 `承让` → "Thừa nhượng!" → nên là "Đã nhường!". Cùng kiểu: "phong đạm vân khinh" (2 lần), "hoàng hoa đại khuê nữ".
13. **VAN_KHO, thấp.** 0202:127 `人模狗样` → "có vẻ ngoài bảnh bao đạo mạo": câu chê đùa bị biến thành lời khen.
14. **LAP_TU, thấp.** 0102:173 "tay run rẩy cầm chặt binh khí mà run lẩy bẩy".

## Gợi ý

- Bỏ luật ép 嗯 → "Ân" khi tone là witty hoặc nhân vật chính xuyên không; cho phép "Ừ/Ừm".
- Thêm bảng thán từ vào prompt: 呵 = Ha/Hờ/Hừm, 哼 = Hừ. Thêm checkRule: dòng raw có 呵 mà out có "Hừ" thì cảnh báo.
- Lập danh sách lóng cố định: 鸡肋, 个锤子, 装逼, 人模狗样.
- Auto-glossary: không thêm danh từ chung viết hoa khi bản dịch đã quen một cách viết khác; chuẩn hoá các mục *公会; gộp mục trùng 帝相猎魔公会/帝相猎魔师公会 và 凝纹境/凝文境; chốt một kiểu "thành X" hoặc "X Thành".
- Chốt quy ước ghi quân bài bằng số/ký hiệu, kể cả trong tiêu đề.
- Đồng bộ addressing giữa story.json và cast.json; app nên báo khi hai file trái nhau.
- Sửa tiêu đề 0083–0091; thêm bước hậu kiểm bắt tiêu đề viết hoa mọi chữ.
- Với 自己 trong đoạn kể có hai nhân vật nam, prompt nên yêu cầu chỉ rõ người.

## Làm tốt

- Văn trôi, ít văn convert; chiến đấu nhịp nhanh, tượng thanh tự nhiên.
- Xưng hô ổn định suốt 203 chương; đại từ chia đúng vai: hắn, gã, lão.
- Giữ nguyên các câu trêu gái theo signature_phrases ("cô nàng ngực khủng", "nhánh nhỏ trĩu quả to"), không bị làm nhẹ.
- Không sót chữ Hán; tên riêng khớp glossary; tự gộp lỗi gõ của tác giả (萧无宇 → Tiêu Vô Du).
- Lời kể cổ trang sạch, còn suy nghĩ của nam chính vẫn giữ chút hiện đại hợp vai.
