# Đóng góp

Cảm ơn bạn đã quan tâm tới Quick Translator Engine. Dự án ưu tiên tính tương thích với
hành vi QT2025 và thay đổi nhỏ, có test, dễ đối chiếu.

## Thiết lập

```bash
git clone <repository-url>
cd quick-translator-engine
cargo build --workspace
```

Để chạy CLI/API với dữ liệu thật, đặt bộ từ điển hợp lệ trong `QT2025/` hoặc truyền một
data directory khác. Unit/integration tests dùng fixture nhỏ và không cần toàn bộ bộ dữ
liệu production.

## Trước khi gửi pull request

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
git diff --check
```

Pull request nên:

- mô tả input làm lộ khác biệt và output mong đợi;
- thêm regression test ở crate gần logic nhất;
- phân biệt rõ hành vi lấy từ QT2025 và contract mở rộng của Rust;
- cập nhật README/API/engine docs nếu public behavior thay đổi;
- không commit dictionary cá nhân, log/history, binary, token hoặc file `.env`.

## Quy ước commit

Repository dùng Conventional Commits, ví dụ:

```text
feat(qt-core): add phrase matching rule
fix(qt-api): preserve pretty target ranges
docs: document dictionary layout
test(qt-cli): cover invalid options
```

Scope nên là `qt-core`, `qt-cli`, `qt-api` hoặc bỏ scope khi thay đổi trải rộng toàn repo.

Bằng việc gửi đóng góp cho phần code Rust, contributor đồng ý phát hành phần đóng góp đó
theo cùng giấy phép `GPL-3.0-only` của dự án.

## Thay đổi thuật toán

Ưu tiên nguồn chứng cứ theo thứ tự:

1. source decompile trong `reference/decompiled/`;
2. output thu trực tiếp từ ứng dụng QT2025;
3. đặc tả trong `docs/engine/`;
4. suy luận tương thích được ghi rõ trong code/test.

Không “sửa đẹp” spacing, capitalization hoặc mapping nếu chưa xác định đó là presentation
layer hay hành vi engine. API option `pretty` là nơi dành cho chuẩn hóa trình bày.

## Báo lỗi bảo mật

Không đưa credential hoặc dữ liệu nhạy cảm vào issue công khai. Xem
[SECURITY.md](SECURITY.md) để biết kênh báo cáo.
