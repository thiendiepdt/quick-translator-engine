# qt-ai-gui: export glossary truyện ra Names.txt / Names2.txt

Ngày: 2026-09-16. Yêu cầu người dùng: lấy glossary của bản dịch AI làm từ điển tên cho bản convert (QT),
định dạng `cụm Trung=cụm Việt` mỗi dòng; được chọn mục để export; mặc định chọn hết trừ Cụm từ đặc trưng;
Xưng hô theo cặp không export.

## Phạm vi

- Nguồn: glossary của truyện đang mở, lấy từ form tab Glossary (kể cả sửa chưa lưu). Không gộp kho chung
  (Cài đặt → Bản mặc định), không lấy Cụm từ đặc trưng bên tab Style.
- Nhóm hiện trong dialog: names, places, items, creatures, skills, common, signature_phrases (thứ tự này).
  Nhóm `addressing` không hiện, không export (khoá `甲→乙` không phải cụm Hán).
- Mặc định tick: mọi dòng hợp lệ của 6 nhóm đầu; `signature_phrases` không tick.

## UI

Tab Glossary (Hồ sơ truyện) thêm nút **Export Names.txt…** ở đầu tab, cạnh ghi chú kho chung. Bấm mở
`GlossaryExportDialog`:

- Cột trái: ô tìm nhanh (lọc theo cả hai cột, không đổi trạng thái tick); danh sách 7 nhóm, mỗi nhóm một
  checkbox tổng (checked / indeterminate / unchecked theo các dòng hợp lệ của nhóm) kèm `đã tick/tổng`, bung
  ra danh sách dòng `Hán = Việt` tick riêng. Nhóm rỗng hiện mờ. Dòng bị bỏ (xem quy tắc) hiện mờ, disabled,
  ghi lý do ngắn.
- Cột phải: xem trước nội dung file (textarea readonly, font mono), dòng tổng "Sẽ ghi N dòng · bỏ M dòng".
- Chân dialog: `Choice` Tên file: `Names.txt` | `Names2.txt` (mặc định Names.txt); nút **Chép** (clipboard,
  không BOM); nút **Lưu…** mở hộp thoại lưu của hệ với tên mặc định theo lựa chọn, ghi file rồi toast kèm
  đường dẫn. Không có dòng nào được tick → hai nút disabled.

## Quy tắc sinh file (bám `docs/engine/dictionaries.md`)

- Mỗi dòng `source=target`, source/target đã trim. Xuống dòng CRLF. File ghi UTF-8 có BOM (như QT); bản chép
  clipboard không BOM.
- Dòng bị bỏ, kèm lý do: source hoặc target rỗng sau trim ("thiếu Hán/Việt"); source hoặc target chứa `=`
  ("có dấu ="; QT tách theo mọi `=` rồi vứt dòng); chứa xuống dòng ("nhiều dòng").
- Trùng source (so sau trim), trong cùng nhóm hay giữa các nhóm: dòng đầu theo thứ tự nhóm rồi thứ tự dòng
  giữ; dòng sau bị bỏ với lý do "trùng nhóm <nhãn>" (form là mảng nên cùng nhóm vẫn trùng được).
- Thứ tự dòng trong file: theo nhóm rồi theo thứ tự trong nhóm.

## Kỹ thuật

- `apps/qt-ai-gui/src/lib/names-export.ts`:
  - `NAMES_EXPORT_KEYS` = 7 nhóm theo thứ tự; `DEFAULT_UNCHECKED_KEYS = ["signature_phrases"]`.
  - `planNamesExport(glossary: StoryFormValues["glossary"]) -> NamesExportRow[]`: mỗi dòng
    `{ id, group, source, target, skip?: string }` (id = `${group}:${index}`), áp quy tắc bỏ/trùng.
  - `defaultSelection(rows) -> Set<string>`; `renderNames(rows, selected, { bom }) -> string`.
  - Vitest phủ: bỏ rỗng/`=`/xuống dòng, trùng nhóm, thứ tự, mặc định bỏ signature_phrases, BOM/CRLF.
- `apps/qt-ai-gui/src/components/glossary-export-dialog.tsx`: props `{ open, onOpenChange, glossary }`;
  state `selected: Set<string>`, `query`, `fileName`. Dùng Dialog, Input, Textarea, Choice, Button có sẵn;
  checkbox là `<input type="checkbox">` native (chưa có component Checkbox; indeterminate qua ref).
  Test: render mặc định (signature_phrases không tick, addressing không hiện), tick nhóm/dòng đổi preview,
  dòng bỏ disabled kèm lý do, Chép gọi `copyText`, Lưu gọi `pickSaveFile` với tên đã chọn rồi
  `writeTextFile`.
- `apps/qt-ai-gui/src/lib/api.ts`: `writeTextFile(path, content)`; `pickSaveFile` nhận thêm `title` tuỳ
  chọn (mặc định giữ "Lưu file gộp").
- Rust `story_cmds::write_text_file(path: String, content: String)`: ghi nguyên `content` (BOM đã nằm
  trong chuỗi), tạo folder cha nếu thiếu, lỗi → `CommandError("io", …)`. Test ghi ra đúng byte.
- `story-page.tsx`: nút mở dialog trong Section glossary, glossary lấy từ `useWatch({ name: "glossary" })`.
- README qt-ai-gui: mục "Export glossary ra Names.txt".
