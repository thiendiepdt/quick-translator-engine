import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { ExtConfigDialog } from "@/components/ext-config-dialog";
import { extClearCache, extLogout, extSaveAuth, extSaveSettings, type ExtConfig } from "@/lib/api";

vi.mock("@/lib/api", () => ({
  extSaveAuth: vi.fn(),
  extClearAuth: vi.fn(),
  extSaveSettings: vi.fn(),
  extLogout: vi.fn(),
  extClearCache: vi.fn(),
}));
vi.mock("sonner", () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

const config: ExtConfig = {
  id: "io.github.dev.demo",
  name: "Demo",
  auth: {
    required: false,
    schemes: [
      { id: "apiKey", label: "API key", fields: [{ key: "apiKey", label: "API key", kind: "secret" }] },
      {
        id: "password",
        label: "Tài khoản",
        fields: [
          { key: "username", label: "Tên đăng nhập", kind: "text" },
          { key: "password", label: "Mật khẩu", kind: "secret" },
        ],
      },
    ],
    scheme: "password",
    values: { username: "an" },
    filled: ["username", "password"],
  },
  settings: [
    { key: "server", label: "Máy chủ", kind: "select", choices: ["cn", "global"], default: "cn", scope: "global", value: null, filled: false },
    { key: "pageSize", label: "Cỡ trang", kind: "number", choices: [], default: 50, scope: "global", value: 20, filled: false },
    { key: "fullWidth", label: "Giữ dấu câu toàn khổ", kind: "boolean", choices: [], default: true, scope: "global", value: null, filled: false },
    { key: "token", label: "Token", kind: "secret", choices: [], default: null, scope: "global", value: null, filled: true },
  ],
  session: { loggedIn: true, expiresAt: null, hasData: true, account: null, guest: false },
  secretStore: "keyring",
  cacheBytes: 2048,
  sharedAuth: null,
  account: null,
};

describe("ExtConfigDialog", () => {
  beforeEach(() => {
    for (const fn of [extSaveAuth, extSaveSettings, extLogout, extClearCache]) vi.mocked(fn).mockReset().mockResolvedValue(config);
  });

  it("auth vẽ từ manifest: ô secret đã lưu để trống là giữ, đổi kiểu thì đổi trường", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(<ExtConfigDialog config={config} open onOpenChange={() => undefined} onChange={onChange} />);
    expect(screen.getByLabelText("Tên đăng nhập")).toHaveValue("an");
    expect(screen.getByLabelText("Mật khẩu")).toHaveAttribute("placeholder", "Đã lưu — để trống để giữ");
    expect(screen.getByLabelText("Mật khẩu")).toHaveAttribute("type", "password");
    await user.click(screen.getByRole("button", { name: "Lưu auth" }));
    expect(extSaveAuth).toHaveBeenCalledWith("io.github.dev.demo", "password", { username: "an", password: "" });
    expect(onChange).toHaveBeenCalledWith(config);

    await user.click(screen.getByRole("combobox", { name: "Kiểu auth" }));
    await user.click(await screen.findByRole("option", { name: "API key" }));
    const key = screen.getByLabelText("API key");
    expect(key).toHaveAttribute("placeholder", "");
    await user.type(key, "k-123");
    await user.click(screen.getByRole("button", { name: "Lưu auth" }));
    expect(extSaveAuth).toHaveBeenLastCalledWith("io.github.dev.demo", "apiKey", { apiKey: "k-123" });
  });

  it("cài đặt chung chỉ gửi trường đã đổi; đăng xuất, xoá cache; cảnh báo khi không có kho khoá", async () => {
    const user = userEvent.setup();
    render(<ExtConfigDialog config={{ ...config, secretStore: "file" }} open onOpenChange={() => undefined} onChange={() => undefined} />);
    expect(screen.getByRole("alert")).toHaveTextContent("không có kho khoá");
    expect(screen.getByRole("button", { name: "Lưu cài đặt" })).toBeDisabled();
    expect(screen.getByLabelText("Cỡ trang")).toHaveValue("20");
    expect(screen.getByRole("switch", { name: "Giữ dấu câu toàn khổ" })).toBeChecked();

    await user.clear(screen.getByLabelText("Cỡ trang"));
    await user.click(screen.getByRole("switch", { name: "Giữ dấu câu toàn khổ" }));
    await user.type(screen.getByLabelText("Token"), "tk-9");
    await user.click(screen.getByRole("button", { name: "Lưu cài đặt" }));
    expect(extSaveSettings).toHaveBeenCalledWith("io.github.dev.demo", { pageSize: null, fullWidth: false, token: "tk-9" });

    expect(screen.getByText("Đã đăng nhập (Tài khoản) · cache 2 KB")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Đăng xuất" }));
    expect(extLogout).toHaveBeenCalledWith("io.github.dev.demo");
    await user.click(screen.getByRole("button", { name: "Xoá cache" }));
    expect(extClearCache).toHaveBeenCalledWith("io.github.dev.demo");
  });
});
