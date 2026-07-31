import {
  cleanup,
  fireEvent,
  render,
  screen,
  within,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
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

afterEach(cleanup);

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
    await user.click(await screen.findByText("Update VietPhrase"));

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
    await user.click(await screen.findByText("Update Phiên Âm"));
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
    await user.click(await screen.findByText("Update Name (chính)"));

    expect(screen.getByLabelText("Tiếng Trung")).toHaveValue("萧炎");
    expect(screen.getByLabelText("Tiếng Việt")).toHaveValue("Tiêu");
  });
});
