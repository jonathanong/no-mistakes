CREATE TABLE public.orders (id int, computed int GENERATED ALWAYS AS (id + 1) STORED);
CREATE TEMP TABLE orders (id int, computed int);
CREATE TABLE audit.unique_orders (id int, computed int);
CREATE TABLE audit.same (id int, computed int);
CREATE TABLE public.same (id int, computed int);
CREATE TABLE "public.orders" (id int, computed int);
