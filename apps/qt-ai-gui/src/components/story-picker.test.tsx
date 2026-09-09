import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { StoryPicker } from "@/components/story-picker";
import { samePath } from "@/lib/paths";
import { appConfigSet, libraryList, pickFolder, recentSummaries } from "@/lib/api";
import { appConfigSchema } from "@/lib/schema";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({
  ApiError: class ApiError extends Error {
    kind: string;
    constructor(kind: string, message: string) {
      super(message);
      this.kind = kind;
    }
  },
  appConfigSet: vi.fn((config: unknown) => Promise.resolve(config)),
  createStory: vi.fn(),
  initStory: vi.fn(),
  libraryList: vi.fn(() => Promise.resolve([])),
  openStory: vi.fn(),
  pickFolder: vi.fn(),
  recentSummaries: vi.fn(() => Promise.resolve([])),
  sessionStop: vi.fn(() => Promise.resolve({ running: [] })),
  slugifyName: vi.fn((name: string) => Promise.resolve(name.toLowerCase())),
}));

const config = appConfigSchema.parse({
  agyPath: null,
  model: null,
  maxSessions: 50,
  recent: ["D:\\truyen-a", "D:\\truyen-hong"],
});

describe("StoryPicker · Mở gần đây", () => {
  beforeEach(() => {
    vi.mocked(appConfigSet).mockClear();
    vi.mocked(libraryList).mockResolvedValue([]);
    vi.mocked(recentSummaries).mockResolvedValue([
      { root: "D:\\truyen-a", name: "Truyện A", done: 2, total: 10 },
      { root: "D:\\truyen-hong", name: null, done: null, total: null },
    ]);
    useStoryStore.setState({ screen: "picker", config, sessions: {}, roots: {}, progress: {} });
  });

  it("bấm X bỏ truyện khỏi danh sách: chỉ ghi config.recent, không gọi gì xoá file", async () => {
    const user = userEvent.setup();
    render(<StoryPicker />);
    expect(await screen.findByText("Truyện A")).toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: /Bỏ .* khỏi danh sách/ })).toHaveLength(2);

    await user.click(screen.getByRole("button", { name: "Bỏ D:\\truyen-a khỏi danh sách" }));

    await waitFor(() =>
      expect(appConfigSet).toHaveBeenCalledWith(expect.objectContaining({ recent: ["D:\\truyen-hong"] })),
    );
    expect(useStoryStore.getState().config?.recent).toEqual(["D:\\truyen-hong"]);
    expect(screen.queryByText("Truyện A")).not.toBeInTheDocument();
    // Folder hỏng (không đọc được) vẫn bỏ được — đó là lúc cần nút này nhất.
    await user.click(screen.getByRole("button", { name: "Bỏ D:\\truyen-hong khỏi danh sách" }));
    await waitFor(() => expect(useStoryStore.getState().config?.recent).toEqual([]));
    expect(screen.queryByText("Mở gần đây")).not.toBeInTheDocument();
  });
});

describe("StoryPicker · Thư viện", () => {
  beforeEach(() => {
    vi.mocked(appConfigSet).mockClear();
    vi.mocked(pickFolder).mockReset();
    vi.mocked(libraryList).mockReset();
    vi.mocked(recentSummaries).mockResolvedValue([{ root: "D:\\truyen-a", name: "Truyện A", done: 2, total: 10 }]);
  });

  it("chưa chọn thư viện: nút Tạo tắt, ô mời chọn → pickFolder → lưu libraryRoot", async () => {
    useStoryStore.setState({ screen: "picker", config: { ...config, recent: [] } });
    vi.mocked(pickFolder).mockResolvedValue("D:\\lib");
    vi.mocked(libraryList).mockResolvedValue([]);
    const user = userEvent.setup();
    render(<StoryPicker />);
    expect(screen.getByRole("button", { name: "Tạo truyện mới" })).toBeDisabled();
    await user.click(screen.getByRole("button", { name: /Chọn thư viện/ }));
    await waitFor(() => expect(appConfigSet).toHaveBeenCalledWith(expect.objectContaining({ libraryRoot: "D:\\lib" })));
    expect(useStoryStore.getState().config?.libraryRoot).toBe("D:\\lib");
    expect(await screen.findByText("D:\\lib")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Tạo truyện mới" })).toBeEnabled();
    expect(screen.getByText(/Thư viện chưa có truyện nào/)).toBeInTheDocument();
  });

  it("có thư viện: liệt kê truyện, folder chưa init hiện mờ, gần đây ẩn truyện đã nằm trong thư viện", async () => {
    useStoryStore.setState({
      screen: "picker",
      config: { ...config, libraryRoot: "D:\\lib", recent: ["d:\\lib\\truyen-a\\", "D:\\ngoai"] },
    });
    vi.mocked(libraryList).mockResolvedValue([
      { root: "D:\\lib\\truyen-a", name: "Truyện A", done: 2, total: 10 },
      { root: "D:\\lib\\chua-init", name: null, done: null, total: null },
    ]);
    vi.mocked(recentSummaries).mockResolvedValue([
      { root: "d:\\lib\\truyen-a\\", name: "Truyện A", done: 2, total: 10 },
      { root: "D:\\ngoai", name: "Ngoài", done: 0, total: 3 },
    ]);
    render(<StoryPicker />);
    expect(await screen.findByText("Truyện A")).toBeInTheDocument();
    expect(screen.getAllByText("Truyện A")).toHaveLength(1); // không lặp ở Mở gần đây
    expect(screen.getByText("D:\\lib\\chua-init")).toHaveAttribute("title", expect.stringContaining("Chưa khởi tạo"));
    expect(await screen.findByText("Ngoài")).toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: /Bỏ .* khỏi danh sách/ })).toHaveLength(1);
  });

  it("mở thủ công một folder chứa truyện con thì hỏi đặt làm thư viện thay vì khởi tạo nhầm", async () => {
    useStoryStore.setState({ screen: "picker", config: { ...config, recent: [] } });
    vi.mocked(pickFolder).mockResolvedValue("D:\\bookshelf\\fanqie");
    const { openStory } = await import("@/lib/api");
    const { ApiError } = await import("@/lib/api");
    const notFound = new ApiError("story_not_found", "chưa có state.json");
    vi.mocked(openStory).mockRejectedValue(notFound);
    vi.mocked(libraryList).mockImplementation((root?: string) =>
      Promise.resolve(
        // Dò folder cụ thể, hoặc (sau khi đặt) thư viện trong config → cùng danh sách.
        root === undefined || root === "D:\\bookshelf\\fanqie"
          ? [
              { root: "D:\\bookshelf\\fanqie\\a", name: "A", done: 1, total: 2 },
              { root: "D:\\bookshelf\\fanqie\\rac", name: null, done: null, total: null },
            ]
          : [],
      ),
    );
    const user = userEvent.setup();
    render(<StoryPicker />);
    await user.click(screen.getByRole("button", { name: "Mở folder truyện" }));
    expect(await screen.findByText("Đây là thư viện?")).toBeInTheDocument();
    expect(screen.getByText(/chứa 1 truyện đã khởi tạo/)).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Đặt làm thư viện" }));
    await waitFor(() => expect(useStoryStore.getState().config?.libraryRoot).toBe("D:\\bookshelf\\fanqie"));
    expect(screen.queryByText("Khởi tạo folder truyện?")).not.toBeInTheDocument();
    expect(await screen.findByText("A")).toBeInTheDocument();
  });

  it("truyện đang dịch hiện nhãn + tiến độ live + nút Dừng gọi session_stop đúng root", async () => {
    useStoryStore.setState({
      screen: "picker",
      config: { ...config, libraryRoot: "D:\\lib", recent: [], maxParallel: 2 },
      sessions: { "d:\\lib\\a": { status: "running", sessionNo: 1 } },
      roots: { "d:\\lib\\a": "D:\\lib\\a" },
      progress: {
        "d:\\lib\\a": { done: 7, queued: 3, translating: 1, error: 0, skipped: 0, warnings_count: 0, current: "0008" },
      },
    });
    vi.mocked(libraryList).mockResolvedValue([
      { root: "D:\\lib\\a", name: "A", done: 2, total: 10 },
      { root: "D:\\lib\\b", name: "B", done: 0, total: 5 },
    ]);
    const { sessionStop } = await import("@/lib/api");
    const user = userEvent.setup();
    render(<StoryPicker />);
    expect(await screen.findByText("A")).toBeInTheDocument();
    expect(screen.getByText("Đang dịch 1/2 truyện")).toBeInTheDocument();
    expect(screen.getByText("Đang dịch · 0008")).toBeInTheDocument();
    expect(screen.getByText("7/10")).toBeInTheDocument(); // tiến độ live đè số đọc từ đĩa (2/10)
    expect(screen.getAllByRole("button", { name: /Dừng dịch/ })).toHaveLength(1);
    await user.click(screen.getByRole("button", { name: "Dừng dịch D:\\lib\\a" }));
    await waitFor(() => expect(sessionStop).toHaveBeenCalledWith("D:\\lib\\a"));
    await waitFor(() => expect(screen.queryByText(/Đang dịch 1\/2/)).not.toBeInTheDocument());
  });

  it("samePath bỏ qua hoa thường và dấu gạch cuối", () => {
    expect(samePath("D:\\Lib\\a\\", "d:/lib/a")).toBe(true);
    expect(samePath("D:\\lib\\a", "D:\\lib\\ab")).toBe(false);
  });
});
