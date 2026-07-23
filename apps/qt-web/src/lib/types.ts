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
    label: "Names",
    shortLabel: "Names",
    filename: "Names.txt",
    description: "Tên chính, ưu tiên trước VietPhrase khi trùng key.",
  },
  {
    key: "names2",
    label: "Names 2",
    shortLabel: "Names2",
    filename: "Names2/*.txt",
    description: "Tên phụ, có độ ưu tiên cao hơn Names.",
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
    label: "Ignored Chinese Phrases",
    shortLabel: "Ignored",
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
  mode: "vietphrase-one";
  wrap: boolean;
  pretty: boolean;
  ranges: true;
  scanRange: number;
  translationAlgorithm: 0 | 1 | 2;
  prioritizedName: boolean;
  dictionaries?: Partial<Record<DictionaryKey, string>>;
}

export interface HealthResponse {
  status: "ok";
}
