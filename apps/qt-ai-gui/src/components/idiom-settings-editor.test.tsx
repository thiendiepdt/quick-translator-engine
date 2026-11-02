import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { IdiomSettingsEditor } from "@/components/idiom-settings-editor";
import { idiomsGet, idiomsRefresh, idiomsSaveUser, type IdiomsInfo } from "@/lib/api";

vi.mock("@/lib/api", () => ({ idiomsGet: vi.fn(), idiomsRefresh: vi.fn(), idiomsSaveUser: vi.fn() }));
vi.mock("sonner", () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

const info: IdiomsInfo = {
  remote: { "三十年河东，三十年河西": "ba mươi năm hà đông, ba mươi năm hà tây", 打草惊蛇: "đánh rắn động cỏ" },
  source: "cache",
  fetchedAt: new Date(2026, 10, 2, 8, 15).getTime(),
  error: null,
  user: {},
  url: "https://raw.githubusercontent.com/x/data/thanh-ngu.json",
};

describe("IdiomSettingsEditor", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    vi.mocked(idiomsGet).mockReset().mockResolvedValue(info);
    vi.mocked(idiomsSaveUser).mockReset().mockImplementation((user) => Promise.resolve({ ...info, user }));
    vi.mocked(idiomsRefresh).mockReset().mockResolvedValue({ ...info, error: "Không tải được bảng thành ngữ: timeout" });
  });

  it("hiện bảng chung; thêm câu, sửa câu chung (đè), tắt câu chung rồi Lưu bảng của người dùng", async () => {
    const user = userEvent.setup();
    render(<IdiomSettingsEditor />);
    expect(await screen.findByText(/Bảng chung:/)).toHaveTextContent("Bảng chung: 2 câu · tải lúc 08:15");
    const save = screen.getByRole("button", { name: /Lưu/ });
    expect(save).toBeDisabled();

    await user.type(screen.getByLabelText("Thành ngữ chữ Hán"), "对牛弹琴");
    await user.type(screen.getByLabelText("Cách dịch"), "đàn gảy tai trâu{Enter}");
    expect(within(screen.getByRole("listitem", { name: "对牛弹琴" })).getByText("Của bạn")).toBeInTheDocument();

    const field = screen.getByLabelText("Cách dịch 打草惊蛇");
    await user.clear(field);
    await user.type(field, "rút dây động rừng");
    expect(within(screen.getByRole("listitem", { name: "打草惊蛇" })).getByText("Đã sửa")).toBeInTheDocument();

    const classic = screen.getByRole("listitem", { name: "三十年河东，三十年河西" });
    await user.click(within(classic).getByRole("button", { name: "Tắt" }));
    expect(within(classic).getByText("Đã tắt")).toBeInTheDocument();

    await user.click(save);
    expect(idiomsSaveUser).toHaveBeenCalledWith({ 对牛弹琴: "đàn gảy tai trâu", 打草惊蛇: "rút dây động rừng", "三十年河东，三十年河西": "" });
    expect(save).toBeDisabled();
  });

  it("Tải lại lỗi thì báo lỗi, giữ bản cũ; tìm lọc theo cách dịch", async () => {
    const user = userEvent.setup();
    render(<IdiomSettingsEditor />);
    await user.click(await screen.findByRole("button", { name: /Tải lại/ }));
    expect(idiomsRefresh).toHaveBeenCalled();
    expect(await screen.findByText(/Lần tải gần nhất lỗi: .*timeout — đang dùng bản cũ/)).toBeInTheDocument();
    await user.type(screen.getByLabelText("Tìm thành ngữ"), "rắn");
    expect(screen.getAllByRole("listitem").map((item) => item.getAttribute("aria-label"))).toEqual(["打草惊蛇"]);
  });

  it("chưa tải được lần nào thì nói đang dùng bản đóng sẵn", async () => {
    vi.mocked(idiomsGet).mockResolvedValue({ ...info, source: "builtin", fetchedAt: null });
    render(<IdiomSettingsEditor />);
    expect(await screen.findByText(/bản đóng sẵn trong app/)).toBeInTheDocument();
  });
});
