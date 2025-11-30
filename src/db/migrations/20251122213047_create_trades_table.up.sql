CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE trades (
    id UUID NOT NULL DEFAULT uuid_generate_v4(),

    buy_order_id  UUID NOT NULL,
    sell_order_id UUID NOT NULL,

    price    NUMERIC(38,18) NOT NULL,
    quantity NUMERIC(38,18) NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

SELECT create_hypertable('trades', 'created_at');