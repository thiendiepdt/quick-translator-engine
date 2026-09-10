import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { SettingsPage } from "@/components/pages/settings-page";
import { appConfigSet } from "@/lib/api";
import { appConfigSchema, storySnapshotSchema } from "@/lib/schema";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({
  agyStatus: vi.fn(),
  appConfigSet: vi.fn((config: unknown) => Promise.resolve(config)),
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
    expect(screen.getByLabelText("Thinking (Gemini 3.x: high ↔ minimal)")).toBeInTheDocument();

    await user.click(screen.getByRole("radio", { name: "OpenAI-compatible" }));
    expect(screen.queryByLabelText("API key Google AI")).not.toBeInTheDocument();
    expect(screen.getByLabelText("API key")).toBeInTheDocument();
    expect(screen.getByLabelText("Mức reasoning OpenAI")).toHaveTextContent("high");
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

  it("thanh Lưu dính đáy chỉ hiện khi có thay đổi; Hoàn tác trả giá trị cũ và ẩn thanh", async () => {
    const user = userEvent.setup();
    render(<SettingsPage />);
    expect(screen.queryByRole("button", { name: "Lưu App + Truyện này" })).not.toBeInTheDocument();

    const parallel = screen.getByLabelText("Số truyện dịch song song");
    await user.clear(parallel);
    await user.type(parallel, "4");
    expect(screen.getByRole("button", { name: "Lưu App + Truyện này" })).toBeEnabled();
    expect(screen.getByTestId("save-bar")).toHaveClass("sticky");
    expect(screen.getByText("Có thay đổi chưa lưu.")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Hoàn tác" }));
    expect(parallel).toHaveValue(2);
    expect(screen.queryByRole("button", { name: "Lưu App + Truyện này" })).not.toBeInTheDocument();
  });

  it("đang dịch: thanh Lưu vẫn hiện nhưng nút Lưu khoá kèm lời nhắc dừng dịch", async () => {
    useStoryStore.getState().openStory({ ...snapshot, sessionRunning: true });
    const user = userEvent.setup();
    render(<SettingsPage />);
    const parallel = screen.getByLabelText("Số truyện dịch song song");
    await user.clear(parallel);
    await user.type(parallel, "3");
    expect(screen.getByRole("button", { name: "Lưu App + Truyện này" })).toBeDisabled();
    expect(screen.getByText(/Dừng dịch rồi mới lưu được/)).toBeInTheDocument();
  });

  it("config engine api nạp sẵn key/model của provider đang chọn", () => {
    useStoryStore.setState({
      config: {
        ...config,
        engine: "api",
        api: { ...config.api, provider: "openai", openai: { apiKey: "sk-hub", model: "gemini-3.7-flash", baseUrl: "http://192.0.2.10/v1" } },
      },
    });
    render(<SettingsPage />);
    expect(screen.getByLabelText("API key")).toHaveValue("sk-hub");
    expect(screen.getByLabelText("Model")).toHaveValue("gemini-3.7-flash");
    expect(screen.getByLabelText("Base URL")).toHaveValue("http://192.0.2.10/v1");
  });
});
