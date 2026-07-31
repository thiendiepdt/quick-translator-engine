import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";

import { createIndexedDbStateStorage } from "@/lib/indexed-db-storage";
import { sampleDictionaryValues, sampleResponse, sampleSource } from "@/lib/sample";
import {
  dictionaryUpdateKeys,
  dictionaryKeys,
  fixedDictionaryPatchKeys,
  type DictionaryDefaults,
  type DictionaryKey,
  type DictionaryPatchPayload,
  type DictionaryUpdateKey,
  type LocalDictionaryEntries,
  type NameFilterResponse,
  type TranslationResponse,
} from "@/lib/types";

interface DictionaryDraft {
  value: string;
  defaultValue: string;
  touched: boolean;
}

interface NameMemoryProfile {
  knownNames: Record<string, string>;
  rejectedNames: string[];
}

type SourceView = "raw" | "linked";
type OutputView = "output" | "json";
export type WorkspaceView = "translate" | "names";

interface WorkspaceState {
  sourceText: string;
  response?: TranslationResponse;
  activeRange?: number;
  activeDictionary: DictionaryKey;
  dictionaries: Record<DictionaryKey, DictionaryDraft>;
  dictionaryOverrides: Partial<Record<DictionaryKey, string>>;
  dictionaryDefaultsEndpoint?: string;
  sourceView: SourceView;
  outputView: OutputView;
  rangePinEnabled: boolean;
  mobileInspectorOpen: boolean;
  workspaceView: WorkspaceView;
  nameFilterResponse?: NameFilterResponse;
  knownNames: Record<string, string>;
  rejectedNames: string[];
  nameMemoryId: string;
  nameMemoryProfiles: Record<string, NameMemoryProfile>;
  localDictionaryEntries: LocalDictionaryEntries;
  setSourceText: (sourceText: string) => void;
  setResponse: (response: TranslationResponse) => void;
  setActiveRange: (activeRange?: number) => void;
  setActiveDictionary: (activeDictionary: DictionaryKey) => void;
  setDictionaryValue: (key: DictionaryKey, value: string) => void;
  resetDictionary: (key: DictionaryKey) => void;
  hydrateDictionaryDefaults: (endpoint: string, defaults: DictionaryDefaults) => void;
  setSourceView: (sourceView: SourceView) => void;
  setOutputView: (outputView: OutputView) => void;
  setRangePinEnabled: (rangePinEnabled: boolean) => void;
  setMobileInspectorOpen: (mobileInspectorOpen: boolean) => void;
  setWorkspaceView: (workspaceView: WorkspaceView) => void;
  setNameFilterResponse: (nameFilterResponse?: NameFilterResponse) => void;
  acceptNameCandidate: (text: string, suggested: string) => void;
  rejectNameCandidate: (text: string) => void;
  clearNameMemory: () => void;
  switchNameMemory: (nameMemoryId: string) => void;
  saveLocalDictionaryEntries: (
    key: DictionaryUpdateKey,
    entries: Record<string, string>,
    previousKeys?: string[],
  ) => void;
  removeLocalDictionaryEntries: (
    key: DictionaryUpdateKey,
    keys: string[],
  ) => void;
  clearWorkspace: () => void;
  loadSample: () => void;
}

function emptyDictionaries(): Record<DictionaryKey, DictionaryDraft> {
  return Object.fromEntries(
    dictionaryKeys.map((key) => [key, { value: "", defaultValue: "", touched: false }]),
  ) as Record<DictionaryKey, DictionaryDraft>;
}

function emptyLocalDictionaryEntries(): LocalDictionaryEntries {
  return Object.fromEntries(
    dictionaryUpdateKeys.map((key) => [key, {}]),
  ) as LocalDictionaryEntries;
}

function isEditableDictionaryKey(
  key: DictionaryUpdateKey,
): key is Exclude<DictionaryUpdateKey, "vietPhrase" | "chinesePhienAmWords"> {
  return key !== "vietPhrase" && key !== "chinesePhienAmWords";
}

function applyPersistentEntries(
  dictionaries: Record<DictionaryKey, DictionaryDraft>,
  entries: LocalDictionaryEntries,
  knownNames: Record<string, string>,
): Record<DictionaryKey, DictionaryDraft> {
  const next = { ...dictionaries };
  for (const key of dictionaryUpdateKeys) {
    if (!isEditableDictionaryKey(key)) continue;
    let value = next[key].value;
    if (key === "names2") value = upsertKnownNames(value, knownNames);
    value = applyDictionaryEntries(value, entries[key]);
    next[key] = {
      ...next[key],
      value,
      touched: value !== next[key].defaultValue,
    };
  }
  return next;
}

function dictionariesFromDefaults(
  defaults: DictionaryDefaults,
  overrides: Partial<Record<DictionaryKey, string>> = {},
): Record<DictionaryKey, DictionaryDraft> {
  return Object.fromEntries(
    dictionaryKeys.map((key) => {
      const hasOverride = Object.hasOwn(overrides, key);
      const value = hasOverride ? (overrides[key] ?? defaults[key]) : defaults[key];
      return [
        key,
        { value, defaultValue: defaults[key], touched: value !== defaults[key] },
      ];
    }),
  ) as Record<DictionaryKey, DictionaryDraft>;
}

function resetDictionaries(
  dictionaries: Record<DictionaryKey, DictionaryDraft>,
  overrides: Partial<Record<DictionaryKey, string>> = {},
): Record<DictionaryKey, DictionaryDraft> {
  return Object.fromEntries(
    dictionaryKeys.map((key) => {
      const defaultValue = dictionaries[key].defaultValue;
      const hasOverride = Object.hasOwn(overrides, key);
      const value = hasOverride ? (overrides[key] ?? defaultValue) : defaultValue;
      return [
        key,
        {
          value,
          defaultValue,
          touched: value !== defaultValue,
        },
      ];
    }),
  ) as Record<DictionaryKey, DictionaryDraft>;
}

function dictionaryBaseValue(
  overrides: Partial<Record<DictionaryKey, string>>,
  key: DictionaryKey,
  defaultValue: string,
): string {
  return Object.hasOwn(overrides, key) ? (overrides[key] ?? defaultValue) : defaultValue;
}

export const legacyWorkspaceStorageKey = "qt-web-name-memory-v1";
export const workspaceStorageKey = "qt-web-workspace-v1";
export const workspaceStateStorage = createIndexedDbStateStorage({
  legacyLocalStorageKeys: [legacyWorkspaceStorageKey],
});

export const useWorkspaceStore = create<WorkspaceState>()(persist((set) => ({
  sourceText: "",
  response: undefined,
  activeRange: undefined,
  activeDictionary: "names",
  dictionaries: emptyDictionaries(),
  dictionaryOverrides: {},
  dictionaryDefaultsEndpoint: undefined,
  sourceView: "raw",
  outputView: "output",
  rangePinEnabled: true,
  mobileInspectorOpen: false,
  workspaceView: "translate",
  nameFilterResponse: undefined,
  knownNames: {},
  rejectedNames: [],
  nameMemoryId: "default",
  nameMemoryProfiles: {},
  localDictionaryEntries: emptyLocalDictionaryEntries(),
  setSourceText: (sourceText) =>
    set({
      sourceText,
      response: undefined,
      nameFilterResponse: undefined,
      activeRange: undefined,
      sourceView: "raw",
    }),
  setResponse: (response) =>
    set({ response, activeRange: undefined, sourceView: "linked", outputView: "output" }),
  setActiveRange: (activeRange) => set({ activeRange }),
  setActiveDictionary: (activeDictionary) => set({ activeDictionary }),
  setDictionaryValue: (key, value) =>
    set((state) => {
      const dictionaryOverrides = { ...state.dictionaryOverrides };
      let localDictionaryEntries = state.localDictionaryEntries;
      if (state.dictionaryDefaultsEndpoint) {
        if (value === state.dictionaries[key].defaultValue) delete dictionaryOverrides[key];
        else dictionaryOverrides[key] = value;

        if (dictionaryUpdateKeys.includes(key as DictionaryUpdateKey)) {
          localDictionaryEntries = {
            ...state.localDictionaryEntries,
            [key as DictionaryUpdateKey]: {},
          };
        }
      }
      return {
        dictionaryOverrides,
        localDictionaryEntries,
        dictionaries: {
          ...state.dictionaries,
          [key]: {
            ...state.dictionaries[key],
            value,
            touched: value !== state.dictionaries[key].defaultValue,
          },
        },
      };
    }),
  resetDictionary: (key) =>
    set((state) => {
      const localDictionaryEntries = { ...state.localDictionaryEntries };
      const dictionaryOverrides = { ...state.dictionaryOverrides };
      delete dictionaryOverrides[key];
      if (dictionaryUpdateKeys.includes(key as DictionaryUpdateKey)) {
        localDictionaryEntries[key as DictionaryUpdateKey] = {};
      }
      let value = state.dictionaries[key].defaultValue;
      if (key === "names2") value = upsertKnownNames(value, state.knownNames);
      return {
        dictionaryOverrides,
        localDictionaryEntries,
        dictionaries: {
          ...state.dictionaries,
          [key]: {
            value,
            defaultValue: state.dictionaries[key].defaultValue,
            touched: value !== state.dictionaries[key].defaultValue,
          },
        },
      };
    }),
  hydrateDictionaryDefaults: (endpoint, defaults) =>
    set((state) => {
      if (state.dictionaryDefaultsEndpoint === endpoint) return state;
      const dictionaries = applyPersistentEntries(
        dictionariesFromDefaults(defaults, state.dictionaryOverrides),
        state.localDictionaryEntries,
        state.knownNames,
      );
      return {
        dictionaries,
        dictionaryDefaultsEndpoint: endpoint,
      };
    }),
  setSourceView: (sourceView) => set({ sourceView }),
  setOutputView: (outputView) => set({ outputView }),
  setRangePinEnabled: (rangePinEnabled) => set({ rangePinEnabled }),
  setMobileInspectorOpen: (mobileInspectorOpen) => set({ mobileInspectorOpen }),
  setWorkspaceView: (workspaceView) => set({ workspaceView }),
  setNameFilterResponse: (nameFilterResponse) => set({ nameFilterResponse }),
  acceptNameCandidate: (text, suggested) =>
    set((state) => {
      const knownNames = { ...state.knownNames, [text]: suggested };
      const names2Value = upsertDictionaryEntry(state.dictionaries.names2.value, text, suggested);
      return {
        knownNames,
        rejectedNames: state.rejectedNames.filter((value) => value !== text),
        nameMemoryProfiles: {
          ...state.nameMemoryProfiles,
          [state.nameMemoryId]: {
            knownNames,
            rejectedNames: state.rejectedNames.filter((value) => value !== text),
          },
        },
        dictionaries: {
          ...state.dictionaries,
          names2: {
            ...state.dictionaries.names2,
            value: names2Value,
            touched: names2Value !== state.dictionaries.names2.defaultValue,
          },
        },
      };
    }),
  rejectNameCandidate: (text) =>
    set((state) => {
      const knownNames = { ...state.knownNames };
      const acceptedValue = knownNames[text];
      delete knownNames[text];
      const names2Value = acceptedValue
        ? removeDictionaryEntry(state.dictionaries.names2.value, text, acceptedValue)
        : state.dictionaries.names2.value;
      return {
        knownNames,
        rejectedNames: state.rejectedNames.includes(text)
          ? state.rejectedNames
          : [...state.rejectedNames, text],
        nameMemoryProfiles: {
          ...state.nameMemoryProfiles,
          [state.nameMemoryId]: {
            knownNames,
            rejectedNames: state.rejectedNames.includes(text)
              ? state.rejectedNames
              : [...state.rejectedNames, text],
          },
        },
        dictionaries: {
          ...state.dictionaries,
          names2: {
            ...state.dictionaries.names2,
            value: names2Value,
            touched: names2Value !== state.dictionaries.names2.defaultValue,
          },
        },
      };
    }),
  clearNameMemory: () =>
    set((state) => {
      const names2Value = Object.entries(state.knownNames).reduce(
        (content, [key, value]) => removeDictionaryEntry(content, key, value),
        state.dictionaries.names2.value,
      );
      return {
        knownNames: {},
        rejectedNames: [],
        nameMemoryProfiles: {
          ...state.nameMemoryProfiles,
          [state.nameMemoryId]: { knownNames: {}, rejectedNames: [] },
        },
        dictionaries: {
          ...state.dictionaries,
          names2: {
            ...state.dictionaries.names2,
            value: names2Value,
            touched: names2Value !== state.dictionaries.names2.defaultValue,
          },
        },
      };
    }),
  switchNameMemory: (nameMemoryId) =>
    set((state) => {
      const normalized = nameMemoryId.trim() || "default";
      if (normalized === state.nameMemoryId) return state;
      const nameMemoryProfiles = {
        ...state.nameMemoryProfiles,
        [state.nameMemoryId]: {
          knownNames: state.knownNames,
          rejectedNames: state.rejectedNames,
        },
      };
      const next = nameMemoryProfiles[normalized] ?? { knownNames: {}, rejectedNames: [] };
      let names2Value = Object.entries(state.knownNames).reduce(
        (content, [key, value]) => removeDictionaryEntry(content, key, value),
        state.dictionaries.names2.value,
      );
      names2Value = upsertKnownNames(names2Value, next.knownNames);
      return {
        nameMemoryId: normalized,
        nameMemoryProfiles,
        knownNames: next.knownNames,
        rejectedNames: next.rejectedNames,
        nameFilterResponse: undefined,
        dictionaries: {
          ...state.dictionaries,
          names2: {
            ...state.dictionaries.names2,
            value: names2Value,
            touched: names2Value !== state.dictionaries.names2.defaultValue,
          },
        },
      };
    }),
  saveLocalDictionaryEntries: (key, entries, previousKeys = []) =>
    set((state) => {
      const group = { ...state.localDictionaryEntries[key] };
      for (const previousKey of previousKeys) {
        if (!(previousKey in entries)) delete group[previousKey];
      }
      Object.assign(group, entries);
      const localDictionaryEntries = {
        ...state.localDictionaryEntries,
        [key]: group,
      };
      if (!isEditableDictionaryKey(key)) return { localDictionaryEntries };

      let value = state.dictionaries[key].value;
      for (const previousKey of previousKeys) {
        if (previousKey in entries) continue;
        value = restoreDictionaryEntry(
          value,
          dictionaryBaseValue(
            state.dictionaryOverrides,
            key,
            state.dictionaries[key].defaultValue,
          ),
          previousKey,
          key === "names2" ? state.knownNames[previousKey] : undefined,
        );
      }
      value = applyDictionaryEntries(value, entries);
      return {
        localDictionaryEntries,
        dictionaries: {
          ...state.dictionaries,
          [key]: {
            ...state.dictionaries[key],
            value,
            touched: value !== state.dictionaries[key].defaultValue,
          },
        },
      };
    }),
  removeLocalDictionaryEntries: (key, keys) =>
    set((state) => {
      const group = { ...state.localDictionaryEntries[key] };
      for (const entryKey of keys) delete group[entryKey];
      const localDictionaryEntries = {
        ...state.localDictionaryEntries,
        [key]: group,
      };
      if (!isEditableDictionaryKey(key)) return { localDictionaryEntries };

      let value = state.dictionaries[key].value;
      for (const entryKey of keys) {
        value = restoreDictionaryEntry(
          value,
          dictionaryBaseValue(
            state.dictionaryOverrides,
            key,
            state.dictionaries[key].defaultValue,
          ),
          entryKey,
          key === "names2" ? state.knownNames[entryKey] : undefined,
        );
      }
      return {
        localDictionaryEntries,
        dictionaries: {
          ...state.dictionaries,
          [key]: {
            ...state.dictionaries[key],
            value,
            touched: value !== state.dictionaries[key].defaultValue,
          },
        },
      };
    }),
  clearWorkspace: () =>
    set((state) => {
      const dictionaries = applyPersistentEntries(
        resetDictionaries(state.dictionaries, state.dictionaryOverrides),
        state.localDictionaryEntries,
        state.knownNames,
      );
      return {
        sourceText: "",
        response: undefined,
        nameFilterResponse: undefined,
        activeRange: undefined,
        dictionaries,
        sourceView: "raw",
        outputView: "output",
      };
    }),
  loadSample: () =>
    set((state) => {
      let dictionaries = resetDictionaries(state.dictionaries, state.dictionaryOverrides);
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
      dictionaries = applyPersistentEntries(
        dictionaries,
        state.localDictionaryEntries,
        state.knownNames,
      );
      return {
        sourceText: sampleSource,
        response: sampleResponse,
        nameFilterResponse: undefined,
        activeRange: undefined,
        dictionaries,
        sourceView: "linked",
        outputView: "output",
      };
    }),
}), {
  name: workspaceStorageKey,
  storage: createJSONStorage(() => workspaceStateStorage),
  merge: (persistedState, currentState) => {
    const merged = {
      ...currentState,
      ...(persistedState as Partial<WorkspaceState>),
    };
    return {
      ...merged,
      dictionaries: applyPersistentEntries(
        resetDictionaries(merged.dictionaries, merged.dictionaryOverrides),
        merged.localDictionaryEntries,
        merged.knownNames,
      ),
    };
  },
  partialize: (state) => ({
    knownNames: state.knownNames,
    rejectedNames: state.rejectedNames,
    nameMemoryId: state.nameMemoryId,
    nameMemoryProfiles: state.nameMemoryProfiles,
    rangePinEnabled: state.rangePinEnabled,
    localDictionaryEntries: state.localDictionaryEntries,
    dictionaryOverrides: state.dictionaryOverrides,
  }),
}));

function upsertKnownNames(content: string, knownNames: Record<string, string>): string {
  return Object.entries(knownNames).reduce(
    (current, [key, value]) => upsertDictionaryEntry(current, key, value),
    content,
  );
}

function upsertDictionaryEntry(content: string, key: string, value: string): string {
  const hasBom = content.startsWith("\uFEFF");
  const body = hasBom ? content.slice(1) : content;
  const lineEnding = body.includes("\r\n") ? "\r\n" : "\n";
  const trailingNewline = body.endsWith("\n");
  const lines = body.length === 0 ? [] : body.split(/\r?\n/);
  if (trailingNewline) lines.pop();
  const prefix = `${key}=`;
  const index = lines.findIndex((line) => line.startsWith(prefix));
  const record = `${key}=${value}`;
  if (index >= 0) lines[index] = record;
  else lines.push(record);
  const suffix = trailingNewline && lines.length > 0 ? lineEnding : "";
  return `${hasBom ? "\uFEFF" : ""}${lines.join(lineEnding)}${suffix}`;
}

function applyDictionaryEntries(
  content: string,
  entries: Record<string, string>,
): string {
  return Object.entries(entries).reduce(
    (current, [key, value]) => upsertDictionaryEntry(current, key, value),
    content,
  );
}

function dictionaryEntry(content: string, key: string): string | undefined {
  const body = content.startsWith("\uFEFF") ? content.slice(1) : content;
  const prefix = `${key}=`;
  const line = body.split(/\r?\n/).find((value) => value.startsWith(prefix));
  return line?.slice(prefix.length);
}

function removeDictionaryKey(content: string, key: string): string {
  const hasBom = content.startsWith("\uFEFF");
  const body = hasBom ? content.slice(1) : content;
  const lineEnding = body.includes("\r\n") ? "\r\n" : "\n";
  const trailingNewline = body.endsWith("\n");
  const prefix = `${key}=`;
  const lines = (body.length === 0 ? [] : body.split(/\r?\n/)).filter(
    (line) => !line.startsWith(prefix),
  );
  if (trailingNewline && lines.at(-1) === "") lines.pop();
  const suffix = trailingNewline && lines.length > 0 ? lineEnding : "";
  return `${hasBom ? "\uFEFF" : ""}${lines.join(lineEnding)}${suffix}`;
}

function restoreDictionaryEntry(
  content: string,
  defaultContent: string,
  key: string,
  preferredValue?: string,
): string {
  const withoutEntry = removeDictionaryKey(content, key);
  const value = preferredValue ?? dictionaryEntry(defaultContent, key);
  return value === undefined
    ? withoutEntry
    : upsertDictionaryEntry(withoutEntry, key, value);
}

function removeDictionaryEntry(content: string, key: string, value: string): string {
  const hasBom = content.startsWith("\uFEFF");
  const body = hasBom ? content.slice(1) : content;
  const lineEnding = body.includes("\r\n") ? "\r\n" : "\n";
  const trailingNewline = body.endsWith("\n");
  const record = `${key}=${value}`;
  const lines = (body.length === 0 ? [] : body.split(/\r?\n/)).filter(
    (line) => line !== record,
  );
  if (trailingNewline && lines.at(-1) === "") lines.pop();
  const suffix = trailingNewline && lines.length > 0 ? lineEnding : "";
  return `${hasBom ? "\uFEFF" : ""}${lines.join(lineEnding)}${suffix}`;
}

export function dictionaryPayload(
  dictionaries: Record<DictionaryKey, DictionaryDraft>,
): Partial<Record<DictionaryKey, string>> | undefined {
  const entries = dictionaryKeys
    .filter((key) => dictionaries[key].touched)
    .map((key) => [key, dictionaries[key].value] as const);
  return entries.length > 0 ? Object.fromEntries(entries) : undefined;
}

export function dictionaryPatchPayload(
  entries: LocalDictionaryEntries,
): DictionaryPatchPayload | undefined {
  const payload = Object.fromEntries(
    fixedDictionaryPatchKeys
      .filter((key) => Object.keys(entries[key]).length > 0)
      .map((key) => [key, entries[key]]),
  ) as DictionaryPatchPayload;
  return Object.keys(payload).length > 0 ? payload : undefined;
}
