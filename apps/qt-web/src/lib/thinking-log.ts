/**
 * Log "quá trình suy nghĩ" tích lũy qua các lượt gọi model của một lần dịch
 * (dịch → dịch lại → các vòng soát), mỗi lượt một header riêng.
 */

export function appendThinking(log: string, label: string, text: string): string {
  if (!text) return log;
  const segment = `── ${label} ──\n${text}`;
  return log ? `${log}\n\n${segment}` : segment;
}

export interface ThinkingSegment {
  label: string;
  text: string;
}

const SEGMENT_HEADER = /^── (.+) ──$/;

/** Tách log do `appendThinking` dựng ngược lại thành từng lượt có nhãn. */
export function parseThinkingLog(log: string): ThinkingSegment[] {
  if (!log) return [];
  const segments: ThinkingSegment[] = [];
  let current: ThinkingSegment | undefined;
  for (const line of log.split("\n")) {
    const header = SEGMENT_HEADER.exec(line);
    if (header) {
      current = { label: header[1], text: "" };
      segments.push(current);
      continue;
    }
    if (!current) {
      current = { label: "", text: "" };
      segments.push(current);
    }
    current.text += current.text ? `\n${line}` : line;
  }
  return segments.map(({ label, text }) => ({ label, text: text.trim() }));
}

export interface ScrollMetrics {
  scrollTop: number;
  scrollHeight: number;
  clientHeight: number;
}

/**
 * Bám đáy khi đã ở gần đáy; chỉ nhả khi scrollTop thật sự giảm (người dùng
 * cuộn lên). Content mọc thêm làm scrollHeight tăng nhưng scrollTop đứng yên
 * — scroll event chen giữa lúc đó không được phép nhả pin, tránh vụ auto
 * scroll chết ngẫu nhiên khi chunk mới về trước khi effect kịp kéo xuống.
 */
export function thinkingPinAfterScroll(
  pinned: boolean,
  lastScrollTop: number,
  node: ScrollMetrics,
): boolean {
  if (node.scrollHeight - node.scrollTop - node.clientHeight < 24) return true;
  if (node.scrollTop < lastScrollTop) return false;
  return pinned;
}
