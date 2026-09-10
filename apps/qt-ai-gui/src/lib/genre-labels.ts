import { GENRE_NAMES, GENRE_SETTINGS } from "@/lib/schema";
import { GENRE_NAMES_LABELS, GENRE_SETTING_LABELS, type GenreNames, type GenreSetting } from "@/lib/types";

/** Nhãn ngắn cho ô chọn radio (không kèm hint). */
export const SETTING_CHOICE_LABELS = Object.fromEntries(
  GENRE_SETTINGS.map((s) => [s, GENRE_SETTING_LABELS[s].label]),
) as Record<GenreSetting, string>;
export const NAMES_CHOICE_LABELS = Object.fromEntries(
  GENRE_NAMES.map((n) => [n, GENRE_NAMES_LABELS[n].label]),
) as Record<GenreNames, string>;
