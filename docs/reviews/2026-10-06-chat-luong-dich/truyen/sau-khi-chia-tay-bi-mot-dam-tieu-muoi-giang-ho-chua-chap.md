# Review: Sau Khi Chia Tay, Bị Một Đám Tiểu Muội Giang Hồ Chứa Chấp

- **Setting:** modern. Thể loại thật: đô thị hệ thống thần hào hài, từ khoảng ch.100 chuyển sang mảng đua mô tô chuyên nghiệp. Ngôi kể thứ ba, nên luật "ta" không áp dụng.
- **Đã dịch:** 587 chương. **Đã đọc trọn:** 0001, 0294, 0586, 0587, có grep toàn bộ out/.

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 5.5 | Các cô gái xưng em–anh với Lâm Dã nhưng hắn đáp tôi–cô, kể cả với người yêu Bạch Hiểu Tịnh. Lỗi nằm sẵn trong addressing của story.json. Nhân vật phụ trôi cậu/anh ngay trong một chương. |
| Giọng văn | 7.5 | Đô thị hiện đại, thoại tiểu muội bỗ bã sống động. Riêng đoạn kỹ thuật/âm mưu dày Hán Việt và từ convert. |
| Dễ hiểu | 7.5 | Trôi chảy, khớp từng đoạn, không sai số. Chủ yếu có calque nhẹ. |
| Khó khăn | 6.5 | Lóng thường dịch tốt, nhưng meme mạng bị dịch nghĩa đen nên mất chất hài. |
| Thuật ngữ | 6.0 | Biệt danh và thuật ngữ cốt lõi trôi theo đợt chương; glossary tách đôi một nhân vật và có nhiều biệt danh phiên Hán Việt nặng. |

## Lỗi (nặng → nhẹ)

1. **XUNG_HO_VAI_VE – cao, hệ thống** (0294:75)
   - Gốc: 不是给我拿名次，是给你自己 → hiện tại: "Không phải giành thứ hạng cho tôi, mà là cho chính cô." → nên sửa: "…cho anh, mà là cho chính em."
   - Bạch Hiểu Tịnh gọi 哥, xưng em và là người yêu của hắn, nên hắn đáp tôi–cô nghe lạnh, sai quan hệ.
   - story.json có 8 cặp bất đối xứng (X→林野 em–anh nhưng 林野→X tôi–cô/tôi–cậu). Lỗi gặp ở ch.0321, 0441, 0481, 0585, 0587.
2. **CAST_MAU_THUAN – cao, hệ thống** (0587:81)
   - Hiện tại: "Cô xuất dữ liệu ra…" → nên sửa: "Em xuất dữ liệu ra…"
   - Cùng chương, hắn xưng anh với Vương Tiểu Oánh (do cast.json có mốc từ ch.581) nhưng vẫn gọi Bạch Hiểu Tịnh là "cô".
   - Trong 381 mục addressing của story.json, chỉ 2 cặp có mốc cập nhật.
3. **GLOSSARY_TRUNG – vừa:** 黄毛 thật ra là biệt danh của 白晓静 (ch.0113:31 tả cùng bộ tóc hai bím màu nâu trà mật). Glossary lại có hai bộ addressing riêng: `黄毛→林野 em–anh` / `林野→黄毛 tôi–cô`.
4. **XUNG_HO_TROI – vừa, lẻ** (0586:23)
   - Gốc: 你看了今天的练习赛数据没有 → hiện tại: "Cậu đã xem số liệu…" → nên sửa: "Anh xem số liệu…"
   - Trần Hải Đào trẻ hơn Lục Kiến Quốc nhưng trong chương gọi Lục là "cậu" 2 lần, "anh" 4 lần (dòng 45 là "anh rõ rồi đấy").
5. **MEME_NGHIA_DEN – cao** (0029:83)
   - Gốc: 这孙子…终极舔狗 → hiện tại: "thằng cháu này đã làm kẻ liếm chó tối thượng" → nên sửa: "thằng cha này làm 'chó liếm' số một…"
   - "Kẻ liếm chó" có nghĩa ngược hẳn. Lỗi xuất hiện 24 lần trong 11 file.
6. **MEME_NGHIA_DEN – vừa** (0317:73 và tiêu đề): 川渝暴恐 → "Khủng Bố Xuyên Du" → nên sửa: "hung thần Xuyên Du". Đây là meme chỉ con gái vùng Xuyên Du dữ dằn, không phải khủng bố.
7. **GLOSSARY_HAN_VIET_NANG – vừa, hệ thống** (0317:89)
   - 小辣椒 → "Tiểu Lạt Tiêu" (92 dòng/43 file), nên dịch "Ớt Nhỏ".
   - 白毛 → "Bạch Mao" (116 lần), trong khi 黄毛 đã dịch là Tóc Vàng.
   - 东瀛十八手 → "Đông Doanh Thập Bát Thủ", 仙人跳 → "bẫy tiên nhân khiêu".
8. **THUAT_NGU_TROI – vừa** (0294:37): 花臂 lúc là "Xăm Tay" (189/46 file), lúc là "Hoa Tí" (54/12 file). Hiện tại: "Hoa Tí! Mày nghe thấy chưa?"
9. **THUAT_NGU_TROI – vừa** (0101:59): 黄毛姐牛逼 → "Chị Hoàng Mao đỉnh vãi!" Đợt ch.0093–0102 phiên âm (Hoàng Mao 65, Lục Mao/Phấn Mao 32) dù glossary đã chốt "tóc vàng".
10. **THUAT_NGU_TROI – thấp**
    - 火星车队 → "Đội xe Sao Hỏa" 22 lần/10 file (0120:139), glossary chốt "đội đua Hỏa Tinh".
    - 维修区 có 3 cách dịch: khu bảo dưỡng 173 / khu kỹ thuật 81 / khu sửa chữa 19.
11. **CALQUE – vừa** (0587:37)
    - Gốc: 你能不能操作 → hiện tại: "ông có thao tác được không?" → nên sửa: "ông có xoay xở được không?"
    - "thao tác" 19 lần/13 file. Cùng chương: 背景不简单 → "bối cảnh không đơn giản", nên là "lai lịch".
12. **VAN_KHO – thấp** (0587:23): "Đại diện kỹ thuật họ Triệu" lặp 11 lần trong chương, cộng thêm hợp quy/dị nghị/thẩm tra nên đọc như công văn. Nên rút gọn thành "ông Triệu bên kỹ thuật".
13. **DINH_DANG – thấp** (0001:75/81): cùng một biệt danh viết "Tóc Vàng" rồi "Tóc vàng"; "Lão Triệu" viết hoa giữa câu.
14. **BAN_THAN – thấp, lẻ** (0113:65): "kết quả bản thân ngồi không vững" → "ai ngờ chính chị ngồi không vững". "bản thân" không dày (167 lần, so với 1645 lần 自己 trong raw).

## Gợi ý

- App ép đối xứng addressing: X→NV chính em–anh thì NV chính→X phải là anh–em, hoặc ít nhất cảnh báo. Sửa ngay các cặp 林野→白晓静/郭二佳/赵小月/张晶晶/徐丽竹… thành anh–em.
- Auto-glossary nhận biệt danh ngoại hình (黄毛, 花臂, 白毛…) là alias của tên thật và gộp vào mục tên thật.
- Prompt modern: biệt danh mô tả dịch nghĩa thuần Việt, không phiên Hán Việt. Thêm bảng meme hay gặp: 舔狗, 孙子 (chửi), 暴恐, 躺平, 操作, 背景.
- Khi truyện chuyển arc, chốt sớm thuật ngữ cốt lõi (维修区, 火星车队) vào glossary.
- Sinh addressing cho cặp nhân vật phụ ngay từ chương đầu họ xuất hiện.

## Làm tốt

- Thoại tiểu muội bỗ bã, đúng chất ("vãi", "đỉnh vãi", tao–mày giữa các cô gái).
- 0 "Ân", 0 "ngươi", không sót chữ Hán, "Hừ" chỉ 3 lần.
- 587/587 chương khớp số đoạn; số liệu và tiền tệ chính xác.
- Thuật ngữ đua xe dễ hiểu; tiêu đề chương viết hoa đúng.
