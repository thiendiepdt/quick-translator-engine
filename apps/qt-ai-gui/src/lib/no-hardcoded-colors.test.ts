import { describe, expect, it } from "vitest";

// Đọc nguồn component qua Vite glob (?raw) để không cần type Node; bỏ ui kit và file test.
const sources: Record<string, string> = import.meta.glob(
  ["../components/**/*.tsx", "!../components/ui/**", "!**/*.test.tsx"],
  { query: "?raw", import: "default", eager: true },
);

const FORBIDDEN =
  /\b(?:bg|text|border|ring|from|to|via)-(?:zinc|slate|gray|neutral|stone|amber|red|green|blue|violet|purple|teal|emerald|orange|yellow|indigo|pink|rose|sky|cyan|lime)-\d{2,3}\b/g;

describe("không dùng màu cứng ngoài ui kit", () => {
  const files = Object.keys(sources);
  it("có quét được component", () => {
    expect(files.length).toBeGreaterThan(5);
  });
  it.each(files)("%s", (file) => {
    const hits = sources[file]?.match(FORBIDDEN) ?? [];
    expect(hits).toEqual([]);
  });
});
