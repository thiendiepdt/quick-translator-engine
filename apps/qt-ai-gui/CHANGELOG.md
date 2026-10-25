# Có gì mới

## Chưa phát hành
- Truyện cổ đại gọi trẻ con theo cách các bản dịch kiếm hiệp vẫn dùng: “thằng bé”, “con bé”, đứa trẻ mới xuất hiện là “một đứa bé trai” / “một đứa bé gái”, 小姑娘 là “tiểu cô nương”, 女孩子 nói chung là “con gái”; không còn “cậu bé”, “cô bé”, “nam hài”, “nữ hài”. Chỗ còn sót được bắt khi kiểm tra chương để bước soát sửa lại. Truyện hỗn hợp chọn theo cảnh: cảnh cổ đại như trên, cảnh hiện đại vẫn “cậu bé”, “cô bé”.
- Thêm nút **Phồn → Giản…** ở trang Dịch: chuyển raw phồn thể (và file GBK, Big5) sang giản thể UTF-8 để glossary và tên riêng khớp, bản gốc giữ ở folder `raw-phon-the/`, hoàn tác được. Khoá chữ Hán trong glossary, bảng nhân vật, sổ tiêu đề đổi theo; chương vốn là giản thể giữ nguyên. Mở truyện có raw phồn thể thì app nhắc ngay dưới thanh công cụ. Bộ chuyển đổi theo cụm từ nên giữ đúng 著名, 乾隆, tên riêng như 大乾王朝, và đổi cả từ vựng Đài Loan sang Đại lục (滑鼠 → 鼠标).

## 1.0.27 — 2026-10-24
- Sửa lỗi đóng app khi đang dịch làm agy chạy ngầm tiếp (vẫn dịch, ăn quota, giữ folder truyện không xoá được): bấm đóng khi đang dịch giờ hỏi trước rồi dừng hết phiên; app bị tắt đột ngột (crash, Task Manager) thì agy cũng dừng theo.
- Thêm kiểm tra chữ tiếng Việt viết sai khi chốt chương: bắt các chữ không thể có trong tiếng Việt do AI sinh hỏng (kiểu “mồhôi”, “pháthuy”, “tươnglai”) rồi đưa vào bước soát để AI sửa lại; chữ kéo dài, nói lắp hay tiếng lóng cố ý trong lời thoại vẫn được giữ.
- Cải thiện bước soát khi dịch bằng API key: AI chỉ trả lại những đoạn cần sửa thay vì in lại cả chương, nên tốn ít token hơn, soát nhanh hơn và không sửa lan sang đoạn khác; AI cũng được xem nguyên văn tiếng Trung của đoạn bị nêu để sửa cho đúng.
- Sửa lỗi xưng hô chỉ dùng trong một cảnh (cải trang, đóng kịch, nói trước mặt người ngoài, hồi tưởng, kiếp trước, mỉa mai) bị lưu thành cặp xưng hô mới và dính sang các chương sau; app giờ bỏ những lần đổi cặp kiểu này, còn đổi vì quan hệ thật (thành người yêu, bái sư, trở mặt, hết đóng kịch) vẫn nhận như cũ.
- Cải thiện văn dịch: AI được dặn bỏ thêm 5 cấu trúc văn convert (“đối với X mà nói”, “đem ánh mắt”, “tiến hành một phen”, “một loại cảm giác”, “lộ ra thần sắc”); bốn cấu trúc sau còn sót thì chương được ghi cảnh báo khi chốt.
- Cải thiện văn dịch truyện cổ đại và hỗn hợp: 气息 tỏa ra từ người, vật, cảnh giới được dịch là “khí tức” thay vì “hơi thở”; chỉ khi nhân vật thở thật mới dùng “hơi thở”.
- Cải thiện xưng hô họ hàng trong truyện hiện đại: thống nhất theo quy ước cha / mẹ (cha vợ, mẹ vợ, cha chồng, mẹ chồng, cha dượng, mẹ kế, anh rể, chị dâu…) thay vì lẫn “bố vợ”, “ba vợ”, “nhạc phụ” trong cùng một truyện; còn sót “bố vợ / bố chồng / bố dượng” thì chương được ghi cảnh báo khi chốt.
- Thống nhất tự xưng 本 đọc “bổn” (bổn cung, bổn vương, bổn tọa, bổn tiểu thư, bổn cô nương…) thay vì lẫn “bản”/“bổn” trong cùng một truyện: AI được dặn trước, chỗ còn sót app tự sửa khi chốt chương (không đụng “bản tôn” chỉ chân thân, “văn bản”, “phiên bản”, tên Nhật như Cung Bản); từ điển của truyện ghi “bản …” thì vẫn theo từ điển.
- Cải thiện dịch thành ngữ và thơ: ưu tiên thành ngữ tiếng Việt tương đương (打草惊蛇 → “đánh rắn động cỏ”), rồi diễn đạt tự nhiên, vẫn giữ thành ngữ Hán-Việt quen thuộc (“danh chính ngôn thuận”, “tâm phục khẩu phục”); thơ, câu đối giữ ý, giữ số câu, không sáng tác thêm.
- Thống nhất vài thuật ngữ ngôn tình, cung đình hay bị dịch lẫn: 白月光 → “bạch nguyệt quang”, 白莲花 → “bạch liên hoa”, 朋友圈 → “vòng bạn bè”, 赐婚 → “ban hôn”.
- Giọng văn ngôn tình: AI được dặn không đẩy nhanh tình cảm (thiện cảm chưa phải yêu) và không thêm cử chỉ thân mật, từ ngọt mà nguyên tác không có.
- Lời kể: nhân vật nữ phản diện, kẻ đáng khinh được gọi “ả” / “mụ” khi nguyên tác lộ giọng khinh (nữ chính vẫn “nàng” / “cô”); không dùng “chàng” làm đại từ ngôi ba trong lời kể (vẫn dùng trong thoại và trong “chàng trai”, “anh chàng”). Đại từ ghi riêng trong bảng nhân vật vẫn được ưu tiên.
- AI điền hồ sơ truyện theo các quy ước mới: luật giọng ghi tự xưng “bổn” (bổn tọa, bổn cung…), truyện hiện đại gọi cha mẹ là “cha”/“mẹ”, không đặt luật dùng “chàng” trong lời kể; luật giọng và cặp xưng hô AI đề xuất còn ghi “bản tọa”, “bản cung” thì app tự đổi sang “bổn”.
- Dịch “đường” / “phố” theo cách nói miền Nam: chỗ đi lại dùng “đường” (trên đường, góc đường, đầu đường, ra đường), còn “phố” dành cho nơi ở, buôn bán (khu phố, phố xá, phố ẩm thực, dạo phố, thành phố); tên đường gọi “đường X”. Truyện cổ đại chỉ giữ “phố xá”, “phố phường”: 街 → “đường”, 街市 → “chợ”, 逛街 → “dạo chơi”.
- Truyện cổ đại: lời kể dùng “mẹ”, “cha mẹ”, và các từ “mẹ ruột”, “mẹ đẻ”, “mẹ kế”, “cha ruột”, “cha dượng” thay vì “nương ruột”, “nương kế”; trong thoại con vẫn gọi mẹ là “nương”.
- AI điền hồ sơ không còn đưa từ thường vào từ điển của truyện (như 入赘 thành “nhập chuế” thay vì “ở rể”), không ghi luật cấm chữ “mẹ” cho truyện cổ đại, và bỏ cặp xưng hô sai định dạng.
- **Tìm & thay** tìm cả trong hồ sơ truyện, kết quả chia tab **Chương / Hồ sơ / Nhân vật / Tiêu đề chương**: đổi tên một nhân vật thì sửa luôn glossary, cặp xưng hô, tóm tắt, tone rules, mốc đổi xưng hô và sổ tiêu đề chương, để các chương dịch sau không quay về tên cũ. Mọi chỗ được tick sẵn, bỏ tick chỗ nào không muốn thay; một lần **Hoàn tác** trả lại cả chương lẫn hồ sơ.

## 1.0.26 — 2026-10-17
- Thêm nút **Đang dịch #N** trên thanh trang Dịch: bấm để cuộn tới và mở ngay chương đang dịch.
- Thay nút **AI xét** của trang Kiểm tra VN bằng nút **AI giải thích** trên từng chỗ: AI dịch nhanh đoạn raw sang tiếng Việt (khi chương chưa dịch), diễn giải đoạn đó nói gì, có nhạy cảm không, và gợi ý cách xử lý; kết quả được lưu lại, mở lại vẫn thấy.
- Thêm trang **Kiểm tra VN** riêng trên thanh bên trái (dưới **Hồ sơ truyện**) để dễ tìm; mục cũ trong Hồ sơ truyện vẫn giữ.
- Thêm nút **Quét nhạy cảm VN** ở thư viện (màn chọn truyện): quét mọi truyện chạy nền, ít luồng để máy không giật, hiện tiến độ trên nút và bấm lại để dừng; lần sau chỉ đọc chương mới hoặc đã sửa nên gần như tức thì. Quét xong huy hiệu **⚠ VN** của từng truyện cập nhật luôn.
- Sửa lỗi hộp **Dịch lại nhiều chương** bị che mất nút **Reset cả truyện…** khi hàng nút quá dài; các nút giờ chia thành hai hàng.
- Sửa lỗi chữ bị hỏng mã hoá (kiểu “ChÆ°Æ¡ng”) khi API trung gian trả về sai: app tự sửa lại, không sửa được thì gọi lại API (3 lần vẫn lỗi thì báo lỗi chương đó rồi dịch tiếp chương sau), và không chốt chương còn chữ lỗi (đoạn lỗi được dịch lại).

## 1.0.24 — 2026-10-16
- Sửa lỗi dịch bằng API key bỏ qua chương hoặc dừng phiên khi nhà cung cấp quá tải, hết hạn mức theo phút hay rớt mạng: app tự gọi lại đúng bước đang lỗi, chờ 5 → 10 → 20 → 40 → 60 giây rồi cứ 1 phút thử một lần tới khi API hoạt động lại; bấm **Dừng** để ngừng chờ. Lỗi ở bước soát không còn làm dịch lại cả chương.
- Sửa lỗi chương bị bỏ qua hoặc lỗi nằm sau chương dịch xong cuối cùng (thường gặp khi phiên dừng vì lỗi API) không hiện trong cảnh báo chương hổng ở trang Dịch; giờ có trong danh sách và trong nút **Dịch lại cả N**.

## 1.0.23 — 2026-10-15
- Bỏ chặn bắt đầu dịch khi tiến độ lệch với bản dịch đã có: trang Dịch chỉ hiện thông báo kèm nút **Đồng bộ…**; bấm Dịch khi chưa đồng bộ thì các chương lệch được dịch lại, ghi đè bản dịch đang có.
- Cải thiện chuyển truyện ở cột **Phiên này**: quay lại truyện đã xem thì trang Dịch mở lại đúng chương, bộ lọc và ô tìm trước đó.
- Sửa lỗi truyện cổ đại xưng "anh–em", "tôi–cậu", "tao–mày" trong lời thoại: anh chị em ruột gọi nhau "ca", "tỷ", tự xưng "muội", "đệ"; AI điền hồ sơ không còn ghi luật xưng hô kiểu hiện đại cho truyện cổ đại, và app không học cặp xưng hô kiểu hiện đại ở truyện cổ đại.

## 1.0.22 — 2026-10-15
- Thêm kiểm tra nội dung nhạy cảm về Việt Nam: app tự quét chương gốc và bản dịch khi mở truyện, quét chương mới hay chốt chương, rồi đánh dấu những chỗ nhắc tới Việt Nam cần xem lại (chỉ cảnh báo, không chặn dịch hay export).
- Thêm tab **Kiểm tra VN** trong trang Truyện để lọc và xem từng chỗ kèm ngữ cảnh, đánh dấu **Không sao** hoặc **Cần xử lý**, và bấm **AI xét** để AI sơ loại trước khi bạn tự xem.
- Thêm **Thay thế…** cho chỗ nhạy cảm VN theo từng chỗ, theo chương hoặc cả truyện; thay nhiều cụm trong một lượt với cụm thay gợi ý sẵn, và hoàn tác cả lượt ở trang **Tìm & thay**.
- Thêm huy hiệu **⚠ VN** ở màn chọn truyện cho truyện còn chỗ nhạy cảm về Việt Nam chưa xác minh.
- Thêm thẻ **Kiểm tra nội dung nhạy cảm về Việt Nam** trong **Cài đặt** để bật/tắt từng mục từ điển có sẵn, thêm mục riêng và sửa bảng cụm thay gợi ý theo bối cảnh cổ đại hoặc hiện đại.
- Thêm export EPUB ở trang **Export**: sách có bìa (tự tải từ link nguồn Fanqie, QQ đọc sách, Qidian, Qimao, SFACG, Tấn Giang hoặc bìa mặc định vẽ tên truyện), trang tiêu đề, trang mục lục bấm tới từng chương, tên tác giả chữ Hán được phiên sang Hán-Việt.
- Thêm mục **Có gì mới** trong **Cài đặt**; sau mỗi lần cập nhật app tự hiện những thay đổi của các bản mới, và hộp hỏi cập nhật cũng kèm danh sách thay đổi.
- Thêm **Đồng bộ tiến độ…** cho truyện đồng bộ qua Google Drive giữa nhiều máy: khi tiến độ lệch với bản dịch đã có, trang Dịch hiện thông báo và chặn bắt đầu dịch cho tới khi bạn đánh dấu các chương đó là đã dịch hoặc chọn **Dịch lại các chương này**.
- Sửa lỗi truyện cổ đại còn lọt "chúng tôi", "nhà tôi" trong lời thoại; bộ kiểm tra bản dịch giờ bắt "tôi" trong thoại để đưa vào soát lại.

## 1.0.21 — 2026-10-13
- Sửa lỗi cặp xưng hô không được ghi lại ở những chương không có tên riêng mới; app giờ nhắc khai cặp xưng hô ở mọi chương và nêu tên nhân vật có mặt chưa có cặp, nên bảng xưng hô học đầy đủ hơn.

## 1.0.20 — 2026-10-12
- Cải thiện prompt dịch: lời kể ngôi ba không còn dùng "cậu" hay "cậu ta", nam là "hắn", nữ là "nàng" hoặc "cô", trẻ nhỏ mặc định là "nó", mỗi nhân vật giữ một đại từ suốt chương.
- Cải thiện tab **Nhân vật** với gợi ý ghi đại từ lời kể "cậu" cho thiếu niên, bé trai trong truyện học đường khi muốn giữ cách gọi này.

## 1.0.19 — 2026-10-12
- Cải thiện truyện bối cảnh hỗn hợp: nhân vật tự xưng "ta" trong lời thoại ở mọi cảnh, hạn chế "tôi"; bộ kiểm tra bản dịch bắt "tôi" trong thoại để đưa vào soát.
- Cải thiện xưng hô gia đình: truyện cổ đại dùng cha, nương, phụ thân, mẫu thân; truyện hiện đại dùng cha, cha mẹ thay cho "bố"; truyện đã chốt cách gọi trong glossary thì giữ nguyên.
- Cải thiện cách dùng "tớ–cậu": chỉ dành cho học sinh, sinh viên, bạn nhỏ tuổi; người lớn ngang hàng xưng "tôi", bạn thân người lớn xưng "tao–mày".

## 1.0.18 — 2026-10-12
- Sửa lỗi kết quả **AI điền hồ sơ** của truyện trước bị áp vào truyện đang mở sau khi chuyển truyện.
- Sửa lỗi kết quả tìm kiếm và hộp thoại của truyện trước còn sót lại khi chuyển sang truyện khác.

## 1.0.17 — 2026-10-12
- Thêm trang **Tìm & thay** (Ctrl+Shift+F) để sửa một từ dịch sai trên toàn bộ bản dịch đã chốt: xem từng chỗ kèm câu gốc, bỏ tick chỗ không muốn thay và hoàn tác lần thay gần nhất.
- Thêm nút **Rà chương cũ** ở tab **Glossary** để tìm những đoạn đã dịch chưa theo cách dịch hiện tại, xem trước rồi thay.
- Thêm lựa chọn **Biến thân** và ô đại từ lời kể cho từng nhân vật ở tab **Nhân vật**, kèm lời nhắc khi nhân vật chưa có mốc đổi xưng hô.
- Cải thiện prompt dịch theo đợt thẩm định chất lượng: thán từ và tiếng cười, số và đơn vị, tiếng lóng và meme, dấu ngoặc kép cong, tiêu đề chương viết hoa kiểu câu, bỏ lời xin phiếu cuối chương.
- Cải thiện bộ kiểm tra bản dịch: bớt báo nhầm, thêm loại lỗi chỉ cảnh báo mà không bắt soát lại, tên lệch dấu so với glossary được tự sửa khi chốt chương.
- Cải thiện cách dịch chức danh hiện đại (chủ tịch huyện, trưởng phòng, giám đốc sở…) và nhận ra thêm nhiều chức danh để biết nhân vật nào có mặt trong chương.
- Cải thiện truyện bối cảnh hiện đại hoặc hỗn hợp: kiểu tên riêng tự chuyển sang Hỗn hợp khi dựng hồ sơ hoặc khi đổi bối cảnh.
- Sửa các lỗi dịch kiểu convert như "chiêu lãm", "người tộc", "không tiền khoáng hậu"; bộ kiểm tra bản dịch bắt và đưa vào soát.

## 1.0.16 — 2026-10-10
- Sửa lỗi Antigravity bản 1.3 báo không nhận ra model đã chọn; danh sách model hiện tên dễ đọc và nhắc chọn lại nếu model cũ không còn.

## 1.0.15 — 2026-10-06
- Thêm nút **Quét chương mới** ở màn chọn truyện để quét chương mới cho cả thư viện, và **Dịch tất cả truyện chưa xong** để bắt đầu dịch mọi truyện còn chương chờ.
- Thêm báo giờ quota Antigravity mở lại khi hết quota, kèm tuỳ chọn **Tự dịch tiếp khi quota agy reset** trong Cài đặt.
- Thêm mục **Tiêu đề chương** trong Hồ sơ truyện: tiêu đề lặp lại của truyện chia phần được dịch thống nhất, đuôi số theo một quy tắc, có thể sửa rồi áp dụng lại cho các chương đã dịch.
- Thêm số dòng ở trang đọc chương, tô sáng dòng còn sót chữ Hán và hiện số chữ của bản gốc và bản dịch.
- Thêm thẻ **Hội thoại agy** trong Cài đặt; app tự dọn các hội thoại Antigravity do app tạo sau mỗi lượt dịch (trước đây có thể chiếm vài GB), hội thoại bạn tự chat không bị đụng tới.
- Cải thiện thẻ truyện ở cột phải: hiện số chương bị bỏ qua hoặc lỗi, kể cả với truyện đang dịch nền.
- Sửa lỗi mất điện hoặc tắt máy đột ngột lúc đang ghi làm hỏng truyện; nếu file tiến độ bị hỏng, app tự dựng lại từ chương gốc và bản dịch đã có.

## 1.0.14 — 2026-10-01
- Thêm tab **Nhân vật** trong Hồ sơ truyện để lưu giới tính nhân vật và các mốc chương đổi xưng hô (ví dụ từ "tôi–cô" sang "anh–em"), có thể ghim cặp xưng hô và đặt mốc bằng tay.
- Thêm nút **Reset truyện…** để đưa truyện về như lúc vừa tạo rồi dịch lại; chương gốc và bản dịch cũ được giữ, hồ sơ cũ có bản sao lưu.
- Cải thiện prompt dịch dựa trên giới tính nhân vật để tránh gọi sai (nữ bị gọi "thầy"); bộ kiểm tra bản dịch bắt lỗi sai giới và cách gọi "đàn em/đàn chị" trong thoại.
- Cải thiện chi phí dịch bằng API key nhờ sắp xếp lại prompt dịch để phần giống nhau giữa các chương được nhà cung cấp lưu đệm.
- Sửa vị trí thông báo: hiện ở giữa cạnh dưới, không còn che các nút ở góc trên phải.

## 1.0.13 — 2026-09-30
- Sửa lỗi chữ Hán còn sót trong bản dịch vẫn được chốt: đoạn còn chữ Hán giờ được dịch lại, hết vòng soát vẫn còn thì chương báo lỗi.
- Sửa lỗi tên riêng lệch dấu so với glossary (ví dụ "Đoạn Phong" thay vì "Đoàn Phong") lọt qua; bộ kiểm tra bản dịch giờ bắt để soát lại, và glossary tự động không học âm đọc trái với âm đã chốt.

## 1.0.12 — 2026-09-25
- Cải thiện prompt dịch truyện hiện đại: 老大, 大哥, 大佬 được gọi theo giới tính người được gọi, nhân vật nữ không bị dịch thành "đại ca".

## 1.0.11 — 2026-09-24
- Cải thiện prompt dịch Hán-Việt với bảng họ có cách đọc riêng (ví dụ 段 là Đoàn), để cụm họ kèm chức danh được phiên đúng.
- Sửa lỗi chương đã dịch xong vẫn bị bỏ qua khi dịch bằng API key vì cờ lọc nội dung ở cuối câu trả lời; bước soát bị chặn giờ giữ lại bản dịch.

## 1.0.10 — 2026-09-19
- Sửa lỗi khi dịch bằng API key kiểu OpenAI: model từ chối hoặc bị lọc nội dung giờ được nhận ra ngay và chương được bỏ qua kèm lý do, thay vì báo sai định dạng và gọi lại vô ích.

## 1.0.9 — 2026-09-19
- Sửa lỗi các truyện trong cột **Phiên này** đổi chỗ mỗi lần chuyển truyện; giờ giữ nguyên thứ tự mở lần đầu.

## 1.0.8 — 2026-09-19
- Sửa lỗi cập nhật trên máy chưa từng cài đặt đầy đủ (bản portable, exe rời): bộ cài giờ tạo thêm shortcut ngoài Desktop.

## 1.0.7 — 2026-09-18
- Cải thiện bộ cài: chỉ phát hành dạng `.exe`, bỏ bộ cài MSI.
- Sửa lỗi sau khi cập nhật không tìm thấy app trong Start Menu và Windows Search; bộ cài tự tạo lại shortcut nếu thiếu.
- Sửa lỗi truyện tạo bằng app bản cũ không nhận được hướng dẫn dịch mới cho Antigravity; giờ được cập nhật khi mở truyện, trừ khi bạn đã sửa tay.

## 1.0.6 — 2026-09-18
- Cải thiện prompt dịch truyện cổ đại: nhân vật lớn tuổi, bề trên vẫn là "hắn" hoặc "nàng" thay vì "ông ta", "bà ta"; bộ kiểm tra bản dịch bắt các cách gọi này.
- Cải thiện dịch bằng Antigravity: khi bộ lọc nội dung chặn, chương được dịch lại tối đa 3 lượt trước khi bị bỏ qua.
- Cải thiện hướng dẫn dịch trong thư mục truyện: tự cập nhật theo bản mới của app khi mở truyện, trừ khi bạn đã sửa tay.

## 1.0.5 — 2026-09-17
- Thêm mục **Giọng văn** trong Hồ sơ truyện với 7 lựa chọn (trung tính, ngôn tình, hài hước, sảng văn, cổ phong, sắc, thanh xuân); **AI điền hồ sơ** cũng đề xuất giọng văn.
- Thêm nút **Export Names.txt…** ở tab **Glossary** để xuất glossary ra Names.txt hoặc Names2.txt dùng cho bản convert.
- Thêm nút **Dịch lại cả N** ở dòng cảnh báo chương hổng để đưa mọi chương bị bỏ qua hoặc lỗi về hàng đợi.
- Cải thiện prompt dịch giữ tên riêng nhất quán: một chữ Hán một âm Hán-Việt, viết hoa tước vị và địa danh, tiêu đề dạng `Chương N: Tiêu đề`; bộ kiểm tra bản dịch bắt lỗi kiểu "đắc được", "thu hoạch được".
- Cải thiện thanh cuộn to hơn, dễ bấm; bấm chip chương hổng sẽ cuộn danh sách tới đúng chương đó.

## 1.0.4 — 2026-09-12
- Thêm số token vào, ra và suy nghĩ của từng lượt và tổng mỗi chương vào log (khi nhà cung cấp có báo).
- Cải thiện dịch bằng API key: glossary được trích ngay trong lượt dịch, bớt một lượt gọi model cho mỗi chương.

## 1.0.3 — 2026-09-12
- Thêm xoá chương ngay trong app, từng chương hoặc theo khoảng; bản dịch đã có được giữ lại.
- Thêm chọn khoảng bằng số thứ tự chương ở trang **Export**, kèm danh sách chương có nút Từ/Đến.
- Thêm bảng **Mức nghĩ theo bước** (dịch, soát, trích glossary, AI điền) trong Cài đặt động cơ dịch; số truyện dịch song song tối đa tăng lên 20.
- Thêm thanh công cụ định dạng (tiêu đề, danh sách, bảng…) cho ô sửa prompt; gõ prompt và cuộn trong hộp thoại không còn giật.
- Cải thiện vòng soát: model giữ nguyên bản dịch thì chốt ngay, đỡ tốn lượt gọi; bỏ vài luật kiểm tra bắt nhầm từ tiếng Việt hợp lệ.
- Sửa lỗi xoá file chương gốc rồi mở app thì báo lỗi; **Quét lại** giờ gỡ các chương đã mất file gốc.

## 1.0.2 — 2026-09-11
- Thêm cảnh báo trên thanh công cụ trang Dịch khi còn chương bị bỏ qua hoặc lỗi nằm trước chương đã dịch cuối cùng; bấm vào chip để mở chương đó.

## 1.0.1 — 2026-09-11
- Cải thiện prompt dịch để model bớt từ chối oan các tình tiết nhạy cảm trong truyện hư cấu đã xuất bản.
- Sửa lỗi lượt dịch bằng Antigravity bị cắt giữa chừng sau 5 phút.

## 1.0.0 — 2026-09-11
- Thêm dịch truyện chữ tiếng Trung bằng hai động cơ: Antigravity hoặc API key (Gemini và các dịch vụ tương thích OpenAI).
- Thêm thư viện truyện: tạo truyện mới, kéo thả file chương vào, quét lại chương, mở nhanh truyện gần đây.
- Thêm Hồ sơ truyện gồm thông tin, thể loại (bối cảnh, kiểu tên riêng), glossary, văn phong và luật kiểm tra, có **AI điền hồ sơ** từ vài chương đầu.
- Thêm bộ kiểm tra bản dịch tự bắt thiếu đoạn, sót chữ Hán, từ cấm rồi cho model soát lại trước khi chốt chương.
- Thêm trang Dịch với tiến độ, danh sách chương, khung đọc bản dịch và bản gốc, sửa tay bản dịch, dịch lại toàn bộ hoặc theo khoảng chương.
- Thêm dịch nhiều truyện song song và cột chuyển nhanh giữa các truyện (Ctrl+K).
- Thêm export bản dịch ra file TXT theo khoảng chương, có báo chương còn hổng.
- Thêm 3 bộ màu sáng/tối, prompt và luật mặc định sửa được, bản portable và tự cập nhật qua GitHub Releases.
