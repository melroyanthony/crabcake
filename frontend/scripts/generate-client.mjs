// Regenerates `src/client/` from the OpenAPI document at the repo root.
//
// @hey-api/openapi-ts still loads the TypeScript JavaScript compiler API, which
// TypeScript 7 no longer ships. A typescript@5 tarball is unpacked under the
// generator's own node_modules so its `import "typescript"` resolves there
// instead of to the app's TypeScript 7.
import { spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const frontend = resolve(here, "..");
const root = resolve(frontend, "..");
const spec = resolve(root, "openapi.json");
const generator = resolve(frontend, "node_modules/@hey-api/openapi-ts");
const nestedRoot = resolve(generator, "node_modules");
const nestedTypescript = resolve(nestedRoot, "typescript/package.json");

if (!existsSync(spec)) {
  console.error(
    "openapi.json is missing. Run `just spec` from the repository root first.",
  );
  process.exit(1);
}

if (!existsSync(generator)) {
  console.error(
    "@hey-api/openapi-ts is not installed. Run `npm install` in frontend/.",
  );
  process.exit(1);
}

if (!existsSync(nestedTypescript)) {
  console.log(
    "Installing typescript@5 under @hey-api/openapi-ts for code generation…",
  );
  ensureTypescript5(nestedRoot);
}

const result = spawnSync(
  "npx",
  ["@hey-api/openapi-ts", "-f", "openapi-ts.config.ts"],
  {
    cwd: frontend,
    stdio: "inherit",
    shell: process.platform === "win32",
  },
);

if (result.status === 0) {
  writeFileSync(
    resolve(frontend, "src/client/README.md"),
    `# Generated API client

This directory is produced by \`just client\` / \`npm run generate-client\` from
the backend OpenAPI document. Do not edit files here by hand — they will be
overwritten.

Use the SDK and TanStack Query helpers through \`@/lib/api\` and
\`@/client/@tanstack/react-query.gen\`. The client's base URL is the empty
string so requests stay same-origin and hit the BFF proxy.
`,
  );
}

process.exit(result.status ?? 1);

function ensureTypescript5(destination) {
  const staging = mkdtempSync(join(tmpdir(), "crabcake-ts5-"));

  try {
    const pack = spawnSync("npm", ["pack", "typescript@5.9.3", "--silent"], {
      cwd: staging,
      encoding: "utf8",
      shell: process.platform === "win32",
    });

    if (pack.status !== 0) {
      console.error(pack.stderr);
      process.exit(pack.status ?? 1);
    }

    const tarball = pack.stdout.trim().split("\n").at(-1);
    if (!tarball) {
      console.error("npm pack did not produce a tarball");
      process.exit(1);
    }

    const extract = spawnSync(
      "tar",
      ["-xzf", resolve(staging, tarball), "-C", staging],
      { stdio: "inherit" },
    );
    if (extract.status !== 0) {
      process.exit(extract.status ?? 1);
    }

    mkdirSync(destination, { recursive: true });
    const target = resolve(destination, "typescript");
    rmSync(target, { recursive: true, force: true });
    renameSync(resolve(staging, "package"), target);

    const version = JSON.parse(
      readFileSync(resolve(target, "package.json"), "utf8"),
    ).version;
    console.log(`typescript@${version} ready for code generation`);
  } finally {
    rmSync(staging, { recursive: true, force: true });
  }
}
