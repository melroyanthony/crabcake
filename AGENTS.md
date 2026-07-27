# AGENTS.md

Guidance for any coding agent or contributor working in this repository. Tool-agnostic:
nothing here is specific to a particular assistant.

## What this repository is

Crabcake is a **`cargo-generate` template**, not an application. The repository root *is* the
template. When someone runs `cargo generate --git .../crabcake`, almost every file here is
copied into their new project.

The practical consequence: **every file is user-facing output.** A `TODO` left in
`backend/src/main.rs` ships to every project generated from this repo forever. Hold these
files to the standard of code you would hand to someone on day one of a new job.

Generated projects contain an Axum + SQLx + PostgreSQL API and a Next.js + TypeScript
frontend whose API client is generated from the backend's OpenAPI spec.

## Layout

| Path                  | Purpose                                                        |
| --------------------- | -------------------------------------------------------------- |
| `cargo-generate.toml` | Placeholders, Liquid `include` whitelist, conditional `ignore`s |
| `pre-script.rhai`     | Runs before expansion; generates secrets                        |
| `post-script.rhai`    | Runs after expansion; renames files, prints next steps          |
| `README.md`           | The template's own landing page, never copied to a project      |
| `README.project.md`   | Becomes the generated project's `README.md`                     |
| `AGENTS.md`, `LICENSE`| Template meta files, never copied to a project                  |
| `backend/`            | Axum API and background worker                                  |
| `frontend/`           | Next.js App Router frontend                                     |
| `.github/workflows/`  | CI shipped **to generated projects**                            |
| `.template/`          | CI for the template itself, never copied to a project           |

## Template rules

These are the constraints that make the template work. Breaking one usually produces a
template that still generates but emits broken projects, so they are easy to miss.

1. **Liquid only runs on the `include` list.** `cargo-generate` would otherwise render every
   file, destroying `{{ ... }}` in TSX and justfiles and `${{ ... }}` in workflows. If a file
   needs a placeholder substituted, add it to `include` in `cargo-generate.toml`. Otherwise
   leave it alone and it is copied byte for byte.
2. **Never add hook scripts to `ignore`.** `cargo-generate` strips them from the output
   itself; ignoring them deletes them before they can run.
3. **`ignore` does not support wildcards.** Only exact file and directory names.
4. **Rhai's `trim()` mutates in place and returns unit.** `system::command(...)` already
   returns trimmed stdout, so never chain `.trim()` onto it — the value silently becomes unit
   and `variable::set` fails.
5. **Verify by generating.** After changing anything in the template layer:

   ```bash
   rm -rf /tmp/crabcake-smoke && mkdir -p /tmp/crabcake-smoke
   cargo generate --path . --name my-app --destination /tmp/crabcake-smoke --silent --allow-commands
   ```

   Then confirm the generated project builds and contains no leftover `{{` or `{%`.

## Commands

Run inside a generated project, or in this repo once `backend/` and `frontend/` exist:

```bash
just             # list every task
just up          # start the full stack in Docker
just test        # backend fmt, clippy, tests; frontend lint, types, build
just client      # regenerate the frontend API client from the backend OpenAPI spec
just prepare     # refresh the SQLx offline query cache after changing SQL
just migrate     # apply database migrations
```

## Code conventions

- **Rust**: `cargo fmt` and `cargo clippy -- -D warnings` must both pass. Handlers return
  `Result<T, AppError>`; errors are converted centrally rather than at each call site.
- **TypeScript**: Biome for lint and format, not ESLint — `typescript-eslint` does not support
  TypeScript 7. Never hand-edit `frontend/src/client/`; it is generated and will be
  overwritten by `just client`.
- **SQL**: every schema change is a migration in `backend/migrations/`. Run `just prepare`
  afterwards or CI and Docker builds will fail on the stale offline query cache.
- **Comments** explain constraints and intent, never what the next line does.

## Commit messages

Conventional Commits with a leading gitmoji, in the style of tiangolo's repositories:

```
<emoji> <type>(<scope>): <imperative, lower-case subject>
```

- Emoji first, then the Conventional Commits header.
- Imperative mood ("add", not "added"), lower case, no trailing period, under 72 characters.
- Body is optional and explains the why, not the what.
- Breaking changes take a `!`, e.g. `💥 feat(backend)!: replace jwt claim layout`.
- **Never add `Co-authored-by` trailers, "Generated with" footers, or any other tool
  attribution.** Commits carry the author's name only.

| Type       | Emoji | Use for                                      |
| ---------- | ----- | -------------------------------------------- |
| `feat`     | ✨    | New capability                               |
| `fix`      | 🐛    | Bug fix                                      |
| `refactor` | ♻️    | Behaviour-preserving restructuring           |
| `perf`     | ⚡️    | Performance work                             |
| `docs`     | 📝    | Documentation                                |
| `test`     | ✅    | Tests                                        |
| `ci`       | 👷    | GitHub Actions and other CI config           |
| `build`    | 📦    | Dockerfiles, Compose, build pipeline         |
| `chore`    | 🔧    | Tooling and config that is none of the above |
| `deps`     | ⬆️    | Dependency bumps                             |
| `style`    | 🎨    | Formatting only                              |
| `revert`   | ⏪️    | Reverts                                      |
| `security` | 🔒️    | Security hardening                           |

Other emoji are fine where they describe the change better, keeping the same type prefix:
🔥 removing code, 🚚 moving or renaming, 🚨 fixing warnings, 💚 fixing CI, 🎉 initial commit.

Scopes: `template`, `backend`, `frontend`, `docker`, `ci`, `docs`, `deps`. Omit the scope when
a change is genuinely repo-wide.

Examples:

```
✨ feat(backend): add argon2 password hashing and jwt issuance
🐛 fix(frontend): refresh the access token before retrying a 401
♻️ refactor(backend): extract sqlx queries into the repo layer
👷 ci(template): build the generated project on every push
⬆️ deps(backend): bump axum to 0.8.9
```
