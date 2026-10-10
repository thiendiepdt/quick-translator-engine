# Review: Ác Nữ, Ta Muốn Ngươi Giúp Ta Tu Hành

- Slug: `ac-nu-ta-muon-nguoi-giup-ta-tu-hanh`. Setting trong story.json: `ancient`. Kể ngôi 3.
- Thể loại thật: đồng nhân tổng võ Kim Dung, xuyên không kèm hệ thống ác niệm, sảng văn hậu cung. Bối cảnh cổ trang, nhưng tác giả chen rất nhiều meme và lóng mạng.
- Đã dịch 1028 chương. Đọc trọn 4 chương: 0001, 0515, 1029, 1030 (hai chương sau cùng có mtime mới nhất là 2026-10-04).

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 6.5 | Trong từng chương thì hợp vai. Giữa các chương thì trôi: Trần Ngọc xưng với sư phụ từ "con" sang "ta", Thang Phái lúc "lão" lúc "hắn". Glossary.addressing và cast.json trùng 75 cặp thì cả 75 đều mâu thuẫn. |
| Giọng văn | 7.5 | Văn võ hiệp gọn, nhịp sảng văn tốt. Bị trừ điểm vì dịch "Ân" và vài cụm Hán Việt nặng. |
| Dễ hiểu | 7.5 | Phần lớn thuần Việt. Còn văn convert ("đem", "Theo việc…", "chiếm tiện nghi"), "bản thân" dày và vài chỗ sai nghĩa. |
| Khó khăn (lóng, hài) | 7.0 | Phần lớn lóng xử lý hay. Chơi chữ đồng âm và meme tự xưng thì bị dịch nghĩa đen. |
| Thuật ngữ | 7.0 | Tên Kim Dung chuẩn, không sót chữ Hán. Glossary phình to và tự mâu thuẫn, có cụm mô tả bị chốt thành tên riêng. |

## Lỗi (nặng → nhẹ)

1. **CAST_MAU_THUAN** (cao, hệ thống): story.json `glossary.addressing` và `cast.json` trùng 75 cặp, cả 75 đều khác nhau.
   - Ví dụ 阿朱→陈钰: glossary ghi "tiểu tăng–Đoàn thế tử" (lúc A Chu cải trang làm sư), cast ghi "muội–Ngọc lang". Glossary đóng băng giá trị lần gặp đầu.
2. **XUNG_HO_TROI** (cao, hệ thống): Trần Ngọc nói với sư phụ Chu Mỹ Xước.
   - Raw: 师父，我无事
   - Hiện tại: "Sư phụ, ta không sao" (1030, dòng 223)
   - Đề xuất: "Sư phụ, con không sao"
   - Từ chương 1018 đến 1029 đều xưng "con", ví dụ 1029 dòng 117: "Sư phụ, con đâu có nói người". Glossary lại ghi sai "ta–sư phụ", nhiều khả năng chính mục này kéo bản dịch lệch.
3. **XUNG_HO_TROI** (vừa): Thang Phái là lão giả hơn 60 tuổi. Chương 1029 gọi "lão" cả chương (41 lần nhắc tên), còn 1026–1028 và 1030 gọi "hắn".
   - Ví dụ 1030 dòng 167: "bỗng hắn cảm thấy dưới chân vấp phải…" → nên là "lão".
4. **AN_THAN_TU** (vừa, hệ thống): có 689 dòng chứa 嗯, trong đó 324 dòng dịch thành "Ân" (243 file). Tỉ lệ tăng lên khoảng 56% từ chương 1000 trở đi.
   - Ví dụ 0515 dòng 69: “Ân, nhớ kỹ rồi.” → nên là “Vâng, nhớ kỹ rồi.”
   - Cùng chương đó, “嗯？” lại dịch thành “A?”.
5. **MEME_NGHIA_DEN** (vừa): chơi chữ 整人菌子 (đồng âm với 正人君子) bị dịch nghĩa đen.
   - Hiện tại: "Ta là cây nấm chỉnh người" (1029, dòng 213)
   - Đề xuất: "Ta là chính nhân quân tử… à không, *chỉnh nhân* quân tử"
6. **MEME_NGHIA_DEN** (vừa): 星宿大爹 dịch thành "Tinh Túc đại cha" (4 lần/2 file). 大爹 là lóng tự xưng kiểu "bố đời", nên dịch "bố già Tinh Túc".
7. **SAI_NGHIA** (vừa): 投名状 dịch thành "làm chứng nhận nộp mình" (1029, dòng 253). Các chương khác, kể cả tiêu đề chương 959, đều dùng "đầu danh trạng".
8. **BAN_THAN** (vừa, hệ thống): "bản thân" xuất hiện 2261 lần/834 file; 1989/8529 dòng có 自己 bị dịch thành "bản thân".
   - Ví dụ 0515 dòng 117: "cố gắng để bản thân biểu hiện thật tự nhiên" → "cố tỏ ra thật tự nhiên".
9. **CALQUE** (vừa, hệ thống): "tiện nghi" 61 lần/55 file, trong đó "chiếm tiện nghi" 33 lần.
   - Ví dụ 0515 dòng 337: 占你便宜 → "chiếm tiện nghi của chàng" → nên là "nhận vơ làm chú chàng".
10. **CALQUE** (vừa, hệ thống): "đem" 604 lần/392 file; "Theo việc/sự…" 69 lần/63 file.
    - Ví dụ 1029 dòng 13: "Quả nhiên, theo ánh mắt lạnh lẽo từ trên đỉnh đầu truyền tới." Câu cụt, không có vị ngữ.
11. **NGUOI_CHUNG** (thấp): 0515 dòng 21 và 25: "Một nữ tử biết ngươi đối tốt với nàng…", "kẻ vừa bòn rút tiền của ngươi…". Ở đây 你 chỉ người chung chung, nên dịch "người ta/mình".
12. **DINH_DANG** (thấp, hệ thống): 250/1028 file dùng ngoặc thẳng "…" (raw dùng “…”), ví dụ 0031 dòng 51. Ngoài ra, khi lặp từ có lúc viết hoa "Tiền... Tiền bối" (141 lần), có lúc viết thường (538 lần).
13. **GLOSSARY_HAN_VIET_NANG** (thấp): auto-glossary xếp 灰袍客 vào names nên thành "Hôi Bào Khách" (8 lần). Cùng chương (1030 dòng 137) lại viết "đám người áo xám". Ngoài ra còn "hắc bào nữ tử" và "toàn thần quán chú" (13 lần/12 file).
14. **THUAT_NGU_TROI** (thấp): 刘慧 trong glossary là Lưu Tuệ, 小慧 là Tiểu Tuệ. Nhưng mục addressing ghi "Tiểu Huệ/Huệ nhi", và chương 1 dịch theo đó.
15. **DAI_TU_MO_HO** (thấp): 1030 dòng 127: "giật lấy trường kiếm trong tay nàng… lo nàng sẽ làm vướng chân đồ nhi nhà nàng". Ba chữ "nàng" trỏ hai người.

Lỗi lặt vặt khác:
- 摩拳擦掌 dịch thành "xoa tay bóp trán" (2 lần; những chỗ khác dịch đúng là "xoa tay bẻ khớp").
- 瞧不上 dịch thành "lão làm sao chướng mắt nổi", ngược nghĩa (0515 dòng 217).
- Lỗi chính tả "Khoảnh khoác" (1030 dòng 59).
- Dòng rác cuối chương 本书地图 dịch thành "Bản đồ của sách" (0001).

## Gợi ý

- Bỏ addressing khỏi glossary, hoặc không nạp nó vào prompt khi đã có cast.json. Nếu vẫn giữ, mỗi mốc cast mới phải ghi đè mục glossary cùng cặp.
- Trong cast.json:
  - ghi đại từ ngôi ba cố định cho nhân vật phụ dài hơi (汤沛: "lão");
  - lấy mốc xưng hô theo chương gần nhất, không lấy giá trị auto sinh lần đầu.
- Bỏ quy tắc bắt buộc 嗯 → "Ân". Thay bằng bảng thán từ theo ngữ cảnh: Ừ/Vâng/Dạ, Ừm…, Hửm?.
- Thêm luật chống convert:
  - 自己: lược đi hoặc thay bằng đại từ;
  - 随着: không dịch "Theo việc";
  - 占便宜: dịch theo nghĩa cụ thể;
  - hạn chế "đem".
- Thêm mục "đồng âm/chơi chữ" vào prompt: dịch theo nghĩa được ám chỉ và giữ miếng hài.
- Auto-glossary:
  - không nhận cụm mô tả (灰袍客) và danh xưng (邓大哥, 焦妹妹) làm tên riêng;
  - chốt thuật ngữ cốt lõi (投名状) vào mục common.
- Hậu xử lý:
  - chuẩn hoá ngoặc kép về “…”;
  - cắt dòng rác cuối chương;
  - kiểm chính tả.

## Làm tốt

- Tên người, môn phái, võ công Kim Dung chuẩn và nhất quán suốt hơn 1000 chương, không sót chữ Hán.
- Lóng và chửi xử lý sống: 木构 → "Con chó cái này", 老登 → "lão già này", 出个屁头 → "Ra mặt cái con khỉ", 软柿子 → "quả hồng mềm dễ nắn".
- Xưng hô theo vai trong từng cảnh tốt: vãn bối–tiền bối, Thuần ca–nàng, sư phụ–Ngọc nhi.
- Cả 1028 chương đều khớp số đoạn với raw. Tiêu đề chương viết hoa đúng kiểu câu.
- Cảnh Đao Bạch Phụng cắt đứt với Đoàn Chính Thuần giữ được cảm xúc.
