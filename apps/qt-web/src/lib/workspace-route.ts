import type { WorkspaceView } from "@/store/workspace";

/**
 * Mỗi khung làm việc có một đường dẫn riêng để bookmark/F5 giữ nguyên tab.
 * Worker assets đã bật single-page-application nên deep link luôn về index.html.
 */
const VIEW_PATHS: Record<WorkspaceView, string> = {
  translate: "/",
  "ai-translate": "/dich-ai",
  names: "/loc-ten",
};

export function pathForWorkspaceView(view: WorkspaceView): string {
  return VIEW_PATHS[view];
}

export function workspaceViewFromPath(pathname: string): WorkspaceView | undefined {
  const clean = pathname.replace(/\/+$/, "") || "/";
  const match = Object.entries(VIEW_PATHS).find(([, path]) => path === clean);
  return match?.[0] as WorkspaceView | undefined;
}

/** View khởi tạo theo URL hiện tại; ngoài danh sách thì về Convert. */
export function initialWorkspaceView(): WorkspaceView {
  if (typeof window === "undefined") return "translate";
  return workspaceViewFromPath(window.location.pathname) ?? "translate";
}
