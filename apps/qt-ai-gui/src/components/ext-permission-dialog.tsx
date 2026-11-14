import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import type { ExtInspection } from "@/lib/api";

interface Props {
  inspection?: ExtInspection;
  busy: boolean;
  onCancel: () => void;
  onConfirm: () => void;
}

/** Hộp xin quyền trước khi cài / cập nhật extension: domain được gọi tới, quyền webview, auth bắt buộc. */
export function ExtPermissionDialog({ inspection, busy, onCancel, onConfirm }: Props) {
  const update = !!inspection?.currentVersion;
  return (
    <Dialog open={!!inspection} onOpenChange={(open) => !open && onCancel()}>
      <DialogContent className="max-h-[85vh] overflow-y-auto sm:max-w-lg">
        {inspection && (
          <>
            <DialogHeader>
              <DialogTitle className="flex items-center gap-2">
                {inspection.icon && <img src={inspection.icon} alt="" className="size-6 rounded" />}
                {update ? `Cập nhật ${inspection.name}` : `Cài ${inspection.name}`}
              </DialogTitle>
              <DialogDescription>{inspection.description}</DialogDescription>
            </DialogHeader>
            <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-sm">
              <dt className="text-muted-foreground">Phiên bản</dt>
              <dd>{update ? `${inspection.currentVersion} → ${inspection.version}` : inspection.version}</dd>
              <dt className="text-muted-foreground">Nhà phát triển</dt>
              <dd>
                {inspection.developer.url ? <a href={inspection.developer.url}>{inspection.developer.name}</a> : inspection.developer.name}
                {inspection.developer.email ? ` · ${inspection.developer.email}` : ""}
              </dd>
              {inspection.license && (
                <>
                  <dt className="text-muted-foreground">Giấy phép</dt>
                  <dd>{inspection.license}</dd>
                </>
              )}
              <dt className="text-muted-foreground">Id</dt>
              <dd className="font-mono text-xs">{inspection.id}</dd>
            </dl>
            <section aria-label="Domain extension được gọi tới" className="flex flex-col gap-1 text-sm">
              <p className="font-medium">Được tải dữ liệu từ</p>
              <ul className="flex flex-wrap gap-1">
                {inspection.hosts.map((host) => (
                  <li key={host}>
                    <Badge variant={update && inspection.newHosts.includes(host) ? "default" : "secondary"}>
                      {host}
                      {update && inspection.newHosts.includes(host) ? " (mới)" : ""}
                    </Badge>
                  </li>
                ))}
              </ul>
            </section>
            {inspection.permissions.includes("webview") && (
              <p className="text-sm">
                Được mở cửa sổ trình duyệt để bạn đăng nhập hoặc giải captcha trên các domain trên
                {update && inspection.newPermissions.includes("webview") ? " (quyền mới)" : ""}.
              </p>
            )}
            {inspection.sharedAuthRepo ? (
              <p className="text-sm">Dùng API key chung của kho {inspection.sharedAuthRepo} (nhập một lần ở phần kho).</p>
            ) : (
              inspection.authRequired && <p className="text-sm">Cần nhập API key hoặc tài khoản trước khi dùng.</p>
            )}
            {update && !inspection.newer && <p className="text-sm text-destructive">Bản này không mới hơn bản đang cài.</p>}
            <DialogFooter>
              <Button variant="outline" onClick={onCancel}>
                Huỷ
              </Button>
              <Button onClick={onConfirm} disabled={busy}>
                {update ? "Cập nhật" : "Cài"}
              </Button>
            </DialogFooter>
          </>
        )}
      </DialogContent>
    </Dialog>
  );
}
