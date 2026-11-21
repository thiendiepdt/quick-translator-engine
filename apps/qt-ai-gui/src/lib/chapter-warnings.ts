/** Một đoạn dính luật: số đoạn (trùng nhãn [[n]] và số dòng tab Bản dịch) + câu trích của đoạn đó nếu có. */
export type WarningHit = { n: number; excerpt?: string };

/** Một luật cùng mọi đoạn dính luật đó. `hits` rỗng: cảnh báo không gắn đoạn nào, hiện nguyên câu. */
export type WarningGroup = { rule: string; hits: WarningHit[] };

const LABEL = /^\[\[(\d+)\]\]\s*/;
/** Harness ghi cảnh báo `[[n]] <luật> — "<trích>"`; luật có thể chứa `—` nên tách ở `— "` đầu tiên. */
const EXCERPT = ' — "';

/**
 * Gom cảnh báo còn lại của chương theo luật, thứ tự nhóm theo lần đầu gặp. Chương nhiều cảnh báo thường chỉ là một
 * luật lặp ở từng đoạn (thư viện: chương 217 cảnh báo còn 1 luật), gom lại thì khung cảnh báo còn vài dòng.
 */
export function groupWarnings(warnings: readonly string[]): WarningGroup[] {
  const groups = new Map<string, WarningGroup>();
  for (const warning of warnings) {
    const label = LABEL.exec(warning);
    const body = label ? warning.slice(label[0].length) : warning;
    const cut = label ? body.indexOf(EXCERPT) : -1;
    const rule = cut >= 0 ? body.slice(0, cut) : body;
    const group = groups.get(rule) ?? { rule, hits: [] };
    groups.set(rule, group);
    if (!label) continue;
    const excerpt = cut >= 0 ? body.slice(cut + EXCERPT.length).replace(/"$/, "") : undefined;
    group.hits.push(excerpt === undefined ? { n: Number(label[1]) } : { n: Number(label[1]), excerpt });
  }
  return [...groups.values()];
}
