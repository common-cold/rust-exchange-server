CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE orders (
    id      UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id           UUID NOT NULL
        REFERENCES users(id)
        ON DELETE CASCADE,

    order_type VARCHAR(10) NOT NULL CHECK (order_type IN ('Limit', 'Market')),
    price NUMERIC(38,18) NOT NULL,
    quantity NUMERIC(38,18) NOT NULL,
    filled_quantity NUMERIC(38,18) NOT NULL DEFAULT 0,

    side VARCHAR(10) NOT NULL CHECK (side IN ('Bid', 'Ask')),
    status VARCHAR(10) NOT NULL CHECK (status IN ('Open', 'Close', 'Cancelled')),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);