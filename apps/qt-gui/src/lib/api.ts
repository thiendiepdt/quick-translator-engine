import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";

import {
  dictionaryDefaultsSchema,
  engineStatusSchema,
  nameFilterResponseSchema,
  openedTextFileSchema,
  translationResponseSchema,
} from "@/lib/schema";
import type {
  DictionaryDefaults,
  EngineStatus,
  NameFilterRequest,
  NameFilterResponse,
  OpenedTextFile,
  TranslationRequest,
  TranslationResponse,
} from "@/lib/types";

export class ApiError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "ApiError";
  }
}

async function invokeLocal<T>(
  command: string,
  args: Record<string, unknown> | undefined,
  parse: (value: unknown) => T,
): Promise<T> {
  try {
    return parse(await invoke(command, args));
  } catch (error) {
    if (error instanceof ApiError) throw error;
    throw new ApiError(
      typeof error === "string"
        ? error
        : error instanceof Error
          ? error.message
          : "Lệnh local thất bại",
    );
  }
}

export function translateChapter(
  request: TranslationRequest,
): Promise<TranslationResponse> {
  return invokeLocal(
    "translate",
    { request },
    (value) => translationResponseSchema.parse(value),
  );
}

export function getEngineStatus(): Promise<EngineStatus> {
  return invokeLocal("engine_status", undefined, (value) => engineStatusSchema.parse(value));
}

export function loadEngine(dataDir: string): Promise<EngineStatus> {
  return invokeLocal(
    "load_engine",
    { dataDir },
    (value) => engineStatusSchema.parse(value),
  );
}

export function fetchDictionaryDefaults(): Promise<DictionaryDefaults> {
  return invokeLocal(
    "dictionary_defaults",
    undefined,
    (value) => dictionaryDefaultsSchema.parse(value),
  );
}

export function filterChapterNames(
  request: NameFilterRequest,
): Promise<NameFilterResponse> {
  return invokeLocal(
    "filter_names",
    { request },
    (value) => nameFilterResponseSchema.parse(value),
  );
}

export async function chooseDataDirectory(): Promise<string | undefined> {
  const selected = await open({
    directory: true,
    multiple: false,
    title: "Chọn thư mục dữ liệu Quick Translator",
  });
  return typeof selected === "string" ? selected : undefined;
}

export async function openSourceFile(): Promise<OpenedTextFile | undefined> {
  const selected = await open({
    directory: false,
    multiple: false,
    title: "Mở chương tiếng Trung",
    filters: [{ name: "Văn bản UTF-8", extensions: ["txt", "text"] }],
  });
  if (typeof selected !== "string") return undefined;
  return invokeLocal(
    "read_text_file",
    { path: selected },
    (value) => openedTextFileSchema.parse(value),
  );
}

export async function saveOutputFile(
  content: string,
  defaultName = "translated.txt",
): Promise<string | undefined> {
  const selected = await save({
    title: "Lưu bản dịch",
    defaultPath: defaultName,
    filters: [{ name: "Văn bản UTF-8", extensions: ["txt"] }],
  });
  if (!selected) return undefined;
  await invokeLocal("write_text_file", { path: selected, content }, () => undefined);
  return selected;
}
