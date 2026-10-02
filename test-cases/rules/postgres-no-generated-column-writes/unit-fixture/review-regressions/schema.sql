CREATE TABLE orders (id bigint, note text);
ALTER TABLE public.orders ADD COLUMN updated_at timestamptz;
CREATE TABLE computed_orders (id bigint);
ALTER TABLE public.computed_orders ADD COLUMN updated_at bigint GENERATED ALWAYS AS (id + 1) STORED;
-- A dot inside a quoted table name is part of its identity.
CREATE TABLE "dotted.orders" (id bigint);
ALTER TABLE public."dotted.orders" ADD COLUMN updated_at timestamptz;
