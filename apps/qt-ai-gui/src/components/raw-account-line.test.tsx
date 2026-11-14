import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { AccountLine } from "@/components/raw-account-line";
import { extAccountCheck, extLogout, type ExtAccount, type ExtConfig } from "@/lib/api";
import { resetRawChecks } from "@/store/raw-check";

vi.mock("@/lib/api", () => ({ extAccountCheck: vi.fn(), extLogout: vi.fn() }));
vi.mock("sonner", () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

const config = (session: Partial<ExtConfig["session"]> = {}): ExtConfig => ({
  id: "org.test.qq",
  name: "Demo",
  auth: { required: true, schemes: [{ id: "apiKey", label: "API key", fields: [] }], scheme: "apiKey", values: {}, filled: [] },
  sharedAuth: null,
  settings: [],
  session: { loggedIn: false, expiresAt: null, hasData: false, account: null, guest: false, ...session },
  account: { title: "Tài khoản QQ阅读", required: false, help: "Bấm Đăng nhập, quét mã bằng QQ hoặc WeChat." },
  secretStore: "keyring",
  cacheBytes: 0,
});

const result = (status: Partial<ExtAccount["status"]>, cfg = config()): ExtAccount => ({
  config: cfg,
  status: { loggedIn: false, name: null, message: null, checked: true, ...status },
  checkedAt: new Date(2026, 10, 14, 10, 42).getTime(),
});

describe("AccountLine", () => {
  beforeEach(() => {
    resetRawChecks();
    vi.mocked(extAccountCheck).mockReset();
    vi.mocked(extLogout).mockReset();
  });

  it("mở trang thì kiểm (nhớ kết quả), hiện lời extension nhắn và hướng dẫn; Kiểm tra thì kiểm lại", async () => {
    const user = userEvent.setup();
    vi.mocked(extAccountCheck).mockResolvedValue(result({ message: "đang tải kiểu khách — chỉ chương miễn phí" }));
    const props = { root: "D:\\s", config: config({ loggedIn: true, guest: true, hasData: true }), busy: false, onLogin: vi.fn(), onConfig: vi.fn() };
    const { unmount } = render(<AccountLine {...props} />);
    expect(screen.getByLabelText("Trạng thái Tài khoản QQ阅读")).toHaveTextContent("đang kiểm…");
    await waitFor(() => expect(screen.getByLabelText("Trạng thái Tài khoản QQ阅读")).toHaveTextContent("chưa đăng nhập — đang tải kiểu khách — chỉ chương miễn phí"));
    expect(screen.getByText("Bấm Đăng nhập, quét mã bằng QQ hoặc WeChat.")).toBeInTheDocument();
    expect(screen.getByText("kiểm lúc 10:42")).toBeInTheDocument();
    expect(extAccountCheck).toHaveBeenCalledWith("D:\\s", "org.test.qq");

    unmount();
    render(<AccountLine {...props} />);
    await screen.findByText(/chưa đăng nhập/);
    expect(extAccountCheck).toHaveBeenCalledTimes(1);

    vi.mocked(extAccountCheck).mockResolvedValue(result({ loggedIn: true, name: "1234567" }));
    await user.click(screen.getByRole("button", { name: /Kiểm tra/ }));
    await waitFor(() => expect(screen.getByLabelText("Trạng thái Tài khoản QQ阅读")).toHaveTextContent("đã đăng nhập: 1234567"));
    expect(extAccountCheck).toHaveBeenCalledTimes(2);

    await user.click(screen.getByRole("button", { name: /Đăng nhập lại/ }));
    expect(props.onLogin).toHaveBeenCalled();
  });

  it("kiểm lỗi thì báo, vẫn theo phiên đã lưu; Đăng xuất xoá phiên rồi kiểm lại", async () => {
    const user = userEvent.setup();
    vi.mocked(extAccountCheck).mockRejectedValueOnce(new Error("mạng lỗi"));
    const onConfig = vi.fn();
    render(<AccountLine root="D:\\s" config={config({ loggedIn: true, account: "Nguyễn A", hasData: true })} busy={false} onLogin={vi.fn()} onConfig={onConfig} />);
    expect(await screen.findByText("Không kiểm được: mạng lỗi")).toBeInTheDocument();
    expect(screen.getByLabelText("Trạng thái Tài khoản QQ阅读")).toHaveTextContent("đã đăng nhập: Nguyễn A");

    vi.mocked(extLogout).mockResolvedValue(config());
    await user.click(screen.getByRole("button", { name: /Đăng xuất/ }));
    expect(extLogout).toHaveBeenCalledWith("org.test.qq");
    expect(onConfig).toHaveBeenCalledWith(config());
  });
});
