import { useState } from "react";
import { toast } from "sonner";

import { ExtField } from "@/components/ext-fields";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { extClearAuth, extClearCache, extLogout, extSaveAuth, extSaveSettings, type ExtAuth, type ExtConfig, type ExtSetting, type ExtValues } from "@/lib/api";
import { invalidateRawChecks } from "@/store/raw-check";
import { authFieldAsSetting, authStatus, changedValues, formatBytes, initialDisplay, toValue, type FieldDisplay } from "@/lib/ext";

const message = (error: unknown) => (error instanceof Error ? error.message : String(error));

interface Props {
  config: ExtConfig;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Cấu hình mới sau mỗi lần lưu / xoá. */
  onChange: (config: ExtConfig) => void;
}

/** Hộp "Auth và cài đặt" của một extension: auth theo kiểu trong manifest, cài đặt chung, đăng xuất, xoá cache. */
export function ExtConfigDialog({ config, open, onOpenChange, onChange }: Props) {
  // Mỗi lần lưu xong dựng lại form từ cấu hình mới (ô secret về trống, placeholder "Đã lưu").
  const [version, setVersion] = useState(0);
  const [busy, setBusy] = useState(false);

  async function act(done: string, task: () => Promise<ExtConfig>) {
    setBusy(true);
    try {
      onChange(await task());
      invalidateRawChecks();
      setVersion((v) => v + 1);
      toast.success(done);
    } catch (error) {
      toast.error(message(error));
    } finally {
      setBusy(false);
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[85vh] overflow-y-auto sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>Auth và cài đặt — {config.name}</DialogTitle>
          <DialogDescription>Khoá và mật khẩu lưu trong Credential Manager của Windows, không nằm trong thư mục truyện.</DialogDescription>
        </DialogHeader>
        {config.secretStore !== "keyring" && (
          <p role="alert" className="text-xs text-destructive">
            Máy này không có kho khoá — khoá đang lưu trong file ở thư mục dữ liệu của extension.
          </p>
        )}
        {config.sharedAuth ? (
          <section aria-label="Auth dùng chung" className="flex flex-col gap-1 text-sm">
            <h3 className="font-medium">Auth</h3>
            <p>
              Dùng API key chung của kho <strong>{config.sharedAuth.name}</strong>
              {config.auth?.scheme ? " (đã nhập)" : " — chưa nhập"}. Sửa ở Cài đặt → Extension tải raw → Kho extension.
            </p>
          </section>
        ) : (
          config.auth && (
          <AuthForm
            key={`auth-${version}`}
            auth={config.auth}
            busy={busy}
            onSave={(scheme, values) => void act("Đã lưu auth", () => extSaveAuth(config.id, scheme, values))}
            onClear={() => void act("Đã xoá auth", () => extClearAuth(config.id))}
          />
          )
        )}
        {config.settings.length > 0 && (
          <SettingsForm key={`settings-${version}`} fields={config.settings} busy={busy} onSave={(values) => void act("Đã lưu cài đặt", () => extSaveSettings(config.id, values))} />
        )}
        <section className="flex flex-col gap-2 border-t pt-4">
          <h3 className="text-sm font-medium">Phiên và cache</h3>
          <p className="text-xs text-muted-foreground">
            {authStatus(config)} · cache {formatBytes(config.cacheBytes)}
          </p>
          <div className="flex gap-2">
            <Button variant="outline" size="sm" disabled={busy || !config.session.hasData} onClick={() => void act("Đã đăng xuất", () => extLogout(config.id))}>
              Đăng xuất
            </Button>
            <Button variant="outline" size="sm" disabled={busy || config.cacheBytes === 0} onClick={() => void act("Đã xoá cache", () => extClearCache(config.id))}>
              Xoá cache
            </Button>
          </div>
        </section>
      </DialogContent>
    </Dialog>
  );
}

export function AuthForm({ auth, busy, onSave, onClear }: { auth: ExtAuth; busy: boolean; onSave: (scheme: string, values: ExtValues) => void; onClear: () => void }) {
  const [scheme, setScheme] = useState(auth.scheme ?? auth.schemes[0]?.id ?? "");
  const [displays, setDisplays] = useState<Record<string, FieldDisplay>>({});
  const current = auth.schemes.find((s) => s.id === scheme) ?? auth.schemes[0];
  const saved = scheme === auth.scheme;
  const fields: ExtSetting[] = (current?.fields ?? []).map((f) => authFieldAsSetting(f, saved ? auth.values[f.key] : null, saved && auth.filled.includes(f.key)));
  const valueOf = (field: ExtSetting) => displays[field.key] ?? initialDisplay(field);

  return (
    <section className="flex flex-col gap-3">
      <h3 className="flex items-center gap-2 text-sm font-medium">
        Auth {auth.required ? <Badge variant="outline">Bắt buộc</Badge> : <span className="text-xs font-normal text-muted-foreground">(tuỳ chọn)</span>}
      </h3>
      {auth.schemes.length > 1 && (
        <div className="flex flex-col gap-1.5">
          <Label htmlFor="ext-auth-scheme">Kiểu</Label>
          <Select
            value={scheme}
            onValueChange={(next) => {
              setScheme(next);
              setDisplays({});
            }}
          >
            <SelectTrigger id="ext-auth-scheme" aria-label="Kiểu auth" className="w-full">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {auth.schemes.map((s) => (
                <SelectItem key={s.id} value={s.id}>
                  {s.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      )}
      {fields.map((field) => (
        <ExtField key={`${scheme}-${field.key}`} idPrefix={`ext-auth-${scheme}`} field={field} value={valueOf(field)} onChange={(v) => setDisplays((d) => ({ ...d, [field.key]: v }))} />
      ))}
      <div className="flex gap-2">
        <Button size="sm" disabled={busy || !current} onClick={() => onSave(scheme, Object.fromEntries(fields.map((f) => [f.key, toValue(f, valueOf(f))])))}>
          Lưu auth
        </Button>
        <Button variant="outline" size="sm" disabled={busy || !auth.scheme} onClick={onClear}>
          Xoá auth
        </Button>
      </div>
    </section>
  );
}

function SettingsForm({ fields, busy, onSave }: { fields: ExtSetting[]; busy: boolean; onSave: (values: ExtValues) => void }) {
  const [displays, setDisplays] = useState<Record<string, FieldDisplay>>({});
  const changed = changedValues(fields, displays);
  return (
    <section className="flex flex-col gap-3 border-t pt-4">
      <h3 className="text-sm font-medium">Cài đặt chung</h3>
      {fields.map((field) => (
        <ExtField key={field.key} idPrefix="ext-setting" field={field} value={displays[field.key] ?? initialDisplay(field)} onChange={(v) => setDisplays((d) => ({ ...d, [field.key]: v }))} />
      ))}
      <Button size="sm" className="self-start" disabled={busy || Object.keys(changed).length === 0} onClick={() => onSave(changed)}>
        Lưu cài đặt
      </Button>
    </section>
  );
}
