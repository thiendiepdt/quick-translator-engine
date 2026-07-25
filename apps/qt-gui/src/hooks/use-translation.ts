import { useMutation, useQuery } from "@tanstack/react-query";

import {
  fetchDictionaryDefaults,
  filterChapterNames,
  getEngineStatus,
  loadEngine,
  translateChapter,
} from "@/lib/api";
import type { NameFilterRequest, TranslationRequest } from "@/lib/types";

export function useTranslationMutation() {
  return useMutation({
    mutationKey: ["desktop", "translate"],
    mutationFn: (request: TranslationRequest) => translateChapter(request),
    retry: false,
  });
}

export function useEngineStatusQuery() {
  return useQuery({
    queryKey: ["desktop", "engine-status"],
    queryFn: getEngineStatus,
    retry: false,
    staleTime: Number.POSITIVE_INFINITY,
  });
}

export function useLoadEngineMutation() {
  return useMutation({
    mutationKey: ["desktop", "load-engine"],
    mutationFn: (dataDir: string) => loadEngine(dataDir),
    retry: false,
  });
}

export function useDictionaryDefaultsQuery(dataDir: string | undefined) {
  return useQuery({
    queryKey: ["desktop", "dictionary-defaults", dataDir],
    queryFn: fetchDictionaryDefaults,
    enabled: Boolean(dataDir),
    retry: false,
    staleTime: Number.POSITIVE_INFINITY,
  });
}

export function useNameFilterMutation() {
  return useMutation({
    mutationKey: ["desktop", "names", "filter"],
    mutationFn: (request: NameFilterRequest) => filterChapterNames(request),
    retry: false,
  });
}
