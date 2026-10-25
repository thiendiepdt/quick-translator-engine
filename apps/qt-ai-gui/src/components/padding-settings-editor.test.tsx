import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { PaddingSettingsEditor } from "@/components/padding-settings-editor";
import { paddingGet, paddingReset, paddingSave } from "@/lib/api";

vi.mock("@/lib/api", () => ({ paddingGet: vi.fn(), paddingSave: vi.fn(), paddingReset: vi.fn() }));
vi.mock("sonner", () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

const saved = { mode: "off" as const, threshold: 500, filler: "Đoạn một.\n\nKết." };

describe("PaddingSettingsEditor", () => {
  beforeEach(() => {
    vi.mocked(paddingGet).mockReset().mockResolvedValue(saved);
    vi.mocked(paddingSave).mockReset().mockImplementation((settings) => Promise.resolve(settings));
    vi.mocked(paddingReset).mockReset().mockResolvedValue({ ...saved, filler: "Văn mẫu gốc." });
  });

  it("mặc định tắt; sửa ngưỡng và văn mẫu rồi Lưu", async () => {
    const user = userEvent.setup();
    render(<PaddingSettingsEditor />);
    const threshold = await screen.findByLabelText("Ngưỡng (chữ)");
    expect(screen.getByRole("combobox", { name: "Khi chốt chương ngắn" })).toHaveTextContent("Tắt — chỉ cảnh báo");
    const save = screen.getByRole("button", { name: /Lưu/ });
    expect(save).toBeDisabled();
    await user.clear(threshold);
    expect(screen.getByText("Ngưỡng phải là số nguyên lớn hơn 0")).toBeInTheDocument();
    await user.type(threshold, "600");
    await user.type(screen.getByLabelText("Văn mẫu"), " Thêm.");
    await user.click(save);
    expect(paddingSave).toHaveBeenCalledWith({ mode: "off", threshold: 600, filler: "Đoạn một.\n\nKết. Thêm." });
    expect(save).toBeDisabled();
  });

  it("Về mặc định hỏi trước rồi nạp lại văn mẫu gốc", async () => {
    const user = userEvent.setup();
    vi.spyOn(window, "confirm").mockReturnValueOnce(true);
    render(<PaddingSettingsEditor />);
    await screen.findByLabelText("Văn mẫu");
    await user.click(screen.getByRole("button", { name: /Về mặc định/ }));
    expect(paddingReset).toHaveBeenCalled();
    expect(await screen.findByDisplayValue("Văn mẫu gốc.")).toBeInTheDocument();
  });
});
