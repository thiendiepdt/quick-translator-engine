import { useMutation, useQuery } from "@tanstack/react-query";

import {
  checkHealth,
  fetchDictionaryDefaults,
  filterChapterNames,
  translateChapter,
} from "@/lib/api";
import { endpointSchema } from "@/lib/schema";
import type { NameFilterRequest, TranslationRequest } from "@/lib/types";

export function useTranslationMutation() {
  return useMutation({
    mutationKey: ["translate", "vietphrase-one"],
    mutationFn: ({ endpoint, request }: { endpoint: string; request: TranslationRequest }) =>
      translateChapter(endpoint, request),
    retry: false,
  });
}

export function useHealthQuery(endpoint: string) {
  return useQuery({
    queryKey: ["health", endpoint],
    queryFn: () => checkHealth(endpoint),
    enabled: false,
    retry: false,
    staleTime: 30_000,
  });
}

export function useDictionaryDefaultsQuery(endpoint: string) {
  const normalizedEndpoint = endpoint.trim();
  return useQuery({
    queryKey: ["dictionary-defaults", normalizedEndpoint],
    queryFn: () => fetchDictionaryDefaults(normalizedEndpoint),
    enabled: endpointSchema.safeParse(normalizedEndpoint).success,
    retry: false,
    staleTime: Number.POSITIVE_INFINITY,
  });
}

export function useNameFilterMutation() {
  return useMutation({
    mutationKey: ["names", "filter"],
    mutationFn: ({ endpoint, request }: { endpoint: string; request: NameFilterRequest }) =>
      filterChapterNames(endpoint, request),
    retry: false,
  });
}
