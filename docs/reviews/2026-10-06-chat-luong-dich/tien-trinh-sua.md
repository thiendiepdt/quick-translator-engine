# Tiến trình sửa theo đợt thẩm định 06/10/2026

Sổ ghi từng mục sửa trong lộ trình của [README.md](README.md) và [tham-dinh-ban-dich.html](tham-dinh-ban-dich.html). Mỗi mục làm theo một vòng: đề xuất cách làm → chủ app chốt → làm → kiểm → ghi vào đây.

Quy ước chung:

- Nguồn thật của prompt là `apps/qt-web/src/lib/ai-translation-prompt.ts`, của luật check là `apps/qt-web/src/lib/ai-translation.ts`. Sau khi sửa phải chạy `npm run golden` trong `apps/qt-ai-cli` để sinh lại `crates/qt-ai-core/prompts/prompts.json` và fixture; bản chép luật trong `crates/qt-ai-core/src/check.rs` sửa tay và được golden test so khớp.
- Đổi base cổ đại/Hán-Việt có chủ ý thì cập nhật `LEGACY_BASE_PROMPT_FNV1A64` (chỉ dùng trong test chốt byte).
- Không sửa hàng loạt bản dịch cũ trong lúc sửa prompt/check; việc đó để mục riêng (P2) khi đã chốt.

## Trạng thái

| # | Mục | Ưu tiên | Trạng thái |
|---|---|---|---|
| 1 | Bỏ luật "Ân", bảng thán từ & tiếng cười, tầng luật cảnh báo | P0 | Xong, chưa commit |
| 2 | 自己 → "bản thân hắn" | P0 | Xong, chưa commit |
| 3 | Thu hẹp luật "tôi → ta" (`check.rs:132`) | P0 | Xong, chưa commit |
| 4 | Vai vế trong thoại cổ đại (tiểu nhân, nô tỳ, thuộc hạ, đệ tử, con) | P0 | Xong, chưa commit |
| 5 | Sửa các dòng prompt dạy sai (舔狗, kiểm trắc, 老X; 佟 giữ nguyên) | P0 | Xong, chưa commit |
| 6 | Mục "Số và đơn vị" (亿, 百亿, 一倍, 斤) | P0 | Xong, chưa commit |
| 7 | Sửa setting sai của 4 truyện | P0 | Xong (dữ liệu đã sửa; code chưa commit) |
| 8 | Glossary tự sinh được khai nhóm `common` | P0 | Xong, chưa commit |
| 9 | Tên lệch dấu so với glossary: code tự sửa (phần lời dặn soát gộp vào mục 12) | P0 | Xong, chưa commit |
| 10 | Hash raw, phát hiện raw đổi sau khi dịch | P0 | Bỏ — chủ app tự kiểm soát tay |
| 11 | Xưng hô: in chương chốt, đổi lời dặn, mở rộng dấu hiệu đổi quan hệ | P0 | Xong, chưa commit |
| 12 | Bắt tên viết gần đúng glossary (soát bắt buộc) | P0 | Xong, chưa commit |
| 13 | Bảng âm Hán-Việt kiểm glossary | P0 | Bỏ — để AI tự dịch tên; giữ danh sách rà tay |
| 14 | Chuẩn hoá ngoặc và gạch dài bằng code; giữ `『』`; sửa bản dịch cũ | P1 | Xong (bản dịch cũ đã sửa; code chưa commit) |
| 15 | Tiêu đề: `Chương N:` và bỏ lời xin phiếu bằng code; viết hoa kiểu câu, xưng hô theo thoại qua prompt | P1 | Xong, chưa commit |
| 16 | Meme, lóng mạng, chơi chữ: mục prompt mới + bảng lóng, ngoại lệ luật không giải thích | P1 | Xong, chưa commit |
| 17 | Calque: 7 dòng bảng convert, cảnh báo "chiếm tiện nghi", "tổng không thể" | P1 | Xong, chưa commit |
| 18 | Bẫy đồng âm theo cặp đoạn raw/dịch: 8 cặp bắt soát, 呵 → Hừ cảnh báo | P1 | Xong, chưa commit |
| 19 | Đại từ lời kể theo nhân vật, giới "biến thân", 祂 → Ngài, 你 phiếm chỉ | P1 | Xong (dữ liệu 5 truyện đã sửa; code chưa commit) |
| 20 | Setting theo arc | P1 | Bỏ — hỗn hợp + mục 7 đủ dùng |
| 21 | Glossary: chặn danh xưng làm tên riêng, bỏ `《》` khỏi khoá, nhóm lạ bỏ; xoá 426 mục danh xưng cũ | P1 | Xong (dữ liệu đã sửa; code chưa commit) |
| 22 | Theo dõi trôi xưng hô lúc accept, chạy bù timeline cast | P1 | Bỏ — mục 11 đã đủ, không chạy bù |
| 23 | Luật hiện đại bỏ qua "con ngươi", "cô nàng", "nàng thơ/dâu/tiên"; `style.avoid` không thành check | P1 | Xong, chưa commit |
| 24 | Chương lời tác giả: nhận diện, chặn glossary, ngưỡng trên độ dài, câu prompt | P1 | Bỏ — quá ít ca |
| P2-1 | Soát nghĩa có chọn lọc | P2 | Bỏ — tốn token/quota, dấu hiệu bắt rộng |
| 25 | Sửa "Ân" trong bản dịch cũ | P2 | Xong (dữ liệu, 3.455 dòng) |
| 26 | Rà ngược chương cũ theo mục glossary; đánh dấu cặp xưng hô đã có mốc | P2 | Xong |
| 27 | 招揽 → "chiêu lãm": dòng bảng convert, bắt soát, sửa 2 chỗ cũ | ngoài lộ trình | Xong, chưa commit |
| 28 | 人族 → "người tộc", 空前绝后 → "không tiền khoáng hậu": prompt, check, sửa 6 chỗ cũ | ngoài lộ trình | Xong |
| 29 | Trang Tìm & thay trong bản dịch đã chốt (Ctrl+Shift+F), hoàn tác lần thay gần nhất | ngoài lộ trình | Xong |
| 30 | Hiện đại / hỗn hợp → tên riêng hỗn hợp (dựng truyện, GUI, 4 truyện cũ) | ngoài lộ trình | Xong |
| 31 | Tham khảo HauTu.txt của QT: thêm chức danh nhận ra nhân vật, chức danh hành chính hiện đại trong prompt | ngoài lộ trình | Xong |
| 32 | Lỗi: kết quả AI điền hồ sơ bị Áp dụng vào truyện khác sau khi chuyển truyện | ngoài lộ trình | Xong, chưa commit |

---

## 1. Bỏ luật "Ân", bảng thán từ & tiếng cười, tầng luật cảnh báo

**Vấn đề.** 嗯 → "Ân" ở 3.383/6.100 dòng truyện cổ đại, 0 dòng truyện hiện đại. Gốc: luật 11 của prompt cổ đại bắt dùng "Ân" và nằm trong khối "không được ghi đè"; check cổ đại `Hừm/Ừm → Ân` còn ép model quay lại "Ân" khi nó tự viết đúng.

**Quyết định (chủ app chốt 10/10/2026).**

- 嗯 đáp nhẹ → Ừ / Ừm / Được; kẻ dưới đáp bề trên → **Dạ** (cảnh hiện đại dùng được cả Vâng). Nhận lệnh 是/遵命 → **Rõ! / Dạ**, cảnh cổ thêm **Tuân lệnh**.
- 嗯？ → **Hả?** (không dùng Hửm).
- 嘿嘿 → **hì hì / hề hề**, bỏ "hắc hắc".
- 咦?/啊? giật mình trong cảnh cổ vẫn là `A?` / `Hả?`, vẫn cấm "Ơ?".
- Luật bắt "Ân" có thể báo nhầm với tên riêng → chấp nhận, nhưng **chỉ cảnh báo**.
- Tạo tầng **luật cảnh báo**: không đẩy chương vào vòng soát, accept ghi vào `warnings` để người dùng xem lại.

**Thay đổi.**

- Prompt: luật thán từ cổ đại và hỗn hợp viết lại, trỏ tới bảng mới; mục 5 thêm bảng `Thán từ & tiếng cười` dùng chung ba bối cảnh (嗯, 嗯……, 嗯？, 嗯~, 了一声, 是/遵命, 呃, 呵, 哼, 哈哈, 嘿嘿, 嘻嘻, 桀桀) và luật "câu dẫn có 笑 thì không được là Hừ".
- Check: xoá luật cổ đại `Hừm/Ừm → Ân`.
- Tầng cảnh báo:
  - TS: `ADVISORY_CHECK_RULES`, `defaultAdvisoryRules`, `checkAdvisoryViolations` (qt-web chưa dùng, giữ để golden chép sang Rust).
  - Rust: `check::ADVISORY_RULES`, `advisory_violations`, `advisory_rules_as_check_rules`.
  - `CheckResult.advisories`: không ảnh hưởng `pass`, không vào lời dặn soát; `run_accept` gộp vào `warnings`. File `work/<id>.check.json` có thêm `advisoryCount`.
  - Hai luật đầu tiên: "Ân" đứng đầu câu/đầu thoại trước dấu câu; "ân một tiếng".
- Golden xuất thêm `advisoryRules` và `advisoryCases`.

**Kiểm.** Golden khớp; test golden mới `check_canh_bao_khop_web`; test lệnh mới `rule_chi_canh_bao_khong_lam_truot_check_khong_tang_vong_nhung_accept_ghi_warnings`; qt-web thêm test luật cảnh báo, sửa 3 test khoá luật cũ.

**Chưa làm.** Sửa 3.383 dòng "Ân" trong bản dịch cũ (để mục sửa hàng loạt).

## 2. 自己 → "bản thân hắn"

**Vấn đề.** Khoảng 6.300 lần "bản thân + đại từ" ở 31 truyện. Gốc: luật "Lời kể gián tiếp" đưa `bản thân hắn` (base cổ đại) và `bản thân cô` (base hiện đại) làm ví dụ cho 自己.

**Quyết định (chủ app chốt 10/10/2026).** Dịch 自己 bằng đại từ ngôi ba (cổ: hắn/nàng, hiện đại: hắn/cô) hoặc lược; chỉ dùng "chính hắn / tự mình" khi raw nhấn đối lập. Không đưa "mình" vào truyện cổ (va với quy ước người kể "ta" và luật `tôi/mình → ta`). Hai luật mới chỉ cảnh báo.

**Thay đổi.**

- Prompt: luật "Lời kể gián tiếp" ở base cổ đại và hiện đại (base hỗn hợp dùng lại) bỏ ví dụ "bản thân hắn/cô", thêm: 自己 là đại từ hồi chỉ; cấm "bản thân" làm chủ ngữ, sau giới từ, ghép đại từ; kèm ví dụ `奈何自己这情况特殊`.
- Luật cảnh báo: "bản thân + hắn/nàng/cô/y/lão/gã/nó"; "Bản thân" mở câu làm chủ ngữ (trừ "Bản thân là…" và khi đã bị luật trước bắt). "bản thân mình" không bị bắt.

**Kiểm.** Golden khớp; qt-web thêm test 5 câu mẫu (bắt 3, bỏ qua "bản thân mình", "Bản thân là").

**Chưa làm.** Sửa các chỗ đã dịch.

## 3. Thu hẹp luật "tôi → ta"

**Vấn đề.** Luật check hiện đại bắt mọi dòng ngoài ngoặc kép có "tôi" rồi ép sửa thành "ta", chạy trên mọi truyện hiện đại, trong khi 60/61 truyện kể ngôi ba. Đạn mạc, ý nghĩ trong 【】, tin nhắn, tiêu đề là câu thoại bị đổi sang "ta" (Trọng sinh thành mèo 396 dòng đạn mạc; Đa tử đa phúc 177 lần trong 【】; tiêu đề lệch xưng hô với thân chương).

**Quyết định (chủ app chốt 11/10/2026).** Luật người kể ngôi một xưng "ta" giữ nguyên trong prompt — người đọc không thích người kể xưng "tôi". Chỉ thôi ép "ta" ở những dòng không phải lời người kể; ở các dòng đó ưu tiên `tui` / `mình` / `tao` thay vì "tôi". Chưa thêm trường `genre.pov` vào story.json (mới 1/61 truyện kể ngôi một).

**Thay đổi.**

- Check: luật hiện đại "tôi → ta" chuyển sang tầng cảnh báo, regex bỏ qua dòng tiêu đề `Chương N`, dòng mở bằng `【` / `『` / `[`, dòng `Tên: …` (đạn mạc, chat). Thông điệp mới: "Người kể ngôi một xưng tôi → ta (truyện kể ngôi ba thì bỏ qua)".
- Mục `# Xưng hô` của prompt chương (`cast.rs`, chỉ qt-ai):
  - Độc thoại nội tâm áp cả cho ý nghĩ trong `【】`, `『』`.
  - Hiện đại: bình luận, đạn mạc, tin nhắn, bài đăng mạng ưu tiên tự xưng `tui` / `mình` / `tao`, gọi người khác `anh ấy` / `cô ấy` / `ảnh` / `bả` / tên; không `ta`, `hắn`, `ngươi`.
  - Mọi bối cảnh: tiêu đề chương là câu nói của nhân vật thì xưng hô như thoại của nhân vật đó.

**Kiểm.** Golden thêm dòng mẫu "tôi" (lời kể, thoại, 【】, `[Hệ thống: …]`, đạn mạc, tiêu đề): chỉ lời kể bị cảnh báo, Rust khớp TS. Test qt-web viết lại cho tầng cảnh báo; test `cast.rs` kiểm hai luật mới có/không theo bối cảnh. Luật cổ đại `check.rs:72` (thoại mở bằng tôi/mình → ta) để sang mục 4.

## 4. Vai vế trong thoại cổ đại

**Vấn đề.** 38 lỗi ở 35 truyện: kẻ dưới xưng "ta" và gọi bề trên "ngươi" (Thiếu niên trọng sinh: câu mở bằng Thiếu gia/Chủ nhân/Đại nhân mà tự xưng "ta" 64 lần/40 file; Rút đao: "Nhi tử của ta, ngươi đã về rồi"). Gốc: prompt cổ đại bắt `我` → `ta` cả trong thoại, bảng 你 → ngươi không có ngoại lệ vai vế, ghi chú bề trên ghi "你 nói với người già vẫn là ngươi"; check `mình/tôi → ta` làm vòng soát đổi "tôi/nô tỳ" thành "ta".

**Quyết định (chủ app chốt 11/10/2026).** Thêm mục vai vế; check tôi/mình giữ ở tầng chính nhưng đổi gợi ý; hai luật mới chỉ cảnh báo; không đưa Công tử/Tiểu thư/Phu nhân vào danh sách danh xưng bề trên (người ngang hàng cũng gọi vậy).

**Thay đổi.**

- Prompt cổ đại (base hỗn hợp dùng lại):
  - `我`: lời kể dùng `ta`; thoại mặc định `ta` trừ khi mục Vai vế quy định khác.
  - Bảng: `你们` → các ngươi (khi gọi từng người là ngươi) / các vị / hai người / mọi người.
  - Khối mới **Vai vế trong thoại**: ta–ngươi chỉ cho ngang hàng xa lạ/đối địch hoặc bề trên nói bề dưới; bề dưới tự xưng tiểu nhân/nô tỳ/thuộc hạ/đệ tử/vãn bối/con (raw có 奴婢/属下/弟子/晚辈/小的 thì dịch đúng chữ), gọi bằng danh xưng hoặc ngài/người; cha mẹ, sư phụ thân tình: ta/cha/mẹ/vi sư – con/đồ nhi; quy tắc 你们; hai ví dụ.
  - Ghi chú bề trên: bỏ "vẫn là ngươi", trỏ tới mục Vai vế, giữ cấm ông/bà.
- Check chính (cổ đại) `mình/tôi`: gợi ý mới "→ ta, hoặc tự xưng theo thân phận (tiểu nhân/nô tỳ/thuộc hạ/đệ tử/con)".
- Luật cảnh báo (cổ đại), áp khi thoại mở bằng Thiếu gia/Chủ nhân/Lão gia/Đại nhân/Sư phụ/Sư tôn/Điện hạ/Bệ hạ/Tiền bối/Phụ thân/Mẫu thân/Phụ hoàng/Mẫu hậu: trong câu đó gọi `ngươi`; hoặc tự xưng `ta` (bỏ qua chúng ta/bọn ta/người ta).

**Kiểm.** Golden thêm 5 câu mẫu: 2 câu sai bị cảnh báo, "chúng ta / người ta", "nô tỳ … ngài", "Công tử, ngươi…" qua; Rust khớp TS. qt-web thêm test tương ứng.

**Chưa làm.** Các cặp ta–ngươi đã học vào glossary/cast (21% cặp truyện cổ) → mục 11.

## 5. Các dòng prompt dạy sai

**Vấn đề.** Đợt thẩm định nêu 4 dòng: bảng họ hiếm `佟 Đông`; bảng lóng `舔狗 → liếm chó` (118 lần, lật nghĩa); bảng "Tránh → Dùng thay" ghi `nhận dạng → kiểm trắc`, tức ra lệnh dùng "kiểm trắc" (335 lần/38 truyện); bảng kính ngữ `老X → Lão X` viết hoa như tên riêng ("Lão Bùi" 33 lần, lọt vào glossary).

**Quyết định (chủ app chốt 11/10/2026).**

- 佟: **giữ `Đông`**. Kiểm lại thì từ điển Thi Viện ghi "(Danh) Họ Đông", Wikipedia tiếng Việt viết Đông Quốc Duy, Đông Giai thị. Kết luận "phải là Đồng" trong bản thẩm định sai; đã đính chính README và trang HTML. Bảng gốc là gist "Họ Trung Quốc có âm Hán-Việt đặc biệt" của chủ app; gist có 2 chữ gõ nhầm (`苈 Mị` phải là 芈, `邑 Lục` phải là 逯), prompt đã đúng chữ.
- 舔狗 → **liếm cẩu**.
- 检测: chủ app để tao chọn → **phát hiện / quét thấy / kiểm tra**; thông báo hệ thống `检测到` → "Phát hiện…".
- 老X → **lão X** viết thường giữa câu.

**Thay đổi.**

- Prompt: dòng "Tránh → Dùng thay" thành `kiểm trắc / nhận dạng (检测) → phát hiện / quét thấy / kiểm tra — 检测到 → Phát hiện`; `老X` ở bảng kính ngữ cổ đại và hiện đại viết thường, ghi rõ không phải tên riêng; `舔狗 → liếm cẩu`, cột cấm "liếm chó".
- Check chính (mọi bối cảnh): bắt "kiểm trắc", "liếm chó" (hai cách dịch luôn sai). Không thêm check cho "Lão X" viết hoa vì dễ báo nhầm tên riêng (Thái Thượng Lão Quân).

**Kiểm.** Golden thêm dòng mẫu "Kiểm trắc thấy một kẻ liếm chó."; qt-web thêm test cho ba bối cảnh.

**Chưa làm.** Chặn glossary lưu "Lão X" thành tên riêng (P1, nhóm danh xưng); sửa bản dịch cũ.

## 6. Số và đơn vị

**Vấn đề.** Prompt không có luật nào về số/đơn vị: 亿 → "ức" 236 lần/17 truyện ("một trăm ức" thay "mười tỷ"); 百亿 → "trăm tỷ" 98/182 lần (đúng là 10 tỷ); 一倍 → "một nửa" (đúng là gấp đôi); 斤 → "cân".

**Quyết định (chủ app chốt 11/10/2026).**

- 万: hiện đại → nghìn/triệu; cổ đại **giữ "vạn"**.
- 亿: hiện đại → trăm triệu/tỷ; cổ đại **"trăm triệu"** cho số đếm cụ thể, cụm cố định 亿万 giữ "ức vạn".
- 斤: **giữ "cân"** ở mọi bối cảnh, đúng số của raw (đã nêu rủi ro người đọc hiểu cân = ký nên "100 cân" thành 100 ký trong khi 100斤 = 50 ký; chủ app chấp nhận).
- 小时 (giờ 60 phút) → "tiếng" / "giờ" ở mọi bối cảnh, không "canh giờ" (1 canh giờ = 2 tiếng); 时辰 → "canh giờ".

**Thay đổi.**

- Prompt (phần dấu câu dùng chung ba bối cảnh): mục mới `Số và đơn vị` theo các quyết định trên, thêm 一倍 = gấp đôi, 翻了一倍 = tăng gấp đôi, 米 = mét, 三五 = dăm ba.
- Luật cảnh báo: số (chữ số hoặc chữ) + "ức" ở mọi bối cảnh, trừ "ức vạn", "ức hiếp"; số + "vạn" chỉ ở hiện đại, trừ vạn tuế/vật/nhất/sự/năng/phần.

**Kiểm.** Golden thêm 6 câu mẫu: hiện đại bắt "một trăm ức", "9,6 ức", "20 vạn", "Mười vạn"; cổ đại và hỗn hợp chỉ bắt hai câu "ức"; "ức hiếp / ức vạn / vạn tuế / vạn nhất" qua. qt-web thêm test.

**Chưa làm.** Bắt chính xác theo raw (raw có 一倍 mà ra "một nửa", raw có 百亿 mà ra "trăm tỷ") cần check so từng cặp đoạn raw ↔ bản dịch → mục P1 "luật check dựa trên raw".

## 7. Sửa setting sai

**Vấn đề.** Mỗi truyện chỉ có một `genre.setting`, quyết định base prompt, luật xưng hô và bộ check. Truyện xuyên nhanh để `ancient` nên arc hiện đại ra "lão sư", "thê tử", "phu quân", "Ân"; truyện đô thị để `ancient` thì check cổ đại cấm "vợ/chồng", sinh "thê tử" 360 lần, 0 lần "vợ". AI điền hồ sơ chỉ đọc 3 chương đầu nên không thấy các arc về sau.

**Quyết định (chủ app chốt 11/10/2026).** Sửa 4 truyện; Ma môn (prologue hiện đại ở ch.1) giữ `ancient`; Biến thân nữ đạo sĩ giữ `mixed` (lỗi "muội" trong lời kể đến từ toneRules, đổi setting không chữa được). Setting theo arc để mục P1.

**Thay đổi dữ liệu** (thư mục `bookshelf/books/common`, ngoài repo; bản cũ lưu `story.json.truoc-sua-setting-20261011.bak` cạnh file):

| Truyện | Trước | Sau | Ghi chú |
|---|---|---|---|
| xuyen-nhanh-phao-hoi-co-ay-khong-di-duong-binh-thuong | ancient | mixed | dịch 44/787, chương tới dùng ngay |
| xuyen-nhanh-tuyet-sac-yeu-nu-dua-vao-sinh-con-song-lau-tram-tuoi | ancient | mixed | đã dịch hết 436, chỉ tác dụng khi dịch lại |
| lao-ba-qua-manh-cung-ta-cach-li-sinh-san-lam-sao-bay-gio | ancient | modern | đã dịch hết 638, chỉ tác dụng khi dịch lại |
| huyen-giam-than-hoa-ngan-van | (không có genre) | ancient / han / neutral | hành vi không đổi, chỉ ghi rõ |

Truyện xuyên nhanh pháo hôi có khoá phiên mồ côi `work/.session.lock` (pid 35588 không còn chạy, từ 06/10) — để nguyên, app tự dọn khi mở phiên mới.

**Thay đổi code.** Chỉ dẫn chọn setting khi lập hồ sơ (`api_fill.rs` cho API key, `apps/qt-ai-cli/antigravity/workflows/setup-story.md` cho agy) thêm: xuyên nhanh / xuyên chư thiên mà các thế giới gồm cả cổ lẫn hiện đại → `mixed` (tên có 快穿 / xuyên nhanh gần như luôn vậy dù vài chương đầu chỉ thấy một thế giới); chỉ có prologue hoặc một arc ngắn khác bối cảnh thì chọn bối cảnh chiếm phần lớn. Không đẩy truyện chư thiên toàn cổ đại (Thiên Long…) sang mixed.

**Kiểm.** `qt-ai status` đọc được story.json đã sửa; test `api_fill`, `templates` xanh; golden khớp.

## 8. Glossary tự sinh được khai nhóm `common`

**Vấn đề.** Ba chỗ ra lệnh trích glossary (lượt trích riêng và đuôi lượt dịch của luồng API, lệnh `next` của agy) chỉ cho names/places/items/creatures/skills và dặn "Bỏ qua từ chung", nên nhóm `common` chỉ chiếm 0,18% mục tự sinh; thuật ngữ cốt lõi trôi suốt truyện (面板 "bảng giao diện" 245 / "bảng điều khiển" 303; 好感度 hảo cảm/thiện cảm; 末世 mạt thế 627 / tận thế 144; 督察局 ba tên).

**Quyết định (chủ app chốt 11/10/2026).** Cho khai `common` cho thuật ngữ riêng của truyện; chấp nhận thuật ngữ được chốt theo lần dịch đầu sau khi sửa (chưa hay thì sửa trong tab Glossary; mục 12 sẽ ép các chương sau theo bản đã chốt). Việc chốt sẵn thuật ngữ đang trôi cho từng truyện là việc dữ liệu riêng, làm sau.

**Thay đổi.** Ở `api_session.rs` (GLOSSARY_EXTRACT_SYSTEM_PROMPT, GLOSSARY_INLINE_INSTRUCTION), `commands/next.rs` và bản gốc TS (`apps/qt-ai-cli/src/commands/next.ts`, `apps/qt-web/src/lib/ai-client.ts` — kể cả `enum` của JSON schema):

- category cho phép thêm `common`.
- Câu mới: khai `common` cho thuật ngữ riêng của truyện lặp lại nhiều chương — chỉ số/bảng/âm báo hệ thống (面板, 好感度, 积分, 检测到), khái niệm thế giới (末世, 丧尸, 大比, 成长体), tổ chức/chương trình (督察局, 春晚); target chép nguyên văn; KHÔNG khai từ thường, thành ngữ, lóng/meme, danh xưng (老X, X阿姨, X女士).
- "Bỏ qua từ chung" → "Bỏ qua từ thông thường không riêng của truyện".

Bước lọc `sanitize_extracted` vốn đã nhận `common` (khoá ≥ 2 chữ Hán, target phải có trong bản dịch), không phải sửa.

**Kiểm.** Test `next` kiểm prompt có `skills|common` và câu chỉ dẫn mới; core, qt-web, qt-ai-cli (43 test) xanh; golden khớp.

## 9. Tên lệch dấu so với glossary: code tự sửa

**Vấn đề.** Check chỉ bắt được tên lệch dấu thanh so với glossary ("Đoạn Phong" thay "Đoàn Phong"). Luồng API đưa vào vòng soát với lời dặn "đúng ngữ cảnh thì giữ nguyên" — model giữ nguyên là chốt kèm cảnh báo, tên sai nằm lại trong out/; luồng agy tốn một vòng soát chỉ để sửa một dấu.

**Quyết định (chủ app chốt 11/10/2026).** Check đã biết chính xác cụm sai và đoạn nào, nên sửa bằng code lúc accept, không qua model. Glossary là chuẩn duy nhất (glossary sai thì sửa glossary; mục 13 chặn âm sai từ đầu). Phần lời dặn soát "tên theo glossary là bắt buộc" gộp vào mục 12, vì chỉ có tác dụng khi check bắt được các lỗi sai hơn một dấu (Sanh/Sênh, Vạn Sĩ/Mặc Kỳ, Lạc ca/anh Lạc).

**Thay đổi.**

- `check::fix_glossary_drifts`: thay cụm lệch dấu bằng target glossary trong từng đoạn, lặp tới khi sạch, trả danh sách `"sai → đúng"`.
- `check_chapter`: lệch dấu không còn là vi phạm (không đẩy vào soát); điểm so bản soát của luồng API cũng bỏ phần này.
- `run_accept`: sửa trước khi ghi out; `AcceptResult.glossary_fixes`. Log luồng API ghi "tự sửa theo glossary: …"; CLI `qt-ai accept` in dòng "Tự sửa theo glossary: …". Không ghi vào cảnh báo.

**Kiểm.** Test lệnh: check qua, accept sửa cả hai chỗ "Đoạn Phong" trong một đoạn, tên không có entry giữ nguyên. Test API: chỉ còn một lượt gọi model (không soát), log có dòng tự sửa, out đúng tên, không cảnh báo. Clippy không có cảnh báo mới.

## 10. Phát hiện raw đổi sau khi dịch — bỏ

**Quyết định (chủ app 11/10/2026).** Không làm công cụ; chủ app tự kiểm soát khi nhập lại raw. Chương nghi lệch đã biết từ đợt thẩm định: phuong-thon-dao-chu 0364–0378 (tiêu đề lệch một bậc), 0370/0371 (hoán đổi đơn xin nghỉ), 0392 (bản dịch ch.382 cạnh raw ch.383); truong-sinh-la-meo 0252 (raw trùng ch.251), 0075 (model bịa thêm lời tác giả).

## 11. Xưng hô đóng băng ở lần gặp đầu

**Vấn đề.** 47 truyện có cặp đã yêu/cưới/bái sư vẫn giữ cặp lúc mới gặp (Đi làm mò cá: sau khi cưới vẫn "Ninh Ninh, cô nghe tôi giải thích" 155 lần; Ác nữ: đồ đệ xưng "ta" với sư phụ ở ch.1030 dù 1018–1029 xưng "con"). Gốc: prompt dặn "Giữ đúng các cặp này" và model coi là lệnh, không biết cặp chốt từ bao giờ; harness chỉ tự đưa cặp ra xét lại khi có dấu hiệu tình cảm và cặp dạng `tôi–…` hoặc `ta–ngươi/cô nương/công tử…`, bỏ sót bái sư, nhận chủ, kết nghĩa và các cặp `tớ–cậu`, `tao–mày`, `bổn cung/lão phu–ngươi`. Truyện có mốc trong cast thì chạy đúng (Tận thế phục hưng 56 mốc).

**Quyết định (chủ app chốt 11/10/2026).** Làm phần in chương chốt, đổi lời dặn, mở rộng phát hiện. **Không** chạy bù timeline cast cho 29 truyện chưa có cast.json — thay bằng báo trong hồ sơ truyện (bàn tiếp). Theo dõi trôi xưng hô và bỏ luật "không quay về cặp liền trước" để P1.

**Thay đổi** (`crates/qt-ai-core/src/cast.rs`, `prompt.rs`):

- Danh sách "Cặp xưng hô đang dùng" đổi từ JSON sang từng dòng, mỗi cặp kèm chương chốt: `- 甲→乙: X–Y (chốt từ ch.N)`; tiêu đề ghi chương đang dịch. Chương chốt lấy từ mốc cast gần nhất ≤ chương đang dịch (`addressing_since`), không có thì từ `autoGlossaryLog`; cặp nhập tay không ghi. Nhãn chương rút gọn từ id (`chapter_label`: `0001_744…` → 1, `10021482_1040_…` → 1040, `chuong-0392` → 392).
- Lời dặn "Giữ đúng các cặp này…" thành: cặp gần nhất đã ghi, mặc định giữ; nếu chương cho thấy quan hệ đã khác (thoại gọi nhau bằng danh xưng khác hẳn 师父/师尊, 主人, 夫君/相公/老公/老婆, 姐/哥, 宝贝…; hoặc đã thành người yêu, vợ chồng, thầy trò, chủ tớ, kết nghĩa, trở mặt) thì dịch theo quan hệ hiện tại cho cả hai chiều và khai lại kèm `note`.
- Phát hiện cặp cần xét lại (`pairs_to_review`):
  - Cặp "xa cách" thêm tự xưng `tớ`, `tao`, `bổn cung`, `bản cung`, `lão phu`, `bổn tọa`, `bản tọa`.
  - Nhóm dấu hiệu mới theo sự kiện `RELATION_MARKERS`: 拜师, 收徒, 为徒, 认主, 效忠, 臣服, 结拜, 结义, 金兰, 认亲, 义父, 义母, 干爹, 干妈 (không lấy danh xưng chung như 师父). `PairReview.kind`: Intimacy / Relation; chương có cả hai thì xét theo Intimacy.
  - Khối "Cặp cần xét lại … vì quan hệ mới" gợi ý cặp theo bối cảnh: cổ đại sư đồ `vi sư/ta – đồ nhi/con`, `đệ tử/con – sư phụ`, chủ tớ `nô tỳ/thuộc hạ – chủ nhân`, kết nghĩa `huynh–đệ`, `tỷ–muội`; hiện đại thầy trò, kết nghĩa, cha mẹ nuôi.
- Câu "khai lại cặp khi quan hệ đổi" ở đuôi lượt dịch API và lệnh `next` của agy thêm "thầy trò, chủ tớ".

**Báo trong hồ sơ truyện (thay cho chạy bù timeline, chủ app chốt 11/10/2026).** Tab "Nhân vật & xưng hô" (`apps/qt-ai-gui/src/components/cast-panel.tsx`): truyện có cặp xưng hô, từ 30 chương trở lên, chưa có mốc đổi hay cặp ghim nào thì khối "Xưng hô đổi theo chương" hiện khung cảnh báo (màu `status-warning`) thay cho dòng mờ "Chưa có mốc nào.": "Chưa có mốc đổi xưng hô nào: N cặp đang giữ nguyên từ lúc hai người mới gặp. Nếu có cặp đã thành người yêu, vợ chồng, thầy trò, chủ tớ… thì thêm mốc ở đây (chọn cặp, chương bắt đầu, cặp mới) để các chương sau dịch đúng." Truyện dưới 30 chương hoặc đã có mốc giữ như cũ. Test component: có cảnh báo khi đủ điều kiện, không có khi 29 chương hoặc đã có mốc.

**Kiểm.** Test `cast.rs`: định dạng mới có chương chốt và chương đang dịch, lời dặn mới; bái sư bật xét lại kiểu Relation, chỉ có 师父 thì không; chương vừa 结婚 vừa 结拜 xét kiểu Intimacy, `tớ–cậu` được coi là xa cách; `chapter_label`, `addressing_since` (mốc cast thắng log, mốc tương lai bỏ qua, cặp nhập tay không có). Core 197 test xanh (trừ test chập chờn đã ghi), clippy không cảnh báo mới.

## 12. Bắt tên viết gần đúng glossary

**Vấn đề.** Glossary đã nạp mà model vẫn viết khác hơn một dấu thanh ("Mặc Trúc Sênh" thay Sanh, "Vạn Sĩ Tiết" thay Mặc Kỳ Tiết, "Đái Mộc Bạch" thay Đới); check cũ chỉ bắt lệch dấu thanh (mục 9 đã chuyển sang tự sửa), còn lời dặn soát cho model "đúng ngữ cảnh thì giữ nguyên".

**Đo trước khi làm** (2.405 chương lấy mẫu ngẫu nhiên, 40 chương/truyện):

| Cách bắt | Chương bị bắt | Nhận xét |
|---|---|---|
| Khoá có trong raw mà bản dịch không có target (mọi nhóm) | 32% | Báo nhầm tràn lan: glossary chứa từ thường (可能 "Có Lẽ", 天下 "Thiên Hạ", 起风了 "Gió Nổi Lên Rồi"), khác trật tự từ ("Thanh Đường Sơn" / "núi Thanh Đường") |
| Như trên, chỉ names, khoá lặp ≥2 lần | 4% | Còn khoảng nửa báo nhầm (点点 "Điểm Điểm", 天天 "Thiên Thiên", "Chung gia" / "nhà họ Chung") |
| Thiếu target nhưng có cụm gần giống (cùng số tiếng, lệch nhẹ) | 9% → 4,1% | Tên 2 tiếng và danh từ viết thường trùng ngẫu nhiên ("Lăng quận" / "Lăng vẫn", "Thần Đô" / "Nhân Đồ") |
| **Gần giống, chỉ tên ≥3 tiếng viết hoa mọi tiếng, lệch đúng 1 tiếng ≤2 chữ cái, i = y** | **1,4%** | Gần như toàn lỗi thật: Mao Thiên Dực/Diệc, Thượng Quan Hoành Vĩ, Đại Phạm Thiên, Thần Bắt Ty, Đái Mộc Bạch, Bát Quái Lư, Đinh Khởi Minh |

**Quyết định (chủ app chốt 11/10/2026).** Làm theo dòng cuối; không để code tự thay (có ca là hai thứ khác thật, như "Cóc Biến Dị" / "Sói Biến Dị"), mà đưa vào soát với lời dặn bắt buộc. Check tổng quát "thiếu target" để sau khi dọn glossary (P1).

**Thay đổi.**

- `check::glossary_near_misses`: theo đúng tiêu chí dòng cuối, xét theo từng đoạn (khoá phải có trong raw của đoạn đó), bỏ cụm trùng target glossary khác. Thông báo `Tên theo glossary: {khoá} là `{target}`, bản dịch viết `{cụm}`` (`GLOSSARY_NAME_ISSUE`), tầng chính.
- `review.md` (agy) tách mục đầu "# Tên theo từ điển — BẮT BUỘC sửa…", sau đó mới tới "# Vi phạm rule".
- Lời dặn soát API tách khối "Tên theo từ điển — BẮT BUỘC sửa: thay đúng cụm sai bằng tên trong từ điển, không giữ nguyên, không tự đổi sang âm khác". Model giữ nguyên mà còn lỗi tên thì không chốt sớm, soát tiếp (hết vòng thì check chốt kèm cảnh báo như cũ). Điểm so bản soát tính cả lỗi tên.
- Prompt chương (riêng qt-ai): tiêu đề mục từ điển thành "Từ điển BẮT BUỘC: raw có khoá nào thì bản dịch dùng đúng target đó (chính tả, hoa/thường, trật tự từ); không tự sửa âm kể cả khi cho là sai." Bản port web (golden) giữ nguyên.

**Kiểm.** Test lệnh: bắt "Mao Thiên Dực", "Tô Tiểu Bạc"; không bắt cụm trùng target khác ("Lâm Thanh Phong"), biến thể i/y ("Diên Hy Cung"), tên 2 tiếng, target có tiếng viết thường; review.md có mục bắt buộc. Test API: model giữ nguyên tên sai → soát tiếp → lượt sau sửa → chốt không cảnh báo. Core 200 test xanh, clippy không cảnh báo mới.

## 13. Bảng âm Hán-Việt kiểm glossary (bỏ)

**Vấn đề.** Glossary tự sinh chép âm từ bản dịch đầu tiên; âm phiên sai được chốt rồi lan cả truyện. Bản thẩm định đề xuất bảng âm Hán-Việt để chặn.

**Đo trước khi làm.** Không nguồn nào đủ chắc để chặn:

- Bảng QT (`QT2025/Resources/ChinesePhienAmWords.txt`) một âm mỗi chữ: kiểm 15.965 tên glossary thì 8,7% bị coi là "lệch", đa số là âm hợp lệ khác (长 trưởng, 武 võ, 主 chúa, 启 khởi, 带 đới…).
- Unihan `kVietnamese` (Unicode) nhiều âm nhưng thiếu âm quen dùng: một mình 26,2% lệch; gộp với QT còn 7,6%.
- Từ điển Lạc Việt (`QT2025/Resources/LacViet.txt`, offline) có âm cho 8.242 chữ, nhưng **vênh với Thi Viện**: 绮 Lạc Việt "khởi" / Thi Viện "khỉ, ỷ"; 屿 Lạc Việt "dữ" / Thi Viện "tự". QT có 1.851 âm mà Lạc Việt và Unihan không ghi, xem mẫu đa số vẫn hợp lệ (幻 huyễn, 场 tràng, 刃 nhận, 晴 tình).
- Thi Viện: tra tự động 3.694 chữ (1 giây/chữ) thì bị chặn sau khoảng 95 chữ — trang trả "truy cập của bạn đã bị từ chối" (có thể ảnh hưởng truy cập Thi Viện từ máy này; liên hệ thivien.contact@gmail.com nếu kéo dài). Đã dừng, không thử lách; dữ liệu Thi Viện không đưa vào bảng.

**Quyết định (chủ app chốt 11/10/2026).** Lúc đầu chốt chỉ nhắc bằng code (không chặn, không gọi AI), gộp mọi nguồn vì bảng QT không chắc. Đã dựng xong rồi **gỡ** cùng ngày: không có nguồn đủ chuẩn (Thi Viện chặn tra tự động; Wiktionary tiếng Việt chỉ có âm cho 598/3.694 chữ trong glossary và lẫn âm sai như 天 yêu, 功 lực, 林 hoằng — khớp trọn 28/95 chữ mẫu Thi Viện, kém cả Lạc Việt 64/95 và QT 56/95), nên chủ app chốt để AI tự dịch tên như cũ. Code không còn trong repo; chỉ giữ danh sách rà tay.

**Đã dựng rồi gỡ (ghi lại để khỏi làm lại).**

- `crates/qt-ai-core/data/build_hanviet.py` sinh `data/hanviet.tsv` (17.630 chữ): QT + Lạc Việt + Unihan kVietnamese (cần file `Unihan_Readings.txt` tải từ unicode.org khi chạy lại) + danh sách âm quen dùng (trưởng, võ, chúa, phụng, khâu, bổn, Tào Tháo, Võ Tòng, Côn Lôn, Kim Cang, Mẫu Đơn, Sư Phụ…).
- `crates/qt-ai-core/src/hanviet.rs`: so âm bằng khoá không phụ thuộc kiểu đặt dấu (hoà/hòa, tuỵ/tụy), y ≡ i; chỉ xét tên phiên âm từng chữ (số tiếng = số chữ, mọi tiếng viết hoa); chỉ nhắc khi lệch đúng **một** tiếng (lệch nhiều tiếng thường là dịch nghĩa/đảo trật tự). Đo trên 30.441 tên: 2,4% bị nhắc.
- `run_accept`: tên glossary tự sinh vừa thêm (names/places/items/creatures/skills) nghi sai âm → `AcceptResult.glossary_notes` "glossary mới nghi sai âm: 李渊蛟 → Lý Uyên Kiều (蛟 viết `Kiều`, bảng tham khảo: giao) — xem lại trong tab Glossary". Vẫn thêm vào glossary, không phải cảnh báo chương. Log luồng API; CLI `qt-ai accept` in "Nhắc: …".

**Giữ lại.**

- Rà glossary hiện có: [nghi-sai-am.md](nghi-sai-am.md) — 703 tên ở 57 truyện, theo truyện, để soát tay. Lẫn lỗi thật (蛟 Kiều→Giao, 冲 Trùng→Xung, 蠹 Đỗ→Đố, 姚 Dao→Diêu, 炽 Xích→Sí, 辟 Bích→Tịch, 境 Kính→Cảnh, 畔 Phán→Bạn) với dịch nghĩa một tiếng (Kiếm Mộ, Vạn Dặm) và âm còn tranh cãi.

**Kiểm.** Sau khi gỡ (xoá `src/hanviet.rs`, `tests/hanviet.rs`, `data/`, `AcceptResult.glossary_notes` và các chỗ in/log): core 200 test xanh, clippy chỉ còn cảnh báo cũ.

## 14. Chuẩn hoá ngoặc và gạch dài bằng code

**Vấn đề.** Định dạng đổi theo từng phiên 10 chương: cả lô chương dùng ngoặc thẳng `"` (6.547/27.343 file có); prompt bắt đổi `『』` thành `“”` nên ý nghĩ thầm thành lời nói ra miệng (Âm thịnh dương suy mất 750/2.045 `『』`); luật `——` → ` — ` sinh `“Boong — ”`, `“Ngươi — !”` (2.578 chỗ) và `“ — Giết!”` (73 chỗ).

**Quyết định (chủ app chốt 11/10/2026).** Chuẩn hoá bằng code lúc chốt, không giao cho model. `『』` (dấu ngoặc, không phải chữ Hán) luôn giữ nguyên như raw để giữ ý tác giả; `「」` vẫn thành `“”`. Chạy luôn cho bản dịch cũ.

**Thay đổi.**

- `normalizeAiPunctuation` (TS, `ai-translation.ts`) / `normalize_punctuation` (Rust, `paragraphs.rs`), gọi cho từng dòng trong `formatAiTranslation` / `format_translation` → áp cho web, qt-ai-cli và app lúc accept.
  - `"` thành `”` nếu ký tự sau là hết dòng, khoảng trắng hoặc dấu đóng; nếu không thì thành `“` khi ký tự trước là đầu dòng, khoảng trắng, ngoặc mở, gạch dài hoặc dấu hai chấm; còn lại là `”`. Không ghép cặp, nên thoại kéo nhiều đoạn (đoạn đầu chỉ có ngoặc mở) vẫn đúng.
  - Bỏ khoảng trắng giữa `—` và dấu đóng (`— !` → `—!`, `— ”` → `—”`), giữa ngoặc mở và `—` (`“ — ` → `“— `).
  - Nháy đơn `'` không đụng (dấu lược trong tên tiếng Anh).
- Prompt (CORE_PUNCT): `「」` → `“”` (ngoặc cong, không ngoặc thẳng); `“”`, `‘’` giữ nguyên; `『』` giữ nguyên như raw (ý nghĩ, truyền âm, tên sách/chiêu thức, thoại lồng…), KHÔNG đổi thành ngoặc kép; gạch dài sát dấu câu/ngoặc. Hash base mới `764d853302ce8ca7`.
- Luật cảnh báo "tôi → ta" coi `『』` như ngoặc kép: ý nghĩ trong `『』` giữa dòng không bị bắt.
- Golden: thêm 3 ca format.
- Bản dịch cũ (`bookshelf/books/common/*/out/*.txt`): chạy cùng luật bằng script Python chép luật (tự kiểm khớp golden trước khi chạy, bỏ qua truyện có phiên dịch đang chạy — lần này không có). 7.934/27.343 file đổi: 351.017 ngoặc thẳng → cong, 2.932 dòng sửa gạch dài. Chưa commit trong repo bookshelf; lùi bằng git được. `『』` đã bị đổi thành `“”` trong bản dịch cũ thì không khôi phục được (phải dịch lại).
- Mặt cười có `"` (`(﹁"﹁)`, khoảng 30 chỗ, gần như một truyện) cũng bị đổi thành `”` — vẫn đọc ra mặt cười, không thêm luật riêng.

**Kiểm.** qt-web 289 test xanh (thêm test ngoặc/gạch dài), golden khớp, qt-ai-cli 43, core 200 xanh (+1 test chập chờn cũ), clippy không cảnh báo mới. Bản dịch cũ: chạy lại không còn gì để đổi; so với bản sao lưu, 7.934 file chỉ khác ở ngoặc kép và khoảng trắng sát gạch dài, không file nào lệch số dòng.

## 15. Tiêu đề chương

**Vấn đề.** Đo dòng đầu 27.343 chương: 127 tiêu đề thiếu dấu hai chấm (`Chương 146 Hoàn thành nhiệm vụ`, `Chương 12. …`); khoảng 390 tiêu đề viết hoa mọi chữ có hư từ (`Thực Ra Bọn Họ Đã Động Phòng Rồi`) — ngoài ra khoảng 860 tiêu đề viết hoa hết nhưng toàn tên riêng (`Cô Tô Mộ Dung`), đúng; 105 tiêu đề kèm lời xin phiếu / thêm chương của tác giả (`(Cầu xin đặt mua!!)`, `【Thêm chương cho “…”】`).

**Quyết định (chủ app chốt 11/10/2026).** Viết hoa: chỉ dặn model hoa chữ đầu (và tên riêng), không thêm check, không soát. Định dạng `Chương N:` và lời xin phiếu: code sửa lúc accept. Tiêu đề là câu thoại: dặn model xưng hô như trong thoại. Bản dịch cũ để nguyên.

**Thay đổi.**

- `titles.rs`: `tidy_chapter_title` — `Chương N Tên` / `Chương N. Tên` / `Chương N - Tên` / `Chương N：Tên` → `Chương N: Tên`; cắt các ngoặc cuối tiêu đề chứa "thêm chương, tăng chương, chương thêm, bổ sung chương, nguyệt phiếu, phiếu tháng, phiếu đề cử, đặt mua, minh chủ"; còn trống thì thành `Chương N`. Dòng không mở đầu bằng `Chương N` giữ nguyên; `Chương 12.5 …` không đụng.
- `run_accept`: gọi `tidy_chapter_title` cho dòng đầu ngay sau `format_translation`, trước `settle_title`, để `titles.json` ghi tên sạch.
- Prompt (luật tiêu đề): tiêu đề viết hoa như câu thường (ví dụ `Thực ra bọn họ đã động phòng rồi`, KHÔNG `Thực Ra Bọn Họ…`); tiêu đề là câu nói của nhân vật thì xưng hô như trong thoại (`Để tôi xem!` ở truyện hiện đại); bỏ lời xin phiếu, thêm chương (`（求月票）`, `【加更】`). Hash base mới `11614a9ed7b5cf5e`.

**Kiểm.** Chạy thử luật trên dòng đầu 27.343 chương cũ (không ghi): 114 sửa định dạng, 68 bỏ lời xin phiếu, không ca lạ. Test `tests/titles.rs`: hàm dọn tiêu đề (các kiểu ngăn, nhiều ngoặc xin phiếu nối nhau, ngoặc là một phần tên giữ nguyên) và accept sửa tiêu đề trước khi ghi `titles.json`. qt-web 289, golden khớp, qt-ai-cli 43, core 202 xanh (+1 test chập chờn cũ), clippy không cảnh báo mới.

## 16. Meme, lóng mạng, chơi chữ

**Vấn đề.** 70 lỗi ở 50 truyện dịch meme/lóng theo nghĩa đen ("vây công cái búa ấy" cho 个锤子, "sói mắt trắng", "lầu trên"). Prompt cấm giải thích ẩn dụ và chú thích trong ngoặc mà không có ngoại lệ, nên với chơi chữ model không còn cách hợp lệ nào ngoài dịch nghĩa đen; biệt danh, meme bị phiên Hán-Việt viết hoa và khai vào glossary names (Tiểu Lạt Tiêu, Bức Vương). Chi tiết: `tong-hop/C_van.md` mục V4.

**Quyết định (chủ app chốt 11/10/2026).** Theo đề xuất V4, áp cho mọi bối cảnh (kể cả cổ đại — tác giả chêm meme thì dịch bằng cách nói tương đương để giữ cái hài); riêng 鸡肋 giữ "gân gà" vì người đọc đã quen.

**Thay đổi.**

- Prompt, mục 6 "Khẩu ngữ & Tiếng lóng" (dùng chung mọi base): thêm "### Meme, lóng mạng, chơi chữ" — lóng/meme dịch nghĩa bóng, nghĩa đen từng chữ là sai nghĩa; chơi chữ dựng lại bằng tiếng Việt, không được thì **được** chèn cụm giải nghĩa ngắn ngay trong câu (ngoại lệ duy nhất của luật không giải thích); biệt danh, meme dịch nghĩa, không phiên Hán-Việt viết hoa. Kèm bảng 28 mục cố định (鸡肋 gân gà, 白眼狼 đồ vô ơn, 个锤子 cái con khỉ, 真香 quay xe, 破防 sụp đổ tâm lý, 绿/戴绿帽 bị cắm sừng, 牛头人 NTR, 装逼 làm màu…).
- Luật "KHÔNG giải thích ẩn dụ…" (triết lý) và "KHÔNG giải thích thuật ngữ trong ngoặc đơn" (biên tập) thêm câu chỉ sang ngoại lệ chơi chữ.
- Chỉ dẫn trích glossary (Rust `next.rs`, `api_session.rs` ×2; TS `next.ts`, `ai-client.ts`): "KHÔNG khai (kể cả vào names) từ thường, thành ngữ, lóng/meme, biệt danh trêu chọc (小辣椒), danh xưng…".
- Hash base mới `bac1b7224f3c9e54`.

**Kiểm.** qt-web 289, golden khớp, qt-ai-cli 43, core 202 xanh (+1 test chập chờn cũ), clippy không cảnh báo mới.

## 17. Calque cú pháp (văn convert)

**Vấn đề.** 118 lỗi CALQUE ở cả 61 truyện (`tong-hop/C_van.md` mục V3): bảng "Dấu hiệu câu vẫn còn là convert" chỉ nêu mẫu hiếm, không nêu mẫu hay gặp. Đo trên 27.343 chương: "trực tiếp" 38% số chương, "đem…" 26%, "một tia/tầng/cỗ + cảm xúc" 2,4%, "chiếm tiện nghi" 1,7%, "Theo sự/việc" đầu câu 1,3%, "tổng không thể" 0,4%.

**Quyết định (chủ app chốt 11/10/2026, xem từng câu thật).**

- Sửa: 占便宜 (lợi dụng / sàm sỡ / bắt nạt theo cảnh), 一阵 + cảm xúc (không "một phen"), 一头 (không "một đầu"), 既然 (không "Đã là + đại từ"), 总不能 (không "tổng không thể"), 而 đầu câu (không "Mà…"), cụm 流量爆炸 (lượt xem bùng nổ).
- Giữ như hiện tại: "đem" (把/将), "Theo sự / Cùng với" (随着), "tiện nghi" + người thân (便宜师尊, 便宜老爹 — dịch ra nghe kỳ), "một tia/một sợi" (一丝/一缕), "trực tiếp" (直接 — dễ sửa sai), "Khá lắm" (好个/好家伙), vi diệu / khủng bố / nghẹn khuất…, "bạn học X", "sao lưu lượng".

**Thay đổi.**

- Prompt, bảng "Dấu hiệu câu vẫn còn là convert" (dùng chung mọi base): thêm 7 dòng theo quyết định trên; dòng 占便宜 ghi rõ `便宜师尊`, `便宜老爹` vẫn giữ `tiện nghi`. Hash base mới `173ca33471c77b7e`.
- Luật cảnh báo (TS `ADVISORY_CHECK_RULES`, Rust `ADVISORY_RULES`, mọi setting): "chiếm (hết) tiện nghi" và "tổng không thể". Không bắt soát. Bỏ "Theo sự/việc" và "một tia" khỏi đề xuất ban đầu theo quyết định trên.
- Golden: thêm 3 dòng vào văn bản mẫu luật cảnh báo.

**Kiểm.** Test TS: cảnh báo đúng hai mẫu ở cả cổ đại lẫn hiện đại, "Tiện nghi sư tôn" không bị bắt. qt-web 290, golden khớp, qt-ai-cli 43, core 202 xanh (+1 test chập chờn cũ), clippy không cảnh báo mới.

## 18. Bẫy đồng âm: check theo cặp đoạn raw / bản dịch

**Vấn đề.** Khoảng 55 lỗi sai nghĩa kiểu "bạn giả Hán-Việt" (`tong-hop/D_nghia_dinhdang.md` mục V3a/b): 头颅 → "đầu lâu", 放肆 → "phóng túng"… Check chỉ đọc bản dịch nên không bắt được — "đầu lâu" đúng khi raw là 骷髅. Cần luật có điều kiện trên raw của cùng đoạn [[n]].

**Đo trước khi làm** (27.337 chương căn được raw, số đoạn dính / số truyện): 头颅/脑袋/人头 → đầu lâu 468/35, 欲哭无泪 → dở khóc dở cười 101/28, 放肆 → phóng túng 79/25, 翻白眼 → trợn tròn 53/23, 总不能 → tổng thể 49/23, 交待 → lời ăn tiếng nói 29/12, 一个头两个大 → hai thứ tóc 9/4, 拳力 → quyền lực 8/7, 藏着掖着 → giấu đầu lòi đuôi 3/3; 呵 → Hừ 1.147/51. Loại vì báo nhầm: 欣慰 → an ủi ("thấy được an ủi" đúng), 龙眼 → mắt rồng (mắt rồng chạm khắc), 老家 → lão gia (老家主 = lão gia chủ). Các cặp chỉ gặp 1 lần (铁青, 前台, 退休, 纸醉金迷, 樱花, 老妇人, 牵引) không đưa vào.

**Quyết định (chủ app chốt 11/10/2026).** 8 cặp bắt soát; 欲哭无泪 → "dở khóc dở cười" giữ nguyên (không coi là lỗi); 呵 → Hừ chỉ cảnh báo.

**Thay đổi.**

- TS `ai-translation.ts`: `RAW_PAIR_RULES` (raw, unless, out, message, advisory) và `checkRawPairViolations(raw[], translated[])` → `{paragraph, message, advisory}`. qt-web chưa dùng (cần cặp đoạn); là nguồn để golden chép sang Rust.
- Rust `check.rs`: `RAW_PAIR_RULES` (chép tay, golden so từng trường), `raw_pair_issues`. `commands/check.rs`: chạy trên `paragraphs` + `parsed`; cặp bắt soát vào `violations` (vào review như vi phạm rule), cặp cảnh báo vào `advisories`.
- 8 cặp bắt soát: 头颅/脑袋/人头 (trừ khi raw có 骷髅/头骨) → đầu lâu; 放肆 → phóng túng; 翻白眼 → trợn tròn; 总不能 → tổng thể; 交待/交代 → lời ăn tiếng nói; 一个头两个大 → hai thứ tóc; 拳力 (trừ 权力) → quyền lực; 藏着掖着 → giấu đầu lòi đuôi. Cảnh báo: 呵 (trừ khi raw có 哼 hoặc 呵斥) → Hừ.
- Golden: `rawPairRules` + `rawPairCases` (đủ 8 cặp, các ca loại trừ, đoạn thiếu).

**Kiểm.** Golden khớp (bộ luật và kết quả từng ca); test `bay_dong_am_so_doan_raw_voi_doan_dich_cung_nhan`: "đầu lâu" cho 头颅 thành vi phạm ở [[1]], "Hừ" cho 呵 thành cảnh báo ở [[2]]. qt-web 290, qt-ai-cli 43, core 204 xanh (+1 test chập chờn cũ), clippy không cảnh báo mới.

## 19. Đại từ lời kể, biến thân, 祂, 你 phiếm chỉ

**Vấn đề.** Cùng một nhân vật lúc "hắn" lúc "lão", lúc "nàng" lúc "bà"; nhân vật nữ bị kể "hắn" (Tề Hoàng 170 dòng / 41 chương); 祂 lúc "Ngài" lúc "hắn"; 你 phiếm chỉ thành "ngươi". Cast chỉ lưu giới, không có đại từ lời kể.

**Đo trước khi làm.**

- 祂: Thính tiên sinh 128 đoạn "Ngài" / 219 đoạn "hắn"; Ta dưỡng thành các nữ thần 10 / 60; Huyền giám 70 / 28.
- "ngươi" ngoài ngoặc kép (raw có 你, raw không có ngoặc thoại): 6.013 đoạn / 37 truyện, nhưng mẫu gần như toàn là thoại mà raw không có ngoặc — không làm check được.
- Khai giới ngược trong cast.json: chỉ 4 nhân vật / 3 truyện.
- Truyện biến thân (5 truyện): tác giả đổi 他/她 theo **từng câu** suốt truyện, không theo mốc chương; model đang theo raw rất sát (raw 他 → hắn, raw 她 → cô/nàng, mỗi truyện chỉ lệch 2–8 đoạn) nhờ 4/5 nhân vật chính chưa có giới trong cast. Một "mốc đổi giới" sẽ ép sai.

**Quyết định (chủ app chốt 11/10/2026).** Bỏ mốc đổi giới, thay bằng giới thứ ba "biến thân" (tự điền và đánh tay được); in đại từ lời kể theo giới và cho sửa tay từng nhân vật; 祂 mặc định "Ngài"; 你 phiếm chỉ chỉ dặn trong prompt.

**Thay đổi.**

- `cast.rs`: `Gender::Swap` ("biến thân", JSON `swap`), `Gender::fixed`, `Cast::fixed_gender_of`; `Character.narration` (đại từ lời kể ghi riêng). Check sai giới (`gender_violations`) và cặp trái giới (`pair_gender_conflict`) bỏ qua nhân vật biến thân.
- Danh sách "Nhân vật có mặt trong chương" trong prompt chương: mỗi dòng ghi giới + đại từ lời kể — nam `hắn`; nữ `nàng` (cổ đại) / `cô` (hiện đại) / theo cảnh (hỗn hợp); biến thân "lời kể theo đại từ của raw từng câu (他 → `hắn`, 她 → …); người khác gọi theo vẻ ngoài lúc đó"; ghi riêng thì "`lão` (ghi riêng)". Nhân vật chưa rõ giới mà có đại từ ghi riêng vẫn được liệt kê. Đầu mục dặn nhân vật lớn tuổi được dùng `lão` / `bà` nếu hợp.
- Tự điền biến thân: chỉ dẫn trích glossary (Rust `next.rs`, `api_session.rs` ×2) và prompt "Quét giới tính (AI)" (`cast_scan.rs`) cho khai `"biến thân"`. `learn_genders`: khai biến thân đổi nhân vật tự học nam/nữ sang biến thân (đặt tay thì chỉ ghi `disputed`); đã biến thân thì bỏ qua khai nam/nữ.
- GUI tab nhân vật: lựa chọn "Biến thân", ô "Đại từ lời kể" (trống = theo giới); xoá giới vẫn giữ đại từ đã ghi. Schema `gender` thêm `swap`, `narration`.
- Prompt: dòng `祂` trong bảng đại từ cổ đại và hiện đại (hỗn hợp ghép cả hai): **Ngài** — một cách cho cả truyện, glossary có `祂` thì theo glossary; KHÔNG hắn, y. Ghi chú "你 phiếm chỉ" trong bảng cổ đại: lời kể không chỉ ai cụ thể → lược chủ ngữ hoặc `người ta`, KHÔNG `ngươi`. Hash base mới `87000a09c6d24bfa`.
- Dữ liệu: đánh dấu biến thân (source auto) cho nhân vật chính 5 truyện — 陆清远, 林小婉, 夏小白, 江璃, 江柠 (江柠 trước là nữ, tự học). Sao lưu `cast.json.truoc-bien-than-20261011.bak` ở 4 truyện; truyện yêu nữ thải bổ trước đó chưa có cast.json (xoá file là về như cũ). Khoá phiên ở 4 truyện là khoá mồ côi của pid 35588 (đã chết từ 06/10), để nguyên.

**Kiểm.** Test `tests/cast.rs`: danh sách nhân vật ghi đại từ đúng theo giới / biến thân / ghi riêng / cổ đại; biến thân không bị check theo giới; học biến thân (tự học → đổi, đặt tay → disputed, đã biến thân → bỏ qua). GUI: test chọn Biến thân + đại từ ghi riêng. qt-web 290, golden khớp, qt-ai-cli 43, GUI 239, core 207 xanh (+1 test chập chờn cũ), clippy core và GUI không cảnh báo mới.

## 20. Setting theo arc (bỏ)

**Đề xuất gốc.** `genre.arcs: [{from, setting}]` + `effective_genre(story, chapter_id)` để một truyện đổi bối cảnh từ chương N.

**Quyết định (chủ app chốt 11/10/2026).** Bỏ. Hai ví dụ của bản thẩm định (Lão bà quá mạnh bị đặt cổ đại, Xuyên nhanh pháo hôi) đã sửa ở mục 7; thư viện còn 5 truyện hỗn hợp (35 cổ đại, 24 hiện đại), prompt hỗn hợp đã có cả hai bảng xưng hô và dặn chọn theo cảnh. Đánh dấu arc phải làm tay cho từng ranh giới, truyện xuyên nhanh có hàng chục arc — công sức lớn, lợi ích thêm không rõ.

## 21. Glossary: danh xưng làm tên riêng, `《X》`, nhóm lạ

**Vấn đề.** `tong-hop/B_glossary.md` B2, B5, B6: biến thể tên và mục trùng, glossary học muộn ghi đè cách dịch cũ, danh xưng (老张, 李老师, 苏夫人…) lưu như tên riêng — ép viết hoa giữa câu ("Thầy Lý", "Lão Trương") và rơi vào bảng nhân vật.

**Đo trên glossary 64 truyện.**

- Danh xưng trong names: 488 mục / 49 truyện.
- Hai khoá cùng bản Việt: 323 nhóm — đa số là cùng một người tác giả gõ lệch (苏清音/苏青音 → "Tô Thanh Âm"), trùng là đúng; va chạm thật ít (窥天境/窥天镜).
- Khoá lệch 1 chữ / đảo chữ mà bản Việt khác: 32.986 cặp — gần như toàn tên khác nhau thật (苍兰州/苍兰城).
- Khoá ngắn nằm trong khoá dài mà bản Việt không khớp: 292 cặp, lẫn nhiều trùng hợp (阳城 ⊂ 洛阳城).
- Có cả `X` và `《X》`: 91 cặp, 20 cặp dịch khác nhau (gần như chỉ 2 truyện).

**Quyết định (chủ app chốt 11/10/2026).** Chặn danh xưng khi tự thêm glossary; bỏ `《》` khỏi khoá; nhóm lạ thì bỏ mục; xoá các mục danh xưng cũ. Không làm cổng kiểm trùng, alias biến thể, kiểm cách dịch cũ trong out/ (nhiễu nhiều, lỗi thật ít — B5 chỉ 9 lỗi).

**Thay đổi.**

- `sanitizeExtractedGlossary` (TS `ai-glossary.ts`) / `sanitize_extracted` (Rust `glossary.rs`):
  - `TITLE_NAME`: khoá nhóm names dạng [老/小/阿] + họ một chữ hoặc họ kép (欧阳, 上官…) + [thứ tự / 大] + danh xưng (姑娘, 夫人, 老师, 大夫, 捕头, 长老, 先生, 小姐, 阿姨, 师姐, 哥, 姐, 少, 导, 总, 董, 经理…), hoặc `老X` → không thêm. Biệt danh hai chữ trước danh xưng (东溟夫人, 厄运小姐, 武天老师) vẫn là tên, vẫn thêm.
  - `《X》` → khoá `X`, bản Việt bỏ `《》` (đã có `X` thì bỏ); raw có `《X》` vẫn nạp mục `X`.
  - Nhóm ghi sai (chuỗi không thuộc nhóm nào) → bỏ mục, không rơi vào names; thiếu hẳn trường nhóm vẫn là names như cũ.
- Golden: thêm 2 ca lọc (danh xưng, `《》` khi chưa có / đã có `X`, nhóm lạ, thiếu nhóm, thứ tự `周七小姐`, biệt danh `东溟夫人`).
- Dữ liệu: xoá **426 mục** danh xưng khỏi `glossary.names` ở 46 truyện, sao lưu `story.json.truoc-xoa-danh-xung-20261011.bak`. Lần quét đầu theo luật lỏng xoá 488 mục, rà lại thấy bắt nhầm tên thật và biệt danh (蒙哥 Mông Kha, 阿里不哥, 厄运小姐 Miss Fortune, 老鼠 Twitch, 东溟夫人…) → siết luật như trên và khôi phục 62 mục (không khớp luật siết, hoặc bản Việt không có chữ danh xưng nào). Cast.json và cặp xưng hô giữ nguyên.

**Kiểm.** Test TS: nhóm lạ bỏ / thiếu nhóm là names; danh xưng không vào names, `《》` bỏ, biệt danh giữ. Dữ liệu: so từng story.json với bản sao lưu — chỉ mất đúng 426 dòng mục, không dòng nào khác đổi. qt-web 291, golden khớp, qt-ai-cli 43, core 207 xanh (+1 test chập chờn cũ), clippy không cảnh báo mới.

## 22. Theo dõi trôi xưng hô lúc accept, chạy bù timeline cast (bỏ)

**Quyết định (chủ app chốt 11/10/2026).** Bỏ. Chạy bù timeline chủ app đã từ chối ở mục 11; phần theo dõi trôi xưng hô đã có ở mục 11 (prompt in cặp kèm chương chốt, raw có dấu hiệu đổi quan hệ thì đưa ra xét lại, tab nhân vật nhắc khi truyện chưa có mốc đổi).

## 23. Check bắn nhầm; `style.avoid` thành check

**Đo.** Luật hiện đại cấm xưng hô cổ trang (`ngươi`, `nàng`, `bọn ta`, `các ngươi`) bắt oan 53 dòng ở truyện hiện đại chỉ vì cụm tiếng Việt hợp lệ: "con ngươi" 34, "cô nàng" 17, "nàng thơ" 1, "nàng dâu" 1 — mỗi lần tốn một lượt soát. 61 truyện có `style.avoid`, nhưng toàn câu hướng dẫn dài ("Tránh câu văn dài dòng, lê thê…"), không phải từ cụ thể; vài từ cụ thể bên trong (cư nhiên, não hải) luật check chung đã bắt.

**Quyết định (chủ app chốt 11/10/2026).** Sửa luật hiện đại; không biến `style.avoid` thành check.

**Thay đổi.** Luật hiện đại (TS `CHECK_RULES`, Rust `DEFAULT_RULES`) thêm `(?<!con )(?<!cô )` trước và `(?!\s(?:thơ|dâu|tiên)(?!\p{L}))` sau — bỏ qua "con ngươi", "cô nàng", "nàng thơ", "nàng dâu", "nàng tiên". Golden: thêm 2 dòng vào văn bản mẫu hiện đại (cụm hợp lệ không bị bắt; "các ngươi" vẫn bị bắt).

**Kiểm.** qt-web 291, golden khớp, qt-ai-cli 43, core 207 xanh (+1 test chập chờn cũ), clippy không cảnh báo mới.

## 24. Chương lời tác giả (bỏ)

**Đo.** Khoảng 30 chương lời tác giả thật ở 11 truyện (xin nghỉ, cảm nghĩ lên kệ, báo truyện mới, rút thăm phiếu); nhận diện bằng từ khoá còn bắt nhầm ngoại truyện (番外) và chương thật. Glossary học từ các chương này 38 mục, phần lớn là tên nhân vật thật — không hại. Tỉ lệ độ dài dịch/raw so với trung vị của truyện: p99,9 = 1,13; chỉ 1 chương trong 27.315 vượt 1,6 lần (Trường sinh là mèo ch.75).

**Quyết định (chủ app chốt 11/10/2026).** Bỏ cả mục, kể cả câu prompt "lời tác giả xưng tôi, gọi độc giả các bạn" — quá ít ca.

## 25. Sửa "Ân" trong bản dịch cũ (P2)

**Quyết định (chủ app chốt 11/10/2026).** Nhóm P2: bỏ "soát nghĩa có chọn lọc" (81% số chương có đoạn dính dấu hiệu rủi ro → thêm một lượt gọi AI mỗi chương, khoảng +15–25% token, ăn quota agy, mà dấu hiệu bắt rộng); làm "sửa hàng loạt dữ liệu cũ" (chỉ còn "Ân" — ngoặc thẳng đã làm ở mục 14, tên sai âm đã bỏ ở mục 13); "lệnh rà glossary" bàn sau.

**Thay đổi (dữ liệu, không đổi code).** Script chạy trên `bookshelf/books/common/*/out/*.txt`, chỉ sửa ở đoạn mà raw cùng nhãn có 嗯/恩, cùng ngữ cảnh với luật cảnh báo mục 1 (đầu dòng, sau ngoặc mở hoặc sau dấu kết câu, theo sau là dấu câu), theo bảng thán từ mục 1:

| Bản dịch cũ | Sửa thành | Số chỗ |
|---|---|---|
| `Ân,` | `Ừm,` | 1.323 |
| `Ân.` / `Ân...` | `Ừm.` / `Ừm...` | 1.290 |
| `ân một tiếng` (kể cả `“Ân” một tiếng`) | `ừm một tiếng` | 299 |
| `Ân~` | `Ưm~` | 180 |
| `Ân!` | `Ừm!` | 174 |
| `Ân?` | `Hả?` | 152 |
| `ân` viết thường sau dấu kết câu (`... ân...`) | `ừm` / `hả` / `ưm` | 54 |

Tổng 3.455 dòng / 2.638 file. Bỏ qua 6 file số đoạn raw ≠ bản dịch, và các dòng raw không có 嗯/恩 (phòng "Ân" là tên người). Không đụng `A?` (bản cũ dịch 嗯？) vì `A?` còn là 啊？. Bản gốc sao lưu ngoài repo (scratchpad `q25_backup/`, kèm manifest); thư viện là repo git, chưa commit.

**Kiểm.** Chạy thử trước, xem mẫu từng loại (kể cả chữ thường). So với bản sao lưu: 2.638 file chỉ khác ở chữ thán từ, không lệch số dòng; chạy lại không còn gì để sửa.

## 26. Rà ngược chương cũ theo mục glossary; đánh dấu cặp xưng hô đã có mốc (P2)

**Vấn đề.** Glossary học muộn hoặc tác giả gõ lệch tên thì các chương đã dịch trước đó giữ cách dịch khác (陆云停 "Lục Vân Đình" nhưng 4 đoạn cũ viết "Lục Đình Vân"); sửa một mục glossary không sửa được chương cũ. Cặp xưng hô trong story.json đã bị mốc trong bảng nhân vật thay thế (bản thẩm định đếm 262) mà tab Glossary không cho biết.

**Quyết định (chủ app chốt 11/10/2026).** Làm rà ngược + đánh dấu cặp xưng hô; không làm lệnh glossary-audit riêng (phần kiểm trùng đã bỏ ở mục 21). Không tốn token — chỉ code và GUI.

**Thay đổi.**

- Core `backfill.rs`: `scan_glossary_usage(paths, key, target)` — đoạn đã dịch mà raw có khoá, bản dịch không chứa cách dịch hiện tại (căn đoạn theo thứ tự dòng không rỗng; chương lệch số đoạn thì bỏ qua và đếm); `backfill_glossary(paths, key, from, to, apply)` — thay `from` → `to` CHỈ trong các đoạn raw có khoá, giữ nguyên kiểu xuống dòng, `to` chứa `from` thì không thay chồng chỗ đã đúng; `apply = false` chỉ đếm.
- Tauri `glossary_cmds.rs`: `glossary_scan` (tổng + 50 đoạn mẫu), `glossary_backfill` (từ chối ghi khi truyện đang có phiên dịch chạy); cả hai `async` + `blocking` như các lệnh nặng khác.
- GUI tab Glossary: mỗi dòng (trừ nhóm xưng hô và cụm style) có nút "Rà chương cũ" → hộp thoại: số đoạn / chương thiếu cách dịch + câu mẫu kèm `[[n]]`; ô "Cụm sai" và "Thay bằng" (điền sẵn cách dịch hiện tại); "Xem trước" → "Sẽ thay K chỗ trong P đoạn, C chương"; "Thay trong chương cũ" (khoá khi đang dịch) → rà lại.
- GUI nhóm xưng hô: cặp đã có mốc trong bảng nhân vật mà khác giá trị glossary thì ghi "Đang dùng: em–anh (từ ch.150, mốc trong bảng nhân vật)". Trang truyện nạp cast.json (nạp lại sau reset); `lib/cast-addressing.ts`: `chapterLabel`, `latestAddressing`.

**Kiểm.** Core `tests/backfill.rs` (quét, chương lệch đoạn, xem trước không ghi, chỉ thay đoạn có khoá + giữ CRLF, không thay chồng + chạy lại không đổi); Tauri: test lệnh + test "lệnh nặng phải async" thêm 2 lệnh mới (crate Tauri chỉ kiểm được bằng clippy trên máy này — test binary lỗi STATUS_ENTRYPOINT_NOT_FOUND như trước); GUI: test hộp thoại (rà → xem trước → thay) và nhãn "Đang dùng", test lib. Thử trên dữ liệu thật (bắt đầu hộ tống): 陆云停 → 4 đoạn thiếu "Lục Vân Đình", đều viết "Lục Đình Vân". Core 210 xanh (+1 test chập chờn cũ), GUI 244, clippy không cảnh báo mới.

## 27. 招揽 → "chiêu lãm" (ngoài lộ trình)

**Vấn đề.** Chủ app báo "chiêu lãm" — âm Hán-Việt của 招揽, tiếng Việt không dùng. Quét thư viện (đã chuẩn hoá Unicode): 2 chỗ thật ở Bắt đầu hộ tống ch.0285; 2 chỗ "chiêu Lãm Tước Vĩ" ở Thâu hương cao thủ là tên chiêu Thái Cực Quyền (揽雀尾), đúng. Trường sinh là mèo trong thư viện này không có chữ nào.

**Quyết định (chủ app chốt 11/10/2026).** Thêm vào bảng convert, bắt soát, sửa tay 2 chỗ cũ.

**Thay đổi.**

- Prompt, bảng "Dấu hiệu câu vẫn còn là convert": `招揽 | chiêu mộ / chiêu nạp (người về làm việc), lôi kéo (về phe), kéo khách / mời chào (buôn bán) | chiêu lãm`. Hash base mới `eec36fac8b773f8d`.
- Luật check bắt soát (TS `CHECK_RULES`, Rust `DEFAULT_RULES`, mọi bối cảnh): `[Cc]hiêu lãm` với "lãm" viết thường — "Lãm" viết hoa là tên chiêu, không bị bắt. Golden: thêm dòng mẫu có cả hai.
- Dữ liệu: Bắt đầu hộ tống ch.0285 — "thử chiêu lãm hắn tới Đông Cung" → "thử lôi kéo hắn về Đông Cung"; "thật sự có thể chiêu lãm được sao" → "thật sự có thể lôi kéo được sao".

**Kiểm.** Golden: dòng mẫu chỉ bị bắt một lần (Chiêu lãm), tên chiêu không bị bắt. qt-web 291, qt-ai-cli 43, core 210 xanh (+1 test chập chờn cũ).

## 28. 人族 → "người tộc", 空前绝后 → "không tiền khoáng hậu" (ngoài lộ trình)

**Vấn đề.** Người dùng khác báo: 人族 có lúc dịch "người tộc" (phải là Nhân tộc); thành ngữ 空前绝后 ra "không tiền khoáng hậu" (phải là "vô tiền khoáng hậu" — ghép nhầm âm 空 "không"). Ghi chú: thư viện chủ app chỉ dùng `books/common`; `fanqie`, `qidian`, `sfacg` không còn dùng, không đụng.

**Đo trên `common`.** "người tộc" 47 chỗ nhưng chỉ ~10 chỗ raw cùng đoạn có 人族; còn lại đúng (哈萨克族人 → "người tộc Cáp Tát Khắc", 我族之人 → "người tộc ta"). "Nhân tộc" đúng 2.995 chỗ. "không tiền khoáng hậu" 0 chỗ ("vô tiền khoáng hậu" 12 chỗ, đúng).

**Quyết định (chủ app chốt 11/10/2026).** Thêm vào prompt, check; sửa chỗ cũ.

**Thay đổi.**

- Prompt, bảng convert: `人族 | Nhân tộc | Người tộc`; `空前绝后 / 前无古人后无来者 | vô tiền khoáng hậu | không tiền khoáng hậu`. Hash base mới `533b87dfe4e7d3c0`.
- Check: "không tiền khoáng hậu" bắt soát (luật thường, mọi bối cảnh). "người tộc" là bẫy đồng âm theo cặp đoạn (mục 18): raw có 人族 đứng riêng — không sau 巨/鱼/矮/兽/半/蛇/狼/羽/魔/亚 (海巨人族 → "bốn người tộc Hải Cự Nhân" là đúng, "trên người tộc lão…" thì "người" là thân thể) — mà bản dịch có "người tộc" → bắt soát. Golden: thêm ca 人族 (bắt), 我族之人 và 海巨人族 (không bắt), dòng mẫu có cả "không tiền khoáng hậu" và "vô tiền khoáng hậu".
- Dữ liệu: 6 chỗ "người tộc" → "nhân tộc" (giữ hoa/thường) ở 4 file (Bài poker ch.48, Mỗi ngày một môn thần thông ch.919, 924, Tiên tử lại xu long ch.549). Lần chạy thử đầu dùng luật lỏng bắt nhầm 4 chỗ tộc Hải Cự Nhân ở Hợp hoan tông → siết luật trước khi ghi. Sao lưu ngoài repo (scratchpad `q28_backup/`).

**Kiểm.** Golden khớp; qt-web 291, qt-ai-cli 43, core 210 xanh (+1 test chập chờn cũ). Chạy lại script sửa dữ liệu không còn gì để sửa.

## 29. Trang Tìm & thay (ngoài lộ trình)

**Vấn đề.** Truyện nhiều chương dịch sai một từ thì phải tìm và thay tay từng file; "Rà chương cũ" (mục 26) chỉ đi theo một mục glossary.

**Quyết định (chủ app chốt 12/10/2026).** Trang riêng trên rail (Ctrl+Shift+F); không regex; hoàn tác lần thay gần nhất.

**Thay đổi.**

- Core `replace.rs`: `find_in_story` (nguyên từ, hoa/thường, chỉ đoạn raw có X, phạm vi chương; trả tổng + số theo chương đủ + tối đa 500 chỗ kèm 50 ký tự ngữ cảnh mỗi bên và raw của đoạn), `replace_in_story` (thay mọi chỗ trừ chương / chỗ bỏ chọn, giữ kiểu xuống dòng, ghi `work/replace-undo.json` trước khi sửa chương), `undo_info`, `undo_last_replace` (bỏ qua chương đã bị sửa tiếp). Nhanh: lọc cả chương bằng tìm chuỗi đã gập hoa/thường trước khi so từng ký tự, chỉ đọc raw khi lọc theo raw hoặc cần ngữ cảnh, quét song song tối đa 8 luồng. Đo trên Hợp hoan tông (1.523 chương, bản release): 750 ms → khoảng 120 ms với từ gặp ở mọi chương, 45 ms với từ hiếm.
- Tauri `replace_cmds.rs`: `replace_find`, `replace_apply`, `replace_undo_info`, `replace_undo` — async + blocking; thay / hoàn tác từ chối khi truyện đang dịch.
- GUI `pages/replace-page.tsx`: ô Tìm / Thay bằng, Nguyên từ (bật sẵn), Phân biệt hoa/thường, Chỉ đoạn raw có, phạm vi chương (số thứ tự như trang Export); gõ xong 0,3 giây mới tìm; kết quả nhóm theo chương (mở sẵn 10 chương đầu), mỗi chỗ hiện câu xung quanh, chữ cũ gạch / chữ mới, raw; tick từng chỗ / cả chương; bấm vào chỗ → mở chương ở trang Dịch; sau khi thay nhắc sửa glossary nếu là tên riêng / thuật ngữ; thanh hoàn tác lần thay gần nhất. README qt-ai-gui có mục "Tìm & thay".

**Kiểm.** Core `tests/replace.rs` (nguyên từ / hoa thường / ngữ cảnh + raw; lọc raw, phạm vi, cắt số chỗ; thay trừ chỗ / chương bỏ chọn + giữ CRLF; hoàn tác bỏ qua chương đã sửa tiếp; không khớp thì không ghi). Tauri: test lệnh + danh sách lệnh nặng (chỉ kiểm được bằng clippy trên máy này). GUI `replace-page.test.tsx` (tìm → bỏ chọn chỗ → thay; bỏ chọn cả chương; mở chương; hoàn tác). Core 215, GUI 249 xanh, clippy không cảnh báo mới.

## 30. Truyện hiện đại / hỗn hợp để tên riêng hỗn hợp (ngoài lộ trình)

**Vấn đề.** Chủ app: truyện bối cảnh hiện đại hoặc xuyên qua lại thường để tên riêng **hỗn hợp** (họ Trung Quốc vẫn phiên Hán-Việt, tên nước ngoài giữ dạng gốc). Thư viện đã gần như theo lệ đó (hiện đại 23 hỗn hợp / 1 Hán-Việt; hỗn hợp 2 / 3) nhưng bước dựng truyện chỉ dặn "Trung Quốc → han, phương Tây → foreign, lẫn → mixed".

**Quyết định (chủ app chốt 12/10/2026).** Bước dựng truyện mặc định hỗn hợp cho hiện đại / hỗn hợp; GUI tự chuyển khi đổi bối cảnh; đổi 4 truyện cũ.

**Thay đổi.**

- `api_fill.rs`: mẫu JSON cho model ghi rõ "setting modern hoặc mixed thì mặc định mixed; foreign chỉ khi bối cảnh hẳn phương Tây/Nhật/Hàn; han cho truyện cổ đại". `merge_fill`: setting modern/mixed mà names `han` hoặc bỏ trống → `mixed` (model chọn `foreign` thì giữ). Workflow agy `setup-story.md` cùng lời dặn.
- GUI Hồ sơ truyện: đổi Bối cảnh sang Hiện đại / Hỗn hợp mà Tên riêng đang Hán-Việt thì tự chuyển sang Hỗn hợp; đổi tay lại được; về Cổ đại không tự đổi. Gợi ý bối cảnh Cổ đại bỏ "Ân" cũ (mục 1) → "thán từ A?/Hả?/Ừm".
- Dữ liệu: `genre.names` han → mixed cho Giải trí ta đàn nhạc cũng là chủ xướng (hiện đại), Sư muội ta nhìn thế nào…, Xuyên nhanh pháo hôi…, Xuyên nhanh tuyệt sắc… (hỗn hợp). Sao lưu `story.json.truoc-ten-hon-hop-20261012.bak`; mỗi file chỉ đổi đúng một dòng.

**Kiểm.** Test `api_fill`: modern+han / mixed thiếu names → mixed, modern+foreign giữ, ancient+han giữ. GUI: đổi bối cảnh → Tên riêng Hỗn hợp, đổi tay về Hán-Việt rồi về Cổ đại giữ Hán-Việt. GUI 249 xanh, clippy không cảnh báo mới.

## 31. Tham khảo bảng hậu tố của QuickTranslator (`QT2025/Resources/HauTu.txt`) (ngoài lộ trình)

**Đánh giá.** File là bảng ~380 hậu tố danh xưng (chức danh, quan hệ sau tên) QT dùng để ghép "张先生" → "Trương tiên sinh". Cách dịch trong file là văn convert, có chỗ sai (老师 lão sư, 同学 đồng học, 教练 giáo luyện, 妈妈 mụ mụ, 博士 bác sĩ, 厅长 giám đốc) và dòng rác riêng một truyện (琳阿姨, 淋小姐, 临字秘) — không lấy. Đáng lấy là **danh sách hậu tố**:

1. Chặn danh xưng làm tên riêng trong glossary (mục 21): thêm hậu tố một chữ thì bắt nhầm tên thật (叶二娘, 鸣卿, 王母, 岳老三); chỉ hậu tố nhiều chữ thì bắt thêm ~200 mục. **Chủ app không chọn.**
2. Nhận ra nhân vật được gọi bằng họ + chức danh (`NEUTRAL_TITLES`, 30 mục gần như toàn hiện đại). **Làm.**
3. Chức danh hành chính hiện đại trong prompt — đo trên truyện hiện đại / hỗn hợp: model đã đúng phần lớn (班长 → lớp trưởng 431, 博士 → tiến sĩ 221, 市长 → thị trưởng 45) nhưng 科长 → "khoa trưởng" 156 (trưởng phòng 45), 处长 → "xử trưởng" 12, 县长 → "huyện trưởng" 14, 镇长 → "trấn trưởng", 厅长 → "sảnh trưởng". **Làm.**

**Thay đổi.**

- `cast.rs` `NEUTRAL_TITLES` thêm 50 chức danh không mang giới từ HauTu (掌门, 宗主, 帮主, 堂主, 盟主, 教主, 城主, 庄主, 阁主, 殿主, 门主, 寨主, 舵主, 坛主, 道友, 道长, 真人, 大侠, 少侠, 大师, 将军, 军师, 统领, 护法, 总管, 管事, 首领, 村长, 镇长, 县令, 知县, 知府, 书记, 会长, 团长, 营长, 所长, 处长, 科长, 厂长, 部长, 市长, 县长, 省长, 司令, 教官, 总裁, 董事长, 站长, 馆长): raw gọi "林掌门", "宋少侠" thì nhân vật 林X / 宋X đã rõ giới vào danh sách nhân vật có mặt trong prompt; check gọi sai giới theo họ + chức danh cũng áp cho các chức danh này. Không thêm chức danh mang giới (公子, 小姐, 夫人, 公公, 嬷嬷, 王爷, 贵妃).
- Prompt bảng thuật ngữ đô thị (hiện đại, hỗn hợp): 县长 / 乡长 / 省长 / 镇长 → chủ tịch huyện / xã / tỉnh / thị trấn; 处长 / 科长 → trưởng phòng (科长 ở bệnh viện, trường học → trưởng khoa); 厅长 → giám đốc sở; 副 + chức danh → phó X hoặc dạng quen (đội phó, cục phó, lớp phó, phó viện trưởng). Base cổ đại không đổi (hash giữ nguyên).

**Kiểm.** Test `nhan_ra_nhan_vat_goi_bang_ho_va_chuc_danh_co_trang` (林掌门, 宋少侠). Golden khớp; qt-web 291, core 217 xanh, clippy không cảnh báo mới.

## 32. Lỗi: kết quả "AI điền hồ sơ" của truyện này bị Áp dụng vào truyện khác (ngoài lộ trình)

**Hiện tượng.** Sau khi reset, thư viện hiện hai truyện cùng tên "Livestream Giả Gái…" (một ở thư mục `lao-ba-qua-manh…`). story.json của Lão Bà bị ghi lúc 03:30 12/10 bằng hồ sơ AI điền của Livestream (tóm tắt Chu Thạch, link fanqie và 9 tên của Livestream; không tên nào có trong raw Lão Bà); Livestream thì không nhận được hồ sơ đó. Chưa chương nào bị dịch với hồ sơ sai (638 chương đều queued).

**Nguyên nhân.** Hộp thoại AI điền luôn được dựng trong trang Hồ sơ truyện; ô tên, link và kết quả lần chạy trước là state khởi tạo một lần nên còn nguyên khi chuyển truyện. Bấm Áp dụng ghi vào truyện đang mở (`currentRoot`), không phải truyện đã chạy AI. Trang Tìm & thay cùng loại rủi ro: ~0,3 giây trước khi tìm lại vẫn hiện kết quả truyện cũ.

**Sửa.**

- `story-page.tsx`: nhớ truyện đã mở hộp thoại (`fillRoot`); đổi truyện thì đóng hẳn; chỉ dựng hộp thoại khi mở, `key` theo truyện — mở lại ở truyện mới là hộp thoại mới, không mang kết quả cũ.
- `app.tsx`: trang hiện tại dựng lại khi đổi truyện (`key={root}`) — kết quả tìm, hộp thoại, state cục bộ của truyện trước không sang truyện mới.
- Dữ liệu: khôi phục story.json Lão Bà từ `story.json.bak` (bản ngay trước lần lưu nhầm — hồ sơ sau reset: tên + link, hiện đại / hỗn hợp, glossary trống); bản bị ghi nhầm giữ ở `story.json.ghi-nham-tu-livestream-20261012.bak`.

**Kiểm.** Test story-page: chạy AI điền ở truyện A, chuyển sang B → hộp thoại đóng, không gọi lưu; mở lại ở B thì tên là của B và Áp dụng bị khoá. GUI 250 xanh, typecheck và lint sạch.

---

## Ghi chú kiểm thử

- `cancel_giet_agy_ngay_va_don_lock` (core `tests/session.rs`) đỏ khi chạy cả binary session trong chiều 10/10/2026, kể cả trên commit `018c38f` chưa có thay đổi nào ở đây; chạy riêng thì xanh. Lệnh huỷ mất hơn 5 giây khi máy bận (`taskkill`). Không liên quan prompt/check.
