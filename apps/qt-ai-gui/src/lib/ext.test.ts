import { describe, expect, it } from "vitest";

import type { ExtConfig, ExtSetting } from "@/lib/api";
import { authStatus, changedValues, formatBytes, initialDisplay, storyValues, toValue } from "@/lib/ext";

const field = (over: Partial<ExtSetting>): ExtSetting => ({ key: "k", label: "K", kind: "text", choices: [], default: null, scope: "story", value: null, filled: false, ...over });

const config = (over: Partial<ExtConfig>): ExtConfig => ({
  id: "x",
  name: "X",
  auth: { required: true, schemes: [{ id: "apiKey", label: "API key", fields: [] }], scheme: null, values: {}, filled: [] },
  settings: [],
  session: { loggedIn: false, expiresAt: null, hasData: false },
  secretStore: "keyring",
  cacheBytes: 0,
  sharedAuth: null,
  ...over,
});

describe("ext", () => {
  it("ô trống là mặc định, secret trống là giữ, boolean theo mặc định", () => {
    expect(initialDisplay(field({ value: 3, kind: "number" }))).toBe("3");
    expect(initialDisplay(field({ kind: "secret", value: "lộ" }))).toBe("");
    expect(initialDisplay(field({ kind: "boolean", default: true }))).toBe(true);
    expect(toValue(field({}), " ")).toBeNull();
    expect(toValue(field({ kind: "secret" }), "")).toBe("");
    expect(toValue(field({ kind: "number" }), "12")).toBe("12");
  });

  it("cài đặt chung gửi trường đổi; cài đặt truyện gửi đủ bộ", () => {
    const fields = [field({ key: "proxy", value: "p1" }), field({ key: "vip", kind: "boolean" }), field({ key: "n", kind: "number" })];
    expect(changedValues(fields, {})).toEqual({});
    expect(changedValues(fields, { proxy: "p1", vip: true })).toEqual({ vip: true });
    expect(storyValues(fields, { proxy: "" })).toEqual({ proxy: null, vip: null, n: null });
    expect(storyValues(fields, { n: "4" })).toEqual({ proxy: "p1", vip: null, n: "4" });
  });

  it("dòng trạng thái auth", () => {
    expect(authStatus(config({}))).toBe("Chưa nhập auth — extension này bắt buộc");
    const filled = config({ auth: { required: false, schemes: [{ id: "apiKey", label: "API key", fields: [] }], scheme: "apiKey", values: {}, filled: ["apiKey"] } });
    expect(authStatus(filled)).toBe("Đã nhập auth: API key");
    expect(authStatus({ ...filled, session: { loggedIn: true, expiresAt: null, hasData: true } })).toBe("Đã đăng nhập (API key)");
    expect(authStatus({ ...filled, session: { loggedIn: true, expiresAt: new Date(2026, 10, 14, 9, 5).getTime(), hasData: true } })).toBe("Đã đăng nhập (API key), hết hạn 09:05 14/11");
    expect(authStatus(config({ auth: null }))).toBe("Extension không cần auth");
    expect(formatBytes(2048)).toBe("2 KB");
    expect(formatBytes(3 * 1024 * 1024)).toBe("3.0 MB");
  });
});
