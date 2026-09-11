import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { CreateStoryDialog } from "@/components/create-story-dialog";
import { createStory, slugifyName } from "@/lib/api";

vi.mock("@/lib/api", () => ({
  createStory: vi.fn(),
  slugifyName: vi.fn((name: string) =>
    Promise.resolve(
      name
        .toLowerCase()
        .replace(/[^a-z0-9]+/g, "-")
        .replace(/^-|-$/g, ""),
    ),
  ),
}));

describe("CreateStoryDialog", () => {
  beforeEach(() => {
    vi.mocked(createStory).mockReset();
    vi.mocked(slugifyName).mockClear();
  });

  it("slug tự sinh từ tên qua Rust, ngừng tự sinh khi sửa tay, submit gọi create_story đúng tham số", async () => {
    const user = userEvent.setup();
    const onCreated = vi.fn();
    const snapshot = { root: "D:\\lib\\ky-chieu" } as never;
    vi.mocked(createStory).mockResolvedValue(snapshot);
    render(<CreateStoryDialog libraryRoot={"D:\\lib"} open onOpenChange={() => undefined} onCreated={onCreated} />);

    expect(screen.getByRole("button", { name: "Tạo và mở" })).toBeDisabled();
    await user.type(screen.getByLabelText("Tên truyện tiếng Việt"), "Ky Chieu");
    await waitFor(() => expect(screen.getByLabelText("Tên folder")).toHaveValue("ky-chieu"));
    expect(screen.getByText("D:\\lib\\ky-chieu")).toBeInTheDocument();

    await user.clear(screen.getByLabelText("Tên folder"));
    await user.type(screen.getByLabelText("Tên folder"), "kcn");
    await user.type(screen.getByLabelText("Tên truyện tiếng Việt"), " 2");
    await waitFor(() => expect(slugifyName).toHaveBeenCalled());
    expect(screen.getByLabelText("Tên folder")).toHaveValue("kcn"); // đã sửa tay thì không bị đè
    await user.type(screen.getByLabelText("Link truyện tiếng Trung (tuỳ chọn)"), "https://x/y");

    await user.click(screen.getByRole("button", { name: "Tạo và mở" }));
    await waitFor(() => expect(createStory).toHaveBeenCalledWith("Ky Chieu 2", "kcn", "https://x/y"));
    expect(onCreated).toHaveBeenCalledWith(snapshot);
  });

  it("slug sai định dạng thì báo và khoá nút", async () => {
    const user = userEvent.setup();
    render(<CreateStoryDialog libraryRoot="/lib" open onOpenChange={() => undefined} onCreated={() => undefined} />);
    await user.type(screen.getByLabelText("Tên truyện tiếng Việt"), "A");
    await user.clear(screen.getByLabelText("Tên folder"));
    await user.type(screen.getByLabelText("Tên folder"), "Sai Slug");
    expect(screen.getByText("Chỉ dùng a-z, 0-9 và dấu gạch nối.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Tạo và mở" })).toBeDisabled();
  });
});
