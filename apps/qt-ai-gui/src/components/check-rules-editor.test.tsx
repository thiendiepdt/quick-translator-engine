import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import { FormProvider, useForm } from "react-hook-form";

import { CheckRulesEditor } from "@/components/check-rules-editor";
import type { StoryFormValues } from "@/lib/story-form";
import type { StoryDefaults } from "@/lib/types";

const defaults: StoryDefaults = {
  basePrompt: "GỐC",
  promptSuffix: "ĐUÔI",
  checkRules: [
    { pattern: "vợ|chồng", flags: "i", message: "thê tử/phu quân" },
    { pattern: "anh ấy", message: "hắn" },
  ],
};

let latest: StoryFormValues | undefined;

function Harness({ rules, defs }: { rules: StoryFormValues["checkRules"]; defs?: StoryDefaults }) {
  const form = useForm<StoryFormValues>({ defaultValues: { checkRules: rules } as unknown as StoryFormValues });
  return (
    <FormProvider {...form}>
      <CheckRulesEditor defaults={defs} />
      <button type="button" onClick={() => (latest = form.getValues())}>
        đọc
      </button>
    </FormProvider>
  );
}

describe("CheckRulesEditor", () => {
  it("trống thì hiện bộ mặc định chỉ đọc; 'Sửa bộ mặc định' sao chép ra ô sửa; 'Về mặc định' xoá bản riêng", async () => {
    const user = userEvent.setup();
    render(<Harness rules={[]} defs={defaults} />);
    expect(screen.getByText("mặc định · 2")).toBeInTheDocument();
    expect(screen.getByText("vợ|chồng")).toBeInTheDocument();
    expect(screen.queryByLabelText("Regex 1")).not.toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: /Sửa bộ mặc định/ }));
    expect(screen.getByLabelText("Regex 1")).toHaveValue("vợ|chồng");
    expect(screen.getByLabelText("Flags 1")).toHaveValue("i");
    expect(screen.getByLabelText("Flags 2")).toHaveValue("");
    expect(screen.getByText("riêng · 2")).toBeInTheDocument();
    await user.type(screen.getByLabelText("Mô tả rule 2"), " hoặc nàng");
    await user.click(screen.getByRole("button", { name: "đọc" }));
    expect(latest?.checkRules[1]?.message).toBe("hắn hoặc nàng");

    await user.click(screen.getByRole("button", { name: "Về mặc định" }));
    expect(screen.getByText("mặc định · 2")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "đọc" }));
    expect(latest?.checkRules).toEqual([]);
  });

  it("'Thêm' khi đang mặc định thì sao chép bộ mặc định rồi thêm dòng trống; chưa tải defaults thì nút Sửa bị khoá", async () => {
    const user = userEvent.setup();
    const { unmount } = render(<Harness rules={[]} defs={defaults} />);
    await user.click(screen.getByRole("button", { name: /Thêm/ }));
    expect(screen.getByLabelText("Regex 3")).toHaveValue("");
    expect(screen.getByLabelText("Regex 1")).toHaveValue("vợ|chồng");
    unmount();
    render(<Harness rules={[]} />);
    expect(screen.getByRole("button", { name: /Sửa bộ mặc định/ })).toBeDisabled();
  });
});
