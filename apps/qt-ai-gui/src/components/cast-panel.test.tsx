import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { CastPanel } from "@/components/cast-panel";
import { castCleanAddressing, castLoad, castSave, castScan } from "@/lib/api";
import { castSchema } from "@/lib/schema";

vi.mock("@/lib/api", () => ({
  castLoad: vi.fn(),
  castSave: vi.fn(),
  castScan: vi.fn(),
  castCleanAddressing: vi.fn(),
}));

const names = { 贺静昭: "Hạ Tĩnh Chiêu", 莫衡: "Mạc Hành", 叶清禾: "Diệp Thanh Hòa" };
const addressing = { "宋时安→贺静昭": "tôi–thầy Hạ", "莫衡→贺静昭": "tôi–cô" };
const cast = castSchema.parse({
  characters: {
    贺静昭: { gender: "female", source: "auto", chapter: "0132", disputed: ["0140"] },
    莫衡: { gender: "male" },
  },
  addressing: {
    "莫衡→贺静昭": { pinned: false, changes: [{ from: "0150", target: "anh–em", note: "thành người yêu" }] },
  },
});

function renderPanel(onStoryChanged = vi.fn()) {
  render(
    <CastPanel root={"D:\\t"} names={names} addressing={addressing} chapterIds={["0001", "0150", "0200"]} running={false} onStoryChanged={onStoryChanged} />,
  );
  return onStoryChanged;
}

describe("CastPanel", () => {
  beforeEach(() => {
    vi.mocked(castLoad).mockReset().mockResolvedValue(cast);
    vi.mocked(castSave).mockReset().mockImplementation((_root, next) => Promise.resolve(next));
    vi.mocked(castScan).mockReset();
    vi.mocked(castCleanAddressing).mockReset();
  });

  it("hiện giới từng nhân vật, cảnh báo tranh chấp, đổi giới rồi Lưu ghi source user", async () => {
    const user = userEvent.setup();
    renderPanel();
    const row = (await screen.findByText("贺静昭")).closest("li")!;
    expect(within(row).getByLabelText("Giới tính 贺静昭")).toHaveValue("female");
    expect(within(row).getByTitle(/Chương 0140 khai giới ngược/)).toBeInTheDocument();
    const unknown = screen.getByText("叶清禾").closest("li")!;
    expect(within(unknown).getByLabelText("Giới tính 叶清禾")).toHaveValue("");

    expect(screen.getByRole("button", { name: "Lưu bảng nhân vật" })).toBeDisabled();
    await user.selectOptions(within(unknown).getByLabelText("Giới tính 叶清禾"), "female");
    await user.click(screen.getByRole("button", { name: "Lưu bảng nhân vật" }));
    const saved = vi.mocked(castSave).mock.calls[0][1];
    expect(saved.characters["叶清禾"]).toMatchObject({ gender: "female", source: "user" });
    expect(saved.characters["贺静昭"]).toMatchObject({ gender: "female", source: "auto" });
  });

  it("liệt kê mốc đổi xưng hô, hoàn tác mốc và ghim cặp", async () => {
    const user = userEvent.setup();
    renderPanel();
    const list = await screen.findByRole("list", { name: "Mốc đổi xưng hô" });
    const item = within(list).getByText("莫衡→贺静昭").closest("li")!;
    expect(within(item).getByText(/từ chương 0150/)).toBeInTheDocument();
    expect(within(item).getByText("anh–em")).toBeInTheDocument();
    expect(within(item).getByText(/thành người yêu/)).toBeInTheDocument();
    await user.click(within(item).getByRole("button", { name: "Ghim cặp 莫衡→贺静昭" }));
    await user.click(within(item).getByRole("button", { name: "Hoàn tác mốc 0150" }));
    await user.click(screen.getByRole("button", { name: "Lưu bảng nhân vật" }));
    const saved = vi.mocked(castSave).mock.calls[0][1];
    expect(saved.addressing["莫衡→贺静昭"]).toEqual({ pinned: true, changes: [] });
  });

  it("đặt mốc tay: cặp + chương + xưng hô mới → thêm mốc source user và ghim", async () => {
    const user = userEvent.setup();
    renderPanel();
    await screen.findByText("贺静昭");
    await user.selectOptions(screen.getByLabelText("Cặp xưng hô"), "宋时安→贺静昭");
    await user.selectOptions(screen.getByLabelText("Từ chương"), "0200");
    await user.type(screen.getByLabelText("Xưng hô mới"), "em–chị");
    await user.click(screen.getByRole("button", { name: "Thêm mốc" }));
    await user.click(screen.getByRole("button", { name: "Lưu bảng nhân vật" }));
    const saved = vi.mocked(castSave).mock.calls[0][1];
    expect(saved.addressing["宋时安→贺静昭"]).toEqual({
      pinned: true,
      changes: [{ from: "0200", target: "em–chị", source: "user" }],
    });
  });

  it("quét giới tính nạp bảng mới và báo số điền được; dọn cặp trái giới báo story đổi", async () => {
    const user = userEvent.setup();
    const scanned = castSchema.parse({ ...cast, characters: { ...cast.characters, 叶清禾: { gender: "female" } } });
    vi.mocked(castScan).mockResolvedValue({ asked: 1, filled: 1, failedBatches: 0, cast: scanned });
    vi.mocked(castCleanAddressing).mockResolvedValue(["宋时安→贺静昭"]);
    const onStoryChanged = renderPanel();
    await screen.findByText("贺静昭");
    await user.click(screen.getByRole("button", { name: /Quét giới tính/ }));
    expect(await screen.findByText(/Đã điền 1 \/ 1 tên/)).toBeInTheDocument();
    expect(within(screen.getByText("叶清禾").closest("li")!).getByLabelText("Giới tính 叶清禾")).toHaveValue("female");

    await user.click(screen.getByRole("button", { name: "Dọn cặp trái giới" }));
    expect(await screen.findByText(/Đã xoá 1 cặp trái giới/)).toBeInTheDocument();
    expect(castCleanAddressing).toHaveBeenCalledWith("D:\\t");
    expect(onStoryChanged).toHaveBeenCalled();
  });
});
