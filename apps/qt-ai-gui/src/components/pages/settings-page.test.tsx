import { getVersion } from "@tauri-apps/api/app";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { SettingsPage } from "@/components/pages/settings-page";
import { agyDataClean, agyDataStats, appConfigSet, baseGet } from "@/lib/api";
import { appConfigSchema, storySnapshotSchema } from "@/lib/schema";
import { useStoryStore } from "@/store/story";

vi.mock("@tauri-apps/api/app", () => ({ getVersion: vi.fn(() => Promise.resolve("1.0.21")) }));
vi.mock("sonner", () => ({ toast: { message: vi.fn(), success: vi.fn(), error: vi.fn() } }));
vi.mock("@/lib/api", () => ({
  agyDataClean: vi.fn(),
  agyDataStats: vi.fn(() =>
    Promise.resolve({ dataDir: null, appCount: 0, appBytes: 0, otherCount: 0, otherBytes: 0 }),
  ),
  agyStatus: vi.fn(),
  appConfigSet: vi.fn((config: unknown) => Promise.resolve(config)),
  baseGet: vi.fn(),
  baseReset: vi.fn(),
  baseSave: vi.fn(),
  pickAgyFile: vi.fn(),
  pickFolder: vi.fn(),
  saveSettings: vi.fn(),
  storySnapshot: vi.fn(),
  vnBuiltinEntries: vi.fn(() => Promise.resolve([])),
  vnSettingsGet: vi.fn(() => Promise.resolve({ extra: [], disabled: [], replacements: [] })),
  vnSettingsSave: vi.fn(),
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

  it("ô 'Tự dịch tiếp khi quota agy reset' mặc định tắt; tick rồi Lưu thì ghi resumeOnQuota vào AppConfig", async () => {
    const user = userEvent.setup();
    vi.mocked(appConfigSet).mockClear();
    render(<SettingsPage />);
    const box = screen.getByLabelText("Tự dịch tiếp khi quota agy reset");
    expect(box).not.toBeChecked();
    await user.click(box);
    await user.click(screen.getByRole("button", { name: "Lưu App + Truyện này" }));
    await waitFor(() => expect(appConfigSet).toHaveBeenCalledWith(expect.objectContaining({ resumeOnQuota: true })));
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

describe("SettingsPage · Hội thoại agy", () => {
  const GB = 1024 * 1024 * 1024;
  const before = {
    dataDir: "C:\\Users\\x\\.gemini\\antigravity-cli",
    appCount: 518,
    appBytes: 3.5 * GB,
    otherCount: 2,
    otherBytes: 12 * 1024 * 1024,
  };

  beforeEach(() => {
    useStoryStore.setState({ root: snapshot.root, snapshot, config, sessions: {} });
    vi.mocked(agyDataClean).mockReset();
  });

  it("hiện số hội thoại app sinh ra + dung lượng; bấm dọn thì gọi lệnh, báo kết quả và cập nhật số", async () => {
    vi.mocked(agyDataStats).mockResolvedValue(before);
    vi.mocked(agyDataClean).mockResolvedValue({
      removed: 515,
      freedBytes: 3.4 * GB,
      kept: 3,
      stats: { ...before, appCount: 3, appBytes: 0.1 * GB },
    });
    const { toast } = await import("sonner");
    const user = userEvent.setup();
    render(<SettingsPage />);
    expect(screen.getByText(/Đang đo hội thoại agy/)).toBeInTheDocument();
    expect(await screen.findByText("518")).toBeInTheDocument();
    expect(screen.getByText("3,5 GB")).toBeInTheDocument();
    expect(screen.getByText(/Hội thoại bạn tự chat bằng agy \(2 · 12 MB\) không bao giờ bị xoá/)).toBeInTheDocument();
    expect(agyDataClean).not.toHaveBeenCalled();

    await user.click(screen.getByRole("button", { name: "Dọn hội thoại của app" }));
    await waitFor(() =>
      expect(toast.success).toHaveBeenLastCalledWith(
        "Đã xoá 515 hội thoại agy, lấy lại 3,4 GB; giữ lại 3 hội thoại của truyện đang dịch hoặc vừa ghi xong",
      ),
    );
    expect(agyDataClean).toHaveBeenCalledTimes(1);
    expect(screen.getByText("102,4 MB")).toBeInTheDocument();
    expect(screen.queryByText("518")).not.toBeInTheDocument();
  });

  it("không còn hội thoại nào của app thì nút dọn tắt; máy chưa có agy thì nói rõ", async () => {
    vi.mocked(agyDataStats).mockResolvedValue({ ...before, appCount: 0, appBytes: 0 });
    const { unmount } = render(<SettingsPage />);
    expect(await screen.findByRole("button", { name: "Dọn hội thoại của app" })).toBeDisabled();
    unmount();

    vi.mocked(agyDataStats).mockResolvedValue({ dataDir: null, appCount: 0, appBytes: 0, otherCount: 0, otherBytes: 0 });
    render(<SettingsPage />);
    expect(await screen.findByText("Máy này chưa có dữ liệu agy CLI.")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Dọn hội thoại của app" })).not.toBeInTheDocument();
  });
});

describe("SettingsPage · Có gì mới", () => {
  beforeEach(() => {
    useStoryStore.setState({ root: snapshot.root, snapshot, config, sessions: {} });
  });

  it("liệt kê các bản tới bản đang dùng, mới nhất trước: bản đang dùng mở sẵn, 5 bản đầu + Xem tất cả", async () => {
    vi.mocked(getVersion).mockResolvedValueOnce("1.0.20");
    const user = userEvent.setup();
    render(<SettingsPage />);
    const card = (await screen.findByRole("heading", { name: "Có gì mới" })).closest("section");
    if (!card) throw new Error("thiếu card Có gì mới");
    const current = await within(card).findByRole("button", { name: /Bản 1\.0\.20/ });
    expect(current).toHaveAttribute("aria-expanded", "true");
    expect(within(card).queryByRole("button", { name: /Bản 1\.0\.21/ })).not.toBeInTheDocument();
    const versions = within(card).getAllByRole("button", { name: /^Bản / });
    expect(versions).toHaveLength(5);
    expect(versions[1]).toHaveTextContent("1.0.19");
    expect(versions[1]).toHaveAttribute("aria-expanded", "false");
    expect(within(card).getByText(/Đang dùng bản 1\.0\.20/)).toBeInTheDocument();

    await user.click(within(card).getByRole("button", { name: /^Xem tất cả/ }));
    expect(within(card).getAllByRole("button", { name: /^Bản / })).toHaveLength(21);
  });
});
