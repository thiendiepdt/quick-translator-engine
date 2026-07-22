import { useMutation, useQuery } from "@tanstack/react-query";

import { checkHealth, translateChapter } from "@/lib/api";
import type { TranslationRequest } from "@/lib/types";

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
