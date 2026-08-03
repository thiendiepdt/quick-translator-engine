export const dictionaryKeys = [
  "names",
  "names2",
  "luatNhan",
  "pronouns",
  "danhTu",
  "hoNguoi",
  "hauTu",
  "ignoredChinesePhrases",
] as const;

export type DictionaryKey = (typeof dictionaryKeys)[number];
export type DictionaryDefaults = Record<DictionaryKey, string>;

export const fixedDictionaryPatchKeys = [
  "vietPhrase",
  "chinesePhienAmWords",
] as const;

export type FixedDictionaryPatchKey = (typeof fixedDictionaryPatchKeys)[number];

export const dictionaryUpdateKeys = [
  "vietPhrase",
  "names",
  "names2",
  "chinesePhienAmWords",
  "danhTu",
  "hauTu",
  "hoNguoi",
  "luatNhan",
] as const;

export type DictionaryUpdateKey = (typeof dictionaryUpdateKeys)[number];
export type LocalDictionaryEntries = Record<
  DictionaryUpdateKey,
  Record<string, string>
>;
export type DictionaryPatchPayload = Partial<
  Record<FixedDictionaryPatchKey, Record<string, string>>
>;

export interface DictionaryDefinition {
  key: DictionaryKey;
  label: string;
  shortLabel: string;
  filename: string;
  description: string;
}

export const dictionaryDefinitions: DictionaryDefinition[] = [
  {
    key: "names",
    label: "Tên",
    shortLabel: "Tên",
    filename: "Names.txt",
    description: "Tên chính, ưu tiên trước VietPhrase khi trùng khóa.",
  },
  {
    key: "names2",
    label: "Tên 2",
    shortLabel: "Tên 2",
    filename: "Names2/*.txt",
    description: "Tên phụ, có độ ưu tiên cao hơn Tên.",
  },
  {
    key: "luatNhan",
    label: "Luật Nhân",
    shortLabel: "Luật Nhân",
    filename: "LuatNhan.txt",
    description: "Thay toàn bộ tập luật regex của request.",
  },
  {
    key: "pronouns",
    label: "Pronouns",
    shortLabel: "Đại từ",
    filename: "Resources/Pronouns.txt",
    description: "Đại từ dùng trong các mẫu Luật Nhân.",
  },
  {
    key: "danhTu",
    label: "Danh Từ",
    shortLabel: "Danh Từ",
    filename: "Resources/DanhTu.txt",
    description: "Được API nhận và parse cho request hiện tại.",
  },
  {
    key: "hoNguoi",
    label: "Họ Người",
    shortLabel: "Họ Người",
    filename: "Resources/HoNguoi.txt",
    description: "Danh sách họ dùng cho luật {h}{t}.",
  },
  {
    key: "hauTu",
    label: "Hậu Từ",
    shortLabel: "Hậu Từ",
    filename: "Resources/HauTu.txt",
    description: "Danh sách hậu tố dùng cho luật {h}{t}.",
  },
  {
    key: "ignoredChinesePhrases",
    label: "Cụm tiếng Trung bỏ qua",
    shortLabel: "Bỏ qua",
    filename: "IgnoredChinesePhrases.txt",
    description: "Cụm bị loại trong bước chuẩn hóa input.",
  },
];

export interface TextRange {
  start: number;
  length: number;
}

export interface TranslationResponse {
  translated: string;
  sourceRanges?: TextRange[];
  targetRanges?: TextRange[];
}

export interface TranslationRequest {
  text: string;
  mode: "hanviet" | "vietphrase" | "vietphrase-one";
  wrap: boolean;
  pretty: boolean;
  ranges: true;
  scanRange: number;
  translationAlgorithm: 0 | 1 | 2;
  prioritizedName: boolean;
  dictionaries?: Partial<Record<DictionaryKey, string>>;
  dictionaryPatches?: DictionaryPatchPayload;
}

export interface LacVietMeaning {
  source: string;
  definition: string;
}

export interface MeaningsResponse {
  entries: LacVietMeaning[];
}

export interface HealthResponse {
  status: "ok";
}

export type NameEntityType = "person" | "location" | "organization" | "title" | "unknown";

export interface NameCandidate {
  text: string;
  suggested: string;
  entityType: NameEntityType;
  score: number;
  occurrences: number;
  ranges: TextRange[];
  contexts: string[];
  reasons: string[];
  sources: string[];
  known: boolean;
}

export interface NameFilterResponse {
  candidates: NameCandidate[];
  stats: {
    scannedCharacters: number;
    ruleCandidates: number;
    aiMergedCandidates: number;
  };
  warnings?: string[];
}

export type NameFilterMode = "qt" | "hybrid";

/** Entity do AI trích ở trình duyệt, gửi lên server như dữ liệu trơ. */
export interface AiEntityPayload {
  text: string;
  entityType?: string;
  suggested?: string;
  confidence: number;
}

export interface NameFilterRequest {
  text: string;
  mode: NameFilterMode;
  minOccurrences: number;
  minConfidence: number;
  maxCandidates: number;
  knownNames: Record<string, string>;
  rejectedNames: string[];
  /** Chỉ gửi khi có entity — server merge chúng với ứng viên từ quy tắc.
   * Trình duyệt tự gọi AI trước đó; server không bao giờ thấy API key. */
  aiEntities?: { entities: AiEntityPayload[]; minConfidence?: number };
  dictionaries?: Partial<Record<DictionaryKey, string>>;
}
