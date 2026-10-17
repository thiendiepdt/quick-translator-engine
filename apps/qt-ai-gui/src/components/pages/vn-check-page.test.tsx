import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { AppRail } from "@/components/app-rail";
import { VnCheckPage } from "@/components/pages/vn-check-page";
import { TooltipProvider } from "@/components/ui/tooltip";
import { useStoryStore } from "@/store/story";

vi.mock("@/components/vn-check-panel", () => ({
  VnCheckPanel: (props: { root: string; running: boolean; setting: string }) => (
    <div data-testid="vn-panel">{`${props.root}|${props.running}|${props.setting}`}</div>
  ),
}));

describe("trang Kiểm tra VN trên rail", () => {
  beforeEach(() => {
    useStoryStore.setState({
      page: "translate",
      root: "D:\\t",
      sessions: {},
      snapshot: { root: "D:\\t", story: { name: "T", genre: { setting: "ancient" } }, chapters: [] } as never,
    });
  });

  it("rail có nút Kiểm tra VN mở trang vn", async () => {
    render(
      <TooltipProvider>
        <AppRail />
      </TooltipProvider>,
    );
    await userEvent.click(screen.getByRole("button", { name: "Kiểm tra VN" }));
    expect(useStoryStore.getState().page).toBe("vn");
  });

  it("trang dựng đúng panel Kiểm tra VN của truyện đang mở", () => {
    render(<VnCheckPage />);
    expect(screen.getByRole("heading", { name: "Kiểm tra nội dung nhạy cảm về Việt Nam" })).toBeInTheDocument();
    expect(screen.getByTestId("vn-panel")).toHaveTextContent("D:\\t|false|ancient");
  });
});
