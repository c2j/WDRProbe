/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly DEV: boolean;
  readonly PROD: boolean;
  readonly MODE: string;
  // Add any custom env variables here
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
