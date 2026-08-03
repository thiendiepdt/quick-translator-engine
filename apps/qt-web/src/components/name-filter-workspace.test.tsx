import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { toast } from "sonner";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { NameFilterWorkspace } from "@/components/name-filter-workspace";
import { TooltipProvider } from "@/components/ui/tooltip";
import type { AiSettings } from "@/lib/ai-settings";
import {
  nameApprovalThresholdStorageKey,
  nameFilterModeStorageKey,
} from "@/lib/name-filter-mode";
import { useWorkspaceStore } from "@/store/workspace";

function requestUrlOf(input: RequestInfo | URL): string {
  if (typeof input === "string") return input;
  return input instanceof URL ? input.href : input.url;
}

function requestBodyOf(init?: RequestInit): Record<string, unknown> {
  return JSON.parse(init?.body as string) as Record<string, unknown>;
}

function renderWorkspace(aiSettings?: AiSettings) {
  const queryClient = new QueryClient({
    defaultOptions: { mutations: { retry: false } },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <TooltipProvider>
        <NameFilterWorkspace endpoint="/api" defaultsReady aiSettings={aiSettings} />
      </TooltipProvider>
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  localStorage.clear();
  useWorkspaceStore.getState().clearNameMemory();
  useWorkspaceStore.setState({ nameFilterResponse: undefined, sourceText: "" });
});

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

describe("name filter mode preference", () => {
  it("defaults to QT Legacy and persists that default", async () => {
    renderWorkspace();

    expect(screen.getByRole("tab", { name: "QT cũ" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    await waitFor(() => {
      expect(localStorage.getItem(nameFilterModeStorageKey)).toBe("qt");
    });
  });

  it("restores and updates the saved mode", async () => {
    localStorage.setItem(nameFilterModeStorageKey, "hybrid");
    const user = userEvent.setup();
    renderWorkspace();

    expect(screen.getByRole("tab", { name: "Kết hợp" })).toHaveAttribute(
      "aria-selected",
      "true",
    );

    await user.click(screen.getByRole("tab", { name: "QT cũ" }));
    expect(localStorage.getItem(nameFilterModeStorageKey)).toBe("qt");
  });

  it("allows changing and restoring the approval threshold", async () => {
    localStorage.setItem(nameApprovalThresholdStorageKey, "72");
    const user = userEvent.setup();
    renderWorkspace();

    const threshold = screen.getByRole("spinbutton", { name: "Ngưỡng duyệt (%)" });
    expect(threshold).toHaveValue(72);

    await user.clear(threshold);
    await user.type(threshold, "100");
    await user.tab();

    expect(threshold).toHaveValue(100);
    expect(localStorage.getItem(nameApprovalThresholdStorageKey)).toBe("100");
  });

  it("shows rejected names and restores them to the review queue", async () => {
    useWorkspaceStore.getState().rejectNameCandidate("萧炎");
    const user = userEvent.setup();
    renderWorkspace();

    await user.click(screen.getByRole("tab", { name: "Đã loại 1" }));
    expect(screen.getByText("萧炎")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Khôi phục 萧炎" }));
    expect(useWorkspaceStore.getState().rejectedNames).not.toContain("萧炎");
    expect(screen.getByRole("tab", { name: "Đã loại 0" })).toBeInTheDocument();
  });

  it("highlights rejected names and restores all of them", async () => {
    useWorkspaceStore.getState().rejectNameCandidate("萧炎");
    useWorkspaceStore.getState().rejectNameCandidate("药老");
    const user = userEvent.setup();
    renderWorkspace();

    await user.click(screen.getByRole("button", { name: "Xem 2 tên đã loại" }));
    expect(screen.getByText("萧炎")).toBeInTheDocument();
    expect(screen.getByText("药老")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Khôi phục tất cả" }));
    expect(useWorkspaceStore.getState().rejectedNames).toEqual([]);
    expect(screen.getByRole("tab", { name: "Chờ duyệt 0" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
  });

  it("omits every AI field from rules-only requests", async () => {
    // Server dùng deny_unknown_fields: field AI lạ làm hỏng cả request, nên
    // payload lọc thường không được chứa chúng.
    const fetchSpy = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      Response.json({
        candidates: [],
        stats: { scannedCharacters: 6, ruleCandidates: 0, aiMergedCandidates: 0 },
      }),
    );
    useWorkspaceStore.getState().setSourceText("张先生走来。");
    const user = userEvent.setup();
    renderWorkspace();

    await user.click(screen.getByRole("button", { name: "Lọc tên" }));

    await waitFor(() => expect(fetchSpy).toHaveBeenCalledTimes(1));
    const body = requestBodyOf(fetchSpy.mock.calls[0][1]);
    expect(body.ai).toBeUndefined();
    expect(body.aiEntities).toBeUndefined();
    expect(body.ner).toBeUndefined();
  });

  it("extracts entities in the browser and sends them as aiEntities", async () => {
    // Lượt 1: trình duyệt gọi thẳng DeepSeek bằng key của user; lượt 2: gửi
    // entities đã trích cho server merge — không còn key trong request server.
    const fetchSpy = vi
      .spyOn(globalThis, "fetch")
      .mockImplementation((input: RequestInfo | URL) => {
        if (requestUrlOf(input).endsWith("/chat/completions")) {
          return Promise.resolve(
            Response.json({
              choices: [
                {
                  message: {
                    content: JSON.stringify({
                      entities: [
                        {
                          text: "萧炎",
                          entityType: "person",
                          suggested: "Tiêu Viêm",
                          confidence: 0.9,
                        },
                      ],
                    }),
                  },
                },
              ],
            }),
          );
        }
        return Promise.resolve(
          Response.json({
            candidates: [],
            stats: { scannedCharacters: 6, ruleCandidates: 0, aiMergedCandidates: 1 },
          }),
        );
      });
    useWorkspaceStore.getState().setSourceText("萧炎走来。");
    const user = userEvent.setup();
    renderWorkspace({
      provider: "deepseek",
      deepseek: { apiKey: "sk-test", model: "deepseek-v4-flash", baseUrl: "" },
      gemini: { apiKey: "", model: "", baseUrl: "" },
    });

    await user.click(screen.getByRole("switch", { name: "Trích AI" }));
    await user.click(screen.getByRole("button", { name: "Lọc tên" }));

    await waitFor(() => expect(fetchSpy).toHaveBeenCalledTimes(2));
    const [aiUrl, aiInit] = fetchSpy.mock.calls[0];
    expect(requestUrlOf(aiUrl)).toBe("https://api.deepseek.com/chat/completions");
    const aiHeaders = aiInit?.headers as Record<string, string>;
    expect(aiHeaders.authorization).toBe("Bearer sk-test");

    const [filterUrl, filterInit] = fetchSpy.mock.calls[1];
    expect(requestUrlOf(filterUrl)).toContain("/names/filter");
    const body = requestBodyOf(filterInit);
    expect(body.ai).toBeUndefined();
    expect(body.aiEntities).toEqual({
      minConfidence: 0.65,
      entities: [
        { text: "萧炎", entityType: "person", suggested: "Tiêu Viêm", confidence: 0.9 },
      ],
    });
  });

  it("reviews ambiguous candidates client-side and applies the decisions", async () => {
    const fetchSpy = vi
      .spyOn(globalThis, "fetch")
      .mockImplementation((input: RequestInfo | URL) => {
        if (requestUrlOf(input).endsWith("/chat/completions")) {
          return Promise.resolve(
            Response.json({
              choices: [
                {
                  message: {
                    content: JSON.stringify({
                      decisions: [
                        { text: "看向", keep: false, confidence: 0.95, entityType: "unknown" },
                      ],
                    }),
                  },
                },
              ],
            }),
          );
        }
        return Promise.resolve(
          Response.json({
            candidates: [
              {
                text: "看向",
                suggested: "khán hướng",
                entityType: "unknown",
                score: 0.5,
                occurrences: 2,
                ranges: [],
                contexts: ["…【看向】…"],
                reasons: [],
                sources: ["ngram"],
                known: false,
              },
            ],
            stats: { scannedCharacters: 6, ruleCandidates: 1, aiMergedCandidates: 0 },
          }),
        );
      });
    useWorkspaceStore.getState().setSourceText("看向远方。看向远方。");
    const user = userEvent.setup();
    renderWorkspace({
      provider: "deepseek",
      deepseek: { apiKey: "sk-test", model: "deepseek-v4-flash", baseUrl: "" },
      gemini: { apiKey: "", model: "", baseUrl: "" },
    });

    await user.click(screen.getByRole("switch", { name: "Duyệt AI" }));
    await user.click(screen.getByRole("button", { name: "Lọc tên" }));

    // AI loại ứng viên mơ hồ → danh sách chờ duyệt trống.
    await waitFor(() => expect(fetchSpy).toHaveBeenCalledTimes(2));
    await waitFor(() => {
      expect(
        useWorkspaceStore.getState().nameFilterResponse?.candidates,
      ).toEqual([]);
    });
  });

  it("warns when enabling an AI toggle without an API key", async () => {
    const warning = vi.spyOn(toast, "warning").mockReturnValue("toast-id");
    const user = userEvent.setup();
    renderWorkspace();

    await user.click(screen.getByRole("switch", { name: "Trích AI" }));

    expect(warning).toHaveBeenCalledTimes(1);
    expect(warning.mock.calls[0]?.[0]).toContain("API key");
  });

  it("keeps a persistent warning on the toggle while AI is on without a key", async () => {
    vi.spyOn(toast, "warning").mockReturnValue("toast-id");
    const user = userEvent.setup();
    renderWorkspace();

    await user.click(screen.getByRole("switch", { name: "Trích AI" }));
    expect(screen.getByLabelText("Trích AI: chưa có API key")).toBeInTheDocument();

    await user.click(screen.getByRole("switch", { name: "Trích AI" }));
    expect(screen.queryByLabelText("Trích AI: chưa có API key")).not.toBeInTheDocument();
  });

  it("does not warn when the active provider already has a key", async () => {
    const warning = vi.spyOn(toast, "warning").mockReturnValue("toast-id");
    const user = userEvent.setup();
    renderWorkspace({
      provider: "deepseek",
      deepseek: { apiKey: "sk-test", model: "deepseek-v4-flash", baseUrl: "" },
      gemini: { apiKey: "", model: "", baseUrl: "" },
    });

    await user.click(screen.getByRole("switch", { name: "Duyệt AI" }));

    expect(warning).not.toHaveBeenCalled();
  });

  it("imports a dropped text file into the source chapter", async () => {
    useWorkspaceStore.getState().setSourceText("");
    renderWorkspace();

    fireEvent.drop(screen.getByPlaceholderText(/Dán một chương tiếng Trung/), {
      dataTransfer: {
        files: [new File(["萧炎来了"], "chuong.txt", { type: "text/plain" })],
        types: ["Files"],
      },
    });

    await waitFor(() => {
      expect(useWorkspaceStore.getState().sourceText).toBe("萧炎来了");
    });
  });

  it("clears the source chapter for a new paste and restores it via undo", async () => {
    const message = vi.spyOn(toast, "message").mockReturnValue("toast-id");
    const user = userEvent.setup();
    useWorkspaceStore.getState().setSourceText("旧章节");
    renderWorkspace();

    await user.click(
      screen.getByRole("button", { name: "Xóa chương nguồn để dán chương mới" }),
    );

    expect(useWorkspaceStore.getState().sourceText).toBe("");
    const options = message.mock.calls[0]?.[1] as
      | { action?: { onClick: () => void } }
      | undefined;
    options?.action?.onClick();
    expect(useWorkspaceStore.getState().sourceText).toBe("旧章节");
  });

  it("undoes approval without rejecting the name", async () => {
    useWorkspaceStore.getState().setNameFilterResponse({
      candidates: [
        {
          text: "萧炎",
          suggested: "Tiêu Viêm",
          entityType: "person",
          score: 0.95,
          occurrences: 3,
          ranges: [],
          contexts: [],
          reasons: [],
          sources: ["qt"],
          known: false,
        },
      ],
      stats: { scannedCharacters: 20, ruleCandidates: 1, aiMergedCandidates: 0 },
    });
    useWorkspaceStore.getState().acceptNameCandidate("萧炎", "Tiêu Viêm");
    const user = userEvent.setup();
    renderWorkspace();

    await user.click(screen.getByRole("button", { name: "Bỏ duyệt 萧炎" }));

    expect(useWorkspaceStore.getState().knownNames).not.toHaveProperty("萧炎");
    expect(useWorkspaceStore.getState().rejectedNames).not.toContain("萧炎");
    expect(screen.getByRole("button", { name: "Duyệt 萧炎" })).toBeInTheDocument();
  });
});
