# Third-party material

Repository chứa hai nhóm artifact không phải source Rust nguyên bản của QT-CLI:

1. Dữ liệu và config tham chiếu dưới `QT2025/`.
2. Source C# được decompile từ engine tham chiếu dưới `reference/decompiled/`.

Phần code Rust của QT-CLI được cấp phép theo `GPL-3.0-only`; xem [LICENSE](LICENSE). Các
artifact tham chiếu được giữ để tái tạo và kiểm chứng hành vi, nhưng repository hiện không
kèm văn bản license xác nhận quyền phân phối lại cho toàn bộ các file đó. Việc một file có
mặt trong repository không có nghĩa file đó được cấp cùng license với phần code Rust.

Trước khi public hoặc phân phối bản sao repository:

- xác nhận nguồn gốc và quyền phân phối của bộ từ điển/config;
- xác nhận quyền lưu hành source decompile;
- giữ nguyên attribution của bên thứ ba khi điều khoản nguồn yêu cầu;
- loại binary, archive, token, history/log người dùng và config máy cá nhân.

`QT2025/columnTemplate.doc` có metadata/attribution của tài liệu Quick Translator gốc.
Thông tin đó thuộc artifact tham chiếu và không phải danh tính maintainer QT-CLI.
