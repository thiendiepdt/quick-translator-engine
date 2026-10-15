import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, onTestFinished, vi } from "vitest";

import { VnReplaceDialog } from "@/components/vn-replace-dialog";
import { vnReplace, vnSettingsGet } from "@/lib/api";
import type { StorySnapshot, VnHitView } from "@/lib/types";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({ vnReplace: vi.fn(), vnSettingsGet: vi.fn() }));

const ROOT = "D:\\t";
const base: Omit<VnHitView, "id" | "chapter" | "paragraph"> = {
  group: "ten",
  level: "canxem",
  side: "out",
  count: 1,
  fingerprint: "x",
  status: "pending",
  ai: null,
  raw: [],
  out: [],
  focus: 0,
  entry: "viet-nam",
  matched: "Việt Nam",
};
const a: VnHitView = { ...base, id: "0001:1:viet-nam", chapter: "0001", paragraph: 1 };
const b: VnHitView = { ...base, id: "0002:3:viet-nam", chapter: "0002", paragraph: 3 };
const c: VnHitView = { ...base, id: "0009:2:viet-nam", chapter: "0009", paragraph: 2 };
const monkey: VnHitView = { ...base, id: "0226:4:viet-nam", chapter: "0226", paragraph: 4, matched: "khỉ Việt Nam" };
const giao: VnHitView = { ...base, id: "0354:7:giao-chi", chapter: "0354", paragraph: 7, entry: "giao-chi", matched: "Giao Chỉ" };

function renderDialog(hits: VnHitView[]) {
  const onDone = vi.fn();
  render(<VnReplaceDialog root={ROOT} setting="modern" hits={hits} allHits={[a, b, c]} open onOpenChange={vi.fn()} onDone={onDone} />);
  return onDone;
}

describe("VnReplaceDialog", () => {
  beforeEach(() => {
    vi.mocked(vnSettingsGet)
      .mockReset()
      .mockResolvedValue({ extra: [], disabled: [], replacements: [{ from: "Việt Nam", to: "Thái Lan", setting: "modern" }] });
    vi.mocked(vnReplace).mockReset().mockResolvedValue({ chapters: 2, occurrences: 2 });
  });

  it("điền sẵn cụm gốc + cụm thay theo bảng; phạm vi 'chỗ đang chọn' gửi đúng đoạn", async () => {
    const user = userEvent.setup();
    const onDone = renderDialog([a, b]);
    expect(await screen.findByDisplayValue("Thái Lan")).toBeInTheDocument();
    expect(screen.getByLabelText("Cụm gốc 1")).toHaveValue("Việt Nam");
    expect(screen.getByText(/2 chỗ ở 2 chương/)).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Thay" }));
    expect(vnReplace).toHaveBeenCalledWith("D:\\t", [
      {
        from: "Việt Nam",
        to: "Thái Lan",
        chapters: ["0001", "0002"],
        paragraphs: [
          ["0001", 1],
          ["0002", 3],
        ],
      },
    ]);
    expect(onDone).toHaveBeenCalled();
  });

  it("phạm vi 'mọi chương có chỗ khớp' không gửi danh sách đoạn", async () => {
    const user = userEvent.setup();
    renderDialog([a]);
    await screen.findByDisplayValue("Thái Lan");
    await user.click(screen.getByRole("radio", { name: /Mọi chương có chỗ khớp/ }));
    expect(screen.getByText("Sẽ thay mọi chỗ của 1 cụm trong 3 chương (đã phát hiện 3 chỗ).")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Thay" }));
    expect(vnReplace).toHaveBeenCalledWith("D:\\t", [{ from: "Việt Nam", to: "Thái Lan", chapters: ["0001", "0002", "0009"] }]);
  });

  it("phạm vi theo chương bỏ chương chỉ còn chỗ đã 'Không sao'", async () => {
    const user = userEvent.setup();
    render(
      <VnReplaceDialog
        root={ROOT}
        setting="modern"
        hits={[a]}
        allHits={[a, b, { ...c, status: "ok" }]}
        open
        onOpenChange={vi.fn()}
        onDone={vi.fn()}
      />,
    );
    await screen.findByDisplayValue("Thái Lan");
    await user.click(screen.getByRole("radio", { name: /Chọn chương/ }));
    expect(screen.queryByRole("checkbox", { name: "0009" })).not.toBeInTheDocument();
    await user.click(screen.getByRole("radio", { name: /Mọi chương có chỗ khớp/ }));
    expect(screen.getByText("Sẽ thay mọi chỗ của 1 cụm trong 2 chương (đã phát hiện 2 chỗ).")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Thay" }));
    expect(vnReplace).toHaveBeenCalledWith("D:\\t", [{ from: "Việt Nam", to: "Thái Lan", chapters: ["0001", "0002"] }]);
  });

  it("phạm vi 'chọn chương' dùng các chương được tick", async () => {
    const user = userEvent.setup();
    renderDialog([a]);
    await screen.findByDisplayValue("Thái Lan");
    await user.click(screen.getByRole("radio", { name: /Chọn chương/ }));
    await user.click(screen.getByRole("checkbox", { name: "0009" }));
    await user.click(screen.getByRole("button", { name: "Thay" }));
    expect(vnReplace).toHaveBeenCalledWith("D:\\t", [{ from: "Việt Nam", to: "Thái Lan", chapters: ["0001", "0009"] }]);
  });

  it("phạm vi 'chọn chương': tick theo khoảng (số thứ tự hoặc mã chương)", async () => {
    const user = userEvent.setup();
    const previous = useStoryStore.getState().snapshot;
    onTestFinished(() => useStoryStore.setState({ snapshot: previous }));
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
  it("xem trước cộng số lần xuất hiện (count), gộp theo đoạn, và chỉ gửi mỗi đoạn một lần", async () => {
    const user = userEvent.setup();
    const many: VnHitView = { ...a, count: 3 };
    const sameParagraph: VnHitView = { ...a, id: "0001:1:khac", entry: "khac", count: 2 };
    renderDialog([many, sameParagraph, b]);
    await screen.findByDisplayValue("Thái Lan");
    expect(screen.getByText("Sẽ thay 4 chỗ ở 2 chương.")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Thay" }));
    expect(vnReplace).toHaveBeenCalledWith("D:\\t", [
      {
        from: "Việt Nam",
        to: "Thái Lan",
        chapters: ["0001", "0002"],
        paragraphs: [
          ["0001", 1],
          ["0002", 3],
        ],
      },
    ]);
  });

  it("đổi Cụm gốc thì gợi ý lại Cụm thay theo bối cảnh, trừ khi người dùng đã tự sửa", async () => {
    vi.mocked(vnSettingsGet).mockResolvedValue({
      extra: [],
      disabled: [],
      replacements: [
        { from: "Việt Nam", to: "Thái Lan", setting: "modern" },
        { from: "Giao Chỉ", to: "Nam Cương", setting: "ancient" },
        { from: "Giao Chỉ", to: "Nam Quốc", setting: "modern" },
      ],
    });
    const user = userEvent.setup();
    const onDone = vi.fn();
    render(<VnReplaceDialog root={ROOT} setting="ancient" hits={[a]} allHits={[a, b, c]} open onOpenChange={vi.fn()} onDone={onDone} />);
    await screen.findByLabelText("Cụm thay 1");
    const from = screen.getByLabelText("Cụm gốc 1");
    await user.clear(from);
    await user.type(from, "Giao Chỉ");
    expect(screen.getByLabelText("Cụm thay 1")).toHaveValue("Nam Cương");
    await user.type(screen.getByLabelText("Cụm thay 1"), "!");
    await user.clear(from);
    await user.type(from, "Việt Nam");
    expect(screen.getByLabelText("Cụm thay 1")).toHaveValue("Nam Cương!");
  });

  it("chọn toàn chỗ ở raw thì báo chưa thay được và khoá nút Thay", async () => {
    const raw: VnHitView = { ...a, side: "raw" };
    renderDialog([raw]);
    expect(await screen.findByText(/chỉ có ở raw \(chương chưa dịch\) — dịch xong mới thay được/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Thay" })).toBeDisabled();
  });

  it("chọn hai chỗ khác cụm: hai dòng, gợi ý theo bảng (cả phần chứa), gửi cả hai cặp đúng đoạn", async () => {
    vi.mocked(vnSettingsGet).mockResolvedValue({
      extra: [],
      disabled: [],
      replacements: [
        { from: "Việt Nam", to: "Thái Lan", setting: "modern" },
        { from: "Việt Nam", to: "Nam Việt", setting: "ancient" },
        { from: "Nam", to: "Bắc", setting: "any" },
        { from: "Giao Chỉ", to: "Chiêm Thành", setting: "ancient" },
      ],
    });
    const user = userEvent.setup();
    const onDone = renderDialog([monkey, giao]);
    expect(await screen.findByDisplayValue("khỉ Thái Lan")).toBeInTheDocument();
    expect(screen.getByLabelText("Cụm gốc 1")).toHaveValue("khỉ Việt Nam");
    expect(screen.getByLabelText("Cụm thay 1")).toHaveValue("khỉ Thái Lan");
    expect(screen.getByLabelText("Cụm gốc 2")).toHaveValue("Giao Chỉ");
    expect(screen.getByLabelText("Cụm thay 2")).toHaveValue("Chiêm Thành");
    expect(screen.queryByText(/khác cụm gốc/)).not.toBeInTheDocument();
    expect(screen.getByText("Sẽ thay 2 chỗ ở 2 chương.")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Thay" }));
    expect(vnReplace).toHaveBeenCalledWith("D:\\t", [
      { from: "khỉ Việt Nam", to: "khỉ Thái Lan", chapters: ["0226"], paragraphs: [["0226", 4]] },
      { from: "Giao Chỉ", to: "Chiêm Thành", chapters: ["0354"], paragraphs: [["0354", 7]] },
    ]);
    expect(onDone).toHaveBeenCalled();
  });

  it("bỏ tick một dòng thì không gửi cặp đó", async () => {
    const user = userEvent.setup();
    renderDialog([a, giao]);
    await screen.findByDisplayValue("Thái Lan");
    await user.type(screen.getByLabelText("Cụm thay 2"), "Chiêm Thành");
    expect(screen.getByText("Sẽ thay 2 chỗ ở 2 chương.")).toBeInTheDocument();
    await user.click(screen.getByRole("checkbox", { name: "Thay cụm 1" }));
    expect(screen.getByText("Sẽ thay 1 chỗ ở 1 chương.")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Thay" }));
    expect(vnReplace).toHaveBeenCalledWith("D:\\t", [{ from: "Giao Chỉ", to: "Chiêm Thành", chapters: ["0354"], paragraphs: [["0354", 7]] }]);
  });

  it("Cụm thay trống thì dòng đó không thay, có nhắc; mọi dòng trống thì khoá nút Thay", async () => {
    const user = userEvent.setup();
    renderDialog([a, giao]);
    await screen.findByDisplayValue("Thái Lan");
    expect(screen.getByText("Bỏ trống = không thay")).toBeInTheDocument();
    expect(screen.getByText("Sẽ thay 1 chỗ ở 1 chương.")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Thay" }));
    expect(vnReplace).toHaveBeenCalledWith("D:\\t", [{ from: "Việt Nam", to: "Thái Lan", chapters: ["0001"], paragraphs: [["0001", 1]] }]);
    await user.clear(screen.getByLabelText("Cụm thay 1"));
    expect(screen.getAllByText("Bỏ trống = không thay")).toHaveLength(2);
    expect(screen.getByRole("button", { name: "Thay" })).toBeDisabled();
  });

  it("gợi ý theo phần chứa chỉ khi khớp nguyên từ, ưu tiên cụm dài nhất", async () => {
    vi.mocked(vnSettingsGet).mockResolvedValue({
      extra: [],
      disabled: [],
      replacements: [
        { from: "Nam", to: "Bắc", setting: "any" },
        { from: "Việt Nam", to: "Thái Lan", setting: "any" },
        { from: "Việt", to: "Lào", setting: "any" },
      ],
    });
    const user = userEvent.setup();
    renderDialog([monkey]);
    await screen.findByDisplayValue("khỉ Thái Lan");
    const from = screen.getByLabelText("Cụm gốc 1");
    await user.clear(from);
    await user.type(from, "Namkha Việtx");
    expect(screen.getByLabelText("Cụm thay 1")).toHaveValue("");
    await user.clear(from);
    await user.type(from, "người Việt");
    expect(screen.getByLabelText("Cụm thay 1")).toHaveValue("người Lào");
  });

  it("phạm vi theo chương với nhiều cụm: mỗi cặp chỉ gửi chương có cụm đó", async () => {
    const user = userEvent.setup();
    const giao2: VnHitView = { ...giao, id: "0009:5:giao-chi", chapter: "0009", paragraph: 5 };
    render(
      <VnReplaceDialog root={ROOT} setting="modern" hits={[a, giao]} allHits={[a, b, c, giao, giao2]} open onOpenChange={vi.fn()} onDone={vi.fn()} />,
    );
    await screen.findByDisplayValue("Thái Lan");
    await user.type(screen.getByLabelText("Cụm thay 2"), "Chiêm Thành");
    await user.click(screen.getByRole("radio", { name: /Chọn chương/ }));
    expect(screen.getAllByRole("checkbox", { name: /^\d{4}$/ }).map((box) => box.getAttribute("aria-label"))).toEqual([
      "0001",
      "0002",
      "0009",
      "0354",
    ]);
    await user.click(screen.getByRole("checkbox", { name: "0009" }));
    expect(screen.getByText("Sẽ thay mọi chỗ của 2 cụm trong 3 chương (đã phát hiện 4 chỗ).")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Thay" }));
    expect(vnReplace).toHaveBeenCalledWith("D:\\t", [
      { from: "Việt Nam", to: "Thái Lan", chapters: ["0001", "0009"] },
      { from: "Giao Chỉ", to: "Chiêm Thành", chapters: ["0354", "0009"] },
    ]);
  });
});
