CREATE TABLE public.orders (id int, computed int GENERATED ALWAYS AS (id + 1) STORED);
CREATE TEMP TABLE orders (id int, computed int);
CREATE TABLE public.reverse_orders (id int, computed int);
CREATE TEMP TABLE pg_temp.reverse_orders (id int, computed int GENERATED ALWAYS AS (id + 1) STORED);
