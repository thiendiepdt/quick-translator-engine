# Hệ thống xưng hô của qt-ai — thiết kế

## Vấn đề

Ba nhóm report từ user, cùng một gốc: app chỉ lưu *kết quả* (`甲→乙: X–Y`), không lưu *sự thật* về
nhân vật, và kết quả đó bị khoá ở lần đầu.

- Nữ bị gọi "thầy": `贺老师` → "thầy Hạ" dù 贺静昭 là nữ (Đỉnh Lưu ch.132); glossary học luôn
  `宋时安→贺静昭: tôi–thầy Hạ` trong khi `莫衡→贺静昭: tôi–cô`.
- Xưng hô không phát triển: `曹雅旋→许浪: tôi–cậu` học ở ch.2 (người lạ), ch.692 vẫn y nguyên; user
  báo "trai gái chưa quen tôi–cô, thành vợ chồng vẫn tôi–cô".
- Danh xưng dịch thô: `学弟，你不要吓学姐好不好` → "Đàn em, cậu đừng dọa đàn chị"; độc thoại nội tâm
  lẫn ta/mình/tôi.
- Dữ liệu bẩn: gửi 167/275 cặp mỗi chương (lọc "một bên có mặt"); 82/275 cặp có key không phải
  nhân vật (`保安`, `助理`); 28/10.051 cặp viết ngược (`mày–tao`).

## Ràng buộc

- `build_system_prompt`, `filter_glossary_for_source`, `sanitize_extracted`, `append_auto_glossary` là
  port từng byte của qt-web (golden do `gen-golden.ts` sinh). **Không sửa** các hàm này; hệ thống xưng
  hô là một lớp mới trong qt-ai-core bọc quanh.
- Truyện cũ không vỡ; bản app cũ mở truyện mới không làm mất dữ liệu → dữ liệu mới nằm ở **file riêng
  `cast.json`** cạnh `story.json`. `story.json` không đổi schema; `glossary.addressing` giữ nguyên nghĩa
  (cặp học lần đầu).
- User tự sửa base prompt / dùng `customPrompt` vẫn phải nhận luật mới → luật nằm trong mục harness tự
  chèn, không nằm trong base.
- Truyện chưa có `cast.json`: không có mục nhân vật, check giới bỏ qua.

## Dữ liệu: `cast.json`

```json
{
  "version": 1,
  "characters": {
    "贺静昭": { "gender": "female", "source": "auto", "chapter": "0132_x", "disputed": [] }
  },
  "addressing": {
    "宋时安→贺静昭": {
      "pinned": false,
      "changes": [{ "from": "0140_x", "target": "em–chị", "note": "thân hơn", "source": "auto" }]
    }
  }
}
```

- `characters`: key là tên Hán như trong `glossary.names`. `gender`: `male` | `female`. `source`:
  `auto` | `user`. `disputed`: các chương khai giới ngược lại (không tự đổi, GUI cảnh báo).
- `addressing[key].changes`: các lần đổi sau giá trị gốc ở `story.json`, sắp theo thứ tự chương.
  Cặp hiệu lực tại chương `c` = thay đổi cuối cùng có `from` ≤ `c`; không có thì lấy `story.json`.
  Đổi cặp xét theo trạng thái quan hệ nên chương khai đổi đã dùng cặp mới cho cả chương.
- `pinned`: user đã sửa tay → harness không tự đổi nữa.
- Đọc lenient: file thiếu/hỏng → rỗng; ghi atomic.

## Giai đoạn 1 — giới tính

- **Khai**: entry `names` trong khối glossary được kèm `"gender": "nam" | "nữ"`. Nhân vật đã có trong
  từ điển mà chưa rõ giới được khai lại `{source, gender}`. Harness chỉ nhận khi tên có trong raw
  chương và raw có dấu hiệu đúng giới (她/女/姐… hoặc 他/男/哥…); đã có giới thì không đè, khai ngược
  ghi vào `disputed`.
- **Quét bù** (`cast_scan`): với tên chưa có giới, gom tối đa 4 câu raw chứa tên, gửi theo lô 40 tên
  qua `complete_json` → `{"genders": {...}}`. Chỉ điền chỗ còn trống.
- **Prompt**: mục `# Xưng hô` chèn trước suffix, liệt kê nhân vật có mặt trong chương đã rõ giới
  (tên đầy đủ, tên bỏ họ, hoặc `họ + chức danh` như 贺老师).
- **Check** (`gender_violations`): đoạn raw có `họ + chức danh trung tính` (老师, 老板, 总, 导, 医生,
  教授, 主任, 队长, 经理, 教练, 前辈, 同学…), họ đó chỉ ứng với **một** nhân vật (ưu tiên nhân vật có
  mặt trong chương), nhân vật đã rõ giới, bản dịch đoạn đó có `từ trái giới + họ Việt` ("thầy Hạ",
  "anh Hạ", "ông Hạ" cho nữ; "cô/chị/bà X" cho nam) → vi phạm mềm, đưa vào soát.
- **Chặn học sai**: cặp xưng hô mới có từ trái giới với bảng → bỏ. `conflicting_addressing` liệt kê
  cặp cũ trái giới để user dọn.

## Giai đoạn 2 — siết glossary xưng hô

- Chỉ gửi cặp khi **cả hai** bên có mặt trong chương.
- Chỉ học cặp mới khi cả hai bên là tên trong `glossary.names` (kể cả tên mới cùng lượt).
- Cặp viết ngược (`Y` là đại từ ngôi một: tao, tôi, tớ, ta, mình…) tự đảo khi học và khi đưa vào prompt.
- Cặp cũ trong `story.json` không bị xoá.

## Giai đoạn 3 — luật xưng hô (trong mục `# Xưng hô`)

1. Từ gọi người không mang giới trong tiếng Trung chọn từ Việt theo giới ở bảng nhân vật.
2. Danh xưng làm đại từ: nhân vật tự xưng/gọi bằng 学姐/学弟/姐/哥/老师/妈… thì dùng chính danh xưng
   Việt làm đại từ ("Em đừng dọa chị"), không "đàn em/đàn chị/học tỷ" làm lời gọi.
3. Độc thoại nội tâm chốt một ngôi mỗi nhân vật: `mình` (hiện đại), `ta` (cổ đại); hỗn hợp theo cảnh.
4. (Hiện đại/hỗn hợp) 老师 gọi nghệ sĩ, đồng nghiệp: vẫn `thầy/cô + họ` theo giới.

Check: raw có 学姐/学弟/学长/学妹 trong thoại mà thoại bản dịch có "đàn chị/đàn em/đàn anh" → vi phạm mềm.

## Giai đoạn 4 — xưng hô theo mốc chương

Chạy thử thật cho thấy model coi cặp trong danh sách là lệnh phải giữ: luật chung "đổi khi quan hệ đổi"
không có tác dụng (đang bàn chuyện cưới vẫn `tôi–cô`). Vì vậy **harness tự phát hiện**, không trông vào
model tự nhận ra:

- **Phát hiện** (`pairs_to_review`): chương có dấu hiệu thân mật trong raw (结婚, 求婚, 老公, 女朋友, 接吻,
  夫君…), một chiều của đôi còn kiểu người lạ (tự xưng `tôi`; cổ đại `ta–ngươi`…), không chiều nào ghim.
- **Prompt**: đôi cần xét bị rút khỏi danh sách "Giữ đúng các cặp này" và nêu đích danh ở khối "Cặp cần
  xét lại trong chương này" kèm dấu hiệu; đã là người yêu/vợ chồng thì bắt buộc dùng cặp mới và khai lại
  kèm `note`, không thì giữ và không khai.
- **Nhận đổi** (`accept`): hai bên có trong raw, có `note`, target khác cặp hiệu lực, hợp giới, cặp
  không ghim, không đảo về giá trị ngay trước đó. Ghi `changes` với `from` = chương đang chốt; dịch lại
  cùng chương thì thay mục cùng `from`.
- **Ghim**: `save_story` thấy user đổi/thêm giá trị `glossary.addressing` → ghim cặp đó và xoá
  `changes` của nó. GUI có danh sách thay đổi (hoàn tác, ghim/bỏ ghim) và form "đặt mốc từ chương N".

## Bề mặt

- **Core**: `cast.rs` (dữ liệu, giới của từ, cặp hiệu lực, mục prompt, check), `cast_scan.rs`,
  `prompt::build_chapter_prompt` (bọc `build_system_prompt`), nối vào `next`, `check`, `accept`,
  `api_session`, chỉ dẫn agent trong `next.rs`.
- **Tauri**: `load_cast`, `save_cast`, `scan_cast`, `clean_cast_addressing`; `save_story` ghim cặp sửa tay.
- **Frontend**: panel "Nhân vật & xưng hô" trên trang Truyện.

## Không làm

- Không check máy việc tuân thủ cặp xưng hô (không xác định được ai nói với ai), không thêm lượt model
  soát xưng hô.
- Không tự đổi cặp khi raw không có tín hiệu; không sửa chương đã dịch.
- Không đụng qt-web, qt-ai-cli (TypeScript), base prompt.
- Quét bù cần động cơ API key; động cơ Antigravity nhận giới dần qua các chương mới dịch và sửa tay.

## Kiểm chứng

Unit test + test vòng dịch với model giả cho từng luật. Chạy thử thật (API, gemini-3.8-flash) trên bản sao:

- Giới tính (Đỉnh Lưu ch.132, 143): code cũ cùng model vẫn "thầy Hạ"; code mới "cô Hạ". Quét bù điền
  129/166 và 339/777 tên, các nhân vật kiểm tay đều đúng.
- Danh xưng (Ký Túc Xá ch.2–8): "Đàn em, cậu đừng dọa đàn chị" → "Em đừng dọa chị"; không còn "đàn em/đàn
  chị" trong thoại.
- Mốc xưng hô (Đi Làm Mò Cá ch.893): `tôi–cô` → `anh–em` cả hai chiều, ghi mốc kèm lý do.
- Truyện có tone rule riêng ghi rõ cặp (Tuyệt Đối Rung Động: "tôi – cậu") thì model giữ theo hồ sơ truyện.
