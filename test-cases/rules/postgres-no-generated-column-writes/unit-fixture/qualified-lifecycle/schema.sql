CREATE TABLE public.orders (id int, generated int GENERATED ALWAYS AS (id + 1) STORED);
CREATE TABLE audit.orders (id int, generated int);
DROP TABLE audit.orders;
CREATE TABLE audit.orders (id int, generated int);
