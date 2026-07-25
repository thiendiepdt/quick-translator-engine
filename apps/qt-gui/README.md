# qt-gui

Ứng dụng desktop Tauri cho Quick Translator Engine. Frontend dùng cùng stack với
`qt-web` (Vite, React, TypeScript, Tailwind CSS, shadcn/ui, React Hook Form, TanStack
Query, Zustand và Zod), nhưng không gọi HTTP API: mọi lệnh dịch và lọc name đi thẳng
qua Tauri IPC vào `qt-core`.

## Chức năng

- Ba mode QT2025: Hán Việt, VietPhrase và VietPhrase một nghĩa.
- Longest-match, Luật Nhân, ưu tiên Name, scan range, wrap và pretty output.
- Source/output mapping UTF-16, click đối chiếu và cập nhật nhanh dictionary.
- Chỉnh tám dictionary tùy biến; patch VietPhrase và ChinesePhienAmWords theo workspace.
- Lọc name QT-compatible hoặc hybrid, kèm memory riêng theo truyện.
- Mở chương UTF-8 từ `.txt`, sao chép và lưu bản dịch ra `.txt`.
- Tự tìm `QT2025` khi chạy trong checkout hoặc đọc `QT_DATA_DIR`; có thể chọn data
  directory khác trong giao diện.

Các thay đổi dictionary trong giao diện là override cục bộ và không ghi đè file gốc.
`VietPhrase/VietPhrase.txt` và `Resources/ChinesePhienAmWords.txt` là hai file bắt buộc
trong data directory. Installer không bundle dữ liệu `QT2025`; người dùng phải trỏ app
tới bộ dữ liệu mà họ có quyền sử dụng.

Tra nghĩa Lạc Việt (`ChineseToMeanings`) chưa có vì chức năng tương ứng chưa được port
vào `qt-core`.

## Development

Trên Windows cần Rust MSVC, Node.js và WebView2. Từ repository root:

```powershell
cd apps/qt-gui
npm ci
npm run tauri dev
```

Nếu không dùng thư mục `QT2025` trong checkout:

```powershell
$env:QT_DATA_DIR = "D:\Data\QT2025"
npm run tauri dev
```

Kiểm tra frontend và Rust:

```powershell
npm run check
cargo test -p qt-gui
cargo clippy -p qt-gui --all-targets -- -D warnings
```

Tạo installer Windows:

```powershell
npm run tauri build
```

Tauri tạo NSIS `.exe` và MSI theo cấu hình trong `src-tauri/tauri.conf.json`.
