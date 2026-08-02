/**
 * Đọc file văn bản chương truyện với dò encoding: novel .txt tải từ các trang
 * Trung Quốc thường là GB18030/GBK (không phải UTF-8), file từ Notepad cũ có
 * thể là UTF-16 kèm BOM. Đọc mù bằng UTF-8 sẽ ra mojibake toàn bộ.
 */

const MAX_TEXT_FILE_BYTES = 4 * 1024 * 1024;

/** Nhận file text theo MIME, hoặc theo đuôi tên khi hệ điều hành bỏ trống MIME. */
export function looksLikeTextFile(file: File): boolean {
  if (file.type) return file.type.startsWith("text/");
  return /\.(txt|md)$/i.test(file.name);
}

export async function readChapterFile(file: File): Promise<string> {
  if (file.size > MAX_TEXT_FILE_BYTES) {
    throw new Error(
      `File quá ${Math.round(MAX_TEXT_FILE_BYTES / 1024 / 1024)} MB — hãy chia nhỏ trước khi nhập`,
    );
  }
  const bytes = new Uint8Array(await file.arrayBuffer());
  return decodeChapterBytes(bytes);
}

/** BOM UTF-16 → UTF-16; còn lại thử UTF-8 nghiêm ngặt rồi mới rơi về GB18030. */
export function decodeChapterBytes(bytes: Uint8Array): string {
  if (bytes.length >= 2 && bytes[0] === 0xff && bytes[1] === 0xfe) {
    return new TextDecoder("utf-16le").decode(bytes.subarray(2));
  }
  if (bytes.length >= 2 && bytes[0] === 0xfe && bytes[1] === 0xff) {
    return new TextDecoder("utf-16be").decode(bytes.subarray(2));
  }
  try {
    return new TextDecoder("utf-8", { fatal: true, ignoreBOM: false }).decode(bytes);
  } catch {
    return new TextDecoder("gb18030").decode(bytes);
  }
}
