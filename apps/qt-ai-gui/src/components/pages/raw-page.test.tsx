import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { RawPage } from "@/components/pages/raw-page";
import { extLogin, rawDownload, rawInfo, rawSave, rawToc, rescanStory, type ExtConfig, type RawInfo, type RawToc } from "@/lib/api";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({
  extLogin: vi.fn(),
  extSaveAuth: vi.fn(),
  extClearAuth: vi.fn(),
  extSaveSettings: vi.fn(),
  extLogout: vi.fn(),
  extClearCache: vi.fn(),
  rawInfo: vi.fn(),
  rawSave: vi.fn(),
  rawToc: vi.fn(),
  rawDownload: vi.fn(),
  rawDownloadCancel: vi.fn(),
  onRawProgress: vi.fn(() => Promise.resolve(() => undefined)),
  rescanStory: vi.fn(),
}));
vi.mock("sonner", () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

const ext = {
  id: "io.github.dev.demo",
  name: "Demo",
  version: "1.0.0",
  description: "Mẫu",
  developer: { name: "dev", url: null, email: null },
  license: "MIT",
  hosts: ["demo.test"],
  dev: false,
};

const info: RawInfo = {
  rawUrl: "https://demo.test/book/1",
  rawUrlIsDefault: true,
  sourceUrl: "https://demo.test/book/1",
  extension: ext,
  extensionChoice: "",
  candidates: [ext],
  installed: 1,
  loadErrors: [],
  fileTemplate: "",
  defaultTemplate: "{chuong:4}_{chapter-id}",
  guessedTemplate: "chuong-{chuong:4}",
  effectiveTemplate: "chuong-{chuong:4}",
  extensionsDir: "C:\\app\\extensions",
  extConfig: null,
  storySettings: [],
};

const toc: RawToc = {
  rows: [
    { index: 1, id: "1", title: "第1章 起", locked: false, file: "chuong-0001", status: "have" },
    { index: 2, id: "2", title: "第2章 承", locked: false, file: null, status: "missing" },
    { index: 3, id: "3", title: "第3章 VIP", locked: true, file: null, status: "locked" },
    { index: 4, id: "4", title: "第4章 合", locked: false, file: null, status: "missing" },
  ],
  by: "order",
  lastIndex: 1,
  mismatches: [],
  unmatchedFiles: 0,
  template: "chuong-{chuong:4}",
  nextName: "chuong-0002",
  nextSortsAfter: true,
};

describe("RawPage", () => {
  beforeEach(() => {
    vi.mocked(rawInfo).mockReset().mockResolvedValue(info);
    vi.mocked(rawSave).mockReset().mockImplementation((_root, s) => Promise.resolve({ ...info, rawUrl: s.rawUrl, rawUrlIsDefault: false, fileTemplate: s.fileTemplate }));
    vi.mocked(rawToc).mockReset().mockResolvedValue(toc);
    vi.mocked(rawDownload).mockReset().mockResolvedValue({ saved: [2, 4], short: [4], locked: [], failed: [], cancelled: false });
    vi.mocked(rescanStory).mockReset().mockResolvedValue({} as never);
    useStoryStore.setState({ root: "D:\\lib\\demo", setSnapshot: vi.fn(), setPage: vi.fn() } as never);
  });

  it("mục lục có trạng thái; Tải chương mới bỏ chương khoá; tải lẻ và tải đã chọn gửi đúng id", async () => {
    const user = userEvent.setup();
    render(<RawPage />);
    expect(await screen.findByText(/Đang theo link nguồn/)).toBeInTheDocument();
    expect(screen.getByText("Demo")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: /Tải mục lục/ }));
    expect(rawToc).toHaveBeenCalledWith("D:\\lib\\demo", undefined);
    const third = await screen.findByRole("listitem", { name: "Mục 3" });
    expect(within(third).getByText("Khoá")).toBeInTheDocument();
    expect(within(screen.getByRole("listitem", { name: "Mục 1" })).getByText("Đã có")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Tải chương mới (2)" }));
    expect(rawDownload).toHaveBeenLastCalledWith("D:\\lib\\demo", "new", [], 1);
    expect(await screen.findByLabelText("Kết quả tải")).toHaveTextContent("Tải được 2 chương · 1 chương ngắn (vẫn ghi): 4");
    expect(rescanStory).toHaveBeenCalledWith("D:\\lib\\demo");

    await user.click(screen.getByRole("button", { name: "Tải lại chương 1" }));
    expect(rawDownload).toHaveBeenLastCalledWith("D:\\lib\\demo", "single", ["1"], 1);

    await user.click(screen.getByLabelText("Chọn 第3章 VIP"));
    await user.click(screen.getByLabelText("Chọn 第4章 合"));
    await user.click(screen.getByRole("button", { name: "Tải đã chọn (2)" }));
    expect(rawDownload).toHaveBeenLastCalledWith("D:\\lib\\demo", "selected", ["3", "4"], 1);
  });

  it("mục lục mặc định xếp mới đến cũ, bấm để đổi chiều", async () => {
    const user = userEvent.setup();
    render(<RawPage />);
    await user.click(await screen.findByRole("button", { name: /Tải mục lục/ }));
    const order = () => screen.getAllByRole("listitem", { name: /^Mục \d+$/ }).map((li) => li.getAttribute("aria-label"));
    await waitFor(() => expect(order()).toEqual(["Mục 4", "Mục 3", "Mục 2", "Mục 1"]));
    await user.click(screen.getByRole("button", { name: /bấm để xếp cũ đến mới/ }));
    expect(order()).toEqual(["Mục 1", "Mục 2", "Mục 3", "Mục 4"]);
    expect(screen.getByRole("button", { name: /bấm để xếp mới đến cũ/ })).toHaveTextContent("Cũ → mới");
  });

  it("chọn nhiều bằng Shift, bấm tên chương, Chọn hết / Bỏ chọn", async () => {
    const user = userEvent.setup();
    render(<RawPage />);
    await user.click(await screen.findByRole("button", { name: /Tải mục lục/ }));
    await user.click(await screen.findByLabelText("Chọn 第1章 起"));
    await user.keyboard("{Shift>}");
    await user.click(screen.getByLabelText("Chọn 第4章 合"));
    await user.keyboard("{/Shift}");
    expect(screen.getByRole("button", { name: "Tải đã chọn (4)" })).toBeEnabled();

    await user.keyboard("{Shift>}");
    await user.click(screen.getByText("第2章 承"));
    await user.keyboard("{/Shift}");
    expect(screen.getByRole("button", { name: "Tải đã chọn (1)" })).toBeEnabled();
    expect(screen.getByLabelText("Chọn 第1章 起")).toBeChecked();

    await user.click(screen.getByRole("button", { name: "Chọn hết" }));
    expect(screen.getByRole("button", { name: "Tải đã chọn (4)" })).toBeEnabled();
    await user.click(screen.getByRole("button", { name: "Bỏ chọn" }));
    expect(screen.getByRole("button", { name: "Tải đã chọn (0)" })).toBeDisabled();
  });

  it("mẫu sai chặn Lưu; sửa link thì phải Lưu trước khi tải", async () => {
    vi.mocked(rawInfo).mockResolvedValue({ ...info, fileTemplate: "{chuong:4}_{chapter-id}", effectiveTemplate: "{chuong:4}_{chapter-id}" });
    const user = userEvent.setup();
    render(<RawPage />);
    const template = await screen.findByLabelText("Mẫu tên file");
    await user.clear(template);
    await user.type(template, "{{chapterTitle}");
    expect(screen.getByText(/phải có \{chuong\}/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Lưu" })).toBeDisabled();
    await user.click(screen.getByRole("button", { name: /Dùng mẫu theo tên cũ/ }));
    expect(template).toHaveValue("chuong-{chuong:4}");

    const link = screen.getByLabelText("Link tải raw");
    await user.clear(link);
    await user.type(link, "https://demo.test/book/2");
    expect(screen.getByRole("button", { name: /Tải mục lục/ })).toBeDisabled();
    await user.click(screen.getByRole("button", { name: "Lưu" }));
    expect(rawSave).toHaveBeenCalledWith("D:\\lib\\demo", { rawUrl: "https://demo.test/book/2", extension: "io.github.dev.demo", fileTemplate: "chuong-{chuong:4}" });
    expect(screen.getByRole("button", { name: /Tải mục lục/ })).toBeEnabled();
  });

  it("chưa có extension thì chỉ chỗ đặt extension; lệch tiêu đề thì hỏi mốc", async () => {
    vi.mocked(rawInfo).mockResolvedValue({ ...info, extension: null, candidates: [], installed: 0, loadErrors: ["hong: manifest: id …"] });
    const user = userEvent.setup();
    const { unmount } = render(<RawPage />);
    expect(await screen.findByText(/Chưa có extension nào khớp link này/)).toHaveTextContent("0 extension đang bật");
    await user.click(screen.getByRole("button", { name: /Cài extension ở Cài đặt/ }));
    expect(useStoryStore.getState().setPage).toHaveBeenCalledWith("settings");
    expect(screen.getByText(/Không nạp được hong/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Tải mục lục/ })).toBeDisabled();
    unmount();

    vi.mocked(rawInfo).mockResolvedValue(info);
    vi.mocked(rawToc).mockResolvedValue({ ...toc, mismatches: [{ index: 1, tocTitle: "序章", fileId: "chuong-0001", fileTitle: "第1章 起" }] });
    render(<RawPage />);
    await user.click(await screen.findByRole("button", { name: /Tải mục lục/ }));
    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent("Mục 1: 序章 ↔ file chuong-0001: 第1章 起");
    const anchor = within(alert).getByLabelText("Chương cuối đã có là mục số");
    await user.clear(anchor);
    await user.type(anchor, "2");
    await user.click(within(alert).getByRole("button", { name: "Khớp lại" }));
    expect(rawToc).toHaveBeenLastCalledWith("D:\\lib\\demo", 2);
  });

  it("trạng thái auth, nút Đăng nhập; cài đặt của truyện lưu cùng nút Lưu", async () => {
    const config: ExtConfig = {
      id: ext.id,
      name: "Demo",
      auth: { required: true, schemes: [{ id: "apiKey", label: "API key", fields: [{ key: "apiKey", label: "API key", kind: "secret" }] }], scheme: "apiKey", values: {}, filled: ["apiKey"] },
      settings: [],
      session: { loggedIn: false, expiresAt: null, hasData: false },
      secretStore: "keyring",
      cacheBytes: 0,
      sharedAuth: null,
    };
    vi.mocked(rawInfo).mockResolvedValue({
      ...info,
      extConfig: config,
      storySettings: [
        { key: "proxy", label: "Proxy riêng", kind: "text", choices: [], default: null, scope: "story", value: "p1", filled: false },
        { key: "vip", label: "Tải chương VIP", kind: "boolean", choices: [], default: false, scope: "story", value: null, filled: false },
      ],
    });
    vi.mocked(extLogin).mockReset().mockResolvedValue({ config: { ...config, session: { loggedIn: true, expiresAt: null, hasData: true } }, hasLogin: true });
    const user = userEvent.setup();
    render(<RawPage />);
    expect(await screen.findByLabelText("Trạng thái auth")).toHaveTextContent("Đã nhập auth: API key");
    await user.click(screen.getByRole("button", { name: "Đăng nhập" }));
    expect(extLogin).toHaveBeenCalledWith("D:\\lib\\demo", ext.id);
    expect(await screen.findByText("Đã đăng nhập (API key)")).toBeInTheDocument();

    const proxy = screen.getByLabelText("Proxy riêng");
    expect(proxy).toHaveValue("p1");
    expect(screen.getByRole("button", { name: "Lưu" })).toBeDisabled();
    await user.clear(proxy);
    await user.type(proxy, "p2");
    expect(screen.getByRole("button", { name: /Tải mục lục/ })).toBeDisabled();
    await user.click(screen.getByRole("button", { name: "Lưu" }));
    expect(rawSave).toHaveBeenCalledWith("D:\\lib\\demo", {
      rawUrl: info.rawUrl,
      extension: ext.id,
      fileTemplate: "",
      storySettings: { extension: ext.id, values: { proxy: "p2", vip: null } },
    });
  });

  it("auth bắt buộc chưa nhập thì báo và khoá Đăng nhập", async () => {
    const config: ExtConfig = {
      id: ext.id,
      name: "Demo",
      auth: { required: true, schemes: [{ id: "apiKey", label: "API key", fields: [{ key: "apiKey", label: "API key", kind: "secret" }] }], scheme: null, values: {}, filled: [] },
      settings: [],
      session: { loggedIn: false, expiresAt: null, hasData: false },
      secretStore: "keyring",
      cacheBytes: 0,
      sharedAuth: null,
    };
    vi.mocked(rawInfo).mockResolvedValue({ ...info, extConfig: config });
    const user = userEvent.setup();
    render(<RawPage />);
    expect(await screen.findByLabelText("Trạng thái auth")).toHaveTextContent("Chưa nhập auth — extension này bắt buộc");
    expect(screen.getByRole("button", { name: "Đăng nhập" })).toBeDisabled();
    await user.click(screen.getByRole("button", { name: /Auth và cài đặt/ }));
    expect(await screen.findByRole("dialog")).toHaveTextContent("Auth và cài đặt — Demo");
  });
});
