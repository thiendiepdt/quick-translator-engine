# Provisional brand spec for qt-web design exploration

No logo, design system, brand colour, UI kit, screenshot, or preferred reference was
present in the repository when these directions were made.

## Assets

- Official logo: not available.
- Product imagery: not required; this is a text-production tool and imagery would be
  decorative rather than informative.
- Existing UI screenshots: not available.
- Wordmark used in demos: plain text `quick translator / engine`, explicitly provisional.

## Shared product character

- Precise and fast, but not visually cold.
- Designed for long-form chapter translation on a laptop.
- Chinese source and Vietnamese output must remain the visual centre.
- Configuration should be discoverable without permanently crowding the reading area.
- Phrase mapping is the signature interaction: selecting either side reveals its pair.

## Constraints

- Do not imply machine-learning or conversational-AI features that the engine does not have.
- Do not invent usage metrics, confidence scores, or translation quality scores.
- Do not present VietPhrase or ChinesePhienAmWords as user-editable dictionaries.
- Do not persist API secrets in the browser. The intended endpoint is the public
  Cloudflare gateway; Lambda credentials remain in Workers Secrets.
- The three demos may use separate palettes while the visual identity is still undecided.
