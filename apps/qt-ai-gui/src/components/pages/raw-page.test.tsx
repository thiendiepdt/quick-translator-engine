import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { RawPage } from "@/components/pages/raw-page";
import { extAccountCheck, extLogin, rawClear, rawDownload, rawFiles, rawInfo, rawSave, rawToc, rescanStory, revealFolder, type ExtConfig, type RawInfo, type RawToc } from "@/lib/api";
import { resetRawChecks, useRawNew } from "@/store/raw-check";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({
  extLogin: vi.fn(),
  extAccountCheck: vi.fn(),
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
  rawClear: vi.fn(),
  rawFiles: vi.fn(),
  revealFolder: vi.fn(),
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
    vi.mocked(rawClear).mockReset().mockResolvedValue({ moved: 1, backup: "D:\\lib\\demo\\work\\raw-backup\\xoa-1" });
    useStoryStore.setState({ root: "D:\\lib\\demo", setSnapshot: vi.fn(), setPage: vi.fn() } as never);
    resetRawChecks();
  });

  it("mục lục có trạng thái; Tải chương mới bỏ chương khoá; tải lẻ và tải đã chọn gửi đúng id", async () => {
    const user = userEvent.setup();
    render(<RawPage />);
    expect(await screen.findByText(/Đang theo link nguồn/)).toBeInTheDocument();
    expect(screen.getByText("Demo")).toBeInTheDocument();
    // Đủ cài đặt → vào trang là tự tải mục lục; số chương mới cho thanh bên.
    const third = await screen.findByRole("listitem", { name: "Mục 3" });
    expect(rawToc).toHaveBeenCalledTimes(1);
    expect(useRawNew.getState().counts["D:\\lib\\demo"]).toBe(2);
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

  it("vào trang là hiện khung ngay: chưa đọc xong cài đặt / chưa tải xong mục lục thì chỉ chỗ đó quay", async () => {
    let giveInfo: (value: RawInfo) => void = () => undefined;
    vi.mocked(rawInfo).mockReturnValue(new Promise<RawInfo>((resolve) => (giveInfo = resolve)));
    let giveToc: (value: RawToc) => void = () => undefined;
    vi.mocked(rawToc).mockReturnValue(new Promise<RawToc>((resolve) => (giveToc = resolve)));
    render(<RawPage />);
    expect(screen.getByRole("heading", { name: "Tải raw" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Tải mục lục/ })).toBeDisabled();
    expect(screen.getByRole("status", { name: "Đang đọc cài đặt tải raw…" })).toBeInTheDocument();

    giveInfo(info);
    expect(await screen.findByRole("status", { name: "Đang tải mục lục…" })).toBeInTheDocument();
    expect(screen.getByLabelText("Link tải raw")).toBeInTheDocument();
    giveToc(toc);
    expect(await screen.findByRole("listitem", { name: "Mục 1" })).toBeInTheDocument();
    expect(screen.queryByRole("status")).not.toBeInTheDocument();
  });

  it("tab Raw trong máy: file trong raw/, khớp mục số mấy, trạng thái dịch, lọc, Xem nhảy sang trang Dịch", async () => {
    const user = userEvent.setup();
    const select = vi.fn();
    const setPage = vi.fn();
    useStoryStore.setState({
      select,
      setPage,
      snapshot: { chapters: [{ id: "chuong-0001", status: "done", reviewRound: 0, reason: null, warnings: [] }] },
    } as never);
    vi.mocked(rawFiles)
      .mockReset()
      .mockResolvedValueOnce({
        dir: "D:\\lib\\demo\\raw",
        files: [
          { id: "chuong-0001", title: "第1章 起", chars: 3120 },
          { id: "le", title: "番外", chars: null },
        ],
      })
      .mockResolvedValue({ dir: "D:\\lib\\demo\\raw", files: [] });
    render(<RawPage />);
    await screen.findByRole("listitem", { name: "Mục 1" });
    expect(rawFiles).not.toHaveBeenCalled();

    await user.click(screen.getByRole("tab", { name: "Raw trong máy" }));
    const first = await screen.findByRole("listitem", { name: "File chuong-0001" });
    expect(within(first).getByText("mục 1")).toBeInTheDocument();
    expect(within(first).getByText("3.120 chữ")).toBeInTheDocument();
    expect(within(first).getByText("Xong")).toBeInTheDocument();
    const odd = screen.getByRole("listitem", { name: "File le" });
    expect(within(odd).getByText("không đọc được")).toBeInTheDocument();
    expect(within(odd).getByRole("button", { name: "Xem le ở trang Dịch" })).toBeDisabled();
    expect(screen.getByText(/2 file · 3\.120 chữ/)).toBeInTheDocument();

    // Mới → cũ mặc định: file sau lên trước.
    expect(screen.getAllByRole("listitem").map((li) => li.getAttribute("aria-label"))).toEqual(["File le", "File chuong-0001"]);
    await user.type(screen.getByLabelText("Lọc raw theo tên file / tiêu đề"), "起");
    expect(screen.queryByRole("listitem", { name: "File le" })).not.toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: /Mở thư mục raw/ }));
    expect(revealFolder).toHaveBeenCalledWith("D:\\lib\\demo\\raw");
    await user.click(within(first).getByRole("button", { name: "Xem chuong-0001 ở trang Dịch" }));
    expect(select).toHaveBeenCalledWith("chuong-0001");
    expect(setPage).toHaveBeenCalledWith("translate");
  });

  it("mục lục mặc định xếp mới đến cũ, bấm để đổi chiều", async () => {
    const user = userEvent.setup();
    render(<RawPage />);
    const order = () => screen.getAllByRole("listitem", { name: /^Mục \d+$/ }).map((li) => li.getAttribute("aria-label"));
    await waitFor(() => expect(order()).toEqual(["Mục 4", "Mục 3", "Mục 2", "Mục 1"]));
    await user.click(screen.getByRole("button", { name: /bấm để xếp cũ đến mới/ }));
    expect(order()).toEqual(["Mục 1", "Mục 2", "Mục 3", "Mục 4"]);
    expect(screen.getByRole("button", { name: /bấm để xếp mới đến cũ/ })).toHaveTextContent("Cũ → mới");
  });

  it("chọn nhiều bằng Shift, bấm tên chương, ô chọn / bỏ chọn hết", async () => {
    const user = userEvent.setup();
    render(<RawPage />);
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

    const all = screen.getByLabelText<HTMLInputElement>("Chọn / bỏ chọn hết");
    expect(all.indeterminate).toBe(true);
    await user.click(all);
    expect(screen.getByRole("button", { name: "Tải đã chọn (4)" })).toBeEnabled();
    expect(all).toBeChecked();
    await user.click(all);
    expect(screen.getByRole("button", { name: "Tải đã chọn (0)" })).toBeDisabled();
  });

  it("chọn theo khoảng mục số, đảo ngược cũng được", async () => {
    const user = userEvent.setup();
    render(<RawPage />);
    await screen.findByRole("listitem", { name: "Mục 1" });
    await user.type(screen.getByLabelText("Từ mục số"), "4");
    await user.type(screen.getByLabelText("Đến mục số"), "2");
    await user.click(screen.getByRole("button", { name: "Chọn khoảng" }));
    expect(screen.getByRole("button", { name: "Tải đã chọn (3)" })).toBeEnabled();
    expect(screen.getByLabelText("Chọn 第1章 起")).not.toBeChecked();
    expect(screen.getByLabelText("Chọn 第3章 VIP")).toBeChecked();
  });

  it("dọn raw cũ hỏi lại, chuyển raw đi rồi tải lại mục lục", async () => {
    const user = userEvent.setup();
    const confirm = vi.spyOn(window, "confirm").mockReturnValueOnce(false).mockReturnValueOnce(true);
    render(<RawPage />);
    await screen.findByRole("listitem", { name: "Mục 1" });
    await user.click(screen.getByRole("button", { name: /Dọn raw cũ/ }));
    expect(rawClear).not.toHaveBeenCalled();
    await user.click(screen.getByRole("button", { name: /Dọn raw cũ/ }));
    expect(confirm).toHaveBeenLastCalledWith(expect.stringContaining("work/raw-backup/xoa-"));
    await waitFor(() => expect(rawClear).toHaveBeenCalledWith("D:\\lib\\demo"));
    await waitFor(() => expect(rawToc).toHaveBeenCalledTimes(2));
    expect(rescanStory).toHaveBeenCalledWith("D:\\lib\\demo");
  });

  it("chip biến chèn vào chỗ con trỏ của ô mẫu; chưa bấm vào ô thì chèn cuối", async () => {
    vi.mocked(rawInfo).mockResolvedValue({ ...info, fileTemplate: "abc", effectiveTemplate: "abc" });
    const user = userEvent.setup();
    render(<RawPage />);
    const template = await screen.findByLabelText<HTMLInputElement>("Mẫu tên file");
    await user.click(screen.getByRole("button", { name: /^Chèn \{chuong:4\}/ }));
    expect(template).toHaveValue("abc{chuong:4}");
    // Con trỏ về sau biến vừa chèn.
    await waitFor(() => expect(template.selectionStart).toBe(13));

    template.setSelectionRange(1, 2);
    await user.click(screen.getByRole("button", { name: /^Chèn \{chapterTitle\}/ }));
    expect(template).toHaveValue("a{chapterTitle}c{chuong:4}");
    expect(screen.getByRole("button", { name: "Lưu" })).toBeEnabled();
  });

  it("mẫu có sẵn đặt cả mẫu; STV Web Downloader lấy mã truyện từ link, link không phải STV thì tắt", async () => {
    const user = userEvent.setup();
    render(<RawPage />);
    const template = await screen.findByLabelText<HTMLInputElement>("Mẫu tên file");
    expect(screen.getByRole("button", { name: "STV Web Downloader" })).toBeDisabled();
    await user.click(screen.getByRole("button", { name: "Chinese Web Downloader" }));
    expect(template).toHaveValue("{chuong:4}_{chapter-id}");
    await user.click(screen.getByRole("button", { name: "DTH" }));
    expect(template).toHaveValue("{chuong:4} - {chapterTitle}");
    expect(screen.getByRole("button", { name: "DTH" })).toHaveAttribute("aria-pressed", "true");

    const link = screen.getByLabelText("Link tải raw");
    await user.clear(link);
    await user.type(link, "https://sangtacviet.vip/truyen/qidian/1/730611/");
    await user.click(screen.getByRole("button", { name: "STV Web Downloader" }));
    expect(template).toHaveValue("730611_{chuong:4}_{chapterTitle}");
  });

  it("mẫu sai chặn Lưu; sửa link thì phải Lưu trước khi tải", { timeout: 15_000 }, async () => {
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
    expect(rawToc).not.toHaveBeenCalled();
    unmount();

    vi.mocked(rawInfo).mockResolvedValue(info);
    vi.mocked(rawToc).mockResolvedValue({ ...toc, mismatches: [{ index: 1, tocTitle: "序章", fileId: "chuong-0001", fileTitle: "第1章 起" }] });
    render(<RawPage />);
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
      session: { loggedIn: false, expiresAt: null, hasData: false, account: null, guest: false },
      secretStore: "keyring",
      cacheBytes: 0,
      sharedAuth: null,
      account: null,
    };
    vi.mocked(rawInfo).mockResolvedValue({
      ...info,
      extConfig: config,
      storySettings: [
        { key: "proxy", label: "Proxy riêng", kind: "text", choices: [], default: null, scope: "story", value: "p1", filled: false },
        { key: "vip", label: "Tải chương VIP", kind: "boolean", choices: [], default: false, scope: "story", value: null, filled: false },
      ],
    });
    vi.mocked(extLogin).mockReset().mockResolvedValue({ config: { ...config, session: { loggedIn: true, expiresAt: null, hasData: true, account: null, guest: false } }, hasLogin: true });
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

  it("extension có tài khoản trang nguồn: dòng API key không có nút Đăng nhập, dòng tài khoản thì có", async () => {
    const config: ExtConfig = {
      id: ext.id,
      name: "Demo",
      auth: { required: true, schemes: [{ id: "apiKey", label: "API key", fields: [{ key: "apiKey", label: "API key", kind: "secret" }] }], scheme: "apiKey", values: {}, filled: ["apiKey"] },
      settings: [],
      session: { loggedIn: true, expiresAt: null, hasData: true, account: null, guest: true },
      secretStore: "keyring",
      cacheBytes: 0,
      sharedAuth: null,
      account: { title: "Tài khoản QQ阅读", required: false, help: null },
    };
    vi.mocked(rawInfo).mockResolvedValue({ ...info, extConfig: config });
    vi.mocked(extAccountCheck).mockReset().mockResolvedValue({ config, status: { loggedIn: false, name: null, message: null, checked: true }, checkedAt: 0 });
    vi.mocked(extLogin).mockReset().mockResolvedValue({ config: { ...config, session: { ...config.session, guest: false, account: "1234567" } }, hasLogin: true });
    const user = userEvent.setup();
    render(<RawPage />);
    expect(await screen.findByLabelText("Trạng thái auth")).toHaveTextContent("Đã nhập auth: API key");
    expect(await screen.findByLabelText("Trạng thái Tài khoản QQ阅读")).toHaveTextContent("chưa đăng nhập");
    expect(screen.getAllByRole("button", { name: /Đăng nhập/ })).toHaveLength(1);

    vi.mocked(extAccountCheck).mockResolvedValue({ config, status: { loggedIn: true, name: "1234567", message: null, checked: true }, checkedAt: 0 });
    await user.click(screen.getByRole("button", { name: "Đăng nhập Tài khoản QQ阅读" }));
    expect(extLogin).toHaveBeenCalledWith("D:\\lib\\demo", ext.id);
    await waitFor(() => expect(screen.getByLabelText("Trạng thái Tài khoản QQ阅读")).toHaveTextContent("đã đăng nhập: 1234567"));
    expect(extAccountCheck).toHaveBeenCalledTimes(2);
  });

  it("auth bắt buộc chưa nhập thì báo và khoá Đăng nhập", async () => {
    const config: ExtConfig = {
      id: ext.id,
      name: "Demo",
      auth: { required: true, schemes: [{ id: "apiKey", label: "API key", fields: [{ key: "apiKey", label: "API key", kind: "secret" }] }], scheme: null, values: {}, filled: [] },
      settings: [],
      session: { loggedIn: false, expiresAt: null, hasData: false, account: null, guest: false },
      secretStore: "keyring",
      cacheBytes: 0,
      sharedAuth: null,
      account: null,
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
