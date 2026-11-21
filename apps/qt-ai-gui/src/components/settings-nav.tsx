import { useEffect, useState, type RefObject } from "react";

import { pickActiveSection, type SettingsSection } from "@/lib/settings-nav";
import { cn } from "@/lib/utils";

/** Cột mục lục trái của trang Cài đặt: bấm cuộn tới khối, tự sáng mục của khối đang xem khi cuộn. */
export function SettingsNav({
  sections,
  scrollRef,
}: {
  sections: readonly SettingsSection[];
  scrollRef: RefObject<HTMLDivElement | null>;
}) {
  const [active, setActive] = useState(0);

  useEffect(() => {
    const frame = scrollRef.current;
    if (!frame) return;
    const update = () => {
      const top = frame.getBoundingClientRect().top;
      const offsets = sections.map((section) => {
        const element = document.getElementById(section.id);
        return element ? element.getBoundingClientRect().top - top : Number.POSITIVE_INFINITY;
      });
      const atBottom = frame.scrollHeight > frame.clientHeight && frame.scrollTop + frame.clientHeight >= frame.scrollHeight - 2;
      setActive(pickActiveSection(offsets, atBottom));
    };
    frame.addEventListener("scroll", update, { passive: true });
    return () => frame.removeEventListener("scroll", update);
  }, [scrollRef, sections]);

  return (
    <nav aria-label="Mục cài đặt" className="sticky top-8 self-start justify-self-end">
      <ul className="flex flex-col gap-0.5 border-l">
        {sections.map((section, index) => (
          <li key={section.id}>
            <button
              type="button"
              aria-current={index === active ? "true" : undefined}
              onClick={() => {
                setActive(index);
                document.getElementById(section.id)?.scrollIntoView({ behavior: "smooth", block: "start" });
              }}
              className={cn(
                // Không đổi độ đậm chữ khi sáng: bề rộng mục lục quyết định lề trái, chữ đậm lên sẽ làm cột nội dung xê dịch.
                "-ml-px w-full border-l-2 py-1.5 pr-2 pl-3 text-left text-sm whitespace-nowrap transition-colors",
                index === active
                  ? "border-primary text-foreground"
                  : "border-transparent text-muted-foreground hover:border-border hover:text-foreground",
              )}
            >
              {section.label}
            </button>
          </li>
        ))}
      </ul>
    </nav>
  );
}
