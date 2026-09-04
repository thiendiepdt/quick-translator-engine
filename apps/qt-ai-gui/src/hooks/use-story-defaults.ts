import { useEffect, useState } from "react";
import { toast } from "sonner";

import { storyDefaults } from "@/lib/api";
import type { StoryDefaults, StoryGenre } from "@/lib/types";

const cache = new Map<string, Promise<StoryDefaults>>();

/** Prompt gốc + rule mặc định theo genre — không đổi trong một phiên app, tải một lần mỗi tổ hợp. */
export function useStoryDefaults(genre: StoryGenre): StoryDefaults | undefined {
  const key = `${genre.setting}/${genre.names}`;
  const [state, setState] = useState<{ key: string; value: StoryDefaults } | undefined>();
  useEffect(() => {
    let cancelled = false;
    let pending = cache.get(key);
    if (!pending) {
      pending = storyDefaults(genre);
      cache.set(key, pending);
    }
    pending
      .then((value) => {
        if (!cancelled) setState({ key, value });
      })
      .catch((error: unknown) => {
        cache.delete(key);
        toast.error(error instanceof Error ? error.message : "Không đọc được prompt mặc định");
      });
    return () => {
      cancelled = true;
    };
  }, [key, genre]);
  return state?.key === key ? state.value : undefined;
}
