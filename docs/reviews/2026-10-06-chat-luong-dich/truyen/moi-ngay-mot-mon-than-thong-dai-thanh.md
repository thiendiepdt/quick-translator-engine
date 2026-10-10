# Review: Mỗi Ngày Một Môn Thần Thông Đại Thành

- **Slug:** moi-ngay-mot-mon-than-thong-dai-thanh
- **Setting trong story.json:** ancient
- **Thể loại thực tế:** huyền huyễn/võ đạo hệ thống sảng văn, cổ đại pha quỷ dị, về sau lên giới diện vạn tộc
- **Ngôi kể:** ngôi 3, rất ít thoại
- **Đã dịch:** 1045 chương
- **Chương đã đọc trọn (raw + out):** 0001, 0520, 1044, 1045 (1044 và 1045 là hai chương dịch mới nhất). Ngoài ra đã grep toàn bộ out/ để xác nhận lỗi lặp.

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 7 | Ít thoại, ta–ngươi nhất quán. Lỗi chủ yếu: 你 chung chung trong lời kể thành "ngươi", cấp dưới xưng "ta" với quản sự, "hắn" mơ hồ chỉ hai người. |
| Giọng văn | 7 | Hán Việt cổ phong hợp thể loại, cảnh giới chuẩn. Nhưng câu cứng, ít nhịp dồn dập. |
| Dễ hiểu | 6 | Calque 随着→"Theo việc/sự", "phiến" khá dày; vài câu hỏng và sai con số. |
| Khô khan | 6 | Meme và chỗ đùa của tác giả bị dịch nghĩa đen. |
| Thuật ngữ | 5.5 | 面板 dịch 2 kiểu, 阶 dịch 2 kiểu, 佟 phiên sai, tiêu đề viết hoa mọi chữ ở chương mới. |

## Lỗi (nặng → nhẹ)

1. **THUAT_NGU_TROI, hệ thống, cao.** 面板 có 2 cách dịch: "bảng giao diện" 245 lần/110 file, "bảng điều khiển" 303 lần/135 file.
   - Ví dụ ch1044 dòng 17: 有面板的融合规则功能 → "chức năng dung hợp quy tắc của bảng điều khiển", trong khi ch0001 dùng "bảng giao diện".
   - Glossary không có mục 面板. Bản thân style.toneRules lại ghi "bảng điều khiển", nên hồ sơ truyện tự mâu thuẫn.

2. **THUAT_NGU_TROI, hệ thống, cao.** 阶 có 2 cách dịch.
   - Ch1045 dòng 117: 七阶开天境中期 → "Khai Thiên cảnh trung kỳ giai bảy". Ch0520 lại viết "quỷ dị tứ giai".
   - Kiểu "giai + số thuần Việt" xuất hiện 87 lần/21 file, phần còn lại của truyện dùng kiểu "tứ giai…" (khoảng 840 lần).
   - Đề xuất: "thất giai Khai Thiên cảnh trung kỳ".

3. **GLOSSARY_PHIEN_AM, hệ thống, cao.** 佟仲秋 bị phiên thành "Đông Trọng Thu" (406 lần/69 file). 佟 đọc Hán Việt là **Đồng**. Lỗi lan sang cả mục xưng hô "Đông lão", "Đông huynh".

4. **TIEU_DE, hệ thống, mới xuất hiện.** Ba chương 1043–1045 viết hoa mọi chữ.
   - Ví dụ: "Đột Phá, Dung Đạo Cảnh Trung Kỳ", "Quân Lâm".
   - Ch1042 trở về trước và titles.json đều viết thường kiểu câu.
   - Đề xuất: "Đột phá, Dung Đạo cảnh trung kỳ".

5. **DAI_TU_MO_HO / sai nghĩa, lẻ.** Ch0520 dòng 35.
   - Raw: 那他这么多年，也是白修炼了 (他 = Tần Hải Sam).
   - Hiện tại: "…để hắn trốn thoát thì bao nhiêu năm nay hắn cũng tu luyện uổng phí rồi". Hai chữ "hắn" đều đọc như Giản Tấn Sinh.
   - Đề xuất: "…mà còn để y chạy thoát thì bao năm tu luyện của Tần Hải Sam coi như vứt đi".

6. **CALQUE, hệ thống.** "Theo việc/sự" 116 lần/101 file, "phiến" 46 lần/19 file.
   - Ch1044 dòng 103: "Theo việc ở Nhật Nguyệt cảnh trong Càn Khôn Phủ được sử dụng gần như không hạn chế các loại tài nguyên, cùng với nền tảng…".
   - Đề xuất: "Hồi Nhật Nguyệt cảnh ở Càn Khôn Phủ được dùng tài nguyên gần như vô hạn, lên Dung Đạo cảnh mỗi ngày lại có hai ngàn hạ phẩm nguyên tinh…".
   - Cùng kiểu lỗi: ch1045 dòng 103 "phiến khu vực này", dòng 19 "một đoàn hỏa diễm hư ảo".

7. **SAI_NGHIA / câu hỏng, lẻ.** Ch1045 dòng 123.
   - Raw: 不说是个笑话，但也可能无法继续打下去.
   - Hiện tại: "…cũng có thể không thể tiếp tục đánh nhau được nữa".
   - Đề xuất: "chưa đến mức thành trò cười, nhưng e là cũng chẳng đánh tiếp nổi".

8. **MEME_NGHIA_DEN, hệ thống.** Ch0001 dòng 131.
   - Raw: 这么变态，陈斐表示，他很喜欢！
   - Hiện tại: "Biến thái như vậy, Trần Phỉ biểu thị, hắn rất thích!"
   - Đề xuất: "Bá đạo cỡ này à? Trần Phỉ xin nói thẳng: hắn thích lắm!"
   - Cùng kiểu: 苍白 → "tái nhợt vô lực" (ch1044 dòng 87). Cụm "tái nhợt" có 67 lần/62 file.

9. **NGUOI_CHUNG, hệ thống.** Chữ 你 chỉ người chung chung trong lời kể bị dịch thành "ngươi".
   - Ch1045 dòng 109: "ngươi đột ngột di dời tới đây…" → đề xuất "tộc nào đột ngột dời tới đây…".
   - Gặp thêm ở ch0001 dòng 15/41/47/91 và ch0520 dòng 113.

10. **VAN_KHO, lẻ.** Ch1045 dòng 73: 得心应手 → "thuận buồm xuôi gió" là dùng sai thành ngữ. Đề xuất: "như tay sai khiến".

11. **XUNG_HO_VAI_VE, gốc ở hệ thống.** Ch0001 dòng 137: tạp dịch nói với quản sự "Vừa rồi ta thấy hắn…" → đề xuất "tiểu nhân thấy…". Bảng addressing để mặc định ta–ngươi cho 191/405 cặp.

12. **SAI_SO, lẻ.**
    - Ch1045 dòng 59: 再崩碎三千块 → "lại đập vỡ thêm ba ngàn khối". Chữ "thêm" khiến người đọc hiểu thành 5000 khối/ngày, trong khi tổng đúng là 3000.
    - Ch0001 dòng 237: 足足快了一倍 → "nhanh hơn hẳn một nửa". Đúng phải là "nhanh gấp đôi".

13. **LAP_TU, hệ thống, nhẹ dần ở các chương sau.** Ch1044 dòng 31: "…cơ thể Trần Phỉ, Trần Phỉ càng vận chuyển…". Ch0001 dòng 77 cũng vậy.

**Ghi chú thêm:**
- "Ân" (嗯) xuất hiện 66 lần/52 file, "bản thân hắn" 62 lần/59 file, "tiện nghi" 9 lần/8 file. Chưa kiểm ngữ cảnh ngoài mẫu, nhưng mật độ thấp vì truyện ít thoại.
- Không có lỗi bối cảnh, vì truyện thuần cổ đại.

## Gợi ý sửa prompt/quy trình

1. **Chốt thuật ngữ cốt lõi:**
   - 面板 = "bảng giao diện", đồng thời sửa toneRules cho khớp.
   - Luật "N阶 → N giai dạng Hán Việt đặt trước".
   - Sửa 佟 → Đồng và thay hàng loạt trong out/.
   - Thêm post-check đếm các biến thể của thuật ngữ cốt lõi.
2. **Danh sách cấm calque:**
   - 随着 → tách câu, dùng "khi/nhờ".
   - 这片 → không dùng "phiến".
   - 一团 → không dùng "một đoàn".
   - 苍白 nghĩa bóng → không dùng "tái nhợt".
   - 你 chung chung trong lời kể → "người ta/kẻ nào" hoặc lược bỏ.
   - Câu có 再/又/一倍 phải giữ đúng con số.
3. **Tiêu đề:** đặt luật cứng trong code là viết thường kiểu câu. Kiểm tra commit cd752eb, nhiều khả năng là nguyên nhân 1043–1045 bị viết hoa mọi chữ.
4. **Xưng hô/đại từ:**
   - Cấp dưới với cấp trên xưng "tiểu nhân/thuộc hạ"; không để auto-glossary mặc định ta–ngươi.
   - Sau lần nhắc tên đầu trong đoạn thì dùng "hắn".
   - Tránh hai chữ "hắn" chỉ hai người khác nhau trong một câu.
   - Với câu đùa/meme của tác giả, cho phép dịch thoáng.

## Làm tốt (đừng sửa nhầm)

- Đủ đoạn, không còn chữ Hán sót.
- Giữ đúng định dạng 【…】; các bậc nhập môn/tinh thông/viên mãn/đại viên mãn và tên cảnh giới khớp glossary.
- Nhiều thành ngữ dùng khéo: "đêm dài lắm mộng", "nước chảy thành sông", "sơn cùng thủy tận", "đầu sóng ngọn gió".
- Tên công pháp/thần thông nhất quán; các chương mẫu không có từ trong danh sách cấm (cư nhiên, thân ảnh…).
