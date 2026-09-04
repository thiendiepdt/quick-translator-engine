import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

/** Dựng folder truyện tạm: raw/ với các chương cho trước. */
export function makeStoryDir(chapters: Record<string, string>): string {
  const root = mkdtempSync(join(tmpdir(), "qt-ai-test-"));
  mkdirSync(join(root, "raw"), { recursive: true });
  for (const [id, text] of Object.entries(chapters)) {
    writeFileSync(join(root, "raw", `${id}.txt`), text, "utf8");
  }
  return root;
}
