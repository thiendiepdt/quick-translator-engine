import { describe, expect, it } from "vitest";

import { activeModel, modelEffort, weakModelWarning } from "@/lib/models";
import type { AgyStatus } from "@/lib/types";

const agy = (defaultModel: AgyStatus["defaultModel"]): AgyStatus => ({
  found: true,
  path: "agy",
  version: "1.3",
  message: null,
  models: [
    { id: "gemini-4-hydrogen-high", label: "Gemini 4 Hydrogen (High)" },
    { id: "gemini-3.8-flash-medium", label: "Gemini 3.8 Flash (Medium)" },
  ],
  defaultModel,
});

describe("model agy", () => {
  it("đọc mức suy nghĩ từ đuôi id hoặc nhãn", () => {
    expect(modelEffort("gemini-3.8-flash-medium")).toBe("medium");
    expect(modelEffort("gemini-4-hydrogen-xhigh")).toBe("xhigh");
    expect(modelEffort("x", "Claude Opus 5.5 (Low)")).toBe("low");
    expect(modelEffort("gemini-pro-orchestrator", "Orchestrator (Carbon)")).toBeNull();
  });

  it("model đang dùng: chọn trong app, trống thì model mặc định của agy", () => {
    expect(activeModel("gemini-3.8-flash-medium", agy(null))).toEqual({ id: "gemini-3.8-flash-medium", label: "Gemini 3.8 Flash (Medium)", fromAgy: false });
    expect(activeModel(" ", agy({ id: "gemini-4-hydrogen-high", label: "Gemini 4 Hydrogen (High)" }))?.fromAgy).toBe(true);
    expect(activeModel(null, agy(null))).toBeNull();
  });

  it("cảnh báo mức Low / Medium, không cảnh báo High trở lên hay model không ghi mức", () => {
    expect(weakModelWarning(activeModel("gemini-3.8-flash-medium", agy(null)))).toBe(
      "Gemini 3.8 Flash (Medium) đang ở mức suy nghĩ Medium — chất lượng dịch giảm khá nhiều. Nên chọn bản High, ví dụ gemini-3.8-flash-high hoặc gemini-4-hydrogen-high.",
    );
    expect(weakModelWarning(activeModel(null, agy({ id: "claude-sonnet-5-5-low", label: "Claude Sonnet 5.5 (Low)" })))).toContain("(model mặc định của agy) đang ở mức suy nghĩ Low");
    expect(weakModelWarning(activeModel("gemini-4-hydrogen-high", agy(null)))).toBeNull();
    expect(weakModelWarning(activeModel("gemini-pro-orchestrator", agy(null)))).toBeNull();
    expect(weakModelWarning(null)).toBeNull();
  });
});
