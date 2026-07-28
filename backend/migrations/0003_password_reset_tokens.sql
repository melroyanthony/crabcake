create table password_reset_tokens (
    id uuid primary key default gen_random_uuid(),
    user_id uuid not null references users (id) on delete cascade,
    -- As with refresh tokens, only the SHA-256 is stored, so a leaked dump cannot be used to
    -- take over accounts.
    token_hash text not null unique,
    expires_at timestamptz not null,
    -- Set on first use. A reset link works once, so an old email in an inbox is not a spare
    -- key to the account.
    used_at timestamptz,
    created_at timestamptz not null default now()
);

create index password_reset_tokens_user_id_idx on password_reset_tokens (user_id);
