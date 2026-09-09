/** So đường dẫn không phân biệt hoa thường và dấu `/` `\` cuối (Windows). */
export function samePath(a: string, b: string): boolean {
  const norm = (p: string) => p.replace(/[\\/]+$/, "").replace(/\//g, "\\").toLowerCase();
  return norm(a) === norm(b);
}
