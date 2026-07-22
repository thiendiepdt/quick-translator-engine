# qt-web — shared design specification for three directions

## Understanding

`qt-web` is not a marketing page and not a generic two-box translator. It is a focused
working surface for readers and translators who process an entire Chinese web-novel
chapter at a time, inspect the Vietnamese VietPhrase one-meaning result, and repeatedly
tune request-scoped dictionaries for character names, pronouns and rule-based phrases.
The product must feel comfortable for a long session: source and output need generous
line height, the primary action must stay obvious, and advanced controls must remain one
gesture away without taking visual priority from the text. A successful design makes the
source-to-output relation tangible. Clicking a source phrase highlights the exact target
phrase, and clicking the translation performs the same lookup in reverse. The range
interaction is therefore not a small extra feature; it is the product-specific visual
motif that distinguishes this tool from a plain textarea form.

The first design round targets a 1440 × 900 desktop viewport and must remain usable down
to tablet width. The intended user sits around one metre from a laptop screen, pastes a
chapter, selects or uploads custom text dictionaries, adjusts a small set of engine
options, sends one request through a Cloudflare gateway to AWS Lambda, and then reads or
copies the result. All three directions use the same real product labels and the same
API contract so visual comparison is honest. The prototype will simulate translation
when its endpoint is an example URL and will use a real `fetch` call when the user enters
an actual endpoint.

## Required content and behaviour

1. A configurable Cloudflare API endpoint ending at the public gateway.
2. A fixed, visible `vietphrase-one` mode; no selector for other engine modes.
3. Source text editing and a Vietnamese output surface.
4. A clear Raw/Output or Source/Output view model.
5. Bidirectional phrase highlighting based on paired UTF-16 ranges.
6. A translated-output view and a raw JSON response view.
7. Engine controls for Pretty, Wrap, Prioritized Name, Scan Range and Translation
   Algorithm. Range mapping is always enabled and explained instead of exposed as an
   off switch.
8. All eight request-scoped dictionary groups: Names, Names2, Luật Nhân, Pronouns,
   Danh Từ, Họ Người, Hậu Từ and Ignored Chinese Phrases.
9. Per-dictionary text editing plus `.txt` file import. Empty content is a deliberate
   replacement with an empty dictionary; an untouched dictionary is omitted.
10. A primary Translate action, request status, copy output action, character counts and
    an error state.

## Interaction assumptions

- The default endpoint is a non-live example so opening a demo never sends data.
- The demos begin with a realistic sample and paired segments, making click mapping
  immediately testable without a backend.
- Changing the source returns the user to edit mode and clears stale pairing until the
  next translation.
- Endpoint and dictionary content are prototype state only. Production decisions about
  local persistence, authentication and content privacy happen after direction approval.
- No decorative photography is required because removing it would not reduce product
  comprehension.

## Form derivation

- Narrative role: an operational workspace, with the current chapter as the hero.
- Viewing distance: laptop at roughly one metre; text is 16–20 px with comfortable line
  height, labels are never below 12 px.
- Temperature: precise and calm, with one direction intentionally more playful.
- Capacity: the 900 px frame must show the translation workspace, core status and a
  meaningful portion of configuration without hiding the primary action.
- Product-specific motif: paired source/target fragments form a visible “translation
  bridge”. Hover and click should make that bridge feel immediate even without drawing
  literal connector lines.

## Three independent directions

### A — Friendly Workshop

Roulette result: second `23`, so `23 % 20 + 1 = 4`, mapping to Friendly Geometric Candy.
It treats dictionaries as a friendly set of cartridges and uses firm pressable controls,
warm paper neutrals, jade green, lacquer red and a small blue accent. The layout is a
three-zone workbench: dictionary rail, source work surface and output work surface.

### B — Precision Split

Reality benchmark: DeepL Translator’s current official web surface confirms the familiar
source/target split and dictionary/glossary workflow. This direction migrates that
immediate mental model into a denser professional tool: a slim navigation rail, a single
hairline-defined split editor, and a collapsible inspector below the text.

### C — Reader’s Desk

Designer lens: Information Architects/iA Writer combined with disciplined Japanese
editorial hierarchy. The translated Vietnamese text becomes the central reading canvas;
the Chinese raw text sits in a numbered manuscript margin and configuration lives in a
right-side folio. It is the most distinctive direction and optimises review rather than
input symmetry.
