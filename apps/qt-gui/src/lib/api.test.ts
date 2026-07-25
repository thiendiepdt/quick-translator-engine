import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn(),
  open: vi.fn(),
  save: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: mocks.open,
  save: mocks.save,
}));

import {
  fetchDictionaryDefaults,
  openSourceFile,
  translateChapter,
} from "@/lib/api";
import type { TranslationRequest } from "@/lib/types";

const request: TranslationRequest = {
  text: "很好",
  mode: "vietphrase-one",
  wrap: false,
  pretty: true,
  ranges: true,
  scanRange: 30,
  translationAlgorithm: 1,
  prioritizedName: true,
  dictionaries: { names: "很好=Rất tốt" },
  dictionaryPatches: {
    vietPhrase: { 很好: "Ổn lắm" },
    chinesePhienAmWords: { 他: "hắn" },
  },
};

beforeEach(() => {
  vi.clearAllMocks();
});

describe("Tauri command client", () => {
  it("invokes local translation with the complete engine request", async () => {
    mocks.invoke.mockResolvedValueOnce({
      translated: "Rất tốt",
      sourceRanges: [{ start: 0, length: 2 }],
      targetRanges: [{ start: 0, length: 7 }],
    });

    await expect(translateChapter(request)).resolves.toMatchObject({
      translated: "Rất tốt",
    });
    expect(mocks.invoke).toHaveBeenCalledWith("translate", { request });
  });

  it("surfaces command errors without an HTTP layer", async () => {
    mocks.invoke.mockRejectedValueOnce("invalid dictionary");

    await expect(translateChapter(request)).rejects.toEqual(
      expect.objectContaining({
        name: "ApiError",
        message: "invalid dictionary",
      }),
    );
  });

  it("loads and validates editable default dictionaries", async () => {
    const defaults = {
      names: "萧炎=Tiêu Viêm",
      names2: "",
      luatNhan: "",
      pronouns: "她=nàng",
      danhTu: "",
      hoNguoi: "",
      hauTu: "",
      ignoredChinesePhrases: "本章完",
    };
    mocks.invoke.mockResolvedValueOnce(defaults);

    await expect(fetchDictionaryDefaults()).resolves.toEqual(defaults);
    expect(mocks.invoke).toHaveBeenCalledWith("dictionary_defaults", undefined);
  });

  it("opens a selected UTF-8 chapter through a dedicated Rust command", async () => {
    mocks.open.mockResolvedValueOnce("D:\\books\\chapter.txt");
    mocks.invoke.mockResolvedValueOnce({
      path: "D:\\books\\chapter.txt",
      name: "chapter.txt",
      content: "很好",
    });

    await expect(openSourceFile()).resolves.toMatchObject({ content: "很好" });
    expect(mocks.invoke).toHaveBeenCalledWith("read_text_file", {
      path: "D:\\books\\chapter.txt",
    });
  });
});
