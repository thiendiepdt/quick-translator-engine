import { create } from "zustand";

import { sampleDictionaryValues, sampleResponse, sampleSource } from "@/lib/sample";
import {
  dictionaryKeys,
  type DictionaryDefaults,
  type DictionaryKey,
  type TranslationResponse,
} from "@/lib/types";

interface DictionaryDraft {
  value: string;
  defaultValue: string;
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
  dictionaryDefaultsEndpoint?: string;
  sourceView: SourceView;
  outputView: OutputView;
  mobileInspectorOpen: boolean;
  setSourceText: (sourceText: string) => void;
  setResponse: (response: TranslationResponse) => void;
  setActiveRange: (activeRange?: number) => void;
  setActiveDictionary: (activeDictionary: DictionaryKey) => void;
  setDictionaryValue: (key: DictionaryKey, value: string) => void;
  resetDictionary: (key: DictionaryKey) => void;
  hydrateDictionaryDefaults: (endpoint: string, defaults: DictionaryDefaults) => void;
  setSourceView: (sourceView: SourceView) => void;
  setOutputView: (outputView: OutputView) => void;
  setMobileInspectorOpen: (mobileInspectorOpen: boolean) => void;
  clearWorkspace: () => void;
  loadSample: () => void;
}

function emptyDictionaries(): Record<DictionaryKey, DictionaryDraft> {
  return Object.fromEntries(
    dictionaryKeys.map((key) => [key, { value: "", defaultValue: "", touched: false }]),
  ) as Record<DictionaryKey, DictionaryDraft>;
}

function dictionariesFromDefaults(
  defaults: DictionaryDefaults,
): Record<DictionaryKey, DictionaryDraft> {
  return Object.fromEntries(
    dictionaryKeys.map((key) => [
      key,
      { value: defaults[key], defaultValue: defaults[key], touched: false },
    ]),
  ) as Record<DictionaryKey, DictionaryDraft>;
}

function resetDictionaries(
  dictionaries: Record<DictionaryKey, DictionaryDraft>,
): Record<DictionaryKey, DictionaryDraft> {
  return Object.fromEntries(
    dictionaryKeys.map((key) => [
      key,
      {
        value: dictionaries[key].defaultValue,
        defaultValue: dictionaries[key].defaultValue,
        touched: false,
      },
    ]),
  ) as Record<DictionaryKey, DictionaryDraft>;
}

export const useWorkspaceStore = create<WorkspaceState>()((set) => ({
  sourceText: "",
  response: undefined,
  activeRange: undefined,
  activeDictionary: "names",
  dictionaries: emptyDictionaries(),
  dictionaryDefaultsEndpoint: undefined,
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
        [key]: {
          ...state.dictionaries[key],
          value,
          touched: value !== state.dictionaries[key].defaultValue,
        },
      },
    })),
  resetDictionary: (key) =>
    set((state) => ({
      dictionaries: {
        ...state.dictionaries,
        [key]: {
          value: state.dictionaries[key].defaultValue,
          defaultValue: state.dictionaries[key].defaultValue,
          touched: false,
        },
      },
    })),
  hydrateDictionaryDefaults: (endpoint, defaults) =>
    set((state) => {
      if (state.dictionaryDefaultsEndpoint === endpoint) return state;
      return {
        dictionaries: dictionariesFromDefaults(defaults),
        dictionaryDefaultsEndpoint: endpoint,
      };
    }),
  setSourceView: (sourceView) => set({ sourceView }),
  setOutputView: (outputView) => set({ outputView }),
  setMobileInspectorOpen: (mobileInspectorOpen) => set({ mobileInspectorOpen }),
  clearWorkspace: () =>
    set((state) => ({
      sourceText: "",
      response: undefined,
      activeRange: undefined,
      dictionaries: resetDictionaries(state.dictionaries),
      sourceView: "raw",
      outputView: "output",
    })),
  loadSample: () =>
    set((state) => {
      const dictionaries = resetDictionaries(state.dictionaries);
      dictionaries.names = {
        ...dictionaries.names,
        value: sampleDictionaryValues.names,
        touched: sampleDictionaryValues.names !== dictionaries.names.defaultValue,
      };
      dictionaries.pronouns = {
        ...dictionaries.pronouns,
        value: sampleDictionaryValues.pronouns,
        touched: sampleDictionaryValues.pronouns !== dictionaries.pronouns.defaultValue,
      };
      return {
        sourceText: sampleSource,
        response: sampleResponse,
        activeRange: undefined,
        dictionaries,
        sourceView: "linked",
        outputView: "output",
      };
    }),
}));

export function dictionaryPayload(
  dictionaries: Record<DictionaryKey, DictionaryDraft>,
): Partial<Record<DictionaryKey, string>> | undefined {
  const entries = dictionaryKeys
    .filter((key) => dictionaries[key].touched)
    .map((key) => [key, dictionaries[key].value] as const);
  return entries.length > 0 ? Object.fromEntries(entries) : undefined;
}
