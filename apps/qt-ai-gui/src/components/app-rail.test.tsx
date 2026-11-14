import { act, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { AppRail } from "@/components/app-rail";
import { TooltipProvider } from "@/components/ui/tooltip";
import { rawInfo, rawToc, type RawInfo, type RawToc } from "@/lib/api";
import { invalidateRawChecks, resetRawChecks } from "@/store/raw-check";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({ rawInfo: vi.fn(), rawToc: vi.fn() }));
vi.mock("@/hooks/use-theme", () => ({ useThemeActions: () => ({ mode: "light", toggleMode: vi.fn() }) }));

const configured = {
  rawUrl: "https://a.test/book/1",
  extension: { id: "x", name: "Demo" },
  candidates: [{ id: "x", name: "Demo" }],
  installed: 1,
  extConfig: null,
} as unknown as RawInfo;
const toc = {
  rows: [1, 2, 3, 4].map((index) => ({ index, id: String(index), title: String(index), locked: false, file: index === 1 ? "1" : null, status: index === 1 ? "have" : "missing" })),
  lastIndex: 1,
} as unknown as RawToc;

describe("AppRail", () => {
  beforeEach(() => {
    resetRawChecks();
    vi.mocked(rawInfo).mockReset();
    vi.mocked(rawToc).mockReset().mockResolvedValue(toc);
  });

  it("mở truyện đã đủ cài đặt tải raw thì icon Tải raw hiện số chương mới", async () => {
    vi.mocked(rawInfo).mockResolvedValue(configured);
    useStoryStore.setState({ root: "D:\\lib\\a", page: "translate" } as never);
    render(
      <TooltipProvider>
        <AppRail />
      </TooltipProvider>,
    );
    const button = await screen.findByRole("button", { name: "Tải raw — 3 chương mới" });
    expect(button).toHaveTextContent("3");
  });

  it("nguồn chưa có extension: icon Tải raw có chấm cảnh báo kèm lý do; cài extension xong thì kiểm lại", async () => {
    vi.mocked(rawInfo).mockResolvedValue({ ...configured, rawUrl: "https://www.book.qq.com/book-detail/1", candidates: [], extension: null, installed: 2 });
    useStoryStore.setState({ root: "D:\\lib\\c", page: "translate" } as never);
    render(
      <TooltipProvider>
        <AppRail />
      </TooltipProvider>,
    );
    expect(await screen.findByRole("button", { name: "Tải raw — chưa có extension tải raw cho book.qq.com" })).toBeInTheDocument();
    expect(rawToc).not.toHaveBeenCalled();

    vi.mocked(rawInfo).mockResolvedValue(configured);
    act(() => invalidateRawChecks());
    expect(await screen.findByRole("button", { name: "Tải raw — 3 chương mới" })).toBeInTheDocument();
  });

  it("chưa chọn extension thì không gọi extension, không có số", async () => {
    vi.mocked(rawInfo).mockResolvedValue({ ...configured, candidates: [configured.extension!, configured.extension!], extension: null });
    useStoryStore.setState({ root: "D:\\lib\\b", page: "translate" } as never);
    render(
      <TooltipProvider>
        <AppRail />
      </TooltipProvider>,
    );
    await waitFor(() => expect(rawInfo).toHaveBeenCalledWith("D:\\lib\\b"));
    expect(rawToc).not.toHaveBeenCalled();
    expect(await screen.findByRole("button", { name: "Tải raw — có 2 extension khớp link, chưa chọn cái nào" })).toBeInTheDocument();
  });
});
