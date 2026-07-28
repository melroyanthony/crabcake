import { defineConfig } from "@hey-api/openapi-ts";

/**
 * Regenerates `src/client/` from the OpenAPI document at the repo root.
 *
 * The browser never talks to Axum directly: the generated client's base URL is
 * the empty string, so every call is same-origin and hits the BFF proxy at
 * `/api/v1/*`, which attaches cookies as Bearer and refreshes on 401.
 */
export default defineConfig({
  input: "../openapi.json",
  output: {
    path: "src/client",
    // The folder is produced entirely by this tool; a stale file from an older
    // generator run should not linger next to the new ones.
    clean: true,
  },
  plugins: [
    "@hey-api/typescript",
    "@hey-api/sdk",
    {
      name: "@hey-api/client-fetch",
      // Relative to the page origin, never an absolute backend URL.
      baseUrl: "",
    },
    "@tanstack/react-query",
    "zod",
  ],
});
