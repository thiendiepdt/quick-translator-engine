import { act, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import App from "@/app";
import { agyStatus, appConfigGet } from "@/lib/api";
import { appConfigSchema } from "@/lib/schema";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({
  agyStatus: vi.fn(),
  appConfigGet: vi.fn(),
  appConfigSet: vi.fn(),
  pickAgyFile: vi.fn(),
}));
vi.mock("@/hooks/use-session-events", () => ({ useSessionEvents: () => undefined }));
vi.mock("@/hooks/use-theme", () => ({ useThemeSync: () => undefined }));
vi.mock("@/components/story-picker", () => ({ StoryPicker: () => <div>PICKER</div> }));
vi.mock("@/components/app-rail", () => ({ AppRail: () => null }));
vi.mock("@/components/story-sidebar", () => ({ StorySidebar: () => null }));
vi.mock("@/components/pages/translate-page", () => ({ TranslatePage: () => null }));
vi.mock("@/components/pages/story-page", () => ({ StoryPage: () => null }));
vi.mock("@/components/pages/export-page", () => ({ ExportPage: () => null }));
vi.mock("@/components/pages/settings-page", () => ({ SettingsPage: () => null }));

const agyConfig = appConfigSchema.parse({ agyPath: null, model: null, maxSessions: 50, recent: [], engine: "agy" });
const found = { found: true, path: "/usr/bin/agy", version: "1.2.0", models: ["m"], message: null };

describe("App · dò agy lúc khởi động", () => {
  beforeEach(() => {
    useStoryStore.setState({ config: undefined, agy: undefined, screen: "picker" });
    vi.mocked(appConfigGet).mockReset().mockResolvedValue(agyConfig);
    vi.mocked(agyStatus).mockReset();
  });

  it("agy_status chậm thì chỉ dò MỘT lần, không dò chồng dù setConfig đổi object", async () => {
    let finish: (value: typeof found) => void = () => undefined;
    vi.mocked(agyStatus).mockReturnValue(new Promise((resolve) => (finish = resolve)));
    render(<App />);
    await screen.findByText(/Đang kiểm tra agy/);
    await act(() => new Promise((r) => setTimeout(r, 150)));
    expect(agyStatus).toHaveBeenCalledTimes(1);
    expect(appConfigGet).toHaveBeenCalledTimes(1);
    finish(found);
    await screen.findByText("PICKER");
    expect(agyStatus).toHaveBeenCalledTimes(1);
  });

  it("dò agy lỗi thì hiện màn Chưa thấy agy kèm lý do, không xoay mãi", async () => {
    vi.mocked(agyStatus).mockRejectedValue(new Error("agy models thoát mã 1"));
    render(<App />);
    await waitFor(() => expect(useStoryStore.getState().agy?.found).toBe(false));
    expect(useStoryStore.getState().agy?.message).toBe("agy models thoát mã 1");
    expect(agyStatus).toHaveBeenCalledTimes(1);
  });
});
