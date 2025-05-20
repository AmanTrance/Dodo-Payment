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
    created_at TIMESTAMP DEFAULT NOW(),
    is_default BOOLEAN DEFAULT FALSE,
    created_by TEXT NOT NULL,
    CONSTRAINT fk_created_by_upis FOREIGN KEY (created_by) REFERENCES public.users (id) ON DELETE CASCADE
);

CREATE TABLE public.transactions (
    id SERIAL PRIMARY KEY,
    tx_time TIMESTAMP DEFAULT NOW(), 
    user_id TEXT NOT NULL,
    from_user TEXT NULL,
    to_user TEXT NULL,
    amount NUMERIC NOT NULL,
    is_external BOOLEAN DEFAULT FALSE,
    tx_status TEXT NOT NULL,
    CONSTRAINT fk_user_transactions FOREIGN KEY (user_id) REFERENCES public.users (id) ON DELETE CASCADE,
    CONSTRAINT fk_from_user_transactions FOREIGN KEY (from_user) REFERENCES public.users (id) ON DELETE CASCADE,
    CONSTRAINT fk_to_user_transactions FOREIGN KEY (to_user) REFERENCES public.users (id) ON DELETE CASCADE
);

-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
-- +goose StatementEnd