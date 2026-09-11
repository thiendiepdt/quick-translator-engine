import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

import { AiStoryConfigDialog } from "@/components/ai-story-config-dialog";
import { defaultAiSettings } from "@/lib/ai-settings";
import { emptyAiStoryConfig } from "@/lib/ai-story";

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

function renderDialog(onSave = vi.fn()) {
  const story = { ...emptyAiStoryConfig(), name: "Gốc" };
  render(
    <AiStoryConfigDialog
      open
      onOpenChange={vi.fn()}
      story={story}
      aiSettings={defaultAiSettings}
      onSave={onSave}
    />,
  );
  return onSave;
}

describe("story genre", () => {
  it("lưu bối cảnh và tên riêng đã chọn", async () => {
    const onSave = renderDialog();
    const user = userEvent.setup();

    await user.click(screen.getByRole("combobox", { name: "Bối cảnh" }));
    await user.click(await screen.findByRole("option", { name: /Hiện đại/ }));
    await user.click(screen.getByRole("combobox", { name: "Tên riêng" }));
    await user.click(await screen.findByRole("option", { name: /Gốc nước ngoài/ }));
    await user.click(screen.getByRole("button", { name: "Lưu cấu hình" }));

    expect(onSave).toHaveBeenCalledWith(
      expect.objectContaining({ genre: { setting: "modern", names: "foreign" } }),
    );
  });

  it("tab Kiểm tra hiện bộ rule theo bối cảnh khi chưa có rule riêng", async () => {
    renderDialog();
    const user = userEvent.setup();
    const hasSpouseRule = () =>
      screen
        .getAllByLabelText(/^Mô tả rule/)
        .some((el) => (el as HTMLInputElement).value.includes("thê tử/phu quân"));

    await user.click(screen.getByRole("tab", { name: /Kiểm tra/ }));
    expect(hasSpouseRule()).toBe(true);

    await user.click(screen.getByRole("tab", { name: /Thông tin/ }));
    await user.click(screen.getByRole("combobox", { name: "Bối cảnh" }));
    await user.click(await screen.findByRole("option", { name: /Hiện đại/ }));
    await user.click(screen.getByRole("tab", { name: /Kiểm tra/ }));
    expect(hasSpouseRule()).toBe(false);
  });

  it("Hỗn hợp: tab Kiểm tra không có rule xưng hô của cả hai bối cảnh", async () => {
    renderDialog();
    const user = userEvent.setup();
    await user.click(screen.getByRole("combobox", { name: "Bối cảnh" }));
    await user.click(await screen.findByRole("option", { name: /Hỗn hợp/ }));
    await user.click(screen.getByRole("tab", { name: /Kiểm tra/ }));
    const messages = screen.getAllByLabelText(/^Mô tả rule/).map((el) => (el as HTMLInputElement).value);
    expect(messages.some((m) => m.includes("thê tử/phu quân"))).toBe(false);
    expect(messages.some((m) => m.includes("Xưng hô cổ trang"))).toBe(false);
  });
});

describe("story config export/import", () => {
  it("imports a JSON file into the draft and saves it", async () => {
    const onSave = renderDialog();
    const user = userEvent.setup();

    const payload = JSON.stringify({ name: "Đấu Phá", autoGlossary: "off" });
    await user.upload(
      screen.getByLabelText("Chọn file cấu hình truyện"),
      new File([payload], "dau-pha.json", { type: "application/json" }),
    );

    await waitFor(() => {
      expect(screen.getByLabelText("Tên truyện")).toHaveValue("Đấu Phá");
    });
    await user.click(screen.getByRole("button", { name: "Lưu cấu hình" }));
    expect(onSave).toHaveBeenCalledWith(
      expect.objectContaining({ name: "Đấu Phá", autoGlossary: "off" }),
    );
  });

  it("keeps the draft when the file is not valid JSON", async () => {
    renderDialog();
    const user = userEvent.setup();

    await user.upload(
      screen.getByLabelText("Chọn file cấu hình truyện"),
      new File(["{hỏng"], "rac.json", { type: "application/json" }),
    );

    await waitFor(() => {
      expect(screen.getByLabelText("Tên truyện")).toHaveValue("Gốc");
    });
  });

  it("exports the current draft as pretty JSON", async () => {
    const blobs: Blob[] = [];
    vi.stubGlobal(
      "URL",
      Object.assign(URL, {
        createObjectURL: vi.fn((blob: Blob) => {
          blobs.push(blob);
          return "blob:test";
        }),
        revokeObjectURL: vi.fn(),
      }),
    );
    const clickSpy = vi
      .spyOn(HTMLAnchorElement.prototype, "click")
      .mockImplementation(() => undefined);
    renderDialog();
    const user = userEvent.setup();

    await user.click(screen.getByRole("button", { name: /Xuất JSON/ }));

    expect(clickSpy).toHaveBeenCalled();
    const exported = JSON.parse(await blobs[0].text()) as { name: string };
    expect(exported.name).toBe("Gốc");
    clickSpy.mockRestore();
  });
});
