import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { ExtRepositories } from "@/components/ext-repositories";
import { repoAdd, repoAuth, repoList, repoRefresh, repoSaveAuth, type ExtListItem, type RepoAuth, type RepoView } from "@/lib/api";
import { compareVersions } from "@/lib/ext";

vi.mock("@/lib/api", () => ({
  repoList: vi.fn(),
  repoAdd: vi.fn(),
  repoRefresh: vi.fn(),
  repoRemove: vi.fn(),
  repoAuth: vi.fn(),
  repoSaveAuth: vi.fn(),
  repoClearAuth: vi.fn(),
}));
vi.mock("sonner", () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

const entry = (id: string, name: string, version: string, sharedAuth = true) => ({
  id,
  name,
  version,
  description: `Tải raw từ ${name}`,
  developer: { name: "dev", url: null, email: null },
  license: "MIT",
  homepage: null,
  hosts: [`${name.toLowerCase()}-api.khotangtruyen.org`],
  permissions: [],
  authRequired: true,
  sharedAuth,
  url: `https://ext.test/extensions/${id}/${version}.zip`,
  sha256: "a".repeat(64),
  size: 1910,
  icon: null,
  updatedAt: null,
});

const repo: RepoView = {
  id: "org.khotangtruyen.official",
  url: "https://ext.test/index.json",
  name: "VNCVT — extension chính thức",
  description: "Extension chính thức",
  maintainer: { name: "thiendiepdt", url: null, email: null },
  homepage: null,
  hasAuth: true,
  authRequired: true,
  extensions: [entry("org.khotangtruyen.qimao", "Qimao", "1.1.0"), entry("org.khotangtruyen.ixdzs", "Ixdzs", "1.0.0"), entry("org.khotangtruyen.xsw", "Xsw", "1.0.0", false)],
  warnings: ["extensions[3] (org.x): thiếu sha256, bỏ qua"],
  addedAt: 1,
  refreshedAt: 1,
  error: null,
  authFilled: false,
};

const installed = (id: string, version: string) => ({ id, version }) as ExtListItem;

const auth: RepoAuth = {
  id: repo.id,
  name: repo.name,
  auth: { required: true, schemes: [{ id: "apiKey", label: "API key", fields: [{ key: "apiKey", label: "API key (userId:secretKey)", kind: "secret" }] }], scheme: null, values: {}, filled: [] },
  secretStore: "keyring",
};

describe("ExtRepositories", () => {
  beforeEach(() => {
    vi.mocked(repoList).mockReset().mockResolvedValue([]);
    vi.mocked(repoAdd).mockReset().mockResolvedValue([repo]);
    vi.mocked(repoRefresh).mockReset().mockResolvedValue({ repositories: [repo], errors: [] });
    vi.mocked(repoAuth).mockReset().mockResolvedValue(auth);
    vi.mocked(repoSaveAuth).mockReset().mockResolvedValue({ ...auth, auth: { ...auth.auth, scheme: "apiKey", filled: ["apiKey"] } });
  });

  it("thêm kho bằng link rồi hiện danh sách có trong kho với trạng thái", async () => {
    const user = userEvent.setup();
    const onInstall = vi.fn(() => Promise.resolve());
    const onUpdateAll = vi.fn(() => Promise.resolve());
    render(<ExtRepositories installed={[installed("org.khotangtruyen.qimao", "1.0.0"), installed("org.khotangtruyen.ixdzs", "1.0.0")]} busy={false} onInstall={onInstall} onUpdateAll={onUpdateAll} />);
    expect(await screen.findByText("Chưa thêm kho nào.")).toBeInTheDocument();
    await user.type(screen.getByLabelText("Link kho (index.json, https)"), "https://ext.test/index.json");
    await user.click(screen.getByRole("button", { name: /Thêm kho/ }));
    expect(repoAdd).toHaveBeenCalledWith("https://ext.test/index.json");

    const row = await screen.findByRole("listitem", { name: `Kho ${repo.name}` });
    expect(row).toHaveTextContent("3 extension · thiendiepdt");
    expect(row).toHaveTextContent("thiếu sha256, bỏ qua");

    const qimao = screen.getByRole("listitem", { name: "Trong kho: Qimao" });
    expect(qimao).toHaveTextContent("Có bản mới (đang 1.0.0)");
    expect(qimao).toHaveTextContent("API key của kho");
    expect(within(screen.getByRole("listitem", { name: "Trong kho: Ixdzs" })).getByText("Đã cài")).toBeInTheDocument();
    const xsw = screen.getByRole("listitem", { name: "Trong kho: Xsw" });
    expect(within(xsw).queryByText("API key của kho")).toBeNull();
    await user.click(within(xsw).getByRole("button", { name: "Cài" }));
    expect(onInstall).toHaveBeenCalledWith({ repoId: repo.id, entry: repo.extensions[2] });

    await user.click(screen.getByRole("button", { name: "Cập nhật hết (1)" }));
    expect(onUpdateAll).toHaveBeenCalledWith([{ repoId: repo.id, entry: repo.extensions[0] }]);

    await user.type(screen.getByLabelText("Lọc extension trong kho"), "ixdzs-api");
    expect(screen.queryByRole("listitem", { name: "Trong kho: Qimao" })).toBeNull();
    expect(screen.getByRole("listitem", { name: "Trong kho: Ixdzs" })).toBeInTheDocument();
  });

  it("API key của kho nhập một lần; lưu xong đóng hộp và dòng kho báo đã nhập", async () => {
    const user = userEvent.setup();
    vi.mocked(repoList).mockResolvedValueOnce([repo]).mockResolvedValue([{ ...repo, authFilled: true }]);
    render(<ExtRepositories installed={[]} busy={false} onInstall={() => Promise.resolve()} onUpdateAll={() => Promise.resolve()} />);
    const row = await screen.findByRole("listitem", { name: `Kho ${repo.name}` });
    expect(row).toHaveTextContent("Chưa nhập API key");
    await user.click(within(row).getByRole("button", { name: /API key của kho/ }));
    const dialog = await screen.findByRole("dialog");
    expect(dialog).toHaveTextContent(`API key của kho — ${repo.name}`);
    await user.type(within(dialog).getByLabelText("API key (userId:secretKey)"), "u1:s1");
    await user.click(within(dialog).getByRole("button", { name: "Lưu auth" }));
    expect(repoSaveAuth).toHaveBeenCalledWith(repo.id, "apiKey", { apiKey: "u1:s1" });
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    expect(await within(row).findByText("Đã nhập API key")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: /^Làm mới$/ }));
    expect(repoRefresh).toHaveBeenCalledWith(repo.id);
  });

  it("so phiên bản", () => {
    expect(compareVersions("1.10.0", "1.9.0")).toBeGreaterThan(0);
    expect(compareVersions("1.0.0", "1.0.0")).toBe(0);
    expect(compareVersions("1.0.0-beta", "1.0.0")).toBeLessThan(0);
    expect(compareVersions("2.0.0", "10.0.0")).toBeLessThan(0);
  });
});
