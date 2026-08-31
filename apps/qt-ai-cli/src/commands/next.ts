import { existsSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { aiParagraphsOf, labeledAiSourcePayload } from "@/lib/ai-paragraphs";
import { buildAiTranslationSystemPrompt } from "@/lib/ai-translation";
import { naturalChapterCompare, type AiStoryConfig } from "@/lib/ai-story";
import {
  loadState, loadStoryConfig, readRawChapter, saveState, storyPaths, workFile,
  type StoryPaths,
} from "../story-fs.ts";

function agentInstructions(id: string): string {
  return [
    "# Việc của agent sau khi dịch xong",
    "",
    `1. Ghi bản dịch tiếng Việt (GIỮ NGUYÊN nhãn [[n]] đầu mỗi đoạn) vào work/${id}.draft.md.`,
    `2. Ghi đề xuất TÊN RIÊNG mới (nhân vật, địa danh, đồ vật/vũ khí, sinh vật, công pháp/kỹ năng)`,
    `   xuất hiện trong raw nhưng chưa có trong từ điển của prompt vào work/${id}.glossary.json, dạng:`,
    `   {"entries": [{"source": "chữ Hán trong raw", "target": "chép nguyên văn từ bản dịch", "category": "names|places|items|creatures|skills"}]}`,
    `   Bỏ qua từ chung, chức danh, đại từ. Không có tên mới thì ghi {"entries": []}.`,
    `3. Chạy: qt-ai check ${id} (xem AGENTS.md để biết lệnh đầy đủ).`,
  ].join("\n");
}

/** Sinh + ghi work/<id>.prompt.md cho chương `id`; trả về đường dẫn đã ghi. */
function writePromptFile(paths: StoryPaths, story: AiStoryConfig, id: string): string {
  const source = readRawChapter(paths, id);
  const system = buildAiTranslationSystemPrompt({}, story, source);
  const payload = labeledAiSourcePayload(aiParagraphsOf(source));
  const prompt = `${system}\n\n---\n\n${payload}\n\n---\n\n${agentInstructions(id)}\n`;
  const promptPath = workFile(paths, id, "prompt");
  writeFileSync(promptPath, prompt, "utf8");
  return promptPath;
}

export function runNext(root: string): { chapterId: string; promptPath: string } {
  const paths = storyPaths(resolve(root));
  const state = loadState(paths);
  const story = loadStoryConfig(paths);

  const pending = Object.entries(state.chapters)
    .filter(([, chapter]) => chapter.status === "translating")
    .map(([id]) => id);
  if (pending.length > 0) {
    // Session trước chết mất work/*.prompt.md của chương đang translating: chương đó
    // không thể tiếp tục (agent không có prompt để đọc) và cũng không đứng chặn được
    // (workflow không tự sinh lại) — phát lại đúng chương này thay vì chặn cứng, không
    // đụng vào reviewRound/status hiện có (chương vẫn ở "translating" như trước).
    const lostId = pending.find((id) => !existsSync(workFile(paths, id, "prompt")));
    if (lostId) {
      const promptPath = writePromptFile(paths, story, lostId);
      return { chapterId: lostId, promptPath };
    }
    throw new Error(
      `Chương ${pending.join(", ")} đang translating chưa chốt — chạy check/accept/skip trước khi lấy chương mới.`,
    );
  }

  const nextId = Object.entries(state.chapters)
    .filter(([, chapter]) => chapter.status === "queued")
    .map(([id]) => id)
    .sort(naturalChapterCompare)[0];
  if (!nextId) throw new Error("Không còn chương nào trong hàng đợi — chạy qt-ai status để xem tổng kết.");

  const promptPath = writePromptFile(paths, story, nextId);
  state.chapters[nextId] = { status: "translating", reviewRound: 0, updatedAt: Date.now() };
  saveState(paths, state);
  return { chapterId: nextId, promptPath };
}
