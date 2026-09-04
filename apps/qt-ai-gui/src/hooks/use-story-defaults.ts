import { useEffect, useState } from "react";
import { toast } from "sonner";

import { storyDefaults } from "@/lib/api";
import type { StoryDefaults } from "@/lib/types";

let cached: Promise<StoryDefaults> | undefined;

/** Prompt gốc + rule mặc định không đổi trong một phiên app — tải một lần, dùng chung. */
export function useStoryDefaults(): StoryDefaults | undefined {
  const [defaults, setDefaults] = useState<StoryDefaults | undefined>();
  useEffect(() => {
    let cancelled = false;
    cached ??= storyDefaults();
    cached
      .then((value) => {
        if (!cancelled) setDefaults(value);
      })
      .catch((error: unknown) => {
        cached = undefined;
        toast.error(error instanceof Error ? error.message : "Không đọc được prompt mặc định");
      });
    return () => {
      cancelled = true;
    };
  }, []);
  return defaults;
}
