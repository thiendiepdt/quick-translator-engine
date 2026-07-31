import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it } from "vitest";
import { FormProvider, useForm } from "react-hook-form";

import { EngineOptions } from "@/components/engine-options";
import type { TranslationOptionsValues } from "@/lib/schema";

function EngineOptionsHarness() {
  const form = useForm<TranslationOptionsValues>({
    defaultValues: {
      endpoint: "/api",
      pretty: true,
      wrap: false,
      prioritizedName: true,
      scanRange: 30,
      translationAlgorithm: 1,
    },
  });
  return (
    <FormProvider {...form}>
      <EngineOptions />
    </FormProvider>
  );
}

afterEach(cleanup);

describe("tùy chọn thuật toán dịch", () => {
  it("hiển thị nhãn dễ hiểu nhưng vẫn chọn đúng giá trị thuật toán", async () => {
    const user = userEvent.setup();
    render(<EngineOptionsHarness />);

    const select = screen.getByRole("combobox", { name: "Thuật toán" });
    expect(select).toHaveTextContent("Mặc định");

    await user.click(select);
    expect(
      screen.getByRole("option", { name: "Cụm dài nhất · nghiêm ngặt" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("option", { name: "Khớp đầu tiên tại vị trí · mặc định" }),
    ).toBeInTheDocument();
    await user.click(
      screen.getByRole("option", { name: "Cụm dài nhất · linh hoạt" }),
    );

    expect(select).toHaveTextContent("Linh hoạt");
  });
});
