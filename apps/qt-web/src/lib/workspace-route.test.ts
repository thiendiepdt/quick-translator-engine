import { describe, expect, it } from "vitest";

import {
  pathForWorkspaceView,
  workspaceViewFromPath,
} from "@/lib/workspace-route";

describe("workspace routes", () => {
  it("maps each workspace view to its own path and back", () => {
    expect(pathForWorkspaceView("translate")).toBe("/");
    expect(pathForWorkspaceView("ai-translate")).toBe("/dich-ai");
    expect(pathForWorkspaceView("names")).toBe("/loc-ten");

    expect(workspaceViewFromPath("/")).toBe("translate");
    expect(workspaceViewFromPath("/dich-ai")).toBe("ai-translate");
    expect(workspaceViewFromPath("/loc-ten")).toBe("names");
  });

  it("tolerates trailing slashes and rejects unknown paths", () => {
    expect(workspaceViewFromPath("/dich-ai/")).toBe("ai-translate");
    expect(workspaceViewFromPath("/khong-ton-tai")).toBeUndefined();
  });
});
