import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { toast } from "sonner";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { TitlesPanel } from "@/components/titles-panel";
import { titlesApply, titlesSurvey } from "@/lib/api";
import type { TitleSurvey } from "@/lib/types";

vi.mock("sonner", () => ({ toast: { message: vi.fn(), success: vi.fn(), error: vi.fn() } }));
vi.mock("@/lib/api", () => ({ titlesApply: vi.fn(), titlesSurvey: vi.fn() }));

const ROOT = "D:\\lib\\xuyen-nhanh";

const survey: TitleSurvey = {
  unparsed: 1,
  groups: [
    {
      source: "天煞孤星摄政王",
      target: "Thiên Sát Cô Tinh Nhiếp Chính Vương",
      settled: false,
      chapters: 60,
      translated: 57,
      variants: [
        { text: "Nhiếp chính vương thiên sát cô tinh", count: 21 },
        { text: "Thiên Sát Cô Tinh Nhiếp Chính Vương", count: 36 },
      ],
      pending: 21,
    },
    {
      source: "清衍伐北海",
      target: "Thanh Diễn phạt Bắc Hải",
      settled: true,
      chapters: 8,
      translated: 8,
      variants: [{ text: "Thanh Diễn phạt Bắc Hải", count: 8 }],
      pending: 0,
    },
    { source: "七零年代炮灰继女", target: "", settled: false, chapters: 23, translated: 0, variants: [], pending: 0 },
  ],
};

describe("TitlesPanel", () => {
  beforeEach(() => {
    vi.mocked(titlesSurvey).mockReset();
    vi.mocked(titlesApply).mockReset();
    vi.mocked(titlesSurvey).mockResolvedValue(survey);
  });

  it("liệt kê tên lặp, tên đang lệch kiểu dịch và số dòng sẽ sửa; lọc chỉ tên lệch", async () => {
    const user = userEvent.setup();
    render(<TitlesPanel root={ROOT} running={false} />);
    expect(await screen.findByText("天煞孤星摄政王")).toBeInTheDocument();
    expect(titlesSurvey).toHaveBeenCalledWith(ROOT);
    expect(screen.getByRole("status")).toHaveTextContent(
      "3 tên lặp · 1 tên đang bị dịch nhiều kiểu · 21 dòng tiêu đề sẽ được sửa khi lưu · 1 chương có dòng tiêu đề không tách được, để nguyên",
    );
    expect(screen.getByText("57/60 chương đã dịch · sửa 21")).toBeInTheDocument();
    expect(screen.getByLabelText("Bản dịch của 天煞孤星摄政王")).toHaveValue("Thiên Sát Cô Tinh Nhiếp Chính Vương");
    expect(screen.getByLabelText("Bản dịch của 七零年代炮灰继女")).toHaveValue("");

    await user.click(screen.getByLabelText("Chỉ tên đang lệch"));
    expect(screen.queryByText("清衍伐北海")).not.toBeInTheDocument();
    expect(screen.getByText("天煞孤星摄政王")).toBeInTheDocument();
  });

  it("chọn một kiểu dịch, điền trước tên chưa dịch rồi Lưu & áp dụng: chỉ gửi dòng đã đổi", async () => {
    vi.mocked(titlesApply).mockResolvedValue({
      changed: 36,
      survey: { unparsed: 0, groups: survey.groups.slice(1, 2) },
    });
    const user = userEvent.setup();
    render(<TitlesPanel root={ROOT} running={false} />);
    await user.click(await screen.findByRole("button", { name: /Nhiếp chính vương thiên sát cô tinh/ }));
    expect(screen.getByLabelText("Bản dịch của 天煞孤星摄政王")).toHaveValue("Nhiếp chính vương thiên sát cô tinh");
    await user.type(screen.getByLabelText("Bản dịch của 七零年代炮灰继女"), "Kế nữ pháo hôi thập niên 70 ");
    // Gõ rồi xoá hết một ô đã có bản dịch: không gửi chuỗi rỗng lên.
    await user.clear(screen.getByLabelText("Bản dịch của 清衍伐北海"));

    await user.click(screen.getByRole("button", { name: "Lưu & áp dụng" }));
    await waitFor(() =>
      expect(titlesApply).toHaveBeenCalledWith(ROOT, {
        天煞孤星摄政王: "Nhiếp chính vương thiên sát cô tinh",
        七零年代炮灰继女: "Kế nữ pháo hôi thập niên 70",
      }),
    );
    await waitFor(() =>
      expect(toast.success).toHaveBeenLastCalledWith("Đã chốt tiêu đề và sửa dòng tiêu đề của 36 chương"),
    );
    // Bảng nạp lại theo khảo sát mới.
    expect(screen.queryByText("天煞孤星摄政王")).not.toBeInTheDocument();
    expect(screen.getByLabelText("Bản dịch của 清衍伐北海")).toHaveValue("Thanh Diễn phạt Bắc Hải");
  });

  it("chưa sửa gì nhưng còn dòng lệch thì vẫn lưu được (chốt theo đa số); đang dịch thì khoá nút", async () => {
    vi.mocked(titlesApply).mockResolvedValue({ changed: 21, survey });
    const user = userEvent.setup();
    const { unmount } = render(<TitlesPanel root={ROOT} running={false} />);
    await screen.findByText("天煞孤星摄政王");
    expect(screen.getByRole("button", { name: "Hoàn tác" })).toBeDisabled();
    await user.click(screen.getByRole("button", { name: "Lưu & áp dụng" }));
    await waitFor(() => expect(titlesApply).toHaveBeenCalledWith(ROOT, {}));
    unmount();

    render(<TitlesPanel root={ROOT} running />);
    expect(await screen.findByRole("button", { name: "Lưu & áp dụng" })).toBeDisabled();
  });

  it("truyện không có tên lặp thì nói rõ, không hiện bảng", async () => {
    vi.mocked(titlesSurvey).mockResolvedValue({ groups: [], unparsed: 0 });
    render(<TitlesPanel root={ROOT} running={false} />);
    expect(await screen.findByText(/không có tên tiêu đề nào lặp lại/)).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Lưu & áp dụng" })).not.toBeInTheDocument();
  });
});
