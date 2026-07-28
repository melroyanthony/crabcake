# Development

## Docker Compose

Start the local stack:

```bash
just up
```

That runs `docker compose up -d --wait`. Compose loads `compose.yml` together with
`compose.override.yml`, which publishes ports, adds Mailcatcher (and MinIO / the OpenTelemetry
collector when those extras were enabled at generation time), and turns the Traefik public
network into a local one so you do not need a reverse proxy on your laptop.

Open these in a browser:

| What | URL |
| ---- | --- |
| Frontend | http://localhost:3000 |
| API | http://localhost:8000 |
| Interactive API docs | http://localhost:8000/docs |
| Mailcatcher | http://localhost:1080 |
| MinIO console (if enabled) | http://localhost:9001 |
| OTLP gRPC collector (if enabled) | http://localhost:4317 |

The first start can take a minute while images build and Postgres becomes ready. Follow progress
with:

```bash
just logs
just logs backend   # one service
```

Stop without deleting data with `just down`. Wipe the database volume with `just reset`.

Sign in with `FIRST_SUPERUSER` and `FIRST_SUPERUSER_PASSWORD` from `.env`.

## Mailcatcher

Locally, SMTP points at Mailcatcher. Every message the API or worker would send is captured
instead of delivered. Open the inbox with:

```bash
just mail
```

Use it to exercise password-reset and other email flows without a real provider. With no
`SMTP_HOST` set, the mailer logs instead of sending, so a bare checkout still runs.

## Local development on the host

Compose publishes the same ports the host tools use (`8000` for the API, `3000` for the
frontend, `5432` for Postgres), so you can stop one container and run that process on the
machine against the rest of the stack.

```bash
docker compose stop backend worker
just dev-backend    # cargo watch -x run
just dev-worker     # cargo watch -x "run --bin worker"
```

```bash
docker compose stop frontend
just dev-frontend   # Next.js on http://localhost:3000
```

`DATABASE_URL` in `.env` already points at `localhost` for host-run tools. Compose overrides
it inside the API and worker containers to use the `db` service name.

For the frontend BFF when running `npm run dev`, copy `frontend/.env.local.example` so
`API_URL` is `http://localhost:8000` rather than the Compose service hostname.

## Compose files and `.env`

| File | Role |
| ---- | ---- |
| `compose.yml` | Production-oriented base (images, Traefik labels, healthchecks) |
| `compose.override.yml` | Local ports, Mailcatcher, optional MinIO/OTel, no external Traefik network |
| `.env` | Secrets and configuration injected into containers and loaded by `just` |

After changing `.env`, recreate affected services (`just up` or
`docker compose up -d --force-recreate <service>`).

Do not commit real production secrets. Generate fresh local values with `just secrets`.

## Habits that keep the tree honest

- **After changing an API route or schema**, run `just client`. That writes `openapi.json` (no
  database required) and regenerates everything under `frontend/src/client/`. Never hand-edit
  that directory.
- **After changing any SQL**, run `just prepare`. SQLx compile-time checks use an offline
  cache; CI and Docker builds fail if it is stale.
- **New migrations**: `just migration add_widgets`, then `just migrate`, then `just prepare`.

## Testing

```bash
just test           # backend fmt, clippy, tests; frontend lint, types, build
just test-backend   # needs Postgres up (`just up` or at least the db service)
just test-frontend
just e2e            # Playwright against the full Compose stack
```

Backend integration tests create and drop an isolated database per test so they can run in
parallel. Expect `DATABASE_URL must be set` if nothing is listening on the URL in `.env`.

Playwright needs Chromium once per machine:

```bash
cd frontend && npx playwright install chromium
just e2e
```

## Layout

| Path | Purpose |
| ---- | ------- |
| `backend/` | Axum API, worker binary, migrations, OpenAPI exporter |
| `frontend/` | Next.js App Router app; `src/client/` is generated |
| `frontend/tests/` | Playwright specs |
| `docker/` | Sidecar config (for example the OTel collector) |
| `compose*.yml` | Local and production Compose definitions |
| `.github/workflows/` | CI for the generated project |

Run `just` with no arguments to list every recipe.
