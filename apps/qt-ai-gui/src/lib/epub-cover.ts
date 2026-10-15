/**
 * Ảnh bìa EPUB vẽ qua canvas của webview: ảnh tải từ nguồn (WebP, PNG…) vẽ lại thành JPEG vì app đọc và
 * Kindle đời cũ không đọc WebP; truyện không có ảnh thì vẽ bìa mặc định có tên truyện, tên gốc, tác giả —
 * JPEG thật nên ảnh thu nhỏ trong thư viện sách của app đọc cũng hiện chữ.
 */

const COVER_WIDTH = 1200;
const COVER_HEIGHT = 1800;
/** Ảnh nguồn to hơn thì thu lại: đủ nét cho màn đọc, sách không nặng thêm vô ích. */
const MAX_COVER_HEIGHT = 1800;
const JPEG_QUALITY = 0.9;

export interface CoverText {
  title: string;
  originalTitle: string;
  author: string;
}

const HAN = /[㐀-䶿一-鿿豈-﫿]/;

/** `"Phương Thốn Đạo Chủ (方寸道主)"` → tên Việt + tên gốc — cùng luật `split_story_name` của core. */
export function splitStoryName(name: string): { title: string; originalTitle: string } {
  const trimmed = name.trim();
  const m = trimmed.match(/^(.*?)\s*[(（]([^()（）]*)[)）]$/);
  if (m && HAN.test(m[2])) return { title: m[1].trim(), originalTitle: m[2].trim() };
  return { title: trimmed, originalTitle: "" };
}

function canvasToJpeg(canvas: HTMLCanvasElement): Promise<Uint8Array> {
  return new Promise((resolve, reject) => {
    canvas.toBlob(
      (blob) => {
        if (!blob) return reject(new Error("Không vẽ được ảnh bìa"));
        blob.arrayBuffer().then((buffer) => resolve(new Uint8Array(buffer)), reject);
      },
      "image/jpeg",
      JPEG_QUALITY,
    );
  });
}

function context(canvas: HTMLCanvasElement): CanvasRenderingContext2D {
  const ctx = canvas.getContext("2d");
  if (!ctx) throw new Error("Webview không hỗ trợ canvas");
  return ctx;
}

/** Ảnh bìa tải về (byte gốc) → JPEG, thu nhỏ nếu cao quá `MAX_COVER_HEIGHT`. */
export async function imageToJpeg(bytes: Uint8Array, mime: string): Promise<Uint8Array> {
  const bitmap = await createImageBitmap(new Blob([bytes as BlobPart], { type: mime }));
  const scale = Math.min(1, MAX_COVER_HEIGHT / bitmap.height);
  const canvas = document.createElement("canvas");
  canvas.width = Math.max(1, Math.round(bitmap.width * scale));
  canvas.height = Math.max(1, Math.round(bitmap.height * scale));
  const ctx = context(canvas);
  // Ảnh PNG/WebP trong suốt: nền trắng thay vì đen khi thành JPEG.
  ctx.fillStyle = "#ffffff";
  ctx.fillRect(0, 0, canvas.width, canvas.height);
  ctx.drawImage(bitmap, 0, 0, canvas.width, canvas.height);
  bitmap.close();
  return canvasToJpeg(canvas);
}

/** Ngắt dòng theo từ cho vừa `maxWidth`; từ dài hơn cả dòng (tên gốc tiếng Trung không dấu cách) ngắt theo chữ. */
export function wrapText(measure: (text: string) => number, text: string, maxWidth: number): string[] {
  const lines: string[] = [];
  let line = "";
  const push = (piece: string, joiner: string) => {
    const next = line ? `${line}${joiner}${piece}` : piece;
    if (!line || measure(next) <= maxWidth) {
      line = next;
    } else {
      lines.push(line);
      line = piece;
    }
  };
  for (const word of text.trim().split(/\s+/).filter(Boolean)) {
    if (measure(word) <= maxWidth) {
      push(word, " ");
      continue;
    }
    for (const char of Array.from(word)) push(char, "");
  }
  if (line) lines.push(line);
  return lines;
}

const SERIF = "'Noto Serif', 'Times New Roman', Georgia, serif";

/** Bìa mặc định: nền mực đậm, khung viền mảnh, tên truyện giữa bìa, tên gốc bên dưới, tác giả gần đáy. */
export async function renderDefaultCover(cover: CoverText): Promise<Uint8Array> {
  const canvas = document.createElement("canvas");
  canvas.width = COVER_WIDTH;
  canvas.height = COVER_HEIGHT;
  const ctx = context(canvas);

  const background = ctx.createLinearGradient(0, 0, COVER_WIDTH, COVER_HEIGHT);
  background.addColorStop(0, "#1f3a4d");
  background.addColorStop(1, "#0d1b26");
  ctx.fillStyle = background;
  ctx.fillRect(0, 0, COVER_WIDTH, COVER_HEIGHT);
  ctx.strokeStyle = "rgba(226, 196, 140, 0.75)";
  ctx.lineWidth = 6;
  ctx.strokeRect(60, 60, COVER_WIDTH - 120, COVER_HEIGHT - 120);
  ctx.lineWidth = 2;
  ctx.strokeRect(84, 84, COVER_WIDTH - 168, COVER_HEIGHT - 168);

  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  const maxWidth = COVER_WIDTH - 280;

  // Tên dài thì thu chữ tới khi còn tối đa 5 dòng.
  let size = 120;
  let titleLines: string[] = [];
  for (; size >= 64; size -= 8) {
    ctx.font = `bold ${size}px ${SERIF}`;
    titleLines = wrapText((t) => ctx.measureText(t).width, cover.title || "Không tên", maxWidth);
    if (titleLines.length <= 5) break;
  }
  const lineHeight = size * 1.25;
  const titleTop = COVER_HEIGHT * 0.42 - ((titleLines.length - 1) * lineHeight) / 2;
  ctx.fillStyle = "#f4ead5";
  titleLines.forEach((line, index) => ctx.fillText(line, COVER_WIDTH / 2, titleTop + index * lineHeight));

  let y = titleTop + titleLines.length * lineHeight + 40;
  if (cover.originalTitle) {
    ctx.font = `60px ${SERIF}`;
    ctx.fillStyle = "rgba(226, 196, 140, 0.95)";
    for (const line of wrapText((t) => ctx.measureText(t).width, cover.originalTitle, maxWidth)) {
      ctx.fillText(line, COVER_WIDTH / 2, y);
      y += 76;
    }
  }
  ctx.fillStyle = "rgba(226, 196, 140, 0.6)";
  ctx.fillRect(COVER_WIDTH / 2 - 120, y + 20, 240, 3);

  if (cover.author) {
    ctx.font = `56px ${SERIF}`;
    ctx.fillStyle = "#f4ead5";
    ctx.fillText(cover.author, COVER_WIDTH / 2, COVER_HEIGHT - 260, maxWidth);
  }
  return canvasToJpeg(canvas);
}
