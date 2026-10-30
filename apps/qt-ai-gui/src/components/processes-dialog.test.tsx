import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { ProcessesDialog } from "@/components/processes-dialog";
import { lockRemove, processOverview, storyKill, type ProcessOverview } from "@/lib/api";
import { pathKey } from "@/lib/paths";
import { sessionOf, useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({
  processOverview: vi.fn(),
  storyKill: vi.fn(),
  agyKill: vi.fn(),
  lockRemove: vi.fn(),
  sessionStop: vi.fn(),
  storySnapshot: vi.fn(),
}));
vi.mock("sonner", () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

const A = "D:\\lib\\dien-phe";
const B = "D:\\lib\\saint-seiya";
const agy = { pid: 4321, startedAt: Date.now() - 5 * 60_000, memory: 300 * 1048576, cpu: 2.5, kind: "translate" as const, owned: true };
const overview: ProcessOverview = {
  appPid: 100,
  stories: [
    { root: A, name: "dien-phe", running: true, agy: [agy], lock: { pid: 100, startedAt: 1, holder: "session", holderName: null } },
    { root: B, name: "saint-seiya", running: false, agy: [], lock: { pid: 19764, startedAt: 1, holder: "other", holderName: "chrome.exe" } },
  ],
  stray: [],
};

describe("ProcessesDialog", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    vi.mocked(processOverview).mockReset().mockResolvedValue(overview);
    vi.mocked(storyKill).mockReset().mockResolvedValue({ ...overview, stories: overview.stories.slice(1) });
    vi.mocked(lockRemove).mockReset().mockResolvedValue({ ...overview, stories: overview.stories.slice(0, 1) });
    useStoryStore.setState({
      processes: { focus: B },
      root: undefined,
      sessions: { [pathKey(A)]: { status: "running", sessionNo: 2 } },
      roots: { [pathKey(A)]: A },
      names: { [pathKey(A)]: "Điên Phê" },
    });
  });

  it("hiện agy và người giữ khoá theo truyện; truyện đang dịch thì Dừng/Kill, không cho gỡ khoá", async () => {
    render(<ProcessesDialog />);
    const running = await screen.findByRole("listitem", { name: "Điên Phê" });
    expect(within(running).getByText(/agy PID 4321/)).toBeInTheDocument();
    expect(within(running).getByText(/300 MB · CPU 2.5% · chạy 5 phút/)).toBeInTheDocument();
    expect(within(running).getByRole("button", { name: /Dừng/ })).toBeInTheDocument();
    expect(within(running).queryByRole("button", { name: /Gỡ khoá/ })).not.toBeInTheDocument();
    const locked = screen.getByRole("listitem", { name: "saint-seiya" });
    expect(within(locked).getByText(/PID 19764: tiến trình khác còn sống giữ \(chrome.exe\)/)).toBeInTheDocument();
  });

  it("Kill hỏi trước, gọi storyKill và đưa phiên trong store về đã dừng ngay", async () => {
    const user = userEvent.setup();
    vi.spyOn(window, "confirm").mockReturnValueOnce(true);
    render(<ProcessesDialog />);
    const running = await screen.findByRole("listitem", { name: "Điên Phê" });
    await user.click(within(running).getByRole("button", { name: /Kill/ }));
    expect(storyKill).toHaveBeenCalledWith(A);
    expect(sessionOf(useStoryStore.getState(), A).status).toBe("stopped");
    expect(screen.queryByRole("listitem", { name: "Điên Phê" })).not.toBeInTheDocument();
  });

  it("Gỡ khoá do tiến trình khác giữ thì cảnh báo trong hộp xác nhận; huỷ thì không gỡ", async () => {
    const user = userEvent.setup();
    const confirm = vi.spyOn(window, "confirm").mockReturnValueOnce(false).mockReturnValueOnce(true);
    render(<ProcessesDialog />);
    const locked = await screen.findByRole("listitem", { name: "saint-seiya" });
    await user.click(within(locked).getByRole("button", { name: /Gỡ khoá/ }));
    expect(confirm.mock.calls[0]?.[0]).toMatch(/vẫn còn sống/);
    expect(lockRemove).not.toHaveBeenCalled();
    await user.click(within(locked).getByRole("button", { name: /Gỡ khoá/ }));
    expect(lockRemove).toHaveBeenCalledWith(B);
  });

  it("Gỡ hết chỉ gỡ khoá không còn ai dùng, chừa khoá tiến trình khác còn sống giữ", async () => {
    const user = userEvent.setup();
    const stale = (root: string) => ({ root, name: root, running: false, agy: [], lock: { pid: 24908, startedAt: 1, holder: "stale" as const, holderName: null } });
    vi.mocked(processOverview).mockResolvedValue({ ...overview, stories: [...overview.stories, stale("D:\\lib\\c"), stale("D:\\lib\\d")] });
    vi.mocked(lockRemove).mockResolvedValue(overview);
    vi.spyOn(window, "confirm").mockReturnValueOnce(true);
    render(<ProcessesDialog />);
    await user.click(await screen.findByRole("button", { name: /Gỡ 2 khoá không dùng/ }));
    expect(vi.mocked(lockRemove).mock.calls).toEqual([["D:\\lib\\c"], ["D:\\lib\\d"]]);
  });

  it("mở từ truyện không kẹt gì thì nói rõ", async () => {
    useStoryStore.setState({ processes: { focus: "D:\\lib\\khac" } });
    render(<ProcessesDialog />);
    expect(await screen.findByText(/Truyện này không có phiên, agy hay khoá phiên nào/)).toBeInTheDocument();
  });
});
