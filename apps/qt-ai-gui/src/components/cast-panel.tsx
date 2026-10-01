import { AlertTriangle, Eraser, Pin, PinOff, Plus, Save, Search, Sparkles, Undo2 } from "lucide-react";
import { useEffect, useMemo, useState } from "react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { castCleanAddressing, castLoad, castSave, castScan } from "@/lib/api";
import type { Cast, Gender } from "@/lib/types";

interface Props {
  root: string;
  /** `glossary.names` đã lưu của truyện (Hán → Việt). */
  names: Record<string, string>;
  /** `glossary.addressing` đã lưu — cặp gốc học lần đầu. */
  addressing: Record<string, string>;
  chapterIds: string[];
  running: boolean;
  /** Dọn cặp trái giới sửa story.json → trang cha nạp lại snapshot. */
  onStoryChanged: () => void;
}

/** Bảng hiện từng khúc chừng này dòng — hàng trăm ô chọn cùng lúc gây lag (như GlossaryEditor). */
const PAGE = 50;
const SELECT_CLASS = "h-8 rounded-md border border-input bg-background px-2 text-sm";

/**
 * Bảng nhân vật (`cast.json`): giới tính từng tên trong glossary và các mốc đổi xưng hô theo chương.
 * Tách khỏi form hồ sơ truyện vì dữ liệu nằm ở file riêng, có nút Lưu riêng.
 */
export function CastPanel({ root, names, addressing, chapterIds, running, onStoryChanged }: Props) {
  const [saved, setSaved] = useState<Cast>();
  const [draft, setDraft] = useState<Cast>();
  const [status, setStatus] = useState("");
  const [busy, setBusy] = useState(false);
  const [query, setQuery] = useState("");
  const [onlyUnknown, setOnlyUnknown] = useState(false);
  const [limit, setLimit] = useState(PAGE);
  const [pairKey, setPairKey] = useState("");
  const [fromChapter, setFromChapter] = useState("");
  const [newTarget, setNewTarget] = useState("");

  useEffect(() => {
    let alive = true;
    void castLoad(root)
      .then((cast) => {
        if (!alive) return;
        setSaved(cast);
        setDraft(cast);
      })
      .catch((error: unknown) => alive && setStatus(error instanceof Error ? error.message : "Không đọc được bảng nhân vật"));
    return () => {
      alive = false;
    };
  }, [root]);

  const dirty = useMemo(() => JSON.stringify(saved) !== JSON.stringify(draft), [saved, draft]);
  const pairKeys = useMemo(
    () => [...new Set([...Object.keys(addressing), ...Object.keys(draft?.addressing ?? {})])],
    [addressing, draft],
  );
  if (!draft) return <p className="text-sm text-muted-foreground">{status || "Đang tải bảng nhân vật…"}</p>;

  const needle = query.trim().toLowerCase();
  const rows = Object.entries(names).filter(([han, viet]) => {
    if (onlyUnknown && draft.characters[han]?.gender) return false;
    return !needle || han.toLowerCase().includes(needle) || viet.toLowerCase().includes(needle);
  });
  const known = Object.keys(names).filter((han) => draft.characters[han]?.gender).length;
  const timelines = Object.entries(draft.addressing).filter(([, timeline]) => timeline.pinned || timeline.changes.length > 0);

  function setGender(han: string, value: string) {
    setDraft((current) => {
      if (!current) return current;
      const characters = { ...current.characters };
      if (value === "male" || value === "female") {
        characters[han] = { ...characters[han], gender: value satisfies Gender, source: "user" };
      } else {
        delete characters[han];
      }
      return { ...current, characters };
    });
  }

  function updateTimeline(key: string, update: (timeline: Cast["addressing"][string]) => Cast["addressing"][string]) {
    setDraft((current) => {
      if (!current) return current;
      const next = update(current.addressing[key] ?? { pinned: false, changes: [] });
      return { ...current, addressing: { ...current.addressing, [key]: next } };
    });
  }

  function addMilestone() {
    const target = newTarget.trim();
    if (!pairKey || !fromChapter || !target) return;
    const order = (id: string) => chapterIds.indexOf(id);
    updateTimeline(pairKey, (timeline) => ({
      pinned: true,
      changes: [...timeline.changes.filter((change) => change.from !== fromChapter), { from: fromChapter, target, source: "user" as const }].sort(
        (a, b) => order(a.from) - order(b.from),
      ),
    }));
    setNewTarget("");
  }

  async function run(task: () => Promise<string>) {
    setBusy(true);
    try {
      setStatus(await task());
    } catch (error) {
      setStatus(error instanceof Error ? error.message : "Lệnh thất bại");
    } finally {
      setBusy(false);
    }
  }

  const save = () =>
    run(async () => {
      const next = await castSave(root, draft);
      setSaved(next);
      setDraft(next);
      return "Đã lưu bảng nhân vật";
    });
  const scan = () =>
    run(async () => {
      const view = await castScan(root);
      setSaved(view.cast);
      setDraft(view.cast);
      const failed = view.failedBatches ? ` (${view.failedBatches} lô model trả hỏng, bấm quét lại)` : "";
      return `Đã điền ${view.filled} / ${view.asked} tên chưa rõ giới${failed}`;
    });
  const clean = () =>
    run(async () => {
      const removed = await castCleanAddressing(root);
      if (removed.length === 0) return "Không có cặp xưng hô nào trái giới";
      onStoryChanged();
      return `Đã xoá ${removed.length} cặp trái giới khỏi glossary: ${removed.join(", ")}`;
    });

  return (
    <div className="flex flex-col gap-4">
      <p className="text-xs text-muted-foreground">
        Giới tính ở đây quyết định thầy/cô, anh/chị, ông/bà… trong bản dịch và được check tự động. Model tự khai khi dịch
        chương mới; sửa tay ở đây luôn thắng. Lưu ở file <code>cast.json</code> cạnh story.json.
      </p>
      <div className="flex flex-wrap items-center gap-2">
        <Button type="button" size="sm" variant="secondary" disabled={busy || running || dirty} onClick={() => void scan()}>
          <Sparkles /> Quét giới tính (AI)
        </Button>
        <Button type="button" size="sm" variant="outline" disabled={busy || running || dirty} onClick={() => void clean()}>
          <Eraser /> Dọn cặp trái giới
        </Button>
        <div className="flex-1" />
        {dirty && <span className="text-xs text-muted-foreground">Có thay đổi chưa lưu</span>}
        <Button type="button" size="sm" disabled={busy || !dirty} onClick={() => void save()}>
          <Save /> Lưu bảng nhân vật
        </Button>
      </div>
      {status && (
        <p role="status" className="rounded-md border bg-muted/40 px-3 py-2 text-xs">
          {status}
        </p>
      )}

      <fieldset className="rounded-lg border bg-card p-4">
        <div className="mb-3 flex flex-wrap items-center gap-2">
          <legend className="contents text-sm font-semibold">Giới tính nhân vật</legend>
          <span className="rounded-full bg-muted px-1.5 text-xs text-muted-foreground">
            {known} / {Object.keys(names).length}
          </span>
          <div className="flex-1" />
          <label className="flex items-center gap-1.5 text-xs text-muted-foreground">
            <input type="checkbox" checked={onlyUnknown} onChange={(e) => setOnlyUnknown(e.target.checked)} />
            Chỉ tên chưa rõ giới
          </label>
          <div className="relative">
            <Search className="pointer-events-none absolute top-1/2 left-2 size-3.5 -translate-y-1/2 text-muted-foreground" />
            <Input
              value={query}
              onChange={(e) => {
                setQuery(e.target.value);
                setLimit(PAGE);
              }}
              placeholder="Tìm…"
              aria-label="Tìm nhân vật"
              className="h-7 w-40 pl-7 text-xs"
            />
          </div>
        </div>
        {rows.length === 0 && <p className="py-2 text-xs text-muted-foreground">Không có tên nào.</p>}
        <ul className="flex flex-col gap-1">
          {rows.slice(0, limit).map(([han, viet]) => {
            const character = draft.characters[han];
            return (
              <li key={han} className="grid grid-cols-[1fr_1fr_auto_1.5rem] items-center gap-2">
                <span className="font-mono text-sm">{han}</span>
                <span className="truncate text-sm">{viet}</span>
                <select
                  aria-label={`Giới tính ${han}`}
                  className={SELECT_CLASS}
                  value={character?.gender ?? ""}
                  onChange={(e) => setGender(han, e.target.value)}
                >
                  <option value="">Chưa rõ</option>
                  <option value="male">Nam</option>
                  <option value="female">Nữ</option>
                </select>
                {character?.disputed?.length ? (
                  <span title={`Chương ${character.disputed.join(", ")} khai giới ngược — kiểm tra lại`}>
                    <AlertTriangle className="size-4 text-destructive" />
                  </span>
                ) : (
                  <span />
                )}
              </li>
            );
          })}
        </ul>
        {rows.length > limit && (
          <Button type="button" size="xs" variant="ghost" className="mt-1" onClick={() => setLimit((l) => l + PAGE)}>
            Hiện thêm {Math.min(PAGE, rows.length - limit)} / còn {rows.length - limit}
          </Button>
        )}
      </fieldset>

      <fieldset className="rounded-lg border bg-card p-4">
        <legend className="contents text-sm font-semibold">Xưng hô đổi theo chương</legend>
        <p className="mt-1 mb-3 text-xs text-muted-foreground">
          Cặp gốc nằm ở Glossary → Xưng hô theo cặp. Khi quan hệ đổi bền (thành người yêu, cưới…), model khai cặp mới và
          app áp từ chương đó trở đi. Ghim cặp để app không tự đổi nữa.
        </p>
        {timelines.length === 0 && <p className="py-2 text-xs text-muted-foreground">Chưa có mốc nào.</p>}
        <ul className="flex flex-col gap-2" aria-label="Mốc đổi xưng hô">
          {timelines.map(([key, timeline]) => (
            <li key={key} className="rounded-md border bg-muted/25 p-2">
              <div className="flex items-center gap-2">
                <span className="font-mono text-sm">{key}</span>
                <span className="text-xs text-muted-foreground">gốc: {addressing[key] ?? "—"}</span>
                <div className="flex-1" />
                <Button
                  type="button"
                  size="icon-sm"
                  variant="ghost"
                  aria-label={`${timeline.pinned ? "Bỏ ghim" : "Ghim"} cặp ${key}`}
                  title={timeline.pinned ? "Đang ghim: app không tự đổi cặp này" : "Ghim: app không tự đổi cặp này nữa"}
                  onClick={() => updateTimeline(key, (t) => ({ ...t, pinned: !t.pinned }))}
                >
                  {timeline.pinned ? <Pin /> : <PinOff />}
                </Button>
              </div>
              {timeline.changes.map((change) => (
                <div key={change.from} className="mt-1 flex items-center gap-2 text-sm">
                  <span className="text-xs text-muted-foreground">từ chương {change.from}</span>
                  <span className="font-medium">{change.target}</span>
                  <span className="truncate text-xs text-muted-foreground">
                    {change.note ? `${change.note} · ` : ""}
                    {change.source === "user" ? "nhập tay" : "model khai"}
                  </span>
                  <div className="flex-1" />
                  <Button
                    type="button"
                    size="icon-sm"
                    variant="ghost"
                    aria-label={`Hoàn tác mốc ${change.from}`}
                    onClick={() => updateTimeline(key, (t) => ({ ...t, changes: t.changes.filter((c) => c.from !== change.from) }))}
                  >
                    <Undo2 />
                  </Button>
                </div>
              ))}
            </li>
          ))}
        </ul>
        <div className="mt-3 grid grid-cols-[1fr_1fr_1fr_auto] items-end gap-2">
          <label className="flex flex-col gap-1 text-xs">
            Cặp xưng hô
            <select aria-label="Cặp xưng hô" className={SELECT_CLASS} value={pairKey} onChange={(e) => setPairKey(e.target.value)}>
              <option value="">Chọn cặp…</option>
              {pairKeys.map((key) => (
                <option key={key} value={key}>
                  {key}
                </option>
              ))}
            </select>
          </label>
          <label className="flex flex-col gap-1 text-xs">
            Từ chương
            <select aria-label="Từ chương" className={SELECT_CLASS} value={fromChapter} onChange={(e) => setFromChapter(e.target.value)}>
              <option value="">Chọn chương…</option>
              {chapterIds.map((id) => (
                <option key={id} value={id}>
                  {id}
                </option>
              ))}
            </select>
          </label>
          <label className="flex flex-col gap-1 text-xs">
            Xưng hô mới
            <Input aria-label="Xưng hô mới" className="h-8" placeholder="em–anh" value={newTarget} onChange={(e) => setNewTarget(e.target.value)} />
          </label>
          <Button type="button" size="sm" variant="secondary" disabled={!pairKey || !fromChapter || !newTarget.trim()} onClick={addMilestone}>
            <Plus /> Thêm mốc
          </Button>
        </div>
      </fieldset>
    </div>
  );
}
