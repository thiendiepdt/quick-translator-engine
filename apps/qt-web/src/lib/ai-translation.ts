import { composeBasePrompt } from "@/lib/ai-translation-prompt";
import { defaultStoryGenre, type AiCheckRule, type AiStoryConfig, type GenreSetting } from "@/lib/ai-story";
import type { LocalDictionaryEntries } from "@/lib/types";

export interface TranslationViolation {
  line: number;
  message: string;
  text: string;
}

export type TranslationGlossary = Record<string, Record<string, string>>;

const TRANSLATION_PROMPT_SUFFIX = `
---
Dịch raw text tiếng Trung phía dưới sang tiếng Việt.

Trong phần suy nghĩ nội bộ, phải lập kế hoạch dịch chứ không chỉ tóm tắt cốt truyện — phân tích truyện chỉ để phục vụ các quyết định dịch:
- Liệt kê các cụm khó hoặc dễ phạm quy tắc (thành ngữ, cấu trúc dễ dịch máy móc theo trật tự tiếng Trung, từ bị cấm, thán từ) kèm phương án tiếng Việt định dùng.
- Chốt xưng hô cho từng cặp nhân vật đối thoại và tên riêng theo glossary trước khi dịch câu đầu tiên.
- Đánh dấu những câu nhiều rủi ro nhất (đổi chủ thể, phủ định, chơi chữ) để dịch chậm lại ở đó.

Trước khi xuất, kiểm tra thầm từng câu:
- Có chữ hoặc ý nào không chỉ được vị trí tương ứng trong raw không? Có thì xóa.
- Có làm câu dài hơn, giàu cảm xúc hơn, rõ hơn hoặc bóng bẩy hơn nguyên tác không? Có thì trả về đúng mức của nguyên tác.
- Nhịp, độ gọn, sự lặp và mơ hồ của tác giả còn nguyên không?
- Khi che raw đi, từng câu có phải cách người Việt thực sự diễn đạt không? Nếu còn khung câu Trung, dựng lại cả câu mà không thêm ý.
- Chủ thể, phủ định, so sánh và mức độ có bị đổi không?
- Mọi câu, mọi đoạn đã có mặt đủ và không câu nào bị đứt giữa chừng chưa?

Chỉ xuất bản dịch tiếng Việt, không giải thích, không comment, không markdown.`;

/**
 * Rule mặc định. Phần tử thứ ba là bối cảnh áp dụng: không có = trung lập (mọi truyện),
 * "ancient" = chỉ cổ đại/tiên hiệp, "modern" = chỉ hiện đại.
 */
const CHECK_RULES: Array<[RegExp, string, GenreSetting?]> = [
  [/[，。、；：？！]/, "Dấu câu tiếng Trung còn sót → dùng dấu câu thường"],
  [/(^|[.!?]\s+|[【(])["“']?(?:but|and|so|the|in|on|at|from|with)\b/i, "Từ nối tiếng Anh lọt vào bản dịch → dịch sang tiếng Việt hoặc chỉ giữ khi có căn cứ"],
  [/(?<!\p{L})(?:vợ|chồng)(?!\p{L})/iu, "Dùng vợ/chồng → thay bằng thê tử/phu quân", "ancient"],
  [/\banh ấy\b|\banh ta\b|\bcô ấy\b|\bchị ấy\b/i, "Đại từ sai → dùng hắn/nàng", "ancient"],
  [/(^|[“"']|,\s+)(?:mình|tôi)(?:\s|[,.!?…])/i, "Dùng mình/tôi làm đại từ → thay bằng ta trong style mặc định", "ancient"],
  [/Miêu Ảnh Vô Tông/, "Sai âm tên riêng → dùng Miêu Ảnh Vô Tung", "ancient"],
  [/một tấc vuông/, "方寸 là không gian hệ thống → dùng Phương Thốn", "ancient"],
  [/tinh thần đại chấn/, "精神大振 → dùng tinh thần phấn chấn hẳn lên"],
  [/mơ hồ nghiệm ra|mùi vị không bình thường/, "品出意味 → dùng nhận ra/nhận thấy điều bất thường"],
  [/bình loạn bắt sống/, "平叛生擒 → dùng dẹp loạn, bắt sống"],
  [/không có rèm che chuyên biệt/, "四面无帷 → dùng không rèm che bốn phía"],
  [/nóng bóng/, "Lỗi chính tả → nóng bỏng"],
  [/ta vất vả một chút/, "我辛苦点 → dùng ta chịu khó một chút"],
  [/đẳng tước vị quân công/, "二十等军功爵 → dùng hai mươi bậc tước quân công"],
  [/vệt đỏ đắc ý/, "一抹得意的红晕 → dùng vệt ửng đỏ vì đắc ý"],
  [/toàn bộ người nghênh đón có mặt đều/, "Tránh chồng chủ thể/lượng từ"],
  [/đã thưởng Minh chủ/, "感谢 X 打赏的盟主 → dùng cảm ơn minh chủ X đã thưởng/ủng hộ"],
  [/não hải/, "não hải → đầu óc / tâm trí"],
  [/(?<!\p{L})(?:Hừm|Ừm)(?!\p{L})/iu, "Hừm/Ừm → Ân", "ancient"],
  [/Ơ\s*[?!,.…]/, "Thán từ Ơ → dùng A trong bối cảnh cổ đại/huyền huyễn", "ancient"],
  [/\bthập phần\b/, "thập phần → vô cùng / hết sức"],
  [/\bsong doanh\b/, "song doanh → đôi bên cùng có lợi"],
  [/còn đừng nói/i, "还别说 → Mà phải nói / Không ngờ thật"],
  [/phụ thân (ở|vào|lên|trong)/, "附身 → nương thân/bám vào"],
  [/nhận dạng/, "nhận dạng → kiểm trắc"],
  [/kho tàng|kho báu/, "kho tàng/báu → bảo khố", "ancient"],
  [/xao động/, "xao động → rung động"],
  [/phát xạ/, "phát xạ → phóng ra"],
  [/thích dụng/, "thích dụng → áp dụng"],
  [/thúc động/, "thúc động → thôi động"],
  [/tiền xa/, "tiền xa → vết xe đổ"],
  [/lãnh tình/, "lãnh tình → cảm kích"],
  [/đợi lát nữa/, "đợi lát nữa → chờ một hồi"],
  [/đại động can qua/, "đại động can qua → làm to chuyện"],
  [/nước thu\b/, "nước thu → thu thủy"],
  [/là tính là/, "là tính là → xem như"],
  [/\bthê tử danh nghĩa\b/, "Sai vị trí → trên danh nghĩa thê tử", "ancient"],
  [/(?<!\p{L})đặc ý(?!\p{L})/iu, "đặc ý → cố ý"],
  [/\bvô ý trung\b/, "vô ý trung → trong lúc vô tình"],
  [/(?<!\p{L})bi thê(?!\p{L})/iu, "bi thê → bi thương"],
  [/(?<!\p{L})u thê(?!\p{L})/iu, "u thê → u sầu"],
  [/\bnhức óc\b/, "nhức óc → đau đầu"],
  [/(?<!\p{L})thôi thì(?!\p{L})/iu, "thôi thì → vậy thì / đã vậy"],
  [/(?<!\p{L})vô ngữ(?!\p{L})/iu, "vô ngữ → bó tay"],
  [/(?<!\p{L})địch phương(?!\p{L})/iu, "địch phương → quân địch"],
  [/\bhữu phương\b|\bhữu quân\b/, "hữu phương/quân → phe bạn"],
  [/quả thực đúng là/, "quả thực đúng là → chọn quả thực hoặc đúng là"],
  [/cư nhiên/, "cư nhiên → lại / dám / không ngờ"],
  [/\bkhấp huyết\b/i, "泣血 → nhuộm máu / đẫm máu"],
  [/\bma diệt\b/i, "抹杀 → xóa sổ / mạt sát"],
  [/lãnh di[êễ]m/i, "冷艳 → lạnh lùng sắc sảo / lạnh lùng kiêu sa"],
  [/\bthị phạm\b/i, "示范 → làm mẫu"],
  [/\bchồng cộng\b/i, "叠加 → chồng lên nhau / kết hợp"],
  // `\b` cuối từ không dùng được ở đây: JS chỉ coi [A-Za-z0-9_] là ký tự từ nên
  // chữ kết thúc bằng nguyên âm có dấu (xẻ, thê) không bao giờ tạo ranh giới.
  [/nửa xẻ/i, "衣衫半解 → y phục bán khai / xiêm y cởi dở"],
  [/vỏ dao|rút dao|thanh dao\b/i, "刀 là đao → vỏ đao / rút đao / thanh đao", "ancient"],
  [/bom khói/i, "烟雾弹 trong bối cảnh cổ → màn khói / hỏa mù", "ancient"],
  [/đông cứng thành/i, "凝成 → ngưng tụ thành"],
  [/phụ lòng tạo hóa/i, "暴殄天物 → phí phạm của quý"],
  [/nhìn theo bụi/i, "望尘莫及 → không sao theo kịp / tự thẹn không bằng"],
  [/đẹp đến nghẹt thở/i, "惊心动魄 → đẹp đến kinh tâm động phách"],
  [/ngón.{0,20}mảnh khảnh/i, "mảnh khảnh chỉ tả người → ngón tay dùng thon / thon dài"],
  [/trời sinh [A-Z]/, "Danh xưng lai nửa Việt nửa Hán → dùng Hán-Việt cả cụm (Thiên Sinh ...)"],
  [/…/, "Còn ký tự … → chuẩn hóa thành dấu chấm ASCII, giữ số lượng (… → ..., …… → ......)"],
  // Rule riêng cho truyện hiện đại: bắt giọng cổ trang lọt vào đô thị.
  [/(?<!\p{L})(?:ngươi|nàng|bọn ta|các ngươi)(?!\p{L})/iu, "Xưng hô cổ trang trong truyện hiện đại → anh/cô/tôi theo quan hệ", "modern"],
  [/(?<!\p{L})(?:thê tử|phu quân|lang quân|phụ thân|mẫu thân)(?!\p{L})/iu, "Từ gia đình cổ trang → vợ/chồng/bố/mẹ", "modern"],
  [/(?<!\p{L})tổng tài(?!\p{L})/iu, "tổng tài → tổng giám đốc", "modern"],
  [/nói đạo/i, "说道 → nói / đáp, không \"nói đạo\"", "modern"],
];

function rulesForSetting(setting: GenreSetting): Array<[RegExp, string]> {
  return CHECK_RULES.filter(([, , tag]) => tag === undefined || tag === setting).map(
    ([pattern, message]) => [pattern, message],
  );
}

/** Rule mặc định cho một bối cảnh: rule trung lập + rule gắn đúng setting, giữ thứ tự khai báo. */
export function defaultAiCheckRules(setting: GenreSetting): AiCheckRule[] {
  return rulesForSetting(setting).map(([pattern, message]) => ({
    pattern: pattern.source,
    ...(pattern.flags ? { flags: pattern.flags } : {}),
    message,
  }));
}

/**
 * Rule cứng chạy trong mọi trường hợp, kể cả khi truyện thay bộ rules riêng —
 * sót Hán tự là lỗi tuyệt đối. `\p{Script=Han}` phủ cả các khối mở rộng
 * (Ext-A/B+…) và chữ hiếm trong tên công pháp mà khoảng U+4E00–U+9FFF bỏ lọt.
 */
const MANDATORY_CHECK_RULES: Array<[RegExp, string]> = [
  [/\p{Script=Han}/u, "CJK còn sót (chưa dịch hết!)"],
];

function nonEmptyRecord(value: Record<string, string>): Record<string, string> | undefined {
  const entries = Object.entries(value).filter(
    ([source, target]) => source.trim() && target.trim(),
  );
  return entries.length > 0 ? Object.fromEntries(entries) : undefined;
}

/** Chỉ gửi phần từ điển cục bộ của workspace; không nhét toàn bộ QT data vào prompt. */
export function buildWorkspaceTranslationGlossary(
  entries: LocalDictionaryEntries,
  knownNames: Record<string, string>,
): TranslationGlossary {
  const groups: Array<[string, Record<string, string>]> = [
    ["names", { ...entries.names, ...knownNames }],
    ["names2", entries.names2],
    ["viet_phrase", entries.vietPhrase],
    ["nouns", entries.danhTu],
  ];
  return Object.fromEntries(
    groups.flatMap(([key, value]) => {
      const normalized = nonEmptyRecord(value);
      return normalized ? [[key, normalized]] : [];
    }),
  );
}

export function countTranslationGlossaryEntries(glossary: TranslationGlossary): number {
  return Object.values(glossary).reduce(
    (total, entries) => total + Object.keys(entries).length,
    0,
  );
}

function mergeStoryGlossary(
  workspaceGlossary: TranslationGlossary,
  story?: AiStoryConfig,
): TranslationGlossary {
  if (!story) return workspaceGlossary;
  const merged: TranslationGlossary = Object.fromEntries(
    Object.entries(workspaceGlossary).map(([key, value]) => [key, { ...value }]),
  );
  for (const [key, entries] of Object.entries(story.glossary)) {
    const normalized = nonEmptyRecord(entries);
    if (!normalized) continue;
    merged[key] = { ...(merged[key] ?? {}), ...normalized };
  }
  return merged;
}

/**
 * Entry glossary được coi là "có mặt trong chương" khi source xuất hiện nguyên
 * văn, hoặc — với tên người 3-4 chữ Hán — khi dạng bỏ họ xuất hiện (văn Trung
 * hay gọi tắt: 赵静文 → 静文, 慕容雪羽 → 雪羽). Không hạ tên 2 chữ xuống 1 chữ.
 */
const HAN_PERSON_NAME = /^\p{Script=Han}{3,4}$/u;

export function glossaryEntryMatchesSource(source: string, text: string): boolean {
  if (text.includes(source)) return true;
  if (!HAN_PERSON_NAME.test(source)) return false;
  if (text.includes(source.slice(1))) return true;
  return source.length === 4 && text.includes(source.slice(2));
}

/**
 * Chỉ giữ entry mà chương thực sự chạm tới — glossary tích lũy cả truyện thì
 * lớn, một chương chỉ dùng phần nhỏ. `signature_phrases` là văn phong, giữ
 * nguyên không lọc. Nhóm rỗng sau lọc bị bỏ hẳn khỏi prompt.
 */
export function filterTranslationGlossaryForSource(
  glossary: TranslationGlossary,
  sourceText: string,
): TranslationGlossary {
  return Object.fromEntries(
    Object.entries(glossary).flatMap(([group, entries]) => {
      if (group === "signature_phrases") return [[group, entries]];
      const kept = Object.fromEntries(
        Object.entries(entries).filter(([source]) =>
          glossaryEntryMatchesSource(source, sourceText),
        ),
      );
      return Object.keys(kept).length > 0 ? [[group, kept]] : [];
    }),
  );
}

export function buildAiTranslationSystemPrompt(
  workspaceGlossary: TranslationGlossary,
  story?: AiStoryConfig,
  sourceText?: string,
): string {
  const merged = mergeStoryGlossary(workspaceGlossary, story);
  const glossary = sourceText
    ? filterTranslationGlossaryForSource(merged, sourceText)
    : merged;
  const glossarySection =
    Object.keys(glossary).length > 0
      ? `\n# Từ điển riêng của truyện\n\nCác mục này được ưu tiên và phải dùng nhất quán:\n\n${JSON.stringify(glossary, null, 2)}\n`
      : "";
  const storyContext = story && (story.name || story.protagonist || story.summary)
    ? `\n# Thông tin truyện\n\n${JSON.stringify({
        name: story.name || undefined,
        protagonist: story.protagonist || undefined,
        summary: story.summary || undefined,
      }, null, 2)}\n`
    : "";
  const hasStyle = story && (
    story.style.voice ||
    story.style.toneRules.length > 0 ||
    Object.keys(story.style.signaturePhrases).length > 0 ||
    story.style.avoid.length > 0
  );
  const styleSection = hasStyle
    ? `\n# Style đặc thù của truyện\n\nStyle chỉ điều chỉnh từ vựng, xưng hô và register trong giới hạn trung thành; không được thêm hoặc bớt nội dung.\n\n${JSON.stringify({
        voice: story.style.voice,
        tone_rules: story.style.toneRules,
        signature_phrases: story.style.signaturePhrases,
        avoid: story.style.avoid,
      }, null, 2)}\n`
    : "";
  const basePrompt = story?.customPrompt.trim() || composeBasePrompt(story?.genre ?? defaultStoryGenre());
  return `${basePrompt}${storyContext}${glossarySection}${styleSection}${TRANSLATION_PROMPT_SUFFIX}`;
}

export function formatAiTranslation(text: string): string {
  const blocks: string[][] = [];
  let current: string[] = [];
  for (const line of text.split(/\r?\n/)) {
    const trimmed = line.trimEnd();
    if (!trimmed) {
      if (current.length > 0) blocks.push(current);
      current = [];
    } else {
      current.push(trimmed);
    }
  }
  if (current.length > 0) blocks.push(current);
  const output = blocks.map((block) => block.join("\n")).join("\n\n");
  return output ? `${output}\n` : "";
}

export function nonEmptyLineCount(text: string): number {
  return text.split(/\r?\n/).filter((line) => line.trim()).length;
}

/** Đếm chữ (âm tiết tách bằng khoảng trắng) — số liệu tự nhiên cho text Việt. */
export function wordCount(text: string): number {
  return text.split(/\s+/).filter(Boolean).length;
}

export function checkAiTranslationViolations(
  text: string,
  configuredRules?: AiCheckRule[],
  setting: GenreSetting = "ancient",
): TranslationViolation[] {
  const violations: TranslationViolation[] = [];
  const baseRules: Array<[RegExp, string]> =
    configuredRules && configuredRules.length > 0
      ? configuredRules.flatMap(({ pattern, flags, message }) => {
          try {
            return [[new RegExp(pattern, flags), message] as [RegExp, string]];
          } catch {
            return [];
          }
        })
      : rulesForSetting(setting);
  const configuredPatterns = new Set(
    (configuredRules ?? []).map(({ pattern }) => pattern),
  );
  const rules: Array<[RegExp, string]> = [
    ...baseRules,
    ...MANDATORY_CHECK_RULES.filter(
      ([pattern]) => !configuredPatterns.has(pattern.source),
    ),
  ];
  text.split(/\r?\n/).forEach((line, index) => {
    for (const [pattern, message] of rules) {
      pattern.lastIndex = 0;
      if (!pattern.test(line)) continue;
      violations.push({ line: index + 1, message, text: line.trim().slice(0, 120) });
    }
  });
  return violations;
}

export function buildAiTranslationReviewPrompt(
  text: string,
  violations: TranslationViolation[],
): { system: string; user: string } {
  const list = violations
    .map((item) => `- Dòng ${item.line}: ${item.message} — "${item.text}"`)
    .join("\n");
  return {
    system:
      "Đây là tác vụ soát tối thiểu một bản dịch tiểu thuyết hư cấu do người dùng cung cấp. Chỉ sửa đúng các vi phạm được liệt kê; tuyệt đối không đổi văn phong, thêm nội dung hoặc chỉnh phần khác. Chỉ xuất toàn bộ text đã soát, không giải thích và không markdown.",
    user: `Bản dịch dưới đây có các vi phạm được phát hiện tự động:\n\n${list}\n\nKiểm tra từng vi phạm. Nếu thực sự sai, chỉ thay từ hoặc cụm gây lỗi bằng phương án ngắn nhất. Nếu đúng trong ngữ cảnh, giữ nguyên. Giữ nguyên toàn bộ chữ, thứ tự câu, dấu câu và ngắt đoạn không liên quan. Không chau chuốt, viết lại, thêm từ nối hoặc thêm miêu tả.\n\n---\n\n${text}`,
  };
}
