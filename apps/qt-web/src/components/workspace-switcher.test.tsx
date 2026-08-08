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
    await waitFor(() => {
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    });
    expect(
      screen.getByRole("button", { name: `Không gian làm việc hiện tại: ${name}` }),
    ).toBeInTheDocument();
  });

  it("deletes the active workspace after confirming and falls back to default", async () => {
    const user = userEvent.setup();
    const name = `Không gian sẽ xóa ${testIndex}`;
    render(<WorkspaceSwitcher />);

    await user.click(
      screen.getByRole("button", { name: "Không gian làm việc hiện tại: Mặc định" }),
    );
    await user.click(screen.getByRole("button", { name: "Tạo không gian làm việc" }));
    await user.type(screen.getByLabelText("Tên không gian làm việc"), name);
    await user.click(screen.getByRole("button", { name: "Tạo không gian làm việc" }));
    await waitFor(() => {
      expect(useWorkspaceCatalogStore.getState().activeWorkspaceId).not.toBe(
        defaultWorkspace.id,
      );
    });

    await user.click(
      screen.getByRole("button", { name: `Không gian làm việc hiện tại: ${name}` }),
    );
    expect(
      screen.queryByRole("button", { name: "Xóa không gian làm việc Mặc định" }),
    ).not.toBeInTheDocument();

    await user.click(
      screen.getByRole("button", { name: `Xóa không gian làm việc ${name}` }),
    );
    await user.click(screen.getByRole("button", { name: "Xóa" }));

    await waitFor(() => {
      expect(useWorkspaceCatalogStore.getState().activeWorkspaceId).toBe(
        defaultWorkspace.id,
      );
    });
    expect(
      useWorkspaceCatalogStore.getState().workspaces.map(({ name: n }) => n),
    ).not.toContain(name);
  });
});
