import { LoaderCircle, Server } from "lucide-react";
import { useFormContext } from "react-hook-form";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { themes, type ThemeName } from "@/lib/theme";
import { useTheme } from "@/lib/theme-context";
import type { TranslationOptionsValues } from "@/lib/schema";
import { cn } from "@/lib/utils";

interface SettingsDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  gatewayStatus: "unknown" | "ok" | "error";
  gatewayChecking: boolean;
  onTestGateway: () => void;
}

/**
 * Endpoint và giao diện là cấu hình đặt-một-lần, không phải điều khiển hằng
 * ngày — trước đây chúng chiếm nguyên khoảng giữa thanh trên. Đưa vào đây,
 * thanh trên chỉ còn một chấm trạng thái bấm được.
 */
export function SettingsDialog({
  open,
  onOpenChange,
  gatewayStatus,
  gatewayChecking,
  onTestGateway,
}: SettingsDialogProps) {
  const {
    register,
    formState: { errors },
  } = useFormContext<TranslationOptionsValues>();
  const { theme, setTheme } = useTheme();
  const endpointError = errors.endpoint?.message;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="gap-5 p-5 sm:max-w-lg">
        <DialogHeader className="pr-8">
          <DialogTitle>Cài đặt</DialogTitle>
          <DialogDescription className="sr-only">
            Endpoint gateway và giao diện.
          </DialogDescription>
        </DialogHeader>

        <div className="grid gap-2">
          <Label htmlFor="endpoint">Cloudflare gateway</Label>
          <div className="flex gap-2">
            <Input
              id="endpoint"
              aria-invalid={Boolean(endpointError)}
              className="min-w-0 flex-1 font-mono text-xs"
              spellCheck={false}
              {...register("endpoint")}
            />
            <Button
              type="button"
              variant="outline"
              disabled={gatewayChecking}
              onClick={onTestGateway}
            >
              {gatewayChecking ? <LoaderCircle className="animate-spin" /> : <Server />}
              Kiểm tra
            </Button>
          </div>
          <p
            className={cn(
              "text-xs",
              endpointError
                ? "text-destructive"
                : gatewayStatus === "ok"
                  ? "text-ok"
                  : gatewayStatus === "error"
                    ? "text-destructive"
                    : "text-muted-foreground",
            )}
          >
            {endpointError
              ?? (gatewayStatus === "ok"
                ? "Gateway đang hoạt động."
                : gatewayStatus === "error"
                  ? "Không kết nối được gateway."
                  : "Chưa kiểm tra.")}
          </p>
        </div>

        <div className="grid gap-2">
          <Label>Giao diện</Label>
          <div className="grid grid-cols-2 gap-2 sm:grid-cols-3">
            {themes.map((item) => (
              <ThemeSwatch
                key={item.value}
                value={item.value}
                label={item.label}
                className={item.className}
                active={theme === item.value}
                onSelect={setTheme}
              />
            ))}
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}

/**
 * Ô chọn theme tự vẽ bằng chính token của theme đó, nên người dùng thấy trước
 * màu thật thay vì phải đoán qua cái tên.
 */
function ThemeSwatch({
  value,
  label,
  className,
  active,
  onSelect,
}: {
  value: ThemeName;
  label: string;
  className: string;
  active: boolean;
  onSelect: (theme: ThemeName) => void;
}) {
  return (
    <button
      type="button"
      aria-pressed={active}
      onClick={() => onSelect(value)}
      className={cn(
        "group overflow-hidden rounded-md border text-left transition-colors",
        active ? "border-primary ring-2 ring-ring/35" : "hover:border-primary/45",
      )}
    >
      <span className={cn("flex h-10 items-center gap-1 bg-background px-2", className)}>
        <span className="h-6 w-1.5 rounded-full bg-sidebar" />
        <span className="h-6 flex-1 rounded-sm border border-border bg-card" />
        <span className="h-6 w-5 rounded-sm bg-reader-paper" />
        <span className="size-3 rounded-full bg-pair" />
        <span className="size-3 rounded-full bg-primary" />
      </span>
      <span className="block truncate border-t bg-card px-2 py-1.5 text-[11px] font-medium">
        {label}
      </span>
    </button>
  );
}
