import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { SettingsPage } from "@/components/pages/settings-page";
import { appConfigSet, baseGet } from "@/lib/api";
import { appConfigSchema, storySnapshotSchema } from "@/lib/schema";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({
  agyStatus: vi.fn(),
  appConfigSet: vi.fn((config: unknown) => Promise.resolve(config)),
  baseGet: vi.fn(),
  baseReset: vi.fn(),
  baseSave: vi.fn(),
  pickAgyFile: vi.fn(),
  pickFolder: vi.fn(),
  saveSettings: vi.fn(),
  storySnapshot: vi.fn(),
}));

const snapshot = storySnapshotSchema.parse({
  root: "D:\\t",
  chapters: [],
  counts: { total: 0, queued: 0, translating: 0, done: 0, error: 0, skipped: 0, withWarnings: 0 },
  settings: { minLengthRatio: 0.75, maxReviewRounds: 3, chaptersPerSession: 10 },
  story: {
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
  },
  sessionRunning: false,
});

const config = appConfigSchema.parse({ agyPath: null, model: null, maxSessions: 50, recent: [] });

describe("SettingsPage · Động cơ dịch", () => {
  beforeEach(() => {
    useStoryStore.setState({ root: snapshot.root, snapshot, config, sessions: {} });
  });

  it("mặc định API key (người mới không cần agy); chọn agy ẩn ô key; đổi provider đổi bộ ô tương ứng", async () => {
    const user = userEvent.setup();
    render(<SettingsPage />);
    const engine = screen.getByRole("radiogroup", { name: "Động cơ dịch" });
    expect(engine.querySelector('[aria-checked="true"]')).toHaveTextContent("API key");
    expect(screen.getByLabelText("API key Google AI")).toHaveAttribute("type", "password");

    await user.click(screen.getByRole("radio", { name: "Antigravity CLI (agy)" }));
    expect(screen.queryByLabelText("API key Google AI")).not.toBeInTheDocument();

    await user.click(screen.getByRole("radio", { name: "API key" }));
    expect(screen.getByLabelText("API key Google AI")).toHaveAttribute("type", "password");
    // Bảng mức nghĩ theo bước: mặc định dịch high, trích glossary low.
    expect(screen.getByLabelText("Mức nghĩ Dịch (kể cả dịch lại, bù đoạn)")).toHaveTextContent("high");
    expect(screen.getByLabelText("Mức nghĩ Trích glossary")).toHaveTextContent("low");

    await user.click(screen.getByRole("radio", { name: "OpenAI-compatible" }));
    expect(screen.queryByLabelText("API key Google AI")).not.toBeInTheDocument();
    expect(screen.getByLabelText("API key")).toBeInTheDocument();
    expect(screen.getByLabelText("Mức nghĩ Soát vi phạm")).toHaveTextContent("high");
    expect(screen.getByLabelText("Mức nghĩ AI điền hồ sơ")).toHaveTextContent("high");
    expect(screen.getByPlaceholderText("https://api.openai.com/v1")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Lưu App + Truyện này" })).toBeEnabled();
  });

  it("chọn chiều ngang văn bản ở Giao diện lưu ngay vào AppConfig", async () => {
    const user = userEvent.setup();
    render(<SettingsPage />);
    const group = screen.getByRole("radiogroup", { name: "Chiều ngang văn bản đọc" });
    expect(group.querySelector('[aria-checked="true"]')).toHaveTextContent("Vừa");
    await user.click(screen.getByRole("radio", { name: "Toàn màn" }));
    await waitFor(() => expect(appConfigSet).toHaveBeenCalledWith(expect.objectContaining({ readingWidth: "full" })));
    expect(useStoryStore.getState().config?.readingWidth).toBe("full");
  });

  it("footer Lưu nằm ngoài khung cuộn, luôn hiện; nút mờ khi chưa sửa, sửa thì sáng, Hoàn tác trả giá trị cũ", async () => {
    const user = userEvent.setup();
    render(<SettingsPage />);
    const bar = screen.getByTestId("save-bar");
    expect(bar.tagName).toBe("FOOTER");
    expect(bar.closest("form")).toBeNull();
    expect(screen.getByRole("button", { name: "Lưu App + Truyện này" })).toHaveAttribute("form", "settings-form");
    expect(screen.getByRole("button", { name: "Lưu App + Truyện này" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Hoàn tác" })).toBeDisabled();

    const parallel = screen.getByLabelText("Số truyện dịch song song");
    await user.clear(parallel);
    await user.type(parallel, "4");
    expect(screen.getByRole("button", { name: "Lưu App + Truyện này" })).toBeEnabled();
    expect(screen.getByText("Có thay đổi chưa lưu.")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Hoàn tác" }));
    expect(parallel).toHaveValue(20);
    expect(screen.getByRole("button", { name: "Lưu App + Truyện này" })).toBeDisabled();
  });

  it("đang dịch: sửa xong nút Lưu vẫn khoá kèm lời nhắc dừng dịch", async () => {
    useStoryStore.getState().openStory({ ...snapshot, sessionRunning: true });
    const user = userEvent.setup();
    render(<SettingsPage />);
    const parallel = screen.getByLabelText("Số truyện dịch song song");
    await user.clear(parallel);
    await user.type(parallel, "3");
    expect(screen.getByRole("button", { name: "Lưu App + Truyện này" })).toBeDisabled();
    expect(screen.getByText(/Dừng dịch rồi mới lưu được/)).toBeInTheDocument();
  });

  it("card Bản mặc định có ba nút; bấm Prompt mặc định mở dialog và nạp base", async () => {
    const user = userEvent.setup();
    vi.mocked(baseGet).mockResolvedValue({ kind: "prompt", setting: "ancient", names: "han", source: "builtin", text: "# x" });
    render(<SettingsPage />);
    expect(screen.getByRole("button", { name: "Rule mặc định" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Glossary chung" })).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Prompt mặc định" }));
    expect(await screen.findByRole("dialog", { name: "Prompt mặc định" })).toBeInTheDocument();
    await waitFor(() => expect(baseGet).toHaveBeenCalledWith("prompt", "ancient", "han"));
  });

  it("config engine api nạp sẵn key/model của provider đang chọn", () => {
    useStoryStore.setState({
      config: {
        ...config,
        engine: "api",
        api: {
          ...config.api,
          provider: "openai",
          openai: { ...config.api.openai, apiKey: "sk-hub", model: "gemini-3.7-flash", baseUrl: "http://192.0.2.10/v1" },
        },
      },
    });
    render(<SettingsPage />);
    expect(screen.getByLabelText("API key")).toHaveValue("sk-hub");
    expect(screen.getByLabelText("Model")).toHaveValue("gemini-3.7-flash");
    expect(screen.getByLabelText("Base URL")).toHaveValue("http://192.0.2.10/v1");
  });
});
