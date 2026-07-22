import { useMemo } from "react";

import { buildTextSegments } from "@/lib/ranges";
import type { TextRange } from "@/lib/types";
import { cn } from "@/lib/utils";

interface MappedTextProps {
  text: string;
  ranges: TextRange[];
  activeRange?: number;
  className?: string;
  emptyMessage: string;
  onRangeSelect: (rangeIndex: number) => void;
}

export function MappedText({
  text,
  ranges,
  activeRange,
  className,
  emptyMessage,
  onRangeSelect,
}: MappedTextProps) {
  const segments = useMemo(() => buildTextSegments(text, ranges), [ranges, text]);

  if (!text) {
    return (
      <div className={cn("grid h-full place-items-center px-8 text-center", className)}>
        <p className="max-w-sm text-sm leading-6 text-muted-foreground">{emptyMessage}</p>
      </div>
    );
  }

  return (
    <div className={cn("whitespace-pre-wrap text-pretty", className)}>
      {segments.map((segment) =>
        segment.kind === "plain" ? (
          <span key={segment.key}>{segment.text}</span>
        ) : (
          <span
            key={segment.key}
            role="button"
            tabIndex={0}
            className="mapped-segment"
            data-active={segment.rangeIndex === activeRange}
            data-range-index={segment.rangeIndex}
            aria-label={`Range ${segment.rangeIndex + 1}: ${segment.text}`}
            aria-pressed={segment.rangeIndex === activeRange}
            onClick={() => onRangeSelect(segment.rangeIndex)}
            onKeyDown={(event) => {
              if (event.key === "Enter" || event.key === " ") {
                event.preventDefault();
                onRangeSelect(segment.rangeIndex);
              }
            }}
          >
            {segment.text}
          </span>
        ),
      )}
    </div>
  );
}
