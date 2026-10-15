import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { VnReplaceDialog } from "@/components/vn-replace-dialog";
import { vnReplace, vnSettingsGet } from "@/lib/api";
import type { StorySnapshot, VnHitView } from "@/lib/types";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({ vnReplace: vi.fn(), vnSettingsGet: vi.fn() }));

const ROOT = "D:\\t";
const base: Omit<VnHitView, "id" | "chapter" | "paragraph"> = { group: "ten", level: "canxem", side: "out", count: 1, fingerprint: "x", status: "pending", ai: null, raw: [], out: [], focus: 0, entry: "viet-nam", matched: "Việt Nam" };
const a: VnHitView = { ...base, id: "0001:1:viet-nam", chapter: "0001", paragraph: 1 };
const b: VnHitView = { ...base, id: "0002:3:viet-nam", chapter: "0002", paragraph: 3 };
const c: VnHitView = { ...base, id: "0009:2:viet-nam", chapter: "0009", paragraph: 2 };

function renderDialog(hits: VnHitView[]) {
  const onDone = vi.fn();
  render(<VnReplaceDialog root={ROOT} setting="modern" hits={hits} allHits={[a, b, c]} open onOpenChange={vi.fn()} onDone={onDone} />);
  return onDone;
}

describe("VnReplaceDialog", () => {
  beforeEach(() => {
    vi.mocked(vnSettingsGet).mockReset().mockResolvedValue({ extra: [], disabled: [], replacements: [{ from: "Việt Nam", to: "Thái Lan", setting: "modern" }] });
    vi.mocked(vnReplace).mockReset().mockResolvedValue({ chapters: 2, occurrences: 2 });
  });

  it("điền sẵn cụm gốc + cụm thay theo bảng; phạm vi 'chỗ đang chọn' gửi đúng đoạn", async () => {
    const user = userEvent.setup();
    const onDone = renderDialog([a, b]);
    expect(await screen.findByDisplayValue("Thái Lan")).toBeInTheDocument();
    expect(screen.getByLabelText("Cụm gốc")).toHaveValue("Việt Nam");
    expect(screen.getByText(/2 chỗ ở 2 chương/)).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Thay" }));
    expect(vnReplace).toHaveBeenCalledWith("D:\\t", {
      from: "Việt Nam",
      to: "Thái Lan",
      chapters: ["0001", "0002"],
      paragraphs: [["0001", 1], ["0002", 3]],
    });
    expect(onDone).toHaveBeenCalled();
  });

  it("phạm vi 'mọi chương có chỗ khớp' không gửi danh sách đoạn", async () => {
    const user = userEvent.setup();
    renderDialog([a]);
    await screen.findByDisplayValue("Thái Lan");
    await user.click(screen.getByRole("radio", { name: /Mọi chương có chỗ khớp/ }));
    expect(screen.getByText(/3 chỗ ở 3 chương/)).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Thay" }));
    expect(vnReplace).toHaveBeenCalledWith("D:\\t", { from: "Việt Nam", to: "Thái Lan", chapters: ["0001", "0002", "0009"] });
  });

  it("phạm vi 'chọn chương' dùng các chương được tick", async () => {
    const user = userEvent.setup();
    renderDialog([a]);
    await screen.findByDisplayValue("Thái Lan");
    await user.click(screen.getByRole("radio", { name: /Chọn chương/ }));
    await user.click(screen.getByRole("checkbox", { name: "0009" }));
    await user.click(screen.getByRole("button", { name: "Thay" }));
    expect(vnReplace).toHaveBeenCalledWith("D:\\t", { from: "Việt Nam", to: "Thái Lan", chapters: ["0001", "0009"] });
  });

  it("phạm vi 'chọn chương': tick theo khoảng (số thứ tự hoặc mã chương)", async () => {
    const user = userEvent.setup();
    useStoryStore.setState({ snapshot: { chapters: ["0001", "0002", "0005", "0009"].map((id) => ({ id })) } as unknown as StorySnapshot });
    renderDialog([a]);
    await screen.findByDisplayValue("Thái Lan");
    await user.click(screen.getByRole("radio", { name: /Chọn chương/ }));
    await user.type(screen.getByLabelText("Từ chương"), "2");
    await user.type(screen.getByLabelText("Tới chương"), "0009");
    await user.click(screen.getByRole("button", { name: "Tick khoảng" }));
    expect(screen.getByRole("checkbox", { name: "0001" })).not.toBeChecked();
    expect(screen.getByRole("checkbox", { name: "0002" })).toBeChecked();
    expect(screen.getByRole("checkbox", { name: "0009" })).toBeChecked();
  });
});
