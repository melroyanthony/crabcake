# Deployment

Deploy with Docker Compose on a server that runs Docker Engine. The base file is
`compose.yml`. Local-only overrides in `compose.override.yml` are not used in production.

If Traefik was included at generation time (`compose.traefik.yml`), TLS and routing use Let's
Encrypt with hostnames driven by `DOMAIN` in `.env`. Otherwise put your own reverse proxy in
front of the `backend` (port 8000) and `frontend` (port 3000) services and set `CORS_ORIGINS`,
`FRONTEND_HOST`, and `BACKEND_HOST` to the public URLs.

This repository ships CI that tests the stack (backend, frontend, Compose, Playwright). It does
not ship opinionated continuous-deployment workflows; wire those to your host or platform once
the manual path below works.

## Preparation

1. A Linux server with [Docker Engine](https://docs.docker.com/engine/install/) installed
   (not Docker Desktop).
2. DNS for your domain pointing at that server.
3. A wildcard (or explicit records) for the subdomains you will use, for example
   `api.example.com` and `dashboard.example.com`, plus `traefik.example.com` if you expose the
   Traefik dashboard.

## Traefik and the public network

When `compose.traefik.yml` is present, `compose.yml` expects an external Docker network named
`traefik-public`. Create it once on the server:

```bash
docker network create traefik-public
```

Bring the app up with Traefik in the same Compose project:

```bash
docker compose -f compose.yml -f compose.traefik.yml up -d --build
```

Traefik needs these environment variables in the shell (or a `.env` next to the Compose files):

| Variable | Purpose |
| -------- | ------- |
| `DOMAIN` | Base domain; routes use `api.$DOMAIN` and `dashboard.$DOMAIN` |
| `EMAIL` | Let's Encrypt account email (a real address, not `@example.com`) |
| `USERNAME` | HTTP basic-auth user for the Traefik dashboard |
| `HASHED_PASSWORD` | `openssl passwd -apr1` hash of the dashboard password |
| `STACK_NAME` | Prefix for Traefik router/service names (must be unique per stack on the host) |

Example:

```bash
export DOMAIN=example.com
export EMAIL=admin@example.com
export USERNAME=admin
export PASSWORD='choose-a-long-password'
export HASHED_PASSWORD="$(openssl passwd -apr1 "$PASSWORD")"
```

The Traefik dashboard is then at `https://traefik.$DOMAIN`, protected by that basic auth.

If several apps share one host, keep a single Traefik on `traefik-public` and attach each
stack to that network. Adjust labels and `STACK_NAME` so router names do not collide.

## Copy the project

From your machine:

```bash
rsync -av --filter=":- .gitignore" ./ user@your-server:/opt/app/
```

`--filter=":- .gitignore"` skips the same paths Git ignores (`target/`, `node_modules/`, and so
on).

## Environment

On the server, maintain a `.env` (or inject the same keys from a secret store). Start from the
generated `.env.example` and change at least:

| Variable | Notes |
| -------- | ----- |
| `ENVIRONMENT` | `staging` or `production` — not `local` |
| `DOMAIN` | Your real domain |
| `FRONTEND_HOST` / `BACKEND_HOST` | Public `https://…` URLs |
| `CORS_ORIGINS` | Include the dashboard origin |
| `SECRET_KEY` | Never leave `changethis`; `just secrets` or `openssl rand -hex 32` |
| `POSTGRES_PASSWORD` | Same — the API refuses placeholder secrets outside `local` |
| `FIRST_SUPERUSER_PASSWORD` | Rotate from the value chosen at generation time |
| `SMTP_*` / `EMAILS_FROM_*` | Real provider credentials for outbound mail |
| `DOCKER_IMAGE_*` / `TAG` | Image names used by Compose `image:` fields |
| `STACK_NAME` | Unique Compose/Traefik identity for this environment |

If you use S3-compatible uploads, point `S3_*` at your bucket. On AWS, leave
`S3_ACCESS_KEY_ID` and `S3_SECRET_ACCESS_KEY` empty and use an instance or task role; clear
`S3_ENDPOINT` and set `S3_FORCE_PATH_STYLE=false` for virtual-hosted–style URLs. Clearing
`S3_BUCKET` disables upload endpoints (`501`) without removing code.

For tracing, set `OTEL_EXPORTER_OTLP_ENDPOINT` to your collector. Set `METRICS_ENABLED=false`
(or do not publish `METRICS_BIND_ADDRESS`) if Prometheus scrape should not be exposed on the
host.

Email without `SMTP_HOST` is logged only — fine for a smoke deploy, not for password reset in
production.

## Bring the stack up

Omit `compose.override.yml` so Mailcatcher and published localhost ports stay off:

```bash
cd /opt/app
# With Traefik:
docker compose -f compose.yml -f compose.traefik.yml build
docker compose -f compose.yml -f compose.traefik.yml up -d

# Without Traefik (own reverse proxy):
# docker compose -f compose.yml build
# docker compose -f compose.yml up -d
```

Confirm health with:

```bash
docker compose -f compose.yml -f compose.traefik.yml ps
curl -fsS "https://api.${DOMAIN}/health/ready"
```

## URLs

Replace `example.com` with your `DOMAIN`.

| What | URL |
| ---- | --- |
| Dashboard | `https://dashboard.example.com` |
| API | `https://api.example.com` |
| API docs | `https://api.example.com/docs` |
| Traefik dashboard | `https://traefik.example.com` |

## Staging vs production

Use a second directory (or host), a distinct `STACK_NAME`, and either a subdomain
(`staging.example.com` with `api.staging…` / `dashboard.staging…`) or a separate domain. Keep
separate Postgres volumes and secrets per environment.

## Security checklist

- Rotate `SECRET_KEY` and `POSTGRES_PASSWORD` (and any generated MinIO/S3 keys) before the
  first public deploy.
- Prefer platform secret injection over a committed `.env` on a shared server.
- Change `FIRST_SUPERUSER_PASSWORD`.
- Restrict who can reach Postgres, metrics, and the Traefik dashboard; only `80`/`443` need to
  be public for the app itself.
