# Review: Cửu Thúc: Bắt Đầu Trọng Đồng, Luyện Sư Phụ Thành Cương Thi

- Setting: `ancient`. Thực tế: đồng nhân Cửu Thúc, tu tiên hắc ám cuối Thanh/Dân Quốc, nhiều meme. Kể ngôi 3.
- 154 chương đã dịch. Đọc trọn: 0001, 0078, 0153, 0154. Grep toàn bộ out/ để đếm lỗi hệ thống.

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 7.5 | Các cặp chính đúng vai vế và ổn định. Dữ liệu addressing/cast thì có mục lỗi thời, mục mâu thuẫn và một người bị tách thành nhiều nhân vật. |
| Giọng văn | 7.5 | Gãy gọn, giữ chất sảng văn hắc ám. Còn tiểu từ "a" kiểu convert và vài chỗ Hán Việt nặng. |
| Dễ hiểu | 7.5 | Trơn, sát nghĩa. Lác đác calque và sai chi tiết nhỏ. |
| Khó khăn | 7 | Nhiều meme Việt hoá khéo. Nhưng 无根, 麻了, 全垒打 bị dịch nghĩa đen, còn "Ân" làm khô nhịp hài. |
| Thuật ngữ | 5.5 | Tên nhân vật chính phiên âm sai trên toàn truyện. 清廷 và 好师父 mỗi từ trôi 2–3 cách dịch. |

## Lỗi (nặng → nhẹ)

1. **GLOSSARY_PHIEN_AM, cao, hệ thống** (Lý Dực 2263 lần/153 file). Ch0001 dòng 3: `煜儿` → "Dực nhi" → **"Dục nhi"**. Âm Hán Việt của 煜 là "dục" (李煜 = Lý Dục), không phải "dực". Lỗi nằm sẵn trong story.json nên lan ra cả truyện.
2. **THUAT_NGU_TROI, cao, hệ thống.** 清廷 (108 lần/32 file) được dịch thành "Mãn Thanh" 51 lần/15 file, "Thanh Đình/thanh đình" 50 lần/16 file và "nhà Thanh" 5 lần/4 file. Mỗi cụm chương dùng một cách. Từ này xuất hiện từ ch27 nhưng mãi ch137 mới vào glossary. Ví dụ ch0078 dòng 89: "triều đình Mãn Thanh nhờ vả đệ".
3. **THUAT_NGU_TROI, vừa, hệ thống.** 好师父 (159 lần) được dịch thành "sư phụ tốt" ~88 lần/24 file và "hảo sư phụ" 71 lần/21 file. Ch0154 dòng 141: "Móng vuốt của sư phụ tốt…" → **"của hảo sư phụ"**. Đây là biệt danh mỉa mai nhưng chưa có trong glossary.
4. **GLOSSARY_TRUNG, vừa, hệ thống.** 任发 và 任老爷 là hai mục riêng, mỗi mục một kiểu xưng hô ("tại hạ–đại sư" và "ta–đại sư"). 九叔/林九/一眉/一眉道人/林凤娇 thành 5 nhân vật, trong khi 林凤娇 là tên thật của Cửu Thúc do Giá Cô gọi. Các cặp 龙大帅/龙大福 và X/X真人 cũng bị tách đôi.
5. **CAST_MAU_THUAN, vừa.** Addressing "李煜→九叔: ta–Nhất Mi đạo hữu" trong khi raw dùng 林道友: bản dịch có "Lâm đạo hữu" 48 lần/23 file, "Nhất Mi đạo hữu" chỉ 3 lần. Bản dịch đúng nhờ bám raw, nhưng mục glossary sai.
6. **AN_THAN_TU, vừa, hệ thống** (31 lần/30 file, không lần nào ra "Ừm"). Ch0001 dòng 33: `嗯....绝不是…` → "Ân.... Tuyệt đối không phải…" → **"Ừm.... Tuyệt đối không phải…"**. Câu này là giọng kể tự trào, dùng "Ân" thì mất nhịp hài.
7. **CALQUE, vừa, hệ thống.** Tiểu từ "a" dịch từ 啊 xuất hiện ~90 lần/37 file, dồn ở ch21–40 và 86–106. "Đem…" đầu câu 10 lần/9 file. Ch0025 dòng 31: "ghen tị đến phát cuồng a!" → **"…phát điên mà!"**.
8. **MEME_NGHIA_DEN, vừa** (ch0078 dòng 217, ch0079). `无根先生` → "tiên sinh không rễ" → **"tiên sinh 'mất của quý'"**. Đây là câu chửi thái giám, dịch nghĩa đen thì người đọc không hiểu.
9. **MEME_NGHIA_DEN, thấp, hệ thống.** 麻了 (8 lần/6 file) thành "tê rần/tê dại cả người"; nghĩa đúng là **"đơ người, cạn lời"**. 全垒打 thành "Cú đánh toàn lũy" (ch0114), người Việt đọc không hiểu.
10. **CALQUE, thấp.** Ch0154 dòng 9: "khí tức lại vô hạn áp sát tới Nguyên Anh" → **"khí tức vậy mà đã sát ngưỡng Nguyên Anh"**.
11. **SAI_NGHIA, thấp.** Ch0154 dòng 7: `千足` → "ngàn đốt" → **"ngàn chân"**.
12. **GLOSSARY_HAN_VIET_NANG, thấp** (10 lần, ch0038). `表姨夫` → "Biểu di trượng, đừng mà biểu di trượng." → **"Dượng ơi, đừng mà dượng!"**. Cách dịch này vừa nặng vừa sai âm (姨夫 đọc là "di phu").
13. **NGUOI_CHUNG, thấp** (ch0001, ch0010). `你以为他躺在太师椅上是装逼` → "Ngươi tưởng hắn nằm…" → **"Đừng tưởng hắn nằm…"**.
14. **HU_HA, thấp, hệ thống** ("Hì hì" 6 lần). Ch0078 dòng 143: Thạch Kiên đang giận mà `呵呵` lại thành "Hì hì..." → **"Hà..."**. Nhìn chung 呵呵 được dịch lẫn lộn: Hì hì/Hừ/Khà khà/Haha.
15. **DINH_DANG, thấp, hệ thống.** 30 file (0011–0020, 0051–0070) dùng ngoặc thẳng `"` thay cho “ ” như raw.

## Gợi ý

- Lúc setup-story, kiểm âm Hán Việt của tên chính bằng từ điển và cảnh báo chữ hiếm/đa âm (煜 → Dục).
- Auto-glossary nên chốt thuật ngữ lặp ngay sau 2 chương (清廷, 好师父, 义庄) rồi chạy rà ngược để thay các biến thể đã dịch.
- Cast cần có alias cho một người nhiều tên, mỗi cặp nhân vật chỉ một mục addressing, và báo khi các mục mâu thuẫn nhau.
- Addressing tự sinh phải được làm mới khi raw đổi cách gọi. Họ hàng đời thường thì dùng từ thuần Việt (dượng).
- Prompt: bỏ luật bắt buộc 嗯→"Ân"; cấm tiểu từ "a" cuối câu; thêm bảng lóng mạng (麻了, 破防, 无根, 全垒打) kèm luật ưu tiên nghĩa lóng.
- Hậu xử lý: chuẩn hoá ngoặc kép, đổi "! ! !" thành "!!!".

## Làm tốt

- Đủ đoạn, không còn chữ Hán, tiêu đề viết hoa kiểu câu.
- Hài đen và chửi thề giữ được độ gắt: "giả nhân giả nghĩa cái mẹ gì", "lấy cổ đi so tài với xà nhà".
- Meme Việt hoá khéo: "xe ba gác đâm đầu vào xe tải hạng nặng", "gái hãm", "hàng mã".
- Xưng hô trong sư môn Mao Sơn và giữa Vương Mộng Nhi với Lý Dực tự nhiên. Raw viết nhầm 他 cho Tần Nguyệt, bản dịch vẫn ra đúng "Nàng".
- 自己 xuất hiện 582 lần trong raw nhưng bản dịch chỉ còn 36 lần "bản thân hắn". Cảnh chiến đấu dồn dập, đúng style.
