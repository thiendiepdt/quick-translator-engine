import { cleanup, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { TranslationWorkspace } from "@/components/translation-workspace";
import { useWorkspaceStore } from "@/store/workspace";

const scrollToMock = vi.fn();

beforeEach(() => {
  scrollToMock.mockClear();
  Object.defineProperty(HTMLElement.prototype, "scrollTo", {
    configurable: true,
    value: scrollToMock,
  });
  useWorkspaceStore.setState({ rangePinEnabled: true });
  useWorkspaceStore.getState().loadSample();
});

afterEach(cleanup);

describe("translation range pin", () => {
  it("scrolls the matching output range when a source range is selected", async () => {
    const user = userEvent.setup();
    render(<TranslationWorkspace isPending={false} requestStatus="Sẵn sàng" />);

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
    render(<TranslationWorkspace isPending={false} requestStatus="Sẵn sàng" />);

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
    expect(localStorage.getItem("qt-web-name-memory-v1")).toContain(
      '"rangePinEnabled":false',
    );
  });

  it("opens the linked source view before scrolling from the output", async () => {
    const user = userEvent.setup();
    useWorkspaceStore.getState().setSourceView("raw");
    render(<TranslationWorkspace isPending={false} requestStatus="Sẵn sàng" />);

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
});
