import { cleanup, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { toast } from "sonner";

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
    const translations = [
      () => firstResponse,
      () => Promise.resolve(geminiSse("Bản dịch chương hai.")),
    ];
    let streamCalls = 0;
    const fetchSpy = vi.spyOn(globalThis, "fetch").mockImplementation((input) => {
      const url = typeof input === "string" ? input : input instanceof URL ? input.href : input.url;
      if (url.includes(":streamGenerateContent")) {
        streamCalls += 1;
        return translations.shift()?.() ?? Promise.reject(new Error("hết mock dịch"));
      }
      // Call trích glossary sau mỗi chương — trả rỗng cho test này.
      return Promise.resolve(
        Response.json({
          candidates: [{ content: { parts: [{ text: JSON.stringify({ entries: [] }) }] } }],
        }),
      );
    });
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

    await waitFor(() => expect(streamCalls).toBe(1));
    expect(fetchSpy).toHaveBeenCalled();
    resolveFirst?.(geminiSse("Bản dịch chương một."));
    await waitFor(() => expect(streamCalls).toBe(2));
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

  it("shows the chapter sidebar with a default chuong-0 entry before any import", () => {
    render(
      <AiTranslationWorkspace
        aiSettings={defaultAiSettings}
        onOpenSettings={vi.fn()}
      />,
    );

    expect(screen.getByText("chuong-0")).toBeInTheDocument();
    expect(screen.getByText("Dán nguyên văn hoặc nhập file")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Nhập chương từ file" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Xóa toàn bộ chương" })).toBeDisabled();
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
            autoGlossary: true,
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

describe("thinking log", () => {
  function geminiThinkingSse(thinking: string, text: string): Response {
    const payload = {
      candidates: [
        { content: { parts: [{ text: thinking, thought: true }, { text }] } },
      ],
    };
    return new Response(`data: ${JSON.stringify(payload)}\n\ndata: [DONE]\n\n`);
  }

  const settingsWithKey = {
    ...defaultAiSettings,
    gemini: { ...defaultAiSettings.gemini, apiKey: "AIza-test" },
    translation: {
      ...defaultAiSettings.translation,
      provider: "gemini" as const,
      thinking: true,
    },
  };

  it("accumulates thinking across translate and review rounds and copies it all", async () => {
    vi.spyOn(globalThis, "fetch")
      // Bản dịch dính rule "thập phần" → chạy một vòng soát.
      .mockResolvedValueOnce(geminiThinkingSse("suy nghĩ dịch", "hắn thập phần cao hứng"))
      .mockResolvedValueOnce(geminiThinkingSse("suy nghĩ soát", "hắn vô cùng cao hứng"));
    const user = userEvent.setup();
    render(
      <AiTranslationWorkspace aiSettings={settingsWithKey} onOpenSettings={vi.fn()} />,
    );

    useWorkspaceStore.getState().setAiTranslationSource("他十分高兴");
    await user.click(screen.getByRole("button", { name: /Dịch AI/ }));

    await waitFor(() => {
      expect(useWorkspaceStore.getState().aiTranslationOutput).toContain("vô cùng");
    });
    const log = useWorkspaceStore.getState().aiTranslationThinking;
    expect(log).toContain("── Dịch ──");
    expect(log).toContain("suy nghĩ dịch");
    expect(log).toContain("── Soát lần 1 ──");
    expect(log).toContain("suy nghĩ soát");

    await user.click(screen.getByRole("button", { name: "Sao chép quá trình suy nghĩ" }));
    expect(await navigator.clipboard.readText()).toBe(log);
  });

  it("shows the thinking log in a dialog", async () => {
    vi.spyOn(globalThis, "fetch")
      .mockResolvedValueOnce(geminiThinkingSse("suy nghĩ dịch", "hắn vô cùng cao hứng"));
    const user = userEvent.setup();
    render(
      <AiTranslationWorkspace aiSettings={settingsWithKey} onOpenSettings={vi.fn()} />,
    );

    useWorkspaceStore.getState().setAiTranslationSource("他十分高兴");
    await user.click(screen.getByRole("button", { name: /Dịch AI/ }));
    await waitFor(() => {
      expect(useWorkspaceStore.getState().aiTranslationOutput).toContain("vô cùng");
    });

    await user.click(screen.getByRole("button", { name: "Mở quá trình suy nghĩ" }));
    const dialog = await screen.findByRole("dialog");
    expect(within(dialog).getByText(/suy nghĩ dịch/)).toBeInTheDocument();
    // Header thô của log phải được render thành chip nhãn, không lộ nguyên văn.
    expect(within(dialog).queryByText(/── Dịch ──/)).not.toBeInTheDocument();
    expect(within(dialog).getByText("Dịch")).toBeInTheDocument();
  });
});

describe("auto glossary loop", () => {
  it("skips extraction entirely when the auto glossary toggle is off", async () => {
    const fetchSpy = mockTranslateAndExtractSpy(() =>
      Promise.reject(new Error("không được gọi trích")),
    );
    const user = userEvent.setup();
    render(
      <AiTranslationWorkspace
        aiSettings={{
          ...settingsWithKeyBase,
          translation: { ...settingsWithKeyBase.translation, autoGlossary: false },
        }}
        onOpenSettings={vi.fn()}
      />,
    );

    useWorkspaceStore.getState().setAiTranslationSource("震雷子看向太清山。");
    await user.click(screen.getByRole("button", { name: /Dịch AI/ }));
    await waitFor(() => {
      expect(useWorkspaceStore.getState().aiTranslationOutput).toContain("Chấn Lôi Tử");
    });
    const extractionCalls = fetchSpy.mock.calls.filter(([input]) => {
      const url = typeof input === "string" ? input : input instanceof URL ? input.href : (input as Request).url;
      return !url.includes(":streamGenerateContent");
    });
    expect(extractionCalls).toHaveLength(0);
    expect(useWorkspaceStore.getState().aiStory.autoGlossaryLog).toEqual([]);
  });

  const settingsWithKeyBase = {
    ...defaultAiSettings,
    gemini: { ...defaultAiSettings.gemini, apiKey: "AIza-test" },
    translation: {
      ...defaultAiSettings.translation,
      provider: "gemini" as const,
      thinking: false,
    },
  };
  const settingsWithKey = settingsWithKeyBase;

  function mockTranslateAndExtractSpy(extraction: () => Promise<Response>) {
    return vi.spyOn(globalThis, "fetch").mockImplementation((input) => {
      const url = typeof input === "string" ? input : input instanceof URL ? input.href : input.url;
      if (url.includes(":streamGenerateContent")) {
        return Promise.resolve(geminiSse("Chấn Lôi Tử nhìn về Thái Thanh Sơn."));
      }
      return extraction();
    });
  }
  const mockTranslateAndExtract = mockTranslateAndExtractSpy;

  it("feeds new names from the finished translation back into the story glossary", async () => {
    mockTranslateAndExtract(() =>
      Promise.resolve(
        Response.json({
          candidates: [{
            content: {
              parts: [{
                text: JSON.stringify({
                  entries: [
                    { source: "震雷子", target: "Chấn Lôi Tử", category: "names" },
                    { source: "太清山", target: "Thái Thanh Sơn", category: "places" },
                  ],
                }),
              }],
            },
          }],
        }),
      ),
    );
    const user = userEvent.setup();
    render(
      <AiTranslationWorkspace aiSettings={settingsWithKey} onOpenSettings={vi.fn()} />,
    );

    const toastSpy = vi.spyOn(toast, "message");
    useWorkspaceStore.getState().setAiTranslationSource("震雷子看向太清山。");
    await user.click(screen.getByRole("button", { name: /Dịch AI/ }));

    await waitFor(() => {
      expect(useWorkspaceStore.getState().aiStory.autoGlossaryLog).toHaveLength(2);
    });
    expect(toastSpy).toHaveBeenCalledWith(
      "chuong-0: +2 tên vào từ điển truyện",
      expect.objectContaining({ description: expect.stringContaining("震雷子 → Chấn Lôi Tử") }),
    );
    const story = useWorkspaceStore.getState().aiStory;
    expect(story.glossary.names["震雷子"]).toBe("Chấn Lôi Tử");
    expect(story.glossary.places["太清山"]).toBe("Thái Thanh Sơn");
    expect(story.autoGlossaryLog[0].chapter).toBe("chuong-0");

    // Badge mở dialog duyệt, gỡ được entry khỏi cả glossary lẫn log.
    await user.click(screen.getByRole("button", { name: /2 tên tự thêm/ }));
    const dialog = await screen.findByRole("dialog");
    expect(within(dialog).getByText("震雷子")).toBeInTheDocument();
    await user.click(within(dialog).getByRole("button", { name: "Gỡ 震雷子" }));
    await waitFor(() => {
      expect(useWorkspaceStore.getState().aiStory.autoGlossaryLog).toHaveLength(1);
    });
    expect(useWorkspaceStore.getState().aiStory.glossary.names["震雷子"]).toBeUndefined();
  });

  it("ships only glossary entries the chapter touches and filters the exclude list", async () => {
    const fetchSpy = mockTranslateAndExtractSpy(() =>
      Promise.resolve(
        Response.json({
          candidates: [{ content: { parts: [{ text: JSON.stringify({ entries: [] }) }] } }],
        }),
      ),
    );
    const story = emptyAiStoryConfig();
    story.glossary.names["震雷子"] = "Chấn Lôi Tử";
    story.glossary.names["萧炎"] = "Tiêu Viêm";
    useWorkspaceStore.getState().updateAiStory(story);
    const user = userEvent.setup();
    render(
      <AiTranslationWorkspace aiSettings={settingsWithKey} onOpenSettings={vi.fn()} />,
    );

    useWorkspaceStore.getState().setAiTranslationSource("震雷子看向太清山。");
    await user.click(screen.getByRole("button", { name: /Dịch AI/ }));
    await waitFor(() => {
      expect(useWorkspaceStore.getState().aiTranslationOutput).toContain("Chấn Lôi Tử");
    });

    const bodies = fetchSpy.mock.calls.map(([input, init]) => ({
      url: typeof input === "string" ? input : input instanceof URL ? input.href : (input as Request).url,
      body: JSON.parse(init?.body as string) as {
        systemInstruction?: { parts: Array<{ text: string }> };
        contents: Array<{ parts: Array<{ text: string }> }>;
      },
    }));
    const translateBody = bodies.find(({ url }) => url.includes(":streamGenerateContent"))?.body;
    const systemText = translateBody?.systemInstruction?.parts[0].text ?? "";
    expect(systemText).toContain("震雷子");
    expect(systemText).not.toContain("萧炎");

    const extractBody = bodies.find(({ url }) => !url.includes(":streamGenerateContent"))?.body;
    const extractUser = extractBody?.contents[0].parts[0].text ?? "";
    expect(extractUser).toContain("震雷子");
    expect(extractUser).not.toContain("萧炎");
  });

  it("lets the story setting override the AI settings toggle in both directions", async () => {
    // Toggle chung tắt nhưng truyện bật → vẫn trích.
    mockTranslateAndExtractSpy(() =>
      Promise.resolve(
        Response.json({
          candidates: [{
            content: {
              parts: [{
                text: JSON.stringify({
                  entries: [{ source: "震雷子", target: "Chấn Lôi Tử", category: "names" }],
                }),
              }],
            },
          }],
        }),
      ),
    );
    useWorkspaceStore.getState().updateAiStory({ autoGlossary: "on" });
    const user = userEvent.setup();
    render(
      <AiTranslationWorkspace
        aiSettings={{
          ...settingsWithKeyBase,
          translation: { ...settingsWithKeyBase.translation, autoGlossary: false },
        }}
        onOpenSettings={vi.fn()}
      />,
    );
    useWorkspaceStore.getState().setAiTranslationSource("震雷子看向太清山。");
    await user.click(screen.getByRole("button", { name: /Dịch AI/ }));
    await waitFor(() => {
      expect(useWorkspaceStore.getState().aiStory.autoGlossaryLog).toHaveLength(1);
    });
  });

  it("story off beats the global on toggle", async () => {
    const fetchSpy = mockTranslateAndExtractSpy(() =>
      Promise.reject(new Error("không được gọi trích")),
    );
    useWorkspaceStore.getState().updateAiStory({ autoGlossary: "off" });
    const user = userEvent.setup();
    render(
      <AiTranslationWorkspace aiSettings={settingsWithKey} onOpenSettings={vi.fn()} />,
    );
    useWorkspaceStore.getState().setAiTranslationSource("震雷子看向太清山。");
    await user.click(screen.getByRole("button", { name: /Dịch AI/ }));
    await waitFor(() => {
      expect(useWorkspaceStore.getState().aiTranslationOutput).toContain("Chấn Lôi Tử");
    });
    const extractionCalls = fetchSpy.mock.calls.filter(([input]) => {
      const url = typeof input === "string" ? input : input instanceof URL ? input.href : (input as Request).url;
      return !url.includes(":streamGenerateContent");
    });
    expect(extractionCalls).toHaveLength(0);
  });

  it("keeps the chapter done when glossary extraction fails", async () => {
    mockTranslateAndExtract(() => Promise.reject(new TypeError("Failed to fetch")));
    const user = userEvent.setup();
    render(
      <AiTranslationWorkspace aiSettings={settingsWithKey} onOpenSettings={vi.fn()} />,
    );

    useWorkspaceStore.getState().setAiTranslationSource("震雷子看向太清山。");
    await user.click(screen.getByRole("button", { name: /Dịch AI/ }));

    await waitFor(() => {
      expect(useWorkspaceStore.getState().aiTranslationOutput).toContain("Chấn Lôi Tử");
    });
    expect(useWorkspaceStore.getState().aiStory.autoGlossaryLog).toEqual([]);
  });
});

describe("chapter list ordinals", () => {
  it("shows the 1-based order used by the download dialog", () => {
    useWorkspaceStore.getState().importAiTranslationChapters([
      { filename: "mo-dau.txt", source: "第一章" },
      { filename: "than-binh.txt", source: "第二章" },
    ]);
    render(
      <AiTranslationWorkspace aiSettings={defaultAiSettings} onOpenSettings={vi.fn()} />,
    );
    // Accessible name của hàng giờ mở đầu bằng số thứ tự; nút "Xóa …" không khớp.
    expect(
      screen.getByRole("button", { name: /^1\s*mo-dau\.txt/ }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /^2\s*than-binh\.txt/ }),
    ).toBeInTheDocument();
  });
});

describe("auto glossary dialog pagination", () => {
  it("renders 50 entries per page with pager controls", async () => {
    useWorkspaceStore.getState().updateAiStory({
      autoGlossaryLog: Array.from({ length: 120 }, (_, index) => ({
        source: `源${index + 1}`,
        target: `Nguồn ${index + 1}`,
        category: "names" as const,
        chapter: "chuong-1.txt",
      })),
    });
    const user = userEvent.setup();
    render(
      <AiTranslationWorkspace aiSettings={defaultAiSettings} onOpenSettings={vi.fn()} />,
    );

    await user.click(screen.getByRole("button", { name: /120 tên tự thêm/ }));
    const dialog = await screen.findByRole("dialog");
    expect(within(dialog).getAllByRole("listitem")).toHaveLength(50);
    expect(within(dialog).getByText("源1")).toBeInTheDocument();
    expect(within(dialog).queryByText("源51")).not.toBeInTheDocument();

    await user.click(within(dialog).getByRole("button", { name: "Trang sau" }));
    expect(within(dialog).getByText("源51")).toBeInTheDocument();
    expect(within(dialog).queryByText("源1")).not.toBeInTheDocument();

    // Trang cuối chỉ còn phần dư.
    await user.click(within(dialog).getByRole("button", { name: "Trang 3" }));
    expect(within(dialog).getAllByRole("listitem")).toHaveLength(20);
  });

  it("searches across source and target and repages the matches", async () => {
    useWorkspaceStore.getState().updateAiStory({
      autoGlossaryLog: [
        ...Array.from({ length: 60 }, (_, index) => ({
          source: `源${index + 1}`,
          target: `Nguồn ${index + 1}`,
          category: "names" as const,
          chapter: "chuong-1.txt",
        })),
        {
          source: "震雷子",
          target: "Chấn Lôi Tử",
          category: "names" as const,
          chapter: "chuong-2.txt",
        },
      ],
    });
    const user = userEvent.setup();
    render(
      <AiTranslationWorkspace aiSettings={defaultAiSettings} onOpenSettings={vi.fn()} />,
    );

    await user.click(screen.getByRole("button", { name: /61 tên tự thêm/ }));
    const dialog = await screen.findByRole("dialog");
    const search = within(dialog).getByPlaceholderText("Tìm tên…");

    // Khớp target, không phân biệt hoa thường.
    await user.type(search, "chấn lôi");
    expect(within(dialog).getAllByRole("listitem")).toHaveLength(1);
    expect(within(dialog).getByText("震雷子")).toBeInTheDocument();
    expect(within(dialog).queryByRole("button", { name: "Trang sau" })).not.toBeInTheDocument();

    // Khớp source, đang ở trang nào cũng nhảy về trang 1 của kết quả.
    await user.clear(search);
    await user.type(search, "源6");
    // 源6 và 源60: đúng 2 mục.
    expect(within(dialog).getAllByRole("listitem")).toHaveLength(2);

    await user.clear(search);
    await user.type(search, "không-tồn-tại");
    expect(within(dialog).queryAllByRole("listitem")).toHaveLength(0);
    expect(within(dialog).getByText(/Không có tên nào khớp/)).toBeInTheDocument();
  });
});

describe("chapter reset", () => {
  it("requeues a done chapter while keeping its output until retranslation", async () => {
    useWorkspaceStore.getState().importAiTranslationChapters([
      { filename: "chuong-1.txt", source: "第一章" },
    ]);
    const chapter = useWorkspaceStore.getState().aiTranslationChapters[0];
    useWorkspaceStore.getState().updateAiTranslationChapter(chapter.id, {
      status: "done",
      output: "Bản dịch cũ.\n",
      reviewRound: 2,
    });
    const user = userEvent.setup();
    render(
      <AiTranslationWorkspace aiSettings={defaultAiSettings} onOpenSettings={vi.fn()} />,
    );

    await user.click(screen.getByRole("button", { name: "Dịch lại chuong-1.txt" }));

    const updated = useWorkspaceStore.getState().aiTranslationChapters[0];
    expect(updated.status).toBe("queued");
    expect(updated.reviewRound).toBe(0);
    expect(updated.output).toBe("Bản dịch cũ.\n");
    expect(screen.getByRole("button", { name: /Dịch hàng đợi · 1/ })).toBeEnabled();
  });

  it("offers no reset for chapters that were never translated", () => {
    useWorkspaceStore.getState().importAiTranslationChapters([
      { filename: "chuong-1.txt", source: "第一章" },
    ]);
    render(
      <AiTranslationWorkspace aiSettings={defaultAiSettings} onOpenSettings={vi.fn()} />,
    );
    expect(
      screen.queryByRole("button", { name: "Dịch lại chuong-1.txt" }),
    ).not.toBeInTheDocument();
  });
});

describe("download translated chapters", () => {
  function stubDownloadSinks() {
    const blobs: Blob[] = [];
    const createObjectURL = vi.fn((blob: Blob) => {
      blobs.push(blob);
      return "blob:test";
    });
    const revokeObjectURL = vi.fn();
    vi.stubGlobal("URL", Object.assign(URL, { createObjectURL, revokeObjectURL }));
    const clickSpy = vi
      .spyOn(HTMLAnchorElement.prototype, "click")
      .mockImplementation(() => undefined);
    return { blobs, createObjectURL, revokeObjectURL, clickSpy };
  }

  function seedChapters(
    entries: Array<{ filename: string; done?: boolean; output?: string }>,
  ) {
    useWorkspaceStore.getState().importAiTranslationChapters(
      entries.map(({ filename }) => ({ filename, source: "第一章" })),
    );
    const chapters = useWorkspaceStore.getState().aiTranslationChapters;
    entries.forEach(({ filename, done, output }) => {
      const chapter = chapters.find((item) => item.filename === filename);
      if (!chapter) throw new Error(`thiếu ${filename}`);
      useWorkspaceStore.getState().updateAiTranslationChapter(chapter.id, {
        status: done ? "done" : "queued",
        output: output ?? "",
      });
    });
  }

  it("defaults the range to the done chapters and zips the selection", async () => {
    const { blobs, clickSpy, revokeObjectURL } = stubDownloadSinks();
    seedChapters([
      { filename: "chuong-1.txt" },
      { filename: "chuong-2.txt", done: true, output: "Bản dịch chương hai.\n" },
      { filename: "chuong-3.txt", done: true, output: "Bản dịch chương ba.\n" },
      { filename: "chuong-4.txt" },
    ]);
    const user = userEvent.setup();
    render(
      <AiTranslationWorkspace aiSettings={defaultAiSettings} onOpenSettings={vi.fn()} />,
    );

    await user.click(
      screen.getByRole("button", { name: "Tải các chương đã dịch (.zip)" }),
    );
    const dialog = await screen.findByRole("dialog");
    expect(within(dialog).getByLabelText("Từ chương")).toHaveValue(2);
    expect(within(dialog).getByLabelText("Đến chương")).toHaveValue(3);
    await user.click(within(dialog).getByRole("button", { name: "Tải xuống" }));

    expect(clickSpy).toHaveBeenCalled();
    const bytes = new Uint8Array(await blobs[0].arrayBuffer());
    const eocd = bytes.length - 22;
    expect(bytes[eocd + 10] | (bytes[eocd + 11] << 8)).toBe(2);
    expect(revokeObjectURL).toHaveBeenCalledWith("blob:test");
    clickSpy.mockRestore();
    vi.unstubAllGlobals();
  });

  it("narrows the range and skips undone chapters inside it", async () => {
    const { blobs, clickSpy } = stubDownloadSinks();
    seedChapters([
      { filename: "chuong-1.txt", done: true, output: "Một.\n" },
      { filename: "chuong-2.txt" },
      { filename: "chuong-3.txt", done: true, output: "Ba.\n" },
    ]);
    const user = userEvent.setup();
    render(
      <AiTranslationWorkspace aiSettings={defaultAiSettings} onOpenSettings={vi.fn()} />,
    );

    await user.click(
      screen.getByRole("button", { name: "Tải các chương đã dịch (.zip)" }),
    );
    const dialog = await screen.findByRole("dialog");
    const from = within(dialog).getByLabelText("Từ chương");
    await user.clear(from);
    await user.type(from, "2");
    await user.click(within(dialog).getByRole("button", { name: "Tải xuống" }));

    const bytes = new Uint8Array(await blobs[0].arrayBuffer());
    const eocd = bytes.length - 22;
    // Khoảng 2..3 nhưng chương 2 chưa dịch → chỉ 1 entry.
    expect(bytes[eocd + 10] | (bytes[eocd + 11] << 8)).toBe(1);
    clickSpy.mockRestore();
    vi.unstubAllGlobals();
  });

  it("merges the selection into one text file separated by blank lines", async () => {
    const { blobs, clickSpy } = stubDownloadSinks();
    seedChapters([
      { filename: "chuong-1.txt", done: true, output: "Chương một.\n" },
      { filename: "chuong-2.txt", done: true, output: "Chương hai.\n" },
    ]);
    useWorkspaceStore.getState().updateAiStory({ name: "Đấu Phá" });
    const user = userEvent.setup();
    render(
      <AiTranslationWorkspace aiSettings={defaultAiSettings} onOpenSettings={vi.fn()} />,
    );

    await user.click(
      screen.getByRole("button", { name: "Tải các chương đã dịch (.zip)" }),
    );
    const dialog = await screen.findByRole("dialog");
    await user.click(within(dialog).getByLabelText("Gộp thành 1 file"));
    await user.click(within(dialog).getByRole("button", { name: "Tải xuống" }));

    expect(blobs[0].type).toContain("text/plain");
    expect(await blobs[0].text()).toBe("Chương một.\n\nChương hai.\n");
    clickSpy.mockRestore();
    vi.unstubAllGlobals();
  });

  it("resets the range back to every done chapter", async () => {
    stubDownloadSinks();
    seedChapters([
      { filename: "chuong-1.txt", done: true, output: "Một.\n" },
      { filename: "chuong-2.txt", done: true, output: "Hai.\n" },
      { filename: "chuong-3.txt", done: true, output: "Ba.\n" },
    ]);
    const user = userEvent.setup();
    render(
      <AiTranslationWorkspace aiSettings={defaultAiSettings} onOpenSettings={vi.fn()} />,
    );

    await user.click(
      screen.getByRole("button", { name: "Tải các chương đã dịch (.zip)" }),
    );
    const dialog = await screen.findByRole("dialog");
    const from = within(dialog).getByLabelText("Từ chương");
    await user.clear(from);
    await user.type(from, "3");
    await user.click(within(dialog).getByRole("button", { name: "Chọn hết" }));
    expect(within(dialog).getByLabelText("Từ chương")).toHaveValue(1);
    expect(within(dialog).getByLabelText("Đến chương")).toHaveValue(3);
    vi.unstubAllGlobals();
  });

  it("disables the download button when no chapter is done", () => {
    seedChapters([{ filename: "chuong-1.txt" }]);
    render(
      <AiTranslationWorkspace aiSettings={defaultAiSettings} onOpenSettings={vi.fn()} />,
    );
    expect(
      screen.getByRole("button", { name: "Tải các chương đã dịch (.zip)" }),
    ).toBeDisabled();
  });
});
