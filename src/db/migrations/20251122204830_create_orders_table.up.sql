CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE orders (
    id      UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id           UUID NOT NULL
        REFERENCES users(id)
        ON DELETE CASCADE,

    price NUMERIC(38,18) NOT NULL,
    quantity NUMERIC(38,18) NOT NULL,
    filled_quantity NUMERIC(38,18) NOT NULL DEFAULT 0,

    side VARCHAR(10) NOT NULL CHECK (side IN ('Bid', 'Ask')),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);