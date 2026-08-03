import {
  cleanup,
  fireEvent,
  render,
  screen,
  within,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { toast } from "sonner";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { TranslationWorkspace } from "@/components/translation-workspace";
import {
  dictionaryUpdateKeys,
  type LocalDictionaryEntries,
} from "@/lib/types";
import {
  useWorkspaceStore,
  workspaceStateStorage,
  workspaceStorageKey,
} from "@/store/workspace";

const scrollToMock = vi.fn();
const onTranslateMock = vi.fn();

function renderWorkspace() {
  return render(
    <TranslationWorkspace
      endpoint="/api"
      canTranslate
      isPending={false}
      onTranslate={onTranslateMock}
      requestStatus="Sẵn sàng"
    />,
  );
}

async function persistedWorkspace(): Promise<string> {
  return (await workspaceStateStorage.getItem(workspaceStorageKey)) ?? "";
}

beforeEach(() => {
  scrollToMock.mockClear();
  onTranslateMock.mockClear();
  Object.defineProperty(HTMLElement.prototype, "scrollTo", {
    configurable: true,
    value: scrollToMock,
  });
  useWorkspaceStore.setState({ rangePinEnabled: true });
  useWorkspaceStore.setState({
    localDictionaryEntries: Object.fromEntries(
      dictionaryUpdateKeys.map((key) => [key, {}]),
    ) as LocalDictionaryEntries,
  });
  useWorkspaceStore.getState().loadSample();
});

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

describe("quick source clear", () => {
  it("clears the source for a new paste and restores it via undo", async () => {
    const message = vi.spyOn(toast, "message").mockReturnValue("toast-id");
    const user = userEvent.setup();
    useWorkspaceStore.getState().setSourceText("旧章节");
    renderWorkspace();

    await user.click(
      screen.getByRole("button", { name: "Xóa nguyên văn để dán chương mới" }),
    );

    expect(useWorkspaceStore.getState().sourceText).toBe("");
    // "Hoàn tác" trong toast trả lại đúng chương vừa xóa.
    const options = message.mock.calls[0]?.[1] as
      | { action?: { onClick: () => void } }
      | undefined;
    options?.action?.onClick();
    expect(useWorkspaceStore.getState().sourceText).toBe("旧章节");
  });
});

describe("translation range pin", () => {
  it("keeps the translation action next to the source text", async () => {
    const user = userEvent.setup();
    useWorkspaceStore.getState().setSourceText("天地");
    renderWorkspace();

    const sourcePane = screen.getByRole("region", { name: "Nguyên văn" });
    expect(within(sourcePane).getByText("2")).toBeInTheDocument();
    const translateButton = within(sourcePane).getByRole("button", {
      name: /Dịch chương/,
    });
    expect(translateButton).toHaveAttribute(
      "title",
      "Dịch chương (Ctrl/⌘ + Enter)",
    );

    await user.click(translateButton);
    expect(onTranslateMock).toHaveBeenCalledOnce();
  });

  it("scrolls the matching output range when a source range is selected", async () => {
    const user = userEvent.setup();
    renderWorkspace();

    const sourcePane = screen.getByRole("region", { name: "Nguyên văn" });
    const outputPane = screen.getByRole("region", { name: "Bản dịch" });
    const outputScroller = outputPane.querySelector(".fine-scrollbar");

    await user.click(
      within(sourcePane).getByRole("button", { name: "Range 1: 萧炎" }),
    );

    expect(
      within(outputPane).getByRole("button", { name: "Range 1: Tiêu Viêm" }),
    ).toHaveAttribute("data-active", "true");
    expect(scrollToMock).toHaveBeenCalledWith({
      top: 0,
      behavior: "smooth",
    });
    expect(scrollToMock.mock.instances.at(-1)).toBe(outputScroller);
  });

  it("keeps range highlighting but stops auto-scroll when pin is off", async () => {
    const user = userEvent.setup();
    renderWorkspace();

    await user.click(screen.getByRole("button", { name: "Tắt tự cuộn range" }));
    scrollToMock.mockClear();

    const sourcePane = screen.getByRole("region", { name: "Nguyên văn" });
    const outputPane = screen.getByRole("region", { name: "Bản dịch" });
    await user.click(
      within(sourcePane).getByRole("button", { name: "Range 2: 看着" }),
    );

    expect(
      within(outputPane).getByRole("button", { name: "Range 2: nhìn" }),
    ).toHaveAttribute("data-active", "true");
    expect(scrollToMock).not.toHaveBeenCalled();
    expect(useWorkspaceStore.getState().rangePinEnabled).toBe(false);
    expect(await persistedWorkspace()).toContain(
      '"rangePinEnabled":false',
    );
  });

  it("opens the linked source view before scrolling from the output", async () => {
    const user = userEvent.setup();
    useWorkspaceStore.getState().setSourceView("raw");
    renderWorkspace();

    const outputPane = screen.getByRole("region", { name: "Bản dịch" });
    await user.click(
      within(outputPane).getByRole("button", { name: "Range 3: nàng" }),
    );

    const sourcePane = screen.getByRole("region", { name: "Nguyên văn" });
    expect(
      within(sourcePane).getByRole("button", { name: "Range 3: 她" }),
    ).toHaveAttribute("data-active", "true");
    expect(useWorkspaceStore.getState().sourceView).toBe("linked");
    expect(scrollToMock.mock.instances.at(-1)).toBe(
      sourcePane.querySelector(".fine-scrollbar"),
    );
  });

  it("updates a mapped VietPhrase entry from the output context menu", async () => {
    const user = userEvent.setup();
    renderWorkspace();

    const outputPane = screen.getByRole("region", { name: "Bản dịch" });
    const segment = within(outputPane).getByRole("button", {
      name: "Range 2: nhìn",
    });
    fireEvent.contextMenu(segment);
    await user.click(await screen.findByText("Cập nhật VietPhrase"));

    expect(screen.getByLabelText("Tiếng Trung")).toHaveValue("看着");
    const target = screen.getByLabelText("Tiếng Việt");
    expect(target).toHaveValue("nhìn");
    await user.clear(target);
    await user.type(target, "quan sát");
    await user.click(screen.getByRole("button", { name: "Lưu local" }));

    expect(
      useWorkspaceStore.getState().localDictionaryEntries.vietPhrase,
    ).toEqual({ 看着: "quan sát" });
    expect(await persistedWorkspace()).toContain(
      '"vietPhrase":{"看着":"quan sát"}',
    );
  });

  it("splits a multi-character Phiên Âm update into one patch per character", async () => {
    const user = userEvent.setup();
    renderWorkspace();

    const outputPane = screen.getByRole("region", { name: "Bản dịch" });
    fireEvent.contextMenu(
      within(outputPane).getByRole("button", {
        name: "Range 1: Tiêu Viêm",
      }),
    );
    await user.click(await screen.findByText("Cập nhật Phiên Âm"));
    await user.click(screen.getByRole("button", { name: "Lưu local" }));

    expect(
      useWorkspaceStore.getState().localDictionaryEntries
        .chinesePhienAmWords,
    ).toEqual({ 萧: "Tiêu", 炎: "Viêm" });
  });

  it("uses text selected inside one mapped range as the initial value", async () => {
    const user = userEvent.setup();
    renderWorkspace();

    const outputPane = screen.getByRole("region", { name: "Bản dịch" });
    const segment = within(outputPane).getByRole("button", {
      name: "Range 1: Tiêu Viêm",
    });
    const textNode = segment.firstChild;
    if (!textNode) throw new Error("missing mapped text node");
    const range = document.createRange();
    range.setStart(textNode, 0);
    range.setEnd(textNode, 4);
    window.getSelection()?.removeAllRanges();
    window.getSelection()?.addRange(range);

    fireEvent.contextMenu(segment);
    await user.click(await screen.findByText("Cập nhật Tên"));

    expect(screen.getByLabelText("Tiếng Trung")).toHaveValue("萧炎");
    expect(screen.getByLabelText("Tiếng Việt")).toHaveValue("Tiêu");
  });

  it("uses all mapped ranges crossed by a text selection", async () => {
    const user = userEvent.setup();
    renderWorkspace();

    const outputPane = screen.getByRole("region", { name: "Bản dịch" });
    const firstSegment = within(outputPane).getByRole("button", {
      name: "Range 1: Tiêu Viêm",
    });
    const secondSegment = within(outputPane).getByRole("button", {
      name: "Range 2: nhìn",
    });
    const firstText = firstSegment.firstChild;
    const secondText = secondSegment.firstChild;
    if (!firstText || !secondText) throw new Error("missing mapped text nodes");
    const range = document.createRange();
    range.setStart(firstText, 0);
    range.setEnd(secondText, secondText.textContent?.length ?? 0);
    window.getSelection()?.removeAllRanges();
    window.getSelection()?.addRange(range);

    fireEvent.contextMenu(secondSegment);
    await user.click(await screen.findByText("Cập nhật Tên 2"));

    expect(screen.getByLabelText("Tiếng Trung")).toHaveValue("萧炎看着");
    expect(screen.getByLabelText("Tiếng Việt")).toHaveValue("Tiêu Viêm nhìn");
  });

  it("extends the clicked range with Shift and an arrow key", async () => {
    const user = userEvent.setup();
    renderWorkspace();

    const outputPane = screen.getByRole("region", { name: "Bản dịch" });
    const firstSegment = within(outputPane).getByRole("button", {
      name: "Range 1: Tiêu Viêm",
    });
    const secondSegment = within(outputPane).getByRole("button", {
      name: "Range 2: nhìn",
    });
    await user.click(firstSegment);
    await user.keyboard("{Shift>}{ArrowRight}{/Shift}");

    expect(firstSegment).toHaveAttribute("data-active", "true");
    expect(secondSegment).toHaveAttribute("data-active", "true");

    fireEvent.contextMenu(secondSegment);
    await user.click(await screen.findByText("Cập nhật Tên 2"));
    expect(screen.getByLabelText("Tiếng Trung")).toHaveValue("萧炎看着");
    expect(screen.getByLabelText("Tiếng Việt")).toHaveValue("Tiêu Viêm nhìn");
  });

  it("removes the last selected range when Shift and the opposite arrow are pressed", async () => {
    const user = userEvent.setup();
    renderWorkspace();

    const outputPane = screen.getByRole("region", { name: "Bản dịch" });
    const firstSegment = within(outputPane).getByRole("button", {
      name: "Range 1: Tiêu Viêm",
    });
    const secondSegment = within(outputPane).getByRole("button", {
      name: "Range 2: nhìn",
    });
    await user.click(secondSegment);
    await user.keyboard("{Shift>}{ArrowLeft}{/Shift}");
    expect(firstSegment).toHaveAttribute("data-active", "true");
    expect(secondSegment).toHaveAttribute("data-active", "true");

    await user.keyboard("{Shift>}{ArrowRight}{/Shift}");
    expect(firstSegment).toHaveAttribute("data-active", "false");
    expect(secondSegment).toHaveAttribute("data-active", "true");
  });

  it("skips zero-length ranges when extending the keyboard selection", async () => {
    const user = userEvent.setup();
    useWorkspaceStore.setState({
      sourceText: "是如此的完整",
      response: {
        translated: "Là như vậy hoàn chỉnh",
        sourceRanges: [
          { start: 0, length: 1 },
          { start: 1, length: 2 },
          { start: 3, length: 1 },
          { start: 4, length: 2 },
        ],
        targetRanges: [
          { start: 0, length: 2 },
          { start: 3, length: 7 },
          { start: 10, length: 0 },
          { start: 11, length: 10 },
        ],
      },
    });
    renderWorkspace();

    const outputPane = screen.getByRole("region", { name: "Bản dịch" });
    const firstSegment = within(outputPane).getByRole("button", {
      name: "Range 1: Là",
    });
    const secondSegment = within(outputPane).getByRole("button", {
      name: "Range 2: như vậy",
    });
    const thirdVisibleSegment = within(outputPane).getByRole("button", {
      name: "Range 4: hoàn chỉnh",
    });
    await user.click(firstSegment);
    await user.keyboard("{Shift>}{ArrowRight}{ArrowRight}{/Shift}");

    expect(firstSegment).toHaveAttribute("data-active", "true");
    expect(secondSegment).toHaveAttribute("data-active", "true");
    expect(thirdVisibleSegment).toHaveAttribute("data-active", "true");
  });
});
