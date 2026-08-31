import { describe, expect, it } from "vitest";
import { main, resolveRootArg } from "../src/main.ts";
import { runInit } from "../src/commands/init.ts";
import { runNext } from "../src/commands/next.ts";
import { loadState, storyPaths } from "../src/story-fs.ts";
import { makeStoryDir } from "./helpers.ts";

describe("resolveRootArg — root tương đối phải theo INIT_CWD của npm, không theo cwd của package CLI", () => {
  it("root tuyệt đối thì giữ nguyên, bỏ qua cả initCwd lẫn cwd", () => {
    expect(resolveRootArg("/abs/story", "/init/cwd", "/pkg/cwd")).toBe("/abs/story");
  });

  it("root tương đối + có INIT_CWD (chạy qua npm --prefix) thì join theo INIT_CWD", () => {
    expect(resolveRootArg(".", "/home/user/my-story", "/repo/apps/qt-ai-cli")).toBe(
      "/home/user/my-story",
    );
    expect(resolveRootArg("../sibling", "/home/user/my-story", "/repo/apps/qt-ai-cli")).toBe(
      "/home/user/sibling",
    );
  });

  it("root tương đối + không có INIT_CWD (gọi tsx trực tiếp) thì join theo cwd", () => {
    expect(resolveRootArg(".", undefined, "/home/user/my-story")).toBe("/home/user/my-story");
  });
});

describe("qt-ai skip: guard usage khi id bị thiếu (main.ts)", () => {
  it("`skip <root> --reason x` không có id → in usage/lỗi thay vì hiểu nhầm --reason là id", () => {
    const root = makeStoryDir({ "0001": "第一章" });
    runInit(root);
    runNext(root);
    const code = main(["skip", root, "--reason", "model từ chối"]);
    expect(code).toBe(2);
    expect(loadState(storyPaths(root)).chapters["0001"]?.status).toBe("translating"); // không bị skip nhầm
  });

  it("`skip <root> <id> --reason x` hợp lệ vẫn skip bình thường", () => {
    const root = makeStoryDir({ "0001": "第一章" });
    runInit(root);
    runNext(root);
    const code = main(["skip", root, "0001", "--reason", "model từ chối"]);
    expect(code).toBe(0);
    expect(loadState(storyPaths(root)).chapters["0001"]?.status).toBe("skipped");
  });
});
