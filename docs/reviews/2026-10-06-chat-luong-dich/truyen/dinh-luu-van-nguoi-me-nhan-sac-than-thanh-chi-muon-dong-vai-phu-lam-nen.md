# Review: Đỉnh Lưu Vạn Người Mê Nhan Sắc Thần Thánh Chỉ Muốn Đóng Vai Phụ Làm Nền

- Setting trong story.json: `modern`. Thực tế truyện là xuyên không + hệ thống, lấy giới giải trí hiện đại, giọng hài; có rất nhiều đoạn phim trong phim (cung đình, tiên hiệp, zombie/vô hạn lưu, vườn trường).
- Ngôi kể: ngôi 3. Đã dịch 189/189 chương.
- Các chương đã đọc trọn, đối chiếu từng dòng với bản gốc: 0001, 0095, 0188, 0189. Hai chương 0188 và 0189 có `updatedAt` mới nhất trong state.json, vì mtime của mọi file trong `out/` đều giống nhau. Ngoài ra đã grep cả `out/` và state.json để đếm lỗi.

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 8.0 | Lời kể dùng "hắn". Thoại tôi–cậu, anh–em, cháu–bà ngoại đúng vai vế. Phim trong phim chuyển đúng sang trẫm/ngươi. Chỉ sai lẻ tẻ. |
| Giọng văn | 8.0 | Văn mượt, hợp giọng hài của truyện giới giải trí. Một số cụm Hán Việt hoặc calque là do glossary ép vào. |
| Dễ hiểu | 7.5 | Không sót đoạn: số dòng khớp ở cả 189 chương. Nhưng 百亿 bị dịch thành "trăm tỷ" một cách hệ thống. |
| Khó khăn | 8.0 | Lóng và meme xử lý tốt. Còn một hai meme bị mờ ý. |
| Thuật ngữ | 6.5 | 丧尸 dịch 4 kiểu, chức danh "X đạo" dùng song song với "đạo diễn X", "Lão Bùi" viết hoa giữa câu, tên phim trong truyện dịch 2 kiểu. |

## Lỗi (xếp từ nặng đến nhẹ)

1. **SAI_SO (cao, hệ thống)**: 百亿 → "trăm tỷ".
   - Đếm được: 百亿 có 12 lần trong 10 file; "trăm tỷ" xuất hiện 14 lần trong 11 file.
   - Ví dụ 0002 dòng 15: `即可获得百亿资产` → "khối tài sản trăm tỷ". Đúng phải là **mười tỷ**.
   - Cùng kiểu: `五百亿` → "năm trăm tỷ" (đúng: 50 tỷ); `几百亿` → "vài trăm tỷ" (đúng: vài chục tỷ).
   - Đây là con số phần thưởng, trục chính của truyện. Summary trong story.json cũng ghi "phần thưởng trăm tỷ" nên lỗi cứ lặp lại.
2. **KHAC – checker báo sai (cao, hệ thống)**: state.json có 48 cảnh báo sai.
   - 25 cảnh báo "xưng hô cổ trang" rơi vào thoại phim cổ trang và cả từ "con ngươi". Riêng 0003 dòng 97 (`瞳仁黑得像深水潭`) bị review tới 3 vòng.
   - 23 cảnh báo "lời kể ngôi một tôi → ta" trong khi truyện kể ngôi ba. Luật này bắn cả vào các tiêu đề "Để tôi!", "Quất tôi đi!".
   - Hậu quả: tốn vòng review và có nguy cơ ép model sửa đúng thành sai.
3. **XUNG_HO_NGUOI_KE (vừa)**: dùng "ta" trong suy nghĩ của nhân vật hiện đại.
   - 0128 dòng 1: `我哥居然是宋时安！` → "Anh họ ta thế mà…". Nên đổi thành "Anh họ **mình**".
   - 0066 dòng 21: "ta thật sự cảm ơn thầy đấy huấn luyện viên". Nên đổi "ta" thành "tôi".
4. **THUAT_NGU_TROI (vừa, hệ thống)**: 丧尸 có 47 lần trong 5 file và được dịch thành tang thi, zombie, xác sống, thây ma.
   - 0100 viết "xác sống", nhưng ngay chương sau 0101 dòng 9 lại là "vài con tang thi".
   - Đề xuất: chốt một từ, ví dụ "zombie" hoặc "xác sống", rồi đưa vào glossary.
5. **THUAT_NGU_TROI (vừa, hệ thống)**: chức danh đạo diễn không thống nhất.
   - "Trương/Vương/Chu đạo" xuất hiện 58 lần, song song với "đạo diễn Trịnh/Hà" 30 lần.
   - Ví dụ 0003 dòng 61: "Trương đạo, hay là thôi đi ạ". Đề xuất: "Đạo diễn Trương, …".
6. **GLOSSARY_HAN_VIET_NANG (vừa)**: 0188 dòng 13 có "Phó lão thái thái mí mắt cũng chẳng thèm nhấc lên".
   - Cụm này lặp 9 lần, đi cùng "Phó lão tiên sinh", "trạch đệ".
   - Đề xuất: "Bà cụ Phó chẳng buồn nhấc mí mắt". Cách dịch hiện tại trái style.avoid.
7. **THUAT_NGU_TROI (vừa)**: tên phim 头号赢家.
   - 0095 dòng 3 dịch là "Kẻ Thắng Cuộc Số Một", nhưng glossary chốt "Đầu Hào Doanh Gia" (16 lần), là bản phiên âm khó hiểu.
8. **GLOSSARY_DANH_XUNG (vừa, hệ thống)**: 老裴 bị lưu thành tên riêng "Lão Bùi", nên viết hoa giữa câu 33 lần.
   - Ví dụ 0024 dòng 53: "Giang Dã đi chơi bóng rồi, Lão Bùi chắc đang có tiết." Nên viết thường "lão Bùi".
9. **XUNG_HO_VAI_VE (vừa)**: 0127 dòng 103, chị ruột nói với em trai "Con anh chẳng phải…".
   - Trong khi bố Tống gọi bà là "Chị à". Glossary ghi 宋巧云→宋父 = "chị–anh", tự mâu thuẫn.
   - Đề xuất: "Con cậu…".
10. **CALQUE (thấp, hệ thống)**:
    - 后台 → "hậu đài" 6 lần trong 4 file. Ví dụ 0189 dòng 71 "đàm phán rót vốn ở hậu đài Tinh Thần Giải Trí"; tuỳ ngữ cảnh nên là "hậu trường" hoặc "trang quản lý".
    - Cùng kiểu: "diễn viên đặc ước", "Triệu chỉ đạo".
11. **SAI_NGHIA (thấp)**: 0189 dòng 9, `等分红` → "chờ chia hoa hồng". Đúng phải là "chia cổ tức".
12. **VAN_KHO (thấp)**: 0188 dòng 77, `脸上的表情明显写着不服` → "vẻ mặt rõ ràng viết đầy sự bất phục". Đề xuất: "mặt mũi thì rõ là không phục".
13. **MEME_NGHIA_DEN (thấp)**: 0189 dòng 83, `科技狠活` → "công nghệ hóa chất". Bản dịch làm mờ meme về phụ gia thực phẩm.
14. **LAP_TU (thấp)**: 0095 dòng 65, "rồi lại tiếp tục xem tiếp".
15. **BAN_THAN (thấp)**: 0001 dòng 21, "Bản thân hắn cũng thấy chột dạ". Nên viết "Chính hắn…". Chỉ gặp 8 lần trên 446 lần 自己.

## Gợi ý sửa prompt / quy trình / app

- **Quy đổi đơn vị số trong prompt:** thêm bảng 万 = 10 nghìn, 亿 = 100 triệu, 百亿 = 10 tỷ, 千亿 = 100 tỷ, kèm ví dụ. Sửa luôn summary trong story.json.
- **Checker xưng hô:**
  - Tắt luật "ngôi một tôi → ta" khi style.voice ghi ngôi ba.
  - Luật "xưng hô cổ trang" phải bỏ qua từ "con ngươi" và thoại thuộc phim cổ trang.
  - Nên thêm cờ "phim trong phim / đa bối cảnh" cho genre.
- **Glossary tự sinh:**
  - Chuẩn hoá chức danh: X导 → "đạo diễn X"; 老X → "lão X" viết thường, không đưa vào names.
  - Không biến 老太太/老先生 thành "lão thái thái"/"lão tiên sinh" khi setting là modern.
- Chốt thuật ngữ thể loại (丧尸, 副本, 主脑) ngay lần đầu gặp để không bị trôi.
- Kiểm tra cặp addressing A→B với B→A cho khớp vai vế; mục nào mâu thuẫn thì đánh cờ.
- Tên tác phẩm trong truyện: ưu tiên dịch nghĩa, và báo khi chương dịch khác glossary.

## Làm tốt

- Không sót đoạn: số dòng khớp ở cả 189 chương.
- Chuyển giọng rất chuẩn giữa đời thực hiện đại và phim trong phim cổ trang.
- Lóng sống động: "Ngon vãi cả chưởng", "đến tết Công Gô mới xong", "giàu nứt đố đổ vách", "Thời An nhà tôi".
- Không có "Ân", không sót chữ Hán, không dịch 呵 thành "Hừ"; tiêu đề chương viết hoa đúng kiểu câu.
- Xưng hô gia đình (bà ngoại, cô, mợ, dượng) và cách gọi của hệ thống ("ký chủ", "anh Thống") nhất quán.
