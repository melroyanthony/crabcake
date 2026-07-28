import { defineConfig, devices } from "@playwright/test";

import { loadRootEnv } from "./tests/load-env";

loadRootEnv();

const baseURL = process.env.FRONTEND_HOST ?? "http://localhost:3000";

/**
 * End-to-end tests against the Compose stack (`just e2e`).
 *
 * There is no local webServer here on purpose: the BFF, worker and Mailcatcher
 * all have to be up for signup, auth cookies and password recovery to work.
 */
export default defineConfig({
  testDir: "./tests",
  fullyParallel: true,
  forbidOnly: Boolean(process.env.CI),
  retries: process.env.CI ? 2 : 0,
  // One worker avoids Mailcatcher races when several recovery tests run at once.
  workers: 1,
  reporter: process.env.CI ? "github" : "list",
  timeout: 60_000,
  expect: { timeout: 10_000 },
  use: {
    baseURL,
    trace: "on-first-retry",
    screenshot: "only-on-failure",
  },
  projects: [
    { name: "setup", testMatch: /.*\.setup\.ts/ },
    {
      name: "chromium",
      use: {
        ...devices["Desktop Chrome"],
        storageState: "playwright/.auth/user.json",
      },
      dependencies: ["setup"],
      testIgnore: /.*\.setup\.ts/,
    },
  ],
});
