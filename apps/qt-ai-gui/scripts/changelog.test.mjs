import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

import { draftFromCommits, FALLBACK_ITEM, parseChangelog, promoteUnreleased, sectionBody } from "./changelog.mjs";
import { updateChangelog } from "./set-version.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const script = join(here, "changelog.mjs");
const realPath = join(here, "..", "CHANGELOG.md");

const SAMPLE = `# Có gì mới

## Chưa phát hành
- Thêm **EPUB**.
- Sửa \`out/\`.

## 1.0.1 — 2026-09-11
- Sửa lỗi A.

## 1.0.0 — 2026-09-10
- Bản đầu tiên.
- Dòng hai.
`;

const RELEASED_ONLY = `# Có gì mới

## 1.0.1 — 2026-09-11
- Sửa lỗi A.
`;

describe("parseChangelog", () => {
  it("đọc mục Chưa phát hành (version/date null) và các bản theo thứ tự trong file", () => {
    expect(parseChangelog(SAMPLE)).toEqual([
      { version: null, date: null, items: ["Thêm **EPUB**.", "Sửa `out/`."] },
      { version: "1.0.1", date: "2026-09-11", items: ["Sửa lỗi A."] },
      { version: "1.0.0", date: "2026-09-10", items: ["Bản đầu tiên.", "Dòng hai."] },
    ]);
  });

  it("chấp nhận CRLF và bỏ qua dòng không phải bullet", () => {
    const text = "# Có gì mới\r\n\r\n## 1.0.0 — 2026-09-10\r\nghi chú lạc\r\n- Một.\r\n";
    expect(parseChangelog(text)).toEqual([{ version: "1.0.0", date: "2026-09-10", items: ["Một."] }]);
  });
});

describe("sectionBody", () => {
  it("trả các dòng bullet của bản đó, không kèm tiêu đề", () => {
    expect(sectionBody(SAMPLE, "1.0.0")).toBe("- Bản đầu tiên.\n- Dòng hai.");
    expect(sectionBody(SAMPLE, "1.0.1")).toBe("- Sửa lỗi A.");
  });

  it("không có bản đó thì null", () => {
    expect(sectionBody(SAMPLE, "9.9.9")).toBeNull();
  });
});

describe("promoteUnreleased", () => {
  it("có Chưa phát hành không rỗng: đổi tiêu đề thành bản mới, giữ bullet, không còn mục chưa phát hành", () => {
    const out = promoteUnreleased(SAMPLE, "1.0.2", "2026-10-15", ["Bị bỏ qua."]);
    expect(out).toBe(SAMPLE.replace("## Chưa phát hành", "## 1.0.2 — 2026-10-15"));
    expect(parseChangelog(out).some((e) => e.version === null)).toBe(false);
  });

  it("không có Chưa phát hành: chèn mục mới ngay sau tiêu đề với bullet nháp", () => {
    const out = promoteUnreleased(RELEASED_ONLY, "1.0.2", "2026-10-15", ["Thêm X.", "Sửa Y."]);
    expect(out).toBe(`# Có gì mới

## 1.0.2 — 2026-10-15
- Thêm X.
- Sửa Y.

## 1.0.1 — 2026-09-11
- Sửa lỗi A.
`);
  });

  it("nháp rỗng: chèn một bullet mặc định", () => {
    const out = promoteUnreleased(RELEASED_ONLY, "1.0.2", "2026-10-15", []);
    expect(sectionBody(out, "1.0.2")).toBe(`- ${FALLBACK_ITEM}`);
    expect(FALLBACK_ITEM).toBe("Sửa lỗi nhỏ và cải thiện độ ổn định.");
  });

  it("Chưa phát hành rỗng: bỏ mục rỗng, chèn mục mới từ nháp", () => {
    const text = RELEASED_ONLY.replace("# Có gì mới\n\n", "# Có gì mới\n\n## Chưa phát hành\n\n");
    const out = promoteUnreleased(text, "1.0.2", "2026-10-15", ["Thêm X."]);
    expect(parseChangelog(out)).toEqual([
      { version: "1.0.2", date: "2026-10-15", items: ["Thêm X."] },
      { version: "1.0.1", date: "2026-09-11", items: ["Sửa lỗi A."] },
    ]);
    expect(out).not.toContain("Chưa phát hành");
  });

  it("file chỉ có tiêu đề: thêm mục vào cuối", () => {
    const out = promoteUnreleased("# Có gì mới\n", "1.0.0", "2026-09-10", ["Một."]);
    expect(out).toBe("# Có gì mới\n\n## 1.0.0 — 2026-09-10\n- Một.\n");
  });

  it("đã có mục cho version đó thì giữ nguyên file", () => {
    expect(promoteUnreleased(SAMPLE, "1.0.1", "2026-10-15", ["Thêm X."])).toBe(SAMPLE);
  });

  it("giữ CRLF nếu file dùng CRLF", () => {
    const crlf = RELEASED_ONLY.replace(/\n/g, "\r\n");
    const out = promoteUnreleased(crlf, "1.0.2", "2026-10-15", ["Thêm X."]);
    expect(out).toBe(promoteUnreleased(RELEASED_ONLY, "1.0.2", "2026-10-15", ["Thêm X."]).replace(/\n/g, "\r\n"));
  });
});

describe("draftFromCommits", () => {
  it("giữ feat/fix/perf, bỏ tiền tố, viết hoa chữ đầu, thêm dấu chấm, xếp feat → perf → fix", () => {
    expect(
      draftFromCommits([
        "fix(qt-ai-core): lệch tiến độ chỉ tính bản dịch mới",
        "chore: dọn",
        "feat(qt-ai-gui): dialog đồng bộ tiến độ",
        "perf: đọc chương nhanh hơn.",
        "release(qt-ai-gui): 1.0.21",
        "feat!: đổi định dạng",
        "docs(x): y",
        "featured: không phải feat",
      ]),
    ).toEqual(["Dialog đồng bộ tiến độ.", "Đổi định dạng.", "Đọc chương nhanh hơn.", "Lệch tiến độ chỉ tính bản dịch mới."]);
  });
});

describe("CLI", () => {
  it("section in phần thân của bản có trong CHANGELOG.md thật", () => {
    const result = spawnSync(process.execPath, [script, "section", "1.0.21"], { encoding: "utf8" });
    expect(result.status).toBe(0);
    expect(result.stdout.trim()).toBe(sectionBody(readFileSync(realPath, "utf8"), "1.0.21"));
  });

  it("section của bản không có thì thoát mã 1", () => {
    const result = spawnSync(process.execPath, [script, "section", "0.0.1"], { encoding: "utf8" });
    expect(result.status).toBe(1);
    expect(result.stderr).toMatch(/0\.0\.1/);
  });

  it("github-output ghi body dạng heredoc vào file GITHUB_OUTPUT", () => {
    const out = join(mkdtempSync(join(tmpdir(), "changelog-")), "out.txt");
    writeFileSync(out, "khac=1\n");
    const result = spawnSync(process.execPath, [script, "github-output", "1.0.21"], {
      encoding: "utf8",
      env: { ...process.env, GITHUB_OUTPUT: out },
    });
    expect(result.status).toBe(0);
    const body = sectionBody(readFileSync(realPath, "utf8"), "1.0.21");
    expect(readFileSync(out, "utf8")).toBe(`khac=1\nbody<<__NOTES__\n${body}\n__NOTES__\n`);
  });

  it("github-output thiếu mục thì thoát mã 1, không ghi gì", () => {
    const out = join(mkdtempSync(join(tmpdir(), "changelog-")), "out.txt");
    writeFileSync(out, "");
    const result = spawnSync(process.execPath, [script, "github-output", "0.0.1"], {
      encoding: "utf8",
      env: { ...process.env, GITHUB_OUTPUT: out },
    });
    expect(result.status).toBe(1);
    expect(readFileSync(out, "utf8")).toBe("");
  });
});

describe("CHANGELOG.md thật", () => {
  const entries = parseChangelog(readFileSync(realPath, "utf8"));
  const released = entries.filter((e) => e.version !== null);

  it("mọi bản đã tag 1.0.0…1.0.21 đều có mục không rỗng", () => {
    for (let patch = 0; patch <= 21; patch++) {
      const entry = released.find((e) => e.version === `1.0.${patch}`);
      expect(entry, `1.0.${patch}`).toBeDefined();
      expect(entry?.items.length, `1.0.${patch}`).toBeGreaterThan(0);
      expect(entry?.date).toMatch(/^\d{4}-\d{2}-\d{2}$/);
    }
  });

  it("version giảm dần nghiêm ngặt, mục Chưa phát hành (nếu có) đứng đầu", () => {
    const key = (v) => v.split(".").map(Number);
    for (let i = 1; i < released.length; i++) {
      const [a, b] = [key(released[i - 1].version), key(released[i].version)];
      const cmp = a[0] - b[0] || a[1] - b[1] || a[2] - b[2];
      expect(cmp, `${released[i - 1].version} > ${released[i].version}`).toBeGreaterThan(0);
    }
    entries.forEach((e, i) => {
      if (e.version === null) expect(i).toBe(0);
    });
  });
});

describe("updateChangelog (set-version)", () => {
  it("ghi CHANGELOG.md trong appDir: đổi Chưa phát hành thành bản mới", () => {
    const dir = mkdtempSync(join(tmpdir(), "set-version-changelog-"));
    writeFileSync(join(dir, "CHANGELOG.md"), SAMPLE);
    expect(updateChangelog(dir, "1.0.2", ["feat: bỏ qua"], new Date(2026, 9, 5))).toBe(true);
    expect(readFileSync(join(dir, "CHANGELOG.md"), "utf8")).toBe(SAMPLE.replace("## Chưa phát hành", "## 1.0.2 — 2026-10-05"));
  });

  it("không có Chưa phát hành: dựng nháp từ commit; đã có mục thì không ghi", () => {
    const dir = mkdtempSync(join(tmpdir(), "set-version-changelog-"));
    writeFileSync(join(dir, "CHANGELOG.md"), RELEASED_ONLY);
    expect(updateChangelog(dir, "1.0.2", ["fix(gui): nút lưu", "chore: x"], new Date(2026, 0, 9))).toBe(true);
    expect(sectionBody(readFileSync(join(dir, "CHANGELOG.md"), "utf8"), "1.0.2")).toBe("- Nút lưu.");
    expect(parseChangelog(readFileSync(join(dir, "CHANGELOG.md"), "utf8"))[0].date).toBe("2026-01-09");
    expect(updateChangelog(dir, "1.0.2", [], new Date())).toBe(false);
  });
});
