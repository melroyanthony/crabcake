#!/usr/bin/env bash
# Writes a local .env for CI (and for first-time Compose) when none is present.
# Generated projects gitignore .env, so workflows call this before `docker compose`.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

if [[ -f .env ]]; then
  exit 0
fi

password="$(openssl rand -hex 24)"
secret="$(openssl rand -hex 32)"
storage="$(openssl rand -hex 24)"
name="${CI_PROJECT_NAME:-ci}"

cat >.env <<EOF
ENVIRONMENT=local
PROJECT_NAME="CI"
STACK_NAME=${name}
DOMAIN=localhost
FRONTEND_HOST=http://localhost:3000
BACKEND_HOST=http://localhost:8000
CORS_ORIGINS=http://localhost:3000,http://localhost:8000
BIND_ADDRESS=0.0.0.0:8000
RUST_LOG=info,app=debug,tower_http=debug,sqlx=warn
SECRET_KEY=${secret}
ACCESS_TOKEN_EXPIRE_MINUTES=30
REFRESH_TOKEN_EXPIRE_DAYS=30
PASSWORD_RESET_TOKEN_EXPIRE_HOURS=1
RATE_LIMIT_PER_SECOND=0
RATE_LIMIT_BURST=50
FIRST_SUPERUSER=admin@example.com
FIRST_SUPERUSER_PASSWORD=changethis
POSTGRES_SERVER=db
POSTGRES_PORT=5432
POSTGRES_DB=app
POSTGRES_USER=postgres
POSTGRES_PASSWORD=${password}
DATABASE_URL=postgres://postgres:${password}@localhost:5432/app
SMTP_HOST=mailcatcher
SMTP_PORT=1025
SMTP_USER=
SMTP_PASSWORD=
SMTP_TLS=false
EMAILS_FROM_NAME="CI"
EMAILS_FROM_EMAIL=info@example.com
S3_ENDPOINT=http://localhost:9000
S3_REGION=us-east-1
S3_BUCKET=${name}-uploads
S3_ACCESS_KEY_ID=${name}
S3_SECRET_ACCESS_KEY=${storage}
S3_FORCE_PATH_STYLE=true
UPLOAD_URL_EXPIRE_SECONDS=900
OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector:4317
OTEL_SERVICE_NAME=${name}-api
METRICS_ENABLED=true
METRICS_BIND_ADDRESS=0.0.0.0:9100
SENTRY_DSN=
NEXT_PUBLIC_APP_NAME="CI"
API_URL=http://backend:8000
DOCKER_IMAGE_BACKEND=${name}-backend
DOCKER_IMAGE_FRONTEND=${name}-frontend
TAG=test
EOF

echo "wrote .env for CI"
