/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_QT_API_URL?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
