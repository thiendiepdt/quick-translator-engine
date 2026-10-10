# Review: Rút Đao Ngàn Tỉ Lần, Hùng Bá Bức Ta Kết Hôn

- Slug: `rut-dao-ngan-ti-lan-hung-ba-buc-ta-ket-hon`. Setting `ancient`. Thể loại thật: tổng võ (trộn Phong Vân, Kim Dung, Cổ Long, Tần Thời), có xuyên không và hệ thống, sảng văn harem. Lời kể cài nhiều meme mạng.
- Truyện kể ngôi 3, đã dịch 551 chương. Không có cast.json.
- Chương đã đọc trọn: 0001, 0276, 0550, 0551. Ngoài ra đã grep toàn bộ out/.

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 6.5 | Cặp cha–con Hùng Bá và Hùng Thiên Hạ xưng hô không ổn định. Đại từ ngôi 3 cho Hùng Bá lúc "hắn", lúc "lão", lúc "gã", lúc "ông". Phần addressing trong story.json mâu thuẫn với toneRules. |
| Giọng văn | 7.5 | Giọng võ hiệp hào sảng, nhịp đánh nhau tốt. Lời kể vẫn dày Hán Việt khuôn sáo, còn calque "đem" và "bản thân hắn". |
| Dễ hiểu | 7.5 | Phần lớn trôi chảy. Còn vài lỗi lẻ: sai nghĩa, thêm ý, `自己` trỏ nhầm người, lặp từ. |
| Khó khăn | 6 | Meme và lóng bị dịch nghĩa đen hoặc mất nhịp hài. "Ân" và "A" dùng xen kẽ không theo quy tắc. |
| Thuật ngữ | 6.5 | Tên riêng nhất quán. Glossary tự sinh có phiên âm sai, 腿 dịch ba kiểu, 7 cặp trùng mục, và hai chữ 剑神/剑晨 cùng ra "Kiếm Thần". |

## Lỗi (nặng → nhẹ)

1. **CALQUE, hệ thống, nặng.** "đem" xuất hiện 258 lần trong 174 file; "phiến" 48 lần trong 35 file; "Theo việc/sự" 20 lần trong 18 file.
   - Ví dụ 0276 dòng 77: `将所有残存真气…尽数灌入这一刀` → "đem tất cả chân khí còn sót lại… rót hết vào trong nhát đao này".
   - Đề xuất: "dồn hết chỗ chân khí còn sót lại… vào nhát đao này".
2. **BAN_THAN, hệ thống, nặng.** "bản thân hắn/nàng/lão" xuất hiện 241 lần trong 154 file.
   - Ví dụ 0551 dòng 7: "Quy Khư chân khí của bản thân hắn".
   - Đề xuất: "Quy Khư chân khí của hắn".
3. **XUNG_HO_VAI_VE, hệ thống, nặng.** 0551 dòng 173: `我儿，你回来了` → "Nhi tử của ta, ngươi đã về rồi."
   - Đề xuất: "Con trai ta, con về rồi đấy à."
   - Hùng Bá gọi U Nhược là "con" nhưng gọi con trai là "ngươi". Nguyên nhân là addressing 雄霸→雄天下 ghi "lão phu–ngươi".
4. **CAST_MAU_THUAN, hệ thống, vừa.** Addressing 雄天下→雄霸 ghi "ta–ngươi", còn toneRules ghi "phụ thân". Kết quả là Hùng Thiên Hạ gọi cha lúc "người" (0006–0007), lúc "ông" (0018 dòng 103: "Cha, ông là cha ruột của ta…").
5. **MEME_NGHIA_DEN, vừa.** 0019 dòng 45: `神他妈三分归元气，七分靠打拼` → "thần mẹ nó tam phân Quy Nguyên Khí, thất phân dựa phấn đấu".
   - Câu gốc chơi chữ với câu hát "ba phần trời định, bảy phần cố gắng"; bản dịch làm mất hẳn chỗ chơi chữ này.
   - Cùng loại lỗi: 0001 "Ngón tay vàng… đã tới tài khoản".
6. **MEME_NGHIA_DEN, vừa.** `小辣椒` bị dịch thành "Tiểu Lạt Tiêu" (10 lần trong 4 file, từ 0543 đến 0550), biến biệt danh thành tên riêng.
   - Đề xuất: "cô nàng ớt cay".
7. **CALQUE, vừa.** `好一个` xuất hiện 80 lần: dịch thành "Hay cho" 15 lần, nhưng có 10 lần thành "Được một cái" và 2 lần thành "Tốt cho".
   - Ví dụ: 0551 dòng 207 "Tốt cho nhi tử của ta!"; 0276 "Được một cái Ma Đao!"; 0516 "Được một cái Phật Môn".
8. **DAI_TU_MO_HO, lẻ, vừa.** 0551 dòng 23: `他不是刚刚才斩断了自己的魔念吗` → "Hắn chẳng phải vừa mới chém đứt ma niệm của mình sao?"
   - `自己` ở đây là Đệ Nhất Tà Hoàng, nhưng bản dịch khiến người đọc hiểu thành ma niệm của Hùng Thiên Hạ.
   - Đề xuất: "…chém đứt ma niệm của lão sao?"
9. **XUNG_HO_TROI, hệ thống, vừa.** Trong các đoạn có `他` nói về Hùng Bá, bản dịch dùng "hắn" 64 lần, "lão" 38, "gã" 5, "ông" 5.
   - Đầu truyện (0003 dòng 117) Hùng Bá là "hắn", trùng đại từ với nhân vật chính. Về sau chuyển thành "lão".
10. **AN_THAN_TU, vừa.** `嗯` có ở 47 đoạn: 20 đoạn ra "Ân", 20 đoạn ra "A".
    - Ví dụ 0094 dòng 151: "Ân, biết rồi." Nên là "Ừ, biết rồi."
11. **GLOSSARY_PHIEN_AM, hệ thống, vừa.** `综武` ra "Tống Võ" (0001 dòng 17, 0288, và cả trong summary); đúng phải là "tổng võ".
    - Cùng loại: `窥天境` ra "Khuy Thiên Kính" (phải là "Cảnh").
    - `腿` được dịch ba kiểu Thối/Thoái/Cước trong 9 mục.
    - Có 7 cặp trùng mục (ví dụ 张三丰/张三疯, 死神渡/度).
    - `神医` bị xếp vào names dù là danh xưng.
    - `剑神` (danh hiệu) và `剑晨` (tên người) đều ra "Kiếm Thần".
12. **SAI_NGHIA, lẻ, vừa.** 0276 dòng 83: `暗金光芒` → "tia sáng hoàng kim mờ ảo"; nên là "vàng sẫm".
    - Cùng chương, dòng 37: `舍生忘死` → "quên mình vì đại nghĩa", tức thêm ý "đại nghĩa".
13. **CALQUE, nhẹ.** `一份子` → "phần tử" (0310, 0551), mang sắc thái chính trị.
    - `便宜老爹` → "tiện nghi lão cha" (0014).
14. **LAP_TU, nhẹ.** Các lỗi lặp từ và dùng sai từ ở chương 0550:
    - "bị ma niệm phản phệ thôn phệ"
    - "dùng hết toàn lực toàn thân"
    - "tận ý thôi động" (`刻意` nghĩa là cố ý)
15. **DINH_DANG, hệ thống, nhẹ.** Ngoặc kép không thống nhất: 130 file chỉ dùng `"`, 421 file dùng `“”`, 32 file trộn cả hai. Ngoài ra "......" (sáu chấm) xuất hiện 1424 lần trong 286 file.

## Gợi ý

- **Luật cứng cho setting ancient:** cấm "đem" khi dịch 将/把, cấm "phiến" khi dịch 片, cấm "bản thân hắn/nàng". Đưa ba mẫu này vào checkRules để `check` đếm và bắt sửa.
- **Sửa addressing:**
  - 雄霸→雄天下: "ta–con".
  - 雄天下→雄霸: "con–cha", hoặc "ta–người" khi trêu "lão già".
  - Tạo cast.json chốt đại từ ngôi 3: Hùng Bá là "lão"; chốt thêm cho Tuyết Ám Thiên và Đệ Nhất Tà Hoàng.
- **Tổng võ có meme:** cho phép lời kể dí dỏm hiện đại khi tác giả cài meme, kèm bảng dịch mẫu:
  - 神他妈X → "X cái quái gì"
  - 金手指 → "bàn tay vàng"
  - 小辣椒 → "cô nàng ớt cay"
- **Sửa glossary:**
  - 综武 → "tổng võ"; 窥天境 → "Khuy Thiên Cảnh".
  - Chốt 腿 = "Cước" cho cả 9 mục.
  - Gộp 7 cặp trùng mục; chuyển 神医 sang addressing.
  - Phân biệt 剑神 (danh hiệu) với 剑晨 (tên người).
- **Bỏ ép `嗯` → "Ân":** dùng Ừ, Hửm, Ừm theo ngữ cảnh.
- **Hậu xử lý trong app:** chuẩn hoá ngoặc kép về `“”` và "......" về "…".
- **Nhắc trong prompt:**
  - `自己` trong độc thoại nội tâm trỏ về người đang nghĩ.
  - `好一个X` dịch là "Hay cho X".

## Làm tốt

- Đủ 551/551 chương, mọi chương khớp 100% số đoạn với bản gốc, không sót chữ Hán.
- Tên và chiêu thức nhất quán qua hơn 500 chương; ví dụ Phong Thần Cước khớp 32/32 lần.
- Cảnh đánh dứt khoát, có lực. Chương 276 (Tạ Thất Đao, bảy đao) rất hay.
- `老登` ra "lão già" đúng signature phrase (111/141 lần). Lời thoại của Hùng Thiên Hạ ngông và có cá tính.
- Nhiều thành ngữ và câu lóng chuyển tự nhiên: "Trời đất ơi" (`我的乖乖`), "ba lần đến mời" (`三顾茅庐`), "Kính rượu cái con khỉ mốc" (`神他妈敬酒`).
