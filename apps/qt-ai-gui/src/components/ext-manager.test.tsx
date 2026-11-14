import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { ExtensionManager } from "@/components/ext-manager";
import { extCheckUpdate, extInspectUrl, extInstall, extList, extTestToc, extUninstall, type ExtInspection, type ExtListItem } from "@/lib/api";

vi.mock("@tauri-apps/api/webview", () => ({
  getCurrentWebview: () => ({ onDragDropEvent: () => Promise.resolve(() => undefined) }),
}));
vi.mock("@/lib/api", () => ({
  extList: vi.fn(),
  extInspectFile: vi.fn(),
  extInspectUrl: vi.fn(),
  extInstall: vi.fn(),
  extUninstall: vi.fn(),
  extSetEnabled: vi.fn(),
  extCheckUpdate: vi.fn(),
  extAddDev: vi.fn(),
  extConfig: vi.fn(),
  extTestToc: vi.fn(),
  extTestChapter: vi.fn(),
  pickFolder: vi.fn(),
  pickZipFile: vi.fn(),
  repoList: vi.fn(() => Promise.resolve([])),
  repoInspect: vi.fn(),
}));
vi.mock("sonner", () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

const linked: ExtListItem = {
  id: "io.github.dev.fanqie",
  name: "Fanqie",
  version: "1.2.0",
  description: "Tải raw từ fanqienovel.com",
  developer: { name: "dev", url: "https://github.com/dev", email: "dev@example.com" },
  license: "MIT",
  homepage: null,
  hosts: ["fanqienovel.com"],
  permissions: ["webview"],
  authRequired: true,
  configurable: true,
  enabled: true,
  dev: false,
  source: { kind: "link", url: "https://github.com/dev/fanqie/releases/download/v1.2.0/fanqie.zip" },
  installedAt: 1,
  updatedAt: 1,
  icon: null,
  error: null,
};

const dev: ExtListItem = { ...linked, id: "io.github.dev.local", name: "Local", version: "0.1.0", dev: true, enabled: false, permissions: [], source: { kind: "dev", dir: "D:\\ext\\local" }, configurable: false };

const inspection: ExtInspection = {
  sha256: "abc",
  id: "io.github.dev.new",
  name: "Mới",
  version: "1.0.0",
  currentVersion: null,
  description: "d",
  developer: { name: "dev", url: null, email: null },
  license: null,
  homepage: null,
  hosts: ["new.test"],
  newHosts: ["new.test"],
  permissions: [],
  newPermissions: [],
  authRequired: false,
  newer: true,
  needsConsent: true,
  icon: null,
  sharedAuthRepo: null,
};

describe("ExtensionManager", () => {
  beforeEach(() => {
    vi.mocked(extList).mockReset().mockResolvedValue([linked, dev]);
    vi.mocked(extInstall).mockReset().mockResolvedValue([linked, dev]);
    vi.mocked(extUninstall).mockReset().mockResolvedValue([dev]);
    vi.mocked(extInspectUrl).mockReset().mockResolvedValue(inspection);
    vi.mocked(extCheckUpdate).mockReset();
  });

  it("vẽ danh sách: nhà phát triển, domain, quyền, nguồn cài, nhãn Dev / Tắt", async () => {
    render(<ExtensionManager />);
    const row = await screen.findByRole("listitem", { name: "Fanqie" });
    expect(within(row).getByRole("link", { name: "dev" })).toHaveAttribute("href", "https://github.com/dev");
    expect(row).toHaveTextContent("dev@example.com · MIT");
    expect(row).toHaveTextContent("Gọi tới fanqienovel.com · mở cửa sổ trình duyệt · cần auth");
    expect(row).toHaveTextContent("Cài từ link https://github.com/dev/fanqie");
    expect(within(row).getByRole("button", { name: /Kiểm tra cập nhật/ })).toBeInTheDocument();
    const local = screen.getByRole("listitem", { name: "Local" });
    expect(within(local).getByText("Dev")).toBeInTheDocument();
    expect(within(local).getByText("Tắt")).toBeInTheDocument();
    expect(within(local).getByRole("button", { name: /Chạy thử/ })).toBeInTheDocument();
    expect(within(local).queryByRole("button", { name: /Kiểm tra cập nhật/ })).toBeNull();
  });

  it("cài từ link: hộp xin quyền rồi mới cài", async () => {
    const user = userEvent.setup();
    render(<ExtensionManager />);
    await screen.findByRole("listitem", { name: "Fanqie" });
    await user.type(screen.getByLabelText("Link gói (.zip, https)"), "https://x.test/new.zip");
    await user.click(screen.getByRole("button", { name: "Cài" }));
    expect(extInspectUrl).toHaveBeenCalledWith("https://x.test/new.zip");
    const dialog = await screen.findByRole("dialog");
    expect(dialog).toHaveTextContent("Cài Mới");
    expect(within(dialog).getByLabelText("Domain extension được gọi tới")).toHaveTextContent("new.test");
    expect(extInstall).not.toHaveBeenCalled();
    await user.click(within(dialog).getByRole("button", { name: "Cài" }));
    expect(extInstall).toHaveBeenCalledWith("abc");
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  });

  it("kiểm tra cập nhật: thêm domain thì hỏi lại, không thêm thì cài luôn", async () => {
    const user = userEvent.setup();
    vi.mocked(extCheckUpdate).mockResolvedValueOnce({ ...inspection, id: linked.id, name: "Fanqie", version: "1.3.0", currentVersion: "1.2.0", hosts: ["fanqienovel.com", "cdn.fq.test"], newHosts: ["cdn.fq.test"] });
    render(<ExtensionManager />);
    const row = await screen.findByRole("listitem", { name: "Fanqie" });
    await user.click(within(row).getByRole("button", { name: /Kiểm tra cập nhật/ }));
    const dialog = await screen.findByRole("dialog");
    expect(dialog).toHaveTextContent("Cập nhật Fanqie");
    expect(dialog).toHaveTextContent("1.2.0 → 1.3.0");
    expect(within(dialog).getByText("cdn.fq.test (mới)")).toBeInTheDocument();
    expect(within(dialog).getByText("fanqienovel.com")).toBeInTheDocument();
    await user.click(within(dialog).getByRole("button", { name: "Huỷ" }));
    expect(extInstall).not.toHaveBeenCalled();

    vi.mocked(extCheckUpdate).mockResolvedValueOnce({ ...inspection, sha256: "def", version: "1.2.1", currentVersion: "1.2.0", newHosts: [], needsConsent: false });
    await user.click(within(row).getByRole("button", { name: /Kiểm tra cập nhật/ }));
    await waitFor(() => expect(extInstall).toHaveBeenCalledWith("def"));
  });

  it("gỡ hỏi lại trước", async () => {
    const user = userEvent.setup();
    const confirm = vi.spyOn(window, "confirm").mockReturnValueOnce(false).mockReturnValueOnce(true);
    render(<ExtensionManager />);
    const row = await screen.findByRole("listitem", { name: "Fanqie" });
    await user.click(within(row).getByRole("button", { name: /Gỡ/ }));
    expect(extUninstall).not.toHaveBeenCalled();
    await user.click(within(row).getByRole("button", { name: /Gỡ/ }));
    expect(confirm).toHaveBeenLastCalledWith(expect.stringContaining("Raw đã tải giữ nguyên"));
    expect(extUninstall).toHaveBeenCalledWith(linked.id);
    await waitFor(() => expect(screen.queryByRole("listitem", { name: "Fanqie" })).toBeNull());
  });

  it("chạy thử toc hiện mục lục, request và log", async () => {
    const user = userEvent.setup();
    vi.mocked(extTestToc).mockResolvedValue({
      ok: true,
      ms: 321,
      result: [
        { id: "1", title: "第1章 起" },
        { id: "2", title: "第2章 承", locked: true },
      ],
      error: null,
      requests: [{ method: "GET", url: "http://127.0.0.1:9000/book/1", status: 200, ms: 12, error: null }],
      logs: [{ level: "info", message: "progress 1/0 trang mục lục 1" }],
    });
    render(<ExtensionManager />);
    const local = await screen.findByRole("listitem", { name: "Local" });
    await user.click(within(local).getByRole("button", { name: /Chạy thử/ }));
    const dialog = await screen.findByRole("dialog");
    await user.type(within(dialog).getByLabelText("Link tải raw"), "http://127.0.0.1:9000/book/1");
    await user.click(within(dialog).getByRole("button", { name: /Chạy toc/ }));
    expect(extTestToc).toHaveBeenCalledWith("io.github.dev.local", "http://127.0.0.1:9000/book/1");
    expect(await within(dialog).findByLabelText("Kết quả toc")).toHaveTextContent("toc: xong · 321 ms · 1 request");
    expect(within(dialog).getByText("2 mục")).toBeInTheDocument();
    expect(within(dialog).getByLabelText("Request toc")).toHaveTextContent("GET http://127.0.0.1:9000/book/1 → 200 (12 ms)");
    expect(within(dialog).getByLabelText("Log toc")).toHaveTextContent("[info] progress 1/0 trang mục lục 1");
    expect(within(dialog).getByRole("button", { name: "Chạy chapter 2" })).toBeInTheDocument();
  });
});
