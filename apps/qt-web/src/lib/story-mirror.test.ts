import { beforeEach, describe, expect, it } from "vitest";

import {
  aiStoryIsEmpty,
  readStoryMirror,
  storyMirrorKeyFor,
  writeStoryMirror,
} from "@/lib/story-mirror";
import { emptyAiStoryConfig } from "@/lib/ai-story";

const STORAGE_KEY = "qt-web-workspace-v1";

beforeEach(() => {
  window.localStorage.clear();
});

describe("aiStoryIsEmpty", () => {
  it("treats a pristine config as empty", () => {
    expect(aiStoryIsEmpty(emptyAiStoryConfig())).toBe(true);
  });

  it("treats any user content as non-empty", () => {
    const named = { ...emptyAiStoryConfig(), name: "Đấu Phá" };
    expect(aiStoryIsEmpty(named)).toBe(false);
    const withGlossary = emptyAiStoryConfig();
    withGlossary.glossary.names["萧炎"] = "Tiêu Viêm";
    expect(aiStoryIsEmpty(withGlossary)).toBe(false);
    const withSetting = { ...emptyAiStoryConfig(), autoGlossary: "off" as const };
    expect(aiStoryIsEmpty(withSetting)).toBe(false);
  });
});

describe("story mirror round-trip", () => {
  it("stores under a key derived from the workspace storage key", () => {
    const story = { ...emptyAiStoryConfig(), name: "Đấu Phá" };
    writeStoryMirror(STORAGE_KEY, story);
    expect(
      window.localStorage.getItem(storyMirrorKeyFor(STORAGE_KEY)),
    ).toBeTruthy();
    expect(readStoryMirror(STORAGE_KEY)?.name).toBe("Đấu Phá");
  });

  it("returns undefined for missing or corrupted mirrors", () => {
    expect(readStoryMirror(STORAGE_KEY)).toBeUndefined();
    window.localStorage.setItem(storyMirrorKeyFor(STORAGE_KEY), "{hỏng");
    expect(readStoryMirror(STORAGE_KEY)).toBeUndefined();
    window.localStorage.setItem(storyMirrorKeyFor(STORAGE_KEY), '"chuỗi"');
    expect(readStoryMirror(STORAGE_KEY)).toBeUndefined();
  });

  it("normalizes legacy mirrors on read", () => {
    window.localStorage.setItem(
      storyMirrorKeyFor(STORAGE_KEY),
      JSON.stringify({ name: "Cũ" }),
    );
    const restored = readStoryMirror(STORAGE_KEY);
    expect(restored?.name).toBe("Cũ");
    expect(restored?.autoGlossary).toBe("inherit");
    expect(restored?.autoGlossaryLog).toEqual([]);
  });
});
