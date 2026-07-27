# {{project_title}}

A full-stack application with a Rust API and a TypeScript frontend, generated from
[🦀 Crabcake](https://github.com/melroyanthony/crabcake).

## ⚡ Stack

- 🦀 [Axum](https://github.com/tokio-rs/axum) and [SQLx](https://github.com/launchbadge/sqlx)
  on [PostgreSQL](https://www.postgresql.org), with queries checked at compile time
- 🚀 [Next.js](https://nextjs.org), React and [TypeScript](https://www.typescriptlang.org),
  styled with [Tailwind CSS](https://tailwindcss.com) and [shadcn/ui](https://ui.shadcn.com)
- 🤖 A frontend API client generated from the backend's OpenAPI spec
- 🔑 JWT authentication with Argon2id password hashing and httpOnly cookies
{% if enable_jobs %}- ⚙️ Background jobs on a Postgres-backed [apalis](https://apalis.dev) queue
{% endif %}{% if enable_otel %}- 📈 OpenTelemetry tracing and Prometheus metrics
{% endif %}{% if enable_s3 %}- 📦 S3-compatible file uploads
{% endif %}- 🐋 Docker Compose for development and production

## 🚀 Get Started

### Requirements

- [Rust](https://rustup.rs) 1.94 or newer
- [Docker](https://docs.docker.com/get-docker/) with Compose
- [Node.js](https://nodejs.org) 22 or newer
- `cargo install just sqlx-cli`

### Run It

```bash
just up
```

| What                        | Where                          |
| --------------------------- | ------------------------------ |
| 🖥️ Frontend                  | http://localhost:3000          |
| 🦀 API                       | http://localhost:8000          |
| 📖 API docs                  | http://localhost:8000/docs     |
| 📬 Mailcatcher               | http://localhost:1080          |
{% if enable_s3 %}| 📦 MinIO console             | http://localhost:9001          |
{% endif %}
Sign in with **{{first_superuser}}** and the password in your `.env`.

Run `just` on its own to see every available task.

## 🛠️ Development

Run either half outside Docker, against the Compose database:

```bash
just dev-backend    # the API, with auto-reload
just dev-frontend   # the Next.js dev server
```

Two habits worth forming:

- 🤖 **After changing an API route or schema**, run `just client`. Everything under
  `frontend/src/client/` is generated, so the frontend can't drift from the API — but only if
  you regenerate it.
- 🗄️ **After changing any SQL**, run `just prepare`. SQLx checks your queries at compile time
  using a cached copy of the schema, and CI and Docker builds fail on a stale cache.

More in [docs/development.md](docs/development.md).

## ✅ Testing

```bash
just test    # backend fmt, clippy and tests, then frontend lint, types and build
just e2e     # Playwright against the running stack
```

Backend tests get their own isolated database each, so they run in parallel without
interfering with one another.

## 🚢 Deployment

Docker Compose with Traefik handling TLS, driven by the workflows in `.github/workflows/`.
See [docs/deployment.md](docs/deployment.md).

## 🔒 Security

Your `.env` was generated with fresh secrets. Before deploying:

- Rotate `SECRET_KEY` and `POSTGRES_PASSWORD`, and supply them from your platform's secret
  store rather than a committed file. Generate new ones with `just secrets`.
- Change `FIRST_SUPERUSER_PASSWORD` from whatever you set at creation time.

The API refuses to start outside `local` while any of these is still `changethis`.

## 📄 License

{{license}}
