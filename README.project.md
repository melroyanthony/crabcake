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
- ⚙️ Background jobs on a Postgres-backed [apalis](https://apalis.dev) queue
{% if enable_otel %}- 📈 OpenTelemetry tracing and Prometheus metrics
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

## 📖 The API Documentation

The OpenAPI document is built from the routers themselves, so a route cannot be added to the
API and forgotten in the docs.

| What                        | Where                                    |
| --------------------------- | ---------------------------------------- |
| 📖 Interactive reference     | http://localhost:8000/docs               |
| 📄 The document             | http://localhost:8000/api/openapi.json   |

To write it to a file without a database or a running server:

```bash
just spec    # writes openapi.json
```

That is what `just client` runs first, which is why regenerating the frontend client works
from a plain checkout.

## 📬 Email and Background Jobs

Email goes out through a Postgres-backed queue rather than during the request, so a mail server
having a bad minute delays a message instead of failing somebody's password reset. Failed sends
are retried.

The queue is drained by a separate `worker` process, which Compose runs for you. Locally every
message is caught by Mailcatcher instead of being delivered, so nothing escapes:

```bash
just mail    # opens http://localhost:1080
```

Email is optional. With no `SMTP_HOST` set, messages are logged instead of sent, so a fresh
checkout runs before you have thought about mail at all.

{% if enable_s3 %}## 📦 File Uploads

Files never travel through the API. A client asks `POST /api/v1/uploads` what to do with a file
and gets back a signed URL to `PUT` it to, then uploads straight to storage:

```bash
# 1. ask where to put it
curl -X POST localhost:8000/api/v1/uploads -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"filename": "photo.jpg", "content_type": "image/jpeg"}'

# 2. send the file to the url that came back, then keep the key
curl -X PUT --upload-file photo.jpg -H 'Content-Type: image/jpeg' "$URL"
```

This keeps large files out of the API's memory, out of its request timeout and out of its body
limit, and it means putting a CDN in front later changes nothing on the server. Store the `key`
against your own records; `POST /api/v1/uploads/link` turns it back into a link when you need to
read the file.

Every key lives under a prefix belonging to the account that made it, so somebody else's key is
a 404 rather than a way to read their files. Locally the bucket is MinIO. Clearing `S3_BUCKET`
switches uploads off, and the endpoints then answer `501` instead of failing.

{% endif %}## ✅ Testing

```bash
just test    # backend fmt, clippy and tests, then frontend lint, types and build
just e2e     # Playwright against the running stack
```

Backend tests get their own isolated database each, created and dropped around the test, so
they run in parallel without interfering with one another. That does mean the database has to be
up: run `just up` first, or expect `DATABASE_URL must be set`.

## 🚢 Deployment

Docker Compose with Traefik handling TLS, driven by the workflows in `.github/workflows/`.
See [docs/deployment.md](docs/deployment.md).

## 🔒 Security

Your `.env` was generated with fresh secrets. Before deploying:

- Rotate `SECRET_KEY`, `POSTGRES_PASSWORD`{% if enable_s3 %} and `S3_SECRET_ACCESS_KEY`{% endif %}, and supply them from your platform's
  secret store rather than a committed file. Generate new ones with `just secrets`.{% if enable_s3 %}
- On AWS, leave `S3_ACCESS_KEY_ID` and `S3_SECRET_ACCESS_KEY` empty and give the task an
  instance role instead. With no keys set, the usual AWS credential chain is used, so nothing
  long-lived has to exist.{% endif %}
- Change `FIRST_SUPERUSER_PASSWORD` from whatever you set at creation time.

The API refuses to start outside `local` while any of these is still `changethis`.

## 📄 License

{{license}}
