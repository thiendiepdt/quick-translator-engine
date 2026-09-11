/** Đọc token từ index.css để test coverage + tương phản. Block bắt đầu bằng comment `/* palette: <id> <mode> *\/`. */

export const REQUIRED_TOKENS = [
  "--background",
  "--foreground",
  "--card",
  "--card-foreground",
  "--popover",
  "--popover-foreground",
  "--primary",
  "--primary-foreground",
  "--secondary",
  "--secondary-foreground",
  "--muted",
  "--muted-foreground",
  "--accent",
  "--accent-foreground",
  "--destructive",
  "--border",
  "--input",
  "--ring",
  "--status-done",
  "--status-error",
  "--status-warning",
  "--status-queued",
  "--status-translating",
  "--log-bg",
  "--log-fg",
  "--font-reading",
  "--radius",
];

/** Token không đổi giữa sáng/tối, chỉ khai báo ở block sáng của mỗi bộ. */
export const LIGHT_ONLY_TOKENS = ["--font-reading", "--radius"];

const MARKER = /\/\*\s*palette:\s*([a-z]+)\s+(light|dark)\s*\*\//g;

export function parsePaletteBlocks(css: string): Map<string, Record<string, string>> {
  const result = new Map<string, Record<string, string>>();
  const markers = [...css.matchAll(MARKER)];
  markers.forEach((match, index) => {
    const start = match.index + match[0].length;
    const next = markers[index + 1];
    const end = next ? next.index : css.length;
    const body = css.slice(start, end);
    const tokens: Record<string, string> = {};
    for (const line of body.matchAll(/(--[a-z0-9-]+)\s*:\s*([^;]+);/g)) {
      const [, name, value] = line;
      if (name && value) tokens[name] = value.trim();
    }
    result.set(`${match[1]}/${match[2]}`, tokens);
  });
  // Block tối kế thừa token chỉ-sáng của cùng bộ.
  for (const [key, tokens] of result) {
    if (!key.endsWith("/dark")) continue;
    const light = result.get(key.replace("/dark", "/light"));
    if (!light) continue;
    for (const token of LIGHT_ONLY_TOKENS) {
      const inherited = light[token];
      if (!(token in tokens) && inherited) tokens[token] = inherited;
    }
  }
  return result;
}

export interface Oklch {
  l: number;
  c: number;
  h: number;
}

export function parseOklch(value: string): Oklch | null {
  const match = /oklch\(\s*([\d.]+)\s+([\d.]+)\s+([\d.]+)/.exec(value);
  if (!match) return null;
  return { l: Number(match[1]), c: Number(match[2]), h: Number(match[3]) };
}

function clamp01(x: number) {
  return Math.min(1, Math.max(0, x));
}

/** OKLCH → linear sRGB (Björn Ottosson) → luminance tương đối WCAG. */
export function relativeLuminance({ l, c, h }: Oklch): number {
  const rad = (h * Math.PI) / 180;
  const a = c * Math.cos(rad);
  const b = c * Math.sin(rad);
  const l_ = l + 0.3963377774 * a + 0.2158037573 * b;
  const m_ = l - 0.1055613458 * a - 0.0638541728 * b;
  const s_ = l - 0.0894841775 * a - 1.291485548 * b;
  const L = l_ ** 3;
  const M = m_ ** 3;
  const S = s_ ** 3;
  const r = clamp01(4.0767416621 * L - 3.3077115913 * M + 0.2309699292 * S);
  const g = clamp01(-1.2684380046 * L + 2.6097574011 * M - 0.3413193965 * S);
  const bl = clamp01(-0.0041960863 * L - 0.7034186147 * M + 1.707614701 * S);
  return 0.2126 * r + 0.7152 * g + 0.0722 * bl;
}

export function contrastRatio(a: Oklch, b: Oklch): number {
  const la = relativeLuminance(a);
  const lb = relativeLuminance(b);
  const [hi, lo] = la > lb ? [la, lb] : [lb, la];
  return (hi + 0.05) / (lo + 0.05);
}
