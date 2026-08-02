import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { DictionaryInspector } from "@/components/dictionary-inspector";
import { dictionaryKeys, type DictionaryDefaults } from "@/lib/types";
import { useWorkspaceStore } from "@/store/workspace";

// EngineOptions cần react-hook-form context của app; không liên quan tới
// phần nhập từ điển đang test nên thay bằng khối rỗng.
vi.mock("@/components/engine-options", () => ({
  EngineOptions: () => null,
}));

const emptyDefaults = Object.fromEntries(
  dictionaryKeys.map((key) => [key, ""]),
) as DictionaryDefaults;

let endpointIndex = 0;

beforeEach(() => {
  endpointIndex += 1;
  useWorkspaceStore
    .getState()
    .hydrateDictionaryDefaults(`/inspector-test-${endpointIndex}`, emptyDefaults);
  useWorkspaceStore.getState().clearWorkspace();
  useWorkspaceStore.getState().setActiveDictionary("names");
});

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

function renderInspector() {
  return render(
    <DictionaryInspector defaultsStatus="ready" onRetry={() => undefined} />,
  );
}

describe("dictionary quick import", () => {
  it("appends clipboard text to the active dictionary", async () => {
    // Không dùng userEvent ở đây: userEvent.setup() tự stub navigator.clipboard.
    Object.defineProperty(window.navigator, "clipboard", {
      configurable: true,
      value: { readText: vi.fn().mockResolvedValue("萧炎=Tiêu Viêm") },
    });
    useWorkspaceStore.getState().setDictionaryValue("names", "药老=Dược Lão");
    renderInspector();

    fireEvent.click(screen.getByRole("button", { name: "Dán" }));

    await waitFor(() => {
      expect(useWorkspaceStore.getState().dictionaries.names.value).toBe(
        "药老=Dược Lão\n萧炎=Tiêu Viêm",
      );
    });
  });

  it("appends a dropped GBK dictionary file with the right encoding", async () => {
    useWorkspaceStore.getState().setDictionaryValue("names", "药老=Dược Lão");
    renderInspector();

    // "你好" mã hóa GBK — đọc mù UTF-8 sẽ ra mojibake.
    const file = new File([new Uint8Array([0xc4, 0xe3, 0xba, 0xc3])], "names.txt", {
      type: "text/plain",
    });
    fireEvent.drop(screen.getByRole("button", { name: /Sửa bản ghi/ }), {
      dataTransfer: { files: [file], types: ["Files"] },
    });

    await waitFor(() => {
      expect(useWorkspaceStore.getState().dictionaries.names.value).toBe("药老=Dược Lão\n你好");
    });
  });
});
