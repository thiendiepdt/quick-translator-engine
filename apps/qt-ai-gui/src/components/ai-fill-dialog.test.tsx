import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { AiFillDialog } from "@/components/ai-fill-dialog";
import { aiFillStory } from "@/lib/api";
import { appConfigSchema, storyConfigSchema } from "@/lib/schema";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({
  aiFillStory: vi.fn(),
}));

const before = storyConfigSchema.parse({
  name: "",
  sourceUrl: "",
  protagonist: "",
  summary: "",
  genre: { setting: "ancient", names: "han" },
  glossary: { names: {}, places: {}, items: {}, creatures: {}, skills: {}, common: {}, signature_phrases: {}, addressing: {} },
  style: { voice: "", toneRules: [], signaturePhrases: {}, avoid: [] },
  customPrompt: "",
  checkRules: [],
  autoGlossaryLog: [],
  autoGlossary: "inherit",
});
const after = {
  ...before,
  name: "Kỳ Chiêu Nguyệt",
  protagonist: "Kỳ Chiêu Nguyệt",
  genre: { setting: "modern" as const, names: "han" as const },
};

const base = appConfigSchema.parse({ agyPath: null, model: null, maxSessions: 50, recent: [] });
const apiConfig = {
  ...base,
  engine: "api" as const,
  api: { ...base.api, provider: "openai" as const, openai: { apiKey: "sk", model: "gemini-3.8-flash", baseUrl: "" } },
};

function renderDialog() {
  const onApply = vi.fn();
  render(
    <AiFillDialog
      root={"D:\\t"}
      initialName="Kỳ Chiêu Nguyệt"
      initialUrl="https://x/y"
      open
      onOpenChange={() => undefined}
      onApply={onApply}
    />,
  );
  return onApply;
}

describe("AiFillDialog · động cơ", () => {
  beforeEach(() => {
    vi.mocked(aiFillStory).mockReset();
    vi.mocked(aiFillStory).mockResolvedValue({ before, after, exitCode: 0, log: ["Đọc 1 chương đầu"] });
  });

  it("động cơ API: mô tả nêu model + không tra web, chạy xong hiện diff và Áp dụng trả after", async () => {
    useStoryStore.setState({ config: apiConfig });
    const user = userEvent.setup();
    const onApply = renderDialog();
    expect(screen.getByText(/API · OpenAI-compatible · gemini-3\.8-flash sẽ đọc 3 chương đầu/)).toBeInTheDocument();
    expect(screen.queryByText(/agy sẽ tra web/)).not.toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Chạy AI điền" }));
    expect(aiFillStory).toHaveBeenCalledWith("D:\\t", "Kỳ Chiêu Nguyệt", "https://x/y");
    expect(await screen.findByText("protagonist")).toBeInTheDocument();
    expect(screen.getByText("Đọc 1 chương đầu")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Áp dụng" }));
    expect(onApply).toHaveBeenCalledWith(after);
  });

  it("động cơ agy: mô tả giữ lời tra web", () => {
    useStoryStore.setState({ config: { ...base, engine: "agy" } });
    renderDialog();
    expect(screen.getByText(/agy sẽ tra web theo tên \+ link/)).toBeInTheDocument();
  });
});
