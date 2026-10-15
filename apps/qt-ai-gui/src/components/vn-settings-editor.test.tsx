import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { VnSettingsEditor } from "@/components/vn-settings-editor";
import { vnBuiltinEntries, vnSettingsGet, vnSettingsSave } from "@/lib/api";
import type { VnEntry, VnSettings } from "@/lib/types";

vi.mock("@/lib/api", () => ({ vnBuiltinEntries: vi.fn(), vnSettingsGet: vi.fn(), vnSettingsSave: vi.fn() }));

const builtin: VnEntry[] = [
  { id: "viet-nam", group: "ten", level: "canxem", weak: false, zh: ["越南"], vi: ["Việt Nam"], exclude: [] },
  { id: "nam-hai", group: "cq", level: "canxem", weak: true, zh: ["南海"], vi: ["Nam Hải", "Biển Đông"], exclude: [] },
];
const settings: VnSettings = {
  extra: [],
  disabled: [],
  replacements: [{ from: "Việt Nam", to: "Thái Lan", setting: "modern" }],
};

describe("VnSettingsEditor", () => {
  beforeEach(() => {
    vi.mocked(vnBuiltinEntries).mockReset().mockResolvedValue(builtin);
    vi.mocked(vnSettingsGet).mockReset().mockResolvedValue(settings);
    vi.mocked(vnSettingsSave)
      .mockReset()
      .mockImplementation((next) => Promise.resolve(next));
  });

  it("sửa, thêm dòng bảng thay rồi Lưu", async () => {
    const user = userEvent.setup();
    render(<VnSettingsEditor />);
    const list = await screen.findByRole("list", { name: "Bảng thay" });
    const to = within(within(list).getAllByRole("listitem")[0]).getByLabelText("Cụm thay");
    await user.clear(to);
    await user.type(to, "Xiêm La");
    await user.click(screen.getByRole("button", { name: "Thêm dòng thay" }));
    const second = within(list).getAllByRole("listitem")[1];
    await user.type(within(second).getByLabelText("Cụm gốc"), "Giao Chỉ");
    await user.type(within(second).getByLabelText("Cụm thay"), "Chiêm Thành");
    await user.selectOptions(within(second).getByLabelText("Bối cảnh"), "ancient");
    await user.click(screen.getByRole("button", { name: "Lưu cấu hình VN" }));
    expect(vnSettingsSave).toHaveBeenCalledWith({
      extra: [],
      disabled: [],
      replacements: [
        { from: "Việt Nam", to: "Xiêm La", setting: "modern" },
        { from: "Giao Chỉ", to: "Chiêm Thành", setting: "ancient" },
      ],
    });
  });

  it("tắt mục có sẵn, thêm mục riêng, bỏ dòng trống khi lưu", async () => {
    const user = userEvent.setup();
    render(<VnSettingsEditor />);
    await user.click(await screen.findByRole("checkbox", { name: /nam-hai/ }));
    await user.click(screen.getByRole("button", { name: "Thêm mục riêng" }));
    const extra = within(screen.getByRole("list", { name: "Mục riêng" })).getAllByRole("listitem")[0];
    await user.type(within(extra).getByLabelText("Chữ Hán"), "猴子国, 猴国人");
    await user.type(within(extra).getByLabelText("Tiếng Việt"), "nước khỉ con");
    await user.click(within(extra).getByRole("checkbox", { name: "Miệt thị" }));
    await user.click(screen.getByRole("button", { name: "Thêm dòng thay" }));
    await user.click(screen.getByRole("button", { name: "Lưu cấu hình VN" }));
    expect(vnSettingsSave).toHaveBeenCalledWith({
      disabled: ["nam-hai"],
      extra: [{ id: "rieng-1", group: "ten", level: "mietthi", weak: false, zh: ["猴子国", "猴国人"], vi: ["nước khỉ con"], exclude: [] }],
      replacements: [{ from: "Việt Nam", to: "Thái Lan", setting: "modern" }],
    });
  });
});
