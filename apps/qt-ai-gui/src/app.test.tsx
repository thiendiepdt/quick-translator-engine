import { act, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import App from "@/app";
import { agyStatus, appConfigGet } from "@/lib/api";
import { appConfigSchema } from "@/lib/schema";
import { LAST_SEEN_KEY } from "@/hooks/use-whats-new";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({
  zhStoryStatus: vi.fn(() => Promise.resolve({ scanned: 0, traditional: 0, notUtf8: 0, undoChapters: null })),
  zhStoryConvert: vi.fn(),
  zhStoryUndo: vi.fn(),
  agyStatus: vi.fn(),
  appConfigGet: vi.fn(),
  appConfigSet: vi.fn(),
  pickAgyFile: vi.fn(),
}));
vi.mock("@tauri-apps/api/app", () => ({ getVersion: vi.fn(() => Promise.resolve("1.0.21")) }));
vi.mock("@/hooks/use-session-events", () => ({ useSessionEvents: () => undefined }));
vi.mock("@/hooks/use-theme", () => ({ useThemeSync: () => undefined }));
vi.mock("@/components/story-picker", () => ({ StoryPicker: () => <div>PICKER</div> }));
vi.mock("@/components/app-rail", () => ({ AppRail: () => null }));
vi.mock("@/components/story-sidebar", () => ({ StorySidebar: () => null }));
vi.mock("@/components/pages/translate-page", () => ({ TranslatePage: () => null }));
vi.mock("@/components/pages/story-page", () => ({ StoryPage: () => null }));
vi.mock("@/components/pages/export-page", () => ({ ExportPage: () => null }));
vi.mock("@/components/pages/settings-page", () => ({ SettingsPage: () => null }));

const agyConfig = appConfigSchema.parse({ agyPath: null, model: null, maxSessions: 50, recent: [], engine: "agy" });
const found = { found: true, path: "/usr/bin/agy", version: "1.2.0", models: [{ id: "m", label: "m" }], message: null };

describe("App · dò agy lúc khởi động", () => {
  beforeEach(() => {
    useStoryStore.setState({ config: undefined, agy: undefined, screen: "picker" });
    vi.mocked(appConfigGet).mockReset().mockResolvedValue(agyConfig);
    vi.mocked(agyStatus).mockReset();
  });

  it("agy_status chậm thì chỉ dò MỘT lần, không dò chồng dù setConfig đổi object", async () => {
    let finish: (value: typeof found) => void = () => undefined;
    vi.mocked(agyStatus).mockReturnValue(new Promise((resolve) => (finish = resolve)));
    render(<App />);
    await screen.findByText(/Đang kiểm tra agy/);
    await act(() => new Promise((r) => setTimeout(r, 150)));
    expect(agyStatus).toHaveBeenCalledTimes(1);
    expect(appConfigGet).toHaveBeenCalledTimes(1);
    finish(found);
    await screen.findByText("PICKER");
    expect(agyStatus).toHaveBeenCalledTimes(1);
  });

  it("dò agy lỗi thì hiện màn Chưa thấy agy kèm lý do, không xoay mãi", async () => {
    vi.mocked(agyStatus).mockRejectedValue(new Error("agy models thoát mã 1"));
    render(<App />);
    await waitFor(() => expect(useStoryStore.getState().agy?.found).toBe(false));
    expect(useStoryStore.getState().agy?.message).toBe("agy models thoát mã 1");
    expect(agyStatus).toHaveBeenCalledTimes(1);
  });
});

describe("App · Có gì mới sau khi cập nhật", () => {
  const apiConfig = appConfigSchema.parse({ agyPath: null, model: null, maxSessions: 50, recent: [], engine: "api" });

  beforeEach(() => {
    localStorage.clear();
    useStoryStore.setState({ config: undefined, agy: undefined, screen: "picker", page: "translate", root: undefined });
    vi.mocked(appConfigGet).mockReset().mockResolvedValue(apiConfig);
  });

  it("cài mới (chưa có truyện gần đây, chưa có khoá): không hiện hộp thoại, chỉ ghi nhớ version", async () => {
    render(<App />);
    await screen.findByText("PICKER");
    await waitFor(() => expect(localStorage.getItem(LAST_SEEN_KEY)).toBe("1.0.21"));
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("bản đầu có tính năng này, người dùng cũ (đã có truyện gần đây) chưa có khoá: hiện Có gì mới của bản đang dùng", async () => {
    vi.mocked(appConfigGet).mockResolvedValue({ ...apiConfig, recent: ["D:\\truyen"] });
    render(<App />);
    const dialog = await screen.findByRole("dialog", { name: "Có gì mới ở bản 1.0.21" });
    expect(within(dialog).queryAllByRole("heading", { level: 3 })).toHaveLength(0);
    expect(localStorage.getItem(LAST_SEEN_KEY)).toBe("1.0.21");
  });

  it("bỏ qua bản 1.0.20 (1.0.19 → 1.0.21): hộp thoại có cả 1.0.21 và 1.0.20", async () => {
    localStorage.setItem(LAST_SEEN_KEY, "1.0.19");
    render(<App />);
    const dialog = await screen.findByRole("dialog", { name: "Có gì mới ở bản 1.0.21" });
    const headings = within(dialog).getAllByRole("heading", { level: 3 });
    expect(headings.map((h) => h.textContent)).toEqual([expect.stringContaining("1.0.21"), expect.stringContaining("1.0.20")]);
  });

  it("hạ bản (đã thấy 1.0.22): không hiện, giữ 1.0.22", async () => {
    localStorage.setItem(LAST_SEEN_KEY, "1.0.22");
    render(<App />);
    await screen.findByText("PICKER");
    await act(() => new Promise((r) => setTimeout(r, 30)));
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    expect(localStorage.getItem(LAST_SEEN_KEY)).toBe("1.0.22");
  });

  it("ở màn chọn truyện: hiện Có gì mới của bản vừa lên; Xem tất cả mở danh sách ngay trong hộp thoại; Đã hiểu đóng", async () => {
    localStorage.setItem(LAST_SEEN_KEY, "1.0.20");
    const user = userEvent.setup();
    render(<App />);
    const dialog = await screen.findByRole("dialog", { name: "Có gì mới ở bản 1.0.21" });
    expect(screen.getByText("PICKER")).toBeInTheDocument();
    await user.click(within(dialog).getByRole("button", { name: "Xem tất cả" }));
    expect(within(dialog).getByRole("button", { name: /Bản 1\.0\.20/ })).toBeInTheDocument();
    await user.click(within(dialog).getByRole("button", { name: "Đã hiểu" }));
    await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
    expect(localStorage.getItem(LAST_SEEN_KEY)).toBe("1.0.21");
  });

  it("đang trong truyện: Xem tất cả chuyển sang trang Cài đặt và đóng hộp thoại", async () => {
    localStorage.setItem(LAST_SEEN_KEY, "1.0.20");
    useStoryStore.setState({ screen: "workbench", page: "translate", root: "D:\\t" });
    const user = userEvent.setup();
    render(<App />);
    const dialog = await screen.findByRole("dialog", { name: "Có gì mới ở bản 1.0.21" });
    await user.click(within(dialog).getByRole("button", { name: "Xem tất cả" }));
    expect(useStoryStore.getState().page).toBe("settings");
    await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
  });
});
