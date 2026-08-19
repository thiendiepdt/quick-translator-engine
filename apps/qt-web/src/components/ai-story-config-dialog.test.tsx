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
