import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { toast } from "sonner";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { SyncProgressDialog } from "@/components/sync-progress-dialog";
import { progressDrift, requeueUntracked, syncProgress } from "@/lib/api";
import { pathKey } from "@/lib/paths";
import { storySnapshotSchema } from "@/lib/schema";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({ progressDrift: vi.fn(), syncProgress: vi.fn(), requeueUntracked: vi.fn() }));
vi.mock("sonner", () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

const ROOT = "D:\\lib\\t";
const synced = storySnapshotSchema.parse({
  root: ROOT,
  chapters: [{ id: "0001", status: "done", reviewRound: 0, reason: null, warnings: [] }],
  counts: { total: 1, queued: 0, translating: 0, done: 1, error: 0, skipped: 0, withWarnings: 0 },
  settings: { minLengthRatio: 0.75, maxReviewRounds: 3, chaptersPerSession: 10 },
  story: {
    name: "Truyện T",
    sourceUrl: "",
    protagonist: "",
    summary: "",
    genre: { setting: "ancient", names: "han" },
    glossary: { names: {}, places: {}, items: {}, creatures: {}, skills: {}, common: {}, signature_phrases: {} },
    style: { voice: "", toneRules: [], signaturePhrases: {}, avoid: [] },
    customPrompt: "",
    checkRules: [],
    autoGlossaryLog: [],
    autoGlossary: "inherit",
  },
  sessionRunning: false,
  progressDrift: { untracked: 0, missing: 1 },
});

describe("SyncProgressDialog", () => {
  beforeEach(() => {
    vi.mocked(progressDrift).mockReset();
    vi.mocked(syncProgress).mockReset();
    vi.mocked(toast.success).mockReset();
    useStoryStore.setState({ root: ROOT, snapshot: undefined, sessions: {} });
  });

  it("mở thì xem trước số chương sẽ đánh dấu và số chương thiếu bản dịch; Đồng bộ gọi lệnh, nạp snapshot, đóng", async () => {
    const user = userEvent.setup();
    const onOpenChange = vi.fn();
    vi.mocked(progressDrift).mockResolvedValue({ untracked: ["0645", "0646", "0647"], missing: ["0012"] });
    vi.mocked(syncProgress).mockResolvedValue({ markedDone: ["0645", "0646", "0647"], missing: ["0012"], snapshot: synced });
    render(<SyncProgressDialog root={ROOT} open onOpenChange={onOpenChange} />);
    expect(progressDrift).toHaveBeenCalledWith(ROOT);
    expect(await screen.findByText(/3 chương đã có bản dịch trong out\/ nhưng tiến độ ghi chưa dịch → sẽ đánh dấu đã dịch\./)).toBeInTheDocument();
    expect(
      screen.getByText(/1 chương ghi đã dịch nhưng chưa thấy bản dịch — giữ nguyên \(có thể Google Drive chưa đồng bộ xong\)\./),
    ).toBeInTheDocument();
    expect(screen.getByText(/Cảnh báo từng chương, cặp xưng hô và glossary học ở máy kia nằm trong story\.json \/ cast\.json/)).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Đồng bộ" }));
    await waitFor(() => expect(syncProgress).toHaveBeenCalledWith(ROOT));
    await waitFor(() => expect(onOpenChange).toHaveBeenCalledWith(false));
    expect(toast.success).toHaveBeenCalledWith("Đã đồng bộ 3 chương");
    expect(useStoryStore.getState().snapshot).toEqual(synced);
  });

  it("không có chương lệch thì khoá nút và không nêu dòng chương thiếu", async () => {
    vi.mocked(progressDrift).mockResolvedValue({ untracked: [], missing: [] });
    render(<SyncProgressDialog root={ROOT} open onOpenChange={vi.fn()} />);
    expect(await screen.findByText(/0 chương đã có bản dịch trong out\//)).toBeInTheDocument();
    expect(screen.queryByText(/chưa thấy bản dịch/)).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Đồng bộ" })).toBeDisabled();
  });

  it("phiên dịch đang chạy thì khoá nút", async () => {
    useStoryStore.setState({ sessions: { [pathKey(ROOT)]: { status: "running", sessionNo: 1 } } });
    vi.mocked(progressDrift).mockResolvedValue({ untracked: ["0645"], missing: [] });
    render(<SyncProgressDialog root={ROOT} open onOpenChange={vi.fn()} />);
    expect(await screen.findByText(/1 chương đã có bản dịch/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Đồng bộ" })).toBeDisabled();
    expect(screen.getByText(/bấm Dừng/)).toBeInTheDocument();
  });

  it("đóng thì không gọi xem trước", () => {
    render(<SyncProgressDialog root={ROOT} open={false} onOpenChange={vi.fn()} />);
    expect(progressDrift).not.toHaveBeenCalled();
  });

  it("Dịch lại các chương này (báo nhầm): gọi lệnh cất .bak, nạp snapshot, đóng; có dòng giải thích hai nút", async () => {
    const user = userEvent.setup();
    const onOpenChange = vi.fn();
    vi.mocked(progressDrift).mockResolvedValue({ untracked: ["0001", "0002"], missing: [] });
    vi.mocked(requeueUntracked)
      .mockReset()
      .mockResolvedValue({ requeued: ["0001", "0002"], snapshot: synced });
    render(<SyncProgressDialog root={ROOT} open onOpenChange={onOpenChange} />);
    await screen.findByText(/2 chương đã có bản dịch/);
    expect(screen.getByText(/Đồng bộ = ghi nhận đã dịch\. Dịch lại = cất bản dịch cũ sang \.bak rồi dịch lại/)).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Dịch lại các chương này" }));
    await waitFor(() => expect(requeueUntracked).toHaveBeenCalledWith(ROOT));
    await waitFor(() => expect(onOpenChange).toHaveBeenCalledWith(false));
    expect(syncProgress).not.toHaveBeenCalled();
    expect(useStoryStore.getState().snapshot).toEqual(synced);
  });

  it("không có chương lệch hoặc phiên đang chạy thì nút Dịch lại cũng khoá", async () => {
    vi.mocked(progressDrift).mockResolvedValue({ untracked: [], missing: [] });
    render(<SyncProgressDialog root={ROOT} open onOpenChange={vi.fn()} />);
    await screen.findByText(/0 chương đã có bản dịch/);
    expect(screen.getByRole("button", { name: "Dịch lại các chương này" })).toBeDisabled();
  });
});
