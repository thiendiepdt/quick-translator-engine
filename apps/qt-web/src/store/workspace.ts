import { create } from "zustand";

import { sampleDictionaryValues, sampleResponse, sampleSource } from "@/lib/sample";
import { dictionaryKeys, type DictionaryKey, type TranslationResponse } from "@/lib/types";

interface DictionaryDraft {
  value: string;
  touched: boolean;
}

type SourceView = "raw" | "linked";
type OutputView = "output" | "json";

interface WorkspaceState {
  sourceText: string;
  response?: TranslationResponse;
  activeRange?: number;
  activeDictionary: DictionaryKey;
  dictionaries: Record<DictionaryKey, DictionaryDraft>;
  sourceView: SourceView;
  outputView: OutputView;
  mobileInspectorOpen: boolean;
  setSourceText: (sourceText: string) => void;
  setResponse: (response: TranslationResponse) => void;
  setActiveRange: (activeRange?: number) => void;
  setActiveDictionary: (activeDictionary: DictionaryKey) => void;
  setDictionaryValue: (key: DictionaryKey, value: string) => void;
  resetDictionary: (key: DictionaryKey) => void;
  setSourceView: (sourceView: SourceView) => void;
  setOutputView: (outputView: OutputView) => void;
  setMobileInspectorOpen: (mobileInspectorOpen: boolean) => void;
  clearWorkspace: () => void;
  loadSample: () => void;
}

function emptyDictionaries(): Record<DictionaryKey, DictionaryDraft> {
  return Object.fromEntries(
    dictionaryKeys.map((key) => [key, { value: "", touched: false }]),
  ) as Record<DictionaryKey, DictionaryDraft>;
}

export const useWorkspaceStore = create<WorkspaceState>()((set) => ({
  sourceText: "",
  response: undefined,
  activeRange: undefined,
  activeDictionary: "names",
  dictionaries: emptyDictionaries(),
  sourceView: "raw",
  outputView: "output",
  mobileInspectorOpen: false,
  setSourceText: (sourceText) =>
    set({ sourceText, response: undefined, activeRange: undefined, sourceView: "raw" }),
  setResponse: (response) =>
    set({ response, activeRange: undefined, sourceView: "linked", outputView: "output" }),
  setActiveRange: (activeRange) => set({ activeRange }),
  setActiveDictionary: (activeDictionary) => set({ activeDictionary }),
  setDictionaryValue: (key, value) =>
    set((state) => ({
      dictionaries: {
        ...state.dictionaries,
        [key]: { value, touched: true },
      },
    })),
  resetDictionary: (key) =>
    set((state) => ({
      dictionaries: {
        ...state.dictionaries,
        [key]: { value: "", touched: false },
      },
    })),
  setSourceView: (sourceView) => set({ sourceView }),
  setOutputView: (outputView) => set({ outputView }),
  setMobileInspectorOpen: (mobileInspectorOpen) => set({ mobileInspectorOpen }),
  clearWorkspace: () =>
    set({
      sourceText: "",
      response: undefined,
      activeRange: undefined,
      dictionaries: emptyDictionaries(),
      sourceView: "raw",
      outputView: "output",
    }),
  loadSample: () => {
    const dictionaries = emptyDictionaries();
    dictionaries.names = { value: sampleDictionaryValues.names, touched: true };
    dictionaries.pronouns = { value: sampleDictionaryValues.pronouns, touched: true };
    set({
      sourceText: sampleSource,
      response: sampleResponse,
      activeRange: undefined,
      dictionaries,
      sourceView: "linked",
      outputView: "output",
    });
  },
}));

export function dictionaryPayload(
  dictionaries: Record<DictionaryKey, DictionaryDraft>,
): Partial<Record<DictionaryKey, string>> | undefined {
  const entries = dictionaryKeys
    .filter((key) => dictionaries[key].touched)
    .map((key) => [key, dictionaries[key].value] as const);
  return entries.length > 0 ? Object.fromEntries(entries) : undefined;
}
