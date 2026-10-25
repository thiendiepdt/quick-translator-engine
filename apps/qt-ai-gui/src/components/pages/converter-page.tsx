import { ArrowLeft, Copy, FileType, FolderOpen, LoaderCircle } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";
import { z } from "zod";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Textarea } from "@/components/ui/textarea";
import { pickFiles, pickFolder, revealFolder, zhConvertFiles, zhConvertText, type ZhOutputMode } from "@/lib/api";
import { copyText } from "@/lib/clipboard";
import { zhConvertReportSchema } from "@/lib/schema";
import { useStoryStore } from "@/store/story";

type Report = z.infer<typeof zhConvertReportSchema>;
type Target = ZhOutputMode["kind"];

/** Gõ xong chừng này ms mới chuyển — dán cả chương cũng không bắn lệnh liên tục. */
const DEBOUNCE_MS = 250;
/** Hiện tối đa chừng này file bị bỏ qua; còn lại chỉ đếm. */
const SKIPPED_SHOWN = 20;

/** Folder chứa file (bỏ phần tên file sau dấu / hoặc \ cuối). */
const parentOf = (path: string) => path.slice(0, Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\"))) || path;

/** Ô chuyển nhanh: dán chữ phồn thể bên trái, bên phải ra giản thể ngay khi gõ. */
function TextConvert() {
  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");

  useEffect(() => {
    if (!input) return;
    let alive = true;
    const timer = setTimeout(() => {
      void zhConvertText(input)
        .then((text) => alive && setOutput(text))
        .catch((error: unknown) => alive && toast.error(error instanceof Error ? error.message : "Không chuyển được"));
    }, DEBOUNCE_MS);
    return () => {
      alive = false;
      clearTimeout(timer);
    };
  }, [input]);

  const shown = input ? output : "";
  return (
    <div className="grid gap-3 md:grid-cols-2">
      <div className="flex flex-col gap-1.5">
        <Label htmlFor="zh-input">Phồn thể</Label>
        <Textarea
          id="zh-input"
          rows={14}
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder="Dán chữ phồn thể vào đây, vd. 他著急地看著後臺"
          spellCheck={false}
        />
      </div>
      <div className="flex flex-col gap-1.5">
        <div className="flex items-center gap-2">
          <Label htmlFor="zh-output" className="flex-1">
            Giản thể
          </Label>
          <Button
            type="button"
            size="xs"
            variant="ghost"
            disabled={!shown}
            onClick={() =>
              void copyText(shown)
                .then(() => toast.success("Đã chép bản giản thể"))
                .catch((error: unknown) => toast.error(error instanceof Error ? error.message : "Không chép được"))
            }
          >
            <Copy /> Chép
          </Button>
        </div>
        <Textarea id="zh-output" rows={14} value={shown} readOnly spellCheck={false} className="bg-muted/40" />
      </div>
    </div>
  );
}

/** Chuyển file đã chọn (`folder = false`) hoặc cả folder; chọn cách ghi kết quả, chạy xong báo số file. */
function FileConvert({ folder }: { folder: boolean }) {
  const [inputs, setInputs] = useState<string[]>([]);
  const [recursive, setRecursive] = useState(true);
  const [extensions, setExtensions] = useState("txt");
  const [target, setTarget] = useState<Target>(folder ? "folder" : "suffix");
  const [dir, setDir] = useState("");
  const [suffix, setSuffix] = useState("_gian");
  const [busy, setBusy] = useState(false);
  const [report, setReport] = useState<Report>();

  const mode: ZhOutputMode | undefined =
    target === "replace"
      ? { kind: "replace" }
      : target === "folder"
        ? dir
          ? { kind: "folder", dir }
          : undefined
        : suffix.trim()
          ? { kind: "suffix", suffix: suffix.trim() }
          : undefined;
  const resultFolder = target === "folder" ? dir : folder ? inputs[0] : inputs[0] && parentOf(inputs[0]);

  async function choose() {
    const list = folder
      ? [await pickFolder("Chọn folder cần chuyển")].filter((path): path is string => Boolean(path))
      : await pickFiles("Chọn file cần chuyển");
    if (list.length > 0) {
      setInputs(list);
      setReport(undefined);
    }
  }

  async function chooseDir() {
    const picked = await pickFolder("Chọn folder để ghi bản giản thể");
    if (picked) setDir(picked);
  }

  async function run() {
    if (!mode) return;
    if (target === "replace" && !window.confirm("Ghi đè file gốc bằng bản giản thể? Không hoàn tác được.")) return;
    setBusy(true);
    try {
      const exts = extensions
        .split(/[,\s]+/)
        .map((ext) => ext.trim().replace(/^\./, ""))
        .filter(Boolean);
      setReport(await zhConvertFiles({ inputs, recursive, extensions: folder ? exts : [], mode }));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không chuyển được");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-col gap-2 rounded-lg border bg-card p-4">
        <div className="flex flex-wrap items-center gap-2">
          <Button type="button" variant="outline" onClick={() => void choose()}>
            <FolderOpen /> {folder ? "Chọn folder…" : "Chọn file…"}
          </Button>
          <span className="min-w-0 flex-1 truncate font-mono text-xs text-muted-foreground" title={inputs.join("\n")}>
            {inputs.length === 0 ? "Chưa chọn" : inputs.length === 1 ? inputs[0] : `${inputs.length} file: ${inputs[0]}, …`}
          </span>
        </div>
        {folder && (
          <div className="flex flex-wrap items-center gap-x-4 gap-y-2 text-sm">
            <label className="flex items-center gap-1.5">
              <input type="checkbox" checked={recursive} onChange={(e) => setRecursive(e.target.checked)} />
              Gồm thư mục con
            </label>
            <label className="flex items-center gap-1.5">
              Đuôi file
              <Input
                aria-label="Đuôi file"
                value={extensions}
                onChange={(e) => setExtensions(e.target.value)}
                placeholder="txt, md — trống = mọi file text"
                className="h-8 w-56 font-mono"
              />
            </label>
          </div>
        )}
      </div>

      <fieldset className="flex flex-col gap-2 rounded-lg border bg-card p-4 text-sm">
        <legend className="px-1 text-sm font-medium">Ghi kết quả</legend>
        <label className="flex flex-wrap items-center gap-2">
          <input type="radio" name={`zh-target-${folder}`} checked={target === "folder"} onChange={() => setTarget("folder")} />
          Ra folder khác{folder ? " (giữ cấu trúc thư mục con)" : ""}
          {target === "folder" && (
            <>
              <Button type="button" size="xs" variant="outline" onClick={() => void chooseDir()}>
                <FolderOpen /> Chọn folder đích…
              </Button>
              <span className="min-w-0 truncate font-mono text-xs text-muted-foreground">{dir || "Chưa chọn"}</span>
            </>
          )}
        </label>
        <label className="flex flex-wrap items-center gap-2">
          <input type="radio" name={`zh-target-${folder}`} checked={target === "suffix"} onChange={() => setTarget("suffix")} />
          Cạnh file gốc, thêm hậu tố vào tên
          {target === "suffix" && (
            <>
              <Input
                aria-label="Hậu tố tên file"
                value={suffix}
                onChange={(e) => setSuffix(e.target.value)}
                className="h-8 w-28 font-mono"
              />
              <span className="font-mono text-xs text-muted-foreground">vd. chuong1{suffix.trim()}.txt</span>
            </>
          )}
        </label>
        <label className="flex items-center gap-2">
          <input type="radio" name={`zh-target-${folder}`} checked={target === "replace"} onChange={() => setTarget("replace")} />
          Ghi đè file gốc <span className="text-xs text-muted-foreground">(không hoàn tác được)</span>
        </label>
      </fieldset>

      <div className="flex items-center gap-2">
        <Button type="button" disabled={busy || inputs.length === 0 || !mode} onClick={() => void run()}>
          {busy ? <LoaderCircle className="animate-spin" /> : <FileType />} Chuyển sang giản thể
        </Button>
        {report && resultFolder && (
          <Button type="button" variant="ghost" onClick={() => void revealFolder(resultFolder)}>
            <FolderOpen /> Mở folder kết quả
          </Button>
        )}
      </div>

      {report && (
        <div role="status" aria-label="Kết quả chuyển" className="rounded-md border bg-muted/40 px-3 py-2 text-sm">
          <p>
            Đã chuyển {report.converted} file sang giản thể
            {report.unchanged > 0 &&
              `; ${report.unchanged} file vốn là giản thể UTF-8 ${target === "replace" ? "nên để yên" : "vẫn ghi bản sao"}`}
            .{report.skipped.length > 0 && ` Bỏ qua ${report.skipped.length} file.`}
          </p>
          {report.skipped.length > 0 && (
            <ul className="mt-1 list-disc pl-5 text-xs text-muted-foreground">
              {report.skipped.slice(0, SKIPPED_SHOWN).map((item) => (
                <li key={item.path}>
                  <span className="font-mono">{item.path}</span> — {item.reason}
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
    </div>
  );
}

/**
 * Công cụ chuyển phồn thể → giản thể, không gắn với truyện nào: ô chuyển nhanh cụm chữ, một hay nhiều file, cả
 * folder. Mở từ màn chọn truyện hoặc thanh trái trong truyện; nút quay lại về đúng chỗ đã mở.
 */
export function ConverterPage() {
  const closeConverter = useStoryStore((s) => s.closeConverter);
  const toolFrom = useStoryStore((s) => s.toolFrom);
  return (
    <div className="fine-scrollbar h-full min-h-0 overflow-y-auto">
      <div className="mx-auto flex max-w-4xl flex-col gap-4 px-6 py-6">
        <div className="flex items-start gap-3">
          <Button type="button" variant="outline" size="sm" onClick={closeConverter}>
            <ArrowLeft /> {toolFrom === "workbench" ? "Về truyện" : "Về danh sách truyện"}
          </Button>
          <div className="min-w-0">
            <h1 className="text-lg font-semibold">Chuyển phồn thể → giản thể</h1>
            <p className="text-xs text-muted-foreground">
              Công cụ riêng, không gắn với truyện nào. Chuyển theo cụm từ, chuẩn Đại lục: 看著 → 看着 nhưng giữ 著名,
              乾隆; 滑鼠 → 鼠标; 「」 → “”. File GBK, Big5, UTF-16 được đọc đúng và ghi ra UTF-8.
            </p>
          </div>
        </div>
        <Tabs defaultValue="text">
          <TabsList aria-label="Chuyển gì">
            <TabsTrigger value="text">Cụm từ</TabsTrigger>
            <TabsTrigger value="file">File</TabsTrigger>
            <TabsTrigger value="folder">Folder</TabsTrigger>
          </TabsList>
          <TabsContent value="text">
            <TextConvert />
          </TabsContent>
          <TabsContent value="file">
            <FileConvert folder={false} />
          </TabsContent>
          <TabsContent value="folder">
            <FileConvert folder />
          </TabsContent>
        </Tabs>
      </div>
    </div>
  );
}
