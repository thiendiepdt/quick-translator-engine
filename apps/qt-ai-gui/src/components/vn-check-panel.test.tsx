import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { toast } from "sonner";
import { beforeEach, describe, expect, it, onTestFinished, vi } from "vitest";

import { VnCheckPanel } from "@/components/vn-check-panel";
import { vnAiVerify, vnRefresh, vnReport, vnSetStatus } from "@/lib/api";
import type { VnHitView } from "@/lib/types";
import { useStoryStore } from "@/store/story";

vi.mock("sonner", () => ({ toast: { message: vi.fn(), success: vi.fn(), error: vi.fn() } }));
vi.mock("@/lib/api", () => ({ vnRefresh: vi.fn(), vnReport: vi.fn(), vnSetStatus: vi.fn(), vnAiVerify: vi.fn(), vnReplace: vi.fn(), vnSettingsGet: vi.fn() }));

const hit = (over: Partial<VnHitView>): VnHitView => ({
  id: "0226:4:khi-viet",
  chapter: "0226",
  paragraph: 4,
  entry: "khi-viet",
  group: "ten",
  level: "mietthi",
  side: "out",
  matched: "khỉ Việt Nam",
  count: 1,
  fingerprint: "x",
  status: "pending",
  ai: null,
  raw: ["前。", "什么东瀛，高丽，越南猴子。", "后。"],
  out: ["Trước.", "Nào Đông Doanh này, Cao Ly này, khỉ Việt Nam này.", "Sau."],
  focus: 1,
  ...over,
});

const giaoChi = (over: Partial<VnHitView> = {}) =>
  hit({ id: "0354:3:giao-chi", chapter: "0354", entry: "giao-chi", level: "canxem", matched: "Giao Chỉ", out: ["", "phía nam đánh chiếm Lữ Tống và Giao Chỉ", ""], ...over });

// JSX không xử lý escape trong thuộc tính chuỗi, nên truyền root qua biểu thức.
const ROOT = "D:\\t";
const renderPanel = (running = false) => render(<VnCheckPanel root={ROOT} running={running} setting="modern" />);

describe("VnCheckPanel", () => {
  beforeEach(() => {
    vi.mocked(toast.error).mockReset();
    vi.mocked(toast.success).mockReset();
    vi.mocked(vnRefresh).mockReset().mockResolvedValue({ hits: 2, open: 2, scanned: 0 });
    vi.mocked(vnReport).mockReset().mockResolvedValue([hit({}), giaoChi({ status: "ok" })]);
    vi.mocked(vnSetStatus).mockReset().mockResolvedValue(1);
    vi.mocked(vnAiVerify).mockReset().mockResolvedValue({ verified: 1, failed: 0 });
  });

  it("quét khi mở, mặc định ẩn chỗ đã 'Không sao', tô chữ khớp, hiện mức miệt thị", async () => {
    renderPanel();
    expect(vnRefresh).toHaveBeenCalledWith("D:\\t");
    const item = (await screen.findByText("0226")).closest("li")!;
    expect(within(item).getByText("khỉ Việt Nam", { selector: "mark" })).toBeInTheDocument();
    expect(within(item).getByText("Miệt thị")).toBeInTheDocument();
    expect(screen.queryByText("0354")).not.toBeInTheDocument();
    await userEvent.setup().selectOptions(screen.getByLabelText("Trạng thái"), "all");
    expect(screen.getByText("0354")).toBeInTheDocument();
  });

  it("bấm Không sao / Cần xử lý gọi đặt trạng thái rồi tải lại", async () => {
    const user = userEvent.setup();
    renderPanel();
    const item = (await screen.findByText("0226")).closest("li")!;
    await user.click(within(item).getByRole("button", { name: "Không sao 0226:4:khi-viet" }));
    expect(vnSetStatus).toHaveBeenCalledWith("D:\\t", ["0226:4:khi-viet"], "ok");
    await user.click(within(item).getByRole("button", { name: "Cần xử lý 0226:4:khi-viet" }));
    expect(vnSetStatus).toHaveBeenCalledWith("D:\\t", ["0226:4:khi-viet"], "action");
    expect(vnReport).toHaveBeenCalledTimes(3);
  });

  it("chọn chỗ rồi AI xét; hiện nhãn và lý do AI", async () => {
    const user = userEvent.setup();
    vi.mocked(vnReport).mockResolvedValueOnce([hit({})]).mockResolvedValueOnce([
      hit({ ai: { label: "xucpham", reason: "Gọi người Việt là khỉ.", model: "Fake", at: 1 } }),
    ]);
    renderPanel();
    const item = (await screen.findByText("0226")).closest("li")!;
    await user.click(within(item).getByRole("checkbox", { name: "Chọn 0226:4:khi-viet" }));
    await user.click(screen.getByRole("button", { name: /AI xét 1 chỗ/ }));
    expect(vnAiVerify).toHaveBeenCalledWith("D:\\t", ["0226:4:khi-viet"]);
    expect(await screen.findByText("Gọi người Việt là khỉ.")).toBeInTheDocument();
    expect(screen.getByText("AI: Xúc phạm")).toBeInTheDocument();
  });

  it("AI xét báo lỗi model: hiện toast lỗi kèm lý do và vẫn tải lại", async () => {
    const user = userEvent.setup();
    vi.mocked(vnAiVerify).mockResolvedValue({ verified: 0, failed: 1, error: "Sai API key" });
    renderPanel();
    const item = (await screen.findByText("0226")).closest("li")!;
    await user.click(within(item).getByRole("checkbox", { name: "Chọn 0226:4:khi-viet" }));
    await user.click(screen.getByRole("button", { name: /AI xét 1 chỗ/ }));
    expect(toast.error).toHaveBeenCalledWith(expect.stringContaining("Sai API key"));
    expect(toast.success).not.toHaveBeenCalled();
    expect(vnReport).toHaveBeenCalledTimes(2);
  });

  it("lọc theo mức, nhãn AI; chọn tất cả đang lọc", async () => {
    const user = userEvent.setup();
    vi.mocked(vnReport).mockResolvedValue([hit({}), giaoChi({ ai: { label: "trungtinh", reason: "r", model: "m", at: 1 } })]);
    renderPanel();
    await screen.findByText("0226");
    await user.selectOptions(screen.getByLabelText("Mức"), "canxem");
    expect(screen.queryByText("0226")).not.toBeInTheDocument();
    expect(screen.getByText("0354")).toBeInTheDocument();
    await user.selectOptions(screen.getByLabelText("Mức"), "all");
    await user.selectOptions(screen.getByLabelText("Nhãn AI"), "none");
    expect(screen.getByText("0226")).toBeInTheDocument();
    expect(screen.queryByText("0354")).not.toBeInTheDocument();
    await user.selectOptions(screen.getByLabelText("Nhãn AI"), "all");
    await user.click(screen.getByRole("checkbox", { name: "Chọn tất cả đang lọc" }));
    expect(screen.getByRole("button", { name: /AI xét 2 chỗ/ })).toBeEnabled();
  });

  it("xếp theo mục từ điển: chỗ cùng mục đứng liền, có đầu nhóm", async () => {
    const user = userEvent.setup();
    vi.mocked(vnReport).mockResolvedValue([hit({}), giaoChi(), hit({ id: "0400:1:khi-viet", chapter: "0400", paragraph: 1 })]);
    renderPanel();
    await screen.findByText("0226");
    await user.selectOptions(screen.getByLabelText("Xếp theo"), "entry");
    const chapters = screen.getAllByTestId("vn-hit-chapter").map((node) => node.textContent);
    expect(chapters).toEqual(["0354", "0226", "0400"]);
    expect(screen.getAllByTestId("vn-entry-header").map((node) => node.textContent)).toEqual(["giao-chi", "khi-viet"]);
  });

  it("Mở chương chuyển sang trang dịch, chọn đúng chương", async () => {
    const user = userEvent.setup();
    const select = vi.fn();
    const setPage = vi.fn();
    const original = { select: useStoryStore.getState().select, setPage: useStoryStore.getState().setPage };
    useStoryStore.setState({ select, setPage });
    onTestFinished(() => useStoryStore.setState(original));
    renderPanel();
    const item = (await screen.findByText("0226")).closest("li")!;
    await user.click(within(item).getByRole("button", { name: "Mở chương 0226" }));
    expect(select).toHaveBeenCalledWith("0226");
    expect(setPage).toHaveBeenCalledWith("translate");
  });

  it("chỗ đã chọn rồi bị ẩn bởi bộ lọc thì không còn tính vào 'AI xét'", async () => {
    const user = userEvent.setup();
    vi.mocked(vnReport).mockReset().mockResolvedValueOnce([hit({})]).mockResolvedValue([hit({ status: "ok" })]);
    renderPanel();
    const item = (await screen.findByText("0226")).closest("li")!;
    await user.click(within(item).getByRole("checkbox", { name: "Chọn 0226:4:khi-viet" }));
    await user.click(within(item).getByRole("button", { name: "Không sao 0226:4:khi-viet" }));
    await waitFor(() => expect(screen.queryByText("0226")).not.toBeInTheDocument());
    expect(screen.getByRole("button", { name: /AI xét 0 chỗ/ })).toBeDisabled();
    expect(screen.getByRole("button", { name: /Thay thế/ })).toBeDisabled();
  });

  it("chỗ đã chọn biến mất sau khi quét lại thì bị bỏ chọn, không quay lại khi nó xuất hiện lại", async () => {
    const user = userEvent.setup();
    vi.mocked(vnReport).mockReset().mockResolvedValueOnce([hit({})]).mockResolvedValueOnce([giaoChi()]).mockResolvedValue([hit({}), giaoChi()]);
    renderPanel();
    const item = (await screen.findByText("0226")).closest("li")!;
    await user.click(within(item).getByRole("checkbox", { name: "Chọn 0226:4:khi-viet" }));
    await user.click(screen.getByRole("button", { name: /Quét lại/ }));
    await waitFor(() => expect(screen.queryByText("0226")).not.toBeInTheDocument());
    expect(screen.getByRole("button", { name: /AI xét 0 chỗ/ })).toBeDisabled();
    await user.click(screen.getByRole("button", { name: /Quét lại/ }));
    expect(await screen.findByText("0226")).toBeInTheDocument();
    expect(screen.getByRole("checkbox", { name: "Chọn 0226:4:khi-viet" })).not.toBeChecked();
  });

  it("đặt trạng thái lỗi thì báo toast, không văng lỗi chưa bắt", async () => {
    const user = userEvent.setup();
    vi.mocked(vnSetStatus).mockRejectedValue(new Error("Đĩa đầy"));
    renderPanel();
    const item = (await screen.findByText("0226")).closest("li")!;
    await user.click(within(item).getByRole("button", { name: "Không sao 0226:4:khi-viet" }));
    expect(toast.error).toHaveBeenCalledWith("Đĩa đầy");
  });

  it("Quét lại lỗi thì báo toast", async () => {
    const user = userEvent.setup();
    renderPanel();
    await screen.findByText("0226");
    vi.mocked(vnRefresh).mockRejectedValueOnce(new Error("Không đọc được"));
    await user.click(screen.getByRole("button", { name: /Quét lại/ }));
    expect(toast.error).toHaveBeenCalledWith("Không đọc được");
    expect(screen.getByRole("button", { name: /Quét lại/ })).toBeEnabled();
  });

  it("quét lần đầu lỗi: hiện lỗi và nút Thử lại, bấm thì quét lại và hiện danh sách", async () => {
    const user = userEvent.setup();
    vi.mocked(vnRefresh).mockRejectedValueOnce(new Error("Không quét được kho"));
    renderPanel();
    expect(await screen.findByText(/Không quét được kho/)).toBeInTheDocument();
    expect(screen.queryByText("Đang quét…")).not.toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Thử lại" }));
    expect(await screen.findByText("0226")).toBeInTheDocument();
    expect(vnRefresh).toHaveBeenCalledTimes(2);
  });

  it("đang dịch thì không cho Thay thế", async () => {
    const user = userEvent.setup();
    renderPanel(true);
    const item = (await screen.findByText("0226")).closest("li")!;
    await user.click(within(item).getByRole("checkbox", { name: "Chọn 0226:4:khi-viet" }));
    expect(screen.getByRole("button", { name: /Thay thế/ })).toBeDisabled();
    expect(screen.getByRole("button", { name: /AI xét 1 chỗ/ })).toBeEnabled();
  });

  it("nút Không sao / Cần xử lý có aria-pressed theo trạng thái", async () => {
    vi.mocked(vnReport).mockResolvedValue([hit({ status: "action" })]);
    renderPanel();
    await screen.findByText("0226");
    expect(screen.getByRole("button", { name: "Cần xử lý 0226:4:khi-viet" })).toHaveAttribute("aria-pressed", "true");
    expect(screen.getByRole("button", { name: "Không sao 0226:4:khi-viet" })).toHaveAttribute("aria-pressed", "false");
  });
});
