CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE user_balance (
    id      UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    user_id           UUID NOT NULL
        REFERENCES users(id)
        ON DELETE CASCADE,

    free_base_qty     NUMERIC(38,18) NOT NULL DEFAULT 0,
    free_quote_qty    NUMERIC(38,18) NOT NULL DEFAULT 0,

    locked_base_qty   NUMERIC(38,18) NOT NULL DEFAULT 0,
    locked_quote_qty  NUMERIC(38,18) NOT NULL DEFAULT 0,

    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);