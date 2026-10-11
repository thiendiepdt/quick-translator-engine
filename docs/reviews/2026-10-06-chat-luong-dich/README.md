# Thẩm định chất lượng bản dịch qt-ai (06/10/2026)

Đánh giá bản dịch AI của 61 truyện trong `bookshelf/books/common`, dịch bằng qt-ai-gui (bản 1.0.15 trở về trước). Mục tiêu: tìm điểm yếu hệ thống của prompt, glossary, cast và bước check để sửa app, không phải chấm từng chương.

Bản đọc nhanh: mở [tham-dinh-ban-dich.html](tham-dinh-ban-dich.html) bằng trình duyệt. Trang có tóm tắt, lộ trình sửa P0/P1/P2, 4 chương phân tích và bảng điểm 61 truyện (bấm tên truyện để xem 3 lỗi nặng nhất).

## Cấu trúc thư mục

| Đường dẫn | Nội dung |
|---|---|
| `tham-dinh-ban-dich.html` | Bản đánh giá hoàn chỉnh, một trang |
| `tien-trinh-sua.md` | Sổ ghi từng mục đã sửa theo lộ trình: quyết định, thay đổi, kiểm thử |
| `nghi-sai-am.md` | Tên glossary nghi sai âm Hán-Việt (703 tên, 57 truyện) để soát tay |
| `tong-hop/A_xungho.md` | Chương 1: xưng hô, vai vế, đại từ, setting (194 lỗi) |
| `tong-hop/B_glossary.md` | Chương 2: glossary và thuật ngữ (215 lỗi) |
| `tong-hop/C_van.md` | Chương 3: calque, thán từ, meme, văn khô (313 lỗi) |
| `tong-hop/D_nghia_dinhdang.md` | Chương 4: sai nghĩa, con số, định dạng, toàn vẹn raw↔out (177 lỗi) |
| `tong-hop/*-danh-sach-loi.md` | Toàn bộ lỗi của từng nhóm, mỗi dòng một lỗi: thẻ, mức, truyện, chương:dòng, raw → bản dịch ⇒ đề xuất |
| `tong-hop/goi-y-va-lam-tot.md` | Gợi ý sửa và điểm làm tốt của từng truyện |
| `truyen/<slug>.md` | Báo cáo đọc được của từng truyện |
| `truyen/<slug>.json` | Cùng nội dung dạng có cấu trúc: điểm, lỗi gắn thẻ, gợi ý |
| `phuong-phap/` | Hướng dẫn đã giao cho sub agent review và tổng hợp, dùng lại khi chạy đợt sau |

## Kết quả chính

- 61 truyện, 250 chương đọc đối chiếu từng đoạn với raw, 27.066 chương quét bằng regex, 899 lỗi gắn thẻ.
- Điểm trung bình 6,96/10: dễ hiểu 7,50 · giọng văn 7,34 · sinh động 6,99 · xưng hô 6,74 · thuật ngữ 6,23.
- Phần lớn lỗi lặp do chính app bảo model làm:
  - Prompt cổ đại bắt 嗯 → "Ân", còn `check.rs:84` ép "Ừm" quay về "Ân" (3.383 dòng cổ đại, 0 dòng hiện đại).
  - `check.rs:132` ép mọi "tôi" ngoài ngoặc kép thành "ta" (đạn mạc, 【】, tiêu đề); `check.rs:72` làm kẻ dưới xưng "ta".
  - `check.rs:68-69` cấm "vợ/chồng/phụ nữ" ở truyện `ancient`, nên truyện hiện đại bị đặt sai setting toàn "thê tử", "nữ nhân".
  - Prompt dạy sai: 舔狗 = "liếm chó", ví dụ "bản thân hắn" cho 自己, bảng "Tránh → Dùng thay" ép "kiểm trắc".
- Xưng hô đóng băng ở lần gặp đầu: 38/60 truyện dính lỗi chưa từng có mốc đổi xưng hô.
- Glossary tự sinh chốt âm Hán Việt sai, tách một người thành hai khi tác giả gõ nhầm, và không bao giờ chốt thuật ngữ chung (nhóm `common` chỉ 0,18% mục tự sinh).
- Raw bị thay sau khi dịch mà state vẫn "done" (phuong-thon-dao-chu, truong-sinh-la-meo).

Nguồn thật của prompt là `apps/qt-web/src/lib/ai-translation-prompt.ts`; `crates/qt-ai-core/prompts/prompts.json` sinh ra bằng `npm run golden`. Luật check gốc ở `apps/qt-web/src/lib/ai-translation.ts`, chép y hệt sang `crates/qt-ai-core/src/check.rs`. Số dòng trích trong báo cáo đúng tại commit `ad912ad`.

## Cần chủ app quyết trước khi sửa

- Dấu lửng: giữ `......` (prompt và check đang cố ý đổi `……` thành vậy) hay chuyển sang `…`?
- 老子: "lão tử" hay "ông đây"?
- Sửa hàng loạt bản dịch cũ (Ân, tên sai âm, ngoặc thẳng) hay chỉ áp cho chương dịch từ nay?
- Crawler: đổi sang id ổn định theo số chương thay vì số thứ tự file.

## Đính chính so với báo cáo từng truyện

- Tiêu đề viết hoa mọi chữ (moi-ngay ch.1043–1045 và nhiều truyện khác) do model, không do commit `cd752eb`. `titles.json` chỉ ghi lại dòng đầu đã dịch.
- Chữ "ta" trong tiêu đề là câu thoại (tan-the-nguoi-khac, sao-the-tinh-dich) do luật người kể "ta" và `check.rs:132` chạy cả trên dòng tiêu đề, không phải do `titles.json` dịch tách rời.
- "story.json và cast.json cùng nạp vào prompt nên mâu thuẫn" không đúng với code hiện tại: prompt chỉ in cặp đang hiệu lực, mốc trong cast thắng (`prompt.rs:223`). Mâu thuẫn thật nằm giữa `style.toneRules` và cặp addressing.

- 佟: báo cáo truyện moi-ngay và bản tổng hợp B ghi "佟 phải đọc Đồng, prompt dạy sai `佟 Đông`". Sai: từ điển Thi Viện ghi "(Danh) Họ Đông", Wikipedia tiếng Việt viết Đông Quốc Duy, Đông Giai thị. Dòng prompt đúng, giữ nguyên; "Đông Trọng Thu" là bản dịch đúng.

- Âm Hán-Việt "sai" phụ thuộc từ điển: 童绮 "Đồng Khởi" (ký túc xá) khớp Lạc Việt (绮 = khởi) dù Thi Viện ghi khỉ/ỷ; 鼓浪屿 "Cổ Lãng Tự" (nữ thần điện tử) khớp Thi Viện (屿 = tự) dù Lạc Việt ghi dữ. Hai kết luận "phiên âm sai" này trong báo cáo truyện không chắc.

## Phương pháp và giới hạn

- Mỗi truyện một sub agent: đọc trọn 4 chương (chương đầu, một chương giữa, 2 chương dịch mới nhất), đối chiếu từng đoạn với raw, grep toàn bộ out/ để đếm lỗi lặp, ghi lỗi theo bộ thẻ chung. Sau đó 4 sub agent tổng hợp theo nhóm và truy nguyên vào code; các nguyên nhân chính được kiểm lại trực tiếp trong code và prompt.
- Điểm do model chấm, chỉ dùng để so sánh tương đối giữa các truyện.
- Số đếm bằng regex là ước lượng, có thể lẫn một ít cách dùng đúng (vd "đem lòng").
- Chưa chạy thử A/B để đo hiệu quả từng đề xuất.
- Bản dịch được đánh giá ở trạng thái 06/10/2026 khoảng 07:40.
