/** Khoá đường dẫn: không phân biệt hoa thường và dấu `/` `\` cuối (Windows) — khớp `root_key` bên Rust. */
export function pathKey(p: string): string {
  return p.replace(/[\\/]+$/, "").replace(/\//g, "\\").toLowerCase();
}

export function samePath(a: string, b: string): boolean {
  return pathKey(a) === pathKey(b);
}
