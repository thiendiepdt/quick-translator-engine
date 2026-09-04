import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import { FormProvider, useForm } from "react-hook-form";

import { GlossaryEditor } from "@/components/glossary-editor";
import type { Pair, StoryFormValues } from "@/lib/story-form";

let latest: StoryFormValues | undefined;

function Harness({ names }: { names: Pair[] }) {
  const form = useForm<StoryFormValues>({
    defaultValues: {
      glossary: { names, places: [], items: [], creatures: [], skills: [], common: [], signature_phrases: [] },
      signaturePhrases: [],
      checkRules: [],
    } as unknown as StoryFormValues,
  });
  return (
    <FormProvider {...form}>
      <GlossaryEditor name="glossary.names" label="Tên nhân vật" />
      <button type="button" onClick={() => (latest = form.getValues())}>
        đọc
      </button>
    </FormProvider>
  );
}

const many = (count: number): Pair[] => Array.from({ length: count }, (_, i) => ({ source: `名${i}`, target: `Tên ${i}` }));

describe("GlossaryEditor", () => {
  it("nhóm dài thu gọn sẵn, mở ra chỉ hiện 50 dòng rồi 'Hiện thêm'", async () => {
    const user = userEvent.setup();
    render(<Harness names={many(120)} />);
    expect(screen.queryByLabelText("Tên nhân vật CN 1")).not.toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: /Tên nhân vật/ }));
    expect(screen.getByLabelText("Tên nhân vật CN 50")).toBeInTheDocument();
    expect(screen.queryByLabelText("Tên nhân vật CN 51")).not.toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: /Hiện thêm 50/ }));
    expect(screen.getByLabelText("Tên nhân vật CN 100")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Hiện thêm 20/ })).toBeInTheDocument();
  });

  it("sửa dạng văn bản: Áp dụng thay cả bảng, Huỷ giữ nguyên", async () => {
    const user = userEvent.setup();
    render(<Harness names={[{ source: "赵静文", target: "Triệu Tĩnh Văn" }]} />);
    await user.click(screen.getByRole("button", { name: /Sửa dạng văn bản/ }));
    const area = screen.getByLabelText("Tên nhân vật dạng văn bản");
    expect(area).toHaveValue("赵静文=Triệu Tĩnh Văn");
    await user.clear(area);
    await user.type(area, "高塔=Cao Tháp{enter}慕容=Mộ Dung");
    await user.click(screen.getByRole("button", { name: "Áp dụng" }));
    expect(screen.getByLabelText("Tên nhân vật CN 1")).toHaveValue("高塔");
    expect(screen.getByLabelText("Tên nhân vật VN 2")).toHaveValue("Mộ Dung");
    await user.click(screen.getByRole("button", { name: "đọc" }));
    expect(latest?.glossary.names).toEqual([
      { source: "高塔", target: "Cao Tháp" },
      { source: "慕容", target: "Mộ Dung" },
    ]);

    await user.click(screen.getByRole("button", { name: /Sửa dạng văn bản/ }));
    await user.type(screen.getByLabelText("Tên nhân vật dạng văn bản"), "{enter}x=y");
    await user.click(screen.getByRole("button", { name: "Huỷ" }));
    expect(screen.queryByLabelText("Tên nhân vật CN 3")).not.toBeInTheDocument();
  });
});
