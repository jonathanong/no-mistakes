SET search_path = public;
CREATE TABLE public.orders (id int, total int GENERATED ALWAYS AS (id) STORED);
-- No-op: `orders` resolves to the live public.orders.
CREATE TABLE IF NOT EXISTS orders (id int);
