import {
  flip,
  offset,
  useFloatingToolbar,
  useFloatingToolbarState,
} from "@platejs/floating";
import { ListStyleType, someList, toggleList } from "@platejs/list";
import { MarkdownPlugin } from "@platejs/markdown";
import {
  deleteColumn,
  deleteRow,
  deleteTable,
  getTableAbove,
  insertTable,
  insertTableColumn,
  insertTableRow,
} from "@platejs/table";
import {
  Bold,
  Code2,
  Heading1,
  Heading2,
  Heading3,
  Italic,
  List,
  ListOrdered,
  Minus,
  Pilcrow,
  Strikethrough,
  Table,
  TextQuote,
  Trash2,
} from "lucide-react";
import {
  Plate,
  PlateContent,
  PlateElement,
  useEditorId,
  useEditorRef,
  useEditorSelector,
  useEventEditorValue,
  usePlateEditor,
  type PlateEditor,
  type PlateElementProps,
} from "platejs/react";
import { useCallback, useEffect, useRef, type ReactNode } from "react";

import { promptEditorPlugins } from "@/components/plate-prompt-editor-plugins";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

/** Gõ xong ngừng chừng này mới serialize Markdown và báo lên form — serialize cả prompt mỗi phím làm giật chữ. */
export const SERIALIZE_DEBOUNCE_MS = 300;

interface PlatePromptEditorProps {
  initialValue: string;
  onChange: (markdown: string) => void;
  /** Cho test: nhận editor để chèn chữ/chọn vùng mà không cần gõ qua contenteditable. */
  onEditor?: (editor: PlateEditor) => void;
}

interface MarkButtonProps {
  icon: ReactNode;
  label: string;
  mark: "bold" | "code" | "italic" | "strikethrough";
}

function MarkButton({ icon, label, mark }: MarkButtonProps) {
  const editor = useEditorRef();
  const active = useEditorSelector((currentEditor) => currentEditor.api.hasMark(mark), [mark]);
  return (
    <Button
      type="button"
      variant={active ? "secondary" : "ghost"}
      size="icon-xs"
      aria-label={label}
      aria-pressed={active}
      onMouseDown={(event) => {
        event.preventDefault();
        editor.tf.toggleMark(mark);
        editor.tf.focus();
      }}
    >
      {icon}
    </Button>
  );
}

function PromptFloatingToolbar() {
  const editorId = useEditorId();
  const focusedEditorId = useEventEditorValue("focus");
  const state = useFloatingToolbarState({
    editorId,
    focusedEditorId,
    floatingOptions: {
      middleware: [offset(8), flip({ padding: 12 })],
      placement: "top",
    },
  });
  const { clickOutsideRef, hidden, props, ref } = useFloatingToolbar(state);

  if (hidden) return null;
  return (
    <div ref={clickOutsideRef}>
      <div
        ref={ref}
        {...props}
        className="z-50 flex items-center gap-0.5 rounded-lg border bg-popover p-1 text-popover-foreground shadow-lg"
      >
        <MarkButton mark="bold" label="In đậm" icon={<Bold />} />
        <MarkButton mark="italic" label="In nghiêng" icon={<Italic />} />
        <MarkButton mark="strikethrough" label="Gạch ngang chữ" icon={<Strikethrough />} />
        <MarkButton mark="code" label="Mã nội dòng" icon={<Code2 />} />
      </div>
    </div>
  );
}

/** Nút thanh công cụ: mousedown chặn mặc định để editor giữ nguyên focus và selection khi bấm. */
function ToolButton({
  label,
  title,
  disabled,
  destructive,
  onAction,
  children,
}: {
  label: string;
  title?: string;
  disabled?: boolean;
  destructive?: boolean;
  onAction: () => void;
  children: ReactNode;
}) {
  return (
    <Button
      type="button"
      variant="ghost"
      size="xs"
      aria-label={label}
      title={title ?? label}
      disabled={disabled}
      className={cn(destructive && "text-destructive hover:text-destructive")}
      onMouseDown={(event) => {
        event.preventDefault();
        onAction();
      }}
    >
      {children}
    </Button>
  );
}

/**
 * Nút đổi kiểu khối: sáng khi khối tại con trỏ đang là kiểu đó. Tiêu đề/đoạn đổi `type` tại chỗ;
 * blockquote là khối bọc (blockquote > p) nên phải `wrap` — setNodes trên nó làm normalizer chèn p và
 * selection trôi sang khối kế tiếp.
 */
function BlockButton({ type, label, icon, wrap }: { type: string; label: string; icon: ReactNode; wrap?: boolean }) {
  const editor = useEditorRef();
  const active = useEditorSelector((currentEditor) => currentEditor.api.some({ match: { type } }), [type]);
  return (
    <Button
      type="button"
      variant={active ? "secondary" : "ghost"}
      size="icon-xs"
      aria-label={label}
      aria-pressed={active}
      title={label}
      onMouseDown={(event) => {
        event.preventDefault();
        editor.tf.toggleBlock(type, wrap ? { wrap: true } : undefined);
      }}
    >
      {icon}
    </Button>
  );
}

/** Nút danh sách gạch đầu dòng / đánh số (kiểu indent-list của Plate, markdown ra "-" và "1."). */
function ListButton({ style, label, icon }: { style: string; label: string; icon: ReactNode }) {
  const editor = useEditorRef();
  const active = useEditorSelector((currentEditor) => someList(currentEditor, style), [style]);
  return (
    <Button
      type="button"
      variant={active ? "secondary" : "ghost"}
      size="icon-xs"
      aria-label={label}
      aria-pressed={active}
      title={label}
      onMouseDown={(event) => {
        event.preventDefault();
        toggleList(editor, { listStyleType: style });
      }}
    >
      {icon}
    </Button>
  );
}

function Separator() {
  return <span className="mx-1 h-4 w-px bg-border" aria-hidden />;
}

/**
 * Thanh công cụ cố định trên editor: kiểu khối, danh sách, gạch ngang, đậm/nghiêng, bảng; con trỏ
 * trong bảng thì thêm nút thêm/xoá dòng, cột, cả bảng. Toolbar nổi khi bôi đen vẫn có đậm/nghiêng.
 */
function PromptToolbar() {
  const editor = useEditorRef();
  const inTable = useEditorSelector((currentEditor) => Boolean(getTableAbove(currentEditor)), []);
  return (
    <div
      role="toolbar"
      aria-label="Công cụ prompt"
      className="flex flex-wrap items-center gap-0.5 rounded-t-md border border-b-0 bg-muted/40 px-2 py-1"
    >
      <BlockButton type="p" label="Đoạn văn" icon={<Pilcrow />} />
      <BlockButton type="h1" label="Tiêu đề 1" icon={<Heading1 />} />
      <BlockButton type="h2" label="Tiêu đề 2" icon={<Heading2 />} />
      <BlockButton type="h3" label="Tiêu đề 3" icon={<Heading3 />} />
      <BlockButton type="blockquote" label="Trích dẫn" icon={<TextQuote />} wrap />
      <Separator />
      <ListButton style={ListStyleType.Disc} label="Danh sách gạch đầu dòng" icon={<List />} />
      <ListButton style={ListStyleType.Decimal} label="Danh sách đánh số" icon={<ListOrdered />} />
      <ToolButton
        label="Gạch ngang"
        title="Chèn đường kẻ ngang (---) sau khối hiện tại"
        onAction={() => {
          // Chèn hr ngay sau khối chứa con trỏ rồi đưa con trỏ xuống khối kế; hr là khối cuối thì thêm
          // một đoạn trống để còn gõ tiếp. Không chèn đoạn trống khi không cần: đoạn rỗng ra markdown thành ZWSP.
          const block = editor.api.block();
          if (!block) return;
          const path = block[1];
          const hrPath = [...path.slice(0, -1), path[path.length - 1] + 1];
          const afterPath = [...path.slice(0, -1), path[path.length - 1] + 2];
          editor.tf.insertNodes({ type: "hr", children: [{ text: "" }] }, { at: hrPath });
          if (!editor.api.node(afterPath)) {
            editor.tf.insertNodes({ type: "p", children: [{ text: "" }] }, { at: afterPath });
          }
          editor.tf.select(editor.api.start(afterPath));
        }}
      >
        <Minus />
      </ToolButton>
      <Separator />
      <MarkButton mark="bold" label="In đậm" icon={<Bold />} />
      <MarkButton mark="italic" label="In nghiêng" icon={<Italic />} />
      <MarkButton mark="strikethrough" label="Gạch ngang chữ" icon={<Strikethrough />} />
      <MarkButton mark="code" label="Mã nội dòng" icon={<Code2 />} />
      <Separator />
      <ToolButton
        label="Chèn bảng"
        title="Chèn bảng 3 cột × 2 dòng (dòng đầu là tiêu đề)"
        onAction={() => {
          insertTable(editor, { rowCount: 2, colCount: 3, header: true }, { select: true });
          editor.tf.focus();
        }}
      >
        <Table /> Chèn bảng
      </ToolButton>
      {inTable && (
        <>
          <Separator />
          <ToolButton label="Thêm dòng trên" onAction={() => insertTableRow(editor, { before: true })}>
            Dòng ↑
          </ToolButton>
          <ToolButton label="Thêm dòng dưới" onAction={() => insertTableRow(editor)}>
            Dòng ↓
          </ToolButton>
          <ToolButton label="Thêm cột trái" onAction={() => insertTableColumn(editor, { before: true })}>
            Cột ←
          </ToolButton>
          <ToolButton label="Thêm cột phải" onAction={() => insertTableColumn(editor)}>
            Cột →
          </ToolButton>
          <Separator />
          <ToolButton label="Xoá dòng" destructive onAction={() => deleteRow(editor)}>
            Xoá dòng
          </ToolButton>
          <ToolButton label="Xoá cột" destructive onAction={() => deleteColumn(editor)}>
            Xoá cột
          </ToolButton>
          <ToolButton label="Xoá bảng" destructive onAction={() => deleteTable(editor)}>
            <Trash2 /> Xoá bảng
          </ToolButton>
        </>
      )}
    </div>
  );
}

function TableElement(props: PlateElementProps) {
  return (
    <PlateElement {...props} as="table">
      <tbody>{props.children}</tbody>
    </PlateElement>
  );
}

function TableRowElement(props: PlateElementProps) {
  return <PlateElement {...props} as="tr" />;
}

function TableCellElement(props: PlateElementProps) {
  return <PlateElement {...props} as="td" />;
}

function TableCellHeaderElement(props: PlateElementProps) {
  return <PlateElement {...props} as="th" />;
}

export function PlatePromptEditor({ initialValue, onChange, onEditor }: PlatePromptEditorProps) {
  const editor = usePlateEditor({
    plugins: promptEditorPlugins,
    components: {
      table: TableElement,
      tr: TableRowElement,
      td: TableCellElement,
      th: TableCellHeaderElement,
    },
    value: (currentEditor) => currentEditor
      .getApi(MarkdownPlugin)
      .markdown.deserialize(initialValue),
  });
  useEffect(() => {
    onEditor?.(editor);
  }, [editor, onEditor]);

  // Serialize trễ: mỗi phím chỉ đánh dấu "có sửa", hết SERIALIZE_DEBOUNCE_MS không gõ nữa (hoặc mất
  // focus/unmount) mới serialize một lần. Bấm Lưu làm editor blur trước click nên form luôn có bản mới nhất.
  const onChangeRef = useRef(onChange);
  useEffect(() => {
    onChangeRef.current = onChange;
  }, [onChange]);
  const pending = useRef(false);
  const timer = useRef<number | undefined>(undefined);
  const flush = useCallback(() => {
    window.clearTimeout(timer.current);
    timer.current = undefined;
    if (!pending.current) return;
    pending.current = false;
    onChangeRef.current(editor.getApi(MarkdownPlugin).markdown.serialize());
  }, [editor]);
  const schedule = useCallback(() => {
    pending.current = true;
    window.clearTimeout(timer.current);
    timer.current = window.setTimeout(flush, SERIALIZE_DEBOUNCE_MS);
  }, [flush]);
  useEffect(() => flush, [flush]);

  // onBlur đặt ở khung ngoài (focusout nổi bọt) thay vì PlateContent: Slate chỉ chuyển tiếp onBlur khi
  // target là contenteditable thật, jsdom không có isContentEditable nên test không bắt được.
  return (
    <div className="relative" onBlur={flush}>
      <Plate editor={editor} onValueChange={schedule}>
        <PromptToolbar />
        <PromptFloatingToolbar />
        <PlateContent
          aria-label="Prompt dịch thuật"
          aria-multiline="true"
          className="plate-prompt-editor reading-font min-h-[420px] w-full rounded-b-md border bg-card px-8 py-7 text-[15px] text-card-foreground shadow-xs outline-none selection:bg-primary/15 focus-visible:ring-2 focus-visible:ring-ring/40"
          placeholder="Viết prompt dịch thuật…"
          spellCheck={false}
        />
      </Plate>
    </div>
  );
}
