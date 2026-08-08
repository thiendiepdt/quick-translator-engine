import {
  flip,
  offset,
  useFloatingToolbar,
  useFloatingToolbarState,
} from "@platejs/floating";
import { MarkdownPlugin } from "@platejs/markdown";
import { Bold, Code2, Italic, Strikethrough } from "lucide-react";
import {
  Plate,
  PlateContent,
  PlateElement,
  useEditorId,
  useEditorRef,
  useEditorSelector,
  useEventEditorValue,
  usePlateEditor,
  type PlateElementProps,
} from "platejs/react";
import type { ReactNode } from "react";

import { promptEditorPlugins } from "@/components/plate-prompt-editor-plugins";
import { Button } from "@/components/ui/button";

interface PlatePromptEditorProps {
  initialValue: string;
  onChange: (markdown: string) => void;
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
        <MarkButton mark="strikethrough" label="Gạch ngang" icon={<Strikethrough />} />
        <MarkButton mark="code" label="Mã nội dòng" icon={<Code2 />} />
      </div>
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

export function PlatePromptEditor({ initialValue, onChange }: PlatePromptEditorProps) {
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

  return (
    <div className="relative">
      <Plate
        editor={editor}
        onValueChange={() => {
          onChange(editor.getApi(MarkdownPlugin).markdown.serialize());
        }}
      >
        <PromptFloatingToolbar />
        <PlateContent
          aria-label="Prompt dịch thuật"
          aria-multiline="true"
          className="plate-prompt-editor min-h-[480px] w-full rounded-sm border border-border/60 bg-reader-paper px-10 py-9 font-serif text-[15px] text-reader-ink shadow-sm outline-none selection:bg-primary/15 focus-visible:ring-2 focus-visible:ring-ring/40 md:px-14"
          placeholder="Viết prompt dịch thuật…"
          spellCheck={false}
        />
      </Plate>
    </div>
  );
}
