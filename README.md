<div align="center">

# 🦀 Crabcake

**Fully-baked full-stack Rust.**

A project template that gives you a production-ready Rust API and a modern TypeScript
dashboard, wired together and ready to deploy.

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.94+-CE422B.svg?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Next.js](https://img.shields.io/badge/Next.js-16-000000.svg?logo=nextdotjs&logoColor=white)](https://nextjs.org)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-18-4169E1.svg?logo=postgresql&logoColor=white)](https://www.postgresql.org)

</div>

## ⚡ Technology Stack and Features

- 🦀 [**Axum**](https://github.com/tokio-rs/axum) for the Rust backend API.
  - 🗄️ [SQLx](https://github.com/launchbadge/sqlx) for database access, with queries checked
    against your real schema **at compile time**.
  - 💾 [PostgreSQL](https://www.postgresql.org) as the SQL database.
  - 📖 [utoipa](https://github.com/juhaku/utoipa) generates the OpenAPI spec from your
    handlers, so the docs can never go stale.
- 🚀 [**Next.js**](https://nextjs.org) for the frontend.
  - 💃 React, the App Router, and [TypeScript 7](https://devblogs.microsoft.com/typescript/announcing-typescript-7-0/).
  - 🎨 [Tailwind CSS](https://tailwindcss.com) and [shadcn/ui](https://ui.shadcn.com) for the
    components.
  - 🤖 An automatically generated frontend client, so the UI can never drift from the API.
  - 🧪 [Playwright](https://playwright.dev) for end-to-end testing.
  - 🦇 Dark mode support.
- 🔑 JWT authentication with rotating refresh tokens.
- 🔒 Secure password hashing with Argon2id by default.
- 🍪 Tokens live in httpOnly cookies, never in JavaScript.
- 📫 Email based password recovery.
- 📬 [Mailcatcher](https://mailcatcher.me) for local email testing during development.
- ⚙️ Background jobs on a Postgres-backed [apalis](https://apalis.dev) queue.
- 📈 [OpenTelemetry](https://opentelemetry.io) tracing and Prometheus metrics.
- 🚦 Per-IP rate limiting.
- 📦 S3-compatible file uploads, with [MinIO](https://min.io) locally.
- ✅ Tests with isolated databases, so they run in parallel without stepping on each other.
- 🐋 [Docker Compose](https://www.docker.com) for development and production.
- 📞 [Traefik](https://traefik.io) as a reverse proxy, with automatic HTTPS.
- 🏭 CI and CD based on GitHub Actions.

## 🚀 How To Use It

### Requirements

- [Rust](https://rustup.rs) 1.94 or newer
- [Docker](https://docs.docker.com/get-docker/) with Compose
- [Node.js](https://nodejs.org) 22 or newer

### Generate a Project

Install [cargo-generate](https://cargo-generate.github.io/cargo-generate/) once:

```bash
cargo install cargo-generate
```

Then create your project:

```bash
cargo generate --git https://github.com/melroyanthony/crabcake
```

You will be asked a handful of questions, and then:

```bash
cd my-awesome-project
just up
```

✨ That's it. Your API, worker, database and frontend are running. ✨

| What                        | Where                          |
| --------------------------- | ------------------------------ |
| 🖥️ Frontend                  | http://localhost:3000          |
| 🦀 API                       | http://localhost:8000          |
| 📖 API docs                  | http://localhost:8000/docs     |
| 📬 Mailcatcher               | http://localhost:1080          |

### Input Variables

Crabcake asks you for these when generating. Every one of them can also be changed later in
`.env`, so don't overthink it.

| Variable                   | Default             | What it is                                          |
| -------------------------- | ------------------- | --------------------------------------------------- |
| `project_title`            | `Crabcake App`      | Human-readable name, shown to API users              |
| `domain`                   | `example.com`       | Production domain, used for Traefik routing          |
| `first_superuser`          | `admin@example.com` | Email of the first admin account                     |
| `first_superuser_password` | `changethis`        | Password for that account                            |
| `emails_from_email`        | `info@example.com`  | Address outgoing email is sent from                  |
| `smtp_host` / `_user` / `_password` | empty      | Your mail provider, set it later if you prefer       |
| `sentry_dsn`               | empty               | Sentry DSN, leave empty to disable                   |
| `license`                  | `MIT`               | License for your project                             |
| `enable_traefik`           | yes                 | Traefik reverse proxy config for production          |
| `enable_jobs`              | yes                 | Background job worker                                |
| `enable_otel`              | yes                 | OpenTelemetry tracing and Prometheus metrics         |
| `enable_s3`                | yes                 | S3-compatible file uploads                           |

Answering **no** to any of the last four leaves those files out of your project entirely, so
you don't carry code you never asked for.

### 🔑 Secret Keys

`SECRET_KEY` and `POSTGRES_PASSWORD` are generated for you at creation time, so your project
never starts life with a password someone typed by hand.

If `openssl` wasn't available, they fall back to `changethis` and you'll see a warning. Get
fresh ones any time with:

```bash
just secrets
```

Pass real secrets as environment variables from your platform's secret store rather than a
committed file. The API refuses to start outside `local` if `SECRET_KEY` is still
`changethis`.

## 📚 Documentation

- 🛠️ [Development](docs/development.md) — running the stack, common tasks, project layout
- 🚢 [Deployment](docs/deployment.md) — Docker Compose, Traefik and automatic HTTPS
- 🦀 [Backend](backend/README.md) — architecture, migrations, testing
- 🖥️ [Frontend](frontend/README.md) — routing, the generated client, components
- 🤖 [AGENTS.md](AGENTS.md) — conventions for contributors and coding agents

## 🧑‍🍳 Working On the Template Itself

This repository is a real, compiling project, not a pile of placeholders. The Rust crate is
always named `app` and the frontend package `frontend`, so you can `cargo test` and
`npm run build` here directly.

Only a few files contain placeholders. After changing anything in the template layer, check
that a real project still comes out the other end:

```bash
cargo generate --path . --name my-app --destination /tmp/smoke --silent --allow-commands
```

See [AGENTS.md](AGENTS.md) for the rules that keep generation working.

## 📄 License

Crabcake is licensed under the terms of the [MIT license](LICENSE).

Inspired by [full-stack-fastapi-template](https://github.com/fastapi/full-stack-fastapi-template).
