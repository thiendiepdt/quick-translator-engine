import { RotateCcw } from "lucide-react";
import { lazy, Suspense, useEffect, useState } from "react";
import { useFormContext, useWatch } from "react-hook-form";

import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import type { StoryFormValues } from "@/lib/story-form";
import type { StoryDefaults } from "@/lib/types";

// Plate + remark nặng; chỉ tải khi mở mục Prompt.
const PlatePromptEditor = lazy(async () => {
  const module = await import("@/components/plate-prompt-editor");
  return { default: module.PlatePromptEditor };
});

/**
 * Editor markdown (Plate, giống qt-web) luôn hiện prompt đang dùng: `customPrompt` trống thì nạp
 * prompt gốc của hệ và cho sửa thẳng — sửa là thành prompt riêng; gõ lại thành y hệt bản gốc hoặc
 * bấm "Về mặc định" thì lưu trống (editor remount để nạp lại bản gốc).
 */
export function PromptEditor({ defaults }: { defaults: StoryDefaults | undefined }) {
  const { control, setValue } = useFormContext<StoryFormValues>();
  const custom = useWatch({ control, name: "customPrompt" }) ?? "";
  const usingDefault = custom.trim() === "";
  const [version, setVersion] = useState(0);
  // So sánh sau khi chuẩn hoá qua chính editor — chuỗi thô của prompt gốc và markdown editor
  // xuất ra khác nhau ở dấu list/dòng trống dù nội dung y hệt.
  const [normalizedDefault, setNormalizedDefault] = useState<string | undefined>();
  const basePrompt = defaults?.basePrompt;
  useEffect(() => {
    if (basePrompt === undefined) return;
    let cancelled = false;
    void import("@/components/plate-prompt-editor-plugins").then((module) => {
      if (!cancelled) setNormalizedDefault(module.normalizeMarkdown(basePrompt));
    });
    return () => {
      cancelled = true;
    };
  }, [basePrompt]);

  const set = (markdown: string) => {
    const isDefault = normalizedDefault !== undefined && markdown.trim() === normalizedDefault;
    setValue("customPrompt", isDefault ? "" : markdown, { shouldDirty: true });
  };
  const reset = () => {
    setValue("customPrompt", "", { shouldDirty: true });
    setVersion((v) => v + 1);
  };

  return (
    <div className="flex flex-col gap-1.5">
      <div className="flex flex-wrap items-center gap-2">
        <Label>
          Prompt dịch{" "}
          <span className="rounded-full bg-muted px-1.5 text-xs font-normal text-muted-foreground">
            {usingDefault ? "mặc định" : "riêng"}
          </span>
        </Label>
        <div className="flex-1" />
        <Button type="button" size="xs" variant="ghost" disabled={usingDefault} onClick={reset}>
          <RotateCcw /> Về mặc định
        </Button>
      </div>
      {defaults ? (
        <Suspense
          fallback={
            <div role="status" className="grid min-h-[420px] place-items-center rounded-md border bg-card text-sm text-muted-foreground">
              Đang tải editor…
            </div>
          }
        >
          <PlatePromptEditor key={version} initialValue={custom || defaults.basePrompt} onChange={set} />
        </Suspense>
      ) : (
        <div role="status" className="grid min-h-[420px] place-items-center rounded-md border bg-card text-sm text-muted-foreground">
          Đang tải prompt mặc định…
        </div>
      )}
      <p className="text-xs text-muted-foreground">
        Markdown, bôi đen để in đậm/nghiêng. Sửa ngay trên bản mặc định là thành prompt riêng của truyện; thông tin
        truyện, glossary và style được nối tự động phía sau.
      </p>
      {defaults && (
        <details className="rounded-md border bg-muted/25 px-3 py-2 text-xs">
          <summary className="cursor-pointer text-muted-foreground">Phần đuôi cố định (luôn nối sau prompt, chỉ đọc)</summary>
          <pre className="mt-2 font-mono whitespace-pre-wrap text-muted-foreground">{defaults.promptSuffix}</pre>
        </details>
      )}
    </div>
  );
}
