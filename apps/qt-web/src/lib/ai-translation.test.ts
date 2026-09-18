import { describe, expect, it } from "vitest";

import {
  buildAiTranslationReviewPrompt,
  buildAiTranslationSystemPrompt,
  buildWorkspaceTranslationGlossary,
  checkAiTranslationViolations,
  defaultAiCheckRules,
  filterTranslationGlossaryForSource,
  formatAiTranslation,
  glossaryEntryMatchesSource,
  wordCount,
} from "@/lib/ai-translation";
import { composeBasePrompt } from "@/lib/ai-translation-prompt";
import { defaultStoryGenre, emptyAiStoryConfig } from "@/lib/ai-story";
import { dictionaryUpdateKeys, type LocalDictionaryEntries } from "@/lib/types";

function emptyEntries(): LocalDictionaryEntries {
  return Object.fromEntries(
    dictionaryUpdateKeys.map((key) => [key, {}]),
  ) as LocalDictionaryEntries;
}

describe("AI translation prompt", () => {
  it("stores the ported prompt with real Markdown line breaks", () => {
    const base = composeBasePrompt(defaultStoryGenre());
    expect(base).toContain("\n# Ngữ cảnh tác vụ chuyển ngữ\n");
    expect(base).not.toContain("\\n# Ngữ cảnh tác vụ chuyển ngữ");
  });

  it("tells the model the source is published fiction already vetted by an official platform", () => {
    const base = composeBasePrompt(defaultStoryGenre());
    const head = base.slice(0, base.indexOf("# Suy nghĩ trước khi dịch"));
    expect(head).toContain("đã qua kiểm duyệt nội dung của nền tảng phát hành chính thống");
    expect(head).toContain("không phải lý do để từ chối");
  });

  it("uses only compact workspace entries and accepted names as its glossary", () => {
    const entries = emptyEntries();
    entries.names = { 药老: "Dược Lão" };
    entries.vietPhrase = { 斗气: "đấu khí" };
    entries.hauTu = { 宗: "tông" };

    const glossary = buildWorkspaceTranslationGlossary(entries, { 萧炎: "Tiêu Viêm" });
    expect(glossary).toEqual({
      names: { 药老: "Dược Lão", 萧炎: "Tiêu Viêm" },
      viet_phrase: { 斗气: "đấu khí" },
    });
    const prompt = buildAiTranslationSystemPrompt(glossary);
    expect(prompt).toContain("chuyển ngữ trung thành");
    expect(prompt).toContain("树倒猢狲散");
    expect(prompt).toContain('"萧炎": "Tiêu Viêm"');
    expect(prompt).toContain("Chỉ xuất bản dịch tiếng Việt");
    // Suffix phải ép thinking lập kế hoạch dịch, không chỉ tóm tắt truyện.
    expect(prompt).toContain("kế hoạch dịch");
  });

  it("keeps the translation-plan nudge when a custom prompt replaces the base", () => {
    const story = emptyAiStoryConfig();
    story.customPrompt = "Prompt riêng của truyện.";
    const prompt = buildAiTranslationSystemPrompt({}, story);
    expect(prompt).toContain("Prompt riêng của truyện.");
    expect(prompt).toContain("kế hoạch dịch");
  });

  it("adds story metadata, glossary, style and a custom prompt", () => {
    const story = emptyAiStoryConfig();
    story.name = "Đấu Phá Thương Khung";
    story.protagonist = "Tiêu Viêm";
    story.glossary.names = { 萧炎: "Tiêu Viêm bản truyện" };
    story.style.voice = "Gọn, không tô màu";
    story.customPrompt = "PROMPT RIÊNG CỦA TRUYỆN";

    const prompt = buildAiTranslationSystemPrompt(
      { names: { 萧炎: "Tiêu Viêm workspace" } },
      story,
    );

    expect(prompt).toContain("PROMPT RIÊNG CỦA TRUYỆN");
    expect(prompt).toContain('"name": "Đấu Phá Thương Khung"');
    expect(prompt).toContain('"萧炎": "Tiêu Viêm bản truyện"');
    expect(prompt).toContain("Gọn, không tô màu");
  });
});

describe("AI translation post-processing", () => {
  it("normalizes paragraph spacing and a trailing newline", () => {
    expect(formatAiTranslation("Đoạn một  \n\n\nĐoạn hai\n")).toBe(
      "Đoạn một\n\nĐoạn hai\n",
    );
  });

  it("finds the ported automatic violations and builds a narrow review prompt", () => {
    const text = "But hắn còn nói…\n萧炎 bước vào】";
    const violations = checkAiTranslationViolations(text);
    expect(violations.map((item) => item.message)).toEqual(
      expect.arrayContaining([
        "Từ nối tiếng Anh lọt vào bản dịch → dịch sang tiếng Việt hoặc chỉ giữ khi có căn cứ",
        "Còn ký tự … → chuẩn hóa thành dấu chấm ASCII, giữ số lượng (… → ..., …… → ......)",
        "CJK còn sót (chưa dịch hết!)",
      ]),
    );
    const review = buildAiTranslationReviewPrompt(text, violations);
    expect(review.user).toContain("chỉ thay từ hoặc cụm gây lỗi");
    expect(review.user).toContain("But hắn còn nói");
    expect(review.system).toContain("không đổi văn phong");
  });

  // Bộ lỗi rút ra từ lần đối chiếu Gemini 3.7 với GLM 5.3 trên cùng một chương.
  it("flags the convert-ese and literal-idiom slips seen in provider output", () => {
    const text = [
      "khấp huyết giết địch vô số",
      "chỉ trong một niệm có thể ma diệt",
      "một thứ lãnh diêm sắc bén",
      "Ta chỉ làm một màn thị phạm.",
      "chỉ cần chồng cộng hai đạo thể",
      "y phục nửa xẻ",
      "rút dao chém chết hắn",
      "quả bom khói ném ra",
      "sát ý đông cứng thành thực chất",
      "quả thực có chút phụ lòng tạo hóa",
      "cũng phải nhìn theo bụi mà không kịp",
      "đẹp đến nghẹt thở",
      "Ngón ngọc thon trắng mảnh khảnh",
      "chỉ nghe nói nàng trời sinh Kiếm Tâm",
    ].join("\n");

    const messages = checkAiTranslationViolations(text).map((item) => item.message);
    expect(messages).toEqual(
      expect.arrayContaining([
        "泣血 → nhuộm máu / đẫm máu",
        "抹杀 → xóa sổ / mạt sát",
        "冷艳 → lạnh lùng sắc sảo / lạnh lùng kiêu sa",
        "示范 → làm mẫu",
        "叠加 → chồng lên nhau / kết hợp",
        "衣衫半解 → y phục bán khai / xiêm y cởi dở",
        "刀 là đao → vỏ đao / rút đao / thanh đao",
        "烟雾弹 trong bối cảnh cổ → màn khói / hỏa mù",
        "凝成 → ngưng tụ thành",
        "暴殄天物 → phí phạm của quý",
        "望尘莫及 → không sao theo kịp / tự thẹn không bằng",
        "惊心动魄 → đẹp đến kinh tâm động phách",
        "mảnh khảnh chỉ tả người → ngón tay dùng thon / thon dài",
        "Danh xưng lai nửa Việt nửa Hán → dùng Hán-Việt cả cụm (Thiên Sinh ...)",
      ]),
    );
  });

  // `\b` của JS chỉ tính [A-Za-z0-9_], nên các rule bọc \b quanh từ bắt đầu
  // hoặc kết thúc bằng chữ có dấu trước đây không bao giờ khớp.
  it("still flags terms whose word boundaries are diacritics", () => {
    const text = [
      "Nàng là vợ hắn.",
      "Ừm, cũng được.",
      "Hắn đặc ý tới đây.",
      "Giọng nói bi thê.",
      "Ánh mắt u thê.",
      "Hắn thật vô ngữ.",
      "Địch phương đã rút lui.",
    ].join("\n");

    expect(checkAiTranslationViolations(text).map((item) => item.message)).toEqual(
      expect.arrayContaining([
        "Dùng vợ/chồng → thay bằng thê tử/phu quân",
        "Hừm/Ừm → Ân",
        "đặc ý → cố ý",
        "bi thê → bi thương",
        "u thê → u sầu",
        "vô ngữ → bó tay",
        "địch phương → quân địch",
      ]),
    );
  });

  it("flags ông ta/bà ta for elderly characters in the ancient setting, even capitalised at line start", () => {
    const text = ["Ông ta vuốt râu.", "Bà ta cười lạnh.", "Ông nội hắn đã mất.", "Lão bà bà lắc đầu."].join("\n");
    const hits = checkAiTranslationViolations(text)
      .filter((item) => item.message.startsWith("ông ta/bà ta"))
      .map((item) => item.line);
    expect(hits).toEqual([1, 2]);
    expect(checkAiTranslationViolations("Ông ta cười.", undefined, "modern")).toEqual([]);
  });

  it("leaves clean prose alone", () => {
    const text = [
      "Nàng đậu nơi vương đình, hai mươi tám năm chẳng bay cũng chẳng hót.",
      "Hắn rút đao ra khỏi vỏ đao, sát ý ngưng tụ thành thực chất.",
      "Ngón tay ngọc thon dài, móng sơn màu đỏ.",
      "Chỉ nghe nói nàng có Thiên Sinh Kiếm Tâm.",
      // Từ Việt hợp lệ từng bị rule đổi-từ bắt nhầm (từ nối Anh "so", nhức óc, xao động, thôi thì).
      "Khác biệt một trời một vực! So với bên ngoài thì bên trong quả là tiên cảnh.",
      "Âm thanh tựa núi gầm biển thét vang lên điếc tai nhức óc.",
      "Ngay lúc tâm thần nàng đang xao động, góc phòng vang lên một tiếng thét.",
      "Thôi thì bỏ qua, đợi lát nữa rồi tính.",
    ].join("\n");

    expect(checkAiTranslationViolations(text)).toEqual([]);
  });

  it("bắt động từ Hán-Việt dán trợ từ Việt: đắc được, đắc thủ, thu hoạch được, X có biệt; tha biệt hiệu/biệt xưng", () => {
    const text = [
      "Trương Vô Kỵ đắc được chân truyền của Điệp Cốc Y Tiên.",
      "Nếu không để Phúc Khang An đắc thủ.",
      "Mò mẫm một hồi mà không thu hoạch được gì.",
      "Nam nữ có biệt, đêm tối không tiện gặp mặt. Mãn Hán có biệt.",
      "Cừu Thiên Nhẫn có biệt hiệu Thiết Chưởng, núi Nga Mi có biệt xưng.",
    ].join("\n");
    const lines = checkAiTranslationViolations(text).map((v) => `${v.line}:${v.message}`);
    expect(lines).toEqual([
      "1:得到 / 深得 → được / nhận được / học được, không \"đắc được\"",
      "2:得手 → thành công / ra tay trót lọt, không \"đắc thủ\"",
      "3:有收获 → thu được / tìm được gì, không \"thu hoạch được\"",
      "4:有别 → hữu biệt / khác biệt, không \"có biệt\"",
    ]);
  });

  it("bộ rule ancient giữ nguyên thứ tự cũ; modern bỏ rule cổ trang và thêm rule xưng hô", () => {
    const ancient = defaultAiCheckRules("ancient");
    const modern = defaultAiCheckRules("modern");
    expect(ancient[0]?.message).toBe("Dấu câu tiếng Trung còn sót → dùng dấu câu thường");
    expect(ancient.map((r) => r.message)).toContain("Dùng vợ/chồng → thay bằng thê tử/phu quân");
    expect(modern.map((r) => r.message)).not.toContain("Dùng vợ/chồng → thay bằng thê tử/phu quân");
    expect(modern.map((r) => r.message)).toContain(
      "Xưng hô cổ trang trong truyện hiện đại → hắn/cô trong lời kể, tôi/anh/em trong thoại",
    );
    expect(ancient.map((r) => r.message)).not.toContain(
      "Xưng hô cổ trang trong truyện hiện đại → hắn/cô trong lời kể, tôi/anh/em trong thoại",
    );
  });

  it("bối cảnh cổ bắt đàn ông/đàn bà/phụ nữ, hiện đại và hỗn hợp cho qua", () => {
    const text = [
      "Đối diện nàng truyền đến giọng của một người đàn ông.",
      "Người phụ nữ ấy lặng im, đàn bà trong thôn đều vậy.",
      "Nam nhân khoác hắc bào chậm rãi thêm củi.",
    ].join("\n");
    const message = "Từ chỉ người đời thường trong bối cảnh cổ → nam nhân/nữ nhân (nam tử/nữ tử)";
    expect(checkAiTranslationViolations(text).map((v) => `${v.line}:${v.message}`)).toEqual([`1:${message}`, `2:${message}`]);
    expect(checkAiTranslationViolations(text, undefined, "modern").map((v) => v.message)).not.toContain(message);
    expect(checkAiTranslationViolations(text, undefined, "mixed").map((v) => v.message)).not.toContain(message);
  });

  it("checks theo setting: modern cho vợ/chồng qua, bắt ngươi/nàng/thê tử/tổng tài", () => {
    const text = [
      "Vợ anh đang đợi ở công ty.",
      "Ngươi dám nói vậy sao?",
      "Nàng im lặng.",
      "Thê tử của tổng tài Lâm.",
      "Bố mẹ tôi ở Bắc Kinh, ừm.",
    ].join("\n");
    const modern = checkAiTranslationViolations(text, undefined, "modern");
    expect(modern.map((v) => `${v.line}:${v.message}`)).toEqual([
      "2:Xưng hô cổ trang trong truyện hiện đại → hắn/cô trong lời kể, tôi/anh/em trong thoại",
      "3:Xưng hô cổ trang trong truyện hiện đại → hắn/cô trong lời kể, tôi/anh/em trong thoại",
      "4:Từ gia đình cổ trang → vợ/chồng/bố/mẹ",
      "4:tổng tài → tổng giám đốc",
      "5:Lời kể ngôi một dùng tôi → ta (tôi chỉ trong thoại theo quan hệ)",
    ]);
    const ancient = checkAiTranslationViolations(text);
    expect(ancient.map((v) => v.message)).toContain("Dùng vợ/chồng → thay bằng thê tử/phu quân");
    expect(ancient.map((v) => v.message)).toContain("Hừm/Ừm → Ân");
  });

  it("modern: tôi ở dòng kể (không ngoặc kép) bị bắt, tôi trong thoại thì không", () => {
    const text = [
      "Tôi lắc đầu.",
      "Sáu năm trước, tôi vì Cao Kiện mà đâm người trọng thương.",
      "Hứa Như Vân cố tình cao giọng: \"Tôi cố tình đấy thì sao, em gái tôi muốn đến ở thì anh không cho.\"",
      "\"Ban ngày anh đi làm, chỉ có một mình tôi ở nhà.\"",
      "Ta lắc đầu.",
    ].join("\n");
    expect(checkAiTranslationViolations(text, undefined, "modern").map((v) => v.line)).toEqual([1, 2]);
    expect(checkAiTranslationViolations(text, undefined, "mixed")).toEqual([]);
  });

  it("mixed chỉ chạy rule trung lập: vợ, ngươi, Ừm qua; dấu câu Trung vẫn bắt", () => {
    const messages = defaultAiCheckRules("mixed").map((r) => r.message);
    expect(messages).not.toContain("Dùng vợ/chồng → thay bằng thê tử/phu quân");
    expect(messages).not.toContain("Xưng hô cổ trang trong truyện hiện đại → hắn/cô trong lời kể, tôi/anh/em trong thoại");
    const text = "Vợ anh nói: Ngươi dám? Ừm，được.";
    expect(checkAiTranslationViolations(text, undefined, "mixed").map((v) => v.message)).toEqual([
      "Dấu câu tiếng Trung còn sót → dùng dấu câu thường",
    ]);
  });

  it("uses configured story rules in place of defaults", () => {
    const violations = checkAiTranslationViolations("Vẫn còn văn convert", [
      { pattern: "CONVERT", flags: "i", message: "Rule riêng" },
    ]);
    expect(violations).toEqual([
      { line: 1, message: "Rule riêng", text: "Vẫn còn văn convert" },
    ]);
  });

  it("always flags leftover Han characters even with custom rules", () => {
    const violations = checkAiTranslationViolations("Hắn đọc 㐀 trong bí tịch", [
      { pattern: "CONVERT", flags: "i", message: "Rule riêng" },
    ]);
    expect(violations.map((item) => item.message)).toEqual([
      "CJK còn sót (chưa dịch hết!)",
    ]);
  });

  it("flags fullwidth Chinese punctuation by default", () => {
    const violations = checkAiTranslationViolations("Hắn gật đầu，rồi rời đi。");
    expect(violations.map((item) => item.message)).toContain(
      "Dấu câu tiếng Trung còn sót → dùng dấu câu thường",
    );
  });
});

describe("wordCount", () => {
  it("counts whitespace-separated words", () => {
    expect(wordCount("hắn vô cùng cao hứng")).toBe(5);
  });

  it("ignores blank lines and extra whitespace", () => {
    expect(wordCount("một  hai\n\nba\n")).toBe(3);
  });

  it("returns zero for empty text", () => {
    expect(wordCount("")).toBe(0);
    expect(wordCount("  \n ")).toBe(0);
  });
});

describe("glossaryEntryMatchesSource", () => {
  it("matches exact occurrences", () => {
    expect(glossaryEntryMatchesSource("震雷子", "震雷子看向太清山。")).toBe(true);
    expect(glossaryEntryMatchesSource("萧炎", "震雷子看向太清山。")).toBe(false);
  });

  it("matches a 3-char person name when the surname is dropped", () => {
    expect(glossaryEntryMatchesSource("赵静文", "静文微微一笑。")).toBe(true);
  });

  it("matches a 4-char name when a compound surname is dropped", () => {
    expect(glossaryEntryMatchesSource("慕容雪羽", "雪羽转身离去。")).toBe(true);
  });

  it("does not degrade 2-char names to single characters", () => {
    expect(glossaryEntryMatchesSource("萧炎", "炎热的天气。")).toBe(false);
  });
});

describe("filterTranslationGlossaryForSource", () => {
  it("keeps matching entries, drops the rest and empty groups", () => {
    const filtered = filterTranslationGlossaryForSource(
      {
        names: { "震雷子": "Chấn Lôi Tử", "萧炎": "Tiêu Viêm" },
        nouns: { "灵石": "linh thạch" },
        signature_phrases: { "三十年河东": "ba mươi năm Hà Đông" },
      },
      "震雷子看向太清山。",
    );
    expect(filtered).toEqual({
      names: { "震雷子": "Chấn Lôi Tử" },
      // signature_phrases là văn phong — không lọc theo raw.
      signature_phrases: { "三十年河东": "ba mươi năm Hà Đông" },
    });
  });
});

describe("glossary filtering in the system prompt", () => {
  it("only ships entries present in the chapter when sourceText is given", () => {
    const story = emptyAiStoryConfig();
    story.glossary.names["赵静文"] = "Triệu Tĩnh Văn";
    story.glossary.places["太清山"] = "Thái Thanh Sơn";
    story.glossary.places["塞下学宫"] = "Tắc Hạ Học Cung";
    const prompt = buildAiTranslationSystemPrompt(
      { names: { "震雷子": "Chấn Lôi Tử", "萧炎": "Tiêu Viêm" } },
      story,
      "震雷子与静文一同看向太清山。",
    );
    expect(prompt).toContain("震雷子");
    expect(prompt).toContain("赵静文"); // khớp dạng bỏ họ
    expect(prompt).toContain("太清山");
    expect(prompt).not.toContain("萧炎");
    expect(prompt).not.toContain("塞下学宫");
  });

  it("keeps the full glossary when sourceText is omitted", () => {
    const prompt = buildAiTranslationSystemPrompt({
      names: { "萧炎": "Tiêu Viêm" },
    });
    expect(prompt).toContain("萧炎");
  });

  it("addressing đi vào prompt khi chương có một bên của cặp, kèm ghi chú cách đọc", () => {
    const glossary = {
      names: { 林枫: "Lâm Phong", 苏雨: "Tô Vũ" },
      addressing: { "林枫→苏雨": "anh–em", "王五→赵六": "tôi–cậu" },
    };
    const kept = filterTranslationGlossaryForSource(glossary, "林枫走了进来。");
    expect(kept.addressing).toEqual({ "林枫→苏雨": "anh–em" });
    expect(glossaryEntryMatchesSource("林枫→苏雨", "苏雨来了")).toBe(true);
    const story = { ...emptyAiStoryConfig(), glossary: { ...emptyAiStoryConfig().glossary, addressing: { "林枫→苏雨": "anh–em" } } };
    const prompt = buildAiTranslationSystemPrompt({}, story, "林枫走了进来。");
    expect(prompt).toContain('"林枫→苏雨": "anh–em"');
    expect(prompt).toContain("Nhóm `addressing`: `甲→乙: X–Y`");
    expect(buildAiTranslationSystemPrompt({}, story, "无人。")).not.toContain("Nhóm `addressing`");
  });
});
