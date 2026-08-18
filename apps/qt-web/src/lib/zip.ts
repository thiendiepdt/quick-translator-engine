/**
 * ZIP tối thiểu cho nhu cầu "tải các chương đã dịch": method store (không
 * nén — text truyện vốn nhỏ), tên file UTF-8 (flag bit 11). Tự viết ~80 dòng
 * thay vì kéo cả jszip vào bundle chỉ để đóng gói vài file text.
 */

const CRC_TABLE = (() => {
  const table = new Uint32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) {
      c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    }
    table[n] = c >>> 0;
  }
  return table;
})();

export function crc32(bytes: Uint8Array): number {
  let crc = 0xffffffff;
  for (const byte of bytes) {
    crc = CRC_TABLE[(crc ^ byte) & 0xff] ^ (crc >>> 8);
  }
  return (crc ^ 0xffffffff) >>> 0;
}

export interface ZipFileInput {
  name: string;
  content: string;
}

/** Giờ DOS cố định (1980-01-01) để cùng input luôn ra cùng bytes. */
const DOS_TIME = 0;
const DOS_DATE = 0x21;

class ByteWriter {
  private chunks: number[] = [];

  u16(value: number) {
    this.chunks.push(value & 0xff, (value >>> 8) & 0xff);
  }

  u32(value: number) {
    this.chunks.push(
      value & 0xff,
      (value >>> 8) & 0xff,
      (value >>> 16) & 0xff,
      (value >>> 24) & 0xff,
    );
  }

  bytes(value: Uint8Array) {
    for (const byte of value) this.chunks.push(byte);
  }

  get length(): number {
    return this.chunks.length;
  }

  toUint8Array(): Uint8Array {
    return Uint8Array.from(this.chunks);
  }
}

/** Cờ bit 11: tên file và comment là UTF-8. */
const UTF8_FLAG = 0x0800;

export function buildZip(files: ZipFileInput[]): Uint8Array {
  const encoder = new TextEncoder();
  const writer = new ByteWriter();
  const central: Array<{
    name: Uint8Array;
    crc: number;
    size: number;
    offset: number;
  }> = [];

  for (const file of files) {
    const name = encoder.encode(file.name);
    const content = encoder.encode(file.content);
    const crc = crc32(content);
    central.push({ name, crc, size: content.length, offset: writer.length });

    writer.u32(0x04034b50);
    writer.u16(20); // version needed
    writer.u16(UTF8_FLAG);
    writer.u16(0); // method: store
    writer.u16(DOS_TIME);
    writer.u16(DOS_DATE);
    writer.u32(crc);
    writer.u32(content.length);
    writer.u32(content.length);
    writer.u16(name.length);
    writer.u16(0); // extra
    writer.bytes(name);
    writer.bytes(content);
  }

  const centralOffset = writer.length;
  for (const entry of central) {
    writer.u32(0x02014b50);
    writer.u16(20); // version made by
    writer.u16(20); // version needed
    writer.u16(UTF8_FLAG);
    writer.u16(0); // method
    writer.u16(DOS_TIME);
    writer.u16(DOS_DATE);
    writer.u32(entry.crc);
    writer.u32(entry.size);
    writer.u32(entry.size);
    writer.u16(entry.name.length);
    writer.u16(0); // extra
    writer.u16(0); // comment
    writer.u16(0); // disk
    writer.u16(0); // internal attrs
    writer.u32(0); // external attrs
    writer.u32(entry.offset);
    writer.bytes(entry.name);
  }
  const centralSize = writer.length - centralOffset;

  writer.u32(0x06054b50);
  writer.u16(0); // disk
  writer.u16(0); // central disk
  writer.u16(central.length);
  writer.u16(central.length);
  writer.u32(centralSize);
  writer.u32(centralOffset);
  writer.u16(0); // comment
  return writer.toUint8Array();
}
