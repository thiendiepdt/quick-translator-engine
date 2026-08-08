import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { AiTranslationWorkspace } from "@/components/ai-translation-workspace";
import { defaultAiSettings } from "@/lib/ai-settings";
import { emptyAiStoryConfig } from "@/lib/ai-story";
import { useWorkspaceStore } from "@/store/workspace";

function geminiSse(text: string): Response {
  return new Response(
    `data: ${JSON.stringify({ candidates: [{ content: { parts: [{ text }] } }] })}\n\ndata: [DONE]\n\n`,
  );
}

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

beforeEach(() => {
  useWorkspaceStore.getState().clearAiTranslationChapters();
  useWorkspaceStore.getState().clearAiTranslation();
  useWorkspaceStore.getState().updateAiStory(emptyAiStoryConfig());
  useWorkspaceStore.setState({ sourceText: "Nội dung Convert không được dùng" });
});

describe("AI translation workspace", () => {
  it("imports multiple chapter files into a naturally sorted queue", async () => {
    const user = userEvent.setup();
    render(
      <AiTranslationWorkspace
        aiSettings={defaultAiSettings}
        onOpenSettings={vi.fn()}
      />,
    );

    const input = screen.getByLabelText("Chọn tệp chương");
    await user.upload(input, [
      new File(["第十章"], "chuong-10.txt", { type: "text/plain" }),
      new File(["第二章"], "chuong-2.txt", { type: "text/plain" }),
    ]);

    await waitFor(() => {
      expect(
        useWorkspaceStore.getState().aiTranslationChapters.map(({ filename }) => filename),
      ).toEqual(["chuong-2.txt", "chuong-10.txt"]);
    });
    expect(screen.getByRole("button", { name: /Dịch hàng đợi · 2/ })).toBeEnabled();
  });

  it("saves story information from the configuration dialog", async () => {
    const user = userEvent.setup();
    render(
      <AiTranslationWorkspace
        aiSettings={defaultAiSettings}
        onOpenSettings={vi.fn()}
      />,
    );

    await user.click(screen.getByRole("button", { name: "Cấu hình truyện" }));
    await user.type(screen.getByLabelText("Tên truyện"), "Đấu Phá Thương Khung");
    await user.click(screen.getByRole("button", { name: "Lưu cấu hình" }));

    expect(useWorkspaceStore.getState().aiStory.name).toBe("Đấu Phá Thương Khung");
  });

  it("translates uploaded chapters sequentially", async () => {
    let resolveFirst: ((response: Response) => void) | undefined;
    const firstResponse = new Promise<Response>((resolve) => {
      resolveFirst = resolve;
    });
    const fetchSpy = vi.spyOn(globalThis, "fetch")
      .mockImplementationOnce(() => firstResponse)
      .mockResolvedValueOnce(geminiSse("Bản dịch chương hai."));
    const user = userEvent.setup();
    render(
      <AiTranslationWorkspace
        aiSettings={{
          ...defaultAiSettings,
          gemini: { ...defaultAiSettings.gemini, apiKey: "AIza-test" },
          translation: {
            ...defaultAiSettings.translation,
            provider: "gemini",
            thinking: false,
          },
        }}
        onOpenSettings={vi.fn()}
      />,
    );

    await user.upload(screen.getByLabelText("Chọn tệp chương"), [
      new File(["第一章"], "chuong-1.txt", { type: "text/plain" }),
      new File(["第二章"], "chuong-2.txt", { type: "text/plain" }),
    ]);
    await user.click(await screen.findByRole("button", { name: /Dịch hàng đợi · 2/ }));

    await waitFor(() => expect(fetchSpy).toHaveBeenCalledTimes(1));
    resolveFirst?.(geminiSse("Bản dịch chương một."));
    await waitFor(() => expect(fetchSpy).toHaveBeenCalledTimes(2));
    await waitFor(() => {
      expect(
        useWorkspaceStore.getState().aiTranslationChapters.map(({ status }) => status),
      ).toEqual(["done", "done"]);
    });
  });

  it("keeps the output pane read-only until Sửa is toggled", async () => {
    const user = userEvent.setup();
    render(
      <AiTranslationWorkspace
        aiSettings={defaultAiSettings}
        onOpenSettings={vi.fn()}
      />,
    );

    expect(
      screen.queryByPlaceholderText(/Bản dịch AI sẽ xuất hiện/),
    ).not.toBeInTheDocument();
    expect(screen.getByText(/mặc định chỉ đọc/)).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Sửa" }));
    const output = screen.getByPlaceholderText(/Bản dịch AI sẽ xuất hiện/);
    expect(output).toBeEnabled();
    expect(output.closest("section")).toHaveClass("flex", "flex-col");

    await user.click(screen.getByRole("button", { name: "Xong" }));
    expect(
      screen.queryByPlaceholderText(/Bản dịch AI sẽ xuất hiện/),
    ).not.toBeInTheDocument();
  });

  it("maps a translated paragraph back to its source paragraph on click", async () => {
    const user = userEvent.setup();
    useWorkspaceStore.setState({
      aiTranslationSource: "第一段\n第二段",
      aiTranslationOutput: "Đoạn một.\n\nĐoạn hai.\n",
    });
    render(
      <AiTranslationWorkspace
        aiSettings={defaultAiSettings}
        onOpenSettings={vi.fn()}
      />,
    );

    await user.click(screen.getByRole("button", { name: "Range 2: Đoạn hai." }));

    const sourceSegment = screen.getByRole("button", { name: "Range 2: 第二段" });
    expect(sourceSegment).toHaveAttribute("data-active", "true");
    expect(
      screen.getByRole("button", { name: "Range 2: Đoạn hai." }),
    ).toHaveAttribute("data-active", "true");
  });

  it("keeps its source/output frame separate from Convert", async () => {
    const user = userEvent.setup();
    render(
      <AiTranslationWorkspace
        aiSettings={defaultAiSettings}
        onOpenSettings={vi.fn()}
      />,
    );

    const source = screen.getByPlaceholderText(/Dán nguyên văn tiếng Trung/);
    await user.type(source, "萧炎走来。 ");

    expect(useWorkspaceStore.getState().aiTranslationSource).toBe("萧炎走来。 ");
    expect(useWorkspaceStore.getState().sourceText).toBe(
      "Nội dung Convert không được dùng",
    );
  });

  it("uses the translation model and thinking setting while sharing the provider key", async () => {
    const fetchSpy = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(
        `data: ${JSON.stringify({ choices: [{ delta: { content: "Tiêu Viêm bước tới." } }] })}\n\ndata: [DONE]\n\n`,
      ),
    );
    const user = userEvent.setup();
    render(
      <AiTranslationWorkspace
        aiSettings={{
          ...defaultAiSettings,
          provider: "deepseek",
          deepseek: {
            apiKey: "sk-shared",
            model: "deepseek-name-filter",
            baseUrl: "",
          },
          translation: {
            provider: "deepseek",
            models: {
              ...defaultAiSettings.translation.models,
              deepseek: "deepseek-translate",
            },
            thinking: false,
          },
        }}
        onOpenSettings={vi.fn()}
      />,
    );

    await user.type(screen.getByPlaceholderText(/Dán nguyên văn tiếng Trung/), "萧炎走来。 ");
    await user.click(screen.getByRole("button", { name: /^Dịch AI/ }));

    await waitFor(() => {
      expect(useWorkspaceStore.getState().aiTranslationOutput).toBe(
        "Tiêu Viêm bước tới.\n",
      );
    });
    const [, init] = fetchSpy.mock.calls[0];
    expect((init?.headers as Record<string, string>).authorization).toBe(
      "Bearer sk-shared",
    );
    const body = JSON.parse(init?.body as string) as {
      model: string;
      thinking: { type: string };
    };
    expect(body.model).toBe("deepseek-translate");
    expect(body.thinking.type).toBe("disabled");
  });
});
