-- Add migration script here
create table users (
    id            UUID         primary key default gen_random_uuid(),
    email         VARCHAR(255) unique      not null,
    password_hash TEXT         not null,
    is_verified   BOOLEAN      not null    default false,
    created_at    TIMESTAMPTZ  not null    default now()
);
