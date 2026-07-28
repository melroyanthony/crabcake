set dotenv-load := true
set shell := ["bash", "-uc"]

# List available recipes
default:
    @just --list

# Start the whole stack in the background
up:
    docker compose up -d --wait

# Stop the stack, keeping volumes
down:
    docker compose down

# Stop the stack and delete the database volume
reset:
    docker compose down -v

# Tail logs for all services, or one: just logs backend
logs service="":
    docker compose logs -f {{ service }}

# Run the API with auto-reload against the Compose database
dev-backend:
    cd backend && cargo watch -x run

# Run the background worker with auto-reload
dev-worker:
    cd backend && cargo watch -x "run --bin worker"

# Run the Next.js dev server
dev-frontend:
    cd frontend && npm run dev

# Apply pending database migrations
migrate:
    cd backend && sqlx migrate run

# Create a new migration: just migration add_widgets
migration name:
    cd backend && sqlx migrate add -r {{ name }}

# Refresh the offline query cache, required after changing any SQL
prepare:
    cd backend && cargo sqlx prepare -- --all-targets

# Write the OpenAPI spec to openapi.json. Needs no database and no running server
spec:
    cd backend && cargo run --quiet --bin openapi -- ../openapi.json

# Open the interactive API documentation
docs:
    open http://localhost:8000/docs

# Open the local inbox, where every email the stack sends lands
mail:
    open http://localhost:1080

# Regenerate the frontend API client from the backend spec
client: spec
    cd frontend && npm run generate-client

# Run every check the CI runs
test: test-backend test-frontend

# Backend formatting, lints and tests. Needs the database up: each test makes its own
test-backend:
    cd backend && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

# Frontend lints, type check and build
test-frontend:
    cd frontend && npm run lint && npx tsc --noEmit && npm run build

# Playwright end-to-end tests against the Compose stack
e2e: up
    cd frontend && npx playwright test

# Format everything
fmt:
    cd backend && cargo fmt
    cd frontend && npm run format

# Generate fresh secrets for .env
secrets:
    @echo "SECRET_KEY=$(openssl rand -hex 32)"
    @echo "POSTGRES_PASSWORD=$(openssl rand -hex 24)"
