# {{project_title}}

Generated from [Crabcake](https://github.com/melroyanthony/crabcake). An Axum + SQLx +
PostgreSQL API with a Next.js + TypeScript frontend whose API client is generated from the
backend's OpenAPI spec.

## Requirements

- [Rust](https://rustup.rs) 1.94 or newer
- [Docker](https://docs.docker.com/get-docker/) with Compose
- [Node.js](https://nodejs.org) 22 or newer
- [just](https://github.com/casey/just) and [sqlx-cli](https://crates.io/crates/sqlx-cli):
  `cargo install just sqlx-cli`

## Getting started

```bash
just up
```

| Service      | URL                            |
| ------------ | ------------------------------ |
| Frontend     | http://localhost:3000          |
| API          | http://localhost:8000          |
| API docs     | http://localhost:8000/docs     |
| Mailcatcher  | http://localhost:1080          |
{% if enable_s3 %}| MinIO console | http://localhost:9001         |
{% endif %}
Sign in with `{{first_superuser}}` and the superuser password from `.env`.

Run `just` to see every available task.

## Development

The API and the frontend can each be run outside Docker against the Compose database:

```bash
just dev-backend    # cargo watch on the api
just dev-frontend   # next dev
```

After changing an API route or schema, regenerate the typed frontend client. The frontend
cannot drift from the API, because everything under `frontend/src/client/` is generated:

```bash
just client
```

After changing any SQL, refresh the offline query cache so CI and Docker builds keep working:

```bash
just prepare
```

See [docs/development.md](docs/development.md) for details.

## Testing

```bash
just test    # backend fmt, clippy and tests, then frontend lint, types and build
just e2e     # Playwright against the Compose stack
```

Backend tests use `#[sqlx::test]`, so each test gets its own isolated database and they run
in parallel without interfering.

## Deployment

`docker compose` with Traefik for TLS termination, driven by the GitHub Actions workflows in
`.github/workflows/`. See [docs/deployment.md](docs/deployment.md).

## Security

`.env` was generated with fresh secrets. Rotate `SECRET_KEY` and `POSTGRES_PASSWORD` for any
real deployment and supply them through your platform's secret store rather than a committed
file.

## License

{{license}}
