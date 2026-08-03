import {
  BookOpenText,
  BrainCircuit,
  Languages,
  LoaderCircle,
  Save,
  Sparkles,
  Trash2,
  X,
} from "lucide-react";
import {
  FormEvent,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  baseUrlProblem,
  explainDictionaryEntryWithAi,
  resolveAiCall,
  translateDictionaryEntryWithAi,
} from "@/lib/ai-client";
import {
  activeAiProviderConfig,
  type AiSettings,
} from "@/lib/ai-settings";
import { fetchMeanings, translateChapter } from "@/lib/api";
import {
  lowercaseText,
  sentenceCaseText,
  titleCaseText,
  uppercaseText,
} from "@/lib/text-case";
import type {
  DictionaryUpdateKey,
  LacVietMeaning,
  LocalDictionaryEntries,
} from "@/lib/types";

export interface DictionaryUpdateSelection {
  source: string;
  target: string;
}

interface DictionaryUpdateDialogProps {
  open: boolean;
  endpoint: string;
  dictionaryKey?: DictionaryUpdateKey;
  selection?: DictionaryUpdateSelection;
  context?: string;
  aiSettings?: AiSettings;
  localEntries: LocalDictionaryEntries;
  onOpenChange: (open: boolean) => void;
  onOpenAiSettings?: () => void;
  onSave: (
    key: DictionaryUpdateKey,
    entries: Record<string, string>,
    previousKeys: string[],
  ) => void;
  onRemove: (key: DictionaryUpdateKey, previousKeys: string[]) => void;
}

const labels: Record<DictionaryUpdateKey, string> = {
  vietPhrase: "VietPhrase",
  names: "Tên",
  names2: "Tên 2",
  chinesePhienAmWords: "Phiên Âm",
  danhTu: "Danh Từ",
  hauTu: "Hậu Từ",
  hoNguoi: "Họ Người",
  luatNhan: "Luật Nhân",
};

type PendingAction = "han-viet" | "ai-translate" | "lac-viet" | "ai-meaning";

type LookupResult =
  | { title: string; entries: LacVietMeaning[] }
  | { title: string; text: string };

function selectedKeys(
  dictionaryKey: DictionaryUpdateKey,
  source: string,
): string[] {
  if (dictionaryKey === "chinesePhienAmWords") {
    return Array.from(source.replace(/\s+/g, ""));
  }
  return [source.trim()];
}

function initialValue(
  dictionaryKey: DictionaryUpdateKey,
  selection: DictionaryUpdateSelection,
  localEntries: LocalDictionaryEntries,
): string {
  const keys = selectedKeys(dictionaryKey, selection.source);
  const saved = keys.map((key) => localEntries[dictionaryKey][key]);
  if (saved.length > 0 && saved.every((value) => value !== undefined)) {
    return saved.join(" ");
  }
  return selection.target.trim();
}

export function DictionaryUpdateDialog({
  open,
  endpoint,
  dictionaryKey,
  selection,
  context = "",
  aiSettings,
  localEntries,
  onOpenChange,
  onOpenAiSettings,
  onSave,
  onRemove,
}: DictionaryUpdateDialogProps) {
  const [source, setSource] = useState(() => selection?.source.trim() ?? "");
  const [target, setTarget] = useState(() =>
    dictionaryKey && selection
      ? initialValue(dictionaryKey, selection, localEntries)
      : "",
  );
  const [pendingAction, setPendingAction] = useState<PendingAction>();
  const [lookupResult, setLookupResult] = useState<LookupResult>();
  const autoHanVietKeyRef = useRef<string | undefined>(undefined);

  const previousKeys = useMemo(
    () =>
      dictionaryKey && selection
        ? selectedKeys(dictionaryKey, selection.source)
        : [],
    [dictionaryKey, selection],
  );
  const hasSavedEntry = Boolean(
    dictionaryKey &&
      previousKeys.some((key) => localEntries[dictionaryKey][key] !== undefined),
  );

  const activeDictionaryKey = dictionaryKey ?? "names";
  const dictionaryLabel = labels[activeDictionaryKey];

  function normalizedSource(): string | undefined {
    const value = source.trim();
    if (value) return value;
    toast.error("Nhập tiếng Trung trước khi dùng tiện ích");
    return undefined;
  }

  function aiCall() {
    if (!aiSettings) {
      toast.warning("Chưa có cấu hình AI");
      return undefined;
    }
    const providerConfig = activeAiProviderConfig(aiSettings);
    if (!providerConfig.apiKey.trim()) {
      toast.warning("Tính năng AI cần API key của bạn", {
        description: "Nhập key DeepSeek/Gemini trong Cài đặt trước khi sử dụng.",
        ...(onOpenAiSettings
          ? { action: { label: "Mở Cài đặt", onClick: onOpenAiSettings } }
          : {}),
      });
      return undefined;
    }
    const problem = baseUrlProblem(providerConfig.baseUrl);
    if (problem) {
      toast.error(problem);
      return undefined;
    }
    return resolveAiCall(aiSettings.provider, providerConfig);
  }

  const translateFromHanViet = useCallback(async () => {
    const text = source.trim();
    if (!text) {
      toast.error("Nhập tiếng Trung trước khi dùng tiện ích");
      return;
    }
    setPendingAction("han-viet");
    try {
      const result = await translateChapter(endpoint, {
        text,
        mode: "hanviet",
        wrap: false,
        pretty: false,
        ranges: true,
        scanRange: 30,
        translationAlgorithm: 1,
        prioritizedName: true,
        dictionaryPatches: {
          chinesePhienAmWords: localEntries.chinesePhienAmWords,
        },
      });
      const translated = titleCaseText(result.translated.trim());
      if (!translated) throw new Error("Bộ máy không trả về âm Hán Việt");
      setTarget(translated);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không thể dịch Hán Việt");
    } finally {
      setPendingAction(undefined);
    }
  }, [endpoint, localEntries.chinesePhienAmWords, source]);

  useEffect(() => {
    if (
      !open ||
      !selection ||
      (dictionaryKey !== "names" && dictionaryKey !== "names2") ||
      hasSavedEntry
    ) {
      return;
    }
    const sourceKey = selection.source.trim();
    if (!sourceKey) return;
    const autoHanVietKey = `${dictionaryKey}:${sourceKey}`;
    if (autoHanVietKeyRef.current === autoHanVietKey) return;
    autoHanVietKeyRef.current = autoHanVietKey;
    void translateFromHanViet();
  }, [dictionaryKey, hasSavedEntry, open, selection, translateFromHanViet]);

  if (!dictionaryKey || !selection) return null;

  async function translateWithAi() {
    const text = normalizedSource();
    const config = aiCall();
    if (!text || !config) return;
    setPendingAction("ai-translate");
    try {
      const translated = await translateDictionaryEntryWithAi(
        {
          source: text,
          currentTranslation: target.trim(),
          context,
          dictionaryLabel,
        },
        config,
      );
      setTarget(translated);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "AI không thể dịch cụm này");
    } finally {
      setPendingAction(undefined);
    }
  }

  async function lookupLacViet() {
    const text = normalizedSource();
    if (!text) return;
    setPendingAction("lac-viet");
    try {
      const result = await fetchMeanings(endpoint, text);
      setLookupResult({ title: "Nghĩa Lạc Việt", entries: result.entries });
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không thể tra Lạc Việt");
    } finally {
      setPendingAction(undefined);
    }
  }

  async function lookupWithAi() {
    const text = normalizedSource();
    const config = aiCall();
    if (!text || !config) return;
    setPendingAction("ai-meaning");
    try {
      const meaning = await explainDictionaryEntryWithAi(
        {
          source: text,
          currentTranslation: target.trim(),
          context,
          dictionaryLabel,
        },
        config,
      );
      setLookupResult({ title: "Giải nghĩa bằng AI", text: meaning });
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "AI không thể tra nghĩa");
    } finally {
      setPendingAction(undefined);
    }
  }

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const normalizedSourceValue = source.trim();
    const normalizedTarget = target.trim();
    if (!normalizedSourceValue || !normalizedTarget) {
      toast.error("Tiếng Trung và nghĩa tiếng Việt không được để trống");
      return;
    }

    if (activeDictionaryKey === "chinesePhienAmWords") {
      const characters = Array.from(normalizedSourceValue.replace(/\s+/g, ""));
      const readings = normalizedTarget.split(/\s+/).filter(Boolean);
      if (characters.length !== readings.length) {
        toast.error("Phiên Âm cần đúng một âm đọc cho mỗi chữ Hán");
        return;
      }
      onSave(
        activeDictionaryKey,
        Object.fromEntries(
          characters.map((character, index) => [character, readings[index]]),
        ),
        previousKeys,
      );
    } else {
      if (
        activeDictionaryKey === "luatNhan" &&
        !normalizedSourceValue.includes("{n}") &&
        !normalizedSourceValue.includes("{s}")
      ) {
        toast.error("Luật Nhân cần chứa placeholder {n} hoặc {s}");
        return;
      }
      onSave(
        activeDictionaryKey,
        { [normalizedSourceValue]: normalizedTarget },
        previousKeys,
      );
    }
    toast.success(`Đã lưu ${dictionaryLabel} vào local`);
    onOpenChange(false);
  }

  function remove() {
    onRemove(activeDictionaryKey, previousKeys);
    toast.success(`Đã xóa bản cập nhật ${dictionaryLabel} khỏi local`);
    onOpenChange(false);
  }

  const fixed =
    dictionaryKey === "vietPhrase" ||
    dictionaryKey === "chinesePhienAmWords";
  const asyncDisabled = pendingAction !== undefined;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(95vw,680px)]">
        <DialogHeader className="border-b px-6 py-5">
          <DialogTitle>Cập nhật {dictionaryLabel}</DialogTitle>
          <DialogDescription>
            {fixed
              ? "Bản ghi được lưu trong trình duyệt và gửi kèm mỗi lần dịch; từ điển gốc trong bộ máy không bị thay đổi."
              : "Bản ghi được lưu trong trình duyệt, ghép vào bản nháp từ điển và dùng ở lần dịch tiếp theo."}
          </DialogDescription>
        </DialogHeader>

        <form className="flex min-h-0 flex-col" onSubmit={submit}>
          <div className="grid min-h-0 min-w-0 gap-5 overflow-x-hidden overflow-y-auto px-6 py-5">
            <div className="grid gap-2">
              <Label htmlFor="dictionary-update-source">
                {dictionaryKey === "luatNhan" ? "Luật" : "Tiếng Trung"}
              </Label>
              <Input
                id="dictionary-update-source"
                value={source}
                onChange={(event) => setSource(event.target.value)}
                autoFocus
              />
              {dictionaryKey === "luatNhan" ? (
                <p className="text-xs text-muted-foreground">
                  Ví dụ: <code>在{"{n}"}身后</code> hoặc{" "}
                  <code>百分之{"{s}"}</code>.
                </p>
              ) : null}
            </div>

            <div className="grid gap-2">
              <Label htmlFor="dictionary-update-target">
                {dictionaryKey === "chinesePhienAmWords"
                  ? "Âm đọc, cách nhau bằng dấu cách"
                  : "Tiếng Việt"}
              </Label>
              <Input
                id="dictionary-update-target"
                value={target}
                onChange={(event) => setTarget(event.target.value)}
              />

              <div className="flex flex-wrap gap-2 pt-1" aria-label="Gợi ý bản dịch">
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  disabled={asyncDisabled}
                  onClick={() => void translateFromHanViet()}
                >
                  {pendingAction === "han-viet" ? (
                    <LoaderCircle className="animate-spin" />
                  ) : (
                    <Languages />
                  )}
                  Dùng âm Hán Việt
                </Button>
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  disabled={asyncDisabled}
                  onClick={() => void translateWithAi()}
                >
                  {pendingAction === "ai-translate" ? (
                    <LoaderCircle className="animate-spin" />
                  ) : (
                    <Sparkles />
                  )}
                  Dịch bằng AI
                </Button>
              </div>

              <div
                className="flex flex-wrap items-center gap-1.5 rounded-md border bg-muted/35 p-2"
                role="group"
                aria-label="Đổi kiểu chữ tiếng Việt"
              >
                <span className="mr-1 text-[11px] font-medium text-muted-foreground">
                  Kiểu chữ
                </span>
                <Button type="button" variant="ghost" size="xs" onClick={() => setTarget(titleCaseText(target))}>
                  Hoa Từng Từ
                </Button>
                <Button type="button" variant="ghost" size="xs" onClick={() => setTarget(lowercaseText(target))}>
                  viết thường
                </Button>
                <Button type="button" variant="ghost" size="xs" onClick={() => setTarget(sentenceCaseText(target))}>
                  Viết hoa câu
                </Button>
                <Button type="button" variant="ghost" size="xs" onClick={() => setTarget(uppercaseText(target))}>
                  VIẾT HOA
                </Button>
              </div>
            </div>

            <section className="grid min-w-0 gap-3 rounded-lg border bg-muted/20 p-3" aria-label="Tra nghĩa">
              <div className="flex min-w-0 flex-wrap items-center justify-between gap-2">
                <div className="min-w-0 flex-1 basis-72">
                  <p className="text-sm font-semibold">Tra nghĩa</p>
                  <p className="text-xs text-muted-foreground">
                    Lạc Việt tra tại bộ máy; AI dùng ngữ cảnh quanh cụm trong nguyên văn.
                  </p>
                  {context ? (
                    <p className="w-full truncate text-[11px] text-muted-foreground/80" title={context}>
                      Ngữ cảnh AI: {context}
                    </p>
                  ) : null}
                </div>
                <div className="flex max-w-full flex-wrap gap-2">
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    disabled={asyncDisabled}
                    onClick={() => void lookupLacViet()}
                  >
                    {pendingAction === "lac-viet" ? (
                      <LoaderCircle className="animate-spin" />
                    ) : (
                      <BookOpenText />
                    )}
                    Lạc Việt
                  </Button>
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    disabled={asyncDisabled}
                    onClick={() => void lookupWithAi()}
                  >
                    {pendingAction === "ai-meaning" ? (
                      <LoaderCircle className="animate-spin" />
                    ) : (
                      <BrainCircuit />
                    )}
                    Tra bằng AI
                  </Button>
                </div>
              </div>

              {lookupResult ? (
                <div className="rounded-md border bg-card p-3 text-sm" role="status">
                  <div className="mb-2 flex items-center justify-between gap-2">
                    <strong>{lookupResult.title}</strong>
                    <Button
                      type="button"
                      variant="ghost"
                      size="icon-xs"
                      aria-label="Đóng kết quả tra nghĩa"
                      onClick={() => setLookupResult(undefined)}
                    >
                      <X />
                    </Button>
                  </div>
                  {"entries" in lookupResult ? (
                    lookupResult.entries.length > 0 ? (
                      <div className="grid max-h-64 gap-3 overflow-y-auto pr-1">
                        {lookupResult.entries.map((entry) => (
                          <div key={entry.source} className="grid gap-1 border-t pt-2 first:border-t-0 first:pt-0">
                            <span lang="zh-Hans" className="font-semibold text-primary">
                              {entry.source}
                            </span>
                            <p className="whitespace-pre-wrap text-muted-foreground">
                              {entry.definition}
                            </p>
                          </div>
                        ))}
                      </div>
                    ) : (
                      <p className="text-muted-foreground">Không tìm thấy nghĩa Lạc Việt.</p>
                    )
                  ) : (
                    <p className="whitespace-pre-wrap text-muted-foreground">
                      {lookupResult.text}
                    </p>
                  )}
                </div>
              ) : null}
            </section>
          </div>

          <DialogFooter className="border-t px-6 py-4">
            {hasSavedEntry ? (
              <Button
                type="button"
                variant="ghost"
                className="mr-auto text-destructive hover:text-destructive"
                onClick={remove}
              >
                <Trash2 />
                Xóa bản vá
              </Button>
            ) : null}
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
            >
              Hủy
            </Button>
            <Button type="submit">
              <Save />
              Lưu local
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
