import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { toast } from "sonner";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { StoryPicker } from "@/components/story-picker";
import { samePath } from "@/lib/paths";
import {
  ApiError,
  appConfigSet,
  libraryList,
  libraryRescan,
  openStory,
  pickFolder,
  recentSummaries,
  sessionStart,
} from "@/lib/api";
import { appConfigSchema } from "@/lib/schema";
import { useStoryStore } from "@/store/story";

vi.mock("sonner", () => ({ toast: { message: vi.fn(), success: vi.fn(), error: vi.fn() } }));
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
  libraryRescan: vi.fn(),
  openStory: vi.fn(),
  pickFolder: vi.fn(),
  recentSummaries: vi.fn(() => Promise.resolve([])),
  sessionStart: vi.fn(() => Promise.resolve({ running: [] })),
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

  it("huy hiệu ⚠ VN chỉ hiện khi vnFlags > 0", async () => {
    vi.mocked(recentSummaries).mockResolvedValue([
      { root: "D:\\truyen-a", name: "Truyện A", done: 2, total: 10, vnFlags: 3 },
      { root: "D:\\truyen-hong", name: "Truyện Hồng", done: 1, total: 5, vnFlags: 0 },
      { root: "D:\\truyen-c", name: "Truyện C", done: 1, total: 5, vnFlags: null },
    ]);
    useStoryStore.setState({ config: appConfigSchema.parse({ ...config, recent: ["D:\\truyen-a", "D:\\truyen-hong", "D:\\truyen-c"] }) });
    render(<StoryPicker />);
    expect(await screen.findByText("Truyện A")).toBeInTheDocument();
    expect(screen.getByTitle("3 chỗ nhạy cảm về Việt Nam chưa xác minh")).toBeInTheDocument();
    expect(screen.getAllByTitle(/chỗ nhạy cảm về Việt Nam/)).toHaveLength(1);
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

  it("nút Làm mới quét lại thư viện, hiện truyện mới và toast tên truyện; không có gì mới thì nói rõ", async () => {
    useStoryStore.setState({ screen: "picker", config: { ...config, libraryRoot: "D:\\lib", recent: [] } });
    const a = { root: "D:\\lib\\truyen-a", name: "Truyện A", done: 2, total: 10 };
    const b = { root: "D:\\lib\\truyen-b", name: "Truyện B", done: 0, total: 5 };
    vi.mocked(libraryList).mockResolvedValue([a]);
    const user = userEvent.setup();
    render(<StoryPicker />);
    await screen.findByText("Truyện A");
    expect(screen.queryByText("Truyện B")).not.toBeInTheDocument();

    vi.mocked(libraryList).mockResolvedValue([a, b]);
    await user.click(screen.getByRole("button", { name: "Làm mới thư viện" }));
    expect(await screen.findByText("Truyện B")).toBeInTheDocument();
    expect(toast.message).toHaveBeenLastCalledWith("Thư viện: 1 truyện mới: Truyện B");

    await user.click(screen.getByRole("button", { name: "Làm mới thư viện" }));
    await waitFor(() => expect(toast.message).toHaveBeenLastCalledWith("Thư viện không có truyện mới"));
    expect(libraryList).toHaveBeenCalledTimes(3);
  });

  it("thư viện dài: vẽ 12 dòng + Xem thêm, ô tìm lọc theo tên/folder và về trang đầu", async () => {
    const user = userEvent.setup();
    useStoryStore.setState({ screen: "picker", config: { ...config, libraryRoot: "D:\\lib", recent: [] } });
    vi.mocked(libraryList).mockResolvedValue(
      Array.from({ length: 30 }, (_, i) => ({ root: `D:\\lib\\t${i + 1}`, name: `Truyện ${i + 1}`, done: i, total: 30 })),
    );
    render(<StoryPicker />);
    expect(await screen.findByText("Truyện 1")).toBeInTheDocument();
    expect(screen.getAllByText(/^Truyện \d+$/)).toHaveLength(12);
    expect(screen.queryByText("Truyện 13")).not.toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Xem thêm (18)" }));
    expect(screen.getAllByText(/^Truyện \d+$/)).toHaveLength(24);
    await user.click(screen.getByRole("button", { name: "Xem thêm (6)" }));
    expect(screen.getAllByText(/^Truyện \d+$/)).toHaveLength(30);
    expect(screen.queryByRole("button", { name: /Xem thêm/ })).not.toBeInTheDocument();

    await user.type(screen.getByLabelText("Tìm truyện trong thư viện"), "t27");
    expect(screen.getAllByText(/^Truyện \d+$/)).toHaveLength(1);
    expect(screen.getByText("Truyện 27")).toBeInTheDocument();
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

  it("nút Quét chương mới quét raw/ cả thư viện, cập nhật số chương và toast truyện có chương mới", async () => {
    useStoryStore.setState({ screen: "picker", config: { ...config, libraryRoot: "D:\\lib", recent: [] } });
    const a = { root: "D:\\lib\\truyen-a", name: "Truyện A", done: 2, total: 10, pending: 8 };
    vi.mocked(libraryList).mockResolvedValue([a]);
    vi.mocked(libraryRescan).mockResolvedValue({
      changed: [
        { root: a.root, name: "Truyện A", added: 5, removed: 0 },
        { root: "D:\\lib\\c", name: null, added: 0, removed: 2 },
      ],
      skippedRunning: ["D:\\lib\\b"],
      failed: ["e-hong: state.json sai schema"],
      stories: [{ ...a, total: 15, pending: 13 }],
    });
    const user = userEvent.setup();
    render(<StoryPicker />);
    expect(await screen.findByText("2/10")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Quét chương mới cho cả thư viện" }));
    expect(await screen.findByText("2/15")).toBeInTheDocument();
    expect(toast.message).toHaveBeenLastCalledWith(
      "Quét thư viện: 1 truyện có chương mới (+5 chương): Truyện A; gỡ 2 chương raw đã mất; bỏ qua 1 truyện đang dịch",
    );
    expect(toast.error).toHaveBeenLastCalledWith("Không quét được 1 truyện: e-hong: state.json sai schema");

    vi.mocked(libraryRescan).mockResolvedValue({ changed: [], skippedRunning: [], failed: [], stories: [a] });
    await user.click(screen.getByRole("button", { name: "Quét chương mới cho cả thư viện" }));
    await waitFor(() =>
      expect(toast.message).toHaveBeenLastCalledWith("Quét thư viện: không truyện nào có chương mới"),
    );
  });

  describe("Dịch tất cả truyện chưa xong", () => {
    const rows = [
      { root: "D:\\lib\\a", name: "A", done: 2, total: 10, pending: 8 },
      { root: "D:\\lib\\xong", name: "Xong", done: 5, total: 5, pending: 0 },
      // Chỉ còn chương lỗi/bỏ qua: done < total nhưng phiên không có gì để dịch.
      { root: "D:\\lib\\loi", name: "Lỗi", done: 3, total: 4, pending: 0 },
      { root: "D:\\lib\\b", name: "B", done: 0, total: 5, pending: 5 },
      { root: "D:\\lib\\c", name: "C", done: 1, total: 4, pending: 3 },
      { root: "D:\\lib\\d", name: "D", done: 0, total: 1, pending: 1 },
      { root: "D:\\lib\\chua-init", name: null, done: null, total: null, pending: null },
    ];

    beforeEach(() => {
      vi.mocked(toast.error).mockClear();
      vi.mocked(sessionStart).mockReset();
      vi.mocked(sessionStart).mockResolvedValue({ running: [] });
      vi.mocked(openStory).mockReset();
      vi.mocked(openStory).mockImplementation((root: string) =>
        Promise.resolve({ root, story: { name: "A" }, counts: { skipped: 0, error: 0 }, sessionRunning: false } as never),
      );
      vi.mocked(libraryList).mockResolvedValue(rows);
      // C đang dịch, giới hạn 3 → còn 2 chỗ cho 3 truyện chưa xong (A, B, D).
      useStoryStore.setState({
        screen: "picker",
        root: undefined,
        config: { ...config, libraryRoot: "D:\\lib", recent: [], maxParallel: 3 },
        sessions: { "d:\\lib\\c": { status: "running", sessionNo: 1 } },
        roots: { "d:\\lib\\c": "D:\\lib\\c" },
        progress: {},
      });
    });

    it("xác nhận rồi bắt đầu phiên cho các truyện còn chương chờ trong giới hạn song song, vào truyện đầu tiên", async () => {
      const user = userEvent.setup();
      render(<StoryPicker />);
      await screen.findByText("A");
      await user.click(screen.getByRole("button", { name: "Dịch tất cả truyện chưa xong (3)" }));

      expect(await screen.findByText(/Bắt đầu phiên dịch cho 2 truyện \(13 chương còn chờ\) bằng API · Gemini/)).toBeInTheDocument();
      expect(screen.getByRole("status")).toHaveTextContent(
        "Còn 1 truyện chưa xong không được bắt đầu lần này: giới hạn 3 truyện dịch song song, đang chạy 1.",
      );
      expect(sessionStart).not.toHaveBeenCalled();

      await user.click(screen.getByRole("button", { name: "Bắt đầu dịch 2 truyện" }));
      await waitFor(() => expect(useStoryStore.getState().screen).toBe("workbench"));
      expect(vi.mocked(sessionStart).mock.calls).toEqual([
        ["D:\\lib\\a", undefined],
        ["D:\\lib\\b", undefined],
      ]);
      // Truyện đầu được mở (open_story quét raw/, ghi state.json) trước khi phiên của nó chạy.
      expect(openStory).toHaveBeenCalledExactlyOnceWith("D:\\lib\\a");
      expect(vi.mocked(openStory).mock.invocationCallOrder[0]).toBeLessThan(
        vi.mocked(sessionStart).mock.invocationCallOrder[0],
      );
      const state = useStoryStore.getState();
      expect(state.root).toBe("D:\\lib\\a");
      expect(state.page).toBe("translate");
      expect(state.sessions["d:\\lib\\a"]).toEqual({ status: "running", sessionNo: 0 });
      expect(toast.success).toHaveBeenLastCalledWith("Đã bắt đầu dịch 2 truyện");
      expect(toast.error).not.toHaveBeenCalled();
    });

    it("thiếu API key thì dừng cả lượt ngay truyện đầu, không vào truyện nào", async () => {
      vi.mocked(sessionStart).mockRejectedValue(new ApiError("api_key_missing", "Chưa nhập API key Gemini"));
      const user = userEvent.setup();
      render(<StoryPicker />);
      await screen.findByText("A");
      await user.click(screen.getByRole("button", { name: "Dịch tất cả truyện chưa xong (3)" }));
      await user.click(await screen.findByRole("button", { name: "Bắt đầu dịch 2 truyện" }));
      await waitFor(() => expect(toast.error).toHaveBeenLastCalledWith("A: Chưa nhập API key Gemini"));
      expect(sessionStart).toHaveBeenCalledTimes(1);
      expect(useStoryStore.getState().screen).toBe("picker");
    });

    it("một truyện không bắt đầu được (lỗi riêng) thì các truyện sau vẫn chạy", async () => {
      vi.mocked(sessionStart).mockImplementation((root: string) =>
        root.endsWith("a")
          ? Promise.reject(new ApiError("session_locked", "Truyện đang có phiên khác chạy (PID 1)."))
          : Promise.resolve({ running: [] }),
      );
      const user = userEvent.setup();
      render(<StoryPicker />);
      await screen.findByText("A");
      await user.click(screen.getByRole("button", { name: "Dịch tất cả truyện chưa xong (3)" }));
      await user.click(await screen.findByRole("button", { name: "Bắt đầu dịch 2 truyện" }));
      await waitFor(() => expect(toast.success).toHaveBeenLastCalledWith("Đã bắt đầu dịch 1 truyện"));
      expect(toast.error).toHaveBeenLastCalledWith("A: Truyện đang có phiên khác chạy (PID 1).");
      expect(sessionStart).toHaveBeenCalledTimes(2);
      // Vẫn vào truyện đầu danh sách, nhưng nó không được đánh dấu đang dịch.
      expect(useStoryStore.getState().root).toBe("D:\\lib\\a");
      expect(useStoryStore.getState().sessions["d:\\lib\\a"]).toBeUndefined();
    });

    it("đã chạy đủ giới hạn hoặc không còn truyện nào chờ thì nút tắt", async () => {
      useStoryStore.setState({ config: { ...config, libraryRoot: "D:\\lib", recent: [], maxParallel: 1 } });
      const { unmount } = render(<StoryPicker />);
      await screen.findByText("A");
      expect(screen.getByRole("button", { name: "Dịch tất cả truyện chưa xong (3)" })).toBeDisabled();
      unmount();

      vi.mocked(libraryList).mockResolvedValue(rows.filter((row) => !row.pending));
      render(<StoryPicker />);
      await screen.findByText("Xong");
      expect(screen.getByRole("button", { name: "Dịch tất cả truyện chưa xong" })).toBeDisabled();
    });
  });

  it("samePath bỏ qua hoa thường và dấu gạch cuối", () => {
    expect(samePath("D:\\Lib\\a\\", "d:/lib/a")).toBe(true);
    expect(samePath("D:\\lib\\a", "D:\\lib\\ab")).toBe(false);
  });
});
