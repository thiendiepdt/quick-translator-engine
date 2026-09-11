import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { RuleTable } from "@/components/rule-table";

describe("RuleTable", () => {
  it("sửa ô, thêm và xoá dòng gọi onChange với mảng mới", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(<RuleTable rows={[{ pattern: "a", flags: "i", message: "m" }]} onChange={onChange} />);
    await user.type(screen.getByLabelText("Mô tả rule 1"), "!");
    expect(onChange).toHaveBeenLastCalledWith([{ pattern: "a", flags: "i", message: "m!" }]);
    await user.click(screen.getByRole("button", { name: "Thêm" }));
    expect(onChange).toHaveBeenLastCalledWith([
      { pattern: "a", flags: "i", message: "m" },
      { pattern: "", flags: "", message: "" },
    ]);
    await user.click(screen.getByRole("button", { name: "Xoá rule" }));
    expect(onChange).toHaveBeenLastCalledWith([]);
  });

  it("readOnly hiện chữ, không có input", () => {
    render(<RuleTable rows={[{ pattern: "a", flags: "", message: "m" }]} onChange={() => undefined} readOnly />);
    expect(screen.queryByRole("textbox")).not.toBeInTheDocument();
    expect(screen.getByText("m")).toBeInTheDocument();
  });
});
