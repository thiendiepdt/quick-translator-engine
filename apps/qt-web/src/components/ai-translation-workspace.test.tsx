import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { AiTranslationWorkspace } from "@/components/ai-translation-workspace";
import { defaultAiSettings } from "@/lib/ai-settings";
import { useWorkspaceStore } from "@/store/workspace";

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

beforeEach(() => {
  useWorkspaceStore.getState().clearAiTranslation();
  useWorkspaceStore.setState({ sourceText: "Nội dung Convert không được dùng" });
});

describe("AI translation workspace", () => {
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
