import { Plus, Save, Trash2 } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { vnBuiltinEntries, vnSettingsGet, vnSettingsSave } from "@/lib/api";
import type { VnEntry, VnReplacement, VnSettings } from "@/lib/types";
import { VN_GROUP_LABELS } from "@/lib/types";

const SELECT_CLASS = "h-8 rounded-md border border-input bg-background px-2 text-sm";

/** Mục riêng đang sửa: chữ Hán / tiếng Việt giữ dạng chuỗi "a, b" để gõ dấu phẩy không bị nuốt. */
interface ExtraDraft {
  id: string;
  group: VnEntry["group"];
  mietthi: boolean;
  weak: boolean;
  zh: string;
  vi: string;
  exclude: string[];
}

const splitList = (text: string) =>
  text
    .split(/[,，]/)
    .map((part) => part.trim())
    .filter(Boolean);

const toDraft = (entry: VnEntry): ExtraDraft => ({
  id: entry.id,
  group: entry.group,
  mietthi: entry.level === "mietthi",
  weak: entry.weak,
  zh: entry.zh.join(", "),
  vi: entry.vi.join(", "),
  exclude: entry.exclude,
});

const fromDraft = (draft: ExtraDraft): VnEntry => ({
  id: draft.id,
  group: draft.group,
  level: draft.mietthi ? "mietthi" : "canxem",
  weak: draft.weak,
  zh: splitList(draft.zh),
  vi: splitList(draft.vi),
  exclude: draft.exclude,
});

function nextExtraId(rows: ExtraDraft[]): string {
  let n = rows.length + 1;
  while (rows.some((row) => row.id === `rieng-${n}`)) n += 1;
  return `rieng-${n}`;
}

/** Cấu hình kiểm tra VN (Cài đặt): bật/tắt mục có sẵn, mục riêng, bảng thay. Lưu ở vn-terms.json cạnh bản mặc định. */
export function VnSettingsEditor() {
  const [builtin, setBuiltin] = useState<VnEntry[]>([]);
  const [disabled, setDisabled] = useState<string[]>();
  const [extra, setExtra] = useState<ExtraDraft[]>([]);
  const [replacements, setReplacements] = useState<VnReplacement[]>([]);
  const [busy, setBusy] = useState(false);

  function load(settings: VnSettings) {
    setDisabled(settings.disabled);
    setExtra(settings.extra.map(toDraft));
    setReplacements(settings.replacements);
  }

  useEffect(() => {
    void Promise.all([vnBuiltinEntries(), vnSettingsGet()])
      .then(([entries, settings]) => {
        setBuiltin(entries);
        load(settings);
      })
      .catch(() => toast.error("Không đọc được cấu hình kiểm tra VN"));
  }, []);

  if (!disabled) return <p className="text-sm text-muted-foreground">Đang tải…</p>;

  const updateRow = (index: number, patch: Partial<VnReplacement>) =>
    setReplacements((rows) => rows.map((row, i) => (i === index ? { ...row, ...patch } : row)));
  const updateExtra = (index: number, patch: Partial<ExtraDraft>) =>
    setExtra((rows) => rows.map((row, i) => (i === index ? { ...row, ...patch } : row)));

  async function save() {
    if (!disabled) return;
    setBusy(true);
    try {
      load(
        await vnSettingsSave({
          extra: extra.map(fromDraft).filter((entry) => entry.zh.length + entry.vi.length > 0),
          disabled,
          replacements: replacements.filter((row) => row.from.trim() && row.to.trim()),
        }),
      );
      toast.success("Đã lưu cấu hình kiểm tra VN");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không lưu được");
    } finally {
      setBusy(false);
    }
  }

  return (
    <>
      <div className="flex flex-col gap-2">
        <h3 className="text-sm font-medium">Mục có sẵn</h3>
        <p className="text-xs text-muted-foreground">Bỏ tick mục hay báo nhầm với truyện của bạn.</p>
        <div className="fine-scrollbar grid max-h-56 gap-1 overflow-y-auto rounded-md border p-2 text-sm sm:grid-cols-2">
          {builtin.map((entry) => (
            <label key={entry.id} className="flex items-start gap-2">
              <input
                type="checkbox"
                className="mt-1"
                checked={!disabled.includes(entry.id)}
                onChange={(e) =>
                  setDisabled((list = []) => (e.target.checked ? list.filter((id) => id !== entry.id) : [...list, entry.id]))
                }
              />
              <span>
                <span className="font-mono text-xs">{entry.id}</span>{" "}
                <span className="text-muted-foreground">
                  {VN_GROUP_LABELS[entry.group]} · {[...entry.vi, ...entry.zh].slice(0, 4).join(", ")}
                </span>
              </span>
            </label>
          ))}
        </div>
      </div>

      <div className="flex flex-col gap-2">
        <h3 className="text-sm font-medium">Mục riêng</h3>
        <p className="text-xs text-muted-foreground">
          Nhiều dạng cách nhau bằng dấu phẩy. Chỗ khớp trong bản dịch chỉ tính khi đoạn raw tương ứng có một dạng chữ Hán.
        </p>
        <ul aria-label="Mục riêng" className="flex flex-col gap-2">
          {extra.map((row, index) => (
            <li key={row.id} className="grid grid-cols-[1fr_1fr_9rem_auto_auto_auto] items-center gap-2 text-sm">
              <Input aria-label="Chữ Hán" value={row.zh} onChange={(e) => updateExtra(index, { zh: e.target.value })} />
              <Input aria-label="Tiếng Việt" value={row.vi} onChange={(e) => updateExtra(index, { vi: e.target.value })} />
              <select
                aria-label="Nhóm"
                className={SELECT_CLASS}
                value={row.group}
                onChange={(e) => updateExtra(index, { group: e.target.value as VnEntry["group"] })}
              >
                {Object.entries(VN_GROUP_LABELS).map(([id, label]) => (
                  <option key={id} value={id}>
                    {label}
                  </option>
                ))}
              </select>
              <label className="flex items-center gap-1">
                <input type="checkbox" checked={row.weak} onChange={(e) => updateExtra(index, { weak: e.target.checked })} />
                Từ yếu
              </label>
              <label className="flex items-center gap-1">
                <input type="checkbox" checked={row.mietthi} onChange={(e) => updateExtra(index, { mietthi: e.target.checked })} />
                Miệt thị
              </label>
              <Button
                type="button"
                size="icon-sm"
                variant="ghost"
                aria-label={`Xoá mục ${row.id}`}
                onClick={() => setExtra((rows) => rows.filter((_, i) => i !== index))}
              >
                <Trash2 />
              </Button>
            </li>
          ))}
        </ul>
        <div>
          <Button
            type="button"
            size="sm"
            variant="outline"
            onClick={() =>
              setExtra((rows) => [...rows, { id: nextExtraId(rows), group: "ten", mietthi: false, weak: false, zh: "", vi: "", exclude: [] }])
            }
          >
            <Plus /> Thêm mục riêng
          </Button>
        </div>
      </div>

      <div className="flex flex-col gap-2">
        <h3 className="text-sm font-medium">Bảng thay</h3>
        <p className="text-xs text-muted-foreground">
          Gợi ý điền sẵn khi bấm "Thay thế…" ở tab Kiểm tra VN. Dòng "Cổ đại" gợi ý cho truyện cổ đại, "Hiện đại" cho truyện hiện
          đại / hỗn hợp.
        </p>
        <ul aria-label="Bảng thay" className="flex flex-col gap-2">
          {replacements.map((row, index) => (
            <li key={index} className="grid grid-cols-[1fr_1fr_8rem_auto] items-center gap-2">
              <Input aria-label="Cụm gốc" value={row.from} onChange={(e) => updateRow(index, { from: e.target.value })} />
              <Input aria-label="Cụm thay" value={row.to} onChange={(e) => updateRow(index, { to: e.target.value })} />
              <select
                aria-label="Bối cảnh"
                className={SELECT_CLASS}
                value={row.setting}
                onChange={(e) => updateRow(index, { setting: e.target.value as VnReplacement["setting"] })}
              >
                <option value="ancient">Cổ đại</option>
                <option value="modern">Hiện đại</option>
                <option value="any">Mọi truyện</option>
              </select>
              <Button
                type="button"
                size="icon-sm"
                variant="ghost"
                aria-label={`Xoá dòng thay ${index + 1}`}
                onClick={() => setReplacements((rows) => rows.filter((_, i) => i !== index))}
              >
                <Trash2 />
              </Button>
            </li>
          ))}
        </ul>
        <div>
          <Button
            type="button"
            size="sm"
            variant="outline"
            onClick={() => setReplacements((rows) => [...rows, { from: "", to: "", setting: "any" }])}
          >
            <Plus /> Thêm dòng thay
          </Button>
        </div>
      </div>

      <div className="flex justify-end">
        <Button type="button" size="sm" disabled={busy} onClick={() => void save()}>
          <Save /> Lưu cấu hình VN
        </Button>
      </div>
    </>
  );
}
