# Tái tạo source decompile của engine tham chiếu

`reference/decompiled/TranslatorEngine.decompiled.cs` được tạo từ
`TranslatorEngine.dll` để đối chiếu hành vi trong quá trình reimplement. File này không
phải source gốc của dự án Rust.

Chỉ thực hiện các bước dưới đây với assembly mà bạn có quyền sử dụng và phân tích. DLL
không được lưu trong Git repository.

## Công cụ đã được kiểm chứng

- .NET SDK 8
- .NET runtime 6
- `ilspycmd` 8.2.0.7535

Đây là bộ version dùng để tái tạo artifact hiện có, không phải tuyên bố về version mới
nhất của ILSpy.

## Linux/macOS

Từ thư mục gốc repository:

```bash
mkdir -p _tools
curl -fsSL -o _tools/dotnet-install.sh https://dot.net/v1/dotnet-install.sh
chmod +x _tools/dotnet-install.sh

./_tools/dotnet-install.sh --channel 8.0 --install-dir "$PWD/_tools/dotnet"
./_tools/dotnet-install.sh --channel 6.0 --runtime dotnet \
  --install-dir "$PWD/_tools/dotnet"

export DOTNET_ROOT="$PWD/_tools/dotnet"
export PATH="$DOTNET_ROOT:$PATH"

dotnet tool install --tool-path _tools/ilspy \
  ilspycmd --version 8.2.0.7535

./_tools/ilspy/ilspycmd QT2025/TranslatorEngine.dll \
  -o reference/decompiled
```

Nếu tool sinh tên file khác, đổi tên output thành
`reference/decompiled/TranslatorEngine.decompiled.cs` rồi rà diff trước khi commit.

## Windows PowerShell

Khi máy đã có .NET SDK/runtime phù hợp:

```powershell
dotnet tool install --tool-path _tools\ilspy `
  ilspycmd --version 8.2.0.7535

_tools\ilspy\ilspycmd.exe QT2025\TranslatorEngine.dll `
  -o reference\decompiled
```

## Kiểm tra sau khi decompile

1. Xác nhận class `TranslatorEngine` và `TransLuatNhan` có mặt.
2. So sánh diff với artifact hiện tại; thay đổi do version decompiler phải được tách khỏi
   thay đổi logic thật.
3. Không commit DLL, archive gốc, token, file history người dùng hoặc config chứa secret.
4. Chạy lại secret scan trước khi commit artifact mới.

Phần charset detector Mozilla ở cuối file không thuộc pipeline dịch chính. Các phần được
đối chiếu thường xuyên là dictionary loader, `StandardizeInput`, `TranslateAll`,
`ChineseToHanViet` và `TransLuatNhan`.

## QuickTranslator.exe (GUI)

Logic lọc name của QT2025 không nằm trong `TranslatorEngine.dll` mà trong GUI
(`LocNameOff.LocNameQT` + flow gọi API MTC/Gemini). Decompile bằng cùng bộ tool:

```bash
./_tools/ilspy/ilspycmd QT2025/QuickTranslator.exe -o <thư mục tạm>
```

KHÔNG commit bản decompile đầy đủ của GUI: file này nhúng email/password đăng nhập
metruyencv và token cache path. Chỉ trích class cần đối chiếu vào
`reference/decompiled/LocNameOff.decompiled.cs` (đã kèm
`GetThresholdBasedOnWordCount`).

`reference/locname_qt2025_replica.py` là replica Python chạy được của `LocNameQT`
(cần `pip install jieba`), dùng để kiểm chứng byte-for-byte mode `qt` của engine:

```bash
python3 reference/locname_qt2025_replica.py chapter.txt [threshold]
```
