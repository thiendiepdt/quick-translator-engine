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
    glossary: { names: {}, places: {}, items: {}, creatures: {}, skills: {}, common: {}, signature_phrases: {} },
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
    useStoryStore.setState({ root: snapshot.root, snapshot, config, session: { status: "idle" } });
  });

  it("mặc định agy, chọn API key mới hiện ô key; đổi provider đổi bộ ô tương ứng", async () => {
    const user = userEvent.setup();
    render(<SettingsPage />);
    const engine = screen.getByRole("radiogroup", { name: "Động cơ dịch" });
    expect(engine.querySelector('[aria-checked="true"]')).toHaveTextContent("Antigravity CLI (agy)");
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
