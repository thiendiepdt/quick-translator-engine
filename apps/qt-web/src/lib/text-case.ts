const vietnameseLocale = "vi-VN";

export function lowercaseText(value: string): string {
  return value.toLocaleLowerCase(vietnameseLocale);
}

export function uppercaseText(value: string): string {
  return value.toLocaleUpperCase(vietnameseLocale);
}

export function sentenceCaseText(value: string): string {
  return lowercaseText(value).replace(/\p{L}/u, (letter) =>
    letter.toLocaleUpperCase(vietnameseLocale),
  );
}

export function titleCaseText(value: string): string {
  return lowercaseText(value).replace(
    /(^|[^\p{L}\p{N}]+)(\p{L})/gu,
    (_match, prefix: string, letter: string) =>
      `${prefix}${letter.toLocaleUpperCase(vietnameseLocale)}`,
  );
}
