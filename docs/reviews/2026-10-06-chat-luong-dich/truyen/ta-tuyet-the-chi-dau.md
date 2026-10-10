# Review: Ta Tuyệt Thế Chị Dâu (ta-tuyet-the-chi-dau)

- Setting: `modern`. Thực tế: đô thị hiện đại, kể ngôi 1 (thỉnh thoảng truyện gốc nhảy sang góc nhìn Như Vân/Cao Kiện).
- Đã dịch 58 chương. Đọc trọn: 0001, 0030, 0057, 0058 (0057 và 0058 có mtime mới nhất). Đã grep toàn bộ out/.

## Điểm

| Tiêu chí | Điểm | Lý do |
|---|---|---|
| Xưng hô | 6.0 | Người kể giữ "ta" chuẩn, nhân vật phụ ổn. Nhưng Như Vân → Trần Phi nhảy qua lại "chị–em" ↔ "tôi–cậu" giữa các chương, có câu trộn cả hai. Addressing còn ghi "tao–mày". |
| Giọng văn | 8.0 | Đô thị bình dân, gọn, đúng style. Trừ điểm vì "chúng ta" trong lời kể. |
| Dễ hiểu | 8.0 | Trôi chảy, có vài lỗi nghĩa lẻ. |
| Khó khăn | 8.0 | Meme/lóng dịch tốt. Còn 呵 → "Hừ", "chiếm tiện nghi". |
| Thuật ngữ | 8.5 | Tên nhất quán, 0 chữ Hán, 0 "Ân"/"ngươi". Glossary có vài mục thừa. |

## Lỗi (nặng → nhẹ)

1. **XUNG_HO_TROI (cao, hệ thống).** Như Vân → Trần Phi dùng "chị–em" ở ch.13–14, 20–24, 39, 42, 57, nhưng "tôi–cậu" ở ch.15, 28–31, 36, 53, 58 (~25 chương). Ví dụ ch.58:35 `陈飞，你帮我拍一下` → "Trần Phi, cậu chụp giúp tôi một kiểu." Trong khi ngay ch.57 cô vừa "Chị hỏi em này". Đề xuất: "em chụp giúp chị một kiểu" (hoặc ghi mốc rõ là trước mặt Cao Kiện thì giữ "tôi–cậu").
2. **XUNG_HO_TROI trong cùng một câu (cao).** ch.54:111 → "…chị dâu có tiền đây, chị muốn góp vốn vào công ty của cậu." Tự xưng "chị" mà gọi "cậu". Kiểu nói nũng nịu "有钱钱" cũng bị mất.
3. **CAST_MAU_THUAN (cao, hệ thống).** story.json addressing có `许如云→陈飞: tao–mày`, `陈飞→许如云: tôi–cô`, chốt từ cảnh túm tóc ở ch.1. Hai mục này mâu thuẫn với toneRules ("tôi–cậu/anh", "chị dâu/chị"). Truyện không có cast.json, cũng không có mốc nào. Đây là gốc của lỗi 1.
4. **XUNG_HO_NGUOI_KE (vừa, hệ thống, 86 lần/43 file).** "Chúng ta" trong lời kể, ví dụ ch.58:13 `我们是坐的高健的车子` → "Chúng ta đi xe của Cao Kiện." Đề xuất "Bọn ta…"; "hai chúng ta" → "hai đứa ta/cả hai".
5. **DINH_DANG (vừa, hệ thống).** Ngoặc kép cong ở 28 file, thẳng ở 30 file (0003–0030, 0049, 0057). Dấu ba chấm có lúc "......" (108 lần/46 file), có lúc "..." (58 lần/29 file).
6. **SAI_NGHIA (vừa).** ch.57:83 `人才市场` → "thị trường việc làm". Đề xuất "trung tâm giới thiệu việc làm".
7. **DAI_TU_MO_HO (vừa).** ch.58:71 `你哥跟你嫂子正暧昧着呢` → "Anh với chị dâu em đang tình tứ…" Thiếu "em" nên không rõ "anh" là ai. Đề xuất "Anh em với chị dâu em…".
8. **CALQUE (vừa, 4 lần/2 file).** "chiếm tiện nghi", ví dụ ch.19:109 "Văn Lệ Quyên muốn chiếm tiện nghi của ta?" Đề xuất "muốn sàm sỡ ta à?".
9. **HU_HA (vừa, 3/26 dòng có 呵).** ch.45:97 `“呵。”` → "Hừ."; ch.47 `呵呵……` → "Hừ hừ......". 呵 là cười nhạt, nên dịch "Ha/Hà hà".
10. **TIEU_DE (thấp, 10 file 0003–0012).** "Chương 7 Tính kế?" thiếu dấu hai chấm.
11. **CALQUE (thấp).** ch.57:69 "hiện trường" (现场) → "bãi".
12. **VAN_KHO (thấp).** ch.30:3 "chuốc say chính mình và ta thì phải" → "chuốc say cả cô lẫn ta".
13. **SAI_NGHIA (thấp).** ch.30:113 "Đêm nay" (这一夜, kể quá khứ) → "Đêm ấy". Cùng chương có "phản tỉnh" → nên là "tự vấn".
14. **GLOSSARY_HAN_VIET_NANG (thấp).** 腾辉绿植 → "Đằng Huy Lục Thực" → nên là "Cây cảnh Đằng Huy". Glossary còn có các mục thừa: 柳下惠, 诸葛亮, 大清, 四大金刚.

Các lỗi đã biết **không gặp**: "Ân", "ngươi", sót chữ Hán, "phiến", "Theo việc/sự", tên chương viết hoa mọi chữ. "bản thân" có 49 lần/29 file, phần lớn tự nhiên. "người phụ nữ" có 26 lần/14 file, chủ yếu là danh từ hợp lý.

## Gợi ý

- Lập cast.json có mốc xưng hô cho cặp chính:
  - Như Vân → Trần Phi: "tôi–cậu" ở ch.1–12 và khi có Cao Kiện; "chị–em" khi riêng tư, từ ch.13.
  - Trần Phi → Như Vân: "em–chị (dâu)"; chỉ "tôi–cô" khi cãi nhau.
- Auto-glossary addressing:
  - Không chốt xưng hô từ cảnh chửi nhau ngoại lệ.
  - Khi chương mới khác đi thì cập nhật hoặc thêm mốc.
  - Đối chiếu với toneRules trước khi lưu.
- Prompt ngôi 1: 我们/咱们 trong lời kể → "bọn ta/hai đứa ta/cả hai", cấm "chúng ta" ngoài ngoặc kép.
- Check rule: một câu thoại không được vừa "chị" vừa "cậu" (hoặc vừa "tôi" vừa "em").
- Hậu xử lý: chuẩn hoá ngoặc kép, dấu ba chấm và tiêu đề "Chương N: ...".
- Bảng calque cho truyện hiện đại:
  - 占便宜 → lợi dụng/sàm sỡ
  - 呵 → Ha/cười nhạt
  - 现场 → bãi/công trường
  - 人才市场 → trung tâm giới thiệu việc làm
- Lọc glossary: bỏ nhân vật lịch sử và triều đại; tên công ty thì dịch phần mô tả ngành.

## Làm tốt

- Người kể "ta" tuyệt đối: 0 lần "tôi"/"Trần Phi" trong lời kể.
- Lóng sống:
  - "em vái cả nón luôn" (我简直靠了)
  - "phi công trẻ" (小奶狗)
  - "vắt cổ chày ra nước" (铁公鸡)
  - "Vãi thật" (我擦)
- Nhân vật phụ xưng hô ổn: "em–anh Phi", Lan Hương "chị–em". Lỗi gõ 表格 (表哥) trong bản gốc cũng được dịch đúng thành "anh họ".
- Cảnh tình cảm ch.30 giữ được cảm xúc; giọng đô thị bình dân đúng style.
