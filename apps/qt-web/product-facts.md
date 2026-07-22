# Product facts used by the design demos

Verified from the repository source and documentation on 2026-07-22.

- Product name: Quick Translator Engine.
- The web client is provisionally named `qt-web`.
- The engine translates Chinese text to Vietnamese through dictionary-based rules.
- This web surface only exposes `vietphrase-one`.
- The HTTP call is `POST /translate` with UTF-8 JSON.
- The UI must request `ranges: true`; `sourceRanges[i]` maps to `targetRanges[i]`.
- Range offsets and lengths use UTF-16 code units, so JavaScript can slice them directly.
- Fixed dictionaries are VietPhrase and ChinesePhienAmWords.
- Request-scoped dictionaries are `names`, `names2`, `luatNhan`, `pronouns`,
  `danhTu`, `hoNguoi`, `hauTu`, and `ignoredChinesePhrases`.
- Engine options exposed by the API are `wrap`, `pretty`, `scanRange`,
  `translationAlgorithm`, and `prioritizedName`.
- The Cloudflare gateway forwards `POST /translate` to an AWS Lambda Function URL.
- The current API/gateway does not yet document browser CORS support. Production
  `qt-web` therefore needs either same-origin proxying or an explicit CORS change.
