# Tái tạo bản decompile engine

Engine gốc `QT2025/TranslatorEngine.dll` là .NET assembly. Bản decompile ra C# là **nguồn chân
lý** cho toàn bộ [docs/engine/](../engine/). File đã lưu sẵn tại:

    reference/decompiled/TranslatorEngine.decompiled.cs   (~5600 dòng)

Tài liệu này ghi lại cách **tái tạo** nó (khi cập nhật DLL mới, hoặc trên máy khác) vì toolchain
không cài sẵn trên hệ thống.

## Yêu cầu
- Có mạng (tải .NET runtime + ilspycmd).
- Không cần `sudo`. Mọi thứ cài vào một thư mục cục bộ (vd `./_tools`).

## Các bước

```bash
TOOLS=./_tools            # thư mục tạm, có thể xoá sau
mkdir -p "$TOOLS" && cd "$TOOLS"

# 1. .NET SDK 8 (chạy ilspycmd) — cài cục bộ, không cần sudo
curl -sSL -o dotnet-install.sh https://dot.net/v1/dotnet-install.sh
chmod +x dotnet-install.sh
./dotnet-install.sh --channel 8.0 --install-dir "$PWD/dotnet"

# 2. .NET 6 runtime (ilspycmd 8.2.x target net6.0)
./dotnet-install.sh --channel 6.0 --runtime dotnet --install-dir "$PWD/dotnet"

export DOTNET_ROOT="$PWD/dotnet"
export PATH="$PWD/dotnet:$PATH"

# 3. ilspycmd — PIN version 8.2.0.7535 (bản mới nhất bị lỗi packaging trên nền này)
dotnet tool install --tool-path ./dotnet-tools ilspycmd --version 8.2.0.7535

# 4. Decompile
./dotnet-tools/ilspycmd ../QT2025/TranslatorEngine.dll -o ./out
cp ./out/TranslatorEngine.decompiled.cs ../reference/decompiled/
```

## Cạm bẫy đã gặp
- `ilspycmd` bản **latest (10.x)** báo `DotnetToolSettings.xml not found` → **pin 8.2.0.7535**.
- ilspycmd 8.2 cần **.NET 6 runtime** (không phải chỉ SDK 8) → bước 2 bắt buộc.
- Hệ thống không có `unrar`/`7z`/`sudo`. Source QT2025 ban đầu là `QT2025.rar` (RAR5); giải nén
  bằng unrar tĩnh của rarlab: `curl -sL https://www.rarlab.com/rar/rarlinux-x64-712.tar.gz`.

## Phần không liên quan (bỏ qua khi đọc)
Dòng ~3329–5624 là bộ **charset detector của Mozilla** (`nsDetector`, `ns*Verifier`,
`*Statistics`) — không thuộc thuật toán dịch. Logic dịch nằm ở class `TranslatorEngine` và
`TransLuatNhan` (~dòng 465–2200).
