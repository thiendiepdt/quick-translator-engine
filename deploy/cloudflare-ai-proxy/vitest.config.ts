import { cloudflareTest } from "@cloudflare/vitest-pool-workers";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [
    cloudflareTest({
      wrangler: {
        configPath: "./wrangler.jsonc",
      },
      miniflare: {
        bindings: {
          UPSTREAM_BASE_URL: "http://hub.test/v1",
          CORS_ALLOWED_ORIGINS: "https://dich.example.com,http://localhost:5173",
        },
      },
    }),
  ],
});
