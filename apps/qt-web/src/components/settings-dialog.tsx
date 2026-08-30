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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  activeAiProviderConfig,
  isAiProvider,
  type AiProviderConfig,
  type AiSettings,
} from "@/lib/ai-settings";
import { Switch } from "@/components/ui/switch";
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
  aiSettings: AiSettings;
  onAiSettingsChange: (settings: AiSettings) => void;
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
  aiSettings,
  onAiSettingsChange,
}: SettingsDialogProps) {
  const {
    register,
    formState: { errors },
  } = useFormContext<TranslationOptionsValues>();
  const { theme, setTheme } = useTheme();
  const endpointError = errors.endpoint?.message;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="gap-5 overflow-y-auto p-5 sm:max-w-2xl">
        <DialogHeader className="pr-8">
          <DialogTitle>Cài đặt</DialogTitle>
          <DialogDescription className="sr-only">
            Đường dẫn API và giao diện.
          </DialogDescription>
        </DialogHeader>

        <div className="grid gap-2">
          <Label htmlFor="endpoint">Đường dẫn API</Label>
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
                ? "API đang hoạt động."
                : gatewayStatus === "error"
                  ? "Không kết nối được API."
                  : "Chưa kiểm tra.")}
          </p>
        </div>

        <AiCredentialsFields aiSettings={aiSettings} onAiSettingsChange={onAiSettingsChange} />

        <AiTranslationFields aiSettings={aiSettings} onAiSettingsChange={onAiSettingsChange} />

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
 * Key/model được đọc và ghi theo provider đang chọn — đổi provider chỉ đổi
 * sang bộ cấu hình của provider đó, nên key DeepSeek không bao giờ bị gửi
 * sang endpoint Google và ngược lại.
 */
function AiCredentialsFields({
  aiSettings,
  onAiSettingsChange,
}: {
  aiSettings: AiSettings;
  onAiSettingsChange: (settings: AiSettings) => void;
}) {
  const active = activeAiProviderConfig(aiSettings);
  const updateActive = (patch: Partial<AiProviderConfig>) => {
    const next = { ...aiSettings };
    next[aiSettings.provider] = { ...active, ...patch };
    onAiSettingsChange(next);
  };

  return (
    <div className="grid gap-2">
      <Label htmlFor="ai-api-key">AI cho lọc tên &amp; từ điển</Label>
      <div className="flex gap-2">
        <Select
          value={aiSettings.provider}
          onValueChange={(value) => {
            if (isAiProvider(value)) onAiSettingsChange({ ...aiSettings, provider: value });
          }}
        >
          <SelectTrigger className="w-32 shrink-0" aria-label="Nhà cung cấp AI">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="deepseek">DeepSeek</SelectItem>
            <SelectItem value="gemini">Gemini</SelectItem>
            <SelectItem value="grok">Grok</SelectItem>
            <SelectItem value="glm">GLM</SelectItem>
          </SelectContent>
        </Select>
        <Input
          id="ai-api-key"
          type="password"
          autoComplete="off"
          spellCheck={false}
          className="min-w-0 flex-1 font-mono text-xs"
          value={active.apiKey}
          onChange={(event) => updateActive({ apiKey: event.target.value })}
          placeholder={
            {
              gemini: "API key Google AI",
              deepseek: "API key DeepSeek",
              grok: "API key xAI",
              glm: "API key Z.ai",
            }[aiSettings.provider]
          }
        />
      </div>
      <Input
        aria-label="Model AI lọc tên"
        autoComplete="off"
        spellCheck={false}
        className="font-mono text-xs"
        value={active.model}
        onChange={(event) => updateActive({ model: event.target.value })}
        placeholder={
          {
            gemini: "Model lọc tên (mặc định gemini-3.1-flash-lite)",
            deepseek: "Model lọc tên (mặc định deepseek-v4-flash)",
            grok: "Model lọc tên (mặc định grok-4.6)",
            glm: "Model lọc tên (mặc định glm-5.3-flash)",
          }[aiSettings.provider]
        }
      />
      <Input
        aria-label="Base URL proxy AI"
        autoComplete="off"
        spellCheck={false}
        className="font-mono text-xs"
        value={active.baseUrl}
        onChange={(event) => updateActive({ baseUrl: event.target.value })}
        placeholder="Base URL proxy (tùy chọn — trống là endpoint chính thức)"
      />
      <p className="text-xs text-muted-foreground">
        Key và proxy này cũng được Dịch AI dùng lại; model dịch được cấu hình riêng bên dưới.
        Key chỉ lưu trên trình duyệt này, tách riêng theo từng nhà cung cấp. Trình duyệt gọi
        thẳng{" "}
        {({ gemini: "Google AI", deepseek: "DeepSeek", grok: "xAI", glm: "Z.ai" })[
          aiSettings.provider
        ]}{" "}
        (hoặc proxy của
        bạn — endpoint cần cho phép CORS, hỗ trợ cả http://localhost); key không đi qua
        server của app. Chi phí tính vào tài khoản của bạn.
      </p>
    </div>
  );
}

function AiTranslationFields({
  aiSettings,
  onAiSettingsChange,
}: {
  aiSettings: AiSettings;
  onAiSettingsChange: (settings: AiSettings) => void;
}) {
  const provider = aiSettings.translation.provider;
  const model = aiSettings.translation.models[provider];

  return (
    <div className="grid gap-2 rounded-md border bg-muted/25 p-3">
      <div>
        <Label htmlFor="ai-translation-provider">Dịch AI</Label>
        <p className="mt-0.5 text-xs text-muted-foreground">
          Dùng key và Base URL của provider tương ứng ở trên.
        </p>
      </div>
      <div className="flex gap-2">
        <Select
          value={provider}
          onValueChange={(value) => {
            if (!isAiProvider(value)) return;
            onAiSettingsChange({
              ...aiSettings,
              translation: { ...aiSettings.translation, provider: value },
            });
          }}
        >
          <SelectTrigger
            id="ai-translation-provider"
            className="w-32 shrink-0"
            aria-label="Nhà cung cấp Dịch AI"
          >
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="deepseek">DeepSeek</SelectItem>
            <SelectItem value="gemini">Gemini</SelectItem>
            <SelectItem value="grok">Grok</SelectItem>
            <SelectItem value="glm">GLM</SelectItem>
          </SelectContent>
        </Select>
        <Input
          aria-label="Model Dịch AI"
          autoComplete="off"
          spellCheck={false}
          className="min-w-0 flex-1 font-mono text-xs"
          value={model}
          onChange={(event) =>
            onAiSettingsChange({
              ...aiSettings,
              translation: {
                ...aiSettings.translation,
                models: {
                  ...aiSettings.translation.models,
                  [provider]: event.target.value,
                },
              },
            })
          }
          placeholder={
            {
              gemini: "gemini-3.7-flash",
              deepseek: "deepseek-v4-flash",
              grok: "grok-4.6",
              glm: "glm-5.3-flash",
            }[provider]
          }
        />
      </div>
      <div className="mt-1 grid gap-1.5">
        <div className="flex items-center justify-between rounded-md bg-background/60 px-3 py-2">
          <Label htmlFor="ai-translation-thinking" className="text-xs font-normal">
            Thinking
          </Label>
          <Switch
            id="ai-translation-thinking"
            checked={aiSettings.translation.thinking}
            onCheckedChange={(thinking) =>
              onAiSettingsChange({
                ...aiSettings,
                translation: { ...aiSettings.translation, thinking },
              })
            }
          />
        </div>
        <div className="flex items-center justify-between rounded-md bg-background/60 px-3 py-2">
          <Label htmlFor="ai-translation-auto-glossary" className="text-xs font-normal">
            Tự thêm tên vào từ điển truyện
          </Label>
          <Switch
            id="ai-translation-auto-glossary"
            checked={aiSettings.translation.autoGlossary}
            onCheckedChange={(autoGlossary) =>
              onAiSettingsChange({
                ...aiSettings,
                translation: { ...aiSettings.translation, autoGlossary },
              })
            }
          />
        </div>
        <div className="flex items-center justify-between rounded-md bg-background/60 px-3 py-2">
          <Label htmlFor="ai-translation-grok-fallback" className="text-xs font-normal">
            Tự chuyển sang Grok khi Gemini chặn nội dung
          </Label>
          <Switch
            id="ai-translation-grok-fallback"
            checked={aiSettings.translation.grokFallback}
            onCheckedChange={(grokFallback) =>
              onAiSettingsChange({
                ...aiSettings,
                translation: { ...aiSettings.translation, grokFallback },
              })
            }
          />
        </div>
      </div>
      <p className="text-xs text-muted-foreground">
        Thinking chỉ áp dụng cho Dịch AI: DeepSeek và Gemini 2.5 có thể tắt hẳn;
        Gemini 3.x và GLM 5.3 không tắt hẳn được, tắt công tắc là hạ xuống mức
        nghĩ thấp nhất (GLM bật rất chậm, mỗi chương có thể mất hàng chục phút).
      </p>
    </div>
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
