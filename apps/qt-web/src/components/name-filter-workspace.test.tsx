import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { NameFilterWorkspace } from "@/components/name-filter-workspace";
import {
  nameApprovalThresholdStorageKey,
  nameFilterModeStorageKey,
} from "@/lib/name-filter-mode";
import { useWorkspaceStore } from "@/store/workspace";

function renderWorkspace() {
  const queryClient = new QueryClient({
    defaultOptions: { mutations: { retry: false } },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <NameFilterWorkspace endpoint="/api" defaultsReady />
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  localStorage.clear();
  useWorkspaceStore.getState().clearNameMemory();
  useWorkspaceStore.setState({ nameFilterResponse: undefined });
});

afterEach(cleanup);

describe("name filter mode preference", () => {
  it("defaults to QT Legacy and persists that default", async () => {
    renderWorkspace();

    expect(screen.getByRole("tab", { name: "QT cũ" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    await waitFor(() => {
      expect(localStorage.getItem(nameFilterModeStorageKey)).toBe("qt");
    });
  });

  it("restores and updates the saved mode", async () => {
    localStorage.setItem(nameFilterModeStorageKey, "hybrid");
    const user = userEvent.setup();
    renderWorkspace();

    expect(screen.getByRole("tab", { name: "Kết hợp" })).toHaveAttribute(
      "aria-selected",
      "true",
    );

    await user.click(screen.getByRole("tab", { name: "QT cũ" }));
    expect(localStorage.getItem(nameFilterModeStorageKey)).toBe("qt");
  });

  it("allows changing and restoring the approval threshold", async () => {
    localStorage.setItem(nameApprovalThresholdStorageKey, "72");
    const user = userEvent.setup();
    renderWorkspace();

    const threshold = screen.getByRole("spinbutton", { name: "Ngưỡng duyệt (%)" });
    expect(threshold).toHaveValue(72);

    await user.clear(threshold);
    await user.type(threshold, "100");
    await user.tab();

    expect(threshold).toHaveValue(100);
    expect(localStorage.getItem(nameApprovalThresholdStorageKey)).toBe("100");
  });

  it("shows rejected names and restores them to the review queue", async () => {
    useWorkspaceStore.getState().rejectNameCandidate("萧炎");
    const user = userEvent.setup();
    renderWorkspace();

    await user.click(screen.getByRole("tab", { name: "Đã loại 1" }));
    expect(screen.getByText("萧炎")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Khôi phục 萧炎" }));
    expect(useWorkspaceStore.getState().rejectedNames).not.toContain("萧炎");
    expect(screen.getByRole("tab", { name: "Đã loại 0" })).toBeInTheDocument();
  });

  it("highlights rejected names and restores all of them", async () => {
    useWorkspaceStore.getState().rejectNameCandidate("萧炎");
    useWorkspaceStore.getState().rejectNameCandidate("药老");
    const user = userEvent.setup();
    renderWorkspace();

    await user.click(screen.getByRole("button", { name: "Xem 2 tên đã loại" }));
    expect(screen.getByText("萧炎")).toBeInTheDocument();
    expect(screen.getByText("药老")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Khôi phục tất cả" }));
    expect(useWorkspaceStore.getState().rejectedNames).toEqual([]);
    expect(screen.getByRole("tab", { name: "Chờ duyệt 0" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
  });

  it("undoes approval without rejecting the name", async () => {
    useWorkspaceStore.getState().setNameFilterResponse({
      candidates: [
        {
          text: "萧炎",
          suggested: "Tiêu Viêm",
          entityType: "person",
          score: 0.95,
          occurrences: 3,
          ranges: [],
          contexts: [],
          reasons: [],
          sources: ["qt"],
          known: false,
        },
      ],
      stats: { scannedCharacters: 20, ruleCandidates: 1, aiExtractedCandidates: 0, aiReviewed: 0 },
      capabilities: { aiConfigured: false },
    });
    useWorkspaceStore.getState().acceptNameCandidate("萧炎", "Tiêu Viêm");
    const user = userEvent.setup();
    renderWorkspace();

    await user.click(screen.getByRole("button", { name: "Bỏ duyệt 萧炎" }));

    expect(useWorkspaceStore.getState().knownNames).not.toHaveProperty("萧炎");
    expect(useWorkspaceStore.getState().rejectedNames).not.toContain("萧炎");
    expect(screen.getByRole("button", { name: "Duyệt 萧炎" })).toBeInTheDocument();
  });
});
