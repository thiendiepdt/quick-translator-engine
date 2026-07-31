import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { NameFilterWorkspace } from "@/components/name-filter-workspace";
import { nameFilterModeStorageKey } from "@/lib/name-filter-mode";

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
});

afterEach(cleanup);

describe("name filter mode preference", () => {
  it("defaults to QT Legacy and persists that default", async () => {
    renderWorkspace();

    expect(screen.getByRole("tab", { name: "QT LEGACY" })).toHaveAttribute(
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

    expect(screen.getByRole("tab", { name: "HYBRID" })).toHaveAttribute(
      "aria-selected",
      "true",
    );

    await user.click(screen.getByRole("tab", { name: "QT LEGACY" }));
    expect(localStorage.getItem(nameFilterModeStorageKey)).toBe("qt");
  });
});
