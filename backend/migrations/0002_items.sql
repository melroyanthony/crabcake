create table items (
    id uuid primary key default gen_random_uuid(),
    owner_id uuid not null references users (id) on delete cascade,
    title text not null,
    description text,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

-- Listing a user's items is the most common query, and it is always newest first.
create index items_owner_id_created_at_idx on items (owner_id, created_at desc);
