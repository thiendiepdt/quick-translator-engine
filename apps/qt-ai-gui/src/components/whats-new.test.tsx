import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { ChangelogList, InlineText, WhatsNewDialog } from "@/components/whats-new";
import { parseChangelog } from "@/lib/changelog";

const ENTRIES = parseChangelog(
  [
    "# Có gì mới",
    "## Chưa phát hành",
    "- Chưa ra.",
    ...Array.from({ length: 8 }, (_, i) => [`## 1.0.${8 - i} — 2026-09-${String(10 + i).padStart(2, "0")}`, `- Ghi chú ${8 - i}.`]).flat(),
  ].join("\n"),
);

describe("InlineText", () => {
  it("**đậm** thành <strong>, `code` thành <code>, chữ giống HTML vẫn là chữ", () => {
    const { container } = render(<InlineText text={"Thêm **Tìm & thay** (`Ctrl+F`) <img src=x onerror=alert(1)>"} />);
    expect(container.querySelector("strong")).toHaveTextContent("Tìm & thay");
    expect(container.querySelector("code")).toHaveTextContent("Ctrl+F");
    expect(container.querySelector("img")).toBeNull();
    expect(container).toHaveTextContent("Thêm Tìm & thay (Ctrl+F) <img src=x onerror=alert(1)>");
  });
});

describe("ChangelogList", () => {
  it("bản đang dùng mở sẵn, bản cũ thu gọn; chỉ 5 bản đầu cho tới khi bấm Xem tất cả; bỏ bản mới hơn và Chưa phát hành", async () => {
    const user = userEvent.setup();
    render(<ChangelogList currentVersion="1.0.7" entries={ENTRIES} />);
    const toggles = screen.getAllByRole("button", { name: /^Bản 1\.0\./ });
    expect(toggles.map((b) => b.textContent)).toEqual([
      expect.stringContaining("1.0.7"),
      expect.stringContaining("1.0.6"),
      expect.stringContaining("1.0.5"),
      expect.stringContaining("1.0.4"),
      expect.stringContaining("1.0.3"),
    ]);
    expect(toggles[0]).toHaveAttribute("aria-expanded", "true");
    expect(toggles[0]).toHaveTextContent("Đang dùng");
    expect(screen.getByText("Ghi chú 7.")).toBeInTheDocument();
    expect(toggles[1]).toHaveAttribute("aria-expanded", "false");
    expect(screen.queryByText("Ghi chú 6.")).not.toBeInTheDocument();
    expect(screen.queryByText("Chưa ra.")).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /Bản 1\.0\.8/ })).not.toBeInTheDocument();

    await user.click(toggles[1]);
    expect(toggles[1]).toHaveAttribute("aria-expanded", "true");
    expect(screen.getByText("Ghi chú 6.")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Xem tất cả (7 bản)" }));
    expect(screen.getAllByRole("button", { name: /^Bản 1\.0\./ })).toHaveLength(7);
    await user.click(screen.getByRole("button", { name: "Thu gọn" }));
    expect(screen.getAllByRole("button", { name: /^Bản 1\.0\./ })).toHaveLength(5);
  });

  it("ít hơn 6 bản thì không có nút Xem tất cả", () => {
    render(<ChangelogList currentVersion="1.0.3" entries={ENTRIES} />);
    expect(screen.getAllByRole("button", { name: /^Bản 1\.0\./ })).toHaveLength(3);
    expect(screen.queryByRole("button", { name: /Xem tất cả/ })).not.toBeInTheDocument();
  });
});

describe("WhatsNewDialog", () => {
  it("hiện bullet của bản vừa cập nhật; Đã hiểu đóng; Xem tất cả gọi onShowAll", async () => {
    const user = userEvent.setup();
    const onClose = vi.fn();
    const onShowAll = vi.fn();
    render(<WhatsNewDialog version="1.0.6" entries={ENTRIES} onClose={onClose} onShowAll={onShowAll} />);
    const dialog = screen.getByRole("dialog", { name: "Có gì mới ở bản 1.0.6" });
    expect(within(dialog).getByText("Ghi chú 6.")).toBeInTheDocument();
    expect(within(dialog).queryByText("Ghi chú 5.")).not.toBeInTheDocument();

    await user.click(within(dialog).getByRole("button", { name: "Xem tất cả" }));
    expect(onShowAll).toHaveBeenCalledTimes(1);
    await user.click(within(dialog).getByRole("button", { name: "Đã hiểu" }));
    expect(onClose).toHaveBeenCalled();
  });

  it("không có onShowAll (đang ở màn chọn truyện, chưa vào được Cài đặt): Xem tất cả mở danh sách ngay trong hộp thoại", async () => {
    const user = userEvent.setup();
    render(<WhatsNewDialog version="1.0.6" entries={ENTRIES} onClose={vi.fn()} />);
    const dialog = screen.getByRole("dialog");
    await user.click(within(dialog).getByRole("button", { name: "Xem tất cả" }));
    // Mở thẳng đủ mọi bản tới bản đang dùng, không bắt bấm "Xem tất cả" lần hai.
    expect(within(dialog).getAllByRole("button", { name: /^Bản 1\.0\./ })).toHaveLength(6);
    expect(within(dialog).getByRole("button", { name: "Thu gọn" })).toBeInTheDocument();
    expect(within(dialog).queryByRole("button", { name: /Xem tất cả/ })).not.toBeInTheDocument();
  });

  it("bỏ qua vài bản: hiện mọi bản sau bản đã thấy tới bản đang dùng, mới nhất trước, đều mở sẵn", () => {
    render(<WhatsNewDialog version="1.0.6" since="1.0.4" entries={ENTRIES} onClose={vi.fn()} />);
    const dialog = screen.getByRole("dialog", { name: "Có gì mới ở bản 1.0.6" });
    const headings = within(dialog).getAllByRole("heading", { level: 3 });
    expect(headings.map((h) => h.textContent)).toEqual([expect.stringContaining("Bản 1.0.6"), expect.stringContaining("Bản 1.0.5")]);
    expect(within(dialog).getByText("Ghi chú 6.")).toBeInTheDocument();
    expect(within(dialog).getByText("Ghi chú 5.")).toBeInTheDocument();
    expect(within(dialog).queryByText("Ghi chú 4.")).not.toBeInTheDocument();
    expect(within(dialog).getByText(/kể từ bản 1\.0\.4/)).toBeInTheDocument();
  });

  it("quá 5 bản thì chỉ hiện 5 bản mới nhất, phần còn lại qua Xem tất cả", () => {
    render(<WhatsNewDialog version="1.0.8" since="1.0.0" entries={ENTRIES} onClose={vi.fn()} />);
    const dialog = screen.getByRole("dialog");
    expect(within(dialog).getAllByRole("heading", { level: 3 })).toHaveLength(5);
    expect(within(dialog).getByText("Ghi chú 4.")).toBeInTheDocument();
    expect(within(dialog).queryByText("Ghi chú 3.")).not.toBeInTheDocument();
    expect(within(dialog).getByRole("button", { name: "Xem tất cả" })).toBeInTheDocument();
  });

  it("không có version thì không hiện gì", () => {
    render(<WhatsNewDialog version={undefined} entries={ENTRIES} onClose={vi.fn()} />);
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });
});
