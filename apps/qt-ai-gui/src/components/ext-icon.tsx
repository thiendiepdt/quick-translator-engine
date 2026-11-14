import { cn } from "@/lib/utils";

/** Icon của extension; extension không có icon.png thì hiện chữ cái đầu của tên. */
export function ExtIcon({ icon, name, className }: { icon?: string | null; name: string; className?: string }) {
  if (icon) return <img src={icon} alt="" className={cn("size-8 shrink-0 rounded", className)} />;
  const letter = Array.from(name.trim())[0]?.toUpperCase() ?? "?";
  return (
    <div aria-hidden className={cn("flex size-8 shrink-0 items-center justify-center rounded bg-primary/10 text-sm font-semibold text-primary", className)}>
      {letter}
    </div>
  );
}
