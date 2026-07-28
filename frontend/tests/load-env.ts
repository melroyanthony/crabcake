import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

/**
 * Loads the repository-root `.env` into `process.env` without adding a dotenv
 * dependency. `just e2e` already injects these via dotenv-load; this covers
 * `npx playwright test` run directly from `frontend/`.
 */
export function loadRootEnv() {
  const path = resolve(process.cwd(), "../.env");
  if (!existsSync(path)) {
    return;
  }

  for (const line of readFileSync(path, "utf8").split("\n")) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("#")) {
      continue;
    }
    const eq = trimmed.indexOf("=");
    if (eq === -1) {
      continue;
    }
    const key = trimmed.slice(0, eq);
    if (process.env[key] !== undefined) {
      continue;
    }
    let value = trimmed.slice(eq + 1);
    if (
      (value.startsWith('"') && value.endsWith('"')) ||
      (value.startsWith("'") && value.endsWith("'"))
    ) {
      value = value.slice(1, -1);
    }
    process.env[key] = value;
  }

  if (!process.env.MAILCATCHER_HOST) {
    process.env.MAILCATCHER_HOST = "http://localhost:1080";
  }
}
