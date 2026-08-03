import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

import { DictionaryUpdateDialog } from "@/components/dictionary-update-dialog";
import { defaultAiSettings } from "@/lib/ai-settings";
import {
  dictionaryUpdateKeys,
  type LocalDictionaryEntries,
} from "@/lib/types";

function localEntries(): LocalDictionaryEntries {
  const entries = Object.fromEntries(
    dictionaryUpdateKeys.map((key) => [key, {}]),
  ) as LocalDictionaryEntries;
  entries.chinesePhienAmWords = { 金: "kim" };
  return entries;
}

function renderDialog(options: { ai?: boolean; target?: string } = {}) {
  const settings = {
    ...defaultAiSettings,
    deepseek: {
      ...defaultAiSettings.deepseek,
      apiKey: options.ai ? "sk-test" : "",
    },
  };
  return render(
    <DictionaryUpdateDialog
      open
      endpoint="/api"
      dictionaryKey="names2"
      selection={{ source: "金美婷", target: options.target ?? "Kim đẹp đÌNH" }}
      context="她叫【金美婷】，是金家的女儿。"
      aiSettings={settings}
      localEntries={localEntries()}
      onOpenChange={vi.fn()}
      onSave={vi.fn()}
      onRemove={vi.fn()}
    />,
  );
}

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

describe("dictionary update utilities", () => {
  it("applies all four Vietnamese casing actions", async () => {
    const user = userEvent.setup();
    vi.spyOn(globalThis, "fetch").mockResolvedValue(
      Response.json({ translated: " kim đẹp đình", sourceRanges: [], targetRanges: [] }),
    );
    renderDialog();
    const target = await screen.findByDisplayValue("Kim Đẹp Đình");

    await user.click(screen.getByRole("button", { name: "Hoa Từng Từ" }));
    expect(target).toHaveValue("Kim Đẹp Đình");
    await user.click(screen.getByRole("button", { name: "VIẾT HOA" }));
    expect(target).toHaveValue("KIM ĐẸP ĐÌNH");
    await user.click(screen.getByRole("button", { name: "viết thường" }));
    expect(target).toHaveValue("kim đẹp đình");
    await user.click(screen.getByRole("button", { name: "Viết hoa câu" }));
    expect(target).toHaveValue("Kim đẹp đình");
  });

  it("uses the Han-Viet mode and workspace pronunciation patches", async () => {
    const user = userEvent.setup();
    const fetchSpy = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      Response.json({
        translated: " kim mỹ đình",
        sourceRanges: [],
        targetRanges: [],
      }),
    );
    renderDialog();

    await screen.findByDisplayValue("Kim Mỹ Đình");
    await user.click(screen.getByRole("button", { name: "Dùng âm Hán Việt" }));

    expect(await screen.findByLabelText("Tiếng Việt")).toHaveValue("Kim Mỹ Đình");
    const body = JSON.parse(fetchSpy.mock.calls[0][1]?.body as string) as {
      mode: string;
      dictionaryPatches: { chinesePhienAmWords: Record<string, string> };
    };
    expect(body.mode).toBe("hanviet");
    expect(body.dictionaryPatches.chinesePhienAmWords).toEqual({ 金: "kim" });
  });

  it("shows Lạc Việt meanings inside the dialog", async () => {
    const user = userEvent.setup();
    vi.spyOn(globalThis, "fetch")
      .mockResolvedValueOnce(
        Response.json({ translated: " kim đẹp đình", sourceRanges: [], targetRanges: [] }),
      )
      .mockResolvedValueOnce(
        Response.json({
          entries: [
            { source: "金", definition: "Hán Việt: KIM\n1. vàng" },
            { source: "美", definition: "Hán Việt: MĨ\n1. đẹp" },
          ],
        }),
      );
    renderDialog();

    await screen.findByDisplayValue("Kim Đẹp Đình");
    await user.click(screen.getByRole("button", { name: "Lạc Việt" }));

    expect(await screen.findByText("Nghĩa Lạc Việt")).toBeInTheDocument();
    expect(screen.getByText(/Hán Việt: KIM/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Đóng kết quả tra nghĩa" })).toBeInTheDocument();
  });

  it("sends raw context to AI for translation and meaning lookup", async () => {
    const user = userEvent.setup();
    const fetchSpy = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValueOnce(
        Response.json({
          translated: " kim đẹp đình",
          sourceRanges: [],
          targetRanges: [],
        }),
      )
      .mockResolvedValueOnce(
        Response.json({
          choices: [
            { message: { content: JSON.stringify({ translation: "Kim Mỹ Đình" }) } },
          ],
        }),
      )
      .mockResolvedValueOnce(
        Response.json({
          choices: [
            { message: { content: JSON.stringify({ meaning: "Tên một cô gái họ Kim." }) } },
          ],
        }),
      );
    renderDialog({ ai: true });

    await screen.findByDisplayValue("Kim Đẹp Đình");
    await user.click(screen.getByRole("button", { name: "Dịch bằng AI" }));
    expect(await screen.findByLabelText("Tiếng Việt")).toHaveValue("Kim Mỹ Đình");
    await user.click(screen.getByRole("button", { name: "Tra bằng AI" }));
    expect(await screen.findByText("Tên một cô gái họ Kim.")).toBeInTheDocument();

    for (const call of fetchSpy.mock.calls.slice(1)) {
      const body = JSON.parse(call[1]?.body as string) as {
        messages: Array<{ content: string }>;
      };
      expect(body.messages[1].content).toContain("她叫【金美婷】");
    }
  });
});
