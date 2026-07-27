create table users (
    id uuid primary key default gen_random_uuid(),
    email text not null,
    hashed_password text not null,
    full_name text,
    is_active boolean not null default true,
    is_superuser boolean not null default false,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

-- Indexed on the lowercased address rather than using citext, which needs an extension that
-- some managed providers do not allow. Addresses are stored as the user typed them.
create unique index users_email_key on users (lower(email));

create table refresh_tokens (
    id uuid primary key default gen_random_uuid(),
    user_id uuid not null references users (id) on delete cascade,
    -- Only the SHA-256 of the token is stored, so a leaked database dump cannot be replayed
    -- as a set of live sessions.
    token_hash text not null unique,
    expires_at timestamptz not null,
    revoked_at timestamptz,
    created_at timestamptz not null default now()
);

create index refresh_tokens_user_id_idx on refresh_tokens (user_id);
