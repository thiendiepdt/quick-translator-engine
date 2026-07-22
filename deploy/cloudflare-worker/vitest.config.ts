import { cloudflareTest } from "@cloudflare/vitest-pool-workers";
import { defineConfig } from "vitest/config";

// Required-secret validation runs before Miniflare applies test bindings.
// These deliberately fake values keep tests offline and silence that warning.
process.env.AWS_ACCESS_KEY_ID ??= "AKIDEXAMPLE";
process.env.AWS_SECRET_ACCESS_KEY ??= "test-secret-access-key";

export default defineConfig({
  plugins: [
    cloudflareTest({
      wrangler: {
        configPath: "./wrangler.jsonc",
      },
      miniflare: {
        bindings: {
          LAMBDA_FUNCTION_URL:
            "https://test-function.lambda-url.ap-southeast-1.on.aws/",
          AWS_REGION: "ap-southeast-1",
          AWS_ACCESS_KEY_ID: "AKIDEXAMPLE",
          AWS_SECRET_ACCESS_KEY: "test-secret-access-key",
        },
      },
    }),
  ],
});
