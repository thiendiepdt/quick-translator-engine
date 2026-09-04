import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import { FormProvider, useForm } from "react-hook-form";

import { PromptEditor } from "@/components/prompt-editor";
import type { StoryFormValues } from "@/lib/story-form";
import type { StoryDefaults } from "@/lib/types";

const defaults: StoryDefaults = { basePrompt: "# Prompt gốc\n\nNội dung **gốc**.", promptSuffix: "Đuôi cố định.", checkRules: [] };
let latest: StoryFormValues | undefined;

function Harness({ custom, defs }: { custom: string; defs?: StoryDefaults }) {
  const form = useForm<StoryFormValues>({ defaultValues: { customPrompt: custom } as unknown as StoryFormValues });
  return (
    <FormProvider {...form}>
      <PromptEditor defaults={defs} />
      <button type="button" onClick={() => (latest = form.getValues())}>
        đọc
      </button>
    </FormProvider>
  );
}

describe("PromptEditor", () => {
  it("trống thì nạp prompt gốc vào editor markdown; 'Về mặc định' bị khoá", async () => {
    render(<Harness custom="" defs={defaults} />);
    const editor = await screen.findByRole("textbox", { name: "Prompt dịch thuật" }, { timeout: 5000 });
    expect(editor).toHaveAttribute("contenteditable", "true");
    expect(screen.getByRole("heading", { name: "Prompt gốc" })).toBeInTheDocument();
    expect(screen.getByText("gốc").closest("strong")).not.toBeNull();
    expect(screen.getByText("mặc định")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Về mặc định/ })).toBeDisabled();
    expect(screen.getByText("Đuôi cố định.")).toBeInTheDocument();
  });

  it("prompt riêng hiện nguyên; 'Về mặc định' lưu trống và nạp lại bản gốc", async () => {
    const user = userEvent.setup();
    render(<Harness custom="Prompt **của tôi**" defs={defaults} />);
    await screen.findByRole("textbox", { name: "Prompt dịch thuật" }, { timeout: 5000 });
    expect(screen.getByText("của tôi").closest("strong")).not.toBeNull();
    expect(screen.getByText("riêng")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: /Về mặc định/ }));
    await waitFor(() => expect(screen.getByRole("heading", { name: "Prompt gốc" })).toBeInTheDocument());
    expect(screen.getByText("mặc định")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "đọc" }));
    expect(latest?.customPrompt).toBe("");
  });

  it("chưa tải defaults thì hiện trạng thái chờ, không dựng editor", () => {
    render(<Harness custom="" />);
    expect(screen.getByRole("status")).toHaveTextContent("Đang tải prompt mặc định");
    expect(screen.queryByRole("textbox")).not.toBeInTheDocument();
  });
});
