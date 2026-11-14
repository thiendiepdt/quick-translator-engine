import { CircleCheck, LoaderCircle, LogIn, LogOut, RefreshCw, UserRound } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { extLogout, type ExtAccount, type ExtConfig } from "@/lib/api";
import { cn } from "@/lib/utils";
import { forgetAccountCheck, loadAccountCheck } from "@/store/raw-check";

const message = (error: unknown) => (error instanceof Error ? error.message : String(error));
const clock = (ms: number) => new Date(ms).toLocaleTimeString("vi-VN", { hour: "2-digit", minute: "2-digit" });

interface Props {
  root: string;
  /** Config của extension đang dùng, có khối `account`. */
  config: ExtConfig;
  busy: boolean;
  onLogin: () => void;
  onConfig: (config: ExtConfig) => void;
}

/** Kết quả kiểm, gắn với phiên lúc kiểm (`key`) — phiên đổi thì kết quả cũ không dùng nữa. */
type Check = ({ kind: "ok"; result: ExtAccount } | { kind: "error"; error: string }) & { key: string };

/**
 * Tài khoản trang nguồn (vd. QQ阅读) — tách khỏi API key: tình trạng đăng nhập, hướng dẫn của extension, nút Đăng
 * nhập (mở cửa sổ của extension), Kiểm tra (gọi `account`, không mở cửa sổ), Đăng xuất. Mở trang thì tự kiểm (nhớ
 * 10 phút); đăng nhập / đăng xuất xong thì kiểm lại.
 */
export function AccountLine({ root, config, busy, onLogin, onConfig }: Props) {
  const account = config.account;
  const [saved, setSaved] = useState<Check>();
  const [forcing, setForcing] = useState(false);
  const [working, setWorking] = useState(false);
  // Đổi extension hoặc phiên đổi (đăng nhập, đăng xuất) thì kiểm lại.
  const key = `${root}|${config.id}|${config.session.loggedIn}|${config.session.guest}|${config.session.account ?? ""}`;

  useEffect(() => {
    let alive = true;
    loadAccountCheck(root, config.id).then(
      (result) => alive && setSaved({ kind: "ok", result, key }),
      (error: unknown) => alive && setSaved({ kind: "error", error: message(error), key }),
    );
    return () => {
      alive = false;
    };
  }, [root, config.id, key]);

  const check = saved?.key === key ? saved : undefined;
  const checking = forcing || !check;

  function recheck() {
    setForcing(true);
    loadAccountCheck(root, config.id, true)
      .then(
        (result) => setSaved({ kind: "ok", result, key }),
        (error: unknown) => setSaved({ kind: "error", error: message(error), key }),
      )
      .finally(() => setForcing(false));
  }

  if (!account) return null;

  async function logout() {
    setWorking(true);
    try {
      const next = await extLogout(config.id);
      forgetAccountCheck(config.id);
      onConfig(next);
      toast.success(`Đã đăng xuất ${account?.title ?? ""}`.trim());
    } catch (error) {
      toast.error(message(error));
    } finally {
      setWorking(false);
    }
  }

  const status = check?.kind === "ok" ? check.result.status : undefined;
  // Chưa kiểm xong / kiểm lỗi thì theo phiên đã lưu.
  const loggedIn = status ? status.loggedIn : config.session.loggedIn && !config.session.guest;
  const name = status?.name ?? config.session.account;
  const note = status?.message ?? (!loggedIn && config.session.guest ? "đang tải kiểu khách" : null);
  const text = loggedIn ? (name ? `đã đăng nhập: ${name}` : "đã đăng nhập") : "chưa đăng nhập";

  return (
    <div className="flex flex-col gap-1.5 rounded-md border p-3 text-sm" aria-label={account.title}>
      <div className="flex flex-wrap items-center gap-2">
        <span className="flex items-center gap-1.5 font-medium">
          <UserRound className="size-4 text-muted-foreground" /> {account.title}
        </span>
        <span
          aria-label={`Trạng thái ${account.title}`}
          className={cn("flex items-center gap-1", loggedIn ? "text-status-done" : account.required ? "text-destructive" : "text-muted-foreground")}
        >
          {checking && !check ? <LoaderCircle className="size-3.5 animate-spin" /> : loggedIn && <CircleCheck className="size-3.5" />}
          {checking && !check ? "đang kiểm…" : text}
          {note && <span className="text-muted-foreground"> — {note}</span>}
        </span>
        {check?.kind === "ok" && <span className="text-xs text-muted-foreground">kiểm lúc {clock(check.result.checkedAt)}</span>}
      </div>
      {check?.kind === "error" && <p className="text-xs text-destructive">Không kiểm được: {check.error}</p>}
      {account.help && <p className="text-xs text-muted-foreground">{account.help}</p>}
      <div className="flex flex-wrap items-center gap-2">
        <Button variant={loggedIn ? "outline" : "default"} size="sm" disabled={busy || working} onClick={onLogin}>
          <LogIn /> {loggedIn ? "Đăng nhập lại" : `Đăng nhập ${account.title}`}
        </Button>
        <Button variant="ghost" size="sm" disabled={busy || working || checking} onClick={recheck}>
          <RefreshCw className={cn(checking && "animate-spin")} /> Kiểm tra
        </Button>
        {config.session.hasData && (
          <Button variant="ghost" size="sm" disabled={busy || working} onClick={() => void logout()}>
            <LogOut /> Đăng xuất
          </Button>
        )}
      </div>
    </div>
  );
}
