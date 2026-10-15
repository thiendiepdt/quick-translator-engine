# Kiểm tra nội dung nhạy cảm về Việt Nam — thiết kế

Ngày: 2026-10-15. Áp cho qt-ai-core + qt-ai-gui.

## Mục tiêu

Hệ thống truyện cấm đăng truyện xúc phạm Việt Nam. App cần:

1. **Báo sớm** truyện/chương có dấu hiệu, cả ở raw (trước khi tốn quota dịch) lẫn bản dịch.
2. **Xác minh**: mỗi chỗ khớp hiện kèm ngữ cảnh; AI sơ loại theo yêu cầu; người dùng chốt "Không sao" / "Cần xử lý".
3. **Thay thế** cụm từ liên quan VN trong bản dịch (một, nhiều hoặc tất cả chương) bằng cụm khác để đăng được.

Chỉ cảnh báo, không chặn dịch/chốt/export.

## Từ điển

Nằm trong qt-ai-core (hằng số), người dùng thêm mục hoặc tắt mục có sẵn trong Cài đặt (lưu ở config app).

Mỗi mục:

| Trường | Ý nghĩa |
| --- | --- |
| `group` | `ten` tên gọi VN & người Việt (kể cả miệt thị) · `cq` chủ quyền biển đảo · `ls` chiến tranh / lịch sử · `hd` địa danh & nhân vật VN hiện đại |
| `level` | `mietthi` (chửi rõ) · `canxem` |
| `strength` | `strong` · `weak` (chỉ báo khi cùng đoạn có từ đi kèm, xem dưới) |
| `zh` | các dạng chữ Hán trong raw |
| `vi` | các dạng tiếng Việt trong bản dịch |
| `exclude_zh` / `exclude_vi` | mẫu loại trừ xét trong cửa sổ quanh chỗ khớp |

Danh sách khởi đầu (đủ để test; bổ sung dần):

- **ten** — strong: 越南, 越南人, 越南籍 / Việt Nam, người Việt Nam; 越南语, 越语 / tiếng Việt; 大越 / Đại Việt (raw: chịu loại trừ `越…越`); 安南(国/人) / An Nam; 交趾, 交阯(人/国/蛮) / Giao Chỉ; 安南都护 / An Nam đô hộ. Miệt thị (level `mietthi`): 越南猴(子), 越猴, 猴国, 安南猴, 交趾蛮, 小越 / khỉ Việt (Nam); 越南新娘, 越南媳妇, 越南老婆, 越南妹 / cô dâu Việt. Weak: 交州 / Giao Châu; 南越(国) / Nam Việt; 越人, 京族 / người Việt, người Kinh.
- **cq** — strong: 西沙 / Tây Sa, Hoàng Sa; 南沙 / Nam Sa, Trường Sa; 中沙, 三沙 / Trung Sa, Tam Sa; 九段线, 十一段线 / đường chín đoạn, đường lưỡi bò; 南海诸岛, 南中国海 / biển Nam Trung Hoa; 永兴岛, 永暑礁, 美济礁, 赤瓜礁 / đảo Phú Lâm, đá Chữ Thập, đá Vành Khăn, Gạc Ma. Weak: 南海 / Nam Hải, Biển Đông; 北部湾, 东京湾 / Vịnh Bắc Bộ.
- **ls** — strong: 对越自卫反击战, 中越战争 / chiến tranh biên giới, phản kích tự vệ; 老山战役, 两山轮战, 法卡山; 征侧, 征贰, 二征 / Hai Bà Trưng, Trưng Trắc, Trưng Nhị; 平定交趾, 郡县安南; 日南郡, 九真郡, 交趾郡; 李常杰 / Lý Thường Kiệt; 黎利 / Lê Lợi; 胡志明 / Hồ Chí Minh; 越共 / Việt Cộng. Weak: 马援, 伏波将军 / Mã Viện, Phục Ba tướng quân; 白藤江 / Bạch Đằng; 越战 / chiến tranh Việt Nam.
- **hd** — strong: 西贡 / Sài Gòn; 胡志明市 / TP Hồ Chí Minh; 岘港 / Đà Nẵng; 芽庄 / Nha Trang; 下龙湾 / Hạ Long; 湄公河 / Mê Kông; 顺化 / Huế (chỉ khi raw có 顺化). Weak: 河内 / Hà Nội; 海防市 / Hải Phòng.

Loại trừ khởi đầu (từ thăm dò thư viện): `越…越` trong cùng vế câu (越战越勇, 越大越好, 他越战，…越快); 天南海北, 南海北; 平安南; 河内郡, 河内郡王, 各郡…河内; 莫西沙; Bách/Sơn/Mân/Âu/Ngô Việt, Việt vương, nước Việt (không kèm Nam), Câu Tiễn, 吴越, 百越, 山越, 闽越, 於越; 南海 + (派|剑|神尼|鳄神|老怪|郡|观音|普陀); Nam Hải (phái|kiếm|thần ni|ngạc thần|quận); Hà Nội (quận|vương) và "quận Hà Nội". Bỏ hẳn 头顿 (đụng 头顿时).

## Luật khớp

1. **Bản dịch là tín hiệu chính** ở chương đã dịch; raw dùng cho chương chưa dịch.
2. **Tiếng Việt khớp nguyên từ** (hai bên không dính chữ cái): "Hoàng Sa" không khớp "Hoàng Sam".
3. **Đối chiếu hai bên**: chỗ khớp ở bản dịch chỉ tính khi đoạn raw cùng nhãn có một dạng `zh` của chính mục đó. Loại "Hoàng Sa" dịch từ 黄沙 (cát vàng), "người Kinh thành", "Nam Việt" = 南粤, "Biển Đông" = East Blue. Đoạn không căn được (lệch số đoạn) thì bỏ luật này, chỉ dùng luật 2, 4, 5.
4. **Từ yếu** chỉ báo khi cùng đoạn (raw hoặc dịch) có một từ đi kèm: một mục strong bất kỳ, hoặc 主权 / 领土 / 领海 / 岛礁 / chủ quyền / lãnh thổ / lãnh hải / quần đảo.
5. **Loại trừ** xét trong cửa sổ ±12 ký tự quanh chỗ khớp.
6. Một đoạn có nhiều chỗ khớp cùng mục → một mục hiển thị, ghi số lần.
7. Các dạng chồng nhau (越南 / 越南人 / 越南猴子) khớp theo cụm dài nhất (Aho-Corasick `LeftmostLongest`); vì vậy mọi cụm chứa một dạng strong đều phải nằm ở mục strong — không để cụm dài ở mục weak nuốt mất chỗ khớp thật.

Thăm dò trên 66 truyện bằng script nháp: không luật nào ~2.800 chỗ; thêm luật 2–5 còn ~90 chỗ ở 18 truyện, phần còn lại sau khi thêm loại trừ ở trên dự kiến ~30 chỗ, trong đó có chỗ thật: "khỉ Việt Nam này" (Trường sinh bắt đầu từ Thiên Long ch.226), "phía nam đánh chiếm Lữ Tống và Giao Chỉ" (cùng truyện, ch.354), "命人领兵夷交趾…设置安南节度" và "交趾人" (Huyền giám tiên tộc), "đánh chiếm thủ đô Hà Nội của Việt Nam" (Thâu hương cao thủ), "đất cũ người Hán như Giao Chỉ" (Xuyên qua chư thiên).

## Chạy ở đâu

- **Quét raw**: sau init / quét chương mới / mở truyện — các chương chưa dịch. Màn chọn truyện hiện huy hiệu ⚠ kèm số chỗ chưa xét.
- **Khi chốt chương** (accept, cả agy lẫn API): quét bản dịch chương đó; có chỗ khớp chưa xét → thêm một cảnh báo vào `warnings` của chương (như advisory, không vào vòng soát). Danh sách chương hiện dấu.
- **Tab "Kiểm tra VN"** trên trang Truyện: danh sách đầy đủ, xác minh, thay thế.

## Hiệu năng

- Aho-Corasick (crate `aho-corasick`, đã có trong cây phụ thuộc qua `regex`) — một lượt tìm mọi dạng `zh` / `vi`; luật loại trừ, từ yếu, đối chiếu chỉ chạy tại chỗ khớp.
- Quét tăng dần: `vn-check.json` ghi dấu từng chương (kích thước + mtime của raw và out); quét lại chỉ đọc chương mới/đổi.
- Song song như `replace.rs` (`std::thread::scope`, ≤ 8 luồng), chạy trong `blocking` của Tauri, có sự kiện tiến độ cho lần quét đầu.
- Accept chỉ quét một chương.

Mục tiêu: lần đầu truyện 2.500 chương ≤ vài giây; quét lại không đổi gì ≤ 100 ms; cả thư viện 66 truyện lần đầu ≤ 10 giây.

## Lưu trữ — `vn-check.json` (thư mục truyện)

```json
{
  "version": 1,
  "lexiconHash": "…",                       // đổi từ điển → quét lại toàn bộ
  "chapters": { "<id>": { "raw": [size, mtime], "out": [size, mtime] } },
  "hits": [{
    "id": "<chương>:<đoạn>:<mục>",
    "chapter": "…", "paragraph": 12, "entry": "giao-chi", "group": "ten", "level": "canxem",
    "side": "out" | "raw", "matched": "Giao Chỉ", "count": 1,
    "fingerprint": "<hash đoạn raw + đoạn dịch>",
    "status": "pending" | "ok" | "action",
    "ai": { "label": "xucpham|nhaycam|trungtinh|khonglienquan", "reason": "…", "model": "…", "at": 0 }
  }]
}
```

Quét lại: hit mới cùng `id` và cùng `fingerprint` giữ `status` + `ai`; đoạn đổi chữ → về `pending`, xoá `ai`. Chương đổi từ chưa dịch sang đã dịch → hit raw thay bằng hit dịch (nếu còn).

## Xác minh (tab Kiểm tra VN)

- Lọc theo nhóm, mức, trạng thái, nhãn AI; nhóm theo mục từ điển hoặc theo chương.
- Mỗi chỗ: đoạn raw + đoạn dịch, tô chỗ khớp; nút "Xem thêm" mở một đoạn trước/sau; nút **Không sao** / **Cần xử lý**; liên kết mở chương ở trang đọc.
- **AI xét** (chọn một/nhiều/tất cả đang lọc): lô ≤ 20 chỗ mỗi lời gọi, mỗi chỗ gửi đoạn trước + đoạn chứa + đoạn sau (raw và dịch). Trả JSON `[{id, label, reason}]`. Động cơ: API key nếu có (`TextModel::complete_json`), không thì `agy -p --output-format json --json-schema`, model theo ô chọn model. JSON hỏng / thiếu id → giữ `pending`, báo số lỗi.
- Huy hiệu và cảnh báo chương chỉ đếm `pending` + `action`.

## Thay thế

- **Bảng thay** trong Cài đặt (config app): `{from, to, setting: ancient|modern|any}`. Mặc định — cổ đại: Giao Chỉ, An Nam, Giao Châu, Đại Việt → Chiêm Thành; hiện đại: Việt Nam → Thái Lan, người Việt → người Thái, tiếng Việt → tiếng Thái, Hà Nội → Băng Cốc, Sài Gòn / TP Hồ Chí Minh → Chiang Mai. Nhóm `cq`, `ls` không có mặc định.
- Từ tab Kiểm tra VN: chọn chỗ (hoặc lọc), "Thay thế…" → hộp thoại điền sẵn `from` (chữ khớp) + `to` (theo bảng, sửa được), chọn phạm vi: các chỗ đang chọn · một/nhiều chương (khoảng hoặc tick) · tất cả chương có chỗ khớp. Xem trước số chỗ/chương.
- Áp bằng engine `replace.rs` (`replace_in_story` với `excluded_chapters` / `excluded_hits` để giới hạn đúng phạm vi) → có hoàn tác lần thay gần nhất. Xong quét lại các chương đó.
- Chỉ thay `out/`; raw giữ nguyên; chỗ chỉ khớp ở raw không thay được (chưa dịch).

## Ngoài phạm vi lần này

- Ghi bảng thay vào glossary để chương dịch sau tự ra tên thay.
- Dùng AI dò ý xúc phạm không chứa từ khoá ("cái nước nhỏ phía nam").
- Chặn export.

## Kiểm thử

TDD, test viết trước:

- Loại trừ: 越战越勇, 他越战…越快, 天南海北, 河内郡王, 越王勾践 / 吴越, 头顿时, 莫西沙 không báo; 越南, 交趾人, 西沙 báo.
- Nguyên từ: "Hoàng Sam" không khớp "Hoàng Sa".
- Đối chiếu: bản dịch "Hoàng Sa" + raw 黄沙 → bỏ; + raw 西沙 → báo. "người Kinh thành" + raw 京城 → bỏ.
- Từ yếu: "Nam Hải" một mình không báo; cùng đoạn có "quần đảo Tây Sa" → báo.
- Quét tăng dần chỉ đọc chương đổi; giữ `status`/`ai` khi đoạn không đổi; reset khi đổi.
- Accept ghi cảnh báo cho chương có chỗ chưa xét, không ghi khi mọi chỗ đã "Không sao".
- AI: dựng lô, đọc JSON tốt / hỏng / thiếu id.
- Thay: đúng chỗ đã chọn, đúng khoảng chương, hoàn tác.
- GUI: lọc, chọn, hộp thoại thay, huy hiệu.

Kiểm chứng thật: quét cả thư viện (đo thời gian, đếm chỗ, đọc tay mẫu báo nhầm); AI xét các chỗ thật kể trên trên Trường sinh bắt đầu từ Thiên Long, Huyền giám tiên tộc, Thâu hương cao thủ.
