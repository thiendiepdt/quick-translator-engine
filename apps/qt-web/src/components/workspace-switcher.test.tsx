import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { WorkspaceSwitcher } from "@/components/workspace-switcher";
import { selectWorkspace } from "@/store/workspace-controller";
import {
  activeWorkspaceStorageKey,
  defaultWorkspace,
  useWorkspaceCatalogStore,
} from "@/store/workspace-catalog";

let testIndex = 0;

beforeEach(async () => {
  testIndex += 1;
  await selectWorkspace(defaultWorkspace.id);
});

afterEach(async () => {
  cleanup();
  await selectWorkspace(defaultWorkspace.id);
});

describe("workspace switcher", () => {
  it("shows the default workspace and creates a new selected workspace", async () => {
    const user = userEvent.setup();
    render(<WorkspaceSwitcher />);

    expect(screen.getByText("Không gian làm việc:")).toBeInTheDocument();
    await user.click(
      screen.getByRole("button", { name: "Không gian làm việc hiện tại: Mặc định" }),
    );
    expect(
      screen.getByRole("heading", { name: "Không gian làm việc" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /Mặc định.*Đang dùng/ }),
    ).toBeInTheDocument();

    await user.click(
      screen.getByRole("button", { name: "Tạo không gian làm việc" }),
    );
    const name = `Không gian giao diện ${testIndex}`;
    await user.type(screen.getByLabelText("Tên không gian làm việc"), name);
    await user.click(
      screen.getByRole("button", { name: "Tạo không gian làm việc" }),
    );

    await waitFor(() => {
      const activeWorkspaceId = useWorkspaceCatalogStore.getState().activeWorkspaceId;
      expect(activeWorkspaceId).not.toBe(defaultWorkspace.id);
      expect(localStorage.getItem(activeWorkspaceStorageKey)).toBe(activeWorkspaceId);
    });
    expect(
      screen.getByRole("button", { name: `Không gian làm việc hiện tại: ${name}` }),
    ).toBeInTheDocument();
  });
});
