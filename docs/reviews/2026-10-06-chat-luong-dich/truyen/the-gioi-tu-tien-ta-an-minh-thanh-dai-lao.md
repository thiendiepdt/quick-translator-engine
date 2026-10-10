# Review: Thế Giới Tu Tiên, Ta Ẩn Mình Thành Đại Lão

- Slug: `the-gioi-tu-tien-ta-an-minh-thanh-dai-lao` · setting `ancient` · ngôi 3 · 145 chương đã dịch
- Thể loại thật: tiên hiệp xuyên không, cẩu đạo sảng văn; lời kể nhiều meme mạng hiện đại
- Đã đọc trọn: 0001, 0073, 0144, 0145 (hai chương cuối là bản dịch mới nhất)

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| xung_ho | 7.0 | Thoại tông môn phần lớn chuẩn. Lời kể gọi cùng một nhân vật lúc hắn, lúc gã, y hay lão tuỳ chương; addressing trong story.json mâu thuẫn với cast.json |
| giong_van | 7.5 | Giọng gãy gọn, đúng tiên hiệp; còn vài thành ngữ Hán Việt nặng |
| de_hieu | 7.5 | Đọc trôi, ít sai nghĩa nhưng nhiều văn convert ("trực tiếp", "kết quả", "một loại", "đem") |
| kho_khan | 7.0 | Nhiều meme xử lý khéo; vài meme dịch sát chữ; "Ân!" làm thoại khô |
| thuat_ngu | 7.5 | Khớp glossary, không sót chữ Hán; tượng thanh dịch hai kiểu, biệt danh lóng bị phiên âm |

## Lỗi (nặng → nhẹ)

1. **CALQUE (hệ thống, 297 lần/114 file):** 直接 bị dịch thành "trực tiếp". Ví dụ 0145 dòng 9: `韩雨直接开口问道` → "Hàn Vũ trực tiếp mở miệng hỏi." Nên sửa thành "Hàn Vũ hỏi thẳng."
2. **AN_THAN_TU (hệ thống, 48 lần/40 file, có ở các chương mới nhất):** 0143 dòng 73: `嗯！你去吧！` → "Ân! Ngươi đi đi!" Nên sửa thành "Được! Ngươi đi đi!" Lỗi này do mục 11 của prompt bắt dịch 嗯 thành "Ân".
3. **XUNG_HO_TROI (hệ thống):** Lời kể gọi Từ Kiến Minh lúc "gã" (0073 dòng 5: "Gã có một loại trực giác…"), lúc "y" (0084), "hắn" (0145), "lão" (0058, 0072). Bảng đại từ chỉ quy định 他 là "hắn", nên chương nào model cũng tự chọn một đại từ khác để phân biệt với Hàn Vũ. "Gã" còn mang sắc khinh với một tông chủ được tả thiện cảm.
4. **CAST_MAU_THUAN (hệ thống, ít nhất 4 cặp):** story.json ghi 徐建明→韩雨 là "ta–ngươi", còn cast.json từ chương 0066 ghi "ta–Hàn Thái thượng". Tương tự, 韩雨→柳烟雨 trong story.json là "đệ tử–Liễu phong chủ", còn cast.json ghi đổi dần sư tôn → đạo hữu → sư tôn. Prompt chương 0146 vẫn in cặp cũ.
5. **XUNG_HO_VAI_VE (lẻ, 0111 dòng 61 và 81):** "Hàn Thái thượng, **ngươi** đây là lại thiếu đan dược…" Các chương lân cận đều dùng "ngài". Đây là hệ quả trực tiếp của lỗi 4.
6. **NGUOI_CHUNG (hệ thống, khoảng 45 dòng/37 file):** 0001 dòng 21: "có thể **ngươi** đang đi trên đường thì đã bị dư ba… chấn chết." Nên sửa thành "có khi người ta đang đi trên đường…". Lỗi gặp nhiều ở các đoạn giải thích luật tông môn.
7. **MEME_NGHIA_DEN (lẻ):** 0144 dòng 139: `乌龟：6` → "Con rùa: 6" (nên là "Con rùa: …(cạn lời)"). Cùng chương, 菊花残满地伤 thành "cúc hoa tàn ngập tràn đau đớn". 0073: 焕发第二春 thành "nghênh đón mùa xuân thứ hai".
8. **MEME_NGHIA_DEN / glossary (lẻ):** 逼王 bị lưu vào nhóm names nên dịch thành "Bức Vương" (0015), nên là "Vua Làm Màu". Tên giả chọc cười 韩跳跳 cũng bị phiên thành "Hàn Khiêu Khiêu".
9. **CALQUE (hệ thống):** "một loại" 27/22 file (ví dụ tiêu đề 0073 "Có một loại trực giác", nên là "Linh cảm"); "đại biểu/đại diện cho" 11/9 file; "kết quả" 73/55 file.
10. **VAN_KHO (lẻ):** 0073 dùng "danh liệt tiền mâu", "trường thịnh bất suy", "huyết tế chi pháp", trái với style.avoid. Ví dụ sửa: "thuộc hàng đứng đầu".
11. **SAI_NGHIA (lẻ):** 0073 dòng 23: 临时出走 → "tạm thời bỏ đi". Nghĩa đúng là "đến lúc vội vã bỏ đi".
12. **CALQUE (hệ thống, mức nhẹ):** "đem" 37/29 file (0145: "muốn **đem** tán tu làm mồi nhử", nên là "lấy… làm"); 随着 dịch thành "Theo…" 12 lần.
13. **THUAT_NGU_TROI:** 桀桀 dịch thành "Kiệt kiệt" 15 lần và "Khặc khặc" 4 lần.
14. **DINH_DANG:** "Động phủ" viết hoa giữa câu 28 lần/10 file, trong khi "động phủ" viết thường 99 lần.
15. **TIEU_DE:** 5 tiêu đề viết hoa mọi chữ, ví dụ "Chương 116: Ngươi Đã Tự Tìm Đường Chết".

## Gợi ý

- Thêm bảng hư từ dễ convert vào prompt:
  - 直接 → thẳng/luôn/liền hoặc bỏ
  - 结果 → ai ngờ/rốt cuộc
  - 一种 → bỏ "một loại"
  - 代表 → nghĩa là
  - 随着 → khi
  - 把/将 → đảo câu, không dùng "đem"

  Thêm checkRule cảnh báo khi một chương có hơn 3 lần "trực tiếp".
- Sửa mục 11 của prompt: 嗯 khi đồng ý dịch thành "Được/Ừ" hoặc lược; bỏ quy định ép "Ân".
- Bảng đại từ: thêm ngoại lệ cho 你 phiếm chỉ trong lời kể → "người ta/ai" hoặc bỏ chủ ngữ.
- cast.json: thêm trường đại từ ngôi ba cố định cho lời kể của từng nhân vật (ví dụ 徐建明: y) và đưa vào prompt.
- Khi cast.json đã có mốc cho một cặp xưng hô, app ẩn hoặc ghi đè cặp trùng trong story.json.
- Thêm vào customPrompt quy tắc dịch meme/lóng mạng Trung bằng lóng Việt tương đương. Auto-glossary không lưu biệt danh lóng vào nhóm names.
- Chốt trong glossary: 桀桀 → "Khặc khặc"; 洞府 → "động phủ" (chữ thường).
- Hậu kiểm tiêu đề: tự hạ về sentence case.

## Làm tốt

- Đủ đoạn, không sót chữ Hán; tên, cảnh giới, đan dược nhất quán theo glossary.
- Chơi chữ "对得起…吗？对不起！" dịch thành "có xứng… không? Không xứng!".
- Nhiều meme chuyển sang lóng Việt tốt: "làm công như trâu ngựa", "ngàn dặm tới nộp mạng", "mất mạng như chơi", "chẳng thèm ngó ngàng", "ngươi đúng thật là chó".
- Câu nhại Lã Bố "nguyện bái làm nghĩa phụ" giữ đúng nhịp văn cổ.
- Thoại đúng vai vế (vãn bối–tiền bối, Hàn Thái thượng–ngài, sư tôn–con). 嗯？ dịch linh hoạt thành A?/Hử?/Sao?.
- Nhận ra lỗi gõ trong raw (炼巅峰) và dịch đúng thành "Luyện Khí đỉnh phong".
