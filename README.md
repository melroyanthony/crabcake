# Crabcake

Fully-baked full-stack Rust. A `cargo-generate` template that scaffolds a production-ready
monorepo: an Axum + SQLx + PostgreSQL API and a Next.js + TypeScript dashboard whose API
client is generated from the backend's own OpenAPI spec.

Inspired by [full-stack-fastapi-template](https://github.com/fastapi/full-stack-fastapi-template),
rebuilt on Rust.

## Usage

```bash
cargo install cargo-generate
cargo generate --git https://github.com/melroyanthony/crabcake
```

You will be prompted for a project name, the first superuser, SMTP details, and which optional
subsystems to include. Then:

```bash
cd my-app
just up
```

## What you get

### Backend

- [Axum](https://github.com/tokio-rs/axum) with [SQLx](https://github.com/launchbadge/sqlx)
  and PostgreSQL — compile-time checked queries, no ORM indirection
- JWT authentication with Argon2id password hashing and rotating refresh tokens
- User and item CRUD, superuser-only admin endpoints, signup, password recovery by email
- OpenAPI 3.1 generated from the handlers via [utoipa](https://github.com/juhaku/utoipa),
  served with a Scalar docs UI
- SQL migrations, plus per-test isolated databases through `#[sqlx::test]`
- Background jobs on a Postgres-backed [apalis](https://apalis.dev) queue with a separate
  worker binary
- Structured tracing with OpenTelemetry export, Prometheus metrics, and per-IP rate limiting
- S3-compatible file uploads (MinIO locally)

### Frontend

- [Next.js](https://nextjs.org) App Router, React, Tailwind CSS and
  [shadcn/ui](https://ui.shadcn.com)
- TypeScript 7, the Go-native compiler
- A fully typed API client and TanStack Query hooks generated from the backend spec by
  [Hey API](https://heyapi.dev) — the frontend cannot drift from the API
- Tokens live in httpOnly cookies; the browser talks to same-origin Next.js route handlers
  that proxy to the API, so there is no CORS surface and no token in JavaScript
- Dark mode, forms with react-hook-form and Zod, Biome for lint and format

### Operations

- Docker Compose for local development, with Mailcatcher, MinIO and an OTel collector
- Traefik configuration for production with automatic HTTPS
- GitHub Actions for backend tests, frontend checks, Playwright end-to-end tests, and
  deployment to staging and production

## Documentation

- [docs/development.md](docs/development.md) — running the stack locally
- [docs/deployment.md](docs/deployment.md) — deploying with Docker Compose and Traefik
- [AGENTS.md](AGENTS.md) — repository conventions

## License

[MIT](LICENSE)
