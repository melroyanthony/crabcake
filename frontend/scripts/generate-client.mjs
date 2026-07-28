// Regenerates `src/client/` from the OpenAPI document at the repo root.
// Wired up properly once the hey-api client milestone lands; until then this
// exists so `just client` has a single entry point rather than a missing script.
import { existsSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const spec = resolve(root, "openapi.json");

if (!existsSync(spec)) {
  console.error(
    "openapi.json is missing. Run `just spec` from the repository root first.",
  );
  process.exit(1);
}

console.error(
  "API client generation is not wired up yet. It arrives with the frontend-client milestone.",
);
process.exit(1);
