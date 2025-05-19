-- +goose Up
-- +goose StatementBegin

CREATE SCHEMA IF NOT EXISTS public;

CREATE TABLE public.users (
    id TEXT PRIMARY KEY,
    created_at TIMESTAMP DEFAULT NOW(),
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password TEXT NULL,
    city TEXT NULL,
    state TEXT NULL,
    country TEXT NULL,
    avatar TEXT NULL
);

CREATE TABLE public.upis (
    upi_id TEXT PRIMARY KEY,
    created_by TEXT NOT NULL,
    CONSTRAINT fk_created_by_upis FOREIGN KEY (created_by) REFERENCES public.users (id) ON DELETE CASCADE
);

-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
-- +goose StatementEnd